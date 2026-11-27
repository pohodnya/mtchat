//! Cross-replica WebSocket delivery and presence over a shared Redis.
//!
//! Each "pod" is a separate connection registry with its own Redis clients,
//! the same way two API replicas run in production.
//!
//! Run with: cargo test --test ws_bus_test
//! Requires: REDIS_URL environment variable (defaults to redis://localhost:6379)

use fred::prelude::*;
use fred::types::Builder;
use multitenancy_chat_api::services::PresenceService;
use multitenancy_chat_api::ws::{
    broadcast_dialog_archived, close_connection, register_connection, release_presence, Connections,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;

fn redis_config() -> Config {
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    Config::from_url(&url).expect("Failed to parse REDIS_URL")
}

async fn redis_pool() -> Arc<Pool> {
    let pool = Builder::from_config(redis_config())
        .build_pool(2)
        .expect("Failed to create Redis pool");
    pool.init().await.expect("Failed to connect to Redis");
    Arc::new(pool)
}

async fn start_pod() -> Connections {
    let publisher = Builder::from_config(redis_config())
        .build()
        .expect("Failed to create Redis publisher");
    publisher.init().await.expect("Failed to connect to Redis");
    let connections = Connections::with_redis(publisher);
    let subscriber = Builder::from_config(redis_config())
        .build_subscriber_client()
        .expect("Failed to create Redis subscriber");
    connections
        .start_subscriber(subscriber)
        .await
        .expect("Failed to subscribe");
    connections
}

fn conns_key(user_id: &str) -> String {
    format!("presence:conns:{}", user_id)
}

fn online_key(user_id: &str) -> String {
    format!("online:{}", user_id)
}

/// Overwrite a connection's expiry timestamp, e.g. to make it already expired.
async fn set_expiry(redis: &Pool, user_id: &str, conn_id: Uuid, expiry: f64) {
    let _: () = redis
        .zadd(
            conns_key(user_id),
            None,
            None,
            false,
            false,
            (expiry, conn_id.to_string()),
        )
        .await
        .unwrap();
}

async fn expiry(redis: &Pool, user_id: &str, conn_id: Uuid) -> f64 {
    redis
        .zscore(conns_key(user_id), conn_id.to_string())
        .await
        .unwrap()
}

fn unix_now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

fn connect(pod: &Connections, user_id: &str) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>(10);
    register_connection(pod, user_id, Uuid::new_v4(), tx);
    rx
}

async fn next_event(rx: &mut mpsc::Receiver<String>) -> Option<String> {
    tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .ok()
        .flatten()
}

async fn assert_no_more_events(rx: &mut mpsc::Receiver<String>) {
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(rx.try_recv().is_err(), "event delivered more than once");
}

#[tokio::test]
async fn event_from_one_pod_reaches_connection_on_another() {
    let pod_a = start_pod().await;
    let pod_b = start_pod().await;
    let user_id = Uuid::new_v4().to_string();
    let dialog_id = Uuid::new_v4();
    let mut tab_on_a = connect(&pod_a, &user_id);

    broadcast_dialog_archived(&pod_b, dialog_id, std::slice::from_ref(&user_id)).await;

    let event = next_event(&mut tab_on_a)
        .await
        .expect("event not delivered");
    assert!(event.contains("dialog.archived"));
    assert!(event.contains(&dialog_id.to_string()));
    assert_no_more_events(&mut tab_on_a).await;
}

#[tokio::test]
async fn tabs_on_both_pods_receive_event_exactly_once() {
    let pod_a = start_pod().await;
    let pod_b = start_pod().await;
    let user_id = Uuid::new_v4().to_string();
    let mut tab_on_a = connect(&pod_a, &user_id);
    let mut tab_on_b = connect(&pod_b, &user_id);

    broadcast_dialog_archived(&pod_a, Uuid::new_v4(), std::slice::from_ref(&user_id)).await;

    assert!(next_event(&mut tab_on_a).await.is_some());
    assert!(next_event(&mut tab_on_b).await.is_some());
    assert_no_more_events(&mut tab_on_a).await;
    assert_no_more_events(&mut tab_on_b).await;
}

