//! Download manager for task outputs

use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;
use url::Url;

use crate::api::response::Task;
use crate::error::{DownloadError, Result};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Download manager for task outputs
pub struct DownloadManager {
    client: Client,
    output_dir: PathBuf,
    overwrite: bool,
    progress: MultiProgress,
}

impl DownloadManager {
    /// Create a new download manager
    #[must_use]
    pub fn new(output_dir: PathBuf, overwrite: bool) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            output_dir,
            overwrite,
            progress: MultiProgress::new(),
        }
    }

    /// Download all outputs from a completed task
    ///
    /// # Errors
    ///
    /// Returns an error if any download fails.
    pub async fn download_task(&self, task: &Task) -> Result<Vec<PathBuf>> {
        // Ensure output directory exists
        tokio::fs::create_dir_all(&self.output_dir).await?;

        let mut downloaded = Vec::new();

        for target in &task.targets {
            for link in &target.output {
                // Use the output name if available, otherwise fall back to model name
                let name = link.name.as_deref().unwrap_or(&target.model);
                let path = self.output_path(name, &link.format);

                if path.exists() && !self.overwrite {
                    return Err(DownloadError::FileExists { path }.into());
                }

                self.download_file(&link.link, &path, link.size).await?;
                downloaded.push(path);
            }
        }

        Ok(downloaded)
    }

    async fn download_file(&self, url: &Url, path: &Path, size_hint: Option<u64>) -> Result<()> {
        let response = self.client.get(url.as_str()).send().await?;
        let total_size = response.content_length().or(size_hint);

        let bar = self.create_progress_bar(path, total_size);

        let mut file = File::create(path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| DownloadError::Failed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

            file.write_all(&chunk).await?;
            bar.inc(chunk.len() as u64);
        }

        file.flush().await?;
        bar.finish_with_message("done");

        info!("Downloaded: {}", path.display());
        Ok(())
    }

    fn output_path(&self, model: &str, format: &str) -> PathBuf {
        self.output_dir.join(format!("{model}.{format}"))
    }

    fn create_progress_bar(&self, path: &Path, size: Option<u64>) -> ProgressBar {
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");

        let bar = size.map_or_else(
            || {
                let bar = self.progress.add(ProgressBar::new_spinner());
                bar.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner} {msg} {bytes}")
                        .unwrap(),
                );
                bar
            },
            |s| {
                let bar = self.progress.add(ProgressBar::new(s));
                bar.set_style(
                    ProgressStyle::default_bar()
                        .template("{msg:>20} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .unwrap()
                        .progress_chars("=>-"),
                );
                bar
            },
        );

        bar.set_message(filename.to_string());
        bar
    }
}
