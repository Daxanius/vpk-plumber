use std::{fs::File, path::Path};

use vpk_plumber::pak::{PakWorker, PakWriter, v1::VPKVersion1};

use crate::common::{self, Result};

#[test]
fn empty() -> Result<()> {
    roundtrip(common::PAK_V1_EMPTY)
}

#[test]
fn single_file() -> Result<()> {
    roundtrip(common::PAK_V1_SINGLE_FILE)
}

#[test]
fn single_file_eof() -> Result<()> {
    roundtrip(common::PAK_V1_SINGLE_FILE_EOF)
}

#[test]
fn large() -> Result<()> {
    roundtrip(common::PAK_V1_PORTAL2)
}

fn roundtrip<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Read a vpk file
    let mut file = File::open(path)?;
    let vpk = VPKVersion1::from_file(&mut file)?;

    // Write it to a directory
    let out = tempfile::NamedTempFile::new()?;
    vpk.write_dir(out.path().to_str().unwrap())?;

    // Read it from the output and check if the result is the same
    let mut file = File::open(&out)?;
    let vpk_result = VPKVersion1::from_file(&mut file)?;

    assert_eq!(vpk.header, vpk_result.header, "Signatures do not match");

    assert_eq!(
        vpk.tree.files.len(),
        vpk_result.tree.files.len(),
        "VPK should contain a single file"
    );

    assert_eq!(
        vpk.tree.files.get(common::SINGLE_FILE_NAME),
        vpk_result.tree.files.get(common::SINGLE_FILE_NAME),
        "File data does not match"
    );

    Ok(())
}
