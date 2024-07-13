//! Main Crate Error

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error("{}", .1)]
    ParseError(PathBuf, String),
}
