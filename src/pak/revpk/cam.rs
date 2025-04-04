#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use super::format::VPKRespawnCamEntry;

const SAMPLE_DEPTH: u16 = 16;

#[must_use] pub fn create_wav_header(cam_entry: &VPKRespawnCamEntry) -> Vec<u8> {
    let mut header = [0u8; 44];

    // "RIFF" magic
    header[0..4].copy_from_slice(&0x52494646_u32.to_be_bytes());

    // File size
    let file_len: u32 = 2 * cam_entry.sample_count * u32::from(cam_entry.channels);
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
    header[22..24].copy_from_slice(&u16::from(cam_entry.channels).to_le_bytes());

    // Sample rate
    header[24..28].copy_from_slice(&cam_entry.sample_rate.to_le_bytes());

    // Sample rate * sample depth * channels / 8
    let bytes_per_sec = cam_entry.sample_rate * u32::from(SAMPLE_DEPTH) * u32::from(cam_entry.channels) / 8;
    header[28..32].copy_from_slice(&bytes_per_sec.to_le_bytes());

    // Sample depth * channels / 8
    header[32..34].copy_from_slice(&(SAMPLE_DEPTH * u16::from(cam_entry.channels) / 8).to_le_bytes());

    // Sample depth
    header[34..36].copy_from_slice(&SAMPLE_DEPTH.to_le_bytes());

    // "data" magic
    header[36..40].copy_from_slice(&0x64617461_u32.to_be_bytes());

    // File length
    header[40..44].copy_from_slice(&file_len.to_le_bytes());

    header.to_vec()
}

pub fn seek_to_wav_data(file: &mut File) -> Result<u64, String> {
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
pub fn seek_to_wav_data_mem_map(file: &FileBuffer, start_pos: u64) -> Result<u64, String> {
    let mut pos = start_pos + 44;
    loop {
        let b = file[pos as usize];
        if b != 0xCB {
            return Ok(pos - start_pos);
        }

        pos += 1;
    }
}
