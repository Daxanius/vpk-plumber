use crate::common::file::{VPKFile, VPKFileReader};
use crate::common::format::{PakReader, VPKDirectoryEntry, VPKTree};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::cmp::min;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::mem;
use std::path::Path;

#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
#[cfg(feature = "mem-map")]
use std::collections::HashMap;

pub const VPK_SIGNATURE_V1: u32 = 0x55AA1234;
pub const VPK_VERSION_V1: u32 = 1;

pub struct VPKHeaderV1 {
    pub signature: u32,
    pub version: u32,

    // Size of the directory tree in bytes
    pub tree_size: u32,
}

impl VPKHeaderV1 {
    pub fn from(file: &mut VPKFile) -> Result<Self, String> {
        let signature = file
            .read_u32()
            .or(Err("Could not read header signature from file"))?;
        let version = file
            .read_u32()
            .or(Err("Could not read header version from file"))?;
        let tree_size = file
            .read_u32()
            .or(Err("Could not read header tree size from file"))?;

        if signature != VPK_SIGNATURE_V1 {
            return Err(format!(
                "VPK header signature should be {:#x}",
                VPK_SIGNATURE_V1
            ));
        }
        if version != VPK_VERSION_V1 {
            return Err(format!("VPK header version should be {}", VPK_VERSION_V1));
        }

        Ok(Self {
            signature,
            version,
            tree_size,
        })
    }

    pub fn is_format(file: &mut VPKFile) -> bool {
        let pos = file.stream_position().unwrap();

        let signature = file.read_u32();
        let version = file.read_u32();

        let _ = file.seek(std::io::SeekFrom::Start(pos));

        signature.unwrap_or(0) == VPK_SIGNATURE_V1 && version.unwrap_or(0) == VPK_VERSION_V1
    }
}

pub struct VPKVersion1 {
    pub header: VPKHeaderV1,
    pub tree: VPKTree<VPKDirectoryEntry>,
}

impl PakReader for VPKVersion1 {
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

    fn from_file(file: &mut VPKFile) -> Result<Self, String> {
        let header = VPKHeaderV1::from(file)?;

        let tree_start = file.stream_position().unwrap();
        let tree = VPKTree::from(file, tree_start, header.tree_size.into())?;

        Ok(Self { header, tree })
    }

