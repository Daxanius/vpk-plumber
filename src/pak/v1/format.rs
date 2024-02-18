use crate::common::file::{VPKFile, VPKFileReader};
use crate::common::format::{VPKDirectoryEntry, PakFormat, VPKTree};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
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

impl PakFormat for VPKVersion1 {
    fn new() -> Self {
        Self {
            header: VPKHeaderV1 {
                signature: VPK_SIGNATURE_V1,
                version: VPK_VERSION_V1,
                tree_size: 0,
            },
            tree: VPKTree::new(),
        }
    }

    fn from_file(file: &mut VPKFile) -> Self {
        let header = VPKHeaderV1::from(file);

        let tree_start = file.stream_position().unwrap();
        let tree = VPKTree::from(file, tree_start, header.tree_size.into());

        Self { header, tree }
    }

    fn read_file(
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

    fn extract_file(
        self: &Self,
        archive_path: &String,
        vpk_name: &String,
        file_path: &String,
        output_path: &String,
    ) -> Result<(), String> {
        let entry = self
            .tree
            .files
            .get(file_path)
            .ok_or("File not found in VPK")?;
        let mut buf: Vec<u8> = Vec::new();

        if entry.preload_bytes > 0 {
            buf.append(
                self.tree
                    .preload
                    .get(file_path)
                    .ok_or("Preload data not found in VPK")?
                    .clone()
                    .as_mut(),
            );
        }

        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        if entry.entry_length > 0 {
            let path = Path::new(archive_path).join(format!(
                "{}_{:0>3}.vpk",
                vpk_name,
                entry.archive_index.to_string()
            ));

            let mut archive_file = File::open(path).or(Err("Failed to open archive file"))?;

            let _ = archive_file.seek(SeekFrom::Start(entry.entry_offset as _));

            let out_path = std::path::Path::new(output_path);
            if let Some(prefix) = out_path.parent() {
                std::fs::create_dir_all(prefix).or(Err("Failed to create parent directories"))?;
            };

            let mut out_file = File::create(out_path).or(Err("Failed to create output file"))?;

            // read chunks of 1MB max into buffer and write to the output file
            let mut remaining = entry.entry_length as usize;
            while remaining > 0 {
                let chunk = archive_file
                    .read_bytes(1024 * 1024)
                    .or(Err("Failed to read from archive file"))?;
                if chunk.len() == 0 {
                    return Err("Failed to read from archive file".to_string());
                }
                out_file
                    .write_all(&chunk)
                    .or(Err("Failed to write to output file"))?;
                remaining -= chunk.len();

                digest.update(&chunk);
            }
        }

        assert_eq!(digest.finalize(), entry.crc, "CRC must match");

        Ok(())
    }
}

impl From<&mut VPKFile> for VPKVersion1 {
    fn from(file: &mut VPKFile) -> Self {
        Self::from_file(file)
    }
}
