//! Configuration loading

use std::path::PathBuf;

use secrecy::SecretString;

use super::types::{AppConfig, ConfigFile};
use crate::cli::args::Cli;
use crate::error::{ConfigError, Result};

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from all sources
    ///
    /// Priority (highest to lowest):
    /// 1. Command-line arguments
    /// 2. Environment variables
    /// 3. .env file in current directory
    /// 4. Config file (~/.config/audioshake/config.toml)
    /// 5. Default values
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is not configured.
    pub fn load(cli: &Cli) -> Result<AppConfig> {
        // Load .env file if present (ignore errors)
        let _ = dotenvy::dotenv();

        // Load config file if exists
        let config_file = Self::load_config_file()?;

        // Build final config with priority
        let api_key = cli
            .api_key
            .clone()
            .or_else(|| {
                std::env::var("AUDIOSHAKE_API_KEY")
                    .ok()
                    .map(SecretString::from)
            })
            .or_else(|| config_file.api_key.clone().map(SecretString::from))
            .ok_or(ConfigError::MissingApiKey)?;

        let base_url = cli.base_url.clone();

        let default_output_dir = config_file
            .default_output_dir
            .unwrap_or_else(|| PathBuf::from("./output"));

        Ok(AppConfig {
            api_key,
            base_url,
            default_output_dir,
            default_format: cli.output_format,
            default_poll_interval: config_file.default_poll_interval.unwrap_or(5),
            default_timeout: config_file.default_timeout.unwrap_or(3600),
            verbose: cli.verbose,
            quiet: cli.quiet,
        })
    }

    fn load_config_file() -> Result<ConfigFile> {
        let path = AppConfig::config_file_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: ConfigFile = toml::from_str(&content).map_err(ConfigError::from)?;
            Ok(config)
        } else {
            Ok(ConfigFile::default())
        }
    }
}
