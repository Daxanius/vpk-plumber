use std::fs::File;

use vpk_plumber::pak::revpk::VPKRespawn;

use crate::common::{self, Result};

#[test]
fn valid_vpk_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_SINGLE_FILE)?;
    let _vpk = VPKRespawn::try_from(&mut file)?;
    Ok(())
}

#[test]
fn valid_vpk_large() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_TITANFALL)?;
    let _vpk = VPKRespawn::try_from(&mut file)?;
    Ok(())
}

#[test]
fn invalid_vpk() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_ARCHIVE)?;
    let vpk = VPKRespawn::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::InvalidSignature(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_v1_empty() -> Result<()> {
    let mut file = File::open(common::PAK_V1_EMPTY)?;
    let vpk = VPKRespawn::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_v1_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_V1_SINGLE_FILE)?;
    let vpk = VPKRespawn::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_v1_eof() -> Result<()> {
    let mut file = File::open(common::PAK_V1_SINGLE_FILE_EOF)?;
    let vpk = VPKRespawn::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_v2_empty() -> Result<()> {
    let mut file = File::open(common::PAK_V2_EMPTY)?;
    let vpk = VPKRespawn::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_v2_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_V1_SINGLE_FILE)?;
    let vpk = VPKRespawn::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}
