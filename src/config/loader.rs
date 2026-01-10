//! Configuration loading

use std::path::PathBuf;

use secrecy::SecretString;
use url::Url;

use super::types::{AppConfig, ConfigFile};
use crate::cli::args::Cli;
use crate::error::{ConfigError, Result};
use crate::storage::config::{R2Config, StorageConfig, StorageProviderType};

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
            .clone()
            .unwrap_or_else(|| PathBuf::from("./output"));

        // Load storage configuration
        let storage = Self::load_storage_config(&config_file);

        Ok(AppConfig {
            api_key,
            base_url,
            default_output_dir,
            default_format: cli.output_format,
            default_poll_interval: config_file.default_poll_interval.unwrap_or(5),
            default_timeout: config_file.default_timeout.unwrap_or(3600),
            verbose: cli.verbose,
            quiet: cli.quiet,
            storage,
        })
    }

    /// Load storage configuration from environment and config file
    fn load_storage_config(config_file: &ConfigFile) -> Option<StorageConfig> {
        // Check environment variables first
        let provider_str = std::env::var("AUDIOSHAKE_STORAGE_PROVIDER")
            .ok()
            .or_else(|| {
                config_file
                    .storage
                    .as_ref()
                    .and_then(|s| s.provider.clone())
            })?;

        let provider: StorageProviderType = provider_str.parse().ok()?;

        if provider == StorageProviderType::None {
            return None;
        }

        let bucket = std::env::var("AUDIOSHAKE_STORAGE_BUCKET")
            .ok()
            .or_else(|| config_file.storage.as_ref().and_then(|s| s.bucket.clone()))?;

        let public_url_base_str = std::env::var("AUDIOSHAKE_STORAGE_PUBLIC_URL_BASE")
            .ok()
            .or_else(|| {
                config_file
                    .storage
                    .as_ref()
                    .and_then(|s| s.public_url_base.clone())
            })?;

        let public_url_base = Url::parse(&public_url_base_str).ok()?;

        let r2 = if provider == StorageProviderType::R2 {
            Self::load_r2_config(config_file)
        } else {
            None
        };

        Some(StorageConfig {
            provider,
            bucket,
            public_url_base,
            r2,
        })
    }

    /// Load R2-specific configuration
    fn load_r2_config(config_file: &ConfigFile) -> Option<R2Config> {
        let account_id = std::env::var("AUDIOSHAKE_R2_ACCOUNT_ID").ok().or_else(|| {
            config_file
                .storage
                .as_ref()
                .and_then(|s| s.r2.as_ref())
                .and_then(|r| r.account_id.clone())
        })?;

        let access_key_id = std::env::var("AUDIOSHAKE_R2_ACCESS_KEY_ID")
            .ok()
            .or_else(|| {
                config_file
                    .storage
                    .as_ref()
                    .and_then(|s| s.r2.as_ref())
                    .and_then(|r| r.access_key_id.clone())
            })?;

        let secret_access_key = std::env::var("AUDIOSHAKE_R2_SECRET_ACCESS_KEY")
            .ok()
            .or_else(|| {
                config_file
                    .storage
                    .as_ref()
                    .and_then(|s| s.r2.as_ref())
                    .and_then(|r| r.secret_access_key.clone())
            })?;

        Some(R2Config {
            account_id,
            access_key_id,
            secret_access_key: SecretString::from(secret_access_key),
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
