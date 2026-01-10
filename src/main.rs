//! `AudioShake` CLI - Command-line interface for the `AudioShake` API
//!
//! This CLI provides access to `AudioShake`'s audio processing capabilities including:
//! - Stem separation (vocals, drums, bass, guitar, etc.)
//! - Transcription and alignment
//! - Dialogue/music/effects separation

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod api;
mod cli;
mod commands;
mod config;
mod download;
mod error;
mod media;
mod output;
mod storage;

use cli::args::{Cli, Commands};
use error::Result;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{}: {}", console::style("error").red().bold(), err);

        // Print cause chain for verbose mode
        if std::env::var("RUST_BACKTRACE").is_ok() {
            use std::error::Error;
            let mut source = err.source();
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }
        }

        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    // Load configuration
    let config = config::loader::ConfigLoader::load(&cli)?;

    // Execute command
    match cli.command {
        Commands::Process(args) => commands::process::execute(args, &config).await,
        Commands::Task { command } => commands::task::execute(command, &config).await,
        Commands::Models(ref args) => commands::models::execute(args, &config),
        Commands::Config { command } => commands::config::execute(command, &config).await,
        Commands::Completion { shell } => commands::completion::execute(shell),
    }
}

fn init_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).without_time())
        .with(filter)
        .init();
}
