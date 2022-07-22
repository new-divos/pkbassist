use std::{env, io, path};

use thiserror::Error;

///
/// Ошибка работы программы.
///
#[derive(Debug, Error)]
pub enum Error {
    #[error("illegal issue number {0}")]
    IllegalIssue(String),

    #[error("illegal notes root {0}")]
    IllegalNotesRoot(path::PathBuf),

    #[error("illegal NASA Astronomy Picture of the Day API key")]
    IllegalNASAKey,

    #[error("illegal URL: {0}")]
    IllegalURL(url::Url),

    #[error("illegal HTML content")]
    IllegalHTMLContent,

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

    #[error("URL parsing error: {0}")]
    URLParseError(#[from] url::ParseError),

    #[error("datetime parsing error: {0}")]
    DateTimeParseError(#[from] chrono::ParseError),
}
