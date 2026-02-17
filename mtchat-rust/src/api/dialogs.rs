use axum::extract::{Path, Query, State};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{system_messages, Dialog, JoinedAs, Message, ParticipantProfile};
use crate::middleware::{OptionalScopeConfig, ScopeConfig, UserId};
use crate::webhooks::WebhookEvent;
use crate::ws;

use super::{ApiError, ApiResponse, AppState};

// ============ DTOs ============

#[derive(Debug, Deserialize)]
pub struct DialogsQuery {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JoinDialogRequest {
    pub display_name: String,
    pub company: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetNotificationsRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct DialogResponse {
    #[serde(flatten)]
    pub dialog: Dialog,
    pub participants_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i_am_participant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_join: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============ Handlers ============

pub async fn list_dialogs(
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
            state
                .dialogs
                .find_participating(user_id, search, archived)
                .await?
        }
        "available" => {
            // Available dialogs are never archived (user is not a participant yet)
            if let Some(scope) = &scope_config {
                state
                    .dialogs
                    .find_available(
                        user_id,
                        scope.tenant_uid,
                        &scope.scope_level1,
                        &scope.scope_level2,
                        search,
                    )
                    .await?
            } else {
                return Err(ApiError::BadRequest(
                    "X-Scope-Config header required for available dialogs".into(),
                ));
            }
        }
        _ => {
            return Err(ApiError::BadRequest("Invalid type parameter".into()));
        }
    };

    // Batch fetch all supplementary data in parallel to avoid N+1 queries
    let dialog_ids: Vec<Uuid> = dialogs.iter().map(|d| d.id).collect();
    let last_message_map = state.dialogs.get_last_message_at_batch(&dialog_ids).await?;
    let participants_count_map = state.dialogs.count_participants_batch(&dialog_ids).await?;
    let participant_map = if dialog_type == "participating" {
        state
            .participants
            .find_by_dialogs_and_user(&dialog_ids, user_id)
            .await?
    } else {
        std::collections::HashMap::new()
    };

    // Build responses using batch-fetched data
    let mut responses = Vec::new();
    for dialog in dialogs {
        let participants_count = participants_count_map.get(&dialog.id).copied().unwrap_or(0);

        let (unread_count, is_archived, is_pinned, notifications_enabled) =
            if dialog_type == "participating" {
                let participant = participant_map.get(&dialog.id);
                (
                    participant.map(|p| p.unread_count as i64),
                    participant.map(|p| p.is_archived),
                    participant.map(|p| p.is_pinned),
                    participant.map(|p| p.notifications_enabled),
                )
            } else {
                (None, None, None, None)
            };

        let last_message_at = last_message_map.get(&dialog.id).copied();

        responses.push(DialogResponse {
            dialog,
            participants_count,
            i_am_participant: Some(dialog_type == "participating"),
            can_join: Some(dialog_type == "available"),
            unread_count,
            is_archived,
            is_pinned,
            notifications_enabled,
            last_message_at,
        });
    }

