use std::io::Seek;

use crate::common::{
    file::{VPKFile, VPKFileReader},
    format::{PakFormat, VPKDirectoryEntry, VPKTree},
};

#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
#[cfg(feature = "mem-map")]
use std::collections::HashMap;

pub const VPK_SIGNATURE_V2: u32 = 0x55AA1234;
pub const VPK_VERSION_V2: u32 = 2;

pub struct VPKHeaderV2 {
    pub signature: u32,
    pub version: u32,

    // A zero based index of the archive this file's data is contained in.
    // If 0x7fff, the data follows the directory.
    pub tree_size: u32,

    // If ArchiveIndex is 0x7fff, the offset of the file data relative to the end of the directory (see the header for more details).
    // Otherwise, the offset of the data from the start of the specified archive.
    pub file_data_section_size: u32,

    // The size, in bytes, of the section containing MD5 checksums for external archive content
    pub archive_md5_section_size: u32,

    // The size, in bytes, of the section containing MD5 checksums for content in this file (should always be 48)
    pub other_md5_section_size: u32,

    // The size, in bytes, of the section containing the public key and signature. This is either 0 (CSGO & The Ship) or 296 (HL2, HL2:DM, HL2:EP1, HL2:EP2, HL2:LC, TF2, DOD:S & CS:S)
    pub signature_section_size: u32,
}

impl VPKHeaderV2 {
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
        let file_data_section_size = file.read_u32().or(Err(
            "Could not read header file data section size from file",
        ))?;
        let archive_md5_section_size = file.read_u32().or(Err(
            "Could not read header archive MD5 section size from file",
        ))?;
        let other_md5_section_size = file.read_u32().or(Err(
            "Could not read header other MD5 section size from file",
        ))?;
        let signature_section_size = file.read_u32().or(Err(
            "Could not read header signature section size from file",
        ))?;

        if signature != VPK_SIGNATURE_V2 {
            return Err(format!(
                "VPK header signature should be {:#x}",
                VPK_SIGNATURE_V2
            ));
        }
        if version != VPK_VERSION_V2 {
            return Err(format!("VPK header version should be {}", VPK_VERSION_V2));
        }
        if archive_md5_section_size % 28 != 0 {
            return Err(
                "VPK header archive MD5 section size should be a multiple of 28".to_string(),
            );
        }
        if other_md5_section_size != 48 {
            return Err("VPK header other MD5 section size should be 48".to_string());
        }
        if signature_section_size != 0 && signature_section_size != 296 {
            return Err("VPK header signature section size should be 0 or 296".to_string());
        }

        Ok(Self {
            signature,
            version,
            tree_size,
            file_data_section_size,
            archive_md5_section_size,
            other_md5_section_size,
            signature_section_size,
        })
    }

    pub fn is_format(file: &mut VPKFile) -> bool {
        let pos = file.stream_position().unwrap();

        let signature = file.read_u32();
        let version = file.read_u32();

        let _ = file.seek(std::io::SeekFrom::Start(pos));

        signature.unwrap_or(0) == VPK_SIGNATURE_V2 && version.unwrap_or(0) == VPK_VERSION_V2
    }
}

pub struct VPKArchiveMD5SectionEntry {
    pub archive_index: u32,
    pub starting_offset: u32,  // where to start reading bytes
    pub count: u32,            // how many bytes to check
    pub md5_checksum: Vec<u8>, // expected checksum. len: 16
}

pub struct VPKOtherMD5Section {
    pub tree_checksum: Vec<u8>,                // len: 16
    pub archive_md5_section_checksum: Vec<u8>, // len: 16
    pub unknown: Vec<u8>,                      // len: 16
}
impl VPKOtherMD5Section {
    pub fn new() -> Self {
        Self {
            tree_checksum: vec![0; 16],
            archive_md5_section_checksum: vec![0; 16],
            unknown: vec![0; 16],
        }
    }
}

pub struct VPKSignatureSection {
    pub public_key_size: u32, // always seen as 160 (0xA0) bytes
    pub public_key: Vec<u8>,

    pub signature_size: u32, // always seen as 128 (0x80) bytes
    pub signature: Vec<u8>,
}

