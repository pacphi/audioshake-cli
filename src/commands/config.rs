//! Config command implementation

use std::fs;

use secrecy::ExposeSecret;

use crate::api::client::ApiClient;
use crate::cli::args::ConfigCommands;
use crate::config::types::{AppConfig, ConfigFile};
use crate::error::Result;

/// Execute config command
///
/// # Errors
///
/// Returns an error if config operations fail.
pub async fn execute(command: ConfigCommands, config: &AppConfig) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            show(config);
            Ok(())
        }
        ConfigCommands::Set { key, value } => set(&key, &value),
        ConfigCommands::Init { force } => init(force),
        ConfigCommands::Validate => validate(config).await,
    }
}

fn show(config: &AppConfig) {
    println!("{}", console::style("Configuration").bold());
    println!("{}", "-".repeat(20));

    // Mask API key
    let api_key = config.api_key.expose_secret();
    let masked = if api_key.len() > 8 {
        format!("{}****{}", &api_key[..4], &api_key[api_key.len() - 4..])
    } else {
        "****".to_string()
    };

    println!("API Key:       {}", console::style(masked).dim());
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

    // Show config file location
    let config_path = AppConfig::config_file_path();
    println!(
        "\nConfig file:   {}",
        if config_path.exists() {
            console::style(config_path.display().to_string()).green()
        } else {
            console::style("(not created)".to_string()).dim()
        }
    );
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
        _ => {
            return Err(crate::error::AppError::User(format!(
                "Unknown config key: {key}. Valid keys: api-key, base-url, default-output-dir, default-format, default-poll-interval, default-timeout"
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
