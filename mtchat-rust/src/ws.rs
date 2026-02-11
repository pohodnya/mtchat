//! WebSocket handling

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::repositories::ParticipantRepository;
use crate::services::PresenceService;

pub type ConnectionTx = mpsc::Sender<String>;
pub type Connections = Arc<RwLock<HashMap<Uuid, ConnectionTx>>>;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Connected { employee_id: Uuid },
    #[serde(rename = "message.new")]
    MessageNew {
        id: Uuid,
        dialog_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        sender_id: Option<Uuid>,
        content: String,
        sent_at: DateTime<Utc>,
        message_type: String,
    },
    #[serde(rename = "message.edited")]
    MessageEdited {
        id: Uuid,
        dialog_id: Uuid,
        content: String,
        last_edited_at: DateTime<Utc>,
    },
    #[serde(rename = "message.deleted")]
    MessageDeleted {
        id: Uuid,
        dialog_id: Uuid,
    },
    #[serde(rename = "message.read")]
    MessageRead {
        dialog_id: Uuid,
        user_id: Uuid,
        last_read_message_id: Uuid,
    },
    #[serde(rename = "participant.joined")]
    ParticipantJoined {
        dialog_id: Uuid,
        user_id: Uuid,
    },
    #[serde(rename = "participant.left")]
    ParticipantLeft {
        dialog_id: Uuid,
        user_id: Uuid,
    },
    #[serde(rename = "dialog.archived")]
    DialogArchived {
        dialog_id: Uuid,
    },
    #[serde(rename = "dialog.unarchived")]
    DialogUnarchived {
        dialog_id: Uuid,
    },
    #[serde(rename = "presence.update")]
    PresenceUpdate {
        user_id: Uuid,
        is_online: bool,
    },
    Pong,
    Error { message: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Subscribe { dialog_id: Uuid },
    Unsubscribe { dialog_id: Uuid },
    Ping,
}

pub async fn handle_socket(
    socket: WebSocket,
    connections: Connections,
    user_id: Uuid,
    presence: Arc<PresenceService>,
    participants: Arc<ParticipantRepository>,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Register connection
    {
        let mut conns = connections.write().await;
        conns.insert(user_id, tx.clone());
    }

    tracing::info!("WebSocket connected: {}", user_id);

    // Set user as online
    if let Err(e) = presence.set_online(user_id).await {
        tracing::warn!("Failed to set user {} online: {}", user_id, e);
    }

    // Broadcast presence update to users in shared dialogs
    broadcast_presence(&connections, &participants, user_id, true).await;

    // Send connected event
    let connected = serde_json::to_string(&WsEvent::Connected { employee_id: user_id }).unwrap();
    let _ = sender.send(Message::Text(connected.into())).await;

    // Spawn task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let presence_for_loop = presence.clone();
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                    match client_msg {
                        WsClientMessage::Ping => {
                            // Refresh online status TTL
                            if let Err(e) = presence_for_loop.refresh_online(user_id).await {
                                tracing::warn!("Failed to refresh user {} online status: {}", user_id, e);
                            }
                            let pong = serde_json::to_string(&WsEvent::Pong).unwrap();
                            let _ = tx.send(pong).await;
                        }
                        WsClientMessage::Subscribe { dialog_id } => {
                            tracing::debug!("User {} subscribed to dialog {}", user_id, dialog_id);
                        }
                        WsClientMessage::Unsubscribe { dialog_id } => {
                            tracing::debug!("User {} unsubscribed from dialog {}", user_id, dialog_id);
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup
    {
        let mut conns = connections.write().await;
        conns.remove(&user_id);
    }

    // Set user as offline
    if let Err(e) = presence.set_offline(user_id).await {
        tracing::warn!("Failed to set user {} offline: {}", user_id, e);
    }

    // Broadcast presence update
    broadcast_presence(&connections, &participants, user_id, false).await;

    send_task.abort();
    tracing::info!("WebSocket disconnected: {}", user_id);
}

/// Broadcast presence update to users who share dialogs with the target user
async fn broadcast_presence(
    connections: &Connections,
    participants: &ParticipantRepository,
    user_id: Uuid,
    is_online: bool,
) {
    // Get all dialogs this user participates in
    let dialog_ids = match participants.get_user_dialogs(user_id).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::warn!("Failed to get dialogs for user {}: {}", user_id, e);
            return;
        }
    };

    if dialog_ids.is_empty() {
        return;
    }

    // Get all users who participate in those dialogs
    let recipient_ids = match participants.get_dialog_participants_user_ids(&dialog_ids).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::warn!("Failed to get participants for dialogs: {}", e);
            return;
        }
    };

    // Build event
    let event = WsEvent::PresenceUpdate { user_id, is_online };
    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    // Broadcast to connected recipients (except the user themselves)
    let conns = connections.read().await;
    for recipient_id in recipient_ids {
        if recipient_id != user_id {
            if let Some(tx) = conns.get(&recipient_id) {
                let _ = tx.send(json.clone()).await;
            }
        }
    }
}

