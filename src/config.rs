use std::{
    borrow::Cow,
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
/// The notes application configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) struct VaultConfig {
    ///
    /// The root directory of the notes set.
    ///
    #[serde(rename = "Root", skip_serializing_if = "Option::is_none")]
    root: Option<PathBuf>,

    ///
    /// The files directory of the notes set.
    ///
    #[serde(rename = "Files", skip_serializing_if = "Option::is_none")]
    files_path: Option<PathBuf>,

    ///
    /// The daily directory of the notes set.
    ///
    #[serde(rename = "Daily", skip_serializing_if = "Option::is_none")]
    daily_path: Option<PathBuf>,

    ///
    /// The Astronomy Picture of the Day directory of the notes set.
    ///
    #[serde(rename = "APoD", skip_serializing_if = "Option::is_none")]
    apod_path: Option<PathBuf>,

    ///
    /// The This Day in Rust directory of the notes set.
    ///
    #[serde(rename = "TWiR", skip_serializing_if = "Option::is_none")]
    twir_path: Option<PathBuf>,
}

///
/// The NASA Astronomy Picture of the Day API configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) struct NASAAPoDAPIConfig {
    ///
    /// The NASA Astronomy Picture of the Day API Key.
    ///
    #[serde(rename = "Key", skip_serializing_if = "Option::is_none")]
    key: Option<String>,

    ///
    /// The NASA Astronomy Picture of the Day API Version.
    ///
    #[serde(rename = "Version")]
    version: apod::Version,
}

///
/// The reference bar (refbar) configuration.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct RefBarConfig {
    ///
    /// The spacing between references.
    ///
    #[serde(rename = "Spacing", skip_serializing_if = "Option::is_none")]
    spacing: Option<usize>,

    ///
    /// The leader of the reference bar.
    ///
    #[serde(rename = "Leader", skip_serializing_if = "Option::is_none")]
    leader: Option<String>,
}

impl Default for RefBarConfig {
    fn default() -> Self {
        Self {
            spacing: Some(10),
            leader: Default::default(),
        }
    }
}

///
/// The application configuration.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    ///
    /// The configuration file path.
    ///
    #[serde(skip)]
    config_file: PathBuf,

    ///
    /// The log file path.
    ///
    #[serde(skip)]
    log_file: PathBuf,

    ///
    /// The notes application configuration.
    ///
    #[serde(rename = "Vault")]
    vault_config: VaultConfig,

    ///
    /// The NASA Astronomy Picture of the Day API configuration.
    ///
    #[serde(rename = "APoD")]
    apod_config: NASAAPoDAPIConfig,

    ///
    /// The reference bar (refbar) configuration.
    ///
    #[serde(rename = "RefBar")]
    refbar_config: RefBarConfig,
}

