//! S3 Service for file storage operations
//!
//! Provides presigned URLs for secure upload/download of attachments.

use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region},
    presigning::PresigningConfig,
    Client,
};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum S3Error {
    #[error("S3 operation failed: {0}")]
    OperationFailed(String),

    #[error("Presigning failed: {0}")]
    PresigningFailed(String),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// S3 configuration from environment variables
#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    /// Public endpoint for presigned URLs (may differ from internal endpoint)
    pub public_endpoint: Option<String>,
    /// Presigned upload URL expiry (default: 5 minutes)
    pub upload_expiry: Duration,
    /// Presigned download URL expiry (default: 1 hour)
    pub download_expiry: Duration,
}

impl S3Config {
    /// Create config from environment variables
    ///
    /// Required:
    /// - S3_ENDPOINT
    /// - S3_REGION
    /// - S3_ACCESS_KEY_ID
    /// - S3_SECRET_ACCESS_KEY
    /// - S3_BUCKET
    ///
    /// Optional:
    /// - S3_PUBLIC_ENDPOINT (default: S3_ENDPOINT)
    /// - S3_PRESIGN_UPLOAD_EXPIRY (default: 300 seconds)
    /// - S3_PRESIGN_DOWNLOAD_EXPIRY (default: 3600 seconds)
    pub fn from_env() -> Result<Self, S3Error> {
        let endpoint = std::env::var("S3_ENDPOINT")
            .map_err(|_| S3Error::ConfigError("S3_ENDPOINT not set".into()))?;

        let region = std::env::var("S3_REGION")
            .map_err(|_| S3Error::ConfigError("S3_REGION not set".into()))?;

        let access_key_id = std::env::var("S3_ACCESS_KEY_ID")
            .map_err(|_| S3Error::ConfigError("S3_ACCESS_KEY_ID not set".into()))?;

        let secret_access_key = std::env::var("S3_SECRET_ACCESS_KEY")
            .map_err(|_| S3Error::ConfigError("S3_SECRET_ACCESS_KEY not set".into()))?;

        let bucket = std::env::var("S3_BUCKET")
            .map_err(|_| S3Error::ConfigError("S3_BUCKET not set".into()))?;

        let public_endpoint = std::env::var("S3_PUBLIC_ENDPOINT").ok();

        let upload_expiry = std::env::var("S3_PRESIGN_UPLOAD_EXPIRY")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(300));

        let download_expiry = std::env::var("S3_PRESIGN_DOWNLOAD_EXPIRY")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(3600));

        Ok(Self {
            endpoint,
            region,
            access_key_id,
            secret_access_key,
            bucket,
            public_endpoint,
            upload_expiry,
            download_expiry,
        })
    }
}

/// S3 service for file operations
#[derive(Clone)]
pub struct S3Service {
    client: Client,
    bucket: String,
    public_endpoint: String,
    upload_expiry: Duration,
    download_expiry: Duration,
}

