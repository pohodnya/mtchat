use axum::extract::{Path, Query, State};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    self, system_messages, Dialog, DialogParticipant, JoinedAs, Message, ParticipantProfile,
};
use crate::middleware::{OptionalScopeConfig, ScopeConfig, UserId};
use crate::webhooks::WebhookEvent;
use crate::ws;

use super::{ApiError, ApiResponse, AppState, ErrorCode};

// ============ DTOs ============

fn default_dialogs_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct DialogsQuery {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default = "default_dialogs_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListByObjectQuery {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
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
pub struct LastMessage {
    pub id: Uuid,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_name: Option<String>,
    pub sent_at: chrono::DateTime<chrono::Utc>,
    pub message_type: String,
}

#[derive(Debug, Serialize)]
pub struct ParticipantSummary {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<LastMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participants: Option<Vec<ParticipantSummary>>,
}

/// Build a `LastMessage` DTO from a message, resolving `sender_name` from the
/// dialog's participants. System messages (no `sender_id`) get no sender name.
fn build_last_message(msg: &Message, participants: &[DialogParticipant]) -> LastMessage {
    let sender_name = msg.sender_id.as_ref().and_then(|sid| {
        participants
            .iter()
            .find(|p| &p.user_id == sid)
            .and_then(|p| p.display_name.clone())
    });
    LastMessage {
        id: msg.id,
        content: msg.content.clone(),
        sender_id: msg.sender_id.clone(),
        sender_name,
        sent_at: msg.sent_at,
        message_type: msg.message_type.as_str().to_string(),
    }
}

/// Build the participant summary list for a dialog from its participants.
fn build_participant_summaries(participants: &[DialogParticipant]) -> Vec<ParticipantSummary> {
    participants
        .iter()
        .map(|p| ParticipantSummary {
            user_id: p.user_id.clone(),
            display_name: p.display_name.clone(),
            company: p.company.clone(),
        })
        .collect()
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

    // Cap limit at 100 to prevent excessive queries
    let limit = params.limit.min(100);
    let offset = params.offset.max(0);

    let dialogs = match dialog_type {
        "participating" => {
            state
                .dialogs
                .find_participating(&user_id, search, archived, limit, offset)
                .await?
        }
        "available" => {
            // Available dialogs are never archived (user is not a participant yet)
            if let Some(scope) = &scope_config {
                state
                    .dialogs
                    .find_available(
                        &user_id,
                        &scope.scope_level0,
                        &scope.scope_level1,
                        &scope.scope_level2,
                        search,
                        limit,
                        offset,
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
            .find_by_dialogs_and_user(&dialog_ids, &user_id)
            .await?
    } else {
        std::collections::HashMap::new()
    };
    let last_message_full_map = state.dialogs.get_last_message_batch(&dialog_ids).await?;
    let all_participants_map = state
        .participants
        .list_by_dialogs_batch(&dialog_ids)
        .await?;

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

        let dialog_participants = all_participants_map.get(&dialog.id);
        // last_message exposes message content, so it is only returned to actual
        // participants (consistent with the v0.3.7 "no reading before join" rule).
        // The participant list itself is not sensitive and is always returned.
        let last_message = if dialog_type == "participating" {
            last_message_full_map.get(&dialog.id).map(|m| {
                build_last_message(m, dialog_participants.map(|v| v.as_slice()).unwrap_or(&[]))
            })
        } else {
            None
        };
        let participants = dialog_participants.map(|v| build_participant_summaries(v));

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
            last_message,
            participants,
        });
    }

    Ok(Json(ApiResponse { data: responses }))
}

pub async fn list_dialogs_by_object(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    OptionalScopeConfig(scope_config): OptionalScopeConfig,
    Path((object_type, object_id)): Path<(String, String)>,
    Query(params): Query<ListByObjectQuery>,
) -> Result<Json<ApiResponse<Vec<DialogResponse>>>, ApiError> {
    // Validate the optional type filter (mirrors `GET /api/v1/dialogs`).
    // Absent → both branches; participating/available → that branch only.
    let dialog_type = match params.r#type.as_deref() {
        None | Some("participating") | Some("available") => params.r#type.as_deref(),
        _ => return Err(ApiError::BadRequest("Invalid type parameter".into())),
    };

    let scope = scope_config.as_ref().map(|s| {
        (
            s.scope_level0.as_slice(),
            s.scope_level1.as_slice(),
            s.scope_level2.as_slice(),
        )
    });

    // Available (can-join) dialogs have no per-user archived state, so the
    // archived filter only applies to the participant branch.
    let archived = if dialog_type == Some("available") {
        None
    } else {
        params.archived
    };

    let dialogs = state
        .dialogs
        .find_all_by_object_for_user(
            &object_type,
            &object_id,
            &user_id,
            scope,
            archived,
            params.search.as_deref(),
            dialog_type,
        )
        .await?;

    // Batch fetch supplementary data in parallel to avoid N+1 queries
    let dialog_ids: Vec<Uuid> = dialogs.iter().map(|d| d.id).collect();
    let participants_count_map = state.dialogs.count_participants_batch(&dialog_ids).await?;
    let last_message_map = state.dialogs.get_last_message_at_batch(&dialog_ids).await?;
    let participant_map = state
        .participants
        .find_by_dialogs_and_user(&dialog_ids, &user_id)
        .await?;
    let last_message_full_map = state.dialogs.get_last_message_batch(&dialog_ids).await?;
    let all_participants_map = state
        .participants
        .list_by_dialogs_batch(&dialog_ids)
        .await?;

    let mut responses = Vec::new();
    for dialog in dialogs {
        let participants_count = participants_count_map.get(&dialog.id).copied().unwrap_or(0);
        let last_message_at = last_message_map.get(&dialog.id).copied();
        let participant = participant_map.get(&dialog.id);
        let i_am_participant = participant.is_some();

        let (unread_count, is_archived, is_pinned, notifications_enabled) = (
            participant.map(|p| p.unread_count as i64),
            participant.map(|p| p.is_archived),
            participant.map(|p| p.is_pinned),
            participant.map(|p| p.notifications_enabled),
        );

        let dialog_participants = all_participants_map.get(&dialog.id);
        // last_message exposes message content, so it is only returned to actual
        // participants (v0.3.7 "no reading before join"). participants is always returned.
        let last_message = if i_am_participant {
            last_message_full_map.get(&dialog.id).map(|m| {
                build_last_message(m, dialog_participants.map(|v| v.as_slice()).unwrap_or(&[]))
            })
        } else {
            None
        };
        let participants = dialog_participants.map(|v| build_participant_summaries(v));

        responses.push(DialogResponse {
            dialog,
            participants_count,
            i_am_participant: Some(i_am_participant),
            can_join: Some(!i_am_participant),
            unread_count,
            is_archived,
            is_pinned,
            notifications_enabled,
            last_message_at,
            last_message,
            participants,
        });
    }

    Ok(Json(ApiResponse { data: responses }))
}

