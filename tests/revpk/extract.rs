use std::{fs::File, io::Read};

use vpk_plumber::pak::{PakReader, revpk::VPKRespawn};

use crate::common::{self, Result};

use filebuffer::FileBuffer;
use std::collections::HashMap;

#[test]
fn vpk_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_SINGLE_FILE)?;
    let vpk = VPKRespawn::try_from(&mut file)?;

    let out_path = tempfile::NamedTempFile::new()?;

    vpk.extract_file(
        common::DIR_REVPK,
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
#[test]
fn vpk_single_file_mem_map() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_SINGLE_FILE)?;
    let vpk = VPKRespawn::try_from(&mut file)?;

    let mut archive_mmaps = HashMap::new();
    archive_mmaps.insert(0, FileBuffer::open(common::PAK_REVPK_ARCHIVE).unwrap());

    let out_path = tempfile::NamedTempFile::new()?;

    vpk.extract_file_mem_map(
        common::DIR_REVPK,
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
        "File contents should match",
    );

    Ok(())
}
