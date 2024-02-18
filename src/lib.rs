pub mod common;
pub mod pak;

#[cfg(test)]
mod tests {
    #[cfg(feature = "revpk")]
    use crate::pak::revpk::format::{
        VPKDirectoryEntryRespawn, VPKFilePartEntryRespawn, VPKRespawn,
    };
    use crate::{
        common::format::{PakFormat, VPKDirectoryEntry},
        pak::{v1::format::VPKVersion1, v2::format::VPKVersion2},
    };
    use std::{fs::File, io::Seek, path::Path};

    #[test]
    fn read_empty_vpk_v1() {
        let path = Path::new("./test_files/empty_v1_dir.vpk");
        let mut file = File::open(path).expect("Failed to open file");
        let vpk = VPKVersion1::from(&mut file);
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
    fn read_single_file_vpk_v1() {
        let path = Path::new("./test_files/single_file_v1_dir.vpk");
        let mut file = File::open(path).expect("Failed to open file");
        let vpk = VPKVersion1::from(&mut file);
        assert_eq!(vpk.tree.files.len(), 1, "VPK tree should have 1 entry");
        assert_eq!(
            vpk.tree.files.get("test/file.txt"),
            Some(&VPKDirectoryEntry {
                crc: 0x4570FA16,
                preload_bytes: 0,
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
    }

    #[test]
    fn read_big_vpk_v1() {
        let path = Path::new("./test_files/portal/pak01_dir.vpk");
        let mut file: File = File::open(path).expect("Failed to open file");
        let vpk = VPKVersion1::from(&mut file);
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
    fn read_single_file_vpk_v2() {
        let path = Path::new("./test_files/single_file_v2_dir.vpk");
        let mut file: File = File::open(path).expect("Failed to open file");
        let vpk = VPKVersion2::from(&mut file);
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
        let vpk = VPKVersion2::from(&mut file);
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

    #[cfg(feature = "revpk")]
    #[test]
    fn read_single_file_vpk_revpk() {
        let path = Path::new("./test_files/single_file_revpk_dir.vpk");
        let mut file = File::open(path).expect("Failed to open file");
        let vpk = VPKRespawn::from(&mut file);

        assert_eq!(vpk.tree.files.len(), 1, "VPK tree should have 1 entry");

        let mut dir_entry = VPKDirectoryEntryRespawn {
            crc: 0x4570FA16,
            preload_bytes: 0,
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
    }

    #[cfg(feature = "revpk")]
    #[test]
    fn read_big_vpk_revpk() {
        use crate::common::file::{VPKFile, VPKFileReader};

        let path = Path::new("./test_files/titanfall/englishclient_mp_colony.bsp.pak000_dir.vpk");
        let mut file = File::open(path).expect("Failed to open file");
        let vpk = VPKRespawn::from(&mut file);
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
                VPKFile::open(Path::new("./test_files/titanfall/mp_colony.txt"))
                    .unwrap()
                    .read_string()
                    .unwrap()
                    .into()
            ),
            "File contents should match ./test_files/titanfall/mp_colony.txt"
        );
    }
}
