use crate::common::file::{VPKFile, VPKFileReader};
use crate::common::format::{VPKDirectoryEntry, VPKTree};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

pub const VPK_SIGNATURE_V1: u32 = 0x55AA1234;
pub const VPK_VERSION_V1: u32 = 1;

pub struct VPKHeaderV1 {
    pub signature: u32,
    pub version: u32,

    // Size of the directory tree in bytes
    pub tree_size: u32,
}

impl VPKHeaderV1 {
    pub fn from(file: &mut VPKFile) -> Self {
        let signature = file
            .read_u32()
            .expect("Could not read header signature from file");
        let version = file
            .read_u32()
            .expect("Could not read header version from file");
        let tree_size = file
            .read_u32()
            .expect("Could not read header tree size from file");

        assert_eq!(
            signature, VPK_SIGNATURE_V1,
            "VPK header signature should be {:#x}",
            VPK_SIGNATURE_V1
        );
        assert_eq!(
            version, VPK_VERSION_V1,
            "VPK header version should be {}",
            VPK_VERSION_V1
        );

        Self {
            signature,
            version,
            tree_size,
        }
    }

    pub fn is_format(file: &mut VPKFile) -> bool {
        let pos = file.stream_position().unwrap();

        let signature = file.read_u32();
        let version = file.read_u32();

        let _ = file.seek(std::io::SeekFrom::Start(pos));

        signature.unwrap_or(0) == VPK_SIGNATURE_V1 && version.unwrap_or(0) == VPK_SIGNATURE_V1
    }
}

pub struct VPKVersion1 {
    pub header: VPKHeaderV1,
    pub tree: VPKTree<VPKDirectoryEntry>,
}

impl VPKVersion1 {
    pub fn new() -> Self {
        Self {
            header: VPKHeaderV1 {
                signature: VPK_SIGNATURE_V1,
                version: VPK_VERSION_V1,
                tree_size: 0,
            },
            tree: VPKTree::new(),
        }
    }

    pub fn from(file: &mut VPKFile) -> Self {
        let header = VPKHeaderV1::from(file);

        let tree_start = file.stream_position().unwrap();
        let tree = VPKTree::from(file, tree_start, header.tree_size.into());

        Self { header, tree }
    }

    pub fn read_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
    ) -> Option<Vec<u8>> {
        let entry = self.tree.files.get(file_path)?;
        let mut buf: Vec<u8> = Vec::new();

        if entry.preload_bytes > 0 {
            buf.append(self.tree.preload.get(file_path)?.clone().as_mut());
        }

        if entry.entry_length > 0 {
            let path = Path::new(archive_path).join(format!(
                "{}_{:0>3}.vpk",
                vpk_name,
                entry.archive_index.to_string()
            ));

            let mut archive_file = File::open(path).ok()?;

            let _ = archive_file.seek(SeekFrom::Start(entry.entry_offset as _));

            buf.append(
                archive_file
                    .read_bytes(entry.entry_length as _)
                    .ok()?
                    .as_mut(),
            );
        }

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();
        digest.update(&buf);

        assert_eq!(digest.finalize(), entry.crc, "CRC must match");

        Some(buf)
    }
}
