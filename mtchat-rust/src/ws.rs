//! WebSocket handling
//!
//! Uses DashMap for lock-free concurrent access to connections,
//! improving performance under high connection churn.

use axum::extract::ws::{Message, WebSocket};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use uuid::Uuid;

use crate::repositories::ParticipantRepository;
use crate::services::PresenceService;

pub type ConnectionTx = mpsc::Sender<String>;
/// Concurrent connection map using DashMap for better performance
/// compared to RwLock<HashMap> under high contention.
/// Key is the external user identifier (String); a user may hold several
/// connections at once (one per browser tab), keyed by connection id.
pub type Connections = Arc<DashMap<String, HashMap<Uuid, ConnectionTx>>>;

/// Register a connection. Returns true if it is the user's first one.
fn register_connection(
    connections: &Connections,
    user_id: &str,
    conn_id: Uuid,
    tx: ConnectionTx,
) -> bool {
    let mut user_connections = connections.entry(user_id.to_string()).or_default();
    user_connections.insert(conn_id, tx);
    user_connections.len() == 1
}

/// Unregister a connection. Returns true if it was the user's last one.
fn unregister_connection(connections: &Connections, user_id: &str, conn_id: Uuid) -> bool {
    if let Some(mut user_connections) = connections.get_mut(user_id) {
        user_connections.remove(&conn_id);
    }
    connections
        .remove_if(user_id, |_, user_connections| user_connections.is_empty())
        .is_some()
}

/// Send a serialized event to every connection of a user without waiting:
/// a client whose buffer is full gets the event dropped instead of stalling
/// the broadcast for everyone else.
fn send_to_user(connections: &Connections, user_id: &str, json: &str) {
    let Some(user_connections) = connections.get(user_id) else {
        return;
    };
    for tx in user_connections.values() {
        if let Err(TrySendError::Full(_)) = tx.try_send(json.to_string()) {
            tracing::warn!("WebSocket buffer full, dropping event for user {}", user_id);
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Connected {
        employee_id: String,
    },
    #[serde(rename = "message.new")]
    MessageNew {
        id: Uuid,
        dialog_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        sender_id: Option<String>,
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
        user_id: String,
        last_read_message_id: Uuid,
    },
    #[serde(rename = "participant.joined")]
    ParticipantJoined {
        dialog_id: Uuid,
        user_id: String,
    },
    #[serde(rename = "participant.left")]
    ParticipantLeft {
        dialog_id: Uuid,
        user_id: String,
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
        user_id: String,
        is_online: bool,
    },
    Pong,
    Error {
        message: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Ping,
}

pub async fn handle_socket(
    socket: WebSocket,
    connections: Connections,
    user_id: String,
    presence: Arc<PresenceService>,
    participants: Arc<ParticipantRepository>,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Register connection
    let conn_id = Uuid::new_v4();
    let first_connection = register_connection(&connections, &user_id, conn_id, tx.clone());

    tracing::info!("WebSocket connected: {} ({})", user_id, conn_id);

    // Set user as online
    if let Err(e) = presence.set_online(&user_id).await {
        tracing::warn!("Failed to set user {} online: {}", user_id, e);
    }

    // Broadcast presence update to users in shared dialogs
    if first_connection {
        broadcast_presence(&connections, &participants, &user_id, true).await;
    }

    // Send connected event
    let connected = serde_json::to_string(&WsEvent::Connected {
        employee_id: user_id.clone(),
    })
    .unwrap();
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
    let user_id_for_loop = user_id.clone();
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                    match client_msg {
                        WsClientMessage::Ping => {
                            // Refresh online status TTL
                            if let Err(e) =
                                presence_for_loop.refresh_online(&user_id_for_loop).await
                            {
                                tracing::warn!(
                                    "Failed to refresh user {} online status: {}",
                                    user_id_for_loop,
                                    e
                                );
                            }
                            let pong = serde_json::to_string(&WsEvent::Pong).unwrap();
                            let _ = tx.send(pong).await;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup: the user stays online while other connections remain
    if unregister_connection(&connections, &user_id, conn_id) {
        if let Err(e) = presence.set_offline(&user_id).await {
            tracing::warn!("Failed to set user {} offline: {}", user_id, e);
        }

        broadcast_presence(&connections, &participants, &user_id, false).await;
    }

    send_task.abort();
    tracing::info!("WebSocket disconnected: {} ({})", user_id, conn_id);
}

/// Broadcast presence update to users who share dialogs with the target user
async fn broadcast_presence(
    connections: &Connections,
    participants: &ParticipantRepository,
    user_id: &str,
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
    let recipient_ids = match participants
        .get_dialog_participants_user_ids(&dialog_ids)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            tracing::warn!("Failed to get participants for dialogs: {}", e);
            return;
        }
    };

    // Build event
    let event = WsEvent::PresenceUpdate {
        user_id: user_id.to_string(),
        is_online,
    };
    let json = match serde_json::to_string(&event) {
        Ok(j) => j,
        Err(_) => return,
    };

    // Broadcast to connected recipients (except the user themselves)
    for recipient_id in recipient_ids {
        if recipient_id != user_id {
            send_to_user(connections, &recipient_id, &json);
        }
    }
}

/// Broadcast an event to the current participants of a dialog, plus `extra` users.
///
/// Dialog events must never reach users outside the dialog: the payload
/// carries message content, so a global broadcast would leak it to every
/// connected user regardless of tenant.
async fn broadcast_to_dialog(
    connections: &Connections,
    participants: &ParticipantRepository,
    dialog_id: Uuid,
    event: &WsEvent,
    extra: &[&str],
) {
    let mut user_ids = match participants
        .get_dialog_participants_user_ids(&[dialog_id])
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            tracing::warn!("Failed to get participants for dialog {}: {}", dialog_id, e);
            return;
        }
    };
    for user_id in extra {
        if !user_ids.iter().any(|id| id == user_id) {
            user_ids.push(user_id.to_string());
        }
    }

    broadcast_to_users(connections, event, &user_ids).await;
}

/// Broadcast an event to specific users
async fn broadcast_to_users(connections: &Connections, event: &WsEvent, user_ids: &[String]) {
    let json = match serde_json::to_string(event) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("Failed to serialize WsEvent: {}", e);
            return;
        }
    };

    for user_id in user_ids {
        send_to_user(connections, user_id, &json);
    }
}

