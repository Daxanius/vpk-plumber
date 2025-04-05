//! Common utilities for the library.
//!
//! Includes file handling, format detection, and, when the `revpk` feature is enabled, support for the [LZHAM alpha](https://github.com/richgel999/lzham_alpha) compression format.

pub use error::{Error, Result};

pub mod file;
#[cfg(feature = "revpk")]
pub mod lzham;

mod error;
