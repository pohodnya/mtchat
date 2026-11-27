//! WebSocket handling
//!
//! Uses DashMap for lock-free concurrent access to connections,
//! improving performance under high connection churn.
//! Events reach connections on other API replicas through Redis pub/sub,
//! see [`Connections`].

use axum::extract::ws::{Message, WebSocket};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use fred::clients::{Pool, SubscriberClient};
use fred::interfaces::{ClientLike, EventInterface, PubsubInterface};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::repositories::ParticipantRepository;
use crate::services::PresenceService;

pub type ConnectionTx = mpsc::Sender<String>;

/// Redis pub/sub channel that carries WebSocket events between API replicas.
pub const EVENTS_CHANNEL: &str = "mtchat:ws:events";

const PUBLISH_TIMEOUT: Duration = Duration::from_secs(2);

/// Events waiting to be published to the other replicas; when Redis stalls
/// and the queue fills up, newer events are dropped.
const PUBLISH_QUEUE_CAPACITY: usize = 1024;

/// How many users get their presence released at once on shutdown.
const RELEASE_CONCURRENCY: usize = 32;

enum Outgoing {
    Event(String),
    Flush(oneshot::Sender<()>),
}

/// WebSocket connections of this process plus, when Redis is configured,
/// the publisher that forwards events to the other API replicas.
///
/// Uses DashMap for better performance compared to RwLock<HashMap> under
/// high contention. Key is the external user identifier (String); a user may
/// hold several connections at once (one per browser tab), keyed by
/// connection id.
#[derive(Clone)]
pub struct Connections {
    local: Arc<DashMap<String, HashMap<Uuid, ConnectionTx>>>,
    /// Identifies this process in the events it publishes.
    pod_id: Uuid,
    /// Queue of the background task that publishes events in order.
    publisher: Option<mpsc::Sender<Outgoing>>,
    dropped: Arc<AtomicU64>,
    /// Set on shutdown, once the presence of local connections is released.
    closing: Arc<AtomicBool>,
}

