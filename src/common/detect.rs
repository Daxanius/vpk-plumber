use std::{fmt, fs::File};

#[cfg(feature = "revpk")]
use crate::pak::revpk::format::VPKHeaderRespawn;
use crate::pak::{v1::format::VPKHeaderV1, v2::format::VPKHeaderV2};

#[derive(PartialEq)]
pub enum PakFormat {
    Unknown,
    VPKVersion1,
    VPKVersion2,
    VPKRespawn,
}

impl fmt::Display for PakFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            PakFormat::Unknown => "Unknown",
            PakFormat::VPKVersion1 => "VPK Version 1",
            PakFormat::VPKVersion2 => "VPK Version 2",
            PakFormat::VPKRespawn => "VPK Respawn",
        };
        
        write!(f, "{}", str)
    }
}

pub fn detect_pak_format(file: &mut File) -> PakFormat {
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
