//! Support for the Respawn VPK format.

use crate::common::file::{VPKFileReader, VPKFileWriter};
use crate::common::format::{DirEntry, PakReader, PakWriter, VPKTree, VPK_ENTRY_TERMINATOR};
use crate::common::lzham::decompress;
use crc::{Crc, CRC_32_ISO_HDLC};
#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

#[cfg(feature = "mem-map")]
use super::cam::seek_to_wav_data_mem_map;
use super::cam::{create_wav_header, seek_to_wav_data};

/// The 4-byte signature found in the header of a valid Respawn VPK file.
pub const VPK_SIGNATURE_REVPK: u32 = 0x55AA1234;
/// The 4-byte version found in the header of a valid Respawn VPK file.
pub const VPK_VERSION_REVPK: u32 = 196610;
/// The 4-byte magic found at the start of a CAM file entry.
pub const RESPAWN_CAM_ENTRY_MAGIC: u32 = 3302889984;

/// The header of a Respawn VPK file.
#[derive(PartialEq, Eq)]
pub struct VPKHeaderRespawn {
    /// VPK signature. Should be equal to [`VPK_SIGNATURE_REVPK`].
    pub signature: u32,
    /// VPK version. Should be equal to [`VPK_VERSION_REVPK`].
    pub version: u32,

    /// Size of the directory tree in bytes.
    pub tree_size: u32,

    /// An unknown field in the VPK header.
    // Should end up as 0, maybe FileDataSectionSize (see https://developer.valvesoftware.com/wiki/VPK_File_Format#VPK_2).
    pub unknown: u32,
}

impl VPKHeaderRespawn {
    /// Read the header from a file.
    pub fn from(file: &mut File) -> Result<Self, String> {
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

    /// Write the header to a file.
    pub fn write(self: &Self, file: &mut File) -> Result<(), String> {
        if self.signature != VPK_SIGNATURE_REVPK {
            return Err(format!(
                "VPK header signature should be {:#x}",
                VPK_SIGNATURE_REVPK
            ));
        }
        if self.version != VPK_VERSION_REVPK {
            return Err(format!(
                "VPK header version should be {}",
                VPK_VERSION_REVPK
            ));
        }

        file.write_u32(self.signature)
            .or(Err("Could not write signature field to file"))?;
        file.write_u32(self.version)
            .or(Err("Could not write version field to file"))?;
        file.write_u32(self.tree_size)
            .or(Err("Could not write header version to file"))?;
        file.write_u32(self.unknown)
            .or(Err("Could not write unknown field to file"))?;

        Ok(())
    }

    /// Check if a file is in the Respawn VPK format.
    pub fn is_format(file: &mut File) -> bool {
        let pos = file.stream_position().unwrap();

        let signature = file.read_u32();
        let version = file.read_u32();

        let _ = file.seek(std::io::SeekFrom::Start(pos));

        signature.unwrap_or(0) == VPK_SIGNATURE_REVPK && version.unwrap_or(0) == VPK_VERSION_REVPK
    }
}

/// Load flags for the Respawn Source engine.
pub enum EPackedLoadFlags {
    LoadNone,
    LoadVisible = 1 << 0,     // FileSystem visibility?
    LoadCache = 1 << 8,       // Only set for assets not stored in the depot directory.
    LoadAcacheUnk0 = 1 << 10, // Acache uses this!!!
    LoadTextureUnk0 = 1 << 18,
    LoadTextureUnk1 = 1 << 19,
    LoadTextureUnk2 = 1 << 20,
}

/// Texture flags for the Respawn Source engine.
pub enum EPackedTextureFlags {
    TextureNone,
    TextureDefault = 1 << 3,
    TextureEnvironmentMap = 1 << 10,
}

/// The entry format used by Respawn VPKs. For the format used by VPK version 1 and version 2 see [`VPKDirectoryEntry`](crate::common::format::VPKDirectoryEntry).
#[derive(Debug, PartialEq, Eq)]
pub struct VPKDirectoryEntryRespawn {
    /// A 32bit CRC of the file's data. Uses the CRC32 ISO HDLC algorithm.
    pub crc: u32,
    /// The number of preload bytes contained in the directory file.
    pub preload_length: u16,
    /// The list of file parts defined in the entry.
    pub file_parts: Vec<VPKFilePartEntryRespawn>,
}

impl VPKDirectoryEntryRespawn {
    pub fn new() -> Self {
        Self {
            crc: 0,
            preload_length: 0,
            file_parts: Vec::new(),
        }
    }
}

impl DirEntry for VPKDirectoryEntryRespawn {
    fn from(file: &mut File) -> Result<Self, String> {
        let crc = file.read_u32().or(Err("Failed to read CRC"))?;
        let preload_length = file.read_u16().or(Err("Failed to read preload length"))?;

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
            preload_length,
            file_parts,
        })
    }

    fn write(self: &Self, file: &mut File) -> Result<(), String> {
        file.write_u32(self.crc).or(Err("Failed to write CRC"))?;
        file.write_u16(self.preload_length)
            .or(Err("Failed to write preload length"))?;

        for file_part in &self.file_parts {
            file.write_u16(file_part.archive_index)
                .or(Err("Failed writing archive index"))?;
            file.write_u16(file_part.load_flags)
                .or(Err("Failed writing load flags"))?;
            file.write_u32(file_part.texture_flags)
                .or(Err("Failed writing texture flags"))?;
            file.write_u64(file_part.entry_offset)
                .or(Err("Failed writing entry offset"))?;
            file.write_u64(file_part.entry_length)
                .or(Err("Failed writing entry length"))?;
            file.write_u64(file_part.entry_length_uncompressed)
                .or(Err("Failed writing uncompressed entry length"))?;
        }

        file.write_u16(VPK_ENTRY_TERMINATOR) // Entry terminator
            .or(Err("Failed writing entry terminator"))?;

        Ok(())
    }

    fn get_preload_length(self: &Self) -> usize {
        self.preload_length as _
    }
}

