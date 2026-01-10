//! Shell completion generation

use std::io;

use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::args::Cli;
use crate::error::Result;

/// Execute completion command
///
/// # Errors
///
/// This function is infallible but returns Result for consistency.
#[allow(clippy::unnecessary_wraps)]
pub fn execute(shell: clap_complete::Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    generate(shell, &mut cmd, name, &mut io::stdout());

    Ok(())
}
