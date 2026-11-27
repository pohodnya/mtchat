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
use fred::types::config::Options;
use futures::{FutureExt, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::repositories::ParticipantRepository;
use crate::services::{PresenceService, PRESENCE_SCRIPT_TIMEOUT};

pub type ConnectionTx = mpsc::Sender<String>;

/// Redis pub/sub channel that carries WebSocket events between API replicas.
pub const EVENTS_CHANNEL: &str = "mtchat:ws:events";

const PUBLISH_TIMEOUT: Duration = Duration::from_secs(2);

/// How often a batch skipped because Redis is not connected is logged.
const NOT_CONNECTED_WARN_INTERVAL: Duration = Duration::from_secs(10);

/// Events waiting to be published to the other replicas; when Redis stalls
/// and the queue fills up, newer events are dropped. Reconnect storms after a
/// replica restart produce a presence update per dialog participant.
const PUBLISH_QUEUE_CAPACITY: usize = 8192;

/// How many queued events are sent to Redis in one pipeline.
const PUBLISH_BATCH_LIMIT: usize = 256;

/// How often the subscriber re-sends SUBSCRIBE in case re-subscribing after a
/// reconnect failed; fred retries it only on the next reconnect otherwise.
const RESUBSCRIBE_INTERVAL: Duration = Duration::from_secs(30);

/// How many users get their presence released at once on shutdown.
const RELEASE_CONCURRENCY: usize = 32;

/// On shutdown, presence scripts that passed the `closing` check just before
/// it was set may still run on other pooled connections; their ZADD must not
/// land after the release, or the connection stays in the set until it expires.
const IN_FLIGHT_SCRIPTS_WAIT: Duration = Duration::from_secs(4);
const _: () = assert!(IN_FLIGHT_SCRIPTS_WAIT.as_millis() > PRESENCE_SCRIPT_TIMEOUT.as_millis());

enum Outgoing {
    Event(String),
    Flush(oneshot::Sender<()>),
}

enum PublishError {
    /// No pooled Redis connection is up, nothing was sent.
    NotConnected,
    /// Publishing stopped at the first failure, the rest of the batch was not sent.
    Failed {
        unpublished: usize,
        error: fred::error::Error,
    },
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
    /// Set once the publisher task is gone, so that is logged only once.
    publisher_closed: Arc<AtomicBool>,
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
            publisher_closed: Arc::new(AtomicBool::new(false)),
            closing: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Which replicas an event is delivered on.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Reach {
    AllReplicas,
    /// Used when Redis failed and other replicas may know better, e.g. that
    /// the user still has a tab connected to them.
    ThisReplica,
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
        Self::with_publisher(PUBLISH_QUEUE_CAPACITY, move |payloads| {
            let publisher = publisher.clone();
            async move { publish_in_order(&publisher, payloads).await }
        })
    }

    /// Connections whose events are published by a single background task,
    /// so request handlers never wait for Redis and events keep their order.
    fn with_publisher<F, Fut>(capacity: usize, publish: F) -> Self
    where
        F: Fn(Vec<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), PublishError>> + Send + 'static,
    {
        let (queue, outgoing) = mpsc::channel(capacity);
        let connections = Self {
            publisher: Some(queue),
            ..Self::default()
        };
        tokio::spawn(run_publisher(
            outgoing,
            publish,
            connections.dropped.clone(),
        ));
        connections
    }

    /// Deliver an event to the recipients' connections on every replica.
    ///
    /// Local connections get the event directly and the replica skips its own
    /// messages from the subscription, so local delivery does not depend on
    /// Redis and nobody receives the event twice.
    fn deliver(&self, user_ids: &[String], json: String, reach: Reach) {
        for user_id in user_ids {
            send_to_user(self, user_id, &json);
        }

        let Some(queue) = &self.publisher else {
            return;
        };
        if user_ids.is_empty() || reach == Reach::ThisReplica {
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
        match queue.try_send(Outgoing::Event(payload)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let dropped = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
                tracing::warn!(
                    "WebSocket bus queue is full, other replicas will miss the event ({} dropped so far)",
                    dropped
                );
            }
            Err(TrySendError::Closed(_)) => {
                let dropped = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
                if !self.publisher_closed.swap(true, Ordering::Relaxed) {
                    tracing::error!(
                        "WebSocket bus publisher stopped, other replicas no longer get events ({} dropped so far)",
                        dropped
                    );
                }
            }
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
            let mut resubscribe = tokio::time::interval(RESUBSCRIBE_INTERVAL);
            resubscribe.tick().await;
            loop {
                tokio::select! {
                    received = messages.recv() => match received {
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
                    },
                    // fred's tracked channels are what the client intends to
                    // be subscribed to, not what the server has, and a
                    // subscriber connection cannot ask the server. SUBSCRIBE to
                    // a channel the connection is already subscribed to is a
                    // no-op in Redis, so it is simply sent again.
                    _ = resubscribe.tick() => {
                        if subscriber.is_connected() {
                            if let Err(e) = subscriber.subscribe(EVENTS_CHANNEL).await {
                                tracing::warn!("Failed to re-subscribe to WebSocket bus: {}", e);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

/// Publishes the events one by one on a single connected client, so they keep
/// their order. Each publish is bounded by fred's own timeout, not an outer one:
/// fred marks a command that timed out and never writes it after a reconnect
/// (`basic_request_response` in fred's `utils.rs`, checked in `write_command`
/// and `retry_buffer`), while a dropped future would leave it buffered, to be
/// published late and out of order. A pipeline is not used for the same reason:
/// fred does not mark pipelined commands on timeout. `fail_fast` makes a command
/// fail instead of waiting in the buffer when the connection drops.
async fn publish_in_order(pool: &Pool, payloads: Vec<String>) -> Result<(), PublishError> {
    let client = pool.next_connected();
    if !client.is_connected() {
        return Err(PublishError::NotConnected);
    }
    let client = client.with_options(&Options {
        timeout: Some(PUBLISH_TIMEOUT),
        fail_fast: true,
        ..Default::default()
    });
    let count = payloads.len();
    for (published, payload) in payloads.into_iter().enumerate() {
        if let Err(error) = client.publish::<(), _, _>(EVENTS_CHANNEL, payload).await {
            return Err(PublishError::Failed {
                unpublished: count - published,
                error,
            });
        }
    }
    Ok(())
}

/// Publishes queued events in order, everything queued so far in one batch.
async fn run_publisher<F, Fut>(
    mut outgoing: mpsc::Receiver<Outgoing>,
    publish: F,
    dropped: Arc<AtomicU64>,
) where
    F: Fn(Vec<String>) -> Fut,
    Fut: Future<Output = Result<(), PublishError>>,
{
    let mut not_connected_warned_at: Option<Instant> = None;
    while let Some(item) = outgoing.recv().await {
        let mut batch = Vec::new();
        let mut flushed = None;
        let mut next = Some(item);
        while let Some(item) = next.take() {
            match item {
                Outgoing::Event(payload) => batch.push(payload),
                Outgoing::Flush(done) => {
                    flushed = Some(done);
                    break;
                }
            }
            if batch.len() < PUBLISH_BATCH_LIMIT {
                next = outgoing.try_recv().ok();
            }
        }

        if !batch.is_empty() {
            let count = batch.len();
            // A panic must cost one batch, not the task: with the task gone
            // every later event would only be delivered locally.
            let publishing = AssertUnwindSafe(async { publish(batch).await }).catch_unwind();
            match publishing.await {
                Ok(Ok(())) => {}
                Ok(Err(PublishError::NotConnected)) => {
                    let dropped = dropped.fetch_add(count as u64, Ordering::Relaxed) + count as u64;
                    if not_connected_warned_at
                        .map_or(true, |at| at.elapsed() >= NOT_CONNECTED_WARN_INTERVAL)
                    {
                        not_connected_warned_at = Some(Instant::now());
                        tracing::warn!(
                            "Redis is not connected, other replicas will miss {} WebSocket events ({} dropped so far)",
                            count,
                            dropped
                        );
                    }
                }
                Ok(Err(PublishError::Failed { unpublished, error })) => {
                    let dropped = dropped.fetch_add(unpublished as u64, Ordering::Relaxed)
                        + unpublished as u64;
                    tracing::warn!(
                        "Failed to publish {} of {} WebSocket events, other replicas will miss them ({} dropped so far): {}",
                        unpublished,
                        count,
                        dropped,
                        error
                    );
                }
                Err(_) => tracing::error!(
                    "Publishing {} WebSocket events panicked, other replicas will miss them",
                    count
                ),
            }
        }
        if let Some(done) = flushed {
            let _ = done.send(());
        }
    }
}

/// Release the presence of every local connection before the replica stops.
///
/// A stopped replica never reaches the cleanup in [`handle_socket`], so its
/// connections would keep the users online until they expire, and closing
/// the last tab on another replica would not send the offline update.
///
/// No offline update is sent: clients reconnect to another replica within
/// seconds, and offline followed by online would make every user of the
/// replica blink on each deploy. A user left with no connection keeps the
/// online flag for a short TTL instead.
pub async fn release_presence(connections: &Connections, presence: &PresenceService) {
    connections.closing.store(true, Ordering::SeqCst);
    // Taken before the wait, so connections that close meanwhile, and skip
    // their own cleanup because the replica is closing, are released too.
    let local: Vec<(String, Vec<Uuid>)> = connections
        .local
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().keys().copied().collect()))
        .collect();
    // With an empty snapshot there is nothing to release and nothing to wait
    // for: a connection registered after `closing` was set skips presence.
    if presence.is_configured() && !local.is_empty() {
        tokio::time::sleep(IN_FLIGHT_SCRIPTS_WAIT).await;
    }

    futures::stream::iter(local)
        .for_each_concurrent(RELEASE_CONCURRENCY, |(user_id, conn_ids)| async move {
            for conn_id in conn_ids {
                if let Err(e) = presence.release(&user_id, conn_id).await {
                    tracing::warn!("Failed to release presence of user {}: {}", user_id, e);
                }
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
        broadcast_presence(
            &connections,
            &participants,
            &user_id,
            true,
            Reach::AllReplicas,
        )
        .await;
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
                                        Reach::AllReplicas,
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
    let offline_reach = if connections.is_closing() {
        None
    } else {
        let disconnected = presence.disconnect(&user_id, conn_id).await;
        if let Err(e) = &disconnected {
            tracing::warn!("Failed to set user {} offline: {}", user_id, e);
        }
        offline_reach(disconnected, last_local)
    };
    if let Some(reach) = offline_reach {
        broadcast_presence(&connections, &participants, &user_id, false, reach).await;
    }

    send_task.abort();
    tracing::info!("WebSocket disconnected: {} ({})", user_id, conn_id);
}

/// Where to send the offline update after a connection closed, if anywhere.
/// When Redis failed, only this replica's connections are known to be gone,
/// so the update stays on this replica.
fn offline_reach(
    disconnected: Result<Option<bool>, fred::error::Error>,
    last_local: bool,
) -> Option<Reach> {
    let (last_connection, reach) = match disconnected {
        Ok(last) => (last.unwrap_or(last_local), Reach::AllReplicas),
        Err(_) => (last_local, Reach::ThisReplica),
    };
    last_connection.then_some(reach)
}

/// Broadcast presence update to users who share dialogs with the target user
async fn broadcast_presence(
    connections: &Connections,
    participants: &ParticipantRepository,
    user_id: &str,
    is_online: bool,
    reach: Reach,
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
    deliver_event(connections, &event, &recipient_ids, reach);
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
    deliver_event(connections, event, user_ids, Reach::AllReplicas);
}

fn deliver_event(connections: &Connections, event: &WsEvent, user_ids: &[String], reach: Reach) {
    let json = match serde_json::to_string(event) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("Failed to serialize WsEvent: {}", e);
            return;
        }
    };

    connections.deliver(user_ids, json, reach);
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

    // On the single-threaded test runtime the publisher runs only when the test
    // yields: it takes the first event alone and the rest queue up behind it,
    // so the events go out in several batches.
    #[tokio::test]
    async fn queued_events_are_published_in_order() {
        let published = Arc::new(std::sync::Mutex::new(Vec::new()));
        let batches = Arc::new(AtomicU64::new(0));
        let remaining = Arc::new(AtomicU64::new(10));
        let sink = published.clone();
        let batch_count = batches.clone();
        // Earlier batches take longer to publish, so concurrent publishing would reorder them.
        let connections = Connections::with_publisher(16, move |payloads| {
            let sink = sink.clone();
            batch_count.fetch_add(1, Ordering::SeqCst);
            let delay = remaining.fetch_sub(1, Ordering::SeqCst);
            async move {
                tokio::time::sleep(Duration::from_millis(delay * 5)).await;
                sink.lock().unwrap().extend(payloads);
                Ok(())
            }
        });
        let dialog_ids: Vec<String> = (0..10).map(|_| Uuid::new_v4().to_string()).collect();

        for dialog_id in &dialog_ids {
            let event = WsEvent::DialogArchived {
                dialog_id: dialog_id.parse().unwrap(),
            };
            broadcast_to_users(&connections, &event, &["user".to_string()]).await;
            tokio::task::yield_now().await;
        }
        connections.flush_publisher(Duration::from_secs(5)).await;

        assert!(batches.load(Ordering::SeqCst) > 1);
        assert_eq!(published_dialog_ids(&published.lock().unwrap()), dialog_ids);
    }

    // The publisher task must not run between the broadcasts, or it would take
    // events off the queue and fewer would be dropped; on a single-threaded
    // runtime it runs only when the test yields.
    #[tokio::test(flavor = "current_thread")]
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

    #[tokio::test(flavor = "current_thread")]
    async fn events_queued_together_are_published_in_one_batch() {
        let batches = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = batches.clone();
        let connections = Connections::with_publisher(16, move |payloads: Vec<String>| {
            sink.lock().unwrap().push(payloads.len());
            async { Ok(()) }
        });

        for _ in 0..3 {
            broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        }
        connections.flush_publisher(Duration::from_secs(5)).await;

        assert_eq!(*batches.lock().unwrap(), vec![3]);
    }

    #[tokio::test]
    async fn publisher_keeps_publishing_after_a_panic() {
        let published = Arc::new(std::sync::Mutex::new(Vec::new()));
        let calls = Arc::new(AtomicU64::new(0));
        let sink = published.clone();
        let connections = Connections::with_publisher(16, move |payloads| {
            if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                panic!("publisher bug");
            }
            sink.lock().unwrap().extend(payloads);
            async { Ok(()) }
        });
        let lost = Uuid::new_v4();
        let delivered = Uuid::new_v4();

        for dialog_id in [lost, delivered] {
            let event = WsEvent::DialogArchived { dialog_id };
            broadcast_to_users(&connections, &event, &["user".to_string()]).await;
            connections.flush_publisher(Duration::from_secs(5)).await;
        }

        assert_eq!(
            published_dialog_ids(&published.lock().unwrap()),
            vec![delivered.to_string()]
        );
    }

    #[tokio::test]
    async fn stopped_publisher_counts_events_as_dropped() {
        let (queue, outgoing) = mpsc::channel(16);
        drop(outgoing);
        let connections = Connections {
            publisher: Some(queue),
            ..Connections::default()
        };
        let (_, mut rx) = connect(&connections, "user");

        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;

        assert_eq!(connections.dropped.load(Ordering::Relaxed), 2);
        assert!(connections.publisher_closed.load(Ordering::Relaxed));
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
    }

    #[tokio::test]
    async fn nothing_is_published_without_a_connected_redis_client() {
        let pool = fred::types::Builder::default_centralized()
            .build_pool(2)
            .unwrap();

        let result = publish_in_order(&pool, vec!["event".to_string()]).await;

        assert!(matches!(result, Err(PublishError::NotConnected)));
    }

    #[tokio::test]
    async fn skipped_and_unpublished_events_are_counted_as_dropped() {
        let calls = Arc::new(AtomicU64::new(0));
        let connections = Connections::with_publisher(16, move |payloads: Vec<String>| {
            let call = calls.fetch_add(1, Ordering::SeqCst);
            async move {
                match call {
                    0 => Err(PublishError::NotConnected),
                    _ => Err(PublishError::Failed {
                        unpublished: payloads.len() - 1,
                        error: fred::error::Error::new(
                            fred::error::ErrorKind::Timeout,
                            "timed out",
                        ),
                    }),
                }
            }
        });

        broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        connections.flush_publisher(Duration::from_secs(5)).await;
        for _ in 0..3 {
            broadcast_to_users(&connections, &deleted_event(), &["user".to_string()]).await;
        }
        connections.flush_publisher(Duration::from_secs(5)).await;

        assert_eq!(connections.dropped.load(Ordering::Relaxed), 1 + 2);
    }

    #[tokio::test]
    async fn stopping_without_connections_does_not_wait_for_scripts() {
        let pool = fred::types::Builder::default_centralized()
            .build_pool(1)
            .unwrap();
        let presence = PresenceService::new(Arc::new(pool));
        let connections = Connections::default();

        let release = release_presence(&connections, &presence);

        assert!(tokio::time::timeout(Duration::from_secs(1), release)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn this_replica_events_are_not_published() {
        let published = Arc::new(AtomicU64::new(0));
        let counter = published.clone();
        let connections = Connections::with_publisher(16, move |payloads: Vec<String>| {
            counter.fetch_add(payloads.len() as u64, Ordering::SeqCst);
            async { Ok(()) }
        });
        let (_, mut rx) = connect(&connections, "user");

        deliver_event(
            &connections,
            &deleted_event(),
            &["user".to_string()],
            Reach::ThisReplica,
        );
        connections.flush_publisher(Duration::from_secs(5)).await;

        assert!(rx.try_recv().is_ok());
        assert_eq!(published.load(Ordering::SeqCst), 0);
    }

    fn redis_down() -> Result<Option<bool>, fred::error::Error> {
        Err(fred::error::Error::new(
            fred::error::ErrorKind::Timeout,
            "presence script timed out",
        ))
    }

    #[test]
    fn offline_after_redis_failure_stays_on_this_replica() {
        assert_eq!(offline_reach(redis_down(), true), Some(Reach::ThisReplica));
        assert_eq!(offline_reach(redis_down(), false), None);
    }

    #[test]
    fn offline_goes_to_all_replicas_when_last_connection_anywhere_closed() {
        assert_eq!(
            offline_reach(Ok(Some(true)), false),
            Some(Reach::AllReplicas)
        );
        assert_eq!(offline_reach(Ok(Some(false)), true), None);
        assert_eq!(offline_reach(Ok(None), true), Some(Reach::AllReplicas));
    }
}
