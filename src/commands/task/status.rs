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
    println!(
        "Status: {}",
        match task.status {
            TaskStatus::Completed => console::style(task.status.as_str()).green(),
            TaskStatus::Failed => console::style(task.status.as_str()).red(),
            TaskStatus::Processing => console::style(task.status.as_str()).yellow(),
            TaskStatus::Pending => console::style(task.status.as_str()).dim(),
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

        if let Some(output) = &target.output {
            for link in &output.links {
                if let Some(size) = link.size {
                    let size_str = humansize::format_size(size, humansize::BINARY);
                    print!(" → {} ({})", link.format, size_str);
                } else {
                    print!(" → {}", link.format);
                }
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
