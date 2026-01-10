//! Task wait command

use std::path::PathBuf;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use crate::api::client::ApiClient;
use crate::api::response::TaskStatus;
use crate::config::types::AppConfig;
use crate::download::manager::DownloadManager;
use crate::error::Result;

/// Execute task wait command
pub async fn execute(
    task_id: &str,
    timeout: u64,
    poll_interval: u64,
    download: bool,
    output_dir: PathBuf,
    config: &AppConfig,
) -> Result<()> {
    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(format!("Waiting for task {task_id}..."));

    let task = client
        .wait_for_task(
            task_id,
            Duration::from_secs(poll_interval),
            Duration::from_secs(timeout),
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

            if download {
                println!(
                    "{} {}",
                    console::style("Downloading to").dim(),
                    console::style(output_dir.display()).cyan()
                );

                let manager = DownloadManager::new(output_dir, false);
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

    Ok(())
}