pub async fn broadcast_message(
    connections: &Connections,
    participants: &ParticipantRepository,
    message: &crate::domain::Message,
) {
    let event = WsEvent::MessageNew {
        id: message.id,
        dialog_id: message.dialog_id,
        sender_id: message.sender_id.clone(),
        content: message.content.clone(),
        sent_at: message.sent_at,
        message_type: message.message_type.as_str().to_string(),
    };
    broadcast_to_dialog(connections, participants, message.dialog_id, &event, &[]).await;
}

pub async fn broadcast_read(
    connections: &Connections,
    participants: &ParticipantRepository,
    dialog_id: Uuid,
    user_id: &str,
    last_read_message_id: Uuid,
) {
    let event = WsEvent::MessageRead {
        dialog_id,
        user_id: user_id.to_string(),
        last_read_message_id,
    };
    broadcast_to_dialog(connections, participants, dialog_id, &event, &[]).await;
}

pub async fn broadcast_message_edited(
    connections: &Connections,
    participants: &ParticipantRepository,
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
    broadcast_to_dialog(connections, participants, message.dialog_id, &event, &[]).await;
}

pub async fn broadcast_message_deleted(
    connections: &Connections,
    participants: &ParticipantRepository,
    dialog_id: Uuid,
    message_id: Uuid,
) {
    let event = WsEvent::MessageDeleted {
        id: message_id,
        dialog_id,
    };
    broadcast_to_dialog(connections, participants, dialog_id, &event, &[]).await;
}

pub async fn broadcast_participant_joined(
    connections: &Connections,
    participants: &ParticipantRepository,
    dialog_id: Uuid,
    user_id: &str,
) {
    let event = WsEvent::ParticipantJoined {
        dialog_id,
        user_id: user_id.to_string(),
    };
    broadcast_to_dialog(connections, participants, dialog_id, &event, &[]).await;
}

