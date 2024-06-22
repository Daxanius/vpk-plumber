use crate::common::format::PakWriter;
use crate::common::{file::VPKFileReader, format::PakReader};
use crate::pak::revpk::format::{VPKDirectoryEntryRespawn, VPKFilePartEntryRespawn, VPKRespawn};

use std::{
    fs::{remove_dir, remove_file, File},
    io::Seek,
    path::Path,
};

use crate::pak::revpk::{
    cam::create_wav_header,
    format::{VPKRespawnCam, VPKRespawnCamEntry},
};

#[cfg(feature = "mem-map")]
use filebuffer::FileBuffer;
#[cfg(feature = "mem-map")]
use std::collections::HashMap;

#[test]
fn read_single_file_vpk_revpk() {
    let path = Path::new("./test_files/single_file_revpk_dir.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKRespawn::try_from(&mut file).expect("Failed to read VPK file");

    assert_eq!(vpk.tree.files.len(), 1, "VPK tree should have 1 entry");

    let mut dir_entry = VPKDirectoryEntryRespawn {
        crc: 0x4570FA16,
        preload_length: 0,
        file_parts: Vec::new(),
    };
    dir_entry.file_parts.push(VPKFilePartEntryRespawn {
        archive_index: 0,
        load_flags: 0,
        texture_flags: 0,
        entry_offset: 0,
        entry_length: 9,
        entry_length_uncompressed: 9,
    });
    assert_eq!(
        vpk.tree.files.get("test/file.txt"),
        Some(&dir_entry),
        "File \"test/file.txt\" should exist"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );

    let test_file = vpk.read_file(
        &String::from("./test_files"),
        &String::from("single_file_revpk"),
        &String::from("test/file.txt"),
    );
    assert_eq!(
        test_file,
        Some(Vec::from("test text")),
        "File contents should be \"test text\""
    );

    let out_path = String::from("./test_files/out/file_revpk.txt");

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

#[cfg(feature = "mem-map")]
#[test]
fn extract_mem_map_single_file_vpk_revpk() {
    let path = Path::new("./test_files/single_file_revpk_dir.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKRespawn::try_from(&mut file).expect("Failed to read VPK file");

    let mut archive_mmaps = HashMap::new();
    archive_mmaps.insert(
        0,
        FileBuffer::open("./test_files/single_file_revpk_000.vpk").unwrap(),
    );

    let out_path = String::from("./test_files/out/file_mem_map_revpk.txt");

    let _ = vpk.extract_file_mem_map(
        &String::from("./test_files"),
        &archive_mmaps,
        &String::from("single_file_revpk"),
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
fn read_big_vpk_revpk() {
    let path = Path::new("./test_files/titanfall/englishclient_mp_colony.bsp.pak000_dir.vpk");
    let mut file = File::open(path).expect("Failed to open file");
    let vpk = VPKRespawn::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(
        vpk.tree.files.len(),
        5723,
        "VPK tree should have 5723 entries"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );

    let test_file = vpk.read_file(
        &String::from("./test_files/titanfall"),
        &String::from("client_mp_colony.bsp.pak000"),
        &String::from("resource/overviews/mp_colony.txt"),
    );

    assert_eq!(
        test_file,
        Some(
            File::open(Path::new("./test_files/titanfall/mp_colony.txt"))
                .unwrap()
                .read_string()
                .unwrap()
                .into()
        ),
        "File contents should match ./test_files/titanfall/mp_colony.txt"
    );
}

#[test]
fn revpk_read_cam() {
    let path = Path::new("./test_files/titanfall/client_mp_common.bsp.pak000_000.vpk.cam");

    let cam = VPKRespawnCam::from_file(&mut File::open(path).unwrap()).unwrap();

    assert_eq!(cam.entries.len(), 17852, "Should have 17852 entries");

    assert_eq!(
        cam.find_entry(10688756183),
        Some(&VPKRespawnCamEntry {
            magic: 3302889984,
            original_size: 315436,
            compressed_size: 29547,
            sample_rate: 44100,
            channels: 1,
            sample_count: 157658,
            header_size: 44,
            vpk_content_offset: 10688756183,
        }),
        "Entry with vpk content offset 10688756183 should exist",
    );

    let wav_header = create_wav_header(cam.find_entry(10688756183).unwrap());

    assert_eq!(
        wav_header,
        [
            82, 73, 70, 70, 216, 207, 4, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0,
            1, 0, 68, 172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0, 100, 97, 116, 97, 180, 207, 4, 0,
        ]
        .to_vec(),
        "WAV header should match",
    );
}

#[test]
fn revpk_read_cam_for_vpk() {
    let mut vpk = VPKRespawn::new();

    let cam_path = String::from("./test_files/titanfall/client_mp_common.bsp.pak000_000.vpk.cam");

    vpk.read_cam(0, &cam_path).unwrap();

    let cam_archive0_opt = vpk.archive_cams.get(&0);
    let cam_archive1_opt = vpk.archive_cams.get(&1);

    assert!(cam_archive0_opt.is_some());
    assert!(cam_archive1_opt.is_none());

    let cam = cam_archive0_opt.unwrap();

    assert_eq!(cam.entries.len(), 17852, "Should have 17852 entries");

    assert_eq!(
        cam.find_entry(10688756183),
        Some(&VPKRespawnCamEntry {
            magic: 3302889984,
            original_size: 315436,
            compressed_size: 29547,
            sample_rate: 44100,
            channels: 1,
            sample_count: 157658,
            header_size: 44,
            vpk_content_offset: 10688756183,
        }),
        "Entry with vpk content offset 10688756183 should exist",
    );

    let wav_header = create_wav_header(cam.find_entry(10688756183).unwrap());

    assert_eq!(
        wav_header,
        [
            82, 73, 70, 70, 216, 207, 4, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0,
            1, 0, 68, 172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0, 100, 97, 116, 97, 180, 207, 4, 0,
        ]
        .to_vec(),
        "WAV header should match",
    );
}

#[test]
fn write_parity_vpk_revpk() {
    let path = Path::new("./test_files/titanfall/englishclient_mp_colony.bsp.pak000_dir.vpk");
    let mut file: File = File::open(path).expect("Failed to open file");
    let vpk = VPKRespawn::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(
        vpk.tree.files.len(),
        5723,
        "VPK tree should have 5723 entries"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );

    let out_path = String::from("./test_files/out/englishclient_mp_colony.bsp.pak000_dir.vpk");

    vpk.write_dir(&out_path).unwrap();

    let mut file: File = File::open(&out_path).expect("Failed to open file");
    let new_vpk = VPKRespawn::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(
        new_vpk.tree.files.len(),
        5723,
        "VPK tree should have 5723 entries"
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