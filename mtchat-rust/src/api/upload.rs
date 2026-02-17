use axum::extract::{Path, State};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain;
use crate::middleware::UserId;

use super::{AppState, ApiResponse, ApiError};

// ============ DTOs ============

#[derive(Debug, Deserialize)]
pub struct PresignUploadRequest {
    pub dialog_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
}

#[derive(Debug, Serialize)]
pub struct PresignUploadResponse {
    pub upload_url: String,
    pub s3_key: String,
    pub expires_in: u64,
}

// ============ Handlers ============

pub async fn presign_upload(
    State(state): State<AppState>,
    UserId(_user_id): UserId,
    Json(req): Json<PresignUploadRequest>,
) -> Result<Json<ApiResponse<PresignUploadResponse>>, ApiError> {
    // Check S3 is configured
    if !state.s3.is_configured() {
        return Err(ApiError::Internal("File uploads are not configured".into()));
    }

    // Validate file type
    if !domain::attachment_limits::is_allowed_type(&req.content_type) {
        return Err(ApiError::BadRequest(format!(
            "File type '{}' is not allowed. Allowed types: {:?}",
            req.content_type,
            domain::attachment_limits::ALLOWED_TYPES
        )));
    }

    // Validate file size
    if !domain::attachment_limits::is_valid_size(req.size) {
        return Err(ApiError::BadRequest(format!(
            "File size must be between 1 byte and {} bytes",
            domain::attachment_limits::MAX_FILE_SIZE
        )));
    }

    // Verify dialog exists
    state.dialogs.find_by_id(req.dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Generate S3 key
    // Format: dialogs/{dialog_id}/pending/{uuid}.{ext}
    let ext = req.filename
        .rsplit('.')
        .next()
        .unwrap_or("bin");
    let file_uuid = Uuid::now_v7();
    let s3_key = format!("dialogs/{}/pending/{}.{}", req.dialog_id, file_uuid, ext);

    // Generate presigned URL
    let upload_url = state.s3.generate_upload_url(&s3_key, &req.content_type).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse {
        data: PresignUploadResponse {
            upload_url,
            s3_key,
            expires_in: 300, // 5 minutes
        }
    }))
}

pub async fn get_attachment_url(
    State(state): State<AppState>,
    Path(attachment_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // Check S3 is configured
    if !state.s3.is_configured() {
        return Err(ApiError::Internal("File storage is not configured".into()));
    }

    // Find attachment
    let attachment = state.attachments.find_by_id(attachment_id).await?
        .ok_or_else(|| ApiError::NotFound("Attachment not found".into()))?;

    // Generate presigned download URL
    let url = state.s3.generate_download_url(&attachment.s3_key).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Also get thumbnail URL if available
    let thumbnail_url = if let Some(ref thumb_key) = attachment.thumbnail_s3_key {
        Some(state.s3.generate_download_url(thumb_key).await
            .map_err(|e| ApiError::Internal(e.to_string()))?)
    } else {
        None
    };

    Ok(Json(ApiResponse {
        data: serde_json::json!({
            "url": url,
            "thumbnail_url": thumbnail_url,
            "expires_in": 3600
        })
    }))
}
