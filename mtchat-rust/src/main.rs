//! MTChat API - Embeddable Chat Service Backend
//!
//! Object-bound chat service with direct and potential participants.

use axum::{
    middleware as axum_middleware,
    routing::{get, post, delete, put},
    Router,
};
use std::{env, sync::Arc};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use multitenancy_chat_api::api::{self, AppState};
use multitenancy_chat_api::middleware;
use multitenancy_chat_api::webhooks::{WebhookSender, WebhookConfig};
use multitenancy_chat_api::services::{S3Service, S3Config, PresenceService};
use multitenancy_chat_api::jobs::{JobProducer, JobContext, WorkerConfig, start_workers, NotificationJob};
use fred::prelude::*;
use fred::types::Builder;
use apalis_redis::RedisStorage;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "multitenancy_chat_api=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize admin token (read once, constant-time comparison)
    middleware::init_admin_token();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/multitenancy_chat".into());

    tracing::info!("Connecting to database...");
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    // Initialize webhook sender
    let webhooks = match (env::var("WEBHOOK_URL"), env::var("WEBHOOK_SECRET")) {
        (Ok(url), Ok(secret)) => {
            tracing::info!("Webhooks enabled, sending to: {}", url);
            WebhookSender::new(WebhookConfig::new(url, secret))
        }
        _ => {
            tracing::info!("Webhooks disabled (WEBHOOK_URL or WEBHOOK_SECRET not set)");
            WebhookSender::noop()
        }
    };

    // Initialize S3 service
    let s3 = match S3Config::from_env() {
        Ok(config) => {
            tracing::info!("S3 enabled, bucket: {}", config.bucket);
            S3Service::new(config).await
        }
        Err(e) => {
            tracing::warn!("S3 disabled: {}", e);
            S3Service::noop()
        }
    };

    // Initialize Redis, presence service, and job queue
    let (presence, jobs, redis_pool) = match env::var("REDIS_URL") {
        Ok(url) => {
            tracing::info!("Connecting to Redis...");
            let config = Config::from_url(&url)
                .expect("Failed to parse REDIS_URL");
            let pool = Builder::from_config(config)
                .build_pool(5)
                .expect("Failed to create Redis pool");
            pool.init().await.expect("Failed to connect to Redis");
            tracing::info!("Redis connected, presence tracking enabled");

            let redis_pool = Arc::new(pool);

            // Initialize job producer
            let worker_config = WorkerConfig::from_env();
            let apalis_conn = apalis_redis::connect(url.clone())
                .await
                .expect("Failed to connect to Redis for job queue");
            let notification_storage: RedisStorage<NotificationJob> =
                RedisStorage::new_with_config(
                    apalis_conn,
                    apalis_redis::Config::default()
                        .set_poll_interval(std::time::Duration::from_millis(200)),
                );

            let jobs = JobProducer::new(notification_storage.clone());

            tracing::info!("Job queue enabled");

            (
                PresenceService::new(redis_pool.clone()),
                jobs,
                Some((redis_pool, notification_storage, worker_config)),
            )
        }
        Err(_) => {
            tracing::info!("Redis disabled (REDIS_URL not set), presence tracking and job queue disabled");
            (PresenceService::noop(), JobProducer::noop(), None)
        }
    };

    let state = AppState::new(db.clone(), webhooks.clone(), s3, presence, jobs);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Management API routes (with admin auth middleware)
    let management_routes = Router::new()
        .route("/dialogs", post(api::management::management_create_dialog))
        .route("/dialogs/{id}", get(api::management::management_get_dialog).delete(api::management::management_delete_dialog))
        .route("/dialogs/{id}/participants", post(api::management::management_add_participant))
        .route("/dialogs/{id}/participants/{user_id}", delete(api::management::management_remove_participant))
        .route("/dialogs/{id}/access-scopes", put(api::management::management_update_access_scopes))
        .layer(axum_middleware::from_fn(middleware::admin_auth::admin_auth));

    let app = Router::new()
        // Health
        .route("/health", get(api::health::health))
        .route("/health/ready", get(api::health::health_ready))

        // Management API (protected)
        .nest("/api/v1/management", management_routes)

        // Chat API - Dialogs
        .route("/api/v1/dialogs", get(api::dialogs::list_dialogs))
        .route("/api/v1/dialogs/{id}", get(api::dialogs::get_dialog))
        .route("/api/v1/dialogs/by-object/{object_type}/{object_id}", get(api::dialogs::get_dialog_by_object))
        .route("/api/v1/dialogs/{id}/join", post(api::dialogs::join_dialog))
        .route("/api/v1/dialogs/{id}/leave", post(api::dialogs::leave_dialog))
        .route("/api/v1/dialogs/{id}/archive", post(api::dialogs::archive_dialog))
        .route("/api/v1/dialogs/{id}/unarchive", post(api::dialogs::unarchive_dialog))
        .route("/api/v1/dialogs/{id}/pin", post(api::dialogs::pin_dialog))
        .route("/api/v1/dialogs/{id}/unpin", post(api::dialogs::unpin_dialog))
        .route("/api/v1/dialogs/{id}/notifications", post(api::dialogs::set_dialog_notifications))
        .route("/api/v1/dialogs/{id}/read", post(api::participants::mark_as_read))
        .route("/api/v1/dialogs/{id}/participants", get(api::participants::list_participants))

        // Chat API - Messages
        .route("/api/v1/dialogs/{dialog_id}/messages", get(api::messages::list_messages).post(api::messages::send_message))
        .route("/api/v1/dialogs/{dialog_id}/messages/{id}", get(api::messages::get_message).put(api::messages::edit_message).delete(api::messages::delete_message))

        // Upload API
        .route("/api/v1/upload/presign", post(api::upload::presign_upload))
        .route("/api/v1/attachments/{id}/url", get(api::upload::get_attachment_url))

        // WebSocket
        .route("/api/v1/ws", get(api::ws_handler::ws_handler))

        .layer(cors)
        .with_state(state.clone());

    // Start job workers if Redis is configured
    if let Some((redis_pool, notification_storage, worker_config)) = redis_pool {
        let job_ctx = JobContext {
            db: db.clone(),
            redis: redis_pool.clone(),
            dialogs: state.dialogs.clone(),
            participants: state.participants.clone(),
            messages: state.messages.clone(),
            webhooks: webhooks.clone(),
            connections: state.connections.clone(),
            archive_after_secs: worker_config.archive_after_secs,
        };

        let monitor = start_workers(notification_storage, redis_pool, job_ctx, worker_config)
            .await
            .expect("Failed to start job workers");

        tokio::spawn(async move {
            tracing::info!("Job workers started");
            if let Err(e) = monitor.run().await {
                tracing::error!("Job workers error: {}", e);
            }
        });
    }

    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().unwrap();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
