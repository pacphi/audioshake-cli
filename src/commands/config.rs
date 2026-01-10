//! Config command implementation

use std::fs;

use secrecy::ExposeSecret;

use crate::api::client::ApiClient;
use crate::cli::args::ConfigCommands;
use crate::config::types::{AppConfig, ConfigFile};
use crate::error::Result;
use crate::storage::config::{R2ConfigFile, StorageConfigFile};

/// Execute config command
///
/// # Errors
///
/// Returns an error if config operations fail.
pub async fn execute(command: ConfigCommands, config: &AppConfig) -> Result<()> {
    match command {
        ConfigCommands::Show { show_sensitive } => {
            show(config, show_sensitive);
            Ok(())
        }
        ConfigCommands::Set { key, value } => set(&key, &value),
        ConfigCommands::Init { force } => init(force),
        ConfigCommands::Validate => validate(config).await,
    }
}

fn show(config: &AppConfig, show_sensitive: bool) {
    println!("{}", console::style("Configuration").bold());
    println!("{}", "-".repeat(40));

    // API key - mask unless --show-sensitive
    let api_key = config.api_key.expose_secret();
    let api_key_display = if show_sensitive {
        api_key.to_string()
    } else {
        mask_secret(api_key)
    };

    println!("API Key:       {}", console::style(api_key_display).dim());
    println!("Base URL:      {}", console::style(&config.base_url).cyan());
    println!(
        "Output Dir:    {}",
        console::style(config.default_output_dir.display()).cyan()
    );
    println!(
        "Poll Interval: {}s",
        console::style(config.default_poll_interval).dim()
    );
    println!(
        "Timeout:       {}s",
        console::style(config.default_timeout).dim()
    );

    // Show storage configuration
    println!();
    println!("{}", console::style("Storage").bold());
    println!("{}", "-".repeat(40));

    if let Some(ref storage) = config.storage {
        println!(
            "Provider:      {}",
            console::style(&storage.provider).cyan()
        );
        println!("Bucket:        {}", console::style(&storage.bucket).cyan());
        println!(
            "Public URL:    {}",
            console::style(&storage.public_url_base).cyan()
        );

        if let Some(ref r2) = storage.r2 {
            // Account ID - mask unless --show-sensitive
            let account_display = if show_sensitive {
                r2.account_id.clone()
            } else {
                mask_secret(&r2.account_id)
            };
            println!("R2 Account:    {}", console::style(account_display).dim());

            // Access key - mask unless --show-sensitive
            let access_key_display = if show_sensitive {
                r2.access_key_id.clone()
            } else {
                mask_secret(&r2.access_key_id)
            };
            println!(
                "R2 Access Key: {}",
                console::style(access_key_display).dim()
            );

            // Secret key - never show, always reference env var
            println!(
                "R2 Secret:     {}",
                console::style("(set via AUDIOSHAKE_R2_SECRET_ACCESS_KEY env var)").dim()
            );
        }
    } else {
        println!(
            "{}",
            console::style("(not configured - required for local file processing)").dim()
        );
        println!();
        println!("Configure with:");
        println!("  audioshake config set storage.provider r2");
        println!("  audioshake config set storage.bucket my-bucket");
        println!("  audioshake config set storage.public-url-base https://my-bucket.r2.dev");
        println!("  audioshake config set storage.r2.account-id YOUR_ACCOUNT_ID");
        println!("  audioshake config set storage.r2.access-key-id YOUR_ACCESS_KEY");
        println!("  export AUDIOSHAKE_R2_SECRET_ACCESS_KEY=YOUR_SECRET");
    }

    // Show config file location
    let config_path = AppConfig::config_file_path();
    println!();
    println!(
        "Config file:   {}",
        if config_path.exists() {
            console::style(config_path.display().to_string()).green()
        } else {
            console::style("(not created)".to_string()).dim()
        }
    );

    if !show_sensitive {
        println!();
        println!(
            "{}",
            console::style("Tip: Use --show-sensitive to reveal masked values").dim()
        );
    }
}

/// Mask a secret value, showing only first 4 and last 4 characters
fn mask_secret(value: &str) -> String {
    if value.len() > 8 {
        format!("{}****{}", &value[..4], &value[value.len() - 4..])
    } else if value.is_empty() {
        "(not set)".to_string()
    } else {
        "****".to_string()
    }
}

