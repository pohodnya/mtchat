use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{self, Message};
use crate::jobs::NotificationJob;
use crate::middleware::UserId;
use crate::webhooks::WebhookEvent;
use crate::ws;

use super::{ApiError, ApiResponse, AppState};

// ============ DTOs ============

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub reply_to: Option<Uuid>,
    #[serde(default)]
    pub attachments: Vec<domain::AttachmentInput>,
}

#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub before: Option<Uuid>,
    pub after: Option<Uuid>,
    pub around: Option<Uuid>,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct MessageWithAttachments {
    #[serde(flatten)]
    pub message: Message,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<domain::AttachmentResponse>,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<MessageWithAttachments>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_unread_message_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more_before: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more_after: Option<bool>,
}

// ============ Handlers ============

pub async fn list_messages(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<MessagesResponse>>, ApiError> {
    // Check user is participant (potential participants cannot read messages)
    if !state.participants.exists(dialog_id, user_id).await? {
        return Err(ApiError::Forbidden(
            "Not a participant. Join the dialog first.".into(),
        ));
    }

    // Determine pagination mode: around, after, before, or latest
    let (messages, has_more_before, has_more_after) = if let Some(around_id) = pagination.around {
        // Load messages centered around a specific message (jump to message)
        state
            .messages
            .list_around(dialog_id, around_id, pagination.limit)
            .await?
    } else if let Some(after_id) = pagination.after {
        // Load messages AFTER a specific message (scroll down to load newer)
        let msgs = state
            .messages
            .list_after(dialog_id, after_id, pagination.limit)
            .await?;
        let has_more = msgs.len() as i64 >= pagination.limit;
        // has_more_before is always true when using "after" (we came from scrolling up)
        // has_more_after is true if we got a full page
        (msgs, true, has_more)
    } else {
        // Regular pagination (before or latest)
        let msgs = state
            .messages
            .list_by_dialog(dialog_id, pagination.limit, pagination.before)
            .await?;
        // For regular pagination, has_more_before is true if we got a full page
        let has_more = msgs.len() as i64 >= pagination.limit;
        (msgs, has_more, false)
    };

    // Get participant to find first unread message (only for regular pagination, not "around")
    let first_unread_message_id = if pagination.around.is_none() {
        let participant = state.participants.find(dialog_id, user_id).await?;
        if let Some(ref p) = participant {
            if let Some(last_read_id) = p.last_read_message_id {
                // Find the last read message to get its sent_at
                let last_read_msg = messages.iter().find(|m| m.id == last_read_id);
                let last_read_sent_at = if let Some(msg) = last_read_msg {
                    Some(msg.sent_at)
                } else {
                    // Last read message not in current page - fetch from DB
                    state
                        .messages
                        .find_by_id(last_read_id)
                        .await?
                        .map(|m| m.sent_at)
                };

                if let Some(sent_at) = last_read_sent_at {
                    // Find first message sent AFTER the last read message
                    messages.iter().find(|m| m.sent_at > sent_at).map(|m| m.id)
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
                state
                    .s3
                    .generate_download_url(&att.s3_key)
                    .await
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

            attachment_responses.push(domain::AttachmentResponse::from_attachment(
                att,
                url,
                thumbnail_url,
            ));
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
            has_more_before: Some(has_more_before),
            has_more_after: Some(has_more_after),
        },
    }))
}

pub async fn send_message(
    State(state): State<AppState>,
    UserId(sender_id): UserId,
    Path(dialog_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<ApiResponse<MessageWithAttachments>>, ApiError> {
    // Verify dialog exists
    let dialog = state
        .dialogs
        .find_by_id(dialog_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Dialog not found".into()))?;

    // Check user is participant (potential participants cannot send messages)
    if !state.participants.exists(dialog_id, sender_id).await? {
        return Err(ApiError::Forbidden(
            "Not a participant. Join the dialog first.".into(),
        ));
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
            let exists = state
                .s3
                .object_exists(&att_input.s3_key)
                .await
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

    // All DB writes in a transaction
    let mut tx = state.db.begin().await?;

    // Create message
    let mut message = Message::new(dialog_id, sender_id, sanitized_content);
    if let Some(reply_to) = req.reply_to {
        message = message.with_reply(reply_to);
    }
    let message = sqlx::query_as::<_, Message>(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, reply_to_id, message_type)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#,
    )
    .bind(message.id)
    .bind(message.dialog_id)
    .bind(message.sender_id)
    .bind(&message.content)
    .bind(message.sent_at)
    .bind(message.reply_to_id)
    .bind(message.message_type.as_str())
    .fetch_one(&mut *tx)
    .await?;

    // Create attachments
    let mut created_attachments = Vec::new();
    for input in &req.attachments {
        let att = domain::Attachment::new(
            message.id,
            &input.filename,
            &input.content_type,
            input.size,
            &input.s3_key,
        );
        let created = sqlx::query_as::<_, domain::Attachment>(
            r#"INSERT INTO attachments (id, message_id, filename, content_type, size, s3_key, width, height, thumbnail_s3_key, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#,
        )
        .bind(att.id)
        .bind(att.message_id)
        .bind(&att.filename)
        .bind(&att.content_type)
        .bind(att.size)
        .bind(&att.s3_key)
        .bind(att.width)
        .bind(att.height)
        .bind(&att.thumbnail_s3_key)
        .bind(att.created_at)
        .fetch_one(&mut *tx)
        .await?;
        created_attachments.push(created);
    }

    // Increment unread count for all participants except the sender
    sqlx::query(
        r#"UPDATE dialog_participants
           SET unread_count = unread_count + 1
           WHERE dialog_id = $1 AND user_id != $2"#,
    )
    .bind(dialog_id)
    .bind(sender_id)
    .execute(&mut *tx)
    .await?;

    // Auto-unarchive: when a new message is sent, unarchive dialog for all participants
    let unarchive_result = sqlx::query(
        r#"UPDATE dialog_participants
           SET is_archived = false
           WHERE dialog_id = $1 AND is_archived = true"#,
    )
    .bind(dialog_id)
    .execute(&mut *tx)
    .await?;
    let unarchived = unarchive_result.rows_affected();

    // Mark sender's own message as read (so divider doesn't appear before own messages)
    sqlx::query(
        r#"UPDATE dialog_participants
           SET unread_count = 0, last_read_message_id = $3
           WHERE dialog_id = $1 AND user_id = $2"#,
    )
    .bind(dialog_id)
    .bind(sender_id)
    .bind(message.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Generate presigned URLs for response (after commit, non-transactional)
    let mut attachment_responses = Vec::new();
    for att in &created_attachments {
        let url = if state.s3.is_configured() {
            state
                .s3
                .generate_download_url(&att.s3_key)
                .await
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

        attachment_responses.push(domain::AttachmentResponse::from_attachment(
            att,
            url,
            thumbnail_url,
        ));
    }

    // Broadcast and webhook after transaction is committed
    if unarchived > 0 {
        tracing::debug!(dialog_id = %dialog_id, count = unarchived, "Auto-unarchived dialog for participants");
        let participant_ids: Vec<Uuid> = state
            .participants
            .list_by_dialog(dialog_id)
            .await?
            .iter()
            .map(|p| p.user_id)
            .collect();
        ws::broadcast_dialog_unarchived(&state.connections, dialog_id, &participant_ids).await;
    }

    ws::broadcast_message(&state.connections, dialog_id, &message).await;
    state
        .webhooks
        .send(WebhookEvent::message_new(&dialog, &message))
        .await;

    // Schedule smart notifications for all participants except sender
    if state.jobs.is_enabled() {
        let participants = state.participants.list_by_dialog(dialog_id).await?;
        for participant in participants {
            if participant.user_id != sender_id {
                let job =
                    NotificationJob::new(dialog_id, participant.user_id, message.id, sender_id);
                if let Err(e) = state.jobs.enqueue_notification(job).await {
                    tracing::warn!(
                        recipient_id = %participant.user_id,
                        error = %e,
                        "Failed to enqueue notification job"
                    );
                }
            }
        }
    }

    Ok(Json(ApiResponse {
        data: MessageWithAttachments {
            message,
            attachments: attachment_responses,
        },
    }))
}

pub async fn get_message(
    State(state): State<AppState>,
    Path((dialog_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<Message>>, ApiError> {
    let message = state
        .messages
        .find_by_id_and_dialog(message_id, dialog_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Message not found".into()))?;

    Ok(Json(ApiResponse { data: message }))
}

pub async fn edit_message(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path((dialog_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<EditMessageRequest>,
) -> Result<Json<ApiResponse<Message>>, ApiError> {
    // Find message
    let message = state
        .messages
        .find_by_id_and_dialog(message_id, dialog_id)
        .await?
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
    state
        .messages
        .save_edit_history(message_id, &message.content)
        .await?;

    // Sanitize and update content
    let sanitized = domain::sanitize_html(&req.content);
    let updated = state
        .messages
        .update_content(message_id, &sanitized)
        .await?
        .ok_or_else(|| ApiError::Internal("Failed to update message".into()))?;

    // Broadcast via WebSocket
    ws::broadcast_message_edited(&state.connections, &updated).await;

    Ok(Json(ApiResponse { data: updated }))
}

pub async fn delete_message(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path((dialog_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    // Find message
    let message = state
        .messages
        .find_by_id_and_dialog(message_id, dialog_id)
        .await?
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
