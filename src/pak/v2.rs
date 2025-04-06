//! Support for the VPK version 1 format.

use super::{Error, PakReader, PakWorker, PakWriter, Result, VPKDirectoryEntry, VPKTree};
use crate::util::file::VPKFileReader;
use std::{fs::File, io::Seek};

#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
#[cfg(feature = "mem-map")]
use std::collections::HashMap;

/// The 4-byte signature found in the header of a valid VPK version 2 file.
pub const VPK_SIGNATURE_V2: u32 = 0x55AA_1234;

/// The 4-byte version found in the header of a valid VPK version 2 file.
pub const VPK_VERSION_V2: u32 = 2;

/// The header of a VPK version 2 file.
#[derive(PartialEq, Eq, Debug)]
pub struct VPKHeaderV2 {
    /// VPK signature. Should be equal to [`VPK_SIGNATURE_V2`].
    pub signature: u32,

    /// VPK version. Should be equal to [`VPK_VERSION_V2`].
    pub version: u32,

    /// Size of the directory tree in bytes.
    pub tree_size: u32,

    /// The size, in bytes, of the section containing file data
    pub file_data_section_size: u32,

    /// The size, in bytes, of the section containing MD5 checksums for external archive content
    pub archive_md5_section_size: u32,

    /// The size, in bytes, of the section containing MD5 checksums for content in this file (should always be 48)
    pub other_md5_section_size: u32,

    /// The size, in bytes, of the section containing the public key and signature. This is either 0 (CS:GO & The Ship) or 296 (HL2, HL2:DM, HL2:EP1, HL2:EP2, HL2:LC, TF2, DOD:S & CS:S)
    pub signature_section_size: u32,
}

#[repr(C)]
#[derive(PartialEq, Eq, Debug)]
pub struct VPKArchiveMD5SectionEntry {
    pub archive_index: u32,
    pub starting_offset: u32,   // where to start reading bytes
    pub count: u32,             // how many bytes to check
    pub md5_checksum: [u8; 16], // expected checksum. len: 16
}

#[repr(C)]
#[derive(PartialEq, Eq, Debug)]
pub struct VPKOtherMD5Section {
    pub tree_checksum: [u8; 16],
    pub archive_md5_section_checksum: [u8; 16],
    pub unknown: [u8; 16],
}

#[repr(C)]
#[derive(PartialEq, Eq, Debug)]
pub struct VPKSignatureSection {
    pub public_key_size: u32, // always seen as 160 (0xA0) bytes
    pub public_key: [u8; 160],

    pub signature_size: u32, // always seen as 128 (0x80) bytes
    pub signature: [u8; 128],
}

impl VPKHeaderV2 {
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
        if signature != VPK_SIGNATURE_V2 {
            return Err(Error::InvalidSignature(format!(
                "Header signature should be {VPK_SIGNATURE_V2:#X} but is {signature:#X}"
            )));
        }

        let version = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read version".to_string(),
        })?;

        // Check the version before moving on
        if version != VPK_VERSION_V2 {
            return Err(Error::BadVersion(format!(
                "Header version should be {VPK_VERSION_V2} but is {version}"
            )));
        }

        let tree_size = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read tree size".to_string(),
        })?;

        let file_data_section_size = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read file data section size".to_string(),
        })?;

        let archive_md5_section_size = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read MD5 section size".to_string(),
        })?;

        // Check the archive md5 section size
        if archive_md5_section_size as usize % size_of::<VPKArchiveMD5SectionEntry>() != 0 {
            return Err(Error::BadData(format!(
                "Header archive MD5 section size should be a multiple of 28 but is {archive_md5_section_size}"
            )));
        }

        let other_md5_section_size = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read other MD5 section size".to_string(),
        })?;

        // Check the other section size
        if other_md5_section_size as usize != size_of::<VPKOtherMD5Section>() {
            return Err(Error::BadData(format!(
                "Header archive MD5 section size should be 48 but is {other_md5_section_size}"
            )));
        }

        let signature_section_size = file.read_u32().map_err(|e| Error::Util {
            source: e,
            context: "Failed to read signature size".to_string(),
        })?;

        if signature_section_size != 0
            && signature_section_size as usize != size_of::<VPKSignatureSection>()
        {
            return Err(Error::BadData(format!(
                "Header signature section size should be 0 or 296 but is {signature_section_size}"
            )));
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

    /// Check if a file is in the VPK version 2 format.
    pub fn is_format(file: &mut File) -> bool {
        let Ok(pos) = file.stream_position() else {
            return false;
        };

        let signature = file.read_u32();
        let version = file.read_u32();

        let _ = file.seek(std::io::SeekFrom::Start(pos));

        signature.unwrap_or(0) == VPK_SIGNATURE_V2 && version.unwrap_or(0) == VPK_VERSION_V2
    }
}

impl Default for VPKOtherMD5Section {
    fn default() -> Self {
        Self::new()
    }
}

