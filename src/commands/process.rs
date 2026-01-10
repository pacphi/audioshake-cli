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
use crate::error::Result;
use crate::tunnel::TunnelService;

/// Execute the process command
///
/// # Errors
///
/// Returns an error if processing fails.
pub async fn execute(args: ProcessArgs, config: &AppConfig) -> Result<()> {
    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    // Determine if input is a URL or local file
    let media_url = if args.input.starts_with("http://") || args.input.starts_with("https://") {
        Url::parse(&args.input)?
    } else {
        // Local file - start tunnel
        let path = Path::new(&args.input);
        if !path.exists() {
            return Err(crate::error::AppError::User(format!(
                "File not found: {}",
                args.input
            )));
        }

        println!(
            "{}",
            console::style("Starting local file server...").dim()
        );

        let mut tunnel = TunnelService::new(path.to_path_buf());
        let url = tunnel.start().await?;

        println!(
            "{} {}",
            console::style("Tunnel URL:").dim(),
            console::style(&url).cyan()
        );

        // Process with tunnel URL
        let result = process_with_url(&client, &args, config, url.clone()).await;

        // Stop tunnel
        tunnel.stop().await;

        return result;
    };

    process_with_url(&client, &args, config, media_url).await
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
    let formats: Vec<String> = args.formats.iter().map(|f| f.api_name().to_string()).collect();

    for model in &args.models {
        let mut target = TaskTarget::new(model.api_name(), formats.clone());

        if args.hq && model.supports_hq() {
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
                    spinner.set_message(format!("Status: {}", t.status));
                },
            )
            .await?;

        spinner.finish_and_clear();

        match task.status {
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
                        println!(
                            "  {} {}",
                            console::style("→").green(),
                            file.display()
                        );
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
