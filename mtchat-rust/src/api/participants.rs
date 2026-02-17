use axum::extract::{Path, State};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::DialogParticipant;
use crate::middleware::{OptionalScopeConfig, UserId};
use crate::ws;

use super::{ApiError, ApiResponse, AppState};

// ============ DTOs ============

#[derive(Debug, Serialize)]
pub struct ParticipantResponse {
    #[serde(flatten)]
    pub participant: DialogParticipant,
    pub is_online: bool,
}

#[derive(Debug, Deserialize)]
pub struct MarkAsReadRequest {
    pub last_read_message_id: Uuid,
}

// ============ Handlers ============

pub async fn list_participants(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    OptionalScopeConfig(scope_config): OptionalScopeConfig,
    Path(dialog_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<ParticipantResponse>>>, ApiError> {
    // Check if user is participant
    let is_participant = state.participants.exists(dialog_id, user_id).await?;

    // If not participant, check scope access (potential participant)
    if !is_participant {
        let has_scope_access = if let Some(scope) = &scope_config {
            state
                .scopes
                .check_access(
                    dialog_id,
                    scope.tenant_uid.clone(),
                    &scope.scope_level1,
                    &scope.scope_level2,
                )
                .await?
        } else {
            false
        };

        if !has_scope_access {
            return Err(ApiError::Forbidden("Not a participant".into()));
        }
    }

    let participants = state.participants.list_by_dialog(dialog_id).await?;

    // Get online status for all participants
    let user_ids: Vec<Uuid> = participants.iter().map(|p| p.user_id).collect();
    let online_users = state
        .presence
        .get_online_users(&user_ids)
        .await
        .unwrap_or_default();

    // Build response with online status
    // For non-participants, hide contact details (email, phone)
    let responses: Vec<ParticipantResponse> = participants
        .into_iter()
        .map(|p| {
            let participant = if is_participant {
                p
            } else {
                // Hide contacts for potential participants
                DialogParticipant {
                    email: None,
                    phone: None,
                    ..p
                }
            };
            ParticipantResponse {
                is_online: online_users.contains(&participant.user_id),
                participant,
            }
        })
        .collect();

    Ok(Json(ApiResponse { data: responses }))
}

pub async fn mark_as_read(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<MarkAsReadRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check dialog exists
    state
        .dialogs
        .find_by_id(dialog_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check user is participant
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden("Not a participant".into()));
    }

    // Mark as read
    state
        .participants
        .mark_as_read(dialog_id, user_id, req.last_read_message_id)
        .await?;

    // Broadcast WebSocket event
    ws::broadcast_read(
        &state.connections,
        dialog_id,
        user_id,
        req.last_read_message_id,
    )
    .await;

    Ok(Json(serde_json::json!({
        "success": true
    })))
}
