use crate::common::file::{VPKFile, VPKFileReader};
use crate::common::format::{DirEntry, PakFormat, VPKTree};
use crate::common::lzham::decompress;
// use crate::common::lzham;
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

pub const VPK_SIGNATURE_REVPK: u32 = 0x55AA1234;
pub const VPK_VERSION_REVPK: u32 = 196610;

pub struct VPKHeaderRespawn {
    pub signature: u32,
    pub version: u32,

    // Size of the directory tree in bytes
    pub tree_size: u32,

    // Should end up as 0, maybe FileDataSectionSize (see https://developer.valvesoftware.com/wiki/VPK_File_Format#VPK_2)
    pub unknown: u32,
}

impl VPKHeaderRespawn {
    pub fn from(file: &mut VPKFile) -> Result<Self, String> {
        let signature = file
            .read_u32()
            .or(Err("Could not read header signature from file"))?;
        let version = file
            .read_u32()
            .or(Err("Could not read header version from file"))?;
        let tree_size = file
            .read_u32()
            .or(Err("Could not read header version from file"))?;
        let unknown = file
            .read_u32()
            .or(Err("Could not read unknown field from file"))?;

        if signature != VPK_SIGNATURE_REVPK {
            return Err(format!(
                "VPK header signature should be {:#x}",
                VPK_SIGNATURE_REVPK
            ));
        }
        if version != VPK_VERSION_REVPK {
            return Err(format!(
                "VPK header version should be {}",
                VPK_VERSION_REVPK
            ));
        }
        if unknown != 0 {
            return Err("VPK header unknown field should be 0".to_string());
        }

        Ok(Self {
            signature,
            version,
            tree_size,
            unknown,
        })
    }

    pub fn is_format(file: &mut VPKFile) -> bool {
        let pos = file.stream_position().unwrap();

        let signature = file.read_u32();
        let version = file.read_u32();

        let _ = file.seek(std::io::SeekFrom::Start(pos));

        signature.unwrap_or(0) == VPK_SIGNATURE_REVPK && version.unwrap_or(0) == VPK_SIGNATURE_REVPK
    }
}

pub enum EPackedLoadFlags {
    LoadNone,
    LoadVisible = 1 << 0,     // FileSystem visibility?
    LoadCache = 1 << 8,       // Only set for assets not stored in the depot directory.
    LoadAcacheUnk0 = 1 << 10, // Acache uses this!!!
    LoadTextureUnk0 = 1 << 18,
    LoadTextureUnk1 = 1 << 19,
    LoadTextureUnk2 = 1 << 20,
}

pub enum EPackedTextureFlags {
    TextureNone,
    TextureDefault = 1 << 3,
    TextureEnvironmentMap = 1 << 10,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VPKDirectoryEntryRespawn {
    pub crc: u32,
    pub preload_bytes: u16,
    pub file_parts: Vec<VPKFilePartEntryRespawn>,
}

impl VPKDirectoryEntryRespawn {
    pub fn new() -> Self {
        Self {
            crc: 0,
            preload_bytes: 0,
            file_parts: Vec::new(),
        }
    }
}

impl DirEntry for VPKDirectoryEntryRespawn {
    fn from(file: &mut VPKFile) -> Result<Self, String> {
        let crc = file.read_u32().or(Err("Failed to read CRC"))?;
        let preload_bytes = file.read_u16().or(Err("Failed to read preload bytes"))?;

        let mut file_parts: Vec<VPKFilePartEntryRespawn> = Vec::new();

        let pos = file.stream_position().unwrap();
        let end = file.seek(SeekFrom::End(0)).unwrap();
        let _ = file.seek(SeekFrom::Start(pos)).unwrap();

        loop {
            let archive_index = file.read_u16().or(Err("Failed reading archive index"))?;
            if archive_index == 0xFFFF || file.stream_position().unwrap() == end {
                break;
            }

            file_parts.push(VPKFilePartEntryRespawn {
                archive_index,
                load_flags: file.read_u16().or(Err("Failed reading load flags"))?,
                texture_flags: file.read_u32().or(Err("Failed reading texture flags"))?,
                entry_offset: file.read_u64().or(Err("Failed reading entry offset"))?,
                entry_length: file.read_u64().or(Err("Failed reading entry length"))?,
                entry_length_uncompressed: file
                    .read_u64()
                    .or(Err("Failed reading uncompressed entry length"))?,
            });
        }

        Ok(Self {
            crc,
            preload_bytes,
            file_parts,
        })
    }

