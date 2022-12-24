use std::{env, io, path};

use thiserror::Error;

///
/// Ошибка работы программы.
///
#[derive(Debug, Error)]
pub enum Error {
    #[error("application initialization error")]
    AppInitError,

    #[error("cannot find the configuration file")]
    CannotFindConfig,

    #[error("illegal issue number {0}")]
    IllegalIssue(String),

    #[error("the vault root path is not set")]
    VaultRootIsAbsent,

    #[error("illegal vault root path {0}")]
    IllegalVaultRoot(path::PathBuf),

    #[error("illegal NASA Astronomy Picture of the Day API key")]
    IllegalNASAKey,

    #[error("illegal URL: {0}")]
    IllegalURL(url::Url),

    #[error("illegal HTML content")]
    IllegalHTMLContent,

    #[error("unknown media type")]
    UnknownMediaType,

    #[error("illegal year number {0}")]
    IllegalYearNumber(i32),

    #[error("illegal month number {0}")]
    IllegalMonthNumber(u32),

    #[error("illegal path {0}")]
    IllegalPath(String),

    #[error("illegal configuration key {0}")]
    IllegalConfKey(String),

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

    #[error("configuration parsing error {0}")]
    ConfigParseError(#[from] toml::de::Error),

    #[error("configuration serialization error {0}")]
    ConfigSerializeError(#[from] toml::ser::Error),

    #[error("logger initialization error {0}")]
    InitLoggerError(#[from] fern::InitError),

    #[error("set logger error {0}")]
    SetLoggerError(#[from] log::SetLoggerError),
}
