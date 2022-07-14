use std::{env, path};

use clap::{Parser, Subcommand};

use crate::error::Error;

///
/// The application arguments.
///
#[derive(Debug, Parser)]
#[clap(
    author = "Roman A. Voronkin",
    version,
    about = "A Very simple Notes Attendant"
)]
pub struct Arguments {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

///
/// The application command.
///
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Command {
    ///
    /// Repair notes set.
    ///
    Repair {
        ///
        /// Repair wiki references.
        ///
        #[clap(short = 'w', long = "wiki-refs", required = false, takes_value = false)]
        wiki_refs: bool,
    },
}

///
/// Конфигурация программы.
///
#[derive(Debug)]
pub struct Config {
    ///
    /// Корневой путь к набору заметок.
    ///
    root: path::PathBuf,
}

impl Config {
    ///
    /// Создать экземпляр конфигурации программы.
    ///
    pub fn new() -> Result<Self, Error> {
        let root = path::PathBuf::from(env::var("NOTES_ROOT")?);
        if !root.exists() || !root.is_dir() {
            return Err(Error::IllegalNotesRoot(root));
        }

        Ok(Self { root })
    }

    ///
    /// Получить корневой путь к набору заметок.
    ///
    #[inline]
    pub fn root(&self) -> &path::Path {
        self.root.as_path()
    }
}