/// The departed user is already removed from the dialog, so they are added
/// explicitly to learn that they left.
pub async fn broadcast_participant_left(
    connections: &Connections,
    participants: &ParticipantRepository,
    dialog_id: Uuid,
    user_id: &str,
) {
    let event = WsEvent::ParticipantLeft {
        dialog_id,
        user_id: user_id.to_string(),
    };
    broadcast_to_dialog(connections, participants, dialog_id, &event, &[user_id]).await;
}

/// Broadcast dialog archived event to specific users.
pub async fn broadcast_dialog_archived(
    connections: &Connections,
    dialog_id: Uuid,
    user_ids: &[String],
) {
    let event = WsEvent::DialogArchived { dialog_id };
    broadcast_to_users(connections, &event, user_ids).await;
}

/// Broadcast dialog unarchived event to specific users.
pub async fn broadcast_dialog_unarchived(
    connections: &Connections,
    dialog_id: Uuid,
    user_ids: &[String],
) {
    tracing::debug!(dialog_id = %dialog_id, user_ids = ?user_ids, "Broadcasting dialog.unarchived");
    let event = WsEvent::DialogUnarchived { dialog_id };
    broadcast_to_users(connections, &event, user_ids).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connect(connections: &Connections, user_id: &str) -> (Uuid, mpsc::Receiver<String>) {
        let conn_id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel::<String>(10);
        register_connection(connections, user_id, conn_id, tx);
        (conn_id, rx)
    }

    fn deleted_event() -> WsEvent {
        WsEvent::MessageDeleted {
            id: Uuid::new_v4(),
            dialog_id: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn broadcast_to_users_skips_unlisted_connections() {
        let connections: Connections = Arc::new(DashMap::new());
        let (_, mut member_rx) = connect(&connections, "member");
        let (_, mut outsider_rx) = connect(&connections, "outsider");

        broadcast_to_users(&connections, &deleted_event(), &["member".to_string()]).await;

        assert!(member_rx.try_recv().unwrap().contains("message.deleted"));
        assert!(outsider_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn broadcast_to_users_reaches_every_connection_of_a_user() {
        let connections: Connections = Arc::new(DashMap::new());
        let (_, mut first_tab) = connect(&connections, "user");
        let (_, mut second_tab) = connect(&connections, "user");

        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;

        assert!(first_tab.try_recv().is_ok());
        assert!(second_tab.try_recv().is_ok());
    }

    #[tokio::test]
    async fn closing_one_tab_keeps_the_other_connected() {
        let connections: Connections = Arc::new(DashMap::new());
        let (first_tab_id, _first_tab) = connect(&connections, "user");
        let (_, mut second_tab) = connect(&connections, "user");

        assert!(!unregister_connection(&connections, "user", first_tab_id));

        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        assert!(second_tab.try_recv().is_ok());
    }

    #[test]
    fn first_and_last_connection_are_reported() {
        let connections: Connections = Arc::new(DashMap::new());
        let (tx, _rx) = mpsc::channel::<String>(10);
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();

        assert!(register_connection(&connections, "user", first, tx.clone()));
        assert!(!register_connection(&connections, "user", second, tx));
        assert!(!unregister_connection(&connections, "user", first));
        assert!(unregister_connection(&connections, "user", second));
        assert!(!connections.contains_key("user"));
    }

    #[tokio::test]
    async fn full_buffer_does_not_block_other_users() {
        let connections: Connections = Arc::new(DashMap::new());
        let (slow_tx, _slow_rx) = mpsc::channel::<String>(1);
        register_connection(&connections, "slow", Uuid::new_v4(), slow_tx);
        let (_, mut fast_rx) = connect(&connections, "fast");
        let recipients = ["slow".to_string(), "fast".to_string()];

        broadcast_to_users(&connections, &deleted_event(), &recipients).await;
        broadcast_to_users(&connections, &deleted_event(), &recipients).await;

        assert!(fast_rx.try_recv().is_ok());
        assert!(fast_rx.try_recv().is_ok());
    }
}
