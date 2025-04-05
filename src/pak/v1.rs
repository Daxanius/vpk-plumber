//! Support for the VPK version 1 format.

use super::{Error, PakReader, PakWorker, PakWriter, Result, VPKDirectoryEntry, VPKTree};
use crate::util::file::{VPKFileReader, VPKFileWriter};
use crc::{CRC_32_ISO_HDLC, Crc};
use std::cmp::min;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::mem;
use std::path::Path;

#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;

#[cfg(feature = "mem-map")]
use std::collections::HashMap;

/// The 4-byte signature found in the header of a valid VPK version 1 file.
pub const VPK_SIGNATURE_V1: u32 = 0x55AA_1234;
/// The 4-byte version found in the header of a valid VPK version 1 file.
pub const VPK_VERSION_V1: u32 = 1;

/// The header of a VPK version 1 file.
#[derive(PartialEq, Eq)]
pub struct VPKHeaderV1 {
    /// VPK signature. Should be equal to [`VPK_SIGNATURE_V1`].
    pub signature: u32,
    /// VPK version. Should be equal to [`VPK_VERSION_V1`].
    pub version: u32,

    /// Size of the directory tree in bytes.
    pub tree_size: u32,
}

impl VPKHeaderV1 {
    /// Read the header from a file.
    /// # Errors
    /// - When the data is invalid
    /// - When the signature is invalid
    /// - When the version does not match
    pub fn from(file: &mut File) -> Result<Self> {
        let signature = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read signature".to_string(),
        })?;

        // Check the signature before moving on
        if signature != VPK_SIGNATURE_V1 {
            return Err(Error::InvalidSignature(format!(
                "Header signature should be {VPK_SIGNATURE_V1:#X} but is {signature:#X}"
            )));
        }

        let version = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read version".to_string(),
        })?;

        // Check the version before moving on
        if version != VPK_VERSION_V1 {
            return Err(Error::BadVersion(format!(
                "Header version should be {VPK_VERSION_V1} but is {version}"
            )));
        }

        let tree_size = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read tree size".to_string(),
        })?;

        Ok(Self {
            signature,
            version,
            tree_size,
        })
    }

    /// Write the header to a file.
    /// # Errors
    /// - When the data is invalid
    /// - When the signature is invalid
    /// - When the version does not match
    pub fn write(&self, file: &mut File) -> Result<()> {
        if self.signature != VPK_SIGNATURE_V1 {
            return Err(Error::InvalidSignature(format!(
                "Header signature should be {VPK_SIGNATURE_V1:#X} but is {:#X}",
                self.signature
            )));
        }

        if self.version != VPK_VERSION_V1 {
            return Err(Error::BadVersion(format!(
                "Header version should be {VPK_VERSION_V1} but is {}",
                self.version
            )));
        }

        file.write_u32(self.signature).map_err(|e| Error::Util {
            source: e,
            context: "Failed to write signature".to_string(),
        })?;

        file.write_u32(self.version).map_err(|e| Error::Util {
            source: e,
            context: "Failed to write version".to_string(),
        })?;

        file.write_u32(self.tree_size).map_err(|e| Error::Util {
            source: e,
            context: "Failed to write tree size".to_string(),
        })?;

        Ok(())
    }

    /// Check if a file is in the VPK version 1 format.
    pub fn is_format(file: &mut File) -> bool {
        let Ok(pos) = file.stream_position() else {
            return false;
        };

        let signature = file.read_u32();
        let version = file.read_u32();

        let _ = file.seek(std::io::SeekFrom::Start(pos));

        signature.unwrap_or(0) == VPK_SIGNATURE_V1 && version.unwrap_or(0) == VPK_VERSION_V1
    }
}

/// The VPK version 1 format.
#[derive(PartialEq, Eq)]
pub struct VPKVersion1 {
    /// The VPK's header.
    pub header: VPKHeaderV1,
    /// The tree of files in the VPK.
    pub tree: VPKTree<VPKDirectoryEntry>,
}

impl PakReader for VPKVersion1 {
    fn read_file(
        &self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
    ) -> Option<Vec<u8>> {
        let entry = self.tree.files.get(file_path)?;
        let mut buf: Vec<u8> = Vec::new();

        if entry.preload_length > 0 {
            buf.append(self.tree.preload.get(file_path)?.clone().as_mut());
        }

        if entry.entry_length > 0 {
            let mut archive_file = if entry.archive_index == 0xFF7F {
                let path = Path::new(archive_path).join(format!("{vpk_name}_dir.vpk"));

                let mut archive_file = File::open(path).ok()?;
                let _ = archive_file.seek(SeekFrom::Start(
                    mem::size_of::<VPKHeaderV1>() as u64
                        + u64::from(self.header.tree_size)
                        + u64::from(entry.entry_offset),
                ));
                archive_file
            } else {
                let path = Path::new(archive_path).join(format!(
                    "{}_{:0>3}.vpk",
                    vpk_name,
                    entry.archive_index.to_string()
                ));

                let mut archive_file = File::open(path).ok()?;
                let _ = archive_file.seek(SeekFrom::Start(entry.entry_offset.into()));
                archive_file
            };

            buf.append(
                archive_file
                    .read_bytes(entry.entry_length as _)
                    .ok()?
                    .as_mut(),
            );
        }

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();
        digest.update(&buf);

        if digest.finalize() == entry.crc {
            Some(buf)
        } else {
            None
        }
    }

