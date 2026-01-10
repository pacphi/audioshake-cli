//! Task list command

use comfy_table::{presets::UTF8_FULL, Table};

use crate::api::client::ApiClient;
use crate::config::types::AppConfig;
use crate::error::Result;

/// Execute task list command
pub async fn execute(
    status: Option<String>,
    limit: u32,
    offset: u32,
    _since: Option<String>,
    _before: Option<String>,
    config: &AppConfig,
) -> Result<()> {
    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    let response = client.list_tasks(status.as_deref(), limit, offset).await?;

    if response.tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Status", "Created", "Targets"]);

    for task in response.tasks {
        let target_count = task.targets.len();
        table.add_row(vec![
            task.id.clone(),
            task.effective_status().to_string(),
            task.created_at.format("%Y-%m-%d %H:%M").to_string(),
            format!("{target_count}"),
        ]);
    }

    println!("{table}");
    println!(
        "\nShowing {} of {} tasks",
        response.limit.min(response.total),
        response.total
    );

    Ok(())
}
