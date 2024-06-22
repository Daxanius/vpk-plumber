//! sourcepak provides support for working with VPK (Valve Pak) files.
//! It allows you to read the directories of VPK files, as well as read their contents into memory or to disk.
//!
//! # Supported formats
//! | Format      | Support     | Game(s)                                                      |
//! | ----------- | ----------- | ------------------------------------------------------------ |
//! | VPK v1      | &#x1F7E2;   | Alien Swarm, Dota 2, L4D, L4D2, Portal 2, SFM                |
//! | VPK v2      | &#x1F7E1; * | CS:GO, CS:S, DoD:S, HL:S, HL2, HL2:DM, Portal, TF2, Source 2 |
//! | Respawn VPK | &#x1F7E2;   | Titanfall                                                    |
//!
//! * sourcepak doesn't currently support reading archive contents or writing directory files for VPK v2.
//!
//! # Features
//! - `revpk`: Add support for Respawn VPK files.
//! - `mem-map`: Use memory mapping to read VPK files. This can be faster and use less memory, but is not supported on all platforms.
//!
//! **Note:** Enabling the `revpk` feature requires additional dependencies (`lzham-alpha-sys`).
//!
//! **Note:** Enabling the `mem-map` feature requires additional dependencies (`filebuffer`).

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod common;
pub mod pak;

#[cfg(test)]
mod tests;
