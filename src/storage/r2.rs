//! Cloudflare R2 storage provider implementation

use std::path::Path;

use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use secrecy::ExposeSecret;
use tracing::{debug, instrument};
use url::Url;
use uuid::Uuid;

use super::config::StorageConfig;
use super::StorageProvider;
use crate::error::{Result, StorageError};

/// Cloudflare R2 storage provider
pub struct R2Storage {
    client: Client,
    bucket: String,
    public_url_base: Url,
}

impl R2Storage {
    /// Create a new R2 storage provider
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid
    pub fn new(config: &StorageConfig) -> Result<Self> {
        let r2_config = config.r2.as_ref().ok_or(StorageError::NotConfigured)?;

        let credentials = Credentials::new(
            &r2_config.access_key_id,
            r2_config.secret_access_key.expose_secret(),
            None, // Session token not used with R2
            None, // Expiry
            "r2-storage",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("auto")) // R2 uses "auto" region
            .endpoint_url(r2_config.endpoint_url())
            .credentials_provider(credentials)
            .force_path_style(true) // Required for R2
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
            public_url_base: config.public_url_base.clone(),
        })
    }

    /// Generate a unique object key for a file
    fn generate_key(path: &Path) -> String {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let random = Uuid::new_v4().simple().to_string()[..8].to_string();

        format!("{timestamp}-{random}-{filename}")
    }

    /// Determine MIME type from file extension
    fn mime_type(path: &Path) -> &'static str {
        match path.extension().and_then(|e| e.to_str()) {
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
        }
    }

    /// Build the public URL for an object
    fn public_url(&self, key: &str) -> Result<Url> {
        self.public_url_base
            .join(key)
            .map_err(|e| StorageError::InvalidUrl(e.to_string()).into())
    }
}

#[async_trait]
impl StorageProvider for R2Storage {
    #[instrument(skip(self), fields(bucket = %self.bucket))]
    async fn upload(&self, path: &Path) -> Result<Url> {
        let key = Self::generate_key(path);
        let content_type = Self::mime_type(path);

        debug!(
            "Uploading {} to R2 bucket {} with key {}",
            path.display(),
            self.bucket,
            key
        );

        let body = ByteStream::from_path(path)
            .await
            .map_err(|e| StorageError::UploadFailed(format!("Failed to read file: {e}")))?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(format!("R2 upload failed: {e}")))?;

        debug!("Upload complete: {}", key);

        self.public_url(&key)
    }

    #[instrument(skip(self), fields(bucket = %self.bucket))]
    async fn delete(&self, key: &str) -> Result<()> {
        debug!("Deleting object {} from R2 bucket {}", key, self.bucket);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(format!("R2 delete failed: {e}")))?;

        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "Cloudflare R2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_key() {
        let path = PathBuf::from("/path/to/song.mp3");
        let key = R2Storage::generate_key(&path);

        assert!(key.ends_with("-song.mp3"));
        assert!(key.len() > 20); // timestamp + random + filename
    }

    #[test]
    fn test_mime_type() {
        assert_eq!(R2Storage::mime_type(Path::new("song.mp3")), "audio/mpeg");
        assert_eq!(R2Storage::mime_type(Path::new("video.mp4")), "video/mp4");
        assert_eq!(
            R2Storage::mime_type(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
    }
}
