//! Configuration types

use std::path::PathBuf;

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::cli::args::OutputFormat;
use crate::storage::config::StorageConfig;

/// Application configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    /// API key for authentication
    pub api_key: SecretString,
    /// API base URL
    pub base_url: Url,
    /// Default output directory
    pub default_output_dir: PathBuf,
    /// Default output format
    pub default_format: OutputFormat,
    /// Default poll interval in seconds
    pub default_poll_interval: u64,
    /// Default timeout in seconds
    pub default_timeout: u64,
    /// Verbosity level
    pub verbose: u8,
    /// Quiet mode
    pub quiet: bool,
    /// Storage configuration for local file uploads
    pub storage: Option<StorageConfig>,
}

impl AppConfig {
    /// Get the configuration directory path
    #[must_use]
    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("ai", "audioshake", "audioshake-cli").map_or_else(
            || PathBuf::from("."),
            |dirs| dirs.config_dir().to_path_buf(),
        )
    }

    /// Get the configuration file path
    #[must_use]
    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }
}

/// Configuration file structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    /// API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// API base URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// Default output directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_output_dir: Option<PathBuf>,

    /// Default output format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_format: Option<String>,

    /// Default poll interval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_poll_interval: Option<u64>,

    /// Default timeout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_timeout: Option<u64>,

    /// Storage configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<crate::storage::config::StorageConfigFile>,
}
