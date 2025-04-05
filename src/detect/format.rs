use std::fmt;

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

        write!(f, "{str}")
    }
}
