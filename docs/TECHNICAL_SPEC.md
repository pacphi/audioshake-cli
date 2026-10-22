# AudioShake CLI - Technical Specification

**Version:** 1.0.0
**Date:** January 2026

---

## Architecture Overview

The AudioShake CLI is built using a modular architecture with clear separation of concerns:

```text
┌─────────────────────────────────────────────────────────────────┐
│                         main.rs                                 │
│                    (Entry Point & CLI Setup)                    │
└───────────────────────────┬─────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                        cli/args.rs                              │
│                  (Clap Argument Definitions)                    │
└───────────────────────────┬─────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                       commands/                                 │
│    ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│    │ process  │  │   task   │  │  models  │  │  config  │       │
│    └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
└─────────┼─────────────┼─────────────┼─────────────┼─────────────┘
          │             │             │             │
┌─────────▼─────────────▼─────────────▼─────────────▼─────────────┐
│                         Core Services                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐ │
│  │ api/client │  │  config/   │  │  storage/  │  │  download/ │ │
│  │            │  │  loader    │  │  r2        │  │  manager   │ │
│  └────────────┘  └────────────┘  └────────────┘  └────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Project Structure

```text
audioshake-cli/
├── Cargo.toml                 # Project manifest
├── Cargo.lock                 # Dependency lock file
├── .env.example               # Example environment file
├── .gitignore
├── README.md
├── LICENSE
│
├── src/
│   ├── main.rs                # Entry point
│   ├── lib.rs                 # Library exports for testing
│   │
│   ├── cli/                   # CLI argument definitions
│   │   ├── mod.rs
│   │   ├── args.rs            # Top-level CLI struct
│   │   ├── process.rs         # Process command args
│   │   ├── task.rs            # Task command args
│   │   ├── models.rs          # Models command args
│   │   ├── config.rs          # Config command args
│   │   └── completion.rs      # Completion command args
│   │
│   ├── commands/              # Command implementations
│   │   ├── mod.rs
│   │   ├── process.rs         # Process command logic
│   │   ├── task/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs
│   │   │   ├── status.rs
│   │   │   ├── download.rs
│   │   │   ├── wait.rs
│   │   │   └── statistics.rs
│   │   ├── models.rs
│   │   ├── config.rs
│   │   └── completion.rs
│   │
│   ├── api/                   # API client
│   │   ├── mod.rs
│   │   ├── client.rs          # HTTP client for Tasks API
│   │   ├── request.rs         # Request types
│   │   └── response.rs        # Response types
│   │
│   ├── storage/               # Cloud storage for local files
│   │   ├── mod.rs             # StorageProvider trait
│   │   ├── config.rs          # Storage configuration types
│   │   └── r2.rs              # Cloudflare R2 implementation
│   │
│   ├── config/                # Configuration
│   │   ├── mod.rs
│   │   ├── loader.rs          # Config loading logic
│   │   └── types.rs           # Config types
│   │
│   ├── download/              # File downloads
│   │   ├── mod.rs
│   │   └── manager.rs         # Download orchestration
│   │
│   ├── output/                # Output formatting
│   │   ├── mod.rs
│   │   ├── text.rs            # Human-readable output
│   │   ├── json.rs            # JSON output
│   │   ├── yaml.rs            # YAML output
│   │   ├── progress.rs        # Progress indicators
│   │   └── table.rs           # Table formatting
│   │
│   ├── error.rs               # Error types
│   └── utils.rs               # Utility functions
│
├── tests/                     # Integration tests
│   ├── common/
│   │   ├── mod.rs
│   │   └── mock_server.rs
│   ├── process_test.rs
│   ├── task_test.rs
│   └── config_test.rs
│
└── docs/                      # Documentation
    ├── PRD.md
    ├── CLI_REFERENCE.md
    ├── TECHNICAL_SPEC.md
    └── API_INTEGRATION.md
