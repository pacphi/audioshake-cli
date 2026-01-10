//! API request types

use serde::Serialize;
use url::Url;

/// Request to create a new processing task
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    /// URL to the audio/video file
    pub url: Url,
    /// Optional webhook URL for notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<Url>,
    /// Processing targets (models and formats)
    pub targets: Vec<TaskTarget>,
}

/// A processing target within a task
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTarget {
    /// Model to use for processing
    pub model: String,
    /// Output formats
    pub formats: Vec<String>,
    /// Model variant (e.g., `high_quality`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Language code for transcription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Include residual stem
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual: Option<bool>,
}

impl CreateTaskRequest {
    /// Create a new task request
    #[must_use]
    pub const fn new(url: Url) -> Self {
        Self {
            url,
            callback_url: None,
            targets: Vec::new(),
        }
    }

    /// Add a callback URL
    #[must_use]
    pub fn with_callback(mut self, url: Url) -> Self {
        self.callback_url = Some(url);
        self
    }

    /// Add a processing target
    #[must_use]
    pub fn with_target(mut self, target: TaskTarget) -> Self {
        self.targets.push(target);
        self
    }
}

impl TaskTarget {
    /// Create a new target
    #[must_use]
    pub fn new(model: impl Into<String>, formats: Vec<String>) -> Self {
        Self {
            model: model.into(),
            formats,
            variant: None,
            language: None,
            residual: None,
        }
    }

    /// Set high-quality variant
    #[must_use]
    pub fn with_hq(mut self) -> Self {
        self.variant = Some("high_quality".to_string());
        self
    }

    /// Set a custom variant
    #[must_use]
    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    /// Set language for transcription
    #[must_use]
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Include residual stem
    #[must_use]
    pub const fn with_residual(mut self) -> Self {
        self.residual = Some(true);
        self
    }
}
