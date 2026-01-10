//! Task statistics command

#![allow(clippy::cast_precision_loss)]

use crate::api::client::ApiClient;
use crate::config::types::AppConfig;
use crate::error::Result;

/// Execute task statistics command
pub async fn execute(
    _since: Option<String>,
    _before: Option<String>,
    config: &AppConfig,
) -> Result<()> {
    let client = ApiClient::new(config.base_url.clone(), config.api_key.clone())?;

    let stats = client.get_statistics("usage").await?;

    println!("{}", console::style("Task Statistics").bold());
    println!("{}", "-".repeat(20));
    println!(
        "Total Tasks:     {}",
        console::style(stats.total_tasks).cyan()
    );

    let completed_pct = if stats.total_tasks > 0 {
        (stats.completed_tasks as f64 / stats.total_tasks as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "Completed:       {} ({:.1}%)",
        console::style(stats.completed_tasks).green(),
        completed_pct
    );

    let failed_pct = if stats.total_tasks > 0 {
        (stats.failed_tasks as f64 / stats.total_tasks as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "Failed:          {} ({:.1}%)",
        console::style(stats.failed_tasks).red(),
        failed_pct
    );

    println!(
        "Credits Used:    {}",
        console::style(format!("{:.1}", stats.credits_used)).yellow()
    );

    println!(
        "Total Duration:  {} minutes",
        console::style(format!("{:.0}", stats.total_duration_minutes)).dim()
    );

    Ok(())
}
