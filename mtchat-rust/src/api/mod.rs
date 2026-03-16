//! HTTP API handlers for MTChat.
//!
//! Organized by domain: health, management, dialogs, messages, upload, participants, websocket.

pub mod dialogs;
pub mod health;
pub mod management;
pub mod messages;
pub mod participants;
pub mod upload;
pub mod ws_handler;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use dashmap::DashMap;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::jobs::JobProducer;
use crate::repositories::{
    AccessScopeRepository, AttachmentRepository, DialogRepository, MessageRepository,
    ParticipantRepository,
};
use crate::services::{PresenceService, S3Service};
use crate::webhooks::WebhookSender;
use crate::ws;

// ============ App State ============

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub connections: ws::Connections,
    // Repositories
    pub dialogs: Arc<DialogRepository>,
    pub participants: Arc<ParticipantRepository>,
    pub scopes: Arc<AccessScopeRepository>,
    pub messages: Arc<MessageRepository>,
    pub attachments: Arc<AttachmentRepository>,
    // Services
    pub s3: Arc<S3Service>,
    pub presence: Arc<PresenceService>,
    // Webhooks
    pub webhooks: WebhookSender,
    // Jobs
    pub jobs: JobProducer,
}

impl AppState {
    pub fn new(
        db: PgPool,
        webhooks: WebhookSender,
        s3: S3Service,
        presence: PresenceService,
        jobs: JobProducer,
    ) -> Self {
        Self {
            dialogs: Arc::new(DialogRepository::new(db.clone())),
            participants: Arc::new(ParticipantRepository::new(db.clone())),
            scopes: Arc::new(AccessScopeRepository::new(db.clone())),
            messages: Arc::new(MessageRepository::new(db.clone())),
            attachments: Arc::new(AttachmentRepository::new(db.clone())),
            connections: Arc::new(DashMap::new()),
            db,
            s3: Arc::new(s3),
            presence: Arc::new(presence),
            webhooks,
            jobs,
        }
    }
}

// ============ Common Response/Error Types ============

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// Structured error codes for API responses
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    // Not Found errors
    DialogNotFound,
    MessageNotFound,
    ParticipantNotFound,
    AttachmentNotFound,
    // Bad Request errors
    InvalidInput,
    FileTooLarge,
    UnsupportedFileType,
    TooManyAttachments,
    // Forbidden errors
    NotParticipant,
    NotMessageAuthor,
    ScopeMismatch,
    // Auth errors
    Unauthorized,
    // Generic fallbacks
    NotFound,
    BadRequest,
    Forbidden,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::DialogNotFound => "DIALOG_NOT_FOUND",
            ErrorCode::MessageNotFound => "MESSAGE_NOT_FOUND",
            ErrorCode::ParticipantNotFound => "PARTICIPANT_NOT_FOUND",
            ErrorCode::AttachmentNotFound => "ATTACHMENT_NOT_FOUND",
            ErrorCode::InvalidInput => "INVALID_INPUT",
            ErrorCode::FileTooLarge => "FILE_TOO_LARGE",
            ErrorCode::UnsupportedFileType => "UNSUPPORTED_FILE_TYPE",
            ErrorCode::TooManyAttachments => "TOO_MANY_ATTACHMENTS",
            ErrorCode::NotParticipant => "NOT_PARTICIPANT",
            ErrorCode::NotMessageAuthor => "NOT_MESSAGE_AUTHOR",
            ErrorCode::ScopeMismatch => "SCOPE_MISMATCH",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::BadRequest => "BAD_REQUEST",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::InternalError => "INTERNAL_ERROR",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorCode::DialogNotFound
            | ErrorCode::MessageNotFound
            | ErrorCode::ParticipantNotFound
            | ErrorCode::AttachmentNotFound
            | ErrorCode::NotFound => StatusCode::NOT_FOUND,

            ErrorCode::InvalidInput
            | ErrorCode::FileTooLarge
            | ErrorCode::UnsupportedFileType
            | ErrorCode::TooManyAttachments
            | ErrorCode::BadRequest => StatusCode::BAD_REQUEST,

            ErrorCode::NotParticipant
            | ErrorCode::NotMessageAuthor
            | ErrorCode::ScopeMismatch
            | ErrorCode::Forbidden => StatusCode::FORBIDDEN,

            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,

            ErrorCode::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub enum ApiError {
    /// Error with structured code
    Structured {
        code: ErrorCode,
        message: String,
    },
    /// Legacy errors (for backward compatibility)
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
}

impl ApiError {
    /// Create a structured error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ApiError::Structured {
            code,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            ApiError::Structured { code, message } => (code.status_code(), code.as_str(), message),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                )
            }
        };

        (
            status,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: code.to_string(),
                    message,
                },
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}
