use std::{
    borrow::Cow,
    io::Write,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::application::{apod, Application};
use crate::error::Error;

///
/// The application options.
///
#[derive(Debug)]
pub struct Options {
    ///
    /// The configuration file path.
    ///
    config_file: PathBuf,

    ///
    /// The log file path.
    ///
    log_file: PathBuf,
}

impl Options {
    ///
    /// The new instance of the application options.
    ///
    pub async fn new() -> Result<Self, Error> {
        let project_dirs = ProjectDirs::from(
            Application::QUALIFIER,
            Application::AUTHOR,
            Application::NAME,
        )
        .ok_or(Error::AppInitError)?;

        if !project_dirs.config_dir().exists() {
            fs::create_dir_all(project_dirs.config_dir()).await?;
        }
        let config_file = project_dirs.config_dir().join("nta.toml");

        let log_path = project_dirs.data_local_dir().join("log");
        if !log_path.exists() {
            fs::create_dir_all(log_path.as_path()).await?;
        }
        let log_file = log_path.join("nta.log");

        Ok(Self {
            config_file,
            log_file,
        })
    }

    ///
    /// Get the configuration file path.
    ///
    #[inline]
    pub fn config_file(&self) -> &Path {
        self.config_file.as_path()
    }

    ///
    /// Get the log file path.
    ///
    #[inline]
    pub fn log_file(&self) -> &Path {
        self.log_file.as_path()
    }
}

///
/// The notes application configuration.
///
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NotesConfig {
    ///
    /// The root directory of the notes set.
    ///
    #[serde(rename = "Root")]
    root: PathBuf,

    ///
    /// The files directory of the notes set.
    ///
    #[serde(rename = "Files")]
    files_path: Option<PathBuf>,

    ///
    /// The daily directory of the notes set.
    ///
    #[serde(rename = "Daily")]
    daily_path: Option<PathBuf>,

    ///
    /// The news directory of the notes set.
    ///
    #[serde(rename = "News")]
    news_path: Option<PathBuf>,
}

///
/// The NASA Astronomy Picture of the Day API configuration.
///
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NASAAPoDAPIConfig {
    ///
    /// The NASA Astronomy Picture of the Day API Key.
    ///
    #[serde(rename = "Key")]
    key: Option<String>,

    ///
    /// The NASA Astronomy Picture of the Day API Version.
    ///
    #[serde(rename = "Version")]
    version: apod::Version,
}

///
/// The application configuration.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    ///
    /// The notes application configuration.
    ///
    #[serde(rename = "Notes")]
    notes: NotesConfig,

    ///
    /// The NASA Astronomy Picture of the Day API configuration.
    ///
    #[serde(rename = "NASA APoD API")]
    nasa_apod: NASAAPoDAPIConfig,
}

impl Config {
    ///
    /// New instance of the application configuration.
    ///
    pub async fn new(options: &Options) -> Result<Self, Error> {
        if !options.config_file().exists() {
            let mut notes_root = String::new();
            print!("Enter the notes root path: ");
            std::io::stdout().flush()?;
            let _ = std::io::stdin().read_line(&mut notes_root).unwrap();
            let notes_root = PathBuf::from(notes_root.trim());

            let files_path = notes_root.join("Files");
            let daily_path = notes_root.join("Daily");
            let base_path = notes_root.join("Base");
            let news_path = base_path.join("News");

            let mut apod_key = String::new();
            print!("Enter the NASA Astronomy Picture of the Day API key: ");
            std::io::stdout().flush()?;
            let _ = std::io::stdin().read_line(&mut apod_key).unwrap();
            let apod_key = apod_key.trim().to_owned();

            let config = Self {
                notes: NotesConfig {
                    root: notes_root,
                    files_path: Some(files_path),
                    daily_path: Some(daily_path),
                    news_path: Some(news_path),
                },
                nasa_apod: NASAAPoDAPIConfig {
                    key: Some(apod_key),
                    version: apod::Version::V1_0,
                },
            };

            let content = toml::to_string(&config)?;
            {
                let mut file = File::create(options.config_file()).await?;
                file.write_all(content.as_bytes()).await?;
                log::info!(
                    "The configuration file \"{}\" has been created",
                    options.config_file().display()
                );
            }
        }

        let mut buffer = String::new();
        {
            let mut file = File::open(options.config_file()).await?;
            file.read_to_string(&mut buffer).await?;
        }

        let config = toml::from_str::<Self>(&buffer)?;
        if !config.is_root_valid() {
            return Err(Error::IllegalNotesRoot(config.notes.root));
        }

        Ok(config)
    }

    ///
    /// Get the root directory of the notes set.
    ///
    #[inline]
    pub fn root(&self) -> &Path {
        self.notes.root.as_path()
    }

    ///
    /// Get the files directory of the notes set.
    ///
    #[inline]
    pub fn files_path(&self) -> Cow<Path> {
        if let Some(ref path) = self.notes.files_path {
            Cow::Borrowed(path.as_path())
        } else {
            Cow::Owned(self.notes.root.join("Files"))
        }
    }

    ///
    /// Get the files directory of the notes set.
    ///
    #[inline]
    pub fn daily_path(&self) -> Cow<Path> {
        if let Some(ref path) = self.notes.daily_path {
            Cow::Borrowed(path.as_path())
        } else {
            Cow::Owned(self.notes.root.join("Daily"))
        }
    }

    ///
    /// Get the news directory of the notes set.
    ///
    #[inline]
    pub fn news_path(&self) -> Cow<Path> {
        if let Some(ref path) = self.notes.news_path {
            Cow::Borrowed(path.as_path())
        } else {
            Cow::Owned(self.notes.root.join("Base").join("News"))
        }
    }

    ///
    /// Get NASA Astronomy Picture of the Day API Key.
    ///
    #[inline]
    pub fn apod_key(&self) -> Option<&str> {
        self.nasa_apod.key.as_deref()
    }

    ///
    /// Get NASA Astronomy Picture of the Day API Version.
    ///
    #[inline]
    pub fn apod_version(&self) -> apod::Version {
        self.nasa_apod.version
    }

    // Validate notes set root.
    #[inline]
    fn is_root_valid(&self) -> bool {
        self.notes.root.exists() && self.notes.root.is_dir()
    }
}
