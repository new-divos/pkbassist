use chrono::Datelike;
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

    ///
    /// Add the additional information to the notes set.
    ///
    Add {
        #[clap(subcommand)]
        annex: Annex,
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
        /// Show the only last issue.
        ///
        #[clap(short = 'l', long = "last", required = false, takes_value = false)]
        last: bool,
    },
}

///
/// The application add command object.
///
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Annex {
    ///
    /// Add the calendar to the monthly note.
    ///
    #[clap(name = "calendar")]
    Calendar {
        ///
        /// The year number.
        ///
        #[clap(
            default_value_t = chrono::offset::Local::today().year(), 
            short = 'y', 
            long = "year"
        )]
        year: i32,

        ///
        /// The month number.
        /// 
        #[clap(
            default_value_t = chrono::offset::Local::today().month(), 
            short = 'm', 
            long = "month"
        )]
        month: u32,
    },

    ///
    /// Add the reference bar to the note.
    /// 
    #[clap (name = "refbar")]
    RefBar {
        ///
        /// The note name.
        /// 
        #[clap(required = true, takes_value = true, short = 'n', long = "note")]
        note: String,

        ///
        /// The references list.
        /// 
        #[clap(required = false, short = 'r', long = "ref")]
        references: Vec<String>,

        ///
        /// The spacing between references.
        /// 
        #[clap(required = false, long = "spacing")]
        spacing: Option<usize>,

        ///
        /// The leader of the reference bar.
        /// 
        #[clap(required = false, long = "leader")]
        leader: Option<String>,
    },
}
