//! Task status command

use crate::api::client::ApiClient;
use crate::api::response::TaskStatus;
use crate::config::types::AppConfig;
use crate::error::Result;

/// Execute task status command
pub async fn execute(task_id: &str, config: &AppConfig) -> Result<()> {
    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    let task = client.get_task(task_id).await?;

    println!("Task: {}", console::style(&task.id).cyan());
    let status = task.effective_status();
    println!(
        "Status: {}",
        match status {
            TaskStatus::Completed => console::style(status.as_str()).green(),
            TaskStatus::Failed => console::style(status.as_str()).red(),
            TaskStatus::Processing => console::style(status.as_str()).yellow(),
            TaskStatus::Pending => console::style(status.as_str()).dim(),
        }
    );
    println!(
        "Created: {}",
        task.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    if let Some(completed) = task.completed_at {
        println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S UTC"));

        let duration = completed - task.created_at;
        println!("Duration: {}s", duration.num_seconds());
    }

    if let Some(cost) = &task.cost {
        println!(
            "Cost: {} credits ({:.1} minutes)",
            cost.credits, cost.duration_minutes
        );
    }

    println!("\nTargets:");
    for target in &task.targets {
        let status_icon = match target.status {
            Some(TaskStatus::Completed) => console::style("[OK]").green(),
            Some(TaskStatus::Failed) => console::style("[FAIL]").red(),
            Some(TaskStatus::Processing) => console::style("[...]").yellow(),
            _ => console::style("[--]").dim(),
        };

        print!("  {} {}", status_icon, target.model);

        for link in &target.output {
            let name = link.name.as_deref().unwrap_or(&link.format);
            if let Some(size) = link.size {
                let size_str = humansize::format_size(size, humansize::BINARY);
                print!(" → {}.{} ({})", name, link.format, size_str);
            } else {
                print!(" → {}.{}", name, link.format);
            }
        }

        if let Some(error) = &target.error {
            print!(" - {}", console::style(error).red());
        }

        println!();
    }

    if let Some(error) = &task.error {
        println!(
            "\n{}: {} - {}",
            console::style("Error").red().bold(),
            error.code,
            error.message
        );
    }

    Ok(())
}
