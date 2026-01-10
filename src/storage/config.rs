//! Storage configuration types

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

/// Storage provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageProviderType {
    /// No storage configured
    #[default]
    None,
    /// Cloudflare R2
    R2,
    /// AWS S3 (future)
    #[allow(dead_code)]
    S3,
    /// Backblaze B2 (future)
    #[allow(dead_code)]
    B2,
}

impl std::fmt::Display for StorageProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::R2 => write!(f, "r2"),
            Self::S3 => write!(f, "s3"),
            Self::B2 => write!(f, "b2"),
        }
    }
}

impl std::str::FromStr for StorageProviderType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "" => Ok(Self::None),
            "r2" => Ok(Self::R2),
            "s3" => Ok(Self::S3),
            "b2" => Ok(Self::B2),
            _ => Err(format!("Unknown storage provider: {s}")),
        }
    }
}

/// Runtime storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Storage provider type
    pub provider: StorageProviderType,
    /// Bucket name
    pub bucket: String,
    /// Base URL for public access (e.g., `https://bucket.r2.dev`)
    pub public_url_base: Url,
    /// R2-specific configuration
    pub r2: Option<R2Config>,
}

/// Cloudflare R2 specific configuration
#[derive(Debug, Clone)]
pub struct R2Config {
    /// Cloudflare account ID
    pub account_id: String,
    /// Access key ID
    pub access_key_id: String,
    /// Secret access key
    pub secret_access_key: SecretString,
}

impl R2Config {
    /// Get the R2 endpoint URL
    #[must_use]
    pub fn endpoint_url(&self) -> String {
        format!("https://{}.r2.cloudflarestorage.com", self.account_id)
    }
}

/// Storage configuration from file/environment
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StorageConfigFile {
    /// Storage provider type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Bucket name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,

    /// Public URL base
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_url_base: Option<String>,

    /// R2-specific settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r2: Option<R2ConfigFile>,
}

/// R2 configuration from file/environment
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct R2ConfigFile {
    /// Cloudflare account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,

    /// Access key ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,

    /// Secret access key (not serialized for security)
    #[serde(skip_serializing)]
    pub secret_access_key: Option<String>,
}
