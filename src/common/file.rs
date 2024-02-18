use std::{
    fs::File,
    io::{Read, Result},
};

pub type VPKFile = File;

pub trait VPKFileReader {
    fn read_u8(self: &mut Self) -> Result<u8>;
    fn read_u16(self: &mut Self) -> Result<u16>;
    fn read_u32(self: &mut Self) -> Result<u32>;
    fn read_u64(self: &mut Self) -> Result<u64>;

    fn read_string(self: &mut Self) -> Result<String>;
    fn read_bytes(self: &mut Self, count: usize) -> Result<Vec<u8>>;
}

impl VPKFileReader for VPKFile {
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
        buffer.shrink_to(size);

        Ok(buffer)
    }
}
