use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::util;

#[derive(Debug)]
pub enum FileErrorKind {
    NotFound,
    Other(io::Error),
}

impl From<io::Error> for FileErrorKind {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound,
            _ => Self::Other(error),
        }
    }
}

#[derive(Debug, Error)]
pub struct FileError {
    pub kind: FileErrorKind,
    pub path: PathBuf,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            FileErrorKind::NotFound => write!(f, "File not found: {}", self.path.display()),
            FileErrorKind::Other(err) => write!(f, "{}", err),
        }
    }
}

pub fn create_file(path: impl AsRef<Path>) -> Result<fs::File, FileError> {
    let path = path.as_ref();

    fs::File::create(path).map_err(|err| FileError {
        kind: err.into(),
        path: util::normalize_path(path),
    })
}

pub fn open_file(path: impl AsRef<Path>) -> Result<fs::File, FileError> {
    let path = path.as_ref();

    fs::File::open(path).map_err(|err| FileError {
        kind: err.into(),
        path: util::normalize_path(path),
    })
}

pub fn create_parent_dir(path: impl AsRef<Path>) -> io::Result<()> {
    if let Some(parent_dir_path) = path.as_ref().parent() {
        fs::create_dir_all(parent_dir_path)?;
    }

    Ok(())
}
