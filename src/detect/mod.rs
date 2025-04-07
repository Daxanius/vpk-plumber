//! This module contains functionality for detecting VPK formats

use crate::pak::PakWorker;
use crate::pak::{
    v1::{VPKHeaderV1, VPKVersion1},
    v2::{VPKHeaderV2, VPKVersion2},
};
use std::fs::File;

#[cfg(feature = "revpk")]
use crate::pak::revpk::{VPKHeaderRespawn, VPKRespawn};

pub use error::{Error, Result};
pub use format::PakFormat;

mod error;
mod format;

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

/// Detects the correct VPK format to use and returns
/// the appropriate `PakWorker` to work with the format.
/// # Errors
/// - When the format is unknown
/// - When the file data is invalid
pub fn find_pak_worker(file: &mut File) -> Result<Box<dyn PakWorker>> {
    match detect_pak_format(file) {
        PakFormat::VPKVersion1 => {
            let packager = VPKVersion1::from_file(file).map_err(Error::Pak)?;
            Ok(Box::new(packager))
        }

        PakFormat::VPKVersion2 => {
            let packager = VPKVersion2::from_file(file).map_err(Error::Pak)?;
            Ok(Box::new(packager))
        }

        #[cfg(feature = "revpk")]
        PakFormat::VPKRespawn => {
            let packager = VPKRespawn::from_file(file).map_err(Error::Pak)?;
            Ok(Box::new(packager))
        }

        _ => Err(Error::UnknownFormat), // Handle other cases
    }
}
