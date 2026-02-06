//! MTChat API - Embeddable Chat Service Backend
//!
//! Object-bound chat service with direct and potential participants.

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    middleware as axum_middleware,
    response::{IntoResponse, Json},
    routing::{get, post, delete, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, net::SocketAddr, sync::Arc, collections::HashMap};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod ws;
mod domain;
mod repositories;
mod middleware;
mod webhooks;
mod services;

use domain::{Dialog, DialogParticipant, DialogAccessScope, Message, JoinedAs, ParticipantProfile, system_messages};
use repositories::{DialogRepository, ParticipantRepository, AccessScopeRepository, MessageRepository, AttachmentRepository};
use middleware::{ScopeConfig, OptionalScopeConfig, UserId};
use webhooks::{WebhookSender, WebhookConfig, WebhookEvent};
use services::{S3Service, S3Config, PresenceService};
use fred::prelude::*;
use fred::types::Builder;

// ============ App State ============

#[derive(Clone)]
struct AppState {
    db: PgPool,
    connections: Arc<RwLock<HashMap<Uuid, ws::ConnectionTx>>>,
    // Repositories
    dialogs: Arc<DialogRepository>,
    participants: Arc<ParticipantRepository>,
    scopes: Arc<AccessScopeRepository>,
    messages: Arc<MessageRepository>,
    attachments: Arc<AttachmentRepository>,
    // Services
    s3: Arc<S3Service>,
    presence: Arc<PresenceService>,
    // Webhooks
    webhooks: WebhookSender,
}

impl AppState {
    fn new(db: PgPool, webhooks: WebhookSender, s3: S3Service, presence: PresenceService) -> Self {
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
        }
    }
}

// ============ Request/Response DTOs ============

/// Participant input with profile info for dialog creation
#[derive(Debug, Deserialize)]
struct ParticipantInput {
    user_id: Uuid,
    display_name: String,
    company: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateDialogRequest {
    object_id: Uuid,
    object_type: String,
    title: Option<String>,
    participants: Vec<ParticipantInput>,
    #[serde(default)]
    access_scopes: Vec<AccessScopeInput>,
}

#[derive(Debug, Deserialize)]
struct AccessScopeInput {
    tenant_uid: Uuid,
    #[serde(default)]
    scope_level1: Vec<String>,
    #[serde(default)]
    scope_level2: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AddParticipantRequest {
    user_id: Uuid,
    display_name: String,
    company: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

/// Request body for joining a dialog
#[derive(Debug, Deserialize)]
struct JoinDialogRequest {
    display_name: String,
    company: String,
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    content: String,
    reply_to: Option<Uuid>,
    #[serde(default)]
    attachments: Vec<domain::AttachmentInput>,
}

#[derive(Debug, Deserialize)]
struct EditMessageRequest {
    content: String,
}

// ============ Upload DTOs ============

#[derive(Debug, Deserialize)]
struct PresignUploadRequest {
    dialog_id: Uuid,
    filename: String,
    content_type: String,
    size: i64,
}

#[derive(Debug, Serialize)]
struct PresignUploadResponse {
    upload_url: String,
    s3_key: String,
    expires_in: u64,
}

/// Message response with attachments
#[derive(Debug, Serialize)]
struct MessageWithAttachments {
    #[serde(flatten)]
    message: Message,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<domain::AttachmentResponse>,
}

#[derive(Debug, Deserialize)]
struct DialogsQuery {
    #[serde(default)]
    r#type: Option<String>, // "participating" or "available"
    #[serde(default)]
    search: Option<String>, // Search by dialog title
    #[serde(default)]
    archived: Option<bool>, // Filter by archived status
}

#[derive(Debug, Deserialize)]
struct MarkAsReadRequest {
    last_read_message_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    before: Option<Uuid>,
}

fn default_limit() -> i64 { 50 }

#[derive(Debug, Serialize)]
struct MessagesResponse {
    messages: Vec<MessageWithAttachments>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_unread_message_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    data: T,
}

#[derive(Debug, Serialize)]
struct DialogResponse {
    #[serde(flatten)]
    dialog: Dialog,
    participants_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    i_am_participant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    can_join: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unread_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_archived: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

// ============ Error Handling ============

enum ApiError {
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
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string())
            }
        };

