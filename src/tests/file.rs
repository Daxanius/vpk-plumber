use crate::util::file::*;
use crate::util::{Error, Result};
use std::io::{Seek, SeekFrom, Write};
use tempfile::tempfile;

#[test]
fn test_u8() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    file.write_u8(42)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    assert_eq!(file.read_u8()?, 42);
    Ok(())
}

#[test]
fn test_u16() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    file.write_u16(0xBEEF)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    assert_eq!(file.read_u16()?, 0xBEEF);
    Ok(())
}

#[test]
fn test_u24() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    file.write_u24(0x00AB_CDEE)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    assert_eq!(file.read_u24()?, 0x00AB_CDEE & 0x00FF_FFFF);
    Ok(())
}

#[test]
fn test_u32() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    file.write_u32(0xDEAD_BEEF)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    assert_eq!(file.read_u32()?, 0xDEAD_BEEF);
    Ok(())
}

#[test]
fn test_u64() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    file.write_u64(0xDEAD_BEEF_CAFE_BABE)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    assert_eq!(file.read_u64()?, 0xDEAD_BEEF_CAFE_BABE);
    Ok(())
}

#[test]
fn test_string() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    let s = "hello_vpk";
    file.write_string(s)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    let read = file.read_string()?;
    assert_eq!(read, s);
    Ok(())
}

#[test]
fn test_bytes() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    let data = vec![1, 2, 3, 4, 5, 6];
    file.write_bytes(&data)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    let read = file.read_bytes(data.len())?;
    assert_eq!(read, data);
    Ok(())
}

#[test]
fn test_partial_read_bytes() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    let data = vec![9, 8, 7];
    file.write_bytes(&data)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    let read = file.read_bytes(2)?;
    assert_eq!(read, vec![9, 8]);
    Ok(())
}

#[test]
fn test_read_u16_from_empty_file() {
    let mut file = tempfile().unwrap();
    let result = file.read_u16();
    assert!(
        matches!(result, Err(Error::Io(_))),
        "Expected I/O error for empty read"
    );
}

#[test]
fn test_read_string_without_null_terminator() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    file.write_all(b"not_null_terminated").map_err(Error::Io)?;
    file.write_bytes(&[255, 255, 255])?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    let result = file.read_string();
    assert!(
        matches!(result, Err(Error::Utf8(_))),
        "Expected error due to missing null terminator"
    );
    Ok(())
}

#[test]
fn test_read_string_without_null_terminator_eof() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    file.write_all(b"not_null_terminated").map_err(Error::Io)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    let result = file.read_string()?;
    assert_eq!(
        result, "not_null_terminated",
        "End of file strings should be valid without null terminator"
    );
    Ok(())
}

#[test]
fn test_invalid_utf8_string() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    let invalid_utf8 = vec![0xff, 0xfe, 0xfd, 0x00]; // ends with null, but invalid UTF-8
    file.write_all(&invalid_utf8).map_err(Error::Io)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    let result = file.read_string();
    assert!(
        matches!(result, Err(Error::Utf8(_))),
        "Expected UTF-8 decode error"
    );
    Ok(())
}

#[test]
fn test_long_multi_byte_string() -> Result<()> {
    let mut file = tempfile().map_err(Error::Io)?;
    let s = "こんにちは、世界! 🌍🚀✨ -- vpk test string with unicode";
    file.write_string(s)?;

    file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
    let read = file.read_string()?;
    assert_eq!(read, s);
    Ok(())
}
