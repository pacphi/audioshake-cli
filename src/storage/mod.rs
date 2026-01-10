//! Cloud storage module for uploading local files
//!
//! This module provides S3-compatible storage integration for uploading
//! local files before processing with the `AudioShake` Tasks API.

pub mod config;
pub mod r2;

use std::path::Path;

use async_trait::async_trait;
use url::Url;

use crate::error::Result;

/// Trait for cloud storage providers
///
/// Implementations provide file upload capabilities to S3-compatible
/// storage services like Cloudflare R2, AWS S3, or Backblaze B2.
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Upload a file and return its public URL
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the local file to upload
    ///
    /// # Returns
    ///
    /// The public URL where the file can be accessed
    ///
    /// # Errors
    ///
    /// Returns an error if the upload fails
    async fn upload(&self, path: &Path) -> Result<Url>;

    /// Delete an object by key
    ///
    /// # Arguments
    ///
    /// * `key` - The object key to delete
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails
    #[allow(dead_code)] // Reserved for future cleanup functionality
    async fn delete(&self, key: &str) -> Result<()>;

    /// Get the provider name for display purposes
    #[allow(dead_code)] // Reserved for future use
    fn provider_name(&self) -> &'static str;
}
