//! API response types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

/// A processing task
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Current status
    pub status: TaskStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Processing targets
    pub targets: Vec<TaskTargetResult>,
    /// Error information if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TaskError>,
    /// Cost information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<TaskCost>,
}

/// Task status
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is queued
    Pending,
    /// Task is being processed
    Processing,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
}

impl TaskStatus {
    /// Check if status is terminal (completed or failed)
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Get display string
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Result for a specific target within a task
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTargetResult {
    /// Model name
    pub model: String,
    /// Target status
    #[serde(default)]
    pub status: Option<TaskStatus>,
    /// Output links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<TaskOutput>,
    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Output files for a target
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskOutput {
    /// Download links
    pub links: Vec<OutputLink>,
}

/// A downloadable output file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputLink {
    /// File format
    pub format: String,
    /// Download URL
    pub link: Url,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Task error information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// Task cost information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCost {
    /// Credits used
    pub credits: f64,
    /// Duration in minutes
    pub duration_minutes: f64,
}

/// Response for listing tasks
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskListResponse {
    /// List of tasks
    pub tasks: Vec<Task>,
    /// Total count
    pub total: u64,
    /// Current offset
    #[serde(default)]
    pub offset: u64,
    /// Current limit
    #[serde(default)]
    pub limit: u64,
}

/// Task statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatistics {
    /// Total tasks
    pub total_tasks: u64,
    /// Completed tasks
    pub completed_tasks: u64,
    /// Failed tasks
    pub failed_tasks: u64,
    /// Credits used
    pub credits_used: f64,
    /// Total duration in minutes
    pub total_duration_minutes: f64,
}

/// Response from file upload (legacy API)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    /// Uploaded file name
    pub name: String,
    /// Asset ID for job requests
    pub id: String,
    /// MIME type
    pub file_type: String,
    /// File extension
    pub format: String,
    /// Download URL for the uploaded file
    pub link: Url,
}
