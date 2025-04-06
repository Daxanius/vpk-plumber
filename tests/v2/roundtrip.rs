use std::{fs::File, path::Path};

use vpk_plumber::pak::{PakWorker, PakWriter, v2::VPKVersion2};

use crate::common::{self, Result};

#[ignore = "not yet implemented"]
#[test]
fn empty() -> Result<()> {
    roundtrip(common::PAK_V2_EMPTY)
}

#[ignore = "not yet implemented"]
#[test]
fn single_file() -> Result<()> {
    roundtrip(common::PAK_V2_SINGLE_FILE)
}

#[ignore = "not yet implemented"]
#[test]
fn large() -> Result<()> {
    roundtrip(common::PAK_V2_PORTAL)
}

fn roundtrip<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Read a vpk file
    let mut file = File::open(path)?;
    let vpk = VPKVersion2::from_file(&mut file)?;

    // Write it to a directory
    let out = tempfile::NamedTempFile::new()?;
    vpk.write_dir(out.path().to_str().unwrap())?;

    // Read it from the output and check if the result is the same
    let mut file = File::open(&out)?;
    let vpk_result = VPKVersion2::from_file(&mut file)?;

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