pub async fn broadcast_message(
    connections: &Connections,
    dialog_id: Uuid,
    message: &crate::domain::Message,
) {
    let event = WsEvent::MessageNew {
        id: message.id,
        dialog_id: message.dialog_id,
        sender_id: message.sender_id,
        content: message.content.clone(),
        sent_at: message.sent_at,
        message_type: message.message_type.as_str().to_string(),
    };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    let conns = connections.read().await;
    for (_, tx) in conns.iter() {
        let _ = tx.send(json.clone()).await;
    }
}

pub async fn broadcast_read(
    connections: &Connections,
    dialog_id: Uuid,
    user_id: Uuid,
    last_read_message_id: Uuid,
) {
    let event = WsEvent::MessageRead {
        dialog_id,
        user_id,
        last_read_message_id,
    };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    let conns = connections.read().await;
    for (_, tx) in conns.iter() {
        let _ = tx.send(json.clone()).await;
    }
}

pub async fn broadcast_message_edited(
    connections: &Connections,
    message: &crate::domain::Message,
) {
    let last_edited_at = match message.last_edited_at {
        Some(ts) => ts,
        None => return,
    };

    let event = WsEvent::MessageEdited {
        id: message.id,
        dialog_id: message.dialog_id,
        content: message.content.clone(),
        last_edited_at,
    };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    let conns = connections.read().await;
    for (_, tx) in conns.iter() {
        let _ = tx.send(json.clone()).await;
    }
}

pub async fn broadcast_message_deleted(
    connections: &Connections,
    dialog_id: Uuid,
    message_id: Uuid,
) {
    let event = WsEvent::MessageDeleted {
        id: message_id,
        dialog_id,
    };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    let conns = connections.read().await;
    for (_, tx) in conns.iter() {
        let _ = tx.send(json.clone()).await;
    }
}

pub async fn broadcast_participant_joined(
    connections: &Connections,
    dialog_id: Uuid,
    user_id: Uuid,
) {
    let event = WsEvent::ParticipantJoined {
        dialog_id,
        user_id,
    };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    let conns = connections.read().await;
    for (_, tx) in conns.iter() {
        let _ = tx.send(json.clone()).await;
    }
}

pub async fn broadcast_participant_left(
    connections: &Connections,
    dialog_id: Uuid,
    user_id: Uuid,
) {
    let event = WsEvent::ParticipantLeft {
        dialog_id,
        user_id,
    };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    let conns = connections.read().await;
    for (_, tx) in conns.iter() {
        let _ = tx.send(json.clone()).await;
    }
}

/// Broadcast dialog archived event to specific users.
pub async fn broadcast_dialog_archived(
    connections: &Connections,
    dialog_id: Uuid,
    user_ids: &[Uuid],
) {
    let event = WsEvent::DialogArchived { dialog_id };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    let conns = connections.read().await;
    for user_id in user_ids {
        if let Some(tx) = conns.get(user_id) {
            let _ = tx.send(json.clone()).await;
        }
    }
}

/// Broadcast dialog unarchived event to specific users.
pub async fn broadcast_dialog_unarchived(
    connections: &Connections,
    dialog_id: Uuid,
    user_ids: &[Uuid],
) {
    let event = WsEvent::DialogUnarchived { dialog_id };

    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    tracing::debug!(
        dialog_id = %dialog_id,
        user_ids = ?user_ids,
        event_json = %json,
        "Broadcasting dialog.unarchived event"
    );

    let conns = connections.read().await;
    let mut sent_count = 0;
    for user_id in user_ids {
        if let Some(tx) = conns.get(user_id) {
            if tx.send(json.clone()).await.is_ok() {
                sent_count += 1;
            }
        }
    }
    tracing::debug!(
        dialog_id = %dialog_id,
        sent_count = sent_count,
        total_recipients = user_ids.len(),
        "Sent dialog.unarchived event"
    );
}