        (status, Json(ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message,
            }
        })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}

// ============ Handlers: Health ============

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn health_ready(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ready"}))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"status": "not_ready"}))),
    }
}

// ============ Handlers: Management API ============

async fn management_create_dialog(
    State(state): State<AppState>,
    Json(req): Json<CreateDialogRequest>,
) -> Result<Json<ApiResponse<Dialog>>, ApiError> {
    // Create dialog
    let created_by = req.participants.first().map(|p| p.user_id);
    let dialog = Dialog::new(
        req.object_id,
        req.object_type,
        req.title,
        created_by,
    );
    let dialog = state.dialogs.create(&dialog).await?;

    // Add participants with their profiles
    for (i, participant) in req.participants.iter().enumerate() {
        let joined_as = if i == 0 { JoinedAs::Creator } else { JoinedAs::Participant };
        let profile = ParticipantProfile {
            display_name: participant.display_name.clone(),
            company: participant.company.clone(),
            email: participant.email.clone(),
            phone: participant.phone.clone(),
        };
        state.participants.add_with_profile(dialog.id, participant.user_id, joined_as, &profile).await?;
    }

    // Add access scopes
    for scope_input in req.access_scopes {
        let scope = DialogAccessScope::new(
            dialog.id,
            scope_input.tenant_uid,
            scope_input.scope_level1,
            scope_input.scope_level2,
        );
        state.scopes.create(&scope).await?;
    }

    // Create system message "chat created"
    if !req.participants.is_empty() {
        let participant_infos: Vec<system_messages::ParticipantInfo> = req.participants
            .iter()
            .map(|p| system_messages::ParticipantInfo {
                name: p.display_name.clone(),
                company: p.company.clone(),
            })
            .collect();
        let system_msg = Message::system(
            dialog.id,
            system_messages::chat_created_content(participant_infos),
        );
        // System messages don't increment unread_count
        state.messages.create(&system_msg).await?;
    }

    Ok(Json(ApiResponse { data: dialog }))
}

async fn management_add_participant(
    State(state): State<AppState>,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<AddParticipantRequest>,
) -> Result<StatusCode, ApiError> {
    // Check dialog exists
    state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    let profile = ParticipantProfile {
        display_name: req.display_name,
        company: req.company,
        email: req.email,
        phone: req.phone,
    };
    state.participants.add_with_profile_if_not_exists(dialog_id, req.user_id, JoinedAs::Participant, &profile).await?;

    Ok(StatusCode::CREATED)
}