pub async fn get_dialog_by_object(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    OptionalScopeConfig(scope_config): OptionalScopeConfig,
    Path((object_type, object_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<Option<DialogResponse>>>, ApiError> {
    let dialog = state
        .dialogs
        .find_by_object(&object_type, &object_id)
        .await?;

    if let Some(dialog) = dialog {
        let i_am_participant = state.participants.exists(dialog.id, &user_id).await?;

        let can_join = if !i_am_participant {
            if let Some(scope) = &scope_config {
                state
                    .scopes
                    .check_access(
                        dialog.id,
                        &scope.scope_level0,
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
            return Err(ApiError::new(
                ErrorCode::ScopeMismatch,
                "No access to this dialog",
            ));
        }

        // Use batch methods to avoid N+1 queries (consistent with list_dialogs)
        let dialog_ids = &[dialog.id];
        let participants_count_map = state.dialogs.count_participants_batch(dialog_ids).await?;
        let last_message_map = state.dialogs.get_last_message_at_batch(dialog_ids).await?;
        let participants_count = participants_count_map.get(&dialog.id).copied().unwrap_or(0);
        let last_message_at = last_message_map.get(&dialog.id).copied();

        let last_message_full_map = state.dialogs.get_last_message_batch(dialog_ids).await?;
        let all_participants_map = state.participants.list_by_dialogs_batch(dialog_ids).await?;
        let dialog_participants = all_participants_map.get(&dialog.id);
        // last_message exposes message content, so it is only returned to actual
        // participants (v0.3.7 "no reading before join"). participants is always returned.
        let last_message = if i_am_participant {
            last_message_full_map.get(&dialog.id).map(|m| {
                build_last_message(m, dialog_participants.map(|v| v.as_slice()).unwrap_or(&[]))
            })
        } else {
            None
        };
        let participants = dialog_participants.map(|v| build_participant_summaries(v));

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
                last_message,
                participants,
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
        .ok_or_else(|| ApiError::new(ErrorCode::DialogNotFound, "Dialog not found"))?;

    // Check if already participant
    if state.participants.exists(dialog_id, &user_id).await? {
        return Err(ApiError::BadRequest("Already a participant".into()));
    }

    // Check scope access
    let has_access = state
        .scopes
        .check_access(
            dialog_id,
            &scope_config.scope_level0,
            &scope_config.scope_level1,
            &scope_config.scope_level2,
        )
        .await?;

    if !has_access {
        return Err(ApiError::Forbidden("No access to join this dialog".into()));
    }

    // Validate input
    domain::validation::validate_display_name(&req.display_name)
        .map_err(|e| ApiError::new(ErrorCode::InvalidInput, e.message))?;
    domain::validation::validate_company(&Some(req.company.clone()))
        .map_err(|e| ApiError::new(ErrorCode::InvalidInput, e.message))?;
    domain::validation::validate_email(&req.email)
        .map_err(|e| ApiError::new(ErrorCode::InvalidInput, e.message))?;
    domain::validation::validate_phone(&req.phone)
        .map_err(|e| ApiError::new(ErrorCode::InvalidInput, e.message))?;

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
    .bind(&user_id)
    .bind(&JoinedAs::Joined)
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
    .bind(&user_id)
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
    ws::broadcast_participant_joined(&state.connections, dialog_id, &user_id).await;
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
        .ok_or_else(|| ApiError::new(ErrorCode::DialogNotFound, "Dialog not found"))?;

    // Get participant before removing to get display_name
    let participant = state.participants.find(dialog_id, &user_id).await?;
    let display_name = participant
        .as_ref()
        .and_then(|p| p.display_name.clone())
        .unwrap_or_else(|| "Участник".to_string());

    // Create system message
    let system_msg = Message::system(
        dialog_id,
        system_messages::participant_left_content(&display_name),
    );

    // All DB writes in a transaction
    let mut tx = state.db.begin().await?;

    // Remove participant
    sqlx::query("DELETE FROM dialog_participants WHERE dialog_id = $1 AND user_id = $2")
        .bind(dialog_id)
        .bind(&user_id)
        .execute(&mut *tx)
        .await?;

    // Insert system message
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
    ws::broadcast_participant_left(&state.connections, dialog_id, &user_id).await;
    state
        .webhooks
        .send(WebhookEvent::participant_left(&dialog, &user_id))
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
    if !state.participants.exists(dialog_id, &user_id).await? {
        return Err(ApiError::new(
            ErrorCode::NotParticipant,
            "Not a participant",
        ));
    }

    state
        .participants
        .set_archived(dialog_id, &user_id, true)
        .await?;

    Ok(Json(serde_json::json!({ "status": "archived" })))
}

pub async fn unarchive_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, &user_id).await? {
        return Err(ApiError::new(
            ErrorCode::NotParticipant,
            "Not a participant",
        ));
    }

    state
        .participants
        .set_archived(dialog_id, &user_id, false)
        .await?;

    Ok(Json(serde_json::json!({ "status": "unarchived" })))
}

pub async fn pin_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, &user_id).await? {
        return Err(ApiError::new(
            ErrorCode::NotParticipant,
            "Not a participant",
        ));
    }

    state
        .participants
        .set_pinned(dialog_id, &user_id, true)
        .await?;

    Ok(Json(serde_json::json!({ "status": "pinned" })))
}

pub async fn unpin_dialog(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check user is participant
    if !state.participants.exists(dialog_id, &user_id).await? {
        return Err(ApiError::new(
            ErrorCode::NotParticipant,
            "Not a participant",
        ));
    }

    state
        .participants
        .set_pinned(dialog_id, &user_id, false)
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
    if !state.participants.exists(dialog_id, &user_id).await? {
        return Err(ApiError::new(
            ErrorCode::NotParticipant,
            "Not a participant",
        ));
    }

    state
        .participants
        .set_notifications(dialog_id, &user_id, body.enabled)
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
        .ok_or_else(|| ApiError::new(ErrorCode::DialogNotFound, "Dialog not found"))?;

    // Check access: must be participant OR have scope access (potential participant)
    let is_participant = state.participants.exists(dialog_id, &user_id).await?;
    let has_scope_access = state
        .scopes
        .check_access(
            dialog_id,
            &scope_config.scope_level0,
            &scope_config.scope_level1,
            &scope_config.scope_level2,
        )
        .await?;

    if !is_participant && !has_scope_access {
        return Err(ApiError::new(
            ErrorCode::ScopeMismatch,
            "No access to this dialog",
        ));
    }

    Ok(Json(ApiResponse { data: dialog }))
}
