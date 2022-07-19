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
    about = "A Very simple Notes Attendant",
    long_about = None,
)]
#[clap(propagate_version = true)]
pub struct Arguments {
    ///
    /// The application command.
    ///
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

    ///
    /// Grab notes to the notes set.
    ///
    Grab {
        #[clap(subcommand)]
        note: Note,
    },

    ///
    /// Show the additional information.
    ///
    Show {
        #[clap(subcommand)]
        info: Info,
    },
}

///
/// The application grab command object.
///
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Note {
    ///
    /// Grab NASA Astronomy Picture of the Day to the notes set.
    ///
    #[clap(name = "apod")]
    APoD {
        ///
        /// Update daily note in notes set.
        ///
        #[clap(
            short = 'd',
            long = "update-daily",
            required = false,
            takes_value = false
        )]
        update_daily: bool,
    },
}

///
/// The application show command object.
///
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Info {
    ///
    /// Show This Week in Rust issues.
    ///
    #[clap(name = "twir")]
    TWiR {
        ///
        /// Show only the last issue.
        ///
        #[clap(short = 'l', long = "last", required = false, takes_value = false)]
        last: bool,
    },
}

///
/// The application configuration.
///
#[derive(Debug)]
pub struct Config {
    ///
    /// The root directory of the notes set.
    ///
    root: path::PathBuf,

    ///
    /// NASA Astronomy Picture of the Day API Key.
    ///
    nasa_key: Option<String>,
}

impl Config {
    ///
    /// New instance of the application configuration.
    ///
    pub fn new() -> Result<Self, Error> {
        let root = path::PathBuf::from(env::var("NOTES_ROOT")?);
        if !root.exists() || !root.is_dir() {
            return Err(Error::IllegalNotesRoot(root));
        }

        Ok(Self {
            root,
            nasa_key: env::var("NASA_KEY").ok(),
        })
    }

    ///
    /// Get the root directory of the notes set.
    ///
    #[inline]
    pub fn root(&self) -> &path::Path {
        self.root.as_path()
    }

    ///
    /// Get NASA Astronomy Picture of the Day API Key.
    ///
    #[inline]
    pub fn nasa_key(&self) -> Option<&str> {
        self.nasa_key.as_deref()
    }
}
