use std::fs::File;

use vpk_plumber::pak::{PakReader, v1::VPKVersion1};

use crate::common::{self, Result};

#[test]
fn vpk_empty() -> Result<()> {
    let mut file = File::open(common::PAK_V1_EMPTY)?;
    let vpk = VPKVersion1::try_from(&mut file)?;

    let result = vpk.read_file(
        &String::from(common::DIR_V1),
        &String::from(common::SINGLE_FILE_ARCHIVE),
        &String::from(common::SINGLE_FILE_NAME),
    );

    assert!(result.is_none(), "File should not exist in empty archive");
    Ok(())
}

#[test]
fn vpk_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_V1_SINGLE_FILE)?;
    let vpk = VPKVersion1::try_from(&mut file)?;

    let result = vpk
        .read_file(
            &String::from(common::DIR_V1),
            &String::from(common::SINGLE_FILE_ARCHIVE),
            &String::from(common::SINGLE_FILE_NAME),
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
    let mut file = File::open(common::PAK_V1_PORTAL2)?;
    let vpk = VPKVersion1::try_from(&mut file)?;

    assert_eq!(
        vpk.tree.files.len(),
        common::PORTAL2_TREE_COUNT,
        "Tree size does not match"
    );

    Ok(())
}
