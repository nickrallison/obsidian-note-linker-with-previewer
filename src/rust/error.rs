//! Main Crate Error

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("parse failure on file: {}", .0.display())]
    ParseError(PathBuf, String),
}