    fn extract_file(
        &self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<()> {
        let entry = self
            .tree
            .files
            .get(file_path)
            .ok_or(Error::FileNotFound(file_path.to_string()))?;

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        let out_path = std::path::Path::new(output_path);
        if let Some(prefix) = out_path.parent() {
            std::fs::create_dir_all(prefix).map_err(Error::Io)?;
        }

        let mut out_file = File::create(out_path).map_err(Error::Io)?;

        // Set the length of the file
        out_file
            .set_len(entry.entry_length.into())
            .map_err(Error::Io)?;

        if entry.preload_length > 0 {
            let chunk = self
                .tree
                .preload
                .get(file_path)
                .ok_or(Error::DataNotFound(file_path.to_string()))?;

            out_file.write_all(chunk).map_err(Error::Io)?;

            digest.update(chunk);
        }

        if entry.entry_length > 0 {
            let mut archive_file = if entry.archive_index == 0xFF7F {
                let path = Path::new(archive_path).join(format!("{vpk_name}_dir.vpk"));

                let mut archive_file = File::open(path).map_err(Error::Io)?;
                let _ = archive_file.seek(SeekFrom::Start(
                    mem::size_of::<VPKHeaderV1>() as u64
                        + u64::from(self.header.tree_size)
                        + u64::from(entry.entry_offset),
                ));
                archive_file
            } else {
                let path = Path::new(archive_path).join(format!(
                    "{}_{:0>3}.vpk",
                    vpk_name,
                    entry.archive_index.to_string()
                ));

                let mut archive_file = File::open(path).map_err(Error::Io)?;
                let _ = archive_file.seek(SeekFrom::Start(entry.entry_offset.into()));
                archive_file
            };

            // read chunks of 1MB max into buffer and write to the output file
            let mut remaining = entry.entry_length as usize;
            while remaining > 0 {
                let chunk = archive_file
                    .read_bytes(min(1024 * 1024, remaining))
                    .map_err(|e| Error::Util {
                        source: e,
                        context: "Failed to read archive section".to_string(),
                    })?;

                if chunk.is_empty() {
                    return Err(Error::BadData("Archive is empty".to_string()));
                }

                out_file.write_all(&chunk).map_err(Error::Io)?;

                if remaining >= chunk.len() {
                    remaining -= chunk.len();
                } else {
                    remaining = 0;
                }

                digest.update(&chunk);
            }
        }

        if digest.finalize() == entry.crc {
            Ok(())
        } else {
            Err(Error::BadData("CRC must match".to_string()))
        }
    }

    #[cfg(feature = "mem-map")]
    fn extract_file_mem_map(
        &self,
        _archive_path: &String,
        archive_mmaps: &HashMap<u16, FileBuffer>,
        _vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<()> {
        let entry = self
            .tree
            .files
            .get(file_path)
            .ok_or(Error::FileNotFound(file_path.to_string()))?;

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        let out_path = std::path::Path::new(output_path);
        if let Some(prefix) = out_path.parent() {
            std::fs::create_dir_all(prefix).map_err(Error::Io)?;
        }

        let mut out_file = File::create(out_path).map_err(Error::Io)?;

        // Set the length of the file
        out_file
            .set_len(entry.entry_length.into())
            .map_err(Error::Io)?;

        if entry.preload_length > 0 {
            let chunk = self
                .tree
                .preload
                .get(file_path)
                .ok_or(Error::DataNotFound(file_path.to_string()))?;

            out_file.write_all(chunk).map_err(Error::Io)?;

            digest.update(chunk);
        }

        if entry.entry_length > 0 {
            let archive_file = archive_mmaps
                .get(&entry.archive_index)
                .ok_or(Error::MemoryMappedFileNotFound(entry.archive_index))?;

            // read chunks of 1MB max into buffer and write to the output file
            let mut remaining = entry.entry_length as usize;
            let mut i = entry.entry_offset as usize;
            while remaining > 0 {
                let chunk = &archive_file[i..(i + min(1024 * 1024, remaining))];

                if chunk.is_empty() {
                    return Err(Error::BadData("Archive is empty".to_string()));
                }

                out_file.write_all(chunk).map_err(Error::Io)?;

                i += chunk.len();

                if remaining >= chunk.len() {
                    remaining -= chunk.len();
                } else {
                    remaining = 0;
                }

                digest.update(chunk);
            }
        }

        if digest.finalize() == entry.crc {
            Ok(())
        } else {
            Err(Error::BadData("CRC must match".to_string()))
        }
    }
}

impl PakWriter for VPKVersion1 {
    fn write_dir(&self, output_path: &str) -> Result<()> {
        let out_path = std::path::Path::new(output_path);
        if let Some(prefix) = out_path.parent() {
            std::fs::create_dir_all(prefix).map_err(Error::Io)?;
        }

        let mut out_file = File::create(out_path).map_err(Error::Io)?;

        self.header.write(&mut out_file)?;
        self.tree.write(&mut out_file)?;

        Ok(())
    }
}

impl PakWorker for VPKVersion1 {
    fn new() -> Self {
        Self {
            header: VPKHeaderV1 {
                signature: VPK_SIGNATURE_V1,
                version: VPK_VERSION_V1,
                tree_size: 0,
            },
            tree: VPKTree::new(),
        }
    }

    fn from_file(file: &mut File) -> Result<Self> {
        let header = VPKHeaderV1::from(file)?;

        let tree_start = file.stream_position().unwrap();
        let tree = VPKTree::from(file, tree_start, header.tree_size.into())?;

        Ok(Self { header, tree })
    }
}

impl TryFrom<&mut File> for VPKVersion1 {
    fn try_from(file: &mut File) -> Result<Self> {
        Self::from_file(file)
    }

    type Error = Error;
}
