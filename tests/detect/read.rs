use std::{fs::File, path::Path};

use vpk_plumber::detect::{self, PakFormat};

use crate::common::{self, Result};

#[test]
fn invalid() -> Result<()> {
    assert_format(common::PAK_V1_ARCHIVE, &PakFormat::Unknown)?;
    assert_format(common::PAK_V2_ARCHIVE, &PakFormat::Unknown)?;
    assert_format(common::PAK_REVPK_ARCHIVE, &PakFormat::Unknown)?;
    Ok(())
}

#[test]
fn empty_v1() -> Result<()> {
    assert_format(common::PAK_V1_EMPTY, &PakFormat::VPKVersion1)
}

#[test]
fn single_file_v1() -> Result<()> {
    assert_format(common::PAK_V1_SINGLE_FILE, &PakFormat::VPKVersion1)
}

#[test]
fn single_file_eof_v1() -> Result<()> {
    assert_format(common::PAK_V1_SINGLE_FILE_EOF, &PakFormat::VPKVersion1)
}

#[test]
fn large_v1() -> Result<()> {
    assert_format(common::PAK_V1_PORTAL2, &PakFormat::VPKVersion1)
}

#[test]
fn empty_v2() -> Result<()> {
    assert_format(common::PAK_V2_EMPTY, &PakFormat::VPKVersion2)
}

#[test]
fn single_file_v2() -> Result<()> {
    assert_format(common::PAK_V2_SINGLE_FILE, &PakFormat::VPKVersion2)
}

#[test]
fn large_v2() -> Result<()> {
    assert_format(common::PAK_V2_PORTAL, &PakFormat::VPKVersion2)
}

#[test]
fn single_file_revpk() -> Result<()> {
    assert_format(common::PAK_REVPK_SINGLE_FILE, &PakFormat::VPKRespawn)
}

#[test]
fn large_revpk() -> Result<()> {
    assert_format(common::PAK_REVPK_TITANFALL, &PakFormat::VPKRespawn)
}

fn assert_format<P>(path: P, expected_format: &PakFormat) -> Result<()>
where
    P: AsRef<Path>,
{
    // Read a vpk file
    let mut file = File::open(path)?;
    let format = detect::detect_pak_format(&mut file);

    assert_eq!(
        format, *expected_format,
        "Format does not match expected format!"
    );

    Ok(())
}