```

---

## Dependencies

### Cargo.toml

```toml
[package]
name = "audioshake-cli"
version = "1.0.0"
edition = "2024"
authors = ["AudioShake CLI Contributors"]
description = "Command-line interface for the AudioShake audio processing API"
license = "MIT"
repository = "https://github.com/pacphi/audioshake-cli"
keywords = ["audio", "stems", "separation", "transcription", "cli"]
categories = ["command-line-utilities", "multimedia::audio"]
rust-version = "1.99"

[dependencies]
# CLI Framework
clap = { version = "4.5", features = ["derive", "env", "wrap_help", "color"] }
clap_complete = "4.5"

# Async Runtime
tokio = { version = "1.43", features = [
    "rt-multi-thread",
    "macros",
    "fs",
    "signal",
    "process",
    "net"
] }

# HTTP Client
reqwest = { version = "0.12", default-features = false, features = [
    "json",
    "stream",
    "rustls-tls",
    "gzip"
] }

# Cloud Storage (S3-compatible)
aws-config = { version = "1.6", features = ["behavior-version-latest"] }
aws-sdk-s3 = "1.82"
aws-credential-types = "1.2"
tower-http = { version = "0.6", features = ["fs", "cors"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"

# Environment & Configuration
dotenvy = "0.15"
directories = "5.0"

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Progress & Output
indicatif = { version = "0.17", features = ["tokio"] }
console = "0.15"
comfy-table = "7.1"

# Utilities
url = { version = "2.5", features = ["serde"] }
chrono = { version = "0.4", features = ["serde"] }
humantime = "2.1"
humansize = "2.1"
futures = "0.3"
async-trait = "0.1"
uuid = { version = "1.11", features = ["v4"] }
rand = "0.8"

# Security
secrecy = { version = "0.10", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Streaming
tokio-stream = "0.1"
futures-util = "0.3"

[dev-dependencies]
tokio-test = "0.4"
wiremock = "0.6"
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.16"
insta = { version = "1.42", features = ["yaml"] }

[profile.release]
lto = true
strip = true
codegen-units = 1
panic = "abort"
opt-level = 3

[profile.dev]
opt-level = 1

[[bin]]
name = "audioshake"
path = "src/main.rs"
```

---

## Core Types

### Error Types (src/error.rs)

```rust
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

    #[error("{0}")]
    User(String),
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("API key not configured. Set AUDIOSHAKE_API_KEY or use --api-key")]
    MissingApiKey,

    #[error("Invalid configuration file: {0}")]
    InvalidFile(String),

    #[error("Configuration file not found: {path}")]
    NotFound { path: PathBuf },

    #[error("Failed to read environment: {0}")]
    EnvError(#[from] dotenvy::Error),
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
    TaskNotFound { id: String },

    #[error("Invalid response format: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Request timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
}

/// Download errors
#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Failed to download {url}: {reason}")]
    Failed { url: String, reason: String },

    #[error("Output directory does not exist: {path}")]
    DirectoryNotFound { path: PathBuf },

    #[error("File exists and --overwrite not specified: {path}")]
    FileExists { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, AppError>;
```

### API Request Types (src/api/request.rs)

```rust
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<Url>,
    pub targets: Vec<TaskTarget>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTarget {
    pub model: Model,
    pub formats: Vec<OutputFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ModelVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
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
    // DMX
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Wav,
    Mp3,
    Flac,
    Aiff,
    Txt,
    Json,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelVariant {
    HighQuality,
}

impl Model {
    pub fn category(&self) -> ModelCategory {
        match self {
            Model::Vocals | Model::Instrumental | Model::Drums | Model::Bass
            | Model::Guitar | Model::Piano | Model::Strings | Model::Wind
            | Model::Other => ModelCategory::Stems,
            Model::Dialogue | Model::Effects | Model::MusicDetection
            | Model::Background | Model::MusicRemoval => ModelCategory::Dmx,
            Model::TwoSpeaker | Model::NSpeaker => ModelCategory::MultiVoice,
            Model::Transcription | Model::Alignment => ModelCategory::Transcription,
        }
    }

    pub fn supports_hq(&self) -> bool {
        matches!(self, Model::Vocals | Model::Instrumental)
    }

    pub fn credits_per_minute(&self) -> f64 {
        match self {
            Model::Vocals | Model::Instrumental | Model::Drums | Model::Bass
            | Model::Guitar | Model::Piano | Model::Strings | Model::Wind
            | Model::Other | Model::Transcription | Model::Alignment => 1.0,
            Model::Dialogue | Model::Effects | Model::Background => 1.5,
            Model::MusicDetection => 0.5,
            Model::MusicRemoval | Model::TwoSpeaker | Model::NSpeaker => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCategory {
    Stems,
    Dmx,
    MultiVoice,
    Transcription,
}
```

### API Response Types (src/api/response.rs)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub targets: Vec<TaskTargetResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TaskError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<TaskCost>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Failed)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTargetResult {
    pub model: String,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<TaskOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskOutput {
    pub links: Vec<OutputLink>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputLink {
    pub format: String,
    pub link: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCost {
    pub credits: f64,
    pub duration_minutes: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
    pub total: u64,
    #[serde(default)]
    pub offset: u64,
    #[serde(default)]
    pub limit: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatistics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub credits_used: f64,
    pub total_duration_minutes: f64,
}
```

---

## API Client Implementation

### Client Structure (src/api/client.rs)

```rust
use reqwest::{Client, Response, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use tracing::{debug, instrument};
use url::Url;

use super::request::CreateTaskRequest;
use super::response::{Task, TaskListResponse, TaskStatistics, TaskStatus};
use crate::error::{ApiError, Result};

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION")
);

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: Url,
    api_key: SecretString,
}

impl ApiClient {
    pub fn new(base_url: Url, api_key: SecretString) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(ApiError::from)?;

        Ok(Self { client, base_url, api_key })
    }

    #[instrument(skip(self, request))]
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task> {
        let url = self.base_url.join("/tasks")?;
        debug!("Creating task with {} targets", request.targets.len());

        let response = self.client
            .post(url)
            .header("x-api-key", self.api_key.expose_secret())
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        let url = self.base_url.join(&format!("/tasks/{}", task_id))?;

        let response = self.client
            .get(url)
            .header("x-api-key", self.api_key.expose_secret())
            .send()
            .await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn list_tasks(
        &self,
        status: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<TaskListResponse> {
        let mut url = self.base_url.join("/tasks")?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("take", &limit.to_string());
            query.append_pair("skip", &offset.to_string());
            if let Some(s) = status {
                query.append_pair("status", s);
            }
        }

        let response = self.client
            .get(url)
            .header("x-api-key", self.api_key.expose_secret())
            .send()
            .await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_statistics(&self, name: &str) -> Result<TaskStatistics> {
        let mut url = self.base_url.join("/tasks/statistics")?;
        url.query_pairs_mut().append_pair("name", name);

        let response = self.client
            .get(url)
            .header("x-api-key", self.api_key.expose_secret())
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn wait_for_task(
        &self,
        task_id: &str,
        poll_interval: Duration,
        timeout: Duration,
        mut on_status: impl FnMut(&Task),
    ) -> Result<Task> {
        let start = std::time::Instant::now();

        loop {
            let task = self.get_task(task_id).await?;
            on_status(&task);

            if task.status.is_terminal() {
                return Ok(task);
            }

            if start.elapsed() > timeout {
                return Err(ApiError::Timeout {
                    seconds: timeout.as_secs(),
                }.into());
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await.map_err(ApiError::from)?)
        } else {
            let message = response.text().await.unwrap_or_default();

            match status {
                StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized.into()),
                StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimited.into()),
                StatusCode::BAD_REQUEST if message.contains("insufficient credits") => {
                    Err(ApiError::InsufficientCredits.into())
                }
                _ => Err(ApiError::Response {
                    status: status.as_u16(),
                    message,
                }.into()),
            }
        }
    }
}
```

---

## Cloud Storage Upload

Local files are uploaded to user-configured cloud storage (Cloudflare R2) before processing.
The AudioShake Tasks API requires publicly accessible URLs, so files must be hosted externally.
See [ADR-002](adr/002-cloud-storage-upload-strategy.md) for the decision rationale.

### Storage Module (src/storage/)

```rust
// src/storage/mod.rs
/// Trait for cloud storage providers
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Upload a file and return its public URL
    async fn upload(&self, path: &Path) -> Result<Url>;

    /// Delete an object by key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Get the provider name for display purposes
    fn provider_name(&self) -> &'static str;
}
```

### R2 Implementation (src/storage/r2.rs)

```rust
use aws_sdk_s3::Client;

pub struct R2Storage {
    client: Client,
    bucket: String,
    public_url_base: Url,
}

impl R2Storage {
    pub fn new(config: &StorageConfig) -> Result<Self> {
        let credentials = Credentials::new(
            &r2_config.access_key_id,
            r2_config.secret_access_key.expose_secret(),
            None, None, "r2-storage",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("auto"))
            .endpoint_url(r2_config.endpoint_url())
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);
        // ...
    }
}

#[async_trait]
impl StorageProvider for R2Storage {
    async fn upload(&self, path: &Path) -> Result<Url> {
        let key = Self::generate_key(path);
        let body = ByteStream::from_path(path).await?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .content_type(Self::mime_type(path))
            .send()
            .await?;

        self.public_url(&key)
    }
}
```

### Configuration

Storage configuration is loaded from environment variables or config file:

```bash
# Environment variables
AUDIOSHAKE_STORAGE_PROVIDER=r2
AUDIOSHAKE_STORAGE_BUCKET=my-bucket
AUDIOSHAKE_STORAGE_PUBLIC_URL_BASE=https://my-bucket.r2.dev
AUDIOSHAKE_R2_ACCOUNT_ID=abc123
AUDIOSHAKE_R2_ACCESS_KEY_ID=xxx
AUDIOSHAKE_R2_SECRET_ACCESS_KEY=xxx
```

```toml
# ~/.config/audioshake/config.toml
[storage]
provider = "r2"
bucket = "my-bucket"
public_url_base = "https://my-bucket.r2.dev"

[storage.r2]
account_id = "abc123"
access_key_id = "xxx"
```

---

## Download Manager

### Implementation (src/download/manager.rs)

```rust
use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;
use url::Url;

use crate::api::response::{OutputLink, Task};
use crate::error::{DownloadError, Result};

pub struct DownloadManager {
    client: Client,
    output_dir: PathBuf,
    overwrite: bool,
    progress: MultiProgress,
}

impl DownloadManager {
    pub fn new(output_dir: PathBuf, overwrite: bool) -> Self {
        Self {
            client: Client::new(),
            output_dir,
            overwrite,
            progress: MultiProgress::new(),
        }
    }

    pub async fn download_task(&self, task: &Task) -> Result<Vec<PathBuf>> {
        // Ensure output directory exists
        tokio::fs::create_dir_all(&self.output_dir).await?;

        let mut downloaded = Vec::new();
        let mut handles = Vec::new();

        for target in &task.targets {
            if let Some(output) = &target.output {
                for link in &output.links {
                    let path = self.output_path(&target.model, &link.format);

                    if path.exists() && !self.overwrite {
                        return Err(DownloadError::FileExists { path }.into());
                    }

                    let client = self.client.clone();
                    let url = link.link.clone();
                    let size = link.size;
                    let bar = self.create_progress_bar(&path, size);

                    let handle = tokio::spawn(async move {
                        Self::download_file(&client, &url, &path, bar).await
                    });

                    handles.push((path, handle));
                }
            }
        }

        for (path, handle) in handles {
            handle.await.map_err(|e| DownloadError::Failed {
                url: path.display().to_string(),
                reason: e.to_string(),
            })??;
            downloaded.push(path);
        }

        Ok(downloaded)
    }

    async fn download_file(
        client: &Client,
        url: &Url,
        path: &Path,
        progress: ProgressBar,
    ) -> Result<()> {
        let response = client.get(url.as_str()).send().await?;
        let total_size = response.content_length();

        if let Some(size) = total_size {
            progress.set_length(size);
        }

        let mut file = File::create(path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| DownloadError::Failed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

            file.write_all(&chunk).await?;
            progress.inc(chunk.len() as u64);
        }

        file.flush().await?;
        progress.finish_with_message("done");

        info!("Downloaded: {}", path.display());
        Ok(())
    }

    fn output_path(&self, model: &str, format: &str) -> PathBuf {
        self.output_dir.join(format!("{}.{}", model, format))
    }

    fn create_progress_bar(&self, path: &Path, size: Option<u64>) -> ProgressBar {
        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let bar = match size {
            Some(s) => {
                let bar = self.progress.add(ProgressBar::new(s));
                bar.set_style(
                    ProgressStyle::default_bar()
                        .template("{msg:>20} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .unwrap()
                        .progress_chars("=>-"),
                );
                bar
            }
            None => {
                let bar = self.progress.add(ProgressBar::new_spinner());
                bar.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner} {msg} {bytes}")
                        .unwrap(),
                );
                bar
            }
        };

        bar.set_message(filename.to_string());
        bar
    }
}
```

---

## Testing Strategy

### Unit Tests

Each module includes inline unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_category() {
        assert_eq!(Model::Vocals.category(), ModelCategory::Stems);
        assert_eq!(Model::Dialogue.category(), ModelCategory::Dmx);
        assert_eq!(Model::Transcription.category(), ModelCategory::Transcription);
    }

    #[test]
    fn test_task_status_terminal() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::Processing.is_terminal());
    }
}
```

### Integration Tests with Mock Server

```rust
// tests/api_test.rs
use audioshake_cli::api::client::ApiClient;
use secrecy::SecretString;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{header, method, path};

#[tokio::test]
async fn test_create_task_success() {
    let mock_server = MockServer::start().await;

    let response = serde_json::json!({
        "id": "task-123",
        "status": "pending",
        "createdAt": "2026-01-10T10:30:00Z",
        "targets": []
    });

    Mock::given(method("POST"))
        .and(path("/tasks"))
        .and(header("x-api-key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(&mock_server)
        .await;

    let client = ApiClient::new(
        url::Url::parse(&mock_server.uri()).unwrap(),
        SecretString::from("test_key"),
    ).unwrap();

    // Test implementation...
}
```

### CLI Integration Tests

```rust
// tests/cli_test.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    Command::cargo_bin("audioshake")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AudioShake CLI"));
}

#[test]
fn test_missing_api_key() {
    Command::cargo_bin("audioshake")
        .unwrap()
        .env_remove("AUDIOSHAKE_API_KEY")
        .arg("process")
        .arg("test.mp3")
        .arg("-m")
        .arg("vocals")
        .assert()
        .failure()
        .stderr(predicate::str::contains("API key not configured"));
}
```

---

## Performance Considerations

1. **Async I/O**: All network and file operations use tokio async runtime
2. **Connection Pooling**: reqwest client maintains connection pool
3. **Parallel Downloads**: Multiple output files downloaded concurrently
4. **Streaming**: Large files streamed instead of buffered in memory
5. **Minimal Dependencies**: Only essential crates included
6. **Release Optimizations**: LTO, stripping, single codegen unit

---

## Security Considerations

1. **API Key Protection**: Uses secrecy crate, never logged or displayed
2. **HTTPS Only**: All API communication uses TLS
3. **Cloud Storage Upload**: Files uploaded to user-controlled R2 bucket (zero egress fees)
4. **No Shell Injection**: All subprocess commands use array args
5. **Credential Masking**: `config show` masks sensitive values by default (use `--show-sensitive` to reveal)

---

## References

- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Reqwest Documentation](https://docs.rs/reqwest/latest/reqwest/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [AudioShake Developer Portal](https://developer.audioshake.ai/)
