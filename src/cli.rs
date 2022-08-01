use clap::{Parser, Subcommand};

use crate::application::twir;

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
        #[clap(long = "wiki-refs", required = false, takes_value = false)]
        wiki_refs: bool,

        ///
        /// Remove unused files.
        ///
        #[clap(long = "remove-unused-files", required = false, takes_value = false)]
        remove_unused_files: bool,
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

    #[clap(name = "twir")]
    TWiR {
        #[clap(
            short = 'i',
            long = "issue",
            required = true,
            takes_value = true,
            parse(try_from_str)
        )]
        issues: twir::Issues,

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