    fn read_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
    ) -> Option<Vec<u8>> {
        let entry = self.tree.files.get(file_path)?;
        let mut buf: Vec<u8> = Vec::new();

        if entry.preload_bytes > 0 {
            buf.append(self.tree.preload.get(file_path)?.clone().as_mut());
        }

        if entry.entry_length > 0 {
            let mut archive_file = if entry.archive_index == 0xFF7F {
                let path = Path::new(archive_path).join(format!("{}_dir.vpk", vpk_name));

                let mut archive_file = File::open(path).ok()?;
                let _ = archive_file.seek(SeekFrom::Start(
                    mem::size_of::<VPKHeaderV1>() as u64
                        + self.header.tree_size as u64
                        + entry.entry_offset as u64,
                ));
                archive_file
            } else {
                let path = Path::new(archive_path).join(format!(
                    "{}_{:0>3}.vpk",
                    vpk_name,
                    entry.archive_index.to_string()
                ));

                let mut archive_file = File::open(path).ok()?;
                let _ = archive_file.seek(SeekFrom::Start(entry.entry_offset as _));
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

        if digest.finalize() != entry.crc {
            None
        } else {
            Some(buf)
        }
    }

    fn extract_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<(), String> {
        let entry = self
            .tree
            .files
            .get(file_path)
            .ok_or("File not found in VPK")?;

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        let out_path = std::path::Path::new(output_path);
        if let Some(prefix) = out_path.parent() {
            std::fs::create_dir_all(prefix).or(Err("Failed to create parent directories"))?;
        };

        let mut out_file = File::create(out_path).or(Err("Failed to create output file"))?;

        // Set the length of the file
        out_file
            .set_len(entry.entry_length as _)
            .or(Err("Failed to set length of output file"))?;

        if entry.preload_bytes > 0 {
            let chunk = self
                .tree
                .preload
                .get(file_path)
                .ok_or("Preload data not found in VPK")?;

            out_file
                .write_all(&chunk)
                .or(Err("Failed to write to output file"))?;

            digest.update(&chunk);
        }

        if entry.entry_length > 0 {
            let mut archive_file = if entry.archive_index == 0xFF7F {
                let path = Path::new(archive_path).join(format!("{}_dir.vpk", vpk_name));

                let mut archive_file = File::open(path).or(Err("Failed to open archive file"))?;
                let _ = archive_file.seek(SeekFrom::Start(
                    mem::size_of::<VPKHeaderV1>() as u64
                        + self.header.tree_size as u64
                        + entry.entry_offset as u64,
                ));
                archive_file
            } else {
                let path = Path::new(archive_path).join(format!(
                    "{}_{:0>3}.vpk",
                    vpk_name,
                    entry.archive_index.to_string()
                ));

                let mut archive_file = File::open(path).or(Err("Failed to open archive file"))?;
                let _ = archive_file.seek(SeekFrom::Start(entry.entry_offset as _));
                archive_file
            };

            // read chunks of 1MB max into buffer and write to the output file
            let mut remaining = entry.entry_length as usize;
            while remaining > 0 {
                let chunk = archive_file
                    .read_bytes(min(1024 * 1024, remaining))
                    .or(Err("Failed to read from archive file"))?;
                if chunk.len() == 0 {
                    return Err("Failed to read from archive file".to_string());
                }
                out_file
                    .write_all(&chunk)
                    .or(Err("Failed to write to output file"))?;

                if remaining >= chunk.len() {
                    remaining -= chunk.len();
                } else {
                    remaining = 0;
                }

                digest.update(&chunk);
            }
        }

        if digest.finalize() != entry.crc {
            Err("CRC must match".to_string())
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "mem-map")]
    fn extract_file_mem_map(
        self: &Self,
        _archive_path: &String,
        archive_mmaps: &HashMap<u16, FileBuffer>,
        _vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<(), String> {
        let entry = self
            .tree
            .files
            .get(file_path)
            .ok_or("File not found in VPK")?;

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        let out_path = std::path::Path::new(output_path);
        if let Some(prefix) = out_path.parent() {
            std::fs::create_dir_all(prefix).or(Err("Failed to create parent directories"))?;
        };

        let mut out_file = File::create(out_path).or(Err("Failed to create output file"))?;

        // Set the length of the file
        out_file
            .set_len(entry.entry_length as _)
            .or(Err("Failed to set length of output file"))?;

        if entry.preload_bytes > 0 {
            let chunk = self
                .tree
                .preload
                .get(file_path)
                .ok_or("Preload data not found in VPK")?;

            out_file
                .write_all(&chunk)
                .or(Err("Failed to write to output file"))?;

            digest.update(&chunk);
        }

        if entry.entry_length > 0 {
            let archive_file = archive_mmaps
                .get(&entry.archive_index)
                .ok_or("Couldn't find memory-mapped file")?;

            // read chunks of 1MB max into buffer and write to the output file
            let mut remaining = entry.entry_length as usize;
            let mut i = entry.entry_offset as usize;
            while remaining > 0 {
                let chunk = &archive_file[i..(i + min(1024 * 1024, remaining))];
                if chunk.len() == 0 {
                    return Err("Failed to read from archive file".to_string());
                }
                out_file
                    .write_all(chunk)
                    .or(Err("Failed to write to output file"))?;

                i += chunk.len();

                if remaining >= chunk.len() {
                    remaining -= chunk.len();
                } else {
                    remaining = 0;
                }

                digest.update(chunk);
            }
        }

        if digest.finalize() != entry.crc {
            Err("CRC must match".to_string())
        } else {
            Ok(())
        }
    }
}

impl TryFrom<&mut VPKFile> for VPKVersion1 {
    fn try_from(file: &mut VPKFile) -> Result<Self, String> {
        Self::from_file(file)
    }

    type Error = String;
}