impl Default for Connections {
    fn default() -> Self {
        Self {
            local: Arc::new(DashMap::new()),
            pod_id: Uuid::new_v4(),
            publisher: None,
            dropped: Arc::new(AtomicU64::new(0)),
            closing: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Event envelope published to [`EVENTS_CHANNEL`].
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BusMessage {
    pub origin: Uuid,
    pub user_ids: Vec<String>,
    pub event_json: String,
}

impl Connections {
    /// Connections that also publish every event to the other replicas.
    pub fn with_redis(publisher: Arc<Pool>) -> Self {
        Self::with_publisher(PUBLISH_QUEUE_CAPACITY, move |payload| {
            let publisher = publisher.clone();
            async move {
                publisher
                    .next()
                    .publish::<(), _, _>(EVENTS_CHANNEL, payload)
                    .await
            }
        })
    }

    /// Connections whose events are published by a single background task,
    /// so request handlers never wait for Redis and events keep their order.
    fn with_publisher<F, Fut>(capacity: usize, publish: F) -> Self
    where
        F: Fn(String) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), fred::error::Error>> + Send + 'static,
    {
        let (queue, outgoing) = mpsc::channel(capacity);
        tokio::spawn(run_publisher(outgoing, publish));
        Self {
            publisher: Some(queue),
            ..Self::default()
        }
    }

    /// Deliver an event to the recipients' connections on every replica.
    ///
    /// Local connections get the event directly and the replica skips its own
    /// messages from the subscription, so local delivery does not depend on
    /// Redis and nobody receives the event twice.
    fn deliver(&self, user_ids: &[String], json: String) {
        for user_id in user_ids {
            send_to_user(self, user_id, &json);
        }

        let Some(queue) = &self.publisher else {
            return;
        };
        if user_ids.is_empty() {
            return;
        }
        let payload = match serde_json::to_string(&BusMessage {
            origin: self.pod_id,
            user_ids: user_ids.to_vec(),
            event_json: json,
        }) {
            Ok(payload) => payload,
            Err(e) => {
                tracing::warn!("Failed to serialize WebSocket bus message: {}", e);
                return;
            }
        };
        if let Err(TrySendError::Full(_)) = queue.try_send(Outgoing::Event(payload)) {
            let dropped = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
            tracing::warn!(
                "WebSocket bus queue is full, other replicas will miss the event ({} dropped so far)",
                dropped
            );
        }
    }

    /// Wait until the events queued so far are published, at most `timeout`.
    pub async fn flush_publisher(&self, timeout: Duration) {
        let Some(queue) = &self.publisher else {
            return;
        };
        let (done, flushed) = oneshot::channel();
        let flush = async {
            if queue.send(Outgoing::Flush(done)).await.is_ok() {
                let _ = flushed.await;
            }
        };
        if tokio::time::timeout(timeout, flush).await.is_err() {
            tracing::warn!("Timed out publishing queued WebSocket events on shutdown");
        }
    }

    fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    /// Deliver an event received from the bus to local connections.
    /// Returns false for messages that are malformed or published by this replica.
    pub fn deliver_from_bus(&self, payload: &str) -> bool {
        let message = match serde_json::from_str::<BusMessage>(payload) {
            Ok(message) => message,
            Err(e) => {
                tracing::warn!("Malformed WebSocket bus message: {}", e);
                return false;
            }
        };
        if message.origin == self.pod_id {
            return false;
        }
        for user_id in &message.user_ids {
            send_to_user(self, user_id, &message.event_json);
        }
        true
    }

    /// Subscribe to [`EVENTS_CHANNEL`] and deliver events from other replicas
    /// to local connections. The subscriber re-subscribes after reconnecting,
    /// as long as it was built with a reconnect policy.
    pub async fn start_subscriber(
        &self,
        subscriber: SubscriberClient,
    ) -> Result<(), fred::error::Error> {
        let mut messages = subscriber.message_rx();
        subscriber.init().await?;
        subscriber.on_error(|(error, server)| async move {
            tracing::warn!(
                "WebSocket bus subscriber connection error ({:?}): {}",
                server,
                error
            );
            Ok(())
        });
        subscriber.on_reconnect(|server| async move {
            tracing::info!("WebSocket bus subscriber reconnected to {}", server);
            Ok(())
        });
        subscriber.manage_subscriptions();
        subscriber.subscribe(EVENTS_CHANNEL).await?;

        let connections = self.clone();
        tokio::spawn(async move {
            // Keeps the client alive for as long as the loop runs.
            let _subscriber = subscriber;
            loop {
                match messages.recv().await {
                    Ok(message) => match message.value.as_str() {
                        Some(payload) => {
                            connections.deliver_from_bus(&payload);
                        }
                        None => tracing::warn!("Non-string WebSocket bus message ignored"),
                    },
                    Err(RecvError::Lagged(skipped)) => tracing::warn!(
                        "WebSocket bus subscriber lagged, {} events dropped",
                        skipped
                    ),
                    Err(RecvError::Closed) => {
                        tracing::error!("WebSocket bus subscriber stopped");
                        break;
                    }
                }
            }
        });
        Ok(())
    }
}

async fn run_publisher<F, Fut>(mut outgoing: mpsc::Receiver<Outgoing>, publish: F)
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<(), fred::error::Error>>,
{
    while let Some(item) = outgoing.recv().await {
        match item {
            Outgoing::Event(payload) => {
                match tokio::time::timeout(PUBLISH_TIMEOUT, publish(payload)).await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => tracing::warn!(
                        "Failed to publish WebSocket event, other replicas will miss it: {}",
                        e
                    ),
                    Err(_) => tracing::warn!(
                        "Publishing WebSocket event timed out, other replicas may miss it"
                    ),
                }
            }
            Outgoing::Flush(done) => {
                let _ = done.send(());
            }
        }
    }
}

/// Release the presence of every local connection before the replica stops,
/// sending offline updates for users left with no connection elsewhere.
///
/// A stopped replica never reaches the cleanup in [`handle_socket`], so its
/// connections would keep the users online until they expire, and closing
/// the last tab on another replica would not send the offline update.
pub async fn release_presence(
    connections: &Connections,
    presence: &PresenceService,
    participants: &ParticipantRepository,
) {
    connections.closing.store(true, Ordering::SeqCst);
    let local: Vec<(String, Vec<Uuid>)> = connections
        .local
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().keys().copied().collect()))
        .collect();

    futures::stream::iter(local)
        .for_each_concurrent(RELEASE_CONCURRENCY, |(user_id, conn_ids)| async move {
            let mut last_connection = false;
            for conn_id in conn_ids {
                match presence.disconnect(&user_id, conn_id).await {
                    Ok(last) => last_connection = last.unwrap_or(false),
                    Err(e) => tracing::warn!("Failed to set user {} offline: {}", user_id, e),
                }
            }
            if last_connection {
                broadcast_presence(connections, participants, &user_id, false).await;
            }
        })
        .await;
}