impl Config {
    ///
    /// New instance of the application configuration.
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
            vault_config: Default::default(),
            apod_config: Default::default(),
            refbar_config: Default::default(),
        })
    }

    ///
    /// Load a configuration from the file.
    ///
    pub async fn load(self) -> Result<Self, Error> {
        if self.config_file.exists() {
            let mut buffer = String::new();
            {
                let mut file = File::open(self.config_file.as_path()).await?;
                file.read_to_string(&mut buffer).await?;
            }

            log::info!(
                "The configuration was loaded from the file \"{}\"",
                self.config_file.as_path().display()
            );

            let mut config = toml::from_str::<Self>(&buffer)?;
            config.config_file = self.config_file;
            config.log_file = self.log_file;

            Ok(config)
        } else {
            self.save().await?;
            Ok(self)
        }
    }

    ///
    /// Save a configuration to the file.
    ///
    pub async fn save(&self) -> Result<(), Error> {
        let content = toml::to_string(self)?;
        {
            let mut file = File::create(self.config_file.as_path()).await?;
            file.write_all(content.as_bytes()).await?;
            log::info!(
                "The configuration was saved to the file  \"{}\"",
                self.config_file.as_path().display()
            );
        }

        Ok(())
    }

    ///
    /// Get the configuration file name.
    ///
    #[inline]
    pub fn config_file(&self) -> &Path {
        self.config_file.as_path()
    }

    ///
    /// Get the logging file name.
    ///
    #[inline]
    pub fn log_file(&self) -> &Path {
        self.log_file.as_path()
    }

    ///
    /// Get the root directory of the vault.
    ///
    #[inline]
    pub fn root(&self) -> Option<&Path> {
        self.vault_config.root.as_deref()
    }

    ///
    /// Set the root directory of the vault.
    ///
    pub(crate) fn set_root(&mut self, path: &Path, all: bool) -> Result<(), Error> {
        self.vault_config.root = Some(PathBuf::from(path));

        if all {
            self.vault_config.files_path = None;
            self.vault_config.files_path = Some(PathBuf::from(
                self.files_path().ok_or(Error::VaultRootIsAbsent)?,
            ));

            self.vault_config.daily_path = None;
            self.vault_config.daily_path = Some(PathBuf::from(
                self.daily_path().ok_or(Error::VaultRootIsAbsent)?,
            ));

            self.vault_config.apod_path = None;
            self.vault_config.apod_path = Some(PathBuf::from(
                self.apod_path().ok_or(Error::VaultRootIsAbsent)?,
            ));

            self.vault_config.twir_path = None;
            self.vault_config.twir_path = Some(PathBuf::from(
                self.twir_path().ok_or(Error::VaultRootIsAbsent)?,
            ));
        }

        Ok(())
    }

    ///
    /// Get the files directory of the vault.
    ///
    #[inline]
    pub fn files_path(&self) -> Option<Cow<Path>> {
        if let Some(ref path_buf) = self.vault_config.files_path {
            Some(Cow::Borrowed(path_buf.as_path()))
        } else {
            self.vault_config
                .root
                .as_ref()
                .map(|path_buf| Cow::Owned(path_buf.join("Files")))
        }
    }

    ///
    /// Set the files directory of the vault.
    ///
    #[inline]
    pub(crate) fn set_files_path(&mut self, path: &Path) {
        self.vault_config.files_path = Some(PathBuf::from(path));
    }

    ///
    /// Get the files directory of the notes set.
    ///
    #[inline]
    pub fn daily_path(&self) -> Option<Cow<Path>> {
        if let Some(ref path_buf) = self.vault_config.daily_path {
            Some(Cow::Borrowed(path_buf.as_path()))
        } else {
            self.vault_config
                .root
                .as_ref()
                .map(|path_buf| Cow::Owned(path_buf.join("Daily")))
        }
    }

    ///
    /// Set the files directory of the notes set.
    ///
    #[inline]
    pub(crate) fn set_daily_path(&mut self, path: &Path) {
        self.vault_config.daily_path = Some(PathBuf::from(path));
    }

    ///
    /// Get the Astronomy Picture of the Day directory of the notes set.
    ///
    #[inline]
    pub fn apod_path(&self) -> Option<Cow<Path>> {
        if let Some(ref path_buf) = self.vault_config.apod_path {
            Some(Cow::Borrowed(path_buf.as_path()))
        } else {
            self.vault_config
                .root
                .as_ref()
                .map(|path_buf| Cow::Owned(path_buf.join("Base").join("Issues")))
        }
    }

    ///
    /// Set the Astronomy Picture of the Day directory of the notes set.
    ///
    #[inline]
    pub(crate) fn set_apod_path(&mut self, path: &Path) {
        self.vault_config.apod_path = Some(PathBuf::from(path));
    }

    ///
    /// Get the This Week in Rust directory of the notes set.
    ///
    #[inline]
    pub fn twir_path(&self) -> Option<Cow<Path>> {
        if let Some(ref path_buf) = self.vault_config.twir_path {
            Some(Cow::Borrowed(path_buf.as_path()))
        } else {
            self.vault_config
                .root
                .as_ref()
                .map(|path_buf| Cow::Owned(path_buf.join("Base").join("Issues")))
        }
    }

    ///
    /// Set the This Week in Rust directory of the notes set.
    ///
    #[inline]
    pub(crate) fn set_twir_path(&mut self, path: &Path) {
        self.vault_config.twir_path = Some(PathBuf::from(path));
    }

    ///
    /// Get NASA Astronomy Picture of the Day API Key.
    ///
    #[inline]
    pub fn apod_key(&self) -> Option<&str> {
        self.apod_config.key.as_deref()
    }

    ///
    /// Set NASA Astronomy Picture of the Day API Key.
    ///
    #[inline]
    pub(crate) fn set_apod_key(&mut self, value: &str) {
        self.apod_config.key = Some(value.to_string());
    }

    ///
    /// Get NASA Astronomy Picture of the Day API Version.
    ///
    #[inline]
    pub fn apod_version(&self) -> apod::Version {
        self.apod_config.version
    }

    ///
    /// Get the spacing between references.
    ///
    #[inline]
    pub fn refbar_spacing(&self) -> usize {
        if let Some(spacing) = self.refbar_config.spacing {
            spacing
        } else {
            10
        }
    }

    ///
    /// Set the spacing between references.
    ///
    #[inline]
    pub(crate) fn set_refbar_spacing(&mut self, value: usize) {
        self.refbar_config.spacing = Some(value);
    }

    ///
    /// Get the leader of the reference bar.
    ///
    #[inline]
    pub fn refbar_leader(&self) -> Option<&str> {
        self.refbar_config.leader.as_deref()
    }

    ///
    /// Set the leader of the reference bar.
    ///
    #[inline]
    pub(crate) fn set_refbar_leader(&mut self, value: &str) {
        self.refbar_config.leader = Some(value.to_string());
    }
}
