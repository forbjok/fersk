use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Context;
use serde_derive::Deserialize;
use tracing::error;

use crate::util;

pub const CONFIG_DIR: &str = "fersk";
pub const CONFIG_FILENAME: &str = "config.toml";

pub const DEFAULT_TOML: &str = include_str!("default.toml");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub work_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_path: dirs::cache_dir().expect("No default cache directory found. Create a config and specify it."),
        }
    }
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, anyhow::Error> {
        let mut file =
            util::open_file(path).with_context(|| format!("Error opening config file: {}", path.display()))?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .with_context(|| format!("Error reading config file: {}", path.display()))?;

        Self::from_str(&toml_str)
    }

    pub fn default_location() -> Option<PathBuf> {
        get_default_config_path()
    }

    fn path_from_location(path: &Path) -> Result<PathBuf, anyhow::Error> {
        Ok(path.join(CONFIG_FILENAME))
    }

    pub fn from_location(path: &Path) -> Result<Self, anyhow::Error> {
        let config_file_path = Self::path_from_location(path)?;

        if config_file_path.exists() {
            Ok(Self::from_file(&config_file_path)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn from_default_location() -> Result<Self, anyhow::Error> {
        let location = Self::default_location().with_context(|| "No default location found")?;

        Self::from_location(&location)
    }

    pub fn write_default() -> Result<(), anyhow::Error> {
        if let Some(config_location) = Self::default_location() {
            let config_file_path = Self::path_from_location(&config_location)?;

            if !config_file_path.exists() {
                // Create config directory if necessary.
                util::create_parent_dir(&config_file_path)
                    .with_context(|| format!("Error creating parent directory for: {}", config_file_path.display()))?;

                // Write config file.
                let mut file = util::create_file(&config_file_path)
                    .with_context(|| format!("Error creating config file at: {}", config_file_path.display()))?;
                file.write_all(DEFAULT_TOML.as_bytes())
                    .with_context(|| format!("Error writing to config file at: {}", config_file_path.display()))?;
            }
        }

        Ok(())
    }
}

impl FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s)?;

        Ok(config)
    }
}

pub fn get_default_config_path() -> Option<PathBuf> {
    let config_path = dirs::config_dir().map(|p| p.join(CONFIG_DIR));

    if config_path.is_none() {
        error!("Could not get configuration path!");
    }

    config_path
}