impl S3Service {
    /// Create a new S3 service from config
    pub async fn new(config: S3Config) -> Self {
        let credentials = Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "mtchat",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region))
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.bucket,
            public_endpoint: config.public_endpoint.unwrap_or(config.endpoint),
            upload_expiry: config.upload_expiry,
            download_expiry: config.download_expiry,
        }
    }

    /// Create a noop S3 service (for testing or when S3 is disabled)
    pub fn noop() -> Self {
        // This will fail on any operation, but allows the app to start without S3
        Self {
            client: {
                let config = aws_sdk_s3::Config::builder()
                    .behavior_version(BehaviorVersion::latest())
                    .region(Region::new("us-east-1"))
                    .build();
                Client::from_conf(config)
            },
            bucket: String::new(),
            public_endpoint: String::new(),
            upload_expiry: Duration::from_secs(300),
            download_expiry: Duration::from_secs(3600),
        }
    }

    /// Check if S3 is properly configured
    pub fn is_configured(&self) -> bool {
        !self.bucket.is_empty()
    }

    /// Generate a presigned URL for uploading a file
    ///
    /// # Arguments
    /// * `key` - The S3 object key (path)
    /// * `content_type` - The MIME type of the file
    ///
    /// # Returns
    /// The presigned URL that can be used for PUT request
    pub async fn generate_upload_url(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<String, S3Error> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(self.upload_expiry)
            .build()
            .map_err(|e| S3Error::PresigningFailed(e.to_string()))?;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| S3Error::PresigningFailed(e.to_string()))?;

        Ok(self.rewrite_url(presigned.uri()))
    }

    /// Generate a presigned URL for downloading a file
    ///
    /// # Arguments
    /// * `key` - The S3 object key (path)
    ///
    /// # Returns
    /// The presigned URL that can be used for GET request
    pub async fn generate_download_url(&self, key: &str) -> Result<String, S3Error> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(self.download_expiry)
            .build()
            .map_err(|e| S3Error::PresigningFailed(e.to_string()))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| S3Error::PresigningFailed(e.to_string()))?;

        Ok(self.rewrite_url(presigned.uri()))
    }

    /// Check if an object exists in S3
    ///
    /// # Arguments
    /// * `key` - The S3 object key (path)
    ///
    /// # Returns
    /// true if the object exists, false otherwise
    pub async fn object_exists(&self, key: &str) -> Result<bool, S3Error> {
        use aws_sdk_s3::operation::head_object::HeadObjectError;

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let service_err = e.into_service_error();
                match service_err {
                    HeadObjectError::NotFound(_) => Ok(false),
                    _ => Err(S3Error::OperationFailed(service_err.to_string())),
                }
            }
        }
    }

    /// Get object metadata
    ///
    /// # Arguments
    /// * `key` - The S3 object key (path)
    ///
    /// # Returns
    /// Content type and size of the object
    pub async fn get_object_info(&self, key: &str) -> Result<(String, i64), S3Error> {
        use aws_sdk_s3::operation::head_object::HeadObjectError;

        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let service_err = e.into_service_error();
                match service_err {
                    HeadObjectError::NotFound(_) => S3Error::NotFound(key.to_string()),
                    _ => S3Error::OperationFailed(service_err.to_string()),
                }
            })?;

        let content_type = response
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let size = response.content_length().unwrap_or(0);

        Ok((content_type, size))
    }

    /// Delete an object from S3
    ///
    /// # Arguments
    /// * `key` - The S3 object key (path)
    pub async fn delete_object(&self, key: &str) -> Result<(), S3Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::OperationFailed(e.to_string()))?;

        Ok(())
    }

    /// Get raw object data (for thumbnail generation)
    ///
    /// # Arguments
    /// * `key` - The S3 object key (path)
    ///
    /// # Returns
    /// The raw bytes of the object
    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        use aws_sdk_s3::operation::get_object::GetObjectError;

        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let service_err = e.into_service_error();
                match service_err {
                    GetObjectError::NoSuchKey(_) => S3Error::NotFound(key.to_string()),
                    _ => S3Error::OperationFailed(service_err.to_string()),
                }
            })?;

        let aggregated = response
            .body
            .collect()
            .await
            .map_err(|e| S3Error::OperationFailed(format!("Failed to read body: {}", e)))?;

        let bytes = aggregated.into_bytes().to_vec();

        Ok(bytes)
    }

    /// Upload raw object data (for thumbnail upload)
    ///
    /// # Arguments
    /// * `key` - The S3 object key (path)
    /// * `data` - The raw bytes to upload
    /// * `content_type` - The MIME type of the data
    pub async fn put_object(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), S3Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| S3Error::OperationFailed(e.to_string()))?;

        Ok(())
    }

    /// Get the bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Rewrite internal S3 URL to public URL
    fn rewrite_url(&self, url: &str) -> String {
        // Find the bucket part and replace the base URL
        if let Some(idx) = url.find(&self.bucket) {
            // URL format: http://internal:9000/bucket/key?signature
            // We need to replace the base URL part
            let path_and_query = &url[idx - 1..]; // Include the leading /
            format!("{}{}", self.public_endpoint, path_and_query)
        } else {
            url.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_url() {
        let service = S3Service {
            client: {
                let config = aws_sdk_s3::Config::builder()
                    .behavior_version(BehaviorVersion::latest())
                    .region(Region::new("us-east-1"))
                    .build();
                Client::from_conf(config)
            },
            bucket: "mtchat-attachments".to_string(),
            public_endpoint: "http://localhost:9000".to_string(),
            upload_expiry: Duration::from_secs(300),
            download_expiry: Duration::from_secs(3600),
        };

        let internal_url = "http://minio:9000/mtchat-attachments/test/file.jpg?X-Amz-Signature=abc";
        let public_url = service.rewrite_url(internal_url);

        assert_eq!(
            public_url,
            "http://localhost:9000/mtchat-attachments/test/file.jpg?X-Amz-Signature=abc"
        );
    }
}
