use crate::common::file::{VPKFile, VPKFileReader};
#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom};

pub const VPK_ENTRY_TERMINATOR: u16 = 0xFFFF;

pub trait DirEntry {
    fn from(file: &mut File) -> Result<Self, String>
    where
        Self: Sized;
    fn get_preload_bytes(self: &Self) -> usize;
}

pub struct VPKTree<DirectoryEntry> {
    pub files: HashMap<String, DirectoryEntry>,
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

    pub fn from(file: &mut VPKFile, start: u64, size: u64) -> Result<Self, String> {
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

#[derive(Debug, PartialEq, Eq)]
pub struct VPKDirectoryEntry {
    pub crc: u32,           // A 32bit CRC of the file's data.
    pub preload_bytes: u16, // The number of bytes contained in the index file.

    // A zero based index of the archive this file's data is contained in.
    // If 0x7fff, the data follows the directory.
    pub archive_index: u16,

    // If ArchiveIndex is 0x7fff, the offset of the file data relative to the end of the directory (see the header for more details).
    // Otherwise, the offset of the data from the start of the specified archive.
    pub entry_offset: u32,

    // If zero, the entire file is stored in the preload data.
    // Otherwise, the number of bytes stored starting at EntryOffset.
    pub entry_length: u32,
    pub terminator: u16, // Should always be 0xFFFF
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
    fn from(file: &mut VPKFile) -> Result<Self, String> {
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

pub trait PakFormat {
    fn new() -> Self;
    fn from_file(file: &mut File) -> Result<Self, String>
    where
        Self: Sized;

    fn read_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
    ) -> Option<Vec<u8>>;

    fn extract_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<(), String>;

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
