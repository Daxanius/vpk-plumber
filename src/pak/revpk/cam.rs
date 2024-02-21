#[cfg(feature = "mem-map")]
use memmap2::Mmap;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::RwLock,
};

use crate::common::file::VPKFile;

use super::format::{VPKRespawnCam, VPKRespawnCamEntry};

const SAMPLE_DEPTH: u16 = 16;

pub fn create_wav_header(cam_entry: &VPKRespawnCamEntry) -> Vec<u8> {
    let mut header = [0u8; 44];

    // "RIFF" magic
    header[0..4].copy_from_slice(&0x52494646_u32.to_be_bytes());

    // File size
    let file_len: u32 = 2 * cam_entry.sample_count * cam_entry.channels as u32;
    header[4..8].copy_from_slice(&(file_len - 8 + 44).to_le_bytes());

    // "RIFF" magic
    header[8..12].copy_from_slice(&0x57415645_u32.to_be_bytes());

    // "fmt\20" magic
    header[12..16].copy_from_slice(&0x666D7420_u32.to_be_bytes());

    // Format data length
    header[16..20].copy_from_slice(&16_u32.to_le_bytes());

    // Type (PCM)
    header[20..22].copy_from_slice(&1_u16.to_le_bytes());

    // Channels
    header[22..24].copy_from_slice(&(cam_entry.channels as u16).to_le_bytes());

    // Sample rate
    header[24..28].copy_from_slice(&cam_entry.sample_rate.to_le_bytes());

    // Sample rate * sample depth * channels / 8
    let bytes_per_sec = cam_entry.sample_rate * SAMPLE_DEPTH as u32 * cam_entry.channels as u32 / 8;
    header[28..32].copy_from_slice(&bytes_per_sec.to_le_bytes());

    // Sample depth * channels / 8
    header[32..34].copy_from_slice(&(SAMPLE_DEPTH * cam_entry.channels as u16 / 8).to_le_bytes());

    // Sample depth
    header[34..36].copy_from_slice(&SAMPLE_DEPTH.to_le_bytes());

    // "data" magic
    header[36..40].copy_from_slice(&0x64617461_u32.to_be_bytes());

    // File length
    header[40..44].copy_from_slice(&file_len.to_le_bytes());

    header.to_vec()
}

pub fn seek_to_wav_data(file: &mut VPKFile) -> Result<u64, String> {
    let pos = file
        .seek(SeekFrom::Current(44))
        .or(Err("Failed to seek in file"))?;
    loop {
        let mut b: [u8; 1] = [0];
        let _ = file.read(&mut b);

        if b[0] != 0xCB {
            let res = file
                .seek(SeekFrom::Current(-1))
                .or(Err("Failed to seek in file"))?;
            return Ok(44 + res - pos);
        }
    }
}

#[cfg(feature = "mem-map")]
pub fn seek_to_wav_data_mem_map(file: &Mmap, start_pos: u64) -> Result<u64, String> {
    let mut pos = start_pos + 44;
    loop {
        let b = file[pos as usize];
        if b != 0xCB {
            return Ok(pos - start_pos);
        }

        pos += 1;
    }
}

static CAM_MAP: Lazy<RwLock<HashMap<PathBuf, VPKRespawnCam>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn get_cam(cam_path: PathBuf) -> Result<VPKRespawnCam, String> {
    {
        let cam_map = CAM_MAP.read().or(Err("CAM cache couldn't be accessed"))?;
        if cam_map.contains_key(&cam_path) {
            return Ok(cam_map.get(&cam_path).unwrap().clone());
        }
    }

    let mut cam_file = File::open(cam_path.clone()).or(Err("Failed to open CAM file"))?;
    let cam = VPKRespawnCam::from_file(&mut cam_file);

    {
        let mut cam_map = CAM_MAP.write().or(Err("CAM cache couldn't be accessed"))?;
        if let Ok(cam) = &cam {
            cam_map.insert(cam_path, cam.clone());
        }
    }

    cam
}

pub fn get_cam_entry(cam_path: PathBuf, entry_offset: u64) -> Result<VPKRespawnCamEntry, String> {
    {
        // Check for existing entry in the cache
        let cam_map = CAM_MAP.read().or(Err("CAM cache couldn't be accessed"))?;
        if cam_map.contains_key(&cam_path) {
            let cam = cam_map.get(&cam_path).unwrap();

            if let Some(entry) = cam.find_entry(entry_offset) {
                return Ok(entry.clone());
            } else {
                return Err("Failed to find cam entry".to_string());
            }
        }
    }

    {
        // Read the cam file and add it to the cache
        let mut cam_map = CAM_MAP.write().or(Err("CAM cache couldn't be accessed"))?;

        // Check again in case we acquired the lock from another thread that was reading the same cam
        if cam_map.contains_key(&cam_path) {
            let cam = cam_map.get(&cam_path).unwrap();

            if let Some(entry) = cam.find_entry(entry_offset) {
                return Ok(entry.clone());
            } else {
                return Err("Failed to find cam entry".to_string());
            }
        }

        let mut cam_file = File::open(cam_path.clone()).or(Err("Failed to open CAM file"))?;
        let cam = VPKRespawnCam::from_file(&mut cam_file);

        if let Ok(cam) = cam {
            let cam_entry = if let Some(entry) = cam.find_entry(entry_offset) {
                Some(entry.clone())
            } else {
                None
            };

            cam_map.insert(cam_path, cam);

            if let Some(entry) = cam_entry {
                return Ok(entry);
            } else {
                return Err("Failed to find cam entry".to_string());
            }
        }
    }

    Err("Failed to find cam entry".to_string())
}
