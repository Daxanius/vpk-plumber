use std::fs::File;

use vpk_plumber::pak::v2::VPKVersion2;

use crate::common::{self, Result};

#[test]
fn valid_vpk_empty() -> Result<()> {
    let mut file = File::open(common::PAK_V2_EMPTY)?;
    let _vpk = VPKVersion2::try_from(&mut file)?;
    Ok(())
}

#[test]
fn valid_vpk_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_V2_SINGLE_FILE)?;
    let _vpk = VPKVersion2::try_from(&mut file)?;
    Ok(())
}

#[test]
fn valid_vpk_large() -> Result<()> {
    let mut file = File::open(common::PAK_V2_PORTAL)?;
    let _vpk = VPKVersion2::try_from(&mut file)?;
    Ok(())
}

#[test]
fn invalid_vpk() -> Result<()> {
    let mut file = File::open(common::PAK_V2_ARCHIVE)?;
    let vpk = VPKVersion2::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::InvalidSignature(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_v2_empty() -> Result<()> {
    let mut file = File::open(common::PAK_V1_EMPTY)?;
    let vpk = VPKVersion2::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_v2_single_file() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_SINGLE_FILE)?;
    let vpk = VPKVersion2::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}

#[test]
fn vpk_revpk_empty() -> Result<()> {
    let mut file = File::open(common::PAK_REVPK_SINGLE_FILE)?;
    let vpk = VPKVersion2::try_from(&mut file);

    assert!(
        vpk.is_err_and(|x| matches!(x, vpk_plumber::pak::Error::BadVersion(_))),
        "VPK file should be invalid",
    );

    Ok(())
}