/// Register a local connection. Returns true if it is the user's first one on this replica.
pub fn register_connection(
    connections: &Connections,
    user_id: &str,
    conn_id: Uuid,
    tx: ConnectionTx,
) -> bool {
    let mut user_connections = connections.local.entry(user_id.to_string()).or_default();
    user_connections.insert(conn_id, tx);
    user_connections.len() == 1
}

/// Unregister a local connection. Returns true if it was the user's last one on this replica.
fn unregister_connection(connections: &Connections, user_id: &str, conn_id: Uuid) -> bool {
    if let Some(mut user_connections) = connections.local.get_mut(user_id) {
        user_connections.remove(&conn_id);
    }
    connections
        .local
        .remove_if(user_id, |_, user_connections| user_connections.is_empty())
        .is_some()
}

/// Send a serialized event to every local connection of a user without waiting:
/// a client whose buffer is full gets the event dropped instead of stalling
/// the broadcast for everyone else.
fn send_to_user(connections: &Connections, user_id: &str, json: &str) {
    let Some(user_connections) = connections.local.get(user_id) else {
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
    let first_local = register_connection(&connections, &user_id, conn_id, tx.clone());

    tracing::info!("WebSocket connected: {} ({})", user_id, conn_id);

    let first_connection = if connections.is_closing() {
        false
    } else {
        match presence.connect(&user_id, conn_id).await {
            Ok(first) => first.unwrap_or(first_local),
            Err(e) => {
                tracing::warn!("Failed to set user {} online: {}", user_id, e);
                first_local
            }
        }
    };

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
                            // Refresh online status TTL; a throttled background
                            // tab may have let it expire, making the user offline.
                            // A closing replica has already released its presence.
                            let refreshed = if connections.is_closing() {
                                Ok(None)
                            } else {
                                presence_for_loop.refresh(&user_id_for_loop, conn_id).await
                            };
                            match refreshed {
                                Ok(Some(true)) => {
                                    broadcast_presence(
                                        &connections,
                                        &participants,
                                        &user_id_for_loop,
                                        true,
                                    )
                                    .await;
                                }
                                Ok(_) => {}
                                Err(e) => tracing::warn!(
                                    "Failed to refresh user {} online status: {}",
                                    user_id_for_loop,
                                    e
                                ),
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

    // Cleanup: the user stays online while other connections remain on any replica
    let last_local = unregister_connection(&connections, &user_id, conn_id);
    let last_connection = if connections.is_closing() {
        false
    } else {
        match presence.disconnect(&user_id, conn_id).await {
            Ok(last) => last.unwrap_or(last_local),
            Err(e) => {
                tracing::warn!("Failed to set user {} offline: {}", user_id, e);
                last_local
            }
        }
    };
    if last_connection {
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
    // Broadcast to connected recipients (except the user themselves)
    let recipient_ids: Vec<String> = recipient_ids
        .into_iter()
        .filter(|recipient_id| recipient_id != user_id)
        .collect();
    broadcast_to_users(connections, &event, &recipient_ids).await;
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

    connections.deliver(user_ids, json);
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
        let connections = Connections::default();
        let (_, mut member_rx) = connect(&connections, "member");
        let (_, mut outsider_rx) = connect(&connections, "outsider");

        broadcast_to_users(&connections, &deleted_event(), &["member".to_string()]).await;

        assert!(member_rx.try_recv().unwrap().contains("message.deleted"));
        assert!(outsider_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn broadcast_to_users_reaches_every_connection_of_a_user() {
        let connections = Connections::default();
        let (_, mut first_tab) = connect(&connections, "user");
        let (_, mut second_tab) = connect(&connections, "user");

        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;

        assert!(first_tab.try_recv().is_ok());
        assert!(second_tab.try_recv().is_ok());
    }

    #[tokio::test]
    async fn closing_one_tab_keeps_the_other_connected() {
        let connections = Connections::default();
        let (first_tab_id, _first_tab) = connect(&connections, "user");
        let (_, mut second_tab) = connect(&connections, "user");

        assert!(!unregister_connection(&connections, "user", first_tab_id));

        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        assert!(second_tab.try_recv().is_ok());
    }

    #[test]
    fn first_and_last_connection_are_reported() {
        let connections = Connections::default();
        let (tx, _rx) = mpsc::channel::<String>(10);
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();

        assert!(register_connection(&connections, "user", first, tx.clone()));
        assert!(!register_connection(&connections, "user", second, tx));
        assert!(!unregister_connection(&connections, "user", first));
        assert!(unregister_connection(&connections, "user", second));
        assert!(!connections.local.contains_key("user"));
    }

    #[tokio::test]
    async fn full_buffer_does_not_block_other_users() {
        let connections = Connections::default();
        let (slow_tx, _slow_rx) = mpsc::channel::<String>(1);
        register_connection(&connections, "slow", Uuid::new_v4(), slow_tx);
        let (_, mut fast_rx) = connect(&connections, "fast");
        let recipients = ["slow".to_string(), "fast".to_string()];

        broadcast_to_users(&connections, &deleted_event(), &recipients).await;
        broadcast_to_users(&connections, &deleted_event(), &recipients).await;

        assert!(fast_rx.try_recv().is_ok());
        assert!(fast_rx.try_recv().is_ok());
    }

    fn bus_payload(origin: Uuid, user_ids: &[&str]) -> String {
        serde_json::to_string(&BusMessage {
            origin,
            user_ids: user_ids.iter().map(|id| id.to_string()).collect(),
            event_json: serde_json::to_string(&deleted_event()).unwrap(),
        })
        .unwrap()
    }

    #[test]
    fn bus_message_round_trips_through_json() {
        let message = BusMessage {
            origin: Uuid::new_v4(),
            user_ids: vec!["user-1".to_string(), "user-2".to_string()],
            event_json: serde_json::to_string(&deleted_event()).unwrap(),
        };

        let payload = serde_json::to_string(&message).unwrap();

        assert_eq!(
            serde_json::from_str::<BusMessage>(&payload).unwrap(),
            message
        );
    }

    #[tokio::test]
    async fn bus_message_reaches_only_listed_local_users() {
        let connections = Connections::default();
        let (_, mut member_rx) = connect(&connections, "member");
        let (_, mut outsider_rx) = connect(&connections, "outsider");

        assert!(connections.deliver_from_bus(&bus_payload(Uuid::new_v4(), &["member"])));

        let event = member_rx.try_recv().unwrap();
        assert!(event.contains("message.deleted"));
        assert!(member_rx.try_recv().is_err());
        assert!(outsider_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn own_bus_message_is_not_delivered_twice() {
        let connections = Connections::default();
        let (_, mut rx) = connect(&connections, "user");

        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        let own_echo = bus_payload(connections.pod_id, &["user"]);

        assert!(!connections.deliver_from_bus(&own_echo));
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn malformed_bus_message_is_ignored() {
        let connections = Connections::default();

        assert!(!connections.deliver_from_bus("not json"));
    }

    fn published_dialog_ids(published: &[String]) -> Vec<String> {
        published
            .iter()
            .map(|payload| {
                let message: BusMessage = serde_json::from_str(payload).unwrap();
                let event: serde_json::Value = serde_json::from_str(&message.event_json).unwrap();
                event["dialog_id"].as_str().unwrap().to_string()
            })
            .collect()
    }

    #[tokio::test]
    async fn queued_events_are_published_in_order() {
        let published = Arc::new(std::sync::Mutex::new(Vec::new()));
        let remaining = Arc::new(AtomicU64::new(10));
        let sink = published.clone();
        // Earlier events take longer to publish, so concurrent publishing would reorder them.
        let connections = Connections::with_publisher(16, move |payload| {
            let sink = sink.clone();
            let delay = remaining.fetch_sub(1, Ordering::SeqCst);
            async move {
                tokio::time::sleep(Duration::from_millis(delay * 5)).await;
                sink.lock().unwrap().push(payload);
                Ok(())
            }
        });
        let dialog_ids: Vec<String> = (0..10).map(|_| Uuid::new_v4().to_string()).collect();

        for dialog_id in &dialog_ids {
            let event = WsEvent::DialogArchived {
                dialog_id: dialog_id.parse().unwrap(),
            };
            broadcast_to_users(&connections, &event, &["user".to_string()]).await;
        }
        connections.flush_publisher(Duration::from_secs(5)).await;

        assert_eq!(published_dialog_ids(&published.lock().unwrap()), dialog_ids);
    }

    #[tokio::test]
    async fn full_publish_queue_drops_events_but_delivers_locally() {
        let connections =
            Connections::with_publisher(1, |_| futures::future::pending::<Result<(), _>>());
        let (_, mut rx) = connect(&connections, "user");

        for _ in 0..3 {
            broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        }

        assert_eq!(connections.dropped.load(Ordering::Relaxed), 2);
        for _ in 0..3 {
            assert!(rx.try_recv().is_ok());
        }
    }
}
