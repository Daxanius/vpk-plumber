use std::fmt;

use crate::util;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidEntryTerminator(String),
    InvalidSignature(String),
    TreeNotFound(std::io::Error),
    BadVersion(String),
    Io(std::io::Error),
    FileNotFound(String),
    Util {
        source: util::Error,
        context: String,
    },
    BadData(String),
    DataNotFound(String),
    MemoryMappedFileNotFound(u16),
    DataTooLarge,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl std::error::Error for Error {}
