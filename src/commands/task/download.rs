//! Task download command

use std::path::PathBuf;

use crate::api::client::ApiClient;
use crate::api::response::TaskStatus;
use crate::config::types::AppConfig;
use crate::download::manager::DownloadManager;
use crate::error::Result;

/// Execute task download command
pub async fn execute(
    task_id: &str,
    output_dir: PathBuf,
    _model: Option<String>,
    overwrite: bool,
    config: &AppConfig,
) -> Result<()> {
    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    let task = client.get_task(task_id).await?;

    if task.status != TaskStatus::Completed {
        return Err(crate::error::AppError::User(format!(
            "Task is not completed. Current status: {}",
            task.status
        )));
    }

    println!(
        "{} {}",
        console::style("Downloading to").dim(),
        console::style(output_dir.display()).cyan()
    );

    let manager = DownloadManager::new(output_dir, overwrite);
    let files = manager.download_task(&task).await?;

    println!();
    for file in files {
        println!(
            "  {} {}",
            console::style("✓").green(),
            file.display()
        );
    }

    Ok(())
}