impl VPKOtherMD5Section {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tree_checksum: [0; 16],
            archive_md5_section_checksum: [0; 16],
            unknown: [0; 16],
        }
    }
}

/// The VPK version 2 format.
pub struct VPKVersion2 {
    /// The VPK's header.
    pub header: VPKHeaderV2,

    /// The tree of files in the VPK.
    pub tree: VPKTree<VPKDirectoryEntry>,

    /// The file data section of the VPK.
    pub file_data: Vec<u8>,

    /// The archive md5 section of the VPK.
    pub archive_md5_section_entries: Vec<VPKArchiveMD5SectionEntry>,

    /// The other md5 section of the VPK.
    pub other_md5_section: VPKOtherMD5Section,

    /// The signature section of the VPK.
    pub signature_section: Option<VPKSignatureSection>,
}

impl PakReader for VPKVersion2 {
    fn read_file(
        &self,
        _archive_path: &String,
        _vpk_name: &String,
        _file_path: &String,
    ) -> Option<Vec<u8>> {
        todo!()
    }

    fn extract_file(
        &self,
        _archive_path: &String,
        _vpk_name: &String,
        _file_path: &String,
        _output_path: &String,
    ) -> Result<()> {
        todo!()
    }

    #[cfg(feature = "mem-map")]
    fn extract_file_mem_map(
        &self,
        _archive_path: &String,
        _archive_mmaps: &HashMap<u16, FileBuffer>,
        _vpk_name: &String,
        _file_path: &String,
        _output_path: &String,
    ) -> Result<()> {
        todo!()
    }
}

impl PakWriter for VPKVersion2 {
    fn write_dir(&self, _out_path: &str) -> Result<()> {
        todo!()
    }
}

impl PakWorker for VPKVersion2 {
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

    fn from_file(file: &mut File) -> Result<Self> {
        let header = VPKHeaderV2::from(file)?;

        let tree_start = file.stream_position().map_err(Error::Io)?;
        let tree = VPKTree::from(file, tree_start, header.tree_size.into())?;

        let file_data = file
            .read_bytes(header.file_data_section_size as _)
            .map_err(|e| Error::Util {
                source: e,
                context: "Failed to read file data section".to_string(),
            })?;

        let mut archive_md5_section_entries = Vec::new();
        while archive_md5_section_entries.len() < (header.archive_md5_section_size / 28) as _ {
            archive_md5_section_entries.push(VPKArchiveMD5SectionEntry {
                archive_index: file.read_u32().map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read archive md5 section archive index".to_string(),
                })?,

                starting_offset: file.read_u32().map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read archive md5 section offset".to_string(),
                })?,

                count: file.read_u32().map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read archive md5 section count".to_string(),
                })?,

                md5_checksum: file
                    .read_bytes(16)
                    .map_err(|e| Error::Util {
                        source: e,
                        context: "Failed to read archive md5 section signature".to_string(),
                    })?
                    .try_into()
                    .expect("Bytes read should match parameter value"),
            });
        }

        let other_md5_section = VPKOtherMD5Section {
            tree_checksum: file
                .read_bytes(16)
                .map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read other md5 section tree checksum".to_string(),
                })?
                .try_into()
                .expect("Bytes read should match parameter value"),

            archive_md5_section_checksum: file
                .read_bytes(16)
                .map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read other md5 section checksum".to_string(),
                })?
                .try_into()
                .expect("Bytes read should match parameter value"),

            unknown: file
                .read_bytes(16)
                .map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read other md5 section unknown".to_string(),
                })?
                .try_into()
                .expect("Bytes read should match parameter value"),
        };

        let signature_section = if header.signature_section_size == 296 {
            let public_key_size = file.read_u32().map_err(|e| Error::Util {
                source: e,
                context: "Failed reading signature public key size".to_string(),
            })?;

            let public_key = file
                .read_bytes(public_key_size as _)
                .map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read signature public key".to_string(),
                })?;

            let signature_size = file.read_u32().map_err(|e| Error::Util {
                source: e,
                context: "Failed to read signature size".to_string(),
            })?;

            let signature = file
                .read_bytes(signature_size as _)
                .map_err(|e| Error::Util {
                    source: e,
                    context: "Failed to read signature".to_string(),
                })?;

            Some(VPKSignatureSection {
                public_key_size,
                public_key: public_key
                    .try_into()
                    .map_err(|_| Error::BadData("Public key must be 160 bytes".to_string()))?,
                signature_size,
                signature: signature
                    .try_into()
                    .map_err(|_| Error::BadData("Signature must be 128 bytes".to_string()))?,
            })
        } else {
            let _ = file.seek(std::io::SeekFrom::Current(
                header.signature_section_size.into(),
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
}

impl TryFrom<&mut File> for VPKVersion2 {
    fn try_from(file: &mut File) -> Result<Self> {
        Self::from_file(file)
    }

    type Error = Error;
}
