//! Attachment entity
//!
//! Represents a file attached to a message.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A file attachment linked to a message
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub message_id: Uuid,
    /// Original filename
    pub filename: String,
    /// MIME type (e.g., "image/jpeg", "application/pdf")
    pub content_type: String,
    /// File size in bytes
    pub size: i64,
    /// S3 object key (bucket is configured globally)
    pub s3_key: String,
    /// Image width in pixels (None for non-images)
    pub width: Option<i32>,
    /// Image height in pixels (None for non-images)
    pub height: Option<i32>,
    /// S3 key for thumbnail (images only)
    pub thumbnail_s3_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(
        message_id: Uuid,
        filename: impl Into<String>,
        content_type: impl Into<String>,
        size: i64,
        s3_key: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            message_id,
            filename: filename.into(),
            content_type: content_type.into(),
            size,
            s3_key: s3_key.into(),
            width: None,
            height: None,
            thumbnail_s3_key: None,
            created_at: Utc::now(),
        }
    }

    /// Add image metadata
    pub fn with_image_metadata(mut self, width: i32, height: i32, thumbnail_key: Option<String>) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self.thumbnail_s3_key = thumbnail_key;
        self
    }

    /// Check if this is an image attachment
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    /// Check if this is a PDF attachment
    pub fn is_pdf(&self) -> bool {
        self.content_type == "application/pdf"
    }

    /// Get the attachment type
    pub fn attachment_type(&self) -> AttachmentType {
        if self.is_image() {
            AttachmentType::Image
        } else if self.is_pdf() {
            AttachmentType::Pdf
        } else {
            AttachmentType::Other
        }
    }
}

/// Type of attachment for display purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    Image,
    Pdf,
    Other,
}

/// Attachment response with presigned URLs
#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Presigned download URL
    pub url: String,
    /// Presigned thumbnail URL (images only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

impl AttachmentResponse {
    /// Create response from attachment with presigned URLs
    pub fn from_attachment(attachment: &Attachment, url: String, thumbnail_url: Option<String>) -> Self {
        Self {
            id: attachment.id,
            filename: attachment.filename.clone(),
            content_type: attachment.content_type.clone(),
            size: attachment.size,
            width: attachment.width,
            height: attachment.height,
            url,
            thumbnail_url,
        }
    }
}

/// Input for creating an attachment (from message send request)
#[derive(Debug, Deserialize)]
pub struct AttachmentInput {
    pub s3_key: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
}

/// Constants for attachment validation
pub mod limits {
    /// Maximum file size (100 MB)
    pub const MAX_FILE_SIZE: i64 = 100 * 1024 * 1024;

    /// Maximum number of attachments per message
    pub const MAX_ATTACHMENTS_PER_MESSAGE: usize = 10;

    /// Allowed MIME types
    pub const ALLOWED_TYPES: &[&str] = &[
        // Images
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/webp",
        "image/svg+xml",
        "image/bmp",
        "image/tiff",
        // Documents
        "application/pdf",
        "application/msword", // .doc
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document", // .docx
        "application/vnd.ms-excel", // .xls
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", // .xlsx
        "application/vnd.ms-powerpoint", // .ppt
        "application/vnd.openxmlformats-officedocument.presentationml.presentation", // .pptx
        "application/vnd.oasis.opendocument.text", // .odt
        "application/vnd.oasis.opendocument.spreadsheet", // .ods
        "application/vnd.oasis.opendocument.presentation", // .odp
        "application/rtf",
        // Text
        "text/plain",
        "text/csv",
        "text/markdown",
        "text/html",
        "text/xml",
        "application/json",
        // Archives
        "application/zip",
        "application/x-rar-compressed",
        "application/vnd.rar",
        "application/x-7z-compressed",
        "application/gzip",
        "application/x-tar",
        // Audio
        "audio/mpeg",
        "audio/wav",
        "audio/ogg",
        "audio/mp4",
        // Video
        "video/mp4",
        "video/webm",
        "video/ogg",
        "video/quicktime",
    ];

    /// Check if a content type is allowed
    pub fn is_allowed_type(content_type: &str) -> bool {
        // Allow empty content type (browser couldn't detect)
        if content_type.is_empty() {
            return true;
        }
        ALLOWED_TYPES.contains(&content_type)
    }

    /// Check if file size is within limits
    pub fn is_valid_size(size: i64) -> bool {
        size > 0 && size <= MAX_FILE_SIZE
    }
}