    fn get_preload_bytes(self: &Self) -> usize {
        self.preload_bytes as _
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VPKFilePartEntryRespawn {
    pub archive_index: u16,
    pub load_flags: u16,
    pub texture_flags: u32,
    pub entry_offset: u64,
    pub entry_length: u64,
    pub entry_length_uncompressed: u64,
}

impl VPKFilePartEntryRespawn {
    pub fn new() -> Self {
        Self {
            archive_index: 0,
            load_flags: 0,
            texture_flags: 0,
            entry_offset: 0,
            entry_length: 0,
            entry_length_uncompressed: 0,
        }
    }
}

pub struct VPKRespawn {
    pub header: VPKHeaderRespawn,
    pub tree: VPKTree<VPKDirectoryEntryRespawn>,
}

impl PakFormat for VPKRespawn {
    fn new() -> Self {
        Self {
            header: VPKHeaderRespawn {
                signature: VPK_SIGNATURE_REVPK,
                version: VPK_VERSION_REVPK,
                tree_size: 0,
                unknown: 0,
            },
            tree: VPKTree::new(),
        }
    }

    fn from_file(file: &mut VPKFile) -> Result<Self, String> {
        let header = VPKHeaderRespawn::from(file)?;

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
        let entry: &VPKDirectoryEntryRespawn = self.tree.files.get(file_path)?;
        let mut buf: Vec<u8> = Vec::new();

        if entry.preload_bytes > 0 {
            buf.append(self.tree.preload.get(file_path)?.clone().as_mut());
        }

        for file_part in &entry.file_parts {
            if file_part.entry_length_uncompressed > 0 {
                let path = Path::new(archive_path).join(format!(
                    "{}_{:0>3}.vpk",
                    vpk_name,
                    file_part.archive_index.to_string()
                ));

                let mut archive_file = File::open(path).ok()?;

                let _ = archive_file.seek(SeekFrom::Start(file_part.entry_offset as _));

                if file_part.entry_length == file_part.entry_length_uncompressed {
                    buf.append(
                        archive_file
                            .read_bytes(file_part.entry_length as _)
                            .ok()?
                            .as_mut(),
                    );
                } else {
                    let compressed_data =
                        archive_file.read_bytes(file_part.entry_length as _).ok()?;

                    let mut decompressed =
                        decompress(&compressed_data, file_part.entry_length_uncompressed as _);
                    buf.append(&mut decompressed);
                }
            }
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
        let entry: &VPKDirectoryEntryRespawn = self
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

        if entry.preload_bytes > 0 {
            out_file
                .write_all(
                    self.tree
                        .preload
                        .get(file_path)
                        .ok_or("Preload data not found in VPK")?
                        .clone()
                        .as_mut(),
                )
                .or(Err("Failed to write to output file"))?;
        }

        for file_part in &entry.file_parts {
            if file_part.entry_length_uncompressed > 0 {
                let path = Path::new(archive_path).join(format!(
                    "{}_{:0>3}.vpk",
                    vpk_name,
                    file_part.archive_index.to_string()
                ));

                let mut archive_file = File::open(path).or(Err("Failed to open archive file"))?;

                let _ = archive_file.seek(SeekFrom::Start(file_part.entry_offset as _));

                if file_part.entry_length == file_part.entry_length_uncompressed {
                    let part = archive_file
                        .read_bytes(file_part.entry_length as _)
                        .or(Err("Failed to read from archive file"))?;

                    out_file
                        .write_all(&part)
                        .or(Err("Failed to write to output file"))?;

                    digest.update(&part);
                } else {
                    let compressed_data = archive_file
                        .read_bytes(file_part.entry_length as _)
                        .or(Err("Failed to read from archive file"))?;

                    let decompressed =
                        decompress(&compressed_data, file_part.entry_length_uncompressed as _);

                    out_file
                        .write_all(&decompressed)
                        .or(Err("Failed to write to output file"))?;

                    digest.update(&decompressed);
                }
            }
        }

        if digest.finalize() != entry.crc {
            Err("CRC must match".to_string())
        } else {
            Ok(())
        }
    }
}

impl From<&mut VPKFile> for VPKRespawn {
    fn from(file: &mut VPKFile) -> Self {
        Self::from_file(file).expect("Failed to read VPK file")
    }
}
