use std::{
    marker::PhantomData,
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
pub struct VaultConfig {
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

    ///
    /// The templates directory of the notes set.
    ///
    #[serde(rename = "Templates", skip_serializing_if = "Option::is_none")]
    templates_path: Option<PathBuf>,
}

impl VaultConfig {
    // The property name for the root path.
    const ROOT_PROPERTY: &'static str = "vault.root";
    // The property name for the files path.
    const FILES_PROPERTY: &'static str = "vault.files";
    // The property name for the daily path.
    const DAILY_PROPERTY: &'static str = "vault.daily";
    // The property name for the base path.
    const BASE_PROPERTY: &'static str = "vault.base";
    // The property name for the templates path.
    const TEMPLATES_PROPERTY: &'static str = "vault.templates";

    ///
    /// Get the root path of the notes set.
    ///
    #[inline]
    pub fn root_path(&self) -> Result<&Path, Error> {
        self.root
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::ROOT_PROPERTY))
    }

    ///
    /// Get the files path of the notes set.
    ///
    #[inline]
    pub fn files_path(&self) -> Result<&Path, Error> {
        self.files_path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::FILES_PROPERTY))
    }

    ///
    /// Get the daily path of the notes set.
    ///
    pub fn daily_path(&self) -> Result<&Path, Error> {
        self.daily_path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::DAILY_PROPERTY))
    }

    ///
    /// Get the base path of the notes set.
    ///
    pub fn base_path(&self) -> Result<&Path, Error> {
        self.base_path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::BASE_PROPERTY))
    }

    ///
    /// Get the templates path of the notes set.
    ///
    #[inline]
    pub fn templates_path(&self) -> Result<&Path, Error> {
        self.templates_path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::TEMPLATES_PROPERTY))
    }
}

///
/// The templates descriptor.
///
pub trait TemplatesDescriptor {
    // The property name for the note file name template.
    const TEMPLATES_FILENAME_PROPERTY: &'static str;
    // The property name for the note content template.
    const TEMPLATES_CONTENT_PROPERTY: &'static str;
    // The property name for the note daily link template.
    const TEMPLATES_DAILYREF_PROPERTY: &'static str;
}

///
/// The templates configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TemplatesConfig<T: TemplatesDescriptor> {
    ///
    /// The note file name template.
    ///
    #[serde(rename = "FileName")]
    filename: String,

    ///
    /// The note content template name.
    ///
    #[serde(rename = "Content", skip_serializing_if = "Option::is_none")]
    content: Option<String>,

    ///
    /// The daily reference to the note template name.
    ///
    #[serde(rename = "DailyRef", skip_serializing_if = "Option::is_none")]
    dailyref: Option<String>,

    ///
    /// The templates descriptor.
    ///
    #[serde(default, skip_serializing, skip_deserializing)]
    descriptor: PhantomData<T>,
}

impl<T: TemplatesDescriptor> TemplatesConfig<T> {
    ///
    /// Get the note file name template.
    ///
    #[inline]
    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }

    ///
    /// Get the note content template name.
    ///
    #[inline]
    pub fn content(&self) -> Result<&str, Error> {
        self.content
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(T::TEMPLATES_CONTENT_PROPERTY))
    }

    ///
    /// Get the daily reference to the note template name.
    ///
    #[inline]
    pub fn dailyref(&self) -> Result<&str, Error> {
        self.dailyref
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(
                T::TEMPLATES_DAILYREF_PROPERTY,
            ))
    }
}

///
/// The NASA Astronomy Picture of the Day notes configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct APoDConfig {
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

    ///
    /// The NASA Astronomy Picture of the Day templates.
    ///
    #[serde(rename = "Templates")]
    templates: TemplatesConfig<Self>,
}

impl TemplatesDescriptor for APoDConfig {
    // The property name for the NASA Astronomy Picture of the Day file name template.
    const TEMPLATES_FILENAME_PROPERTY: &'static str = "apod.templates.filename";
    // The property name for the NASA Astronomy Picture of the Day content template.
    const TEMPLATES_CONTENT_PROPERTY: &'static str = "apod.templates.content";
    // The property name for the NASA Astronomy Picture of the Day daily link template.
    const TEMPLATES_DAILYREF_PROPERTY: &'static str = "apod.templates.dailyref";
}

impl APoDConfig {
    // The property name for the NASA Astronomy Picture of the Day download path.
    const PATH_PROPERTY: &'static str = "apod.path";
    // The property name for the NASA Astronomy Picture of the Day API Key.
    const APIKEY_PROPERTY: &'static str = "apod.key";
    // The property name for the NASA Astronomy Picture of the Day API Version.
    const APIVERSION_PROPERTY: &'static str = "apod.version";
    // The property name for the NASA Astronomy Picture of the Day note banner.
    const BANNER_PROPERTY: &'static str = "apod.banner";
    // The property name for the NASA Astronomy Picture of the Day daily link prefix.
    const PREFIX_PROPERTY: &'static str = "apod.prefix";
    // The property name for the NASA Astronomy Picture of the Day daily link marker.
    const MARKER_PROPERTY: &'static str = "apod.marker";
    // The property name for the NASA Astronomy Picture of the Day icon.
    const ICON_PROPERTY: &'static str = "apod.icon";