#[tokio::test]
async fn event_does_not_reach_unlisted_user_on_another_pod() {
    let pod_a = start_pod().await;
    let pod_b = start_pod().await;
    let recipient_id = Uuid::new_v4().to_string();
    let mut recipient_on_a = connect(&pod_a, &recipient_id);
    let mut outsider_on_a = connect(&pod_a, &Uuid::new_v4().to_string());

    broadcast_dialog_archived(&pod_b, Uuid::new_v4(), &[recipient_id]).await;

    assert!(next_event(&mut recipient_on_a).await.is_some());
    assert_no_more_events(&mut outsider_on_a).await;
}

#[tokio::test]
async fn closing_connection_on_one_pod_keeps_user_online() {
    let pod_a = PresenceService::new(redis_pool().await);
    let pod_b = PresenceService::new(redis_pool().await);
    let user_id = Uuid::new_v4().to_string();
    let conn_a = Uuid::new_v4();
    let conn_b = Uuid::new_v4();

    assert_eq!(pod_a.connect(&user_id, conn_a).await.unwrap(), Some(true));
    assert_eq!(pod_b.connect(&user_id, conn_b).await.unwrap(), Some(false));

    assert_eq!(
        pod_a.disconnect(&user_id, conn_a).await.unwrap(),
        Some(false)
    );
    assert!(pod_b.is_online(&user_id).await.unwrap());

    assert_eq!(
        pod_b.disconnect(&user_id, conn_b).await.unwrap(),
        Some(true)
    );
    assert!(!pod_a.is_online(&user_id).await.unwrap());
}

#[tokio::test]
async fn connection_of_crashed_pod_expires() {
    let redis = redis_pool().await;
    let presence = PresenceService::new(redis.clone());
    let user_id = Uuid::new_v4().to_string();
    // A crashed pod leaves its connection behind; its expiry has already passed.
    let _: () = redis
        .zadd(
            format!("presence:conns:{}", user_id),
            None,
            None,
            false,
            false,
            (1.0, Uuid::new_v4().to_string()),
        )
        .await
        .unwrap();
    let conn = Uuid::new_v4();

    assert_eq!(presence.connect(&user_id, conn).await.unwrap(), Some(true));
    assert_eq!(
        presence.disconnect(&user_id, conn).await.unwrap(),
        Some(true)
    );
    assert!(!presence.is_online(&user_id).await.unwrap());
}

#[tokio::test]
async fn refresh_extends_connection_expiry() {
    let redis = redis_pool().await;
    let presence = PresenceService::new(redis.clone());
    let user_id = Uuid::new_v4().to_string();
    let conn = Uuid::new_v4();
    presence.connect(&user_id, conn).await.unwrap();
    set_expiry(&redis, &user_id, conn, unix_now() + 5.0).await;

    assert_eq!(presence.refresh(&user_id, conn).await.unwrap(), Some(false));

    assert!(expiry(&redis, &user_id, conn).await > unix_now() + 60.0);
}

#[tokio::test]
async fn refresh_of_expired_connection_reports_user_back_online() {
    let redis = redis_pool().await;
    let presence = PresenceService::new(redis.clone());
    let user_id = Uuid::new_v4().to_string();
    let conn = Uuid::new_v4();
    presence.connect(&user_id, conn).await.unwrap();
    set_expiry(&redis, &user_id, conn, 1.0).await;

    assert_eq!(presence.refresh(&user_id, conn).await.unwrap(), Some(true));
    assert!(presence.is_online(&user_id).await.unwrap());
}

