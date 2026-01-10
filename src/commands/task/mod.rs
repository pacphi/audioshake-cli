//! Task command implementations

use crate::cli::args::TaskCommands;
use crate::config::types::AppConfig;
use crate::error::Result;

mod download;
mod list;
mod statistics;
mod status;
mod wait;

/// Execute task subcommand
///
/// # Errors
///
/// Returns an error if the command fails.
pub async fn execute(command: TaskCommands, config: &AppConfig) -> Result<()> {
    match command {
        TaskCommands::List {
            status,
            limit,
            offset,
            since,
            before,
        } => list::execute(status, limit, offset, since, before, config).await,
        TaskCommands::Status { task_id } => status::execute(&task_id, config).await,
        TaskCommands::Download {
            task_id,
            output_dir,
            model,
            overwrite,
        } => download::execute(&task_id, output_dir, model, overwrite, config).await,
        TaskCommands::Wait {
            task_id,
            timeout,
            poll_interval,
            download,
            output_dir,
        } => wait::execute(&task_id, timeout, poll_interval, download, output_dir, config).await,
        TaskCommands::Statistics { since, before } => {
            statistics::execute(since, before, config).await
        }
    }
}
