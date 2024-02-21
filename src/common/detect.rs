#[cfg(feature = "revpk")]
use crate::pak::revpk::format::VPKHeaderRespawn;
use crate::pak::{v1::format::VPKHeaderV1, v2::format::VPKHeaderV2};

use super::file::VPKFile;

#[derive(Debug, PartialEq)]
pub enum PakFormat {
    Unknown,
    VPKVersion1,
    VPKVersion2,
    VPKRespawn,
}

pub fn detect_pak_format(file: &mut VPKFile) -> PakFormat {
    if VPKHeaderV1::is_format(file) {
        return PakFormat::VPKVersion1;
    }
    if VPKHeaderV2::is_format(file) {
        return PakFormat::VPKVersion2;
    }
    #[cfg(feature = "revpk")]
    if VPKHeaderRespawn::is_format(file) {
        return PakFormat::VPKRespawn;
    }

    PakFormat::Unknown
}
