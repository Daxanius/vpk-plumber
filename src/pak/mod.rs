//! Support for various VPK formats with traits to allow for extension.

use crate::util::file::{VPKFileReader, VPKFileWriter};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;

pub use error::{Error, Result};

pub mod v1;
pub mod v2;

#[cfg(feature = "revpk")]
pub mod revpk;

mod error;

/// The terminator sequence (2 bytes) for a [`VPKDirectoryEntry`].
pub const VPK_ENTRY_TERMINATOR: u16 = 0xFFFF;

/// Trait for common methods on the various directory entry formats used in versions of VPK files.
pub trait DirEntry {
    /// Reads a directory entry from a file.
    /// # Errors
    /// - When an IO operation fails
    /// - When the file contains invalid data
    fn from(file: &mut File) -> Result<Self>
    where
        Self: Sized;

    /// Write the directory entry to a file.
    /// # Errors
    /// When an IO operation fails
    /// When the data is invalid
    fn write(&self, file: &mut File) -> Result<()>;

    /// Returns the number of bytes of preload data for an entry, this is 0 if all the data is stored in archives.
    fn get_preload_length(&self) -> usize;
}

/// The file tree parsed from a VPK directory files.
#[derive(PartialEq, Eq)]
pub struct VPKTree<DirectoryEntry>
where
    DirectoryEntry: DirEntry,
{
    /// A map pointing every file described in the directory tree to its corresponding entry.
    pub files: HashMap<String, DirectoryEntry>,
    /// A map pointing every file with preload data to its preload data. A path will only be a valid key if the file at that path has a non-zero amount of preload data.
    pub preload: HashMap<String, Vec<u8>>,
}

impl<DirectoryEntry> Default for VPKTree<DirectoryEntry>
where
    DirectoryEntry: DirEntry,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<DirectoryEntry> VPKTree<DirectoryEntry>
where
    DirectoryEntry: DirEntry,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            preload: HashMap::new(),
        }
    }

    /// Reads from a file
    /// # Errors
    /// - When the data is invalid
    /// - When IO operations fail
    pub fn from(file: &mut File, start: u64, size: u64) -> Result<Self> {
        file.seek(SeekFrom::Start(start))
            .map_err(Error::TreeNotFound)?;

        let mut tree = Self::new();

        while file.stream_position().map_err(Error::Io)? < start + size {
            let extension = file.read_string().map_err(|e| Error::Util {
                source: e,
                context: "Failed to read extension".to_string(),
            })?;

            if extension.is_empty() {
                break;
            }

            loop {
                let path = file.read_string().map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to path".to_string(),
                })?;

                if path.is_empty() || file.stream_position().map_err(Error::Io)? > start + size {
                    break;
                }

                loop {
                    let file_name = file.read_string().map_err(|e| Error::Util {
                        source: e,
                        context: "Failed to read file name".to_string(),
                    })?;

                    if file_name.is_empty()
                        || file.stream_position().map_err(Error::Io)? > start + size
                    {
                        break;
                    }

                    let file_path = format!("{path}/{file_name}.{extension}");

                    let entry = DirectoryEntry::from(file)?;

                    if entry.get_preload_length() > 0 {
                        tree.preload.insert(
                            file_path.clone(),
                            file.read_bytes(entry.get_preload_length()).map_err(|e| {
                                Error::Util {
                                    source: e,
                                    context: "Failed to read preload data".to_string(),
                                }
                            })?,
                        );
                    }

                    tree.files.insert(file_path, entry);
                }
            }
        }

        Ok(tree)
    }

    /// Write q file
    /// # Panics
    /// - Should never panic, if it does, contact the crate author
    /// # Errors
    /// - When an IO operation fails
    pub fn write(&self, file: &mut File) -> Result<()> {
        #[allow(clippy::type_complexity)]
        let mut treeified: HashMap<
            String,
            HashMap<String, Vec<(String, &DirectoryEntry, Option<&Vec<u8>>)>>,
        > = HashMap::new();

        for (path_str, entry) in &self.files {
            let path = Path::new(&path_str);

            let extension = path
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_owned();

            if !treeified.contains_key(&extension) {
                treeified.insert(extension.clone(), HashMap::new());
            }

            let dir = path
                .parent()
                .unwrap_or(Path::new(""))
                .to_str()
                .unwrap_or("/")
                .to_owned();

            let file_name = path
                .file_stem()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_owned();

            let dir_map = treeified.get_mut(&extension).unwrap();

            if !dir_map.contains_key(&dir) {
                dir_map.insert(dir.clone(), Vec::new());
            }

            let preload_bytes: Option<&Vec<u8>> = self.preload.get(path_str);
            dir_map
                .get_mut(&dir)
                .unwrap()
                .push((file_name, entry, preload_bytes));
        }

        for (extension, dir_map) in treeified {
            file.write_string(&extension).map_err(|e| Error::Util {
                source: e,
                context: "Failed to write file extension".to_string(),
            })?;

            for (dir, files) in dir_map {
                file.write_string(&dir).map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to write file directory".to_string(),
                })?;

                for (file_name, entry, preload_bytes) in files {
                    file.write_string(&file_name).map_err(|e| Error::Util {
                        source: e,
                        context: "Failed to write file name".to_string(),
                    })?;

                    entry.write(file)?;

                    if let Some(preload_bytes) = preload_bytes {
                        file.write_bytes(preload_bytes).map_err(|e| Error::Util {
                            source: e,
                            context: "Failed to write preload data".to_string(),
                        })?;
                    }
                }

                file.write_u8(0).map_err(|e| Error::Util {
                    source: e,
                    context: "Error writing separator".to_string(),
                })?;
            }

            file.write_u8(0).map_err(|e| Error::Util {
                source: e,
                context: "Error writing separator".to_string(),
            })?;
        }

        Ok(())
    }
}

