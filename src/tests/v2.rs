use crate::pak::v2::format::VPKVersion2;
use std::{fs::File, io::Seek, path::Path};

#[test]
fn read_single_file_vpk_v2() {
    let path = Path::new("./test_files/single_file_v2_dir.vpk");
    let mut file: File = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion2::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(vpk.tree.files.len(), 1, "VPK tree should have 1 entry");
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );
}

#[test]
fn read_big_vpk_v2() {
    let path = Path::new("./test_files/tf2/tf2_sound_misc_dir.vpk");
    let mut file: File = File::open(path).expect("Failed to open file");
    let vpk = VPKVersion2::try_from(&mut file).expect("Failed to read VPK file");
    assert_eq!(
        vpk.tree.files.len(),
        3230,
        "VPK tree should have 3230 entries"
    );
    assert!(
        file.stream_position().unwrap() >= file.seek(std::io::SeekFrom::End(0)).unwrap() - 1,
        "Should be at end of file"
    );
}
