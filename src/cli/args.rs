//! CLI argument definitions using clap

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use secrecy::SecretString;
use url::Url;

/// `AudioShake` CLI - Command-line interface for the `AudioShake` API
#[derive(Parser, Debug)]
#[command(name = "audioshake")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Increase verbosity level (can be used multiple times: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// API key (overrides `AUDIOSHAKE_API_KEY` environment variable)
    #[arg(
        long,
        env = "AUDIOSHAKE_API_KEY",
        hide_env_values = true,
        global = true
    )]
    pub api_key: Option<SecretString>,

    /// Override API base URL
    #[arg(long, default_value = "https://api.audioshake.ai", global = true)]
    pub base_url: Url,

    /// Output format for command results
    #[arg(long, value_enum, default_value = "text", global = true)]
    pub output_format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Submit audio/video files for processing
    Process(ProcessArgs),

    /// Manage and monitor tasks
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    /// List available processing models
    Models(ModelsArgs),

    /// Manage CLI configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}

/// Output format for CLI results
#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Yaml,
}

/// Process command arguments
#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProcessArgs {
    /// Input file path or HTTPS URL
    #[arg(required = true)]
    pub input: String,

    /// Model(s) to apply (can be repeated)
    #[arg(short, long = "model", required = true)]
    pub models: Vec<Model>,

    /// Output format(s) (can be repeated)
    #[arg(short, long = "format", default_values = ["wav"])]
    pub formats: Vec<AudioFormat>,

    /// Use high-quality variant (1.5 credits/min)
    #[arg(long)]
    pub hq: bool,

    /// Include residual stem
    #[arg(long)]
    pub residual: bool,

    /// Language code for transcription (ISO 639-1)
    #[arg(long)]
    pub language: Option<String>,

    /// Webhook URL for completion notification
    #[arg(long)]
    pub callback_url: Option<Url>,

    /// Output directory
    #[arg(short, long, default_value = "./output")]
    pub output_dir: PathBuf,

    /// Wait for task completion and download results
    #[arg(long)]
    pub wait: bool,

    /// Timeout for --wait in seconds
    #[arg(long, default_value = "3600")]
    pub timeout: u64,

    /// Polling interval for --wait in seconds
    #[arg(long, default_value = "5")]
    pub poll_interval: u64,

    /// With --wait, don't download results automatically
    #[arg(long)]
    pub no_download: bool,
}

/// Available processing models
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    // Stem separation
    Vocals,
    Instrumental,
    Drums,
    Bass,
    Guitar,
    Piano,
    Strings,
    Wind,
    Other,
    // DMX (Dialogue, Music, Effects)
    Dialogue,
    Effects,
    MusicDetection,
    Background,
    MusicRemoval,
    // Multi-voice
    TwoSpeaker,
    NSpeaker,
    // Transcription
    Transcription,
    Alignment,
}

impl Model {
    /// Get the API model name
    #[must_use]
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Vocals => "vocals",
            Self::Instrumental => "instrumental",
            Self::Drums => "drums",
            Self::Bass => "bass",
            Self::Guitar => "guitar",
            Self::Piano => "piano",
            Self::Strings => "strings",
            Self::Wind => "wind",
            Self::Other => "other",
            Self::Dialogue => "dialogue",
            Self::Effects => "effects",
            Self::MusicDetection => "music_detection",
            Self::Background => "background",
            Self::MusicRemoval => "music_removal",
            Self::TwoSpeaker => "two_speaker",
            Self::NSpeaker => "n_speaker",
            Self::Transcription => "transcription",
            Self::Alignment => "alignment",
        }
    }

    /// Check if model supports high-quality variant
    #[must_use]
    pub const fn supports_hq(self) -> bool {
        matches!(self, Self::Vocals | Self::Instrumental)
    }

    /// Get credits per minute
    #[must_use]
    #[allow(dead_code)]
    pub const fn credits_per_minute(self) -> f64 {
        match self {
            Self::Dialogue | Self::Effects | Self::Background => 1.5,
            Self::MusicDetection => 0.5,
            _ => 1.0,
        }
    }
}

/// Output audio formats
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Flac,
    Aiff,
    Txt,
    Json,
}

impl AudioFormat {
    /// Get the API format name
    #[must_use]
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Aiff => "aiff",
            Self::Txt => "txt",
            Self::Json => "json",
        }
    }
}

/// Task subcommands
#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// List tasks with optional filtering
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,

        /// Maximum tasks to return
        #[arg(long, default_value = "20")]
        limit: u32,

        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: u32,

        /// Tasks after date (ISO 8601)
        #[arg(long)]
        since: Option<String>,

        /// Tasks before date (ISO 8601)
        #[arg(long)]
        before: Option<String>,
    },

    /// Get detailed status of a task
    Status {
        /// Task ID
        task_id: String,
    },

    /// Download completed task outputs
    Download {
        /// Task ID
        task_id: String,

        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output_dir: PathBuf,

        /// Download specific model only
        #[arg(long)]
        model: Option<String>,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },

    /// Wait for a task to complete
    Wait {
        /// Task ID
        task_id: String,

        /// Timeout in seconds
        #[arg(long, default_value = "3600")]
        timeout: u64,

        /// Polling interval in seconds
        #[arg(long, default_value = "5")]
        poll_interval: u64,

        /// Download results when complete
        #[arg(long)]
        download: bool,

        /// Directory for downloads
        #[arg(short, long, default_value = "./output")]
        output_dir: PathBuf,
    },

    /// Show task statistics
    Statistics {
        /// Stats from date (ISO 8601)
        #[arg(long)]
        since: Option<String>,

        /// Stats until date (ISO 8601)
        #[arg(long)]
        before: Option<String>,
    },
}

/// Models command arguments
#[derive(Parser, Debug)]
pub struct ModelsArgs {
    /// Filter by category
    #[arg(long)]
    pub category: Option<String>,

    /// Show detailed information including costs
    #[arg(long)]
    pub detailed: bool,
}

/// Config subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Display current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },

    /// Initialize configuration file
    Init {
        /// Overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// Validate API key and connectivity
    Validate,
}
