//! This module contains common structs for the VPK format.

use crate::common::file::VPKFileReader;
#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom};

/// The terminator sequence (2 bytes) for a [`VPKDirectoryEntry`].
pub const VPK_ENTRY_TERMINATOR: u16 = 0xFFFF;

/// Trait for common methods on the various directory entry formats used in versions of VPK files.
pub trait DirEntry {
    /// Reads a directory entry from a file.
    fn from(file: &mut File) -> Result<Self, String>
    where
        Self: Sized;
    /// Returns the number of bytes of preload data for an entry, this is 0 if all the data is stored in archives.
    fn get_preload_bytes(self: &Self) -> usize;
}

/// The file tree parsed from a VPK directory files.
pub struct VPKTree<DirectoryEntry>
where
    DirectoryEntry: DirEntry,
{
    /// A map pointing every file described in the directory tree to its corresponding entry.
    pub files: HashMap<String, DirectoryEntry>,
    /// A map pointing every file with preload data to its preload data. A path will only be a valid key if the file at that path has a non-zero amount of preload data.
    pub preload: HashMap<String, Vec<u8>>,
}

impl<DirectoryEntry> VPKTree<DirectoryEntry>
where
    DirectoryEntry: DirEntry,
{
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            preload: HashMap::new(),
        }
    }

    pub fn from(file: &mut File, start: u64, size: u64) -> Result<Self, String> {
        file.seek(SeekFrom::Start(start))
            .or(Err("Could not seek to start of tree"))?;

        let mut tree = Self::new();

        while file.stream_position().unwrap() < start + size {
            let extension = file.read_string().or(Err("Failed to read extension"))?;
            if extension.len() == 0 {
                break;
            }

            loop {
                let path = file.read_string().or(Err("Failed to read path"))?;
                if path.len() == 0 || file.stream_position().unwrap() > start + size {
                    break;
                }

                loop {
                    let file_name = file.read_string().or(Err("Failed to read file name"))?;
                    if file_name.len() == 0 || file.stream_position().unwrap() > start + size {
                        break;
                    }

                    let file_path = format!("{}/{}.{}", path, file_name, extension);

                    let entry = DirectoryEntry::from(file)?;

                    if entry.get_preload_bytes() > 0 {
                        tree.preload.insert(
                            file_path.clone(),
                            file.read_bytes(entry.get_preload_bytes())
                                .or(Err("Failed to read preload data"))?,
                        );
                    }

                    tree.files.insert(file_path, entry);
                }
            }
        }

        Ok(tree)
    }
}

/// The entry format used by VPK version 1 and VPK version 2. For the format used by Respawn VPKs see [`VPKDirectoryRespawn`](crate::pak::revpk::format::VPKDirectoryEntryRespawn).
#[derive(Debug, PartialEq, Eq)]
pub struct VPKDirectoryEntry {
    /// A 32bit CRC of the file's data. Uses the CRC32 ISO HDLC algorithm.
    pub crc: u32,
    /// The number of preload bytes contained in the directory file.
    pub preload_bytes: u16,

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

impl VPKDirectoryEntry {
    pub fn new() -> Self {
        Self {
            crc: 0,
            preload_bytes: 0,
            archive_index: 0,
            entry_offset: 0,
            entry_length: 0,
            terminator: VPK_ENTRY_TERMINATOR,
        }
    }
}

impl DirEntry for VPKDirectoryEntry {
    fn from(file: &mut File) -> Result<Self, String> {
        let crc = file.read_u32().or(Err("Failed to read CRC"))?;
        let preload_bytes = file.read_u16().or(Err("Failed to read preload bytes"))?;
        let archive_index = file.read_u16().or(Err("Failed to read archive index"))?;
        let entry_offset = file.read_u32().or(Err("Failed to read entry offset"))?;
        let entry_length = file.read_u32().or(Err("Failed to read entry length"))?;
        let terminator = file.read_u16().or(Err("Failed to read terminator"))?;

        if terminator != VPK_ENTRY_TERMINATOR {
            return Err(String::from("VPK entry terminator should be 0xFFFF"));
        }

        Ok(Self {
            crc,
            preload_bytes,
            archive_index,
            entry_offset,
            entry_length,
            terminator,
        })
    }

    fn get_preload_bytes(self: &Self) -> usize {
        self.preload_bytes as _
    }
}

/// Trait for reading VPK files.
pub trait PakReader {
    /// Create an empty readable VPK which can then be constructed programmatically.
    fn new() -> Self;
    /// Create a readable VPK from a directory file.
    fn from_file(file: &mut File) -> Result<Self, String>
    where
        Self: Sized;

    /// Read the contents of a file stored in the VPK into memory.
    fn read_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
    ) -> Option<Vec<u8>>;

    /// Extract the contents of a file stored in the VPK to a file system location.
    fn extract_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<(), String>;

    /// Extract the contents of a file stored in the VPK to a file system location using memory-mapped files.
    /// Memory mapped files for every archive used in the extraction must be provided.
    #[cfg(feature = "mem-map")]
    fn extract_file_mem_map(
        self: &Self,
        archive_path: &String,
        archive_mmaps: &HashMap<u16, FileBuffer>,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<(), String>;
}