fn set(key: &str, value: &str) -> Result<()> {
    let config_path = AppConfig::config_file_path();

    // Load existing config or create new
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        toml::from_str(&content).unwrap_or_default()
    } else {
        ConfigFile::default()
    };

    // Update value
    match key {
        "api-key" => config.api_key = Some(value.to_string()),
        "base-url" => config.base_url = Some(value.to_string()),
        "default-output-dir" => config.default_output_dir = Some(value.into()),
        "default-format" => config.default_format = Some(value.to_string()),
        "default-poll-interval" => {
            config.default_poll_interval = Some(value.parse().map_err(|_| {
                crate::error::AppError::User("Invalid number for poll interval".to_string())
            })?);
        }
        "default-timeout" => {
            config.default_timeout = Some(value.parse().map_err(|_| {
                crate::error::AppError::User("Invalid number for timeout".to_string())
            })?);
        }
        // Storage configuration keys
        "storage.provider" => {
            ensure_storage_config(&mut config);
            config.storage.as_mut().unwrap().provider = Some(value.to_string());
        }
        "storage.bucket" => {
            ensure_storage_config(&mut config);
            config.storage.as_mut().unwrap().bucket = Some(value.to_string());
        }
        "storage.public-url-base" => {
            ensure_storage_config(&mut config);
            config.storage.as_mut().unwrap().public_url_base = Some(value.to_string());
        }
        // R2-specific keys
        "storage.r2.account-id" => {
            ensure_storage_config(&mut config);
            ensure_r2_config(&mut config);
            config
                .storage
                .as_mut()
                .unwrap()
                .r2
                .as_mut()
                .unwrap()
                .account_id = Some(value.to_string());
        }
        "storage.r2.access-key-id" => {
            ensure_storage_config(&mut config);
            ensure_r2_config(&mut config);
            config
                .storage
                .as_mut()
                .unwrap()
                .r2
                .as_mut()
                .unwrap()
                .access_key_id = Some(value.to_string());
        }
        "storage.r2.secret-access-key" => {
            // Don't store secret in config file, just inform user
            println!(
                "{} For security, set secret via environment variable instead:",
                console::style("Note:").yellow()
            );
            println!(
                "  export AUDIOSHAKE_R2_SECRET_ACCESS_KEY={}",
                console::style(value).dim()
            );
            return Ok(());
        }
        _ => {
            return Err(crate::error::AppError::User(format!(
                "Unknown config key: {key}\n\nValid keys:\n  \
                api-key, base-url, default-output-dir, default-format,\n  \
                default-poll-interval, default-timeout,\n  \
                storage.provider, storage.bucket, storage.public-url-base,\n  \
                storage.r2.account-id, storage.r2.access-key-id"
            )));
        }
    }

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write config
    let content = toml::to_string_pretty(&config)
        .map_err(|e| crate::error::AppError::User(format!("Failed to serialize config: {e}")))?;

    fs::write(&config_path, content)?;

    println!(
        "{} Set {} = {}",
        console::style("✓").green(),
        console::style(key).cyan(),
        value
    );

    Ok(())
}

/// Ensure storage config section exists
fn ensure_storage_config(config: &mut ConfigFile) {
    if config.storage.is_none() {
        config.storage = Some(StorageConfigFile::default());
    }
}

/// Ensure R2 config section exists within storage
fn ensure_r2_config(config: &mut ConfigFile) {
    if let Some(ref mut storage) = config.storage {
        if storage.r2.is_none() {
            storage.r2 = Some(R2ConfigFile::default());
        }
    }
}

fn init(force: bool) -> Result<()> {
    let config_path = AppConfig::config_file_path();

    if config_path.exists() && !force {
        return Err(crate::error::AppError::User(format!(
            "Config file already exists: {}. Use --force to overwrite.",
            config_path.display()
        )));
    }

    // Create default config
    let config = ConfigFile {
        api_key: None,
        base_url: Some("https://api.audioshake.ai".to_string()),
        default_output_dir: Some("./output".into()),
        default_format: Some("wav".to_string()),
        default_poll_interval: Some(5),
        default_timeout: Some(3600),
        storage: None,
    };

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write config
    let content = toml::to_string_pretty(&config)
        .map_err(|e| crate::error::AppError::User(format!("Failed to serialize config: {e}")))?;

    fs::write(&config_path, content)?;

    println!(
        "{} Created config file: {}",
        console::style("✓").green(),
        console::style(config_path.display()).cyan()
    );
    println!();
    println!("Add your API key with:");
    println!(
        "  {} config set api-key YOUR_API_KEY",
        console::style("audioshake").dim()
    );

    Ok(())
}

async fn validate(config: &AppConfig) -> Result<()> {
    println!("Validating configuration...");

    // Check API key format
    let api_key = config.api_key.expose_secret();
    if api_key.starts_with("ashke_") {
        println!("  {} API key format valid", console::style("[OK]").green());
    } else {
        println!(
            "  {} API key format may be invalid (expected 'ashke_' prefix)",
            console::style("[WARN]").yellow()
        );
    }

    // Test API connection
    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    match client.validate().await {
        Ok(()) => {
            println!(
                "  {} API connection successful",
                console::style("[OK]").green()
            );
        }
        Err(e) => {
            println!(
                "  {} API connection failed: {}",
                console::style("[FAIL]").red(),
                e
            );
            return Err(e);
        }
    }

    println!();
    println!(
        "{} Configuration is valid!",
        console::style("✓").green().bold()
    );

    Ok(())
}