async fn management_remove_participant(
    State(state): State<AppState>,
    Path((dialog_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.participants.remove(dialog_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn management_delete_dialog(
    State(state): State<AppState>,
    Path(dialog_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.dialogs.delete(dialog_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct UpdateAccessScopesRequest {
    access_scopes: Vec<AccessScopeInput>,
}

async fn management_update_access_scopes(
    State(state): State<AppState>,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<UpdateAccessScopesRequest>,
) -> Result<Json<ApiResponse<Vec<DialogAccessScope>>>, ApiError> {
    // Check dialog exists
    state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Delete existing scopes and create new ones
    let new_scopes: Vec<DialogAccessScope> = req.access_scopes
        .into_iter()
        .map(|s| DialogAccessScope::new(dialog_id, s.tenant_uid, s.scope_level1, s.scope_level2))
        .collect();

    let created = state.scopes.replace_for_dialog(dialog_id, new_scopes).await?;

    Ok(Json(ApiResponse { data: created }))
}

async fn management_get_dialog(
    State(state): State<AppState>,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ManagementDialogResponse>>, ApiError> {
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    let participants = state.participants.list_by_dialog(dialog_id).await?;
    let access_scopes = state.scopes.find_by_dialog(dialog_id).await?;

    Ok(Json(ApiResponse {
        data: ManagementDialogResponse {
            dialog,
            participants,
            access_scopes,
        }
    }))
}

#[derive(Debug, Serialize)]
struct ManagementDialogResponse {
    #[serde(flatten)]
    dialog: Dialog,
    participants: Vec<DialogParticipant>,
    access_scopes: Vec<DialogAccessScope>,
}

// ============ Handlers: Chat API ============

async fn list_dialogs(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    OptionalScopeConfig(scope_config): OptionalScopeConfig,
    Query(params): Query<DialogsQuery>,
) -> Result<Json<ApiResponse<Vec<DialogResponse>>>, ApiError> {
    let dialog_type = params.r#type.as_deref().unwrap_or("participating");

    let search = params.search.as_deref();
    let archived = params.archived;

    let dialogs = match dialog_type {
        "participating" => {
            state.dialogs.find_participating(user_id, search, archived).await?
        }
        "available" => {
            // Available dialogs are never archived (user is not a participant yet)
            if let Some(scope) = &scope_config {
                state.dialogs.find_available(
                    user_id,
                    scope.tenant_uid,
                    &scope.scope_level1,
                    &scope.scope_level2,
                    search,
                ).await?
            } else {
                return Err(ApiError::BadRequest("X-Scope-Config header required for available dialogs".into()));
            }
        }
        _ => {
            return Err(ApiError::BadRequest("Invalid type parameter".into()));
        }
    };

    // Build responses with counts
    let mut responses = Vec::new();
    for dialog in dialogs {
        let participants_count = state.dialogs.count_participants(dialog.id).await?;

        // Get participant info for participating dialogs
        let (unread_count, is_archived) = if dialog_type == "participating" {
            let participant = state.participants.find(dialog.id, user_id).await?;
            (
                participant.as_ref().map(|p| p.unread_count as i64),
                participant.as_ref().map(|p| p.is_archived),
            )
        } else {
            (None, None)
        };

        responses.push(DialogResponse {
            dialog,
            participants_count,
            i_am_participant: Some(dialog_type == "participating"),
            can_join: Some(dialog_type == "available"),
            unread_count,
            is_archived,
        });
    }

    Ok(Json(ApiResponse { data: responses }))
}

async fn get_dialog_by_object(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    OptionalScopeConfig(scope_config): OptionalScopeConfig,
    Path((object_type, object_id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<Option<DialogResponse>>>, ApiError> {
    let dialog = state.dialogs.find_by_object(&object_type, object_id).await?;

    if let Some(dialog) = dialog {
        let i_am_participant = state.participants.exists(dialog.id, user_id).await?;

        let can_join = if !i_am_participant {
            if let Some(scope) = &scope_config {
                state.scopes.check_access(
                    dialog.id,
                    scope.tenant_uid,
                    &scope.scope_level1,
                    &scope.scope_level2,
                ).await?
            } else {
                false
            }
        } else {
            false
        };

        if !i_am_participant && !can_join {
            return Err(ApiError::Forbidden("No access to this dialog".into()));
        }

        let participants_count = state.dialogs.count_participants(dialog.id).await?;

        Ok(Json(ApiResponse {
            data: Some(DialogResponse {
                dialog,
                participants_count,
                i_am_participant: Some(i_am_participant),
                can_join: Some(can_join),
                unread_count: None,
                is_archived: None,
            })
        }))
    } else {
        Ok(Json(ApiResponse { data: None }))
    }
}

async fn join_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    scope_config: ScopeConfig,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<JoinDialogRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check dialog exists
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check if already participant
    if state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::BadRequest("Already a participant".into()));
    }

    // Check scope access
    let has_access = state.scopes.check_access(
        dialog_id,
        scope_config.tenant_uid,
        &scope_config.scope_level1,
        &scope_config.scope_level2,
    ).await?;

    if !has_access {
        return Err(ApiError::Forbidden("No access to join this dialog".into()));
    }

    // Join with profile
    let profile = ParticipantProfile {
        display_name: req.display_name.clone(),
        company: Some(req.company.clone()),
        email: req.email.clone(),
        phone: req.phone.clone(),
    };
    let participant = state.participants.add_with_profile(dialog_id, user_id, JoinedAs::Joined, &profile).await?;

    // Set unread count to total messages in dialog
    state.participants.set_unread_count_from_messages(dialog_id, user_id).await?;

    // Create system message "participant joined"
    let system_msg = Message::system(
        dialog_id,
        system_messages::participant_joined_content(&req.display_name, Some(&req.company)),
    );
    let system_msg = state.messages.create(&system_msg).await?;

    // Broadcast system message via WebSocket
    ws::broadcast_message(&state.connections, dialog_id, &system_msg).await;

    // Send webhook
    state.webhooks.send(WebhookEvent::participant_joined(&dialog, &participant)).await;

    Ok(Json(serde_json::json!({
        "status": "joined",
        "dialog_id": dialog.id
    })))
}

async fn leave_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get dialog for webhook
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Get participant before removing to get display_name
    let participant = state.participants.find(dialog_id, user_id).await?;
    let display_name = participant
        .as_ref()
        .and_then(|p| p.display_name.clone())
        .unwrap_or_else(|| "Участник".to_string());

    state.participants.remove(dialog_id, user_id).await?;

    // Create system message "participant left"
    let system_msg = Message::system(
        dialog_id,
        system_messages::participant_left_content(&display_name),
    );
    let system_msg = state.messages.create(&system_msg).await?;

    // Broadcast system message via WebSocket
    ws::broadcast_message(&state.connections, dialog_id, &system_msg).await;

    // Send webhook
    state.webhooks.send(WebhookEvent::participant_left(&dialog, user_id)).await;

    Ok(Json(serde_json::json!({
        "status": "left"
    })))
}

async fn archive_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state.participants.set_archived(dialog_id, user_id, true).await?;

    Ok(Json(serde_json::json!({ "status": "archived" })))
}

async fn unarchive_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state.participants.set_archived(dialog_id, user_id, false).await?;

    Ok(Json(serde_json::json!({ "status": "unarchived" })))
}

async fn get_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    scope_config: ScopeConfig,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Dialog>>, ApiError> {
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check access: must be participant OR have scope access (potential participant)
    let is_participant = state.participants.exists(dialog_id, user_id).await?;
    let has_scope_access = state.scopes.check_access(
        dialog_id,
        scope_config.tenant_uid,
        &scope_config.scope_level1,
        &scope_config.scope_level2,
    ).await?;

    if !is_participant && !has_scope_access {
        return Err(ApiError::Forbidden("No access to this dialog".into()));
    }

    Ok(Json(ApiResponse { data: dialog }))
}

// ============ Handlers: Messages ============

async fn list_messages(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<MessagesResponse>>, ApiError> {
    // Check user is participant (potential participants cannot read messages)
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant. Join the dialog first.".into()));
    }

    let messages = state.messages.list_by_dialog(dialog_id, pagination.limit, pagination.before).await?;

    // Get participant to find first unread message
    let participant = state.participants.find(dialog_id, user_id).await?;
    let first_unread_message_id = if let Some(ref p) = participant {
        if let Some(last_read_id) = p.last_read_message_id {
            // Find the last read message to get its sent_at
            let last_read_msg = messages.iter().find(|m| m.id == last_read_id);
            let last_read_sent_at = if let Some(msg) = last_read_msg {
                Some(msg.sent_at)
            } else {
                // Last read message not in current page - fetch from DB
                state.messages.find_by_id(last_read_id).await?
                    .map(|m| m.sent_at)
            };

            if let Some(sent_at) = last_read_sent_at {
                // Find first message sent AFTER the last read message
                messages.iter()
                    .find(|m| m.sent_at > sent_at)
                    .map(|m| m.id)
            } else {
                // Last read message doesn't exist - treat as never read
                messages.first().map(|m| m.id)
            }
        } else if !messages.is_empty() {
            // Never read - first message is unread
            Some(messages[0].id)
        } else {
            None
        }
    } else {
        None
    };

    // Batch fetch attachments for all messages
    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let all_attachments = state.attachments.list_by_messages(&message_ids).await?;

    // Group attachments by message_id
    let mut attachments_map: HashMap<Uuid, Vec<domain::Attachment>> = HashMap::new();
    for att in all_attachments {
        attachments_map.entry(att.message_id).or_default().push(att);
    }

    // Build response with attachments and presigned URLs
    let mut messages_with_attachments = Vec::with_capacity(messages.len());
    for message in messages {
        let attachments = attachments_map.remove(&message.id).unwrap_or_default();

        let mut attachment_responses = Vec::new();
        for att in &attachments {
            let url = if state.s3.is_configured() {
                state.s3.generate_download_url(&att.s3_key).await
                    .unwrap_or_else(|_| String::new())
            } else {
                String::new()
            };

            let thumbnail_url = if let Some(ref thumb_key) = att.thumbnail_s3_key {
                if state.s3.is_configured() {
                    state.s3.generate_download_url(thumb_key).await.ok()
                } else {
                    None
                }
            } else {
                None
            };

            attachment_responses.push(domain::AttachmentResponse::from_attachment(att, url, thumbnail_url));
        }

        messages_with_attachments.push(MessageWithAttachments {
            message,
            attachments: attachment_responses,
        });
    }

    Ok(Json(ApiResponse {
        data: MessagesResponse {
            messages: messages_with_attachments,
            first_unread_message_id,
        }
    }))
}

async fn send_message(
    State(state): State<AppState>,
    UserId(sender_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<ApiResponse<MessageWithAttachments>>, ApiError> {
    // Verify dialog exists
    let dialog = state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check user is participant (potential participants cannot send messages)
    if !state.participants.exists(dialog_id, sender_id).await? {
        return Err(ApiError::Forbidden("Not a participant. Join the dialog first.".into()));
    }

    // Validate attachment count
    if req.attachments.len() > domain::attachment_limits::MAX_ATTACHMENTS_PER_MESSAGE {
        return Err(ApiError::BadRequest(format!(
            "Maximum {} attachments per message",
            domain::attachment_limits::MAX_ATTACHMENTS_PER_MESSAGE
        )));
    }

    // Validate and verify attachments exist in S3
    for att_input in &req.attachments {
        // Validate type
        if !domain::attachment_limits::is_allowed_type(&att_input.content_type) {
            return Err(ApiError::BadRequest(format!(
                "File type '{}' is not allowed",
                att_input.content_type
            )));
        }

        // Validate size
        if !domain::attachment_limits::is_valid_size(att_input.size) {
            return Err(ApiError::BadRequest("Invalid file size".into()));
        }

        // Verify file exists in S3 (only if S3 is configured)
        if state.s3.is_configured() {
            let exists = state.s3.object_exists(&att_input.s3_key).await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            if !exists {
                return Err(ApiError::BadRequest(format!(
                    "File not found in storage: {}",
                    att_input.s3_key
                )));
            }
        }
    }

    // Sanitize message content (removes XSS, preserves formatting)
    let sanitized_content = domain::sanitize_html(&req.content);

    // Create message
    let mut message = Message::new(dialog_id, sender_id, sanitized_content);
    if let Some(reply_to) = req.reply_to {
        message = message.with_reply(reply_to);
    }
    let message = state.messages.create(&message).await?;

    // Create attachments
    let mut attachment_responses = Vec::new();
    if !req.attachments.is_empty() {
        let attachments: Vec<domain::Attachment> = req.attachments
            .iter()
            .map(|input| domain::Attachment::new(
                message.id,
                &input.filename,
                &input.content_type,
                input.size,
                &input.s3_key,
            ))
            .collect();

        let created = state.attachments.create_many(&attachments).await?;

        // Generate presigned URLs for response
        for att in &created {
            let url = if state.s3.is_configured() {
                state.s3.generate_download_url(&att.s3_key).await
                    .unwrap_or_else(|_| String::new())
            } else {
                String::new()
            };

            let thumbnail_url = if let Some(ref thumb_key) = att.thumbnail_s3_key {
                if state.s3.is_configured() {
                    state.s3.generate_download_url(thumb_key).await.ok()
                } else {
                    None
                }
            } else {
                None
            };

            attachment_responses.push(domain::AttachmentResponse::from_attachment(att, url, thumbnail_url));
        }
    }

    // Increment unread count for all participants except the sender
    state.participants.increment_unread(dialog_id, sender_id).await?;

    // Mark sender's own message as read (so divider doesn't appear before own messages)
    state.participants.mark_as_read(dialog_id, sender_id, message.id).await?;

    // Broadcast to WebSocket connections
    ws::broadcast_message(&state.connections, dialog_id, &message).await;

    // Send webhook
    state.webhooks.send(WebhookEvent::message_new(&dialog, &message)).await;

    Ok(Json(ApiResponse {
        data: MessageWithAttachments {
            message,
            attachments: attachment_responses,
        }
    }))
}

async fn get_message(
    State(state): State<AppState>,
    Path((dialog_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<Message>>, ApiError> {
    let message = state.messages.find_by_id_and_dialog(message_id, dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Message not found".into()))?;

    Ok(Json(ApiResponse { data: message }))
}

async fn edit_message(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path((dialog_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<EditMessageRequest>,
) -> Result<Json<ApiResponse<Message>>, ApiError> {
    // Find message
    let message = state.messages.find_by_id_and_dialog(message_id, dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Message not found".into()))?;

    // Check ownership (only author can edit)
    if message.sender_id != Some(user_id) {
        return Err(ApiError::Forbidden("Can only edit own messages".into()));
    }

    // Can't edit system messages
    if message.message_type != domain::MessageType::User {
        return Err(ApiError::BadRequest("Cannot edit system messages".into()));
    }

    // Save old content to history
    state.messages.save_edit_history(message_id, &message.content).await?;

    // Sanitize and update content
    let sanitized = domain::sanitize_html(&req.content);
    let updated = state.messages.update_content(message_id, &sanitized).await?
        .ok_or_else(|| ApiError::Internal("Failed to update message".into()))?;

    // Broadcast via WebSocket
    ws::broadcast_message_edited(&state.connections, &updated).await;

    Ok(Json(ApiResponse { data: updated }))
}

async fn delete_message(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path((dialog_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    // Find message
    let message = state.messages.find_by_id_and_dialog(message_id, dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Message not found".into()))?;

    // Check ownership
    if message.sender_id != Some(user_id) {
        return Err(ApiError::Forbidden("Can only delete own messages".into()));
    }

    // Delete message
    state.messages.delete(message_id).await?;

    // Broadcast via WebSocket
    ws::broadcast_message_deleted(&state.connections, dialog_id, message_id).await;

    Ok(StatusCode::NO_CONTENT)
}

// ============ Handlers: Upload ============

async fn presign_upload(
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

async fn get_attachment_url(
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

// ============ Handlers: Participants ============

/// Participant response with online status
#[derive(Debug, Serialize)]
struct ParticipantResponse {
    #[serde(flatten)]
    participant: DialogParticipant,
    is_online: bool,
}

async fn list_participants(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<ParticipantResponse>>>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    let participants = state.participants.list_by_dialog(dialog_id).await?;

    // Get online status for all participants
    let user_ids: Vec<Uuid> = participants.iter().map(|p| p.user_id).collect();
    let online_users = state.presence.get_online_users(&user_ids).await.unwrap_or_default();

    // Build response with online status
    let responses: Vec<ParticipantResponse> = participants
        .into_iter()
        .map(|p| ParticipantResponse {
            is_online: online_users.contains(&p.user_id),
            participant: p,
        })
        .collect();

    Ok(Json(ApiResponse { data: responses }))
}

async fn mark_as_read(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<MarkAsReadRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check dialog exists
    state.dialogs.find_by_id(dialog_id).await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    // Mark as read
    state.participants.mark_as_read(dialog_id, user_id, req.last_read_message_id).await?;

    // Broadcast WebSocket event
    ws::broadcast_read(&state.connections, dialog_id, user_id, req.last_read_message_id).await;

    Ok(Json(serde_json::json!({
        "success": true
    })))
}

// WebSocket
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = params.get("user_id")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(Uuid::new_v4);

    ws.on_upgrade(move |socket| {
        ws::handle_socket(
            socket,
            state.connections,
            user_id,
            state.presence,
            state.participants,
        )
    })
}

// ============ Main ============

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "multitenancy_chat_api=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/multitenancy_chat".into());

    tracing::info!("Connecting to database...");
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    // Initialize webhook sender
    let webhooks = match (env::var("WEBHOOK_URL"), env::var("WEBHOOK_SECRET")) {
        (Ok(url), Ok(secret)) => {
            tracing::info!("Webhooks enabled, sending to: {}", url);
            WebhookSender::new(WebhookConfig::new(url, secret))
        }
        _ => {
            tracing::info!("Webhooks disabled (WEBHOOK_URL or WEBHOOK_SECRET not set)");
            WebhookSender::noop()
        }
    };

    // Initialize S3 service
    let s3 = match S3Config::from_env() {
        Ok(config) => {
            tracing::info!("S3 enabled, bucket: {}", config.bucket);
            S3Service::new(config).await
        }
        Err(e) => {
            tracing::warn!("S3 disabled: {}", e);
            S3Service::noop()
        }
    };

    // Initialize Redis and presence service
    let presence = match env::var("REDIS_URL") {
        Ok(url) => {
            tracing::info!("Connecting to Redis...");
            let config = Config::from_url(&url)
                .expect("Failed to parse REDIS_URL");
            let pool = Builder::from_config(config)
                .build_pool(5)
                .expect("Failed to create Redis pool");
            pool.init().await.expect("Failed to connect to Redis");
            tracing::info!("Redis connected, presence tracking enabled");
            PresenceService::new(Arc::new(pool))
        }
        Err(_) => {
            tracing::info!("Redis disabled (REDIS_URL not set), presence tracking disabled");
            PresenceService::noop()
        }
    };

    let state = AppState::new(db, webhooks, s3, presence);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Management API routes (with admin auth middleware)
    let management_routes = Router::new()
        .route("/dialogs", post(management_create_dialog))
        .route("/dialogs/{id}", get(management_get_dialog).delete(management_delete_dialog))
        .route("/dialogs/{id}/participants", post(management_add_participant))
        .route("/dialogs/{id}/participants/{user_id}", delete(management_remove_participant))
        .route("/dialogs/{id}/access-scopes", put(management_update_access_scopes))
        .layer(axum_middleware::from_fn(middleware::admin_auth::admin_auth));

    let app = Router::new()
        // Health
        .route("/health", get(health))
        .route("/health/ready", get(health_ready))

        // Management API (protected)
        .nest("/api/v1/management", management_routes)

        // Chat API - Dialogs
        .route("/api/v1/dialogs", get(list_dialogs))
        .route("/api/v1/dialogs/{id}", get(get_dialog))
        .route("/api/v1/dialogs/by-object/{object_type}/{object_id}", get(get_dialog_by_object))
        .route("/api/v1/dialogs/{id}/join", post(join_dialog))
        .route("/api/v1/dialogs/{id}/leave", post(leave_dialog))
        .route("/api/v1/dialogs/{id}/archive", post(archive_dialog))
        .route("/api/v1/dialogs/{id}/unarchive", post(unarchive_dialog))
        .route("/api/v1/dialogs/{id}/read", post(mark_as_read))
        .route("/api/v1/dialogs/{id}/participants", get(list_participants))

        // Chat API - Messages
        .route("/api/v1/dialogs/{dialog_id}/messages", get(list_messages).post(send_message))
        .route("/api/v1/dialogs/{dialog_id}/messages/{id}", get(get_message).put(edit_message).delete(delete_message))

        // Upload API
        .route("/api/v1/upload/presign", post(presign_upload))
        .route("/api/v1/attachments/{id}/url", get(get_attachment_url))

        // WebSocket
        .route("/api/v1/ws", get(ws_handler))

        .layer(cors)
        .with_state(state);

    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
