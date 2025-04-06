use std::{fs::File, io::Read};

use vpk_plumber::pak::{PakReader, v1::VPKVersion1};

use crate::common::{self, Result};

use filebuffer::FileBuffer;
use std::collections::HashMap;

#[test]
fn extract_vpk_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_V1_SINGLE_FILE)?;
    let vpk = VPKVersion1::try_from(&mut file)?;

    let mut archive_mmaps = HashMap::new();
    archive_mmaps.insert(0, FileBuffer::open(common::PAK_V1_ARCHIVE).unwrap());

    let out_path = tempfile::NamedTempFile::new()?;

    vpk.extract_file_mem_map(
        common::DIR_V1,
        &archive_mmaps,
        common::SINGLE_FILE_ARCHIVE,
        common::SINGLE_FILE_NAME,
        out_path.path().to_str().unwrap(),
    )?;

    let mut result = String::new();
    File::open(&out_path)?.read_to_string(&mut result)?;

    assert_eq!(
        result,
        common::SINGLE_FILE_CONTENT,
        "File contents should match {out_path:#?}",
    );

    Ok(())
}
