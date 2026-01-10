//! HTTP client for the `AudioShake` API

use std::path::Path;
use std::time::Duration;

use reqwest::multipart::{Form, Part};
use reqwest::{Client, Response, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{debug, instrument};
use url::Url;

use super::request::CreateTaskRequest;
use super::response::{Task, TaskListResponse, TaskStatistics, UploadResponse};
use crate::error::{ApiError, Result};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Legacy upload endpoint - deprecated in favor of cloud storage (R2/S3)
/// See ADR-002 for details on the new cloud storage upload strategy.
#[deprecated(
    since = "0.2.0",
    note = "Use cloud storage (R2) upload instead. Configure with `audioshake config storage`. See ADR-002."
)]
#[allow(dead_code)]
const LEGACY_UPLOAD_URL: &str = "https://groovy.audioshake.ai/upload";

/// `AudioShake` API client
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: Url,
    api_key: SecretString,
}

impl ApiClient {
    /// Create a new API client
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(base_url: Url, api_key: SecretString) -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(ApiError::from)?;

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Create a new processing task
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    #[instrument(skip(self, request))]
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task> {
        let url = self.base_url.join("/tasks")?;
        debug!("Creating task with {} targets", request.targets.len());

        let response = self
            .client
            .post(url)
            .header("x-api-key", self.api_key.expose_secret())
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a task by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    #[instrument(skip(self))]
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        let url = self.base_url.join(&format!("/tasks/{task_id}"))?;

        let response = self
            .client
            .get(url)
            .header("x-api-key", self.api_key.expose_secret())
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// List tasks with optional filtering
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
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

        let response = self
            .client
            .get(url)
            .header("x-api-key", self.api_key.expose_secret())
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get task statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    #[instrument(skip(self))]
    pub async fn get_statistics(&self, name: &str) -> Result<TaskStatistics> {
        let mut url = self.base_url.join("/tasks/statistics")?;
        url.query_pairs_mut().append_pair("name", name);

        let response = self
            .client
            .get(url)
            .header("x-api-key", self.api_key.expose_secret())
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Wait for a task to complete
    ///
    /// # Errors
    ///
    /// Returns an error if the task times out or fails.
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

            if task.effective_status().is_terminal() {
                return Ok(task);
            }

            if start.elapsed() > timeout {
                return Err(ApiError::Timeout {
                    seconds: timeout.as_secs(),
                }
                .into());
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Validate the API key by making a test request
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is invalid.
    pub async fn validate(&self) -> Result<()> {
        // Try to list tasks with a small limit
        let _ = self.list_tasks(None, 1, 0).await?;
        Ok(())
    }

    /// Upload a local file to `AudioShake`'s servers (DEPRECATED)
    ///
    /// Uses the legacy API endpoint to upload files directly.
    /// Returns the hosted URL that can be used with the Tasks API.
    ///
    /// # Deprecation
    ///
    /// This method uses the legacy upload API which requires a separate authentication
    /// token and may be unreliable. Use cloud storage (Cloudflare R2) instead.
    /// See ADR-002 for the recommended approach.
    ///
    /// # Errors
    ///
    /// Returns an error if the upload fails.
    #[deprecated(
        since = "0.2.0",
        note = "Use cloud storage (R2) upload instead. Configure with `audioshake config storage`. See ADR-002."
    )]
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn upload_file(&self, path: &Path) -> Result<UploadResponse> {
        debug!("Uploading file: {}", path.display());

        // Read file contents
        let mut file = File::open(path).await.map_err(|e| {
            ApiError::Request(format!("Failed to open file {}: {e}", path.display()))
        })?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await.map_err(|e| {
            ApiError::Request(format!("Failed to read file {}: {e}", path.display()))
        })?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        // Determine MIME type from extension
        let mime_type = match path.extension().and_then(|e| e.to_str()) {
            Some("mp3") => "audio/mpeg",
            Some("wav") => "audio/wav",
            Some("flac") => "audio/flac",
            Some("aac") => "audio/aac",
            Some("m4a") => "audio/mp4",
            Some("ogg") => "audio/ogg",
            Some("mp4") => "video/mp4",
            Some("mov") => "video/quicktime",
            Some("avi") => "video/x-msvideo",
            Some("mkv") => "video/x-matroska",
            Some("webm") => "video/webm",
            _ => "application/octet-stream",
        };

        // Create multipart form
        let part = Part::bytes(contents)
            .file_name(file_name)
            .mime_str(mime_type)
            .map_err(|e| ApiError::Request(format!("Invalid MIME type: {e}")))?;

        let form = Form::new().part("file", part);

        // Use a longer timeout for uploads
        let upload_client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(600)) // 10 minutes for large files
            .build()
            .map_err(ApiError::from)?;

        #[allow(deprecated)]
        let response = upload_client
            .post(LEGACY_UPLOAD_URL)
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.expose_secret()),
            )
            .multipart(form)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let body = response.text().await.map_err(ApiError::from)?;
            Ok(serde_json::from_str(&body).map_err(|e| {
                debug!("Failed to parse response: {}", body);
                ApiError::Request(format!("Failed to parse API response: {e}. Body: {body}"))
            })?)
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
                }
                .into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    #[test]
    fn test_client_creation() {
        let result = ApiClient::new(
            Url::parse("https://api.audioshake.ai").unwrap(),
            SecretString::from("test_key".to_string()),
        );
        assert!(result.is_ok());
    }
}