#[tokio::test]
async fn refresh_of_expired_connection_keeps_online_user_silent() {
    let redis = redis_pool().await;
    let presence = PresenceService::new(redis.clone());
    let user_id = Uuid::new_v4().to_string();
    let background_tab = Uuid::new_v4();
    presence.connect(&user_id, background_tab).await.unwrap();
    presence.connect(&user_id, Uuid::new_v4()).await.unwrap();
    set_expiry(&redis, &user_id, background_tab, 1.0).await;

    assert_eq!(
        presence.refresh(&user_id, background_tab).await.unwrap(),
        Some(false)
    );
}

#[tokio::test]
async fn expired_connection_does_not_keep_user_online_when_last_tab_closes() {
    let redis = redis_pool().await;
    let presence = PresenceService::new(redis.clone());
    let user_id = Uuid::new_v4().to_string();
    let live_tab = Uuid::new_v4();
    let stale_tab = Uuid::new_v4();
    presence.connect(&user_id, live_tab).await.unwrap();
    presence.connect(&user_id, stale_tab).await.unwrap();
    set_expiry(&redis, &user_id, stale_tab, 1.0).await;

    assert_eq!(
        presence.disconnect(&user_id, live_tab).await.unwrap(),
        Some(true)
    );
    assert!(!presence.is_online(&user_id).await.unwrap());
}

#[tokio::test]
async fn stopping_pod_releases_its_connections_without_taking_users_offline() {
    let redis = redis_pool().await;
    let pod_a = start_pod().await;
    let presence_a = PresenceService::new(redis_pool().await);
    let presence_b = PresenceService::new(redis.clone());
    let only_on_a = Uuid::new_v4().to_string();
    let on_both = Uuid::new_v4().to_string();
    let mut conns_on_a = Vec::new();
    for user_id in [&only_on_a, &on_both] {
        let conn = Uuid::new_v4();
        let (tx, _rx) = mpsc::channel::<String>(10);
        register_connection(&pod_a, user_id, conn, tx);
        presence_a.connect(user_id, conn).await.unwrap();
        conns_on_a.push((user_id.clone(), conn));
    }
    presence_b.connect(&on_both, Uuid::new_v4()).await.unwrap();

    release_presence(&pod_a, &presence_a).await;

    for (user_id, conn) in &conns_on_a {
        let score: Option<f64> = redis
            .zscore(conns_key(user_id), conn.to_string())
            .await
            .unwrap();
        assert!(score.is_none(), "connection of the stopped pod left behind");
    }
    // The user's client is expected to reconnect elsewhere within the short TTL.
    let ttl: i64 = redis.ttl(online_key(&only_on_a)).await.unwrap();
    assert!((1..=30).contains(&ttl), "unexpected online TTL {}", ttl);
    let ttl: i64 = redis.ttl(online_key(&on_both)).await.unwrap();
    assert!(
        ttl > 30,
        "user with a connection on another pod lost the TTL"
    );
    assert_eq!(
        presence_b
            .connect(&only_on_a, Uuid::new_v4())
            .await
            .unwrap(),
        Some(true)
    );
    let ttl: i64 = redis.ttl(online_key(&only_on_a)).await.unwrap();
    assert!(ttl > 30, "reconnect did not extend the online flag");
}

#[tokio::test]
async fn connection_closed_while_pod_stops_releases_its_presence() {
    let redis = redis_pool().await;
    let pod = start_pod().await;
    let presence = PresenceService::new(redis.clone());
    let user_id = Uuid::new_v4().to_string();
    let conn = Uuid::new_v4();
    presence.connect(&user_id, conn).await.unwrap();
    // The connection is not registered locally, as if it was unregistered
    // just before the release took its snapshot.
    release_presence(&pod, &presence).await;

    let offline_reach = close_connection(&pod, &presence, &user_id, conn).await;

    assert_eq!(offline_reach, None);
    let score: Option<f64> = redis
        .zscore(conns_key(&user_id), conn.to_string())
        .await
        .unwrap();
    assert!(score.is_none(), "closed connection left in presence");
    let ttl: i64 = redis.ttl(online_key(&user_id)).await.unwrap();
    assert!((1..=30).contains(&ttl), "unexpected online TTL {}", ttl);
}
