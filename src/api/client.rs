//! HTTP client for the `AudioShake` API

use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use tracing::{debug, instrument};
use url::Url;

use super::request::CreateTaskRequest;
use super::response::{Task, TaskListResponse, TaskStatistics};
use crate::error::{ApiError, Result};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

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

            if task.status.is_terminal() {
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
