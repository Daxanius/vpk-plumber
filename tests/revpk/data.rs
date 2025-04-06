use std::fs::File;

use vpk_plumber::pak::{PakReader, revpk::VPKRespawn};

use crate::common::{self, Result};

#[ignore = "no test vpk"]
#[test]
fn vpk_empty() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_SINGLE_FILE)?;
    let vpk = VPKRespawn::try_from(&mut file)?;

    let result = vpk.read_file(
        common::DIR_REVPK,
        common::SINGLE_FILE_ARCHIVE,
        common::SINGLE_FILE_NAME,
    );

    assert!(result.is_none(), "File should not exist in empty archive");
    Ok(())
}

#[test]
fn vpk_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_SINGLE_FILE)?;
    let vpk = VPKRespawn::try_from(&mut file)?;

    let result = vpk
        .read_file(
            common::DIR_REVPK,
            common::SINGLE_FILE_ARCHIVE,
            common::SINGLE_FILE_NAME,
        )
        .unwrap();

    assert_eq!(
        result,
        common::SINGLE_FILE_CONTENT.as_bytes(),
        "Content does not match expected"
    );

    Ok(())
}

#[test]
fn vpk_large() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_TITANFALL)?;
    let vpk = VPKRespawn::try_from(&mut file)?;

    assert_eq!(
        vpk.tree.files.len(),
        common::TITANFALL_TREE_COUNT,
        "Tree size does not match"
    );

    Ok(())
}
