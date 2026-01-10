//! Error types for the `AudioShake` CLI

use std::path::PathBuf;
use thiserror::Error;

/// Top-level application error
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Download error: {0}")]
    Download(#[from] DownloadError),

    #[error("Tunnel error: {0}")]
    Tunnel(#[from] TunnelError),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    User(String),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(
        "API key not configured. Set AUDIOSHAKE_API_KEY environment variable or use --api-key"
    )]
    MissingApiKey,

    #[error("Invalid configuration file: {0}")]
    #[allow(dead_code)]
    InvalidFile(String),

    #[error("Configuration file not found: {path}")]
    #[allow(dead_code)]
    NotFound { path: PathBuf },

    #[error("Failed to read environment: {0}")]
    EnvError(#[from] dotenvy::Error),

    #[error("Failed to parse TOML: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// API client errors
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API returned error {status}: {message}")]
    Response { status: u16, message: String },

    #[error("Authentication failed: Invalid API key")]
    Unauthorized,

    #[error("Rate limit exceeded. Please wait and retry.")]
    RateLimited,

    #[error("Insufficient credits. Please add credits to your account.")]
    InsufficientCredits,

    #[error("Task not found: {id}")]
    #[allow(dead_code)]
    TaskNotFound { id: String },

    #[error("Invalid response format: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Request timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
}

impl ApiError {
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited)
    }
}

/// Download-related errors
#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Failed to download {url}: {reason}")]
    Failed { url: String, reason: String },

    #[error("Output directory does not exist: {path}")]
    #[allow(dead_code)]
    DirectoryNotFound { path: PathBuf },

    #[error("File exists and --overwrite not specified: {path}")]
    FileExists { path: PathBuf },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Tunnel-related errors
#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("cloudflared not found. Install with: brew install cloudflared")]
    CloudflaredNotFound,

    #[error("Failed to start tunnel: {0}")]
    StartFailed(String),

    #[error("Tunnel terminated unexpectedly")]
    Terminated,

    #[error("Failed to parse tunnel URL")]
    UrlParseFailed,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, AppError>;
