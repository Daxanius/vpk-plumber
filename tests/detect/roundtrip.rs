use std::{fs::File, path::Path};

use vpk_plumber::detect::{self, PakFormat};

use crate::common::{self, Result};

#[test]
fn empty_v1() -> Result<()> {
    roundtrip(common::PAK_V1_EMPTY, &PakFormat::VPKVersion1)
}

#[test]
fn single_file_v1() -> Result<()> {
    roundtrip(common::PAK_V1_SINGLE_FILE, &PakFormat::VPKVersion1)
}

#[test]
fn single_file_eof_v1() -> Result<()> {
    roundtrip(common::PAK_V1_SINGLE_FILE_EOF, &PakFormat::VPKVersion1)
}

#[test]
fn large_v1() -> Result<()> {
    roundtrip(common::PAK_V1_PORTAL2, &PakFormat::VPKVersion1)
}

#[ignore = "not yet implemented"]
#[test]
fn empty_v2() -> Result<()> {
    roundtrip(common::PAK_V2_EMPTY, &PakFormat::VPKVersion2)
}

#[ignore = "not yet implemented"]
#[test]
fn single_file_v2() -> Result<()> {
    roundtrip(common::PAK_V2_SINGLE_FILE, &PakFormat::VPKVersion2)
}

#[ignore = "not yet implemented"]
#[test]
fn large_v2() -> Result<()> {
    roundtrip(common::PAK_V2_PORTAL, &PakFormat::VPKVersion2)
}

#[test]
fn single_file_revpk() -> Result<()> {
    roundtrip(common::PAK_REVPK_SINGLE_FILE, &PakFormat::VPKRespawn)
}

#[test]
fn large_revpk() -> Result<()> {
    roundtrip(common::PAK_REVPK_TITANFALL, &PakFormat::VPKRespawn)
}

fn roundtrip<P>(path: P, expected_format: &PakFormat) -> Result<()>
where
    P: AsRef<Path>,
{
    // Read a vpk file
    let mut file = File::open(path)?;
    let vpk = detect::find_pak_worker(&mut file)?;

    // Write it to a directory
    let out = tempfile::NamedTempFile::new()?;
    vpk.write_dir(out.path().to_str().unwrap())?;

    // Read it from the output and check if the result is the same
    let mut file = File::open(&out)?;
    let format = detect::detect_pak_format(&mut file);

    assert_eq!(
        format, *expected_format,
        "Format does not match expected format!"
    );

    Ok(())
}
