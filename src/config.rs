use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    string::ToString,
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
    /// The base directory of the notes set.
    ///
    #[serde(rename = "Base", skip_serializing_if = "Option::is_none")]
    base_path: Option<PathBuf>,
}

///
/// The NASA Astronomy Picture of the Day notes configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) struct APoDConfig {
    ///
    /// The NASA Astronomy Picture of the Day download path.
    ///
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

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

    ///
    /// The NASA Astronomy Picture of the Day note banner.
    ///
    #[serde(rename = "Banner", skip_serializing_if = "Option::is_none")]
    banner: Option<String>,

    ///
    /// The NASA Astronomy Picture of the Day daily link prefix.
    ///
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,

    ///
    /// Insert the NASA Astronomy Picture of the Day daily link
    /// after the following string.
    ///
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    marker: Option<String>,

    ///
    /// The NASA Astronomy Picture of the Day icon.
    ///
    #[serde(rename = "Icon", skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

///
/// This Week in Rust notes configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) struct TWiRConfig {
    ///
    /// This Week in Rust download path.
    ///
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

    ///
    /// This Week in Rust note banner.
    ///
    #[serde(rename = "Banner", skip_serializing_if = "Option::is_none")]
    banner: Option<String>,

    ///
    /// This Week in Rust daily link prefix.
    ///
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,

    ///
    /// Insert the This Week in Rust daily link
    /// after the following string.
    ///
    #[serde(rename = "Marker", skip_serializing_if = "Option::is_none")]
    marker: Option<String>,

    ///
    /// The This Week in Rust icon.
    ///
    #[serde(rename = "Icon", skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

///
/// The Raindrop.io notes configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RaindropConfig {
    ///
    /// The Raindrop.io download path.
    ///
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

    ///
    /// The Raindrop.io file name prefix.
    ///
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
}

