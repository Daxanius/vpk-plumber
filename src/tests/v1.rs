use crate::{
    common::{
        file::VPKFileReader,
        format::{PakReader, PakWriter, VPKDirectoryEntry},
    },
    pak::v1::format::VPKVersion1,
};
use std::{
    fs::{File, remove_dir, remove_file},
    io::Seek,
    path::Path,
};

#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
#[cfg(feature = "mem-map")]
use std::collections::HashMap;

#[test]
fn read_empty_vpk_v1() {
    let path = Path::new("./test_files/empty_v1_dir.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(vpk.tree.files.len(), 0, "VPK tree should have 0 entries");
    assert_eq!(
        vpk.tree.files.get("test/file.txt"),
        None,
        "File \"test/file.txt\" shouldn't exist"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );
}

#[test]
fn read_invalid_vpk_v1() {
    let path = Path::new("./test_files/single_file_v1_000.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion1::try_from(&mut file);
    assert!(
        vpk.is_err_and(|x| x.contains("VPK header signature should be 0x55aa1234")),
        "VPK file should be invalid",
    );
}

#[test]
fn read_single_file_vpk_v1() {
    let path = Path::new("./test_files/single_file_v1_dir.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(vpk.tree.files.len(), 1, "VPK tree should have 1 entry");
    assert_eq!(
        vpk.tree.files.get("test/file.txt"),
        Some(&VPKDirectoryEntry {
            crc: 0x4570_FA16,
            preload_length: 0,
            archive_index: 0,
            entry_length: 9,
            entry_offset: 0,
            terminator: 0xFFFF,
        }),
        "File \"test/file.txt\" should exist"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );

    let test_file = vpk.read_file(
        &String::from("./test_files"),
        &String::from("single_file_v1"),
        &String::from("test/file.txt"),
    );

    assert_eq!(
        test_file,
        Some(Vec::from("test text")),
        "File contents should be \"test text\""
    );

    let out_path = String::from("./test_files/out/file_v1.txt");

    let _ = vpk.extract_file(
        &String::from("./test_files"),
        &String::from("single_file_v1"),
        &String::from("test/file.txt"),
        &out_path,
    );

    assert_eq!(
        test_file,
        Some(File::open(&out_path).unwrap().read_string().unwrap().into()),
        "File contents should match {}",
        &out_path
    );

    let _ = remove_file(out_path);
    let _ = remove_dir("./test_files/out");
}

#[test]
fn read_single_file_eof_data_vpk_v1() {
    let path = Path::new("./test_files/single_file_eof_data_v1_dir.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(vpk.tree.files.len(), 1, "VPK tree should have 1 entry");
    assert_eq!(
        vpk.tree.files.get("test/file.txt"),
        Some(&VPKDirectoryEntry {
            crc: 0x4570_FA16,
            preload_length: 0,
            archive_index: 0xFF7F,
            entry_length: 9,
            entry_offset: 0,
            terminator: 0xFFFF,
        }),
        "File \"test/file.txt\" should exist"
    );

    let test_file = vpk.read_file(
        &String::from("./test_files"),
        &String::from("single_file_eof_data_v1"),
        &String::from("test/file.txt"),
    );

    assert_eq!(
        test_file,
        Some(Vec::from("test text")),
        "File contents should be \"test text\""
    );

    let out_path = String::from("./test_files/out/file_v1.txt");

    let _ = vpk.extract_file(
        &String::from("./test_files"),
        &String::from("single_file_eof_data_v1"),
        &String::from("test/file.txt"),
        &out_path,
    );

    assert_eq!(
        test_file,
        Some(File::open(&out_path).unwrap().read_string().unwrap().into()),
        "File contents should match {}",
        &out_path
    );

    let _ = remove_file(out_path);
    let _ = remove_dir("./test_files/out");
}

#[cfg(feature = "mem-map")]
#[test]
fn extract_mem_map_single_file_vpk_v1() {
    let path = Path::new("./test_files/single_file_v1_dir.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");

    let mut archive_mmaps = HashMap::new();
    archive_mmaps.insert(
        0,
        FileBuffer::open("./test_files/single_file_v1_000.vpk").unwrap(),
    );

    let out_path = String::from("./test_files/out/file_mem_map_v1.txt");

    let _ = vpk.extract_file_mem_map(
        &String::from("./test_files"),
        &archive_mmaps,
        &String::from("single_file_v1"),
        &String::from("test/file.txt"),
        &out_path,
    );

    assert_eq!(
        Some(Vec::from("test text")),
        Some(File::open(&out_path).unwrap().read_string().unwrap().into()),
        "File contents should match {}",
        &out_path
    );

    let _ = remove_file(out_path);
    let _ = remove_dir("./test_files/out");
}

#[test]
fn read_big_vpk_v1() {
    let path = Path::new("./test_files/portal/pak01_dir.vpk");
    let mut file: File = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(
        vpk.tree.files.len(),
        449,
        "VPK tree should have 449 entries"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );
}

#[test]
fn write_parity_vpk_v1() {
    let path = Path::new("./test_files/portal/pak01_dir.vpk");
    let mut file: File = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(
        vpk.tree.files.len(),
        449,
        "VPK tree should have 449 entries"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );

    let out_path = String::from("./test_files/out/pak01_dir.vpk");

    vpk.write_dir(&out_path).unwrap();

    let mut file: File = File::open(&out_path).expect("Failed to open file");
    let new_vpk = VPKVersion1::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(
        new_vpk.tree.files.len(),
        449,
        "VPK tree should have 449 entries"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );

    let _ = remove_file(out_path);
    let _ = remove_dir("./test_files/out");

    assert!(
        new_vpk == vpk,
        "Written VPK did not contain the same data as the original"
    );
}
