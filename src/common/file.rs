//! File reading utilities for VPK files.

use std::{
    fs::File,
    io::{Read, Result},
};

/// Trait for reading data from binary files.
///
/// Always uses little-endian byte order. Moves cursor forward after reading.
pub trait VPKFileReader {
    /// Reads a single byte from the file into a [`u8`].
    fn read_u8(self: &mut Self) -> Result<u8>;
    /// Reads 2 bytes from the file into a [`u16`].
    fn read_u16(self: &mut Self) -> Result<u16>;
    /// Reads 3 bytes from the file into a [`u32`].
    fn read_u24(self: &mut Self) -> Result<u32>;
    /// Reads 4 bytes from the file into a [`u32`].
    fn read_u32(self: &mut Self) -> Result<u32>;
    /// Reads 8 bytes from the file into a [`u64`].
    fn read_u64(self: &mut Self) -> Result<u64>;

    /// Reads a null-terminated string from the file.
    fn read_string(self: &mut Self) -> Result<String>;
    /// Reads a specified number of bytes from the file into a [`Vec<u8>`].
    fn read_bytes(self: &mut Self, count: usize) -> Result<Vec<u8>>;
}

impl VPKFileReader for File {
    fn read_u8(self: &mut Self) -> Result<u8> {
        let mut b: [u8; 1] = [0];
        self.read(&mut b)?;

        Ok(b[0])
    }

    fn read_u16(self: &mut Self) -> Result<u16> {
        let mut b: [u8; 2] = [0, 0];
        self.read(&mut b)?;

        Ok(u16::from_le_bytes(b))
    }

    fn read_u24(self: &mut Self) -> Result<u32> {
        let mut b: [u8; 3] = [0, 0, 0];
        self.read(&mut b)?;

        let b_u32: [u8; 4] = [b[0], b[1], b[2], 0];

        Ok(u32::from_le_bytes(b_u32))
    }

    fn read_u32(self: &mut Self) -> Result<u32> {
        let mut b: [u8; 4] = [0, 0, 0, 0];
        self.read(&mut b)?;

        Ok(u32::from_le_bytes(b))
    }

    fn read_u64(self: &mut Self) -> Result<u64> {
        let mut b: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        self.read(&mut b)?;

        Ok(u64::from_le_bytes(b))
    }

    fn read_string(self: &mut Self) -> Result<String> {
        let mut str_buf = Vec::new();
        loop {
            let mut b: [u8; 1] = [0];
            self.read(&mut b)?;

            if b[0] == 0 {
                break;
            }
            str_buf.push(b[0]);
        }

        Ok(String::from_utf8(str_buf).unwrap())
    }

    fn read_bytes(self: &mut Self, count: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; count];
        let size = self.read(&mut buffer)?;
        buffer.truncate(size);

        Ok(buffer)
    }
}
