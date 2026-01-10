//! Process command implementation

use std::path::Path;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use url::Url;

use crate::api::client::ApiClient;
use crate::api::request::{CreateTaskRequest, TaskTarget};
use crate::api::response::TaskStatus;
use crate::cli::args::ProcessArgs;
use crate::config::types::AppConfig;
use crate::download::manager::DownloadManager;
use crate::error::{Result, StorageError};
use crate::media;
use crate::storage::config::StorageProviderType;
use crate::storage::r2::R2Storage;
use crate::storage::StorageProvider;

/// Execute the process command
///
/// # Errors
///
/// Returns an error if processing fails.
#[allow(clippy::missing_panics_doc)] // Template string is compile-time constant
pub async fn execute(args: ProcessArgs, config: &AppConfig) -> Result<()> {
    // Handle dry-run mode
    if args.dry_run {
        return execute_dry_run(&args);
    }

    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    // Determine if input is a URL or local file
    let media_url = if args.input.starts_with("http://") || args.input.starts_with("https://") {
        Url::parse(&args.input)?
    } else {
        // Local file - upload to cloud storage first
        let path = Path::new(&args.input);
        if !path.exists() {
            return Err(crate::error::AppError::User(format!(
                "File not found: {}",
                args.input
            )));
        }

        upload_local_file(path, config).await?
    };

    process_with_url(&client, &args, config, media_url).await
}

/// Execute dry-run mode: estimate cost without submitting
fn execute_dry_run(args: &ProcessArgs) -> Result<()> {
    // Check if input is a URL (can't estimate without file)
    if args.input.starts_with("http://") || args.input.starts_with("https://") {
        return Err(crate::error::AppError::User(
            "Cannot estimate cost for URLs. --dry-run only works with local files.".to_string(),
        ));
    }

    let path = Path::new(&args.input);
    if !path.exists() {
        return Err(crate::error::AppError::User(format!(
            "File not found: {}",
            args.input
        )));
    }

    // Get file duration
    println!(
        "{} {}",
        console::style("Analyzing").dim(),
        console::style(path.file_name().unwrap_or_default().to_string_lossy()).cyan()
    );

    let duration_secs = media::get_duration(path)?;
    let duration_mins = duration_secs / 60.0;

    println!(
        "{} {}",
        console::style("Duration:").dim(),
        console::style(media::format_duration(duration_secs)).cyan()
    );

    // Calculate cost for each model
    println!();
    println!("{}", console::style("Cost Estimate").bold());
    println!("{}", "-".repeat(50));

    let is_hq = args.hq
        || args.variant.as_deref() == Some("hq")
        || args.variant.as_deref() == Some("high_quality");
    let mut total_credits = 0.0;

    for model in &args.models {
        let credits_per_min = model.credits_per_minute(is_hq);
        let model_credits = duration_mins * credits_per_min;
        total_credits += model_credits;

        let model_display = match (&args.variant, is_hq && model.supports_hq()) {
            (Some(v), false) if model.supports_variant() => {
                format!("{} ({v})", model.api_name())
            }
            (_, true) => format!("{} (HQ)", model.api_name()),
            _ => model.api_name().to_string(),
        };

        println!(
            "  {model_display:<30} {model_credits:>6.1} credits  ({credits_per_min:.1} credits/min × {duration_mins:.1} min)"
        );
    }

    println!("{}", "-".repeat(50));
    println!(
        "  {:<30} {:>6.1} credits",
        console::style("TOTAL").bold(),
        total_credits
    );

    println!();
    println!(
        "{}",
        console::style("Run without --dry-run to submit the task.").dim()
    );

    Ok(())
}

