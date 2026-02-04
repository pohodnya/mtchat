//! WebSocket handling

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
        sender_id: Uuid,
        content: String,
        sent_at: DateTime<Utc>,
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

pub async fn handle_socket(socket: WebSocket, connections: Connections, employee_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Register connection
    {
        let mut conns = connections.write().await;
        conns.insert(employee_id, tx.clone());
    }

    tracing::info!("WebSocket connected: {}", employee_id);

    // Send connected event
    let connected = serde_json::to_string(&WsEvent::Connected { employee_id }).unwrap();
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
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                    match client_msg {
                        WsClientMessage::Ping => {
                            let pong = serde_json::to_string(&WsEvent::Pong).unwrap();
                            let _ = tx.send(pong).await;
                        }
                        WsClientMessage::Subscribe { dialog_id } => {
                            tracing::debug!("Employee {} subscribed to dialog {}", employee_id, dialog_id);
                        }
                        WsClientMessage::Unsubscribe { dialog_id } => {
                            tracing::debug!("Employee {} unsubscribed from dialog {}", employee_id, dialog_id);
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
        conns.remove(&employee_id);
    }

    send_task.abort();
    tracing::info!("WebSocket disconnected: {}", employee_id);
}

// Simple message for broadcast
#[derive(Debug, Serialize)]
struct SimpleMessage {
    id: Uuid,
    dialog_id: Uuid,
    sender_id: Uuid,
    content: String,
    sent_at: DateTime<Utc>,
}

pub async fn broadcast_message(
    connections: &Connections,
    dialog_id: Uuid,
    message: &super::Message,
) {
    let event = WsEvent::MessageNew {
        id: message.id,
        dialog_id: message.dialog_id,
        sender_id: message.sender_id,
        content: message.content.clone(),
        sent_at: message.sent_at,
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