    Ok(Json(ApiResponse { data: responses }))
}

pub async fn get_dialog_by_object(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    OptionalScopeConfig(scope_config): OptionalScopeConfig,
    Path((object_type, object_id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<Option<DialogResponse>>>, ApiError> {
    let dialog = state
        .dialogs
        .find_by_object(&object_type, object_id)
        .await?;

    if let Some(dialog) = dialog {
        let i_am_participant = state.participants.exists(dialog.id, user_id).await?;

        let can_join = if !i_am_participant {
            if let Some(scope) = &scope_config {
                state
                    .scopes
                    .check_access(
                        dialog.id,
                        scope.tenant_uid,
                        &scope.scope_level1,
                        &scope.scope_level2,
                    )
                    .await?
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

        let last_message_at = state.dialogs.get_last_message_at(dialog.id).await?;

        Ok(Json(ApiResponse {
            data: Some(DialogResponse {
                dialog,
                participants_count,
                i_am_participant: Some(i_am_participant),
                can_join: Some(can_join),
                unread_count: None,
                is_archived: None,
                is_pinned: None,
                notifications_enabled: None,
                last_message_at,
            }),
        }))
    } else {
        Ok(Json(ApiResponse { data: None }))
    }
}

pub async fn join_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    scope_config: ScopeConfig,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<JoinDialogRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check dialog exists
    let dialog = state
        .dialogs
        .find_by_id(dialog_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check if already participant
    if state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::BadRequest("Already a participant".into()));
    }

    // Check scope access
    let has_access = state
        .scopes
        .check_access(
            dialog_id,
            scope_config.tenant_uid,
            &scope_config.scope_level1,
            &scope_config.scope_level2,
        )
        .await?;

    if !has_access {
        return Err(ApiError::Forbidden("No access to join this dialog".into()));
    }

    // All DB writes in a transaction
    let mut tx = state.db.begin().await?;

    // Join with profile
    let profile = ParticipantProfile {
        display_name: req.display_name.clone(),
        company: Some(req.company.clone()),
        email: req.email.clone(),
        phone: req.phone.clone(),
    };
    let participant = sqlx::query_as::<_, crate::domain::DialogParticipant>(
        r#"INSERT INTO dialog_participants
           (dialog_id, user_id, joined_as, joined_at, display_name, company, email, phone)
           VALUES ($1, $2, $3, NOW(), $4, $5, $6, $7)
           RETURNING *"#,
    )
    .bind(dialog_id)
    .bind(user_id)
    .bind(JoinedAs::Joined.as_str())
    .bind(&profile.display_name)
    .bind(&profile.company)
    .bind(&profile.email)
    .bind(&profile.phone)
    .fetch_one(&mut *tx)
    .await?;

    // Set unread count to total messages in dialog
    sqlx::query(
        r#"UPDATE dialog_participants
           SET unread_count = (SELECT COUNT(*) FROM messages WHERE dialog_id = $1)
           WHERE dialog_id = $1 AND user_id = $2"#,
    )
    .bind(dialog_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Create system message "participant joined"
    let system_msg = Message::system(
        dialog_id,
        system_messages::participant_joined_content(&req.display_name, Some(&req.company)),
    );
    let system_msg = sqlx::query_as::<_, Message>(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, reply_to_id, message_type)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#,
    )
    .bind(system_msg.id)
    .bind(system_msg.dialog_id)
    .bind(system_msg.sender_id)
    .bind(&system_msg.content)
    .bind(system_msg.sent_at)
    .bind(system_msg.reply_to_id)
    .bind(system_msg.message_type.as_str())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    // Broadcast and webhook after transaction is committed
    ws::broadcast_message(&state.connections, dialog_id, &system_msg).await;
    ws::broadcast_participant_joined(&state.connections, dialog_id, user_id).await;
    state
        .webhooks
        .send(WebhookEvent::participant_joined(&dialog, &participant))
        .await;

    Ok(Json(serde_json::json!({
        "status": "joined",
        "dialog_id": dialog.id
    })))
}

pub async fn leave_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get dialog for webhook
    let dialog = state
        .dialogs
        .find_by_id(dialog_id)
        .await?
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

    // Broadcast participant left event (for dialog list updates)
    ws::broadcast_participant_left(&state.connections, dialog_id, user_id).await;

    // Send webhook
    state
        .webhooks
        .send(WebhookEvent::participant_left(&dialog, user_id))
        .await;

    Ok(Json(serde_json::json!({
        "status": "left"
    })))
}

pub async fn archive_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state
        .participants
        .set_archived(dialog_id, user_id, true)
        .await?;

    Ok(Json(serde_json::json!({ "status": "archived" })))
}

pub async fn unarchive_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state
        .participants
        .set_archived(dialog_id, user_id, false)
        .await?;

    Ok(Json(serde_json::json!({ "status": "unarchived" })))
}

pub async fn pin_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state
        .participants
        .set_pinned(dialog_id, user_id, true)
        .await?;

    Ok(Json(serde_json::json!({ "status": "pinned" })))
}

pub async fn unpin_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state
        .participants
        .set_pinned(dialog_id, user_id, false)
        .await?;

    Ok(Json(serde_json::json!({ "status": "unpinned" })))
}

pub async fn set_dialog_notifications(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Json(body): Json<SetNotificationsRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    state
        .participants
        .set_notifications(dialog_id, user_id, body.enabled)
        .await?;

    Ok(Json(serde_json::json!({
        "status": if body.enabled { "enabled" } else { "disabled" }
    })))
}

pub async fn get_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    scope_config: ScopeConfig,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Dialog>>, ApiError> {
    let dialog = state
        .dialogs
        .find_by_id(dialog_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check access: must be participant OR have scope access (potential participant)
    let is_participant = state.participants.exists(dialog_id, user_id).await?;
    let has_scope_access = state
        .scopes
        .check_access(
            dialog_id,
            scope_config.tenant_uid,
            &scope_config.scope_level1,
            &scope_config.scope_level2,
        )
        .await?;

    if !is_participant && !has_scope_access {
        return Err(ApiError::Forbidden("No access to this dialog".into()));
    }

    Ok(Json(ApiResponse { data: dialog }))
}