/// A file part entry within a Respawn VPK directory entry.
#[derive(Debug, PartialEq, Eq)]
pub struct VPKFilePartEntryRespawn {
    /// The archive index this part is contained in.
    pub archive_index: u16,
    /// The load flags for the file part. (See [`EPackedLoadFlags`])
    pub load_flags: u16,
    /// The texture flags for the file part. (See [`EPackedTextureFlags`])
    pub texture_flags: u32,
    /// The offset of the file part in the archive.
    pub entry_offset: u64,
    /// The length of the file part in the archive.
    pub entry_length: u64,
    /// The length of the file part when decompressd.
    /// Will be equal to `entry_length` if the file was not compressed.
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

/// A Respawn VPK CAM file.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VPKRespawnCam {
    /// The entries in the CAM file.
    /// Map key is VPK archive content offset of the file's first part.
    pub entries: HashMap<u64, VPKRespawnCamEntry>,
}

impl VPKRespawnCam {
    /// Read a CAM from a file.
    pub fn from_file(file: &mut File) -> Result<Self, String> {
        let mut entries: HashMap<u64, VPKRespawnCamEntry> = HashMap::new();

        let file_len = file
            .seek(SeekFrom::End(0))
            .or(Err("Failed to determine length of CAM file"))?;
        let _ = file
            .seek(SeekFrom::Start(0))
            .or(Err("Failed to seek to start of CAM file"))?;

        while file.stream_position().unwrap() < file_len {
            let entry = VPKRespawnCamEntry {
                magic: file.read_u32().or(Err("Failed to read magic"))?,
                original_size: file.read_u32().or(Err("Failed to read original size"))?,
                compressed_size: file.read_u32().or(Err("Failed to read compressed size"))?,
                sample_rate: file.read_u24().or(Err("Failed to read sample rate"))?,
                channels: file.read_u8().or(Err("Failed to read channels"))?,
                sample_count: file.read_u32().or(Err("Failed to read sample count"))?,
                header_size: file.read_u32().or(Err("Failed to read header size"))?,
                vpk_content_offset: file
                    .read_u64()
                    .or(Err("Failed to read VPK content offset"))?,
            };

            if entry.magic == RESPAWN_CAM_ENTRY_MAGIC {
                entries.insert(entry.vpk_content_offset, entry);
            }
        }

        Ok(Self { entries })
    }