pub struct VPKVersion2 {
    pub header: VPKHeaderV2,
    pub tree: VPKTree<VPKDirectoryEntry>,
    pub file_data: Vec<u8>,
    pub archive_md5_section_entries: Vec<VPKArchiveMD5SectionEntry>,
    pub other_md5_section: VPKOtherMD5Section,
    pub signature_section: Option<VPKSignatureSection>,
}

impl PakFormat for VPKVersion2 {
    fn new() -> Self {
        Self {
            header: VPKHeaderV2 {
                signature: VPK_SIGNATURE_V2,
                version: VPK_VERSION_V2,
                tree_size: 0,
                file_data_section_size: 0,
                archive_md5_section_size: 0,
                other_md5_section_size: 48,
                signature_section_size: 0,
            },
            tree: VPKTree::new(),
            file_data: Vec::new(),
            archive_md5_section_entries: Vec::new(),
            other_md5_section: VPKOtherMD5Section::new(),
            signature_section: None,
        }
    }

    fn from_file(file: &mut VPKFile) -> Result<Self, String> {
        let header = VPKHeaderV2::from(file)?;

        let tree_start = file.stream_position().unwrap();
        let tree = VPKTree::from(file, tree_start, header.tree_size.into())?;

        let file_data = file
            .read_bytes(header.file_data_section_size as _)
            .or(Err("Failed reading file data section"))?;

        let mut archive_md5_section_entries = Vec::new();
        while archive_md5_section_entries.len() < (header.archive_md5_section_size / 28) as _ {
            archive_md5_section_entries.push(VPKArchiveMD5SectionEntry {
                archive_index: file
                    .read_u32()
                    .or(Err("Failed reading archive MD5 section archive index"))?,
                starting_offset: file
                    .read_u32()
                    .or(Err("Failed reading archive MD5 section starting offset"))?,
                count: file
                    .read_u32()
                    .or(Err("Failed reading archive MD5 section count"))?,
                md5_checksum: file
                    .read_bytes(16)
                    .or(Err("Failed reading archive MD5 section count"))?,
            });
        }

        let other_md5_section = VPKOtherMD5Section {
            tree_checksum: file
                .read_bytes(16)
                .or(Err("Failed reading other MD5 section tree checksum"))?,
            archive_md5_section_checksum: file.read_bytes(16).or(Err(
                "Failed reading other MD5 section archive MD5 section checksum",
            ))?,
            unknown: file
                .read_bytes(16)
                .or(Err("Failed reading other MD5 section unknown"))?,
        };

        let signature_section = if header.signature_section_size == 296 {
            let public_key_size = file
                .read_u32()
                .or(Err("Failed reading signature section public key size"))?;
            let public_key = file
                .read_bytes(public_key_size as _)
                .or(Err("Failed reading signature section public key"))?;

            let signature_size = file
                .read_u32()
                .or(Err("Failed reading signature section signature size"))?;
            let signature = file
                .read_bytes(signature_size as _)
                .or(Err("Failed reading signature section signature"))?;

            Some(VPKSignatureSection {
                public_key_size,
                public_key,
                signature_size,
                signature,
            })
        } else {
            let _ = file.seek(std::io::SeekFrom::Current(
                header.signature_section_size as _,
            ));
            None
        };

        Ok(Self {
            header,
            tree,
            file_data,
            archive_md5_section_entries,
            other_md5_section,
            signature_section,
        })
    }

    fn read_file(
        self: &Self,
        _archive_path: &String,
        _vpk_name: &String,
        _file_path: &String,
    ) -> Option<Vec<u8>> {
        todo!()
    }

    fn extract_file(
        self: &Self,
        _archive_path: &String,
        _vpk_name: &String,
        _file_path: &String,
        _output_path: &String,
    ) -> Result<(), String> {
        todo!()
    }

    #[cfg(feature = "mem-map")]
    fn extract_file_mem_map(
        self: &Self,
        _archive_path: &String,
        _archive_mmaps: &HashMap<u16, FileBuffer>,
        _vpk_name: &String,
        _file_path: &String,
        _output_path: &String,
    ) -> Result<(), String> {
        todo!()
    }
}

impl TryFrom<&mut VPKFile> for VPKVersion2 {
    fn try_from(file: &mut VPKFile) -> Result<Self, String> {
        Self::from_file(file)
    }

    type Error = String;
}
