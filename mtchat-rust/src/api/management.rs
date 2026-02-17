use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    Dialog, DialogParticipant, DialogAccessScope, JoinedAs, ParticipantProfile,
    system_messages, Message,
};
use crate::ws;

use super::{AppState, ApiResponse, ApiError};

// ============ DTOs ============

#[derive(Debug, Deserialize)]
pub struct ParticipantInput {
    pub user_id: Uuid,
    pub display_name: String,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDialogRequest {
    pub object_id: Uuid,
    pub object_type: String,
    pub title: Option<String>,
    pub object_url: Option<String>,
    pub participants: Vec<ParticipantInput>,
    #[serde(default)]
    pub access_scopes: Vec<AccessScopeInput>,
}

#[derive(Debug, Deserialize)]
pub struct AccessScopeInput {
    pub tenant_uid: Uuid,
    #[serde(default)]
    pub scope_level1: Vec<String>,
    #[serde(default)]
    pub scope_level2: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddParticipantRequest {
    pub user_id: Uuid,
    pub display_name: String,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccessScopesRequest {
    pub access_scopes: Vec<AccessScopeInput>,
}

#[derive(Debug, Serialize)]
pub struct ManagementDialogResponse {
    #[serde(flatten)]
    pub dialog: Dialog,
    pub participants: Vec<DialogParticipant>,
    pub access_scopes: Vec<DialogAccessScope>,
}

// ============ Handlers ============

pub async fn management_create_dialog(
    State(state): State<AppState>,
    Json(req): Json<CreateDialogRequest>,
) -> Result<Json<ApiResponse<Dialog>>, ApiError> {
    let mut tx = state.db.begin().await?;

    // Create dialog
    let created_by = req.participants.first().map(|p| p.user_id);
    let dialog = Dialog::new(
        req.object_id,
        req.object_type,
        req.title,
        req.object_url,
        created_by,
    );
    let dialog = sqlx::query_as::<_, Dialog>(
        r#"INSERT INTO dialogs (id, object_id, object_type, title, object_url, created_by, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#,
    )
    .bind(dialog.id)
    .bind(dialog.object_id)
    .bind(&dialog.object_type)
    .bind(&dialog.title)
    .bind(&dialog.object_url)
    .bind(dialog.created_by)
    .bind(dialog.created_at)
    .fetch_one(&mut *tx)
    .await?;

    // Add participants with their profiles
    for participant in req.participants.iter() {
        let profile = ParticipantProfile {
            display_name: participant.display_name.clone(),
            company: participant.company.clone(),
            email: participant.email.clone(),
            phone: participant.phone.clone(),
        };
        sqlx::query(
            r#"INSERT INTO dialog_participants
               (dialog_id, user_id, joined_as, joined_at, display_name, company, email, phone)
               VALUES ($1, $2, $3, NOW(), $4, $5, $6, $7)"#,
        )
        .bind(dialog.id)
        .bind(participant.user_id)
        .bind(JoinedAs::Participant.as_str())
        .bind(&profile.display_name)
        .bind(&profile.company)
        .bind(&profile.email)
        .bind(&profile.phone)
        .execute(&mut *tx)
        .await?;
    }

    // Add access scopes
    for scope_input in req.access_scopes {
        let scope = DialogAccessScope::new(
            dialog.id,
            scope_input.tenant_uid,
            scope_input.scope_level1,
            scope_input.scope_level2,
        );
        sqlx::query(
            r#"INSERT INTO dialog_access_scopes (id, dialog_id, tenant_uid, scope_level1, scope_level2, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(scope.id)
        .bind(scope.dialog_id)
        .bind(scope.tenant_uid)
        .bind(&scope.scope_level1)
        .bind(&scope.scope_level2)
        .bind(scope.created_at)
        .execute(&mut *tx)
        .await?;
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
        sqlx::query(
            r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, reply_to_id, message_type)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(system_msg.id)
        .bind(system_msg.dialog_id)
        .bind(system_msg.sender_id)
        .bind(&system_msg.content)
        .bind(system_msg.sent_at)
        .bind(system_msg.reply_to_id)
        .bind(system_msg.message_type.as_str())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Broadcast events after transaction is committed
    for participant in req.participants.iter() {
        ws::broadcast_participant_joined(&state.connections, dialog.id, participant.user_id).await;
    }

    Ok(Json(ApiResponse { data: dialog }))
}

pub async fn management_add_participant(
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

    // Broadcast participant joined event (for dialog list updates)
    ws::broadcast_participant_joined(&state.connections, dialog_id, req.user_id).await;

    Ok(StatusCode::CREATED)
}

pub async fn management_remove_participant(
    State(state): State<AppState>,
    Path((dialog_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.participants.remove(dialog_id, user_id).await?;

    // Broadcast participant left event (for dialog list updates)
    ws::broadcast_participant_left(&state.connections, dialog_id, user_id).await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn management_delete_dialog(
    State(state): State<AppState>,
    Path(dialog_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.dialogs.delete(dialog_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn management_update_access_scopes(
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

pub async fn management_get_dialog(
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