    /// Find the entry in a CAM for a given offset.
    pub fn find_entry(&self, vpk_content_offset: u64) -> Option<&VPKRespawnCamEntry> {
        self.entries.get(&vpk_content_offset)
    }
}

/// An entry in a CAM.
///
/// Some audio files don't have a CAM entry, for this case we can generate a default entry with little effort (see [`Self::default`]).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VPKRespawnCamEntry {
    /// The magic number of the entry. Should equal [`RESPAWN_CAM_ENTRY_MAGIC`].
    pub magic: u32,
    /// The original size of the file. (The size of the WAV file including its header).
    pub original_size: u32,
    /// The compressed size of the file. (The size of the OGG file prior to running `audio_installer.exe` on first game launch).
    pub compressed_size: u32,
    /// The sample rate of the audio in the file. (This is actually a u24 in the file but we use a u32 here for simplicity).
    pub sample_rate: u32, // Actually u24
    /// The number of channels in the audio file.
    pub channels: u8,
    /// The number of samples in the audio file.
    pub sample_count: u32,
    /// The size of the header of the audio file. Should always be 44 as the header of a WAV RIFF file is 44 bytes long.
    pub header_size: u32,
    /// The VPK content offset of the file's first part.
    pub vpk_content_offset: u64,
}

impl VPKRespawnCamEntry {
    /// Create a new CAM entry with default values.
    pub fn new() -> Self {
        Self {
            magic: RESPAWN_CAM_ENTRY_MAGIC,
            original_size: 0,
            compressed_size: 0,
            sample_rate: 0,
            channels: 0,
            sample_count: 0,
            header_size: 0,
            vpk_content_offset: 0,
        }
    }

    /// Create a CAM entry with default values for the given directory entry.
    pub fn default(entry: &VPKDirectoryEntryRespawn) -> Self {
        let original_size: u32 = entry
            .file_parts
            .iter()
            .map(|e| e.entry_length_uncompressed as u32)
            .sum();

        VPKRespawnCamEntry {
            magic: RESPAWN_CAM_ENTRY_MAGIC,
            original_size,
            compressed_size: entry
                .file_parts
                .iter()
                .map(|e: &VPKFilePartEntryRespawn| e.entry_length as u32)
                .sum(),
            sample_rate: 44100,
            channels: 1,
            sample_count: (original_size - 44 + 8) / 2,
            header_size: 44,
            vpk_content_offset: entry.file_parts[0].entry_offset,
        }
    }
}

/// The Respawn VPK format.
#[derive(PartialEq, Eq)]
pub struct VPKRespawn {
    /// The VPK's header.
    pub header: VPKHeaderRespawn,
    /// The tree of files in the VPK.
    pub tree: VPKTree<VPKDirectoryEntryRespawn>,
    /// The parsed CAM files for this VPK (external files, not included int dir.vpk file)
    pub archive_cams: HashMap<u16, VPKRespawnCam>,
}

impl PakReader for VPKRespawn {
    fn new() -> Self {
        Self {
            header: VPKHeaderRespawn {
                signature: VPK_SIGNATURE_REVPK,
                version: VPK_VERSION_REVPK,
                tree_size: 0,
                unknown: 0,
            },
            tree: VPKTree::new(),
            archive_cams: HashMap::new(),
        }
    }

    fn from_file(file: &mut File) -> Result<Self, String> {
        let header = VPKHeaderRespawn::from(file)?;

        let tree_start = file.stream_position().unwrap();
        let tree = VPKTree::from(file, tree_start, header.tree_size.into())?;

        let archive_cams = HashMap::new();

        Ok(Self {
            header,
            tree,
            archive_cams,
        })
    }

