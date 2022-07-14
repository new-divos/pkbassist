use std::{env, io, path};

use thiserror::Error;

///
/// Ошибка работы программы.
///
#[derive(Debug, Error)]
pub enum Error {
    #[error("illegal notes root {0}")]
    IllegalNotesRoot(path::PathBuf),

    #[error("found {0:?} failed executors")]
    MultipleExecutorsError(Vec<Error>),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("environment variable error: {0}")]
    VarError(#[from] env::VarError),
}