    ///
    /// Get the NASA Astronomy Picture of the Day download path.
    ///
    #[inline]
    pub fn path(&self) -> Result<&Path, Error> {
        self.path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::PATH_PROPERTY))
    }

    ///
    /// Get the NASA Astronomy Picture of the Day API Key.
    ///
    #[inline]
    pub fn api_key(&self) -> Result<&str, Error> {
        self.key
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::APIKEY_PROPERTY))
    }

    ///
    /// Get the NASA Astronomy Picture of the Day API Version.
    ///
    #[inline]
    pub fn api_version(&self) -> apod::Version {
        self.version
    }

    ///
    /// Get the NASA Astronomy Picture of the Day note banner.
    ///
    #[inline]
    pub fn banner(&self) -> Option<&str> {
        self.banner.as_deref()
    }

    ///
    /// Get the NASA Astronomy Picture of the Day daily link prefix.
    ///
    #[inline]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    ///
    /// Get the NASA Astronomy Picture of the Day daily link marker.
    ///
    #[inline]
    pub fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    ///
    /// Get the NASA Astronomy Picture of the Day icon.
    ///
    #[inline]
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    ///
    /// Get the NASA Astronomy Picture of the Day templates.
    ///
    #[inline]
    pub fn templates(&self) -> &TemplatesConfig<Self> {
        &self.templates
    }
}

///
/// This Week in Rust notes configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TWiRConfig {
    ///
    /// The This Week in Rust download path.
    ///
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

    ///
    /// The This Week in Rust note banner.
    ///
    #[serde(rename = "Banner", skip_serializing_if = "Option::is_none")]
    banner: Option<String>,

    ///
    /// The This Week in Rust daily link prefix.
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

impl TWiRConfig {
    // The property name for the This Week in Rust download path.
    const PATH_PROPERTY: &'static str = "twir.path";
    // The property name for the This Week in Rust note banner.
    const BANNER_PROPERTY: &'static str = "twir.banner";
    // The property name for the This Week in Rust link prefix.
    const PREFIX_PROPERTY: &'static str = "twir.prefix";
    // The property name for the This Week in Rust daily link marker.
    const MARKER_PROPERTY: &'static str = "twir.marker";
    // The property name for the This Week in Rust icon.
    const ICON_PROPERTY: &'static str = "twir.icon";

    ///
    /// Get the This Week in Rust download path.
    ///
    #[inline]
    pub fn path(&self) -> Result<&Path, Error> {
        self.path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::PATH_PROPERTY))
    }

    ///
    /// Get the This Week in Rust note banner.
    ///
    #[inline]
    pub fn banner(&self) -> Option<&str> {
        self.banner.as_deref()
    }

    ///
    /// Get the This Week in Rust link prefix.
    ///
    #[inline]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    ///
    /// Get the This Week in Rust link marker.
    ///
    #[inline]
    pub fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }

    ///
    /// Get the This Week in Rust icon.
    ///
    #[inline]
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
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
    // The property name for the Raindrop.io download path.
    const PATH_PROPERTY: &'static str = "raindrop.path";
    // The property name for the Raindrop.io file name prefix.
    const PREFIX_PROPERTY: &'static str = "raindrop.prefix";

    ///
    /// Get the Raindrop.io download path.
    ///
    #[inline]
    pub fn path(&self) -> Result<&Path, Error> {
        self.path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::PATH_PROPERTY))
    }

    ///
    /// Get the Raindrop.io file name prefix.
    ///
    #[inline]
    pub fn prefix(&self) -> Result<&str, Error> {
        self.prefix
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::PREFIX_PROPERTY))
    }
}

///
/// The Omnivore notes configuration.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct OmnivoreConfig {
    ///
    /// The Omnivore download path.
    ///
    #[serde(rename = "Path", skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

    ///
    /// The Omnivore file name prefix.
    ///
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
}

impl OmnivoreConfig {
    // The property name for the Omnivore download path.
    const PATH_PROPERTY: &'static str = "omnivore.path";
    // The property name for the Omnivore file name prefix.
    const PREFIX_PROPERTY: &'static str = "omnivore.prefix";

