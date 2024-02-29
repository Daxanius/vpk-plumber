//! Common utilities for the library.
//!
//! Includes file handling, format detection, and, when the `revpk` feature is enabled, support for the [LZHAM alpha](https://github.com/richgel999/lzham_alpha) compression format.

pub mod detect;
pub mod file;
pub mod format;
#[cfg(feature = "revpk")]
pub mod lzham;