    fn read_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
    ) -> Option<Vec<u8>> {
        let entry: &VPKDirectoryEntryRespawn = self.tree.files.get(file_path)?;
        let mut buf: Vec<u8> = Vec::new();

        if entry.preload_length > 0 {
            buf.append(self.tree.preload.get(file_path)?.clone().as_mut());
        }

        if entry.file_parts.len() == 0 {
            return None;
        }

        let mut archive_index = entry.file_parts[0].archive_index;
        let path = Path::new(archive_path).join(format!(
            "{}_{:0>3}.vpk",
            vpk_name,
            archive_index.to_string()
        ));

        let mut archive_file = File::open(&path)
            .or(Err("Failed to open archive file"))
            .ok()?;

        // We have to do extra processing if it's a wav file
        let mut expected_len = 0;
        if file_path.ends_with(".wav") {
            let cam_entry = if let Some(cam) = self.archive_cams.get(&archive_index) {
                if let Some(cam_entry) = cam.find_entry(entry.file_parts[0].entry_offset) {
                    cam_entry.to_owned()
                } else {
                    VPKRespawnCamEntry::default(entry)
                }
            } else {
                VPKRespawnCamEntry::default(entry)
            };

            expected_len = cam_entry.original_size;

            let mut header = create_wav_header(&cam_entry);
            buf.append(&mut header);
        }

        let mut total_len = 0;
        for (i, file_part) in entry.file_parts.iter().enumerate() {
            if file_part.entry_length_uncompressed > 0 {
                if file_part.archive_index != archive_index {
                    archive_index = file_part.archive_index;
                    let path = Path::new(archive_path).join(format!(
                        "{}_{:0>3}.vpk",
                        vpk_name,
                        archive_index.to_string()
                    ));
                    archive_file = File::open(path)
                        .or(Err("Failed to open archive file"))
                        .ok()?;
                }

                let _ = archive_file.seek(SeekFrom::Start(file_part.entry_offset as _));

                let mut entry_len = file_part.entry_length;

                if i == 0 && file_path.ends_with(".wav") {
                    entry_len -= seek_to_wav_data(&mut archive_file).ok()?;
                }

                total_len += entry_len;

                if file_part.entry_length == file_part.entry_length_uncompressed {
                    let mut part = archive_file.read_bytes(entry_len as _).ok()?;

                    // Truncate WAV files that exceed their expected length
                    if expected_len > 0
                        && file_path.ends_with(".wav")
                        && total_len > expected_len as _
                    {
                        let new_len = (entry_len as u64) + (expected_len as u64) - total_len;
                        part.truncate(new_len as _);
                    }

                    buf.append(&mut part);
                } else {
                    let compressed_data = archive_file.read_bytes(entry_len as _).ok()?;

                    let mut decompressed =
                        decompress(&compressed_data, file_part.entry_length_uncompressed as _);
                    buf.append(&mut decompressed);
                }
            }
        }

        // Truncate WAV files that exceed their expected length
        if expected_len > 0 && file_path.ends_with(".wav") {
            buf.truncate(expected_len as _);
        }

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();
        digest.update(&buf);

        // We can't check CRCs on wav files because the CRC wasn't calculated with the actual unpacked data
        if digest.finalize() != entry.crc && !file_path.ends_with(".wav") {
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

        if entry.preload_length > 0 {
            let preload_data = self
                .tree
                .preload
                .get(file_path)
                .ok_or("Preload data not found in VPK")?;

            digest.update(&preload_data);

            out_file
                .write_all(&preload_data)
                .or(Err("Failed to write to output file"))?;
        }

        if entry.file_parts.len() == 0 {
            return Err("File had no parts".to_string());
        }

        let mut archive_index = entry.file_parts[0].archive_index;
        let path = Path::new(archive_path).join(format!(
            "{}_{:0>3}.vpk",
            vpk_name,
            archive_index.to_string()
        ));

        let mut archive_file = File::open(&path).or(Err("Failed to open archive file"))?;

        // We have to do extra processing if it's a wav file
        let mut expected_len = 0;
        if file_path.ends_with(".wav") {
            let cam_entry = if let Some(cam) = self.archive_cams.get(&archive_index) {
                if let Some(cam_entry) = cam.find_entry(entry.file_parts[0].entry_offset) {
                    cam_entry.to_owned()
                } else {
                    VPKRespawnCamEntry::default(entry)
                }
            } else {
                VPKRespawnCamEntry::default(entry)
            };

            expected_len = cam_entry.original_size;

            let header = create_wav_header(&cam_entry);
            digest.update(&header);
            out_file
                .write_all(&header)
                .or(Err("Failed to write WAV header"))?;
        }

        let mut total_len = 0;
        for (i, file_part) in entry.file_parts.iter().enumerate() {
            if file_part.entry_length_uncompressed > 0 {
                if file_part.archive_index != archive_index {
                    archive_index = file_part.archive_index;
                    let path = Path::new(archive_path)
                        .join(format!("{}_{:0>3}.vpk", vpk_name, archive_index,));
                    archive_file = File::open(path).or(Err("Failed to open archive file"))?;
                }

                let _ = archive_file.seek(SeekFrom::Start(file_part.entry_offset as _));

                let mut entry_len = file_part.entry_length;

                if i == 0 && file_path.ends_with(".wav") {
                    entry_len -= seek_to_wav_data(&mut archive_file)?;
                }

                total_len += entry_len;

                if file_part.entry_length == file_part.entry_length_uncompressed {
                    let mut part = archive_file
                        .read_bytes(entry_len as _)
                        .or(Err("Failed to read from archive file"))?;

                    // Truncate WAV files that exceed their expected length
                    if expected_len > 0
                        && file_path.ends_with(".wav")
                        && total_len > expected_len as _
                    {
                        let new_len = (entry_len as u64) + (expected_len as u64) - total_len;
                        part.truncate(new_len as _);
                    }

                    out_file
                        .write_all(&part)
                        .or(Err("Failed to write to output file"))?;

                    digest.update(&part);
                } else {
                    let compressed_data = archive_file
                        .read_bytes(entry_len as _)
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

        // We can't check CRCs on wav files because the CRC wasn't calculated with the actual unpacked data
        if digest.finalize() != entry.crc && !file_path.ends_with(".wav") {
            Err("CRC must match".to_string())
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "mem-map")]
    fn extract_file_mem_map(
        self: &Self,
        archive_path: &String,
        archive_mmaps: &HashMap<u16, FileBuffer>,
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

        if entry.preload_length > 0 {
            let preload_data = self
                .tree
                .preload
                .get(file_path)
                .ok_or("Preload data not found in VPK")?;

            digest.update(&preload_data);

            out_file
                .write_all(&preload_data)
                .or(Err("Failed to write to output file"))?;
        }

        if entry.file_parts.len() == 0 {
            return Err("File had no parts".to_string());
        }

        let mut archive_index = entry.file_parts[0].archive_index;
        let _path = Path::new(archive_path).join(format!(
            "{}_{:0>3}.vpk",
            vpk_name,
            archive_index.to_string()
        ));

        let mut archive_file = archive_mmaps
            .get(&archive_index)
            .ok_or("Couldn't find memory-mapped file")?;

        archive_file.prefetch(
            entry.file_parts[0].entry_offset as _,
            entry.file_parts[0].entry_length as _,
        );

        // We have to do extra processing if it's a wav file
        let mut expected_len = entry
            .file_parts
            .iter()
            .map(|e| e.entry_length_uncompressed as u32)
            .sum();
        if file_path.ends_with(".wav") {
            let cam_entry = if let Some(cam) = self.archive_cams.get(&archive_index) {
                if let Some(cam_entry) = cam.find_entry(entry.file_parts[0].entry_offset) {
                    cam_entry.to_owned()
                } else {
                    VPKRespawnCamEntry::default(entry)
                }
            } else {
                VPKRespawnCamEntry::default(entry)
            };

            expected_len = cam_entry.original_size;

            let header = create_wav_header(&cam_entry);
            digest.update(&header);
            out_file
                .write_all(&header)
                .or(Err("Failed to write WAV header"))?;
        }

        // Set the length of the file
        out_file
            .set_len(expected_len as _)
            .or(Err("Failed to set length of output file"))?;

        let mut total_len = 0;
        for (i, file_part) in entry.file_parts.iter().enumerate() {
            // Prefetch next file part
            if i < entry.file_parts.len() - 1 {
                archive_mmaps
                    .get(&archive_index)
                    .ok_or("Couldn't find memory-mapped file")?
                    .prefetch(
                        entry.file_parts[i + 1].entry_offset as _,
                        entry.file_parts[i + 1].entry_length as _,
                    );
            }

            if file_part.entry_length_uncompressed > 0 {
                if file_part.archive_index != archive_index {
                    archive_index = file_part.archive_index;

                    archive_file = archive_mmaps
                        .get(&archive_index)
                        .ok_or("Couldn't find memory-mapped file")?;
                }

                let mut entry_offset = file_part.entry_offset;
                let mut entry_len = file_part.entry_length;

                if i == 0 && file_path.ends_with(".wav") {
                    let seek = seek_to_wav_data_mem_map(&archive_file, entry_offset)?;
                    entry_offset += seek;
                    entry_len -= seek;
                }

                total_len += entry_len;

                if file_part.entry_length == file_part.entry_length_uncompressed {
                    // Truncate WAV files that exceed their expected length
                    if expected_len > 0
                        && file_path.ends_with(".wav")
                        && total_len > expected_len as _
                    {
                        entry_len = (entry_len as u64) + (expected_len as u64) - total_len;
                    }

                    let part =
                        &archive_file[(entry_offset as usize)..(entry_offset + entry_len) as usize];

                    out_file
                        .write_all(part)
                        .or(Err("Failed to write to output file"))?;

                    digest.update(part);
                } else {
                    let compressed_data = archive_file
                        .get(
                            file_part.entry_offset as usize
                                ..(file_part.entry_offset + entry_len) as usize,
                        )
                        .ok_or("Failed to read from archive file")?
                        .to_vec();

                    let decompressed =
                        decompress(&compressed_data, file_part.entry_length_uncompressed as _);

                    out_file
                        .write_all(&decompressed)
                        .or(Err("Failed to write to output file"))?;

                    digest.update(&decompressed);
                }
            }
        }

        // We can't check CRCs on wav files because the CRC wasn't calculated with the actual unpacked data
        if digest.finalize() != entry.crc && !file_path.ends_with(".wav") {
            Err("CRC must match".to_string())
        } else {
            Ok(())
        }
    }
}

impl PakWriter for VPKRespawn {
    fn write_dir(self: &Self, output_path: &String) -> Result<(), String> {
        let out_path = std::path::Path::new(output_path);
        if let Some(prefix) = out_path.parent() {
            std::fs::create_dir_all(prefix).or(Err("Failed to create parent directories"))?;
        };

        let mut out_file = File::create(out_path).or(Err("Failed to create output file."))?;

        self.header.write(&mut out_file)?;
        self.tree.write(&mut out_file)?;

        Ok(())
    }
}

impl VPKRespawn {
    /// Reads a CAM file and adds it to the map of parsed CAMs for this VPK
    pub fn read_cam(self: &mut Self, archive_index: u16, cam_path: &String) -> Result<(), String> {
        let mut cam_file = File::open(cam_path).or(Err(format!(
            "Failed to open CAM file for archive {}",
            archive_index
        )))?;

        let cam = VPKRespawnCam::from_file(&mut cam_file)?;
        self.archive_cams.insert(archive_index, cam);

        Ok(())
    }

    /// Reads all CAM files for this VPK and adds them to the map of parsed CAMs for this VPK
    pub fn read_all_cams(
        self: &mut Self,
        archive_path: &String,
        vpk_name: &String,
    ) -> Result<(), String> {
        let mut archive_indices = HashSet::<u16>::new();
        for (path, entry) in self.tree.files.iter_mut() {
            let archive_index = entry.file_parts[0].archive_index;
            if path.ends_with(".wav") {
                archive_indices.insert(archive_index);
            }
        }

        let mut res = Ok(());

        let path = Path::new(archive_path);
        for archive_index in archive_indices {
            if !self.archive_cams.contains_key(&archive_index) {
                let cam_path = path
                    .join(format!(
                        "{}_{:0>3}.vpk.cam",
                        vpk_name,
                        archive_index.to_string()
                    ))
                    .to_str()
                    .ok_or(format!(
                        "Failed to determine CAM path for archive {}",
                        archive_index
                    ))?
                    .to_string();

                match self.read_cam(archive_index, &cam_path) {
                    Ok(_) => (),
                    Err(err) => {
                        res = match res {
                            Ok(_) => Err(format!("Encountered erors reading CAM files: {}", err)),
                            Err(org) => Err(format!("{}, {}", org, err)),
                        };
                    }
                };
            }
        }

        res
    }
}

impl TryFrom<&mut File> for VPKRespawn {
    fn try_from(file: &mut File) -> Result<Self, String> {
        Self::from_file(file)
    }

    type Error = String;
}