    ///
    /// Get the Omnivore download path.
    ///
    #[inline]
    pub fn path(&self) -> Result<&Path, Error> {
        self.path
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::PATH_PROPERTY))
    }

    ///
    /// Get the Omnivore file name prefix.
    ///
    #[inline]
    pub fn prefix(&self) -> Result<&str, Error> {
        self.prefix
            .as_deref()
            .ok_or(Error::ConfigPropertyIsAbsent(Self::PREFIX_PROPERTY))
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

    ///
    /// The Onmivore notes configuration.
    ///
    #[serde(rename = "Omnivore")]
    omnivore_config: OmnivoreConfig,
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
        let config_file = project_dirs.config_dir().join("pkbassist.toml");

        let log_path = project_dirs.data_local_dir().join("log");
        if !log_path.exists() {
            fs::create_dir_all(log_path.as_path()).await?;
        }
        let log_file = log_path.join("pkbassist.log");

        Ok(Self {
            config_file,
            log_file,
            vault_config: Default::default(),
            apod_config: Default::default(),
            twir_config: Default::default(),
            raindrop_config: Default::default(),
            omnivore_config: Default::default(),
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
    /// Get the vault configuration.
    ///
    #[inline]
    pub fn vault(&self) -> &VaultConfig {
        &self.vault_config
    }

    ///
    /// Get the NASA Astronomy Picture of the Day configuration.
    ///
    #[inline]
    pub fn apod(&self) -> &APoDConfig {
        &self.apod_config
    }

    ///
    /// Get the This Week in Rust configuration.
    ///
    #[inline]
    pub fn twir(&self) -> &TWiRConfig {
        &self.twir_config
    }

    ///
    /// Get the Raindrop.io configuration.
    ///
    #[inline]
    pub fn raindrop(&self) -> &RaindropConfig {
        &self.raindrop_config
    }

    ///
    /// Get the Omnivore configuration.
    ///
    #[inline]
    pub fn omnivore(&self) -> &OmnivoreConfig {
        &self.omnivore_config
    }

    ///
    /// Set the configuration property value.
    ///
    pub fn set<K: AsRef<str>, V: AsRef<str>>(&mut self, key: K, value: V) -> Result<(), Error> {
        let key = key.as_ref();
        let value = value.as_ref();

        match key {
            VaultConfig::ROOT_PROPERTY => {
                self.vault_config.root = Some(PathBuf::from(value));
            }

            VaultConfig::FILES_PROPERTY => {
                self.vault_config.files_path = Some(PathBuf::from(value));
            }

            VaultConfig::DAILY_PROPERTY => {
                self.vault_config.daily_path = Some(PathBuf::from(value));
            }

            VaultConfig::BASE_PROPERTY => {
                self.vault_config.base_path = Some(PathBuf::from(value));
            }

            VaultConfig::TEMPLATES_PROPERTY => {
                self.vault_config.templates_path = Some(PathBuf::from(value));
            }

            APoDConfig::PATH_PROPERTY => {
                self.apod_config.path = Some(PathBuf::from(value));
            }

            APoDConfig::APIKEY_PROPERTY => {
                self.apod_config.key = Some(value.to_string());
            }

            APoDConfig::APIVERSION_PROPERTY => match value.to_lowercase().as_str() {
                "v1" => self.apod_config.version = apod::Version::V1_0,
                _ => return Err(Error::IllegalConfValue(value.to_string())),
            },

            APoDConfig::BANNER_PROPERTY => {
                self.apod_config.banner = Some(value.to_string());
            }

            APoDConfig::PREFIX_PROPERTY => {
                self.apod_config.prefix = Some(value.to_string());
            }

            APoDConfig::MARKER_PROPERTY => {
                self.apod_config.marker = Some(value.to_string());
            }

            APoDConfig::ICON_PROPERTY => {
                self.apod_config.icon = Some(value.to_string());
            }

            APoDConfig::TEMPLATES_FILENAME_PROPERTY => {
                self.apod_config.templates.filename = value.to_string();
            }

            APoDConfig::TEMPLATES_CONTENT_PROPERTY => {
                self.apod_config.templates.content = Some(value.to_string());
            }

            APoDConfig::TEMPLATES_DAILYREF_PROPERTY => {
                self.apod_config.templates.dailyref = Some(value.to_string());
            }

            TWiRConfig::PATH_PROPERTY => {
                self.twir_config.path = Some(PathBuf::from(value));
            }

            TWiRConfig::BANNER_PROPERTY => {
                self.twir_config.banner = Some(value.to_string());
            }

            TWiRConfig::PREFIX_PROPERTY => {
                self.twir_config.prefix = Some(value.to_string());
            }

            TWiRConfig::MARKER_PROPERTY => {
                self.twir_config.marker = Some(value.to_string());
            }

            TWiRConfig::ICON_PROPERTY => {
                self.twir_config.icon = Some(value.to_string());
            }

            RaindropConfig::PATH_PROPERTY => {
                self.raindrop_config.path = Some(PathBuf::from(value));
            }

            RaindropConfig::PREFIX_PROPERTY => {
                self.raindrop_config.prefix = Some(value.to_string());
            }

            OmnivoreConfig::PATH_PROPERTY => {
                self.omnivore_config.path = Some(PathBuf::from(value));
            }

            OmnivoreConfig::PREFIX_PROPERTY => {
                self.omnivore_config.prefix = Some(value.to_string());
            }

            _ => return Err(Error::IllegalConfKey(key.to_string())),
        }

        Ok(())
    }
}