/// Upload a local file to cloud storage and return the public URL
async fn upload_local_file(path: &Path, config: &AppConfig) -> Result<Url> {
    let storage_config = config.storage.as_ref().ok_or(StorageError::NotConfigured)?;

    // Get file size for progress display
    let metadata = std::fs::metadata(path)?;
    let size_str = humansize::format_size(metadata.len(), humansize::BINARY);

    let provider_name = match storage_config.provider {
        StorageProviderType::R2 => "Cloudflare R2",
        StorageProviderType::S3 => "AWS S3",
        StorageProviderType::B2 => "Backblaze B2",
        StorageProviderType::None => return Err(StorageError::NotConfigured.into()),
    };

    println!(
        "{} {} ({}) to {}",
        console::style("Uploading").dim(),
        console::style(path.file_name().unwrap_or_default().to_string_lossy()).cyan(),
        size_str,
        provider_name
    );

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(format!("Uploading to {provider_name}..."));

    let url = match storage_config.provider {
        StorageProviderType::R2 => {
            let storage = R2Storage::new(storage_config)?;
            storage.upload(path).await?
        }
        StorageProviderType::S3 | StorageProviderType::B2 => {
            return Err(crate::error::AppError::User(format!(
                "Storage provider '{}' is not yet implemented. Use 'r2' for Cloudflare R2.",
                storage_config.provider
            )));
        }
        StorageProviderType::None => return Err(StorageError::NotConfigured.into()),
    };

    spinner.finish_and_clear();

    println!(
        "{} {} ({})",
        console::style("✓").green().bold(),
        console::style("Upload complete").green(),
        console::style(url.as_str()).dim()
    );

    Ok(url)
}

async fn process_with_url(
    client: &ApiClient,
    args: &ProcessArgs,
    _config: &AppConfig,
    media_url: Url,
) -> Result<()> {
    // Build request
    let mut request = CreateTaskRequest::new(media_url);

    if let Some(callback) = &args.callback_url {
        request = request.with_callback(callback.clone());
    }

    // Add targets
    let formats: Vec<String> = args
        .formats
        .iter()
        .map(|f| f.api_name().to_string())
        .collect();

    for model in &args.models {
        let mut target = TaskTarget::new(model.api_name(), formats.clone());

        // Handle variant: explicit --variant takes precedence over --hq shorthand
        if let Some(variant) = &args.variant {
            target = target.with_variant(variant);
        } else if args.hq && model.supports_hq() {
            target = target.with_hq();
        }

        if let Some(lang) = &args.language {
            target = target.with_language(lang);
        }

        if args.residual {
            target = target.with_residual();
        }

        request = request.with_target(target);
    }

    // Create task
    println!(
        "{} {}",
        console::style("Creating task with").dim(),
        console::style(format!("{} target(s)", request.targets.len())).cyan()
    );

    let task = client.create_task(request).await?;

    println!(
        "{} {}",
        console::style("Task created:").green().bold(),
        console::style(&task.id).cyan()
    );

    // Wait for completion if requested
    if args.wait {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message("Processing...");

        let task = client
            .wait_for_task(
                &task.id,
                Duration::from_secs(args.poll_interval),
                Duration::from_secs(args.timeout),
                |t| {
                    spinner.set_message(format!("Status: {}", t.effective_status()));
                },
            )
            .await?;

        spinner.finish_and_clear();

        match task.effective_status() {
            TaskStatus::Completed => {
                println!(
                    "{} Task completed successfully!",
                    console::style("✓").green().bold()
                );

                // Download results
                if !args.no_download {
                    let output_dir = &args.output_dir;
                    println!(
                        "{} {}",
                        console::style("Downloading to").dim(),
                        console::style(output_dir.display()).cyan()
                    );

                    let manager = DownloadManager::new(output_dir.clone(), false);
                    let files = manager.download_task(&task).await?;

                    for file in files {
                        println!("  {} {}", console::style("→").green(), file.display());
                    }
                }
            }
            TaskStatus::Failed => {
                let error_msg = task
                    .error
                    .map_or_else(|| "Unknown error".to_string(), |e| e.message);
                println!(
                    "{} Task failed: {}",
                    console::style("✗").red().bold(),
                    error_msg
                );
            }
            _ => {}
        }
    }

    Ok(())
}
