//! Utilities to detect which format a VPK file is in.

use std::{fmt, fs::File};

#[cfg(feature = "revpk")]
use crate::pak::revpk::format::VPKHeaderRespawn;
use crate::pak::{v1::format::VPKHeaderV1, v2::format::VPKHeaderV2};

/// Lists the different formats of VPK files.
#[derive(PartialEq)]
pub enum PakFormat {
    /// Unknown format.
    Unknown,
    /// VPK version 1.
    VPKVersion1,
    /// VPK version 2.
    VPKVersion2,
    /// Respawn VPK.
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

/// Detects the format of a VPK file by reading its header.
/// Leaves the file cursor in the position it was at when the function was called.
///
/// This calls the `is_format` method of each VPK header format until it finds a match.
///
/// *Will not test for the Respawn VPK format if the `revpk` feature is not enabled.*
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