/// The entry format used by VPK version 1 and VPK version 2. For the format used by Respawn VPKs see [`VPKDirectoryRespawn`](crate::pak::revpk::format::VPKDirectoryEntryRespawn).
#[derive(Debug, PartialEq, Eq)]
pub struct VPKDirectoryEntry {
    /// A 32bit CRC of the file's data. Uses the CRC32 ISO HDLC algorithm.
    pub crc: u32,

    /// The number of preload bytes contained in the directory file.
    pub preload_length: u16,

    /// A zero based index of the archive this file's data is contained in.
    /// If `0x7FFF` (big-endian), the data follows the directory.
    pub archive_index: u16,

    /// If `archive_index` is `0x7FFF`, the offset of the file data relative to the end of the directory.
    /// Otherwise, the offset of the data from the start of the specified archive.
    pub entry_offset: u32,

    /// If zero, the entire file is stored in the preload data.
    /// Otherwise, the number of bytes stored starting at `entry_offset`.
    pub entry_length: u32,

    /// Entry terminator. Should always be 0xFFFF.
    pub terminator: u16,
}
// Note: If a file contains preload data, the preload data immediately follows the above structure. The entire size of a file is PreloadBytes + EntryLength.

impl Default for VPKDirectoryEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl VPKDirectoryEntry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            crc: 0,
            preload_length: 0,
            archive_index: 0,
            entry_offset: 0,
            entry_length: 0,
            terminator: VPK_ENTRY_TERMINATOR,
        }
    }
}

impl DirEntry for VPKDirectoryEntry {
    fn from(file: &mut File) -> Result<Self> {
        let crc = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read CRC".to_string(),
        })?;

        let preload_length = file.read_u16().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read preload length".to_string(),
        })?;

        let archive_index = file.read_u16().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read archive index".to_string(),
        })?;

        let entry_offset = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read entry offset".to_string(),
        })?;

        let entry_length = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read entry length".to_string(),
        })?;

        let terminator = file.read_u16().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read terminator".to_string(),
        })?;

        if terminator != VPK_ENTRY_TERMINATOR {
            return Err(Error::InvalidEntryTerminator(format!(
                "Should be 0xFFFF but found {terminator:X}"
            )));
        }

        Ok(Self {
            crc,
            preload_length,
            archive_index,
            entry_offset,
            entry_length,
            terminator,
        })
    }

    fn write(&self, file: &mut File) -> Result<()> {
        if self.terminator != VPK_ENTRY_TERMINATOR {
            return Err(Error::InvalidEntryTerminator(format!(
                "Should be 0xFFFF but found {:X}",
                self.terminator,
            )));
        }

        file.write_u32(self.crc).map_err(|e| Error::Util {
            source: e,
            context: "Failed to write CRC".to_string(),
        })?;

        file.write_u16(self.preload_length)
            .map_err(|e| Error::Util {
                source: e,
                context: "Failed to write preload length".to_string(),
            })?;

        file.write_u16(self.archive_index)
            .map_err(|e| Error::Util {
                source: e,
                context: "Failed to write archive index".to_string(),
            })?;

        file.write_u32(self.entry_offset).map_err(|e| Error::Util {
            source: e,
            context: "Failed to write entry offset".to_string(),
        })?;

        file.write_u32(self.entry_length).map_err(|e| Error::Util {
            source: e,
            context: "Failed to write entry length".to_string(),
        })?;

        file.write_u16(self.terminator).map_err(|e| Error::Util {
            source: e,
            context: "Failed to write entry terminator".to_string(),
        })?;

        Ok(())
    }

    fn get_preload_length(&self) -> usize {
        self.preload_length as _
    }
}

/// Trait for reading VPK files.
pub trait PakReader {
    /// Read the contents of a file stored in the VPK into memory.
    fn read_file(
        &self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
    ) -> Option<Vec<u8>>;

    /// Extract the contents of a file stored in the VPK to a file system location.
    fn extract_file(
        &self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<()>;

    /// Extract the contents of a file stored in the VPK to a file system location using memory-mapped files.
    /// Memory mapped files for every archive used in the extraction must be provided.
    #[cfg(feature = "mem-map")]
    fn extract_file_mem_map(
        &self,
        archive_path: &String,
        archive_mmaps: &HashMap<u16, FileBuffer>,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<()>;
}

/// Trait for writing VPK files.
pub trait PakWriter {
    /// Write the dir.vpk file for this VPK to disk with a given path.
    /// Does not modify or create archives if the any [`VPKDirectoryEntry`] has changed.
    fn write_dir(&self, output_path: &str) -> Result<()>;
}

pub trait PakWorker: PakReader + PakWriter {
    /// Create an empty readable VPK which can then be constructed programmatically.
    fn new() -> Self
    where
        Self: Sized;

    /// Create a readable VPK from a directory file.
    /// # Errors
    /// - When the file is in an invalid format
    fn from_file(file: &mut File) -> Result<Self>
    where
        Self: Sized;
}
