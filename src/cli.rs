use chrono::Datelike;
use clap::{ArgAction, Parser, Subcommand};

use crate::application::{twir, Application};

///
/// The application arguments.
///
#[derive(Debug, Parser)]
#[command(
    author = Application::AUTHOR,
    version,
    about = Application::DESCRIPTION,
    long_about = None,
)]
#[command(propagate_version = true)]
pub struct Arguments {
    ///
    /// The verbosity level.
    ///
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbosity: u8,

    ///
    /// The application command.
    ///
    #[command(subcommand)]
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
        /// Repair the wiki references.
        ///
        #[arg(long = "wiki-refs", action = ArgAction::SetTrue)]
        wiki_refs: bool,

        ///
        /// Remove the unused files.
        ///
        #[arg(long = "remove-unused-files", action = ArgAction::SetTrue)]
        remove_unused_files: bool,

        ///
        /// Rename the attached files.
        ///
        #[arg(long = "rename-files", action = ArgAction::SetTrue)]
        rename_files: bool,

        ///
        /// Repair the This Week in Rust issues.
        /// 
        #[arg(long = "twir-issues", action = ArgAction::SetTrue)]
        twir_issues: bool,

        ///
        /// Repair the Astronomy Picture of the Day issues.
        /// 
        #[arg(long = "apod-issues", action = ArgAction::SetTrue)]
        apod_issues: bool,

        ///
        /// Remove creation date of the notes.
        /// 
        #[arg(long = "remove-created", action = ArgAction::SetTrue)]
        remove_created: bool,

        ///
        /// Repair the banners info.
        /// 
        #[arg(long = "banners", action = ArgAction::SetTrue)]
        banners: bool,
    },

    ///
    /// Grab notes to the notes set.
    ///
    Grab {
        #[command(subcommand)]
        note: Note,
    },

    ///
    /// Show the additional information.
    ///
    Show {
        #[command(subcommand)]
        info: Info,
    },

    ///
    /// Add the additional information to the notes set.
    ///
    Add {
        #[command(subcommand)]
        object: AddedObject,
    },

    ///
    /// Rename the object of the notes set.
    /// 
    Rename {
        #[command(subcommand)]
        object: RenamedObject,
    },

    ///
    /// Remove the object of the notes set.
    /// 
    Remove {
        #[command(subcommand)]
        object: RemovedObject,
    },

    ///
    /// Configure the application.
    /// 
    Config {
        ///
        /// The configuration key.
        /// 
        #[arg(name = "key", required = true)]
        key: String,

        ///
        /// The configuration value.
        /// 
        #[arg(name = "value", required = true)]
        value: String,

        ///
        /// Change all dependencies.
        ///
        #[arg(long, action = ArgAction::SetTrue)]
        update: bool,
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
    #[command(name = "apod")]
    APoD {
        ///
        /// Update daily note in notes set.
        ///
        #[arg(short = 'd', long = "update-daily", action = ArgAction::SetTrue)]
        update_daily: bool,

        ///
        /// List of subtags of the parent tag #astronomy.
        /// 
        #[arg(short = 't', long = "subtag", required = false)]
        subtags: Option<Vec<String>>,
    },

    #[command(name = "twir")]
    TWiR {
        #[arg(
            short,
            long = "issue",
            required = true,
            action = ArgAction::Set,
        )]
        issues: twir::Issues,

        ///
        /// Update daily note in notes set.
        ///
        #[arg(short = 'd', long = "update-daily", action = ArgAction::SetTrue)]
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
    #[command(name = "twir")]
    TWiR {
        ///
        /// Show the only last issue.
        ///
        #[arg(short, long, action = ArgAction::SetTrue)]
        last: bool,
    },
}

///
/// The application add command object.
///
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum AddedObject {
    ///
    /// Add a banner to the note.
    /// 
    #[command(name = "banner")]
    Banner {
        ///
        /// The banner file name.
        /// 
        #[arg(required = true)]
        file_name: String,

        ///
        /// The note type.
        /// 
        #[arg(short = 't', long = "type", required = true)]
        note_type: String,

        ///
        /// The note tags.
        /// 
        #[arg(long = "tag", required = false)]
        note_tags: Option<Vec<String>>,
    },

    ///
    /// Add the calendar to the monthly note.
    ///
    #[command(name = "calendar")]
    Calendar {
        ///
        /// The year number.
        ///
        #[arg(
            default_value_t = chrono::offset::Local::now().year(), 
            short, long
        )]
        year: i32,

        ///
        /// The month number.
        /// 
        #[arg(
            default_value_t = chrono::offset::Local::now().month(), 
            short, long
        )]
        month: u32,
    },

    ///
    /// Add a creation date to the notes.
    /// 
    #[command(name = "created")]
    Created {
        ///
        /// The note type.
        /// 
        #[arg(short = 't', long = "type", required = true)]
        note_type: String,
    },
}

///
/// The application rename command object.
/// 
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum RenamedObject {
    ///
    /// Rename the banner file name.
    /// 
    #[command(name = "banner")]
    Banner {
        ///
        /// The old banner file name.
        /// 
        #[arg(required = true)]
        old_name: String,

        ///
        /// The new banner file name.
        /// 
        #[arg(required = true)]
        new_name: String,
    },
}

///
/// The application rename command object.
/// 
#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum RemovedObject {
    ///
    /// Remove the line from the all files of the vault.
    /// 
    #[command(name = "line")]
    Line {
        ///
        /// The line to remove.
        /// 
        #[arg(required = true)]
        line: String,
    },

    ///
    /// Remove the bookmarks from the vault.
    /// 
    #[command(name = "bookmarks")]
    Bookmarks,
}