impl RaindropConfig {
    ///
    /// Get the Raindrop.io download path.
    ///
    #[inline]
    pub fn path(&self) -> Result<&Path, Error> {
        self.path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent("raindrop.path"))
    }

    ///
    /// Get the Raindrop.io file name prefix.
    ///
    #[inline]
    pub fn prefix(&self) -> Result<&str, Error> {
        self.prefix
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent("raindrop.prefix"))
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
    /// The NASA Astronomy Picture of the Day notes configuration.
    ///
    #[serde(rename = "APoD")]
    apod_config: APoDConfig,

    ///
    /// This Week in Rust notes configuration.
    ///
    #[serde(rename = "TWiR")]
    twir_config: TWiRConfig,

    ///
    /// The Raindrop.io notes configuration.
    ///
    #[serde(rename = "Raindrop")]
    raindrop_config: RaindropConfig,
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
            twir_config: Default::default(),
            raindrop_config: Default::default(),
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

            if let Some(root_path) = config.vault_config.root.as_deref() {
                if config.raindrop_config.path.is_none() {
                    config.raindrop_config.path = Some(root_path.join("Base"));
                }
            } else {
                return Err(Error::VaultRootIsAbsent);
            }

            Ok(config)
        } else {
            self.save().await?;
            Ok(self)
        }
    }

    ///
    /// Save the configuration to the file.
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
    /// Get the Raindrop.io configuration.
    ///
    #[inline]
    pub fn raindrop(&self) -> &RaindropConfig {
        &self.raindrop_config
    }

    ///
    /// Set the configuration property value.
    ///
    pub fn set<K: AsRef<str>, V: AsRef<str>>(&mut self, key: K, value: V) -> Result<(), Error> {
        let key = key.as_ref();
        let value = value.as_ref();

        match key {
            "raindrop.path" => {
                self.raindrop_config.path = Some(PathBuf::from(value));
            }

            "raindrop.prefix" => {
                self.raindrop_config.prefix = Some(value.to_string());
            }

            _ => return Err(Error::IllegalConfKey(key.to_string())),
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
    pub(crate) fn set_root(&mut self, path: &Path, update: bool) -> Result<(), Error> {
        self.vault_config.root = Some(PathBuf::from(path));

        if update {
            self.vault_config.files_path = Some(PathBuf::from(
                self.files_path().ok_or(Error::VaultRootIsAbsent)?,
            ));

            self.vault_config.daily_path = Some(PathBuf::from(
                self.daily_path().ok_or(Error::VaultRootIsAbsent)?,
            ));

            self.vault_config.base_path = Some(PathBuf::from(
                self.base_path().ok_or(Error::VaultRootIsAbsent)?,
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
    /// Get the base directory of the notes set.
    ///
    #[inline]
    pub fn base_path(&self) -> Option<Cow<Path>> {
        if let Some(ref path_buf) = self.vault_config.base_path {
            Some(Cow::Borrowed(path_buf.as_path()))
        } else {
            self.vault_config
                .root
                .as_ref()
                .map(|path_buf| Cow::Owned(path_buf.join("Base")))
        }
    }

    ///
    /// Set the base directory of the notes set.
    ///
    #[inline]
    pub(crate) fn set_base_path(&mut self, path: &Path) {
        self.vault_config.base_path = Some(PathBuf::from(path));
    }

    ///
    /// Get the Astronomy Picture of the Day directory of the notes set.
    ///
    #[inline]
    pub fn apod_path(&self) -> Option<Cow<Path>> {
        if let Some(ref path_buf) = self.apod_config.path {
            Some(Cow::Borrowed(path_buf.as_path()))
        } else {
            self.base_path()
        }
    }

    ///
    /// Set the Astronomy Picture of the Day directory of the notes set.
    ///
    #[inline]
    pub(crate) fn set_apod_path(&mut self, path: &Path) {
        self.apod_config.path = Some(PathBuf::from(path));
    }

    ///
    /// Get the This Week in Rust directory of the notes set.
    ///
    #[inline]
    pub fn twir_path(&self) -> Option<Cow<Path>> {
        if let Some(ref path_buf) = self.twir_config.path {
            Some(Cow::Borrowed(path_buf.as_path()))
        } else {
            self.vault_config
                .root
                .as_ref()
                .map(|path_buf| Cow::Owned(path_buf.join("Base")))
        }
    }

    ///
    /// Set the This Week in Rust directory of the notes set.
    ///
    #[inline]
    pub(crate) fn set_twir_path(&mut self, path: &Path) {
        self.twir_config.path = Some(PathBuf::from(path));
    }

    ///
    /// Get the NASA Astronomy Picture of the Day API Key.
    ///
    #[inline]
    pub fn apod_key(&self) -> Option<&str> {
        self.apod_config.key.as_deref()
    }

    ///
    /// Set the NASA Astronomy Picture of the Day API Key.
    ///
    #[inline]
    pub(crate) fn set_apod_key<S: ToString>(&mut self, value: S) {
        self.apod_config.key = Some(value.to_string());
    }

    ///
    /// Get the NASA Astronomy Picture of the Day note banner.
    ///
    #[inline]
    pub fn apod_banner(&self) -> Option<&str> {
        self.apod_config.banner.as_deref()
    }

    ///
    /// Set the NASA Astronomy Picture of the Day note banner.
    ///
    #[inline]
    pub(crate) fn set_apod_banner<S: ToString>(&mut self, value: S) {
        self.apod_config.banner = Some(value.to_string())
    }

    ///
    /// Get the NASA Astronomy Picture of the Day daily
    /// link prefix.
    ///
    #[inline]
    pub fn apod_prefix(&self) -> Option<&str> {
        self.apod_config.prefix.as_deref()
    }

    ///
    /// Set the NASA Astronomy Picture of the Day daily
    /// link prefix.
    ///
    #[inline]
    pub(crate) fn set_apod_prefix<S: ToString>(&mut self, value: S) {
        self.apod_config.prefix = Some(value.to_string())
    }

    ///
    /// Get the NASA Astronomy Picture of the Day daily
    /// link marker.
    ///
    #[inline]
    pub fn apod_marker(&self) -> Option<&str> {
        self.apod_config.marker.as_deref()
    }

    ///
    /// Set the NASA Astronomy Picture of the Day daily
    /// link marker.
    ///
    #[inline]
    pub(crate) fn set_apod_marker<S: ToString>(&mut self, value: S) {
        self.apod_config.marker = Some(value.to_string())
    }

    ///
    /// Get the NASA Astronomy Picture of the Day icon.
    ///
    #[inline]
    pub fn apod_icon(&self) -> Option<&str> {
        self.apod_config.icon.as_deref()
    }

    ///
    /// Set the NASA Astronomy Picture of the Day icon.
    ///
    #[inline]
    pub(crate) fn set_apod_icon<S: ToString>(&mut self, value: S) {
        self.apod_config.icon = Some(value.to_string())
    }

    ///
    /// Get the NASA Astronomy Picture of the Day API Version.
    ///
    #[inline]
    pub fn apod_version(&self) -> apod::Version {
        self.apod_config.version
    }

    ///
    /// Get This Week in Rust note banner.
    ///
    #[inline]
    pub fn twir_banner(&self) -> Option<&str> {
        self.twir_config.banner.as_deref()
    }

    ///
    /// Set This Week in Rust note banner.
    ///
    #[inline]
    pub(crate) fn set_twir_banner<S: ToString>(&mut self, value: S) {
        self.twir_config.banner = Some(value.to_string())
    }

    ///
    /// Get This Week in Rust daily link prefix.
    ///
    #[inline]
    pub fn twir_prefix(&self) -> Option<&str> {
        self.twir_config.prefix.as_deref()
    }

    ///
    /// Set This Week in Rust daily link prefix.
    ///
    #[inline]
    pub(crate) fn set_twir_prefix<S: ToString>(&mut self, value: S) {
        self.twir_config.prefix = Some(value.to_string())
    }

    ///
    /// Get This Week in Rust daily link marker.
    ///
    #[inline]
    pub fn twir_marker(&self) -> Option<&str> {
        self.twir_config.marker.as_deref()
    }

    ///
    /// Set This Week in Rust daily link marker.
    ///
    #[inline]
    pub(crate) fn set_twir_marker<S: ToString>(&mut self, value: S) {
        self.twir_config.marker = Some(value.to_string())
    }

    ///
    /// Get the This Week in Rust icon.
    ///
    #[inline]
    pub fn twir_icon(&self) -> Option<&str> {
        self.twir_config.icon.as_deref()
    }

    ///
    /// Set the This Week in Rust icon.
    ///
    pub(crate) fn set_twir_icon<S: ToString>(&mut self, value: S) {
        self.twir_config.icon = Some(value.to_string());
    }
}
