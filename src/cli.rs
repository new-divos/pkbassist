use clap::{Parser, Subcommand};

use crate::application::{twir, Application};

///
/// The application arguments.
///
#[derive(Debug, Parser)]
#[clap(
    author = Application::AUTHOR,
    version,
    about = Application::DESCRIPTION,
    long_about = None,
)]
#[clap(propagate_version = true)]
pub struct Arguments {
    ///
    /// The verbosity level.
    ///
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub(crate) verbosity: i32,

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
    /// Repair the notes set.
    ///
    Repair {
        ///
        /// Repair wiki references.
        ///
        #[clap(long = "wiki-refs", parse(from_flag))]
        wiki_refs: bool,

        ///
        /// Remove unused files.
        ///
        #[clap(long = "remove-unused-files", parse(from_flag))]
        remove_unused_files: bool,

        ///
        /// Rename attached files.
        ///
        #[clap(long = "rename-files", parse(from_flag))]
        rename_files: bool,
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
        #[clap(short = 'd', long = "update-daily", parse(from_flag))]
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
        #[clap(short = 'd', long = "update-daily", parse(from_flag))]
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
