use std::{env, io, path};

use thiserror::Error;

///
/// Ошибка работы программы.
///
#[derive(Debug, Error)]
pub enum Error {
    #[error("illegal notes root {0}")]
    IllegalNotesRoot(path::PathBuf),

    #[error("illegal NASA Astronomy Picture of the Day API key")]
    IllegalNASAKey,

    #[error("illegal URL: {0}")]
    IllegalURL(url::Url),

    #[error("unknown media type")]
    UnknownMediaType,

    #[error("found {0:?} failed executors")]
    MultipleExecutorsError(Vec<Error>),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("environment variable error: {0}")]
    VarError(#[from] env::VarError),

    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    URLParseError(#[from] url::ParseError),
}
