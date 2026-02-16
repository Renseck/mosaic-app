//! Configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application environment
    #[serde(default = "default_environment")]
    pub environment: String,

    /// Debug mode
    #[serde(default)]
    pub debug: bool,

    /// Application settings
    #[serde(default)]
    pub app: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,

    /// Data directory
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            environment: default_environment(),
            debug: false,
            app: AppConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            data_dir: default_data_dir(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &str) -> Result {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &str) -> Result {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path))?;

        Ok(())
    }
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_app_name() -> String {
    "mosaic-app".to_string()
}

fn default_data_dir() -> String {
    "./data".to_string()
}