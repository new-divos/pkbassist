use std::path;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::application::apod;
use crate::error::Error;

///
/// The notes application configuration.
///
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NotesConfig {
    ///
    /// The root directory of the notes set.
    ///
    #[serde(rename = "Root")]
    root: path::PathBuf,
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
    pub async fn new() -> Result<Self, Error> {
        let config_path = dirs::config_dir().ok_or(Error::CannotFindConfig)?;
        if !config_path.exists() || !config_path.is_dir() {
            return Err(Error::CannotFindConfig);
        }

        let config_path = config_path.join("nta");
        fs::create_dir_all(config_path.as_path()).await?;
        let config_path = config_path.join("config.toml");
        if !config_path.exists() || !config_path.is_file() {
            Self::create_config(config_path.as_path()).await?;
        }

        let mut buffer = String::new();
        {
            let mut file = File::open(config_path.as_path()).await?;
            file.read_to_string(&mut buffer).await?;
        }

        let config = toml::from_str::<Self>(&buffer)?;
        if !config.is_root_valid() {
            return Err(Error::IllegalNotesRoot(config.notes.root));
        }

        Ok(config)
    }

    ///
    /// Create the configuration file.
    ///
    async fn create_config(path: &path::Path) -> Result<(), Error> {
        let mut notes_root = String::new();
        print!("Enter the notes root path: ");
        let _ = std::io::stdin().read_line(&mut notes_root).unwrap();
        let notes_root = path::PathBuf::from(notes_root);

        let mut apod_key = String::new();
        print!("Enter the NASA Astronomy Picture of the Day API key: ");
        let _ = std::io::stdin().read_line(&mut apod_key).unwrap();

        let config = Self {
            notes: NotesConfig { root: notes_root },
            nasa_apod: NASAAPoDAPIConfig {
                key: Some(apod_key),
                version: apod::Version::V1_0,
            },
        };

        let content = toml::to_string(&config)?;
        {
            let mut file = File::create(path).await?;
            file.write_all(content.as_bytes()).await?;
            log::info!(
                "The configuration file \"{}\" has been created",
                path.display()
            );
        }

        Ok(())
    }

    ///
    /// Get the root directory of the notes set.
    ///
    #[inline]
    pub fn root(&self) -> &path::Path {
        self.notes.root.as_path()
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
