use std::{fs::File, io::Read};

use vpk_plumber::pak::{PakReader, v2::VPKVersion2};

use crate::common::{self, Result};

use filebuffer::FileBuffer;
use std::collections::HashMap;

#[ignore = "not yet implemented"]
#[test]
fn vpk_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_V2_SINGLE_FILE)?;
    let vpk = VPKVersion2::try_from(&mut file)?;

    let out_path = tempfile::NamedTempFile::new()?;

    vpk.extract_file(
        common::DIR_V2,
        common::SINGLE_FILE_ARCHIVE,
        common::SINGLE_FILE_NAME,
        out_path.path().to_str().unwrap(),
    )?;

    let mut result = String::new();
    File::open(&out_path)?.read_to_string(&mut result)?;

    assert_eq!(
        result,
        common::SINGLE_FILE_CONTENT,
        "File contents should match",
    );
    Ok(())
}

#[cfg(feature = "mem-map")]
#[ignore = "not yet implemented"]
#[test]
fn vpk_single_file_mem_map() -> Result<()> {
    let mut file = File::open(common::PAK_V2_SINGLE_FILE)?;
    let vpk = VPKVersion2::try_from(&mut file)?;

    let mut archive_mmaps = HashMap::new();
    archive_mmaps.insert(0, FileBuffer::open(common::PAK_V2_ARCHIVE).unwrap());

    let out_path = tempfile::NamedTempFile::new()?;

    vpk.extract_file_mem_map(
        common::DIR_V2,
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
