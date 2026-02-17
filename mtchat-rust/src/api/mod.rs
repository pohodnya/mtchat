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
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

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
    pub connections: Arc<RwLock<HashMap<Uuid, ws::ConnectionTx>>>,
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
            connections: Arc::new(RwLock::new(HashMap::new())),
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

pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Forbidden(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
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
