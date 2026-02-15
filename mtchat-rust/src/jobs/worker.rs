//! Worker configuration and startup.

use std::str::FromStr;
use std::sync::Arc;

use apalis::prelude::*;
use apalis_cron::{CronStream, Schedule};
use apalis_redis::RedisStorage;
use fred::clients::Pool as RedisPool;

use super::handlers::{handle_auto_archive, handle_notification, JobContext};
use super::types::NotificationJob;

/// Worker configuration.
#[derive(Clone)]
pub struct WorkerConfig {
    /// Cron schedule for auto-archive job.
    pub archive_cron: String,
    /// Seconds of inactivity before auto-archive (default: 259200 = 3 days).
    pub archive_after_secs: i64,
    /// Number of concurrent notification workers.
    pub notification_concurrency: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            archive_cron: "0 */5 * * * *".to_string(), // every 5 minutes
            archive_after_secs: 259200, // 3 days
            notification_concurrency: 4,
        }
    }
}

impl WorkerConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Self {
        Self {
            archive_cron: std::env::var("ARCHIVE_CRON")
                .unwrap_or_else(|_| "0 */5 * * * *".to_string()),
            archive_after_secs: std::env::var("ARCHIVE_AFTER_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(259200), // 3 days
            notification_concurrency: std::env::var("NOTIFICATION_CONCURRENCY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(4),
        }
    }
}

/// Start background job workers.
///
/// Returns a Monitor that manages the workers.
pub async fn start_workers(
    notification_storage: RedisStorage<NotificationJob>,
    _redis: Arc<RedisPool>,
    ctx: JobContext,
    config: WorkerConfig,
) -> Result<Monitor, WorkerError> {
    // Build notification worker
    let notification_worker = WorkerBuilder::new("mtchat-notifications")
        .concurrency(config.notification_concurrency)
        .data(ctx.clone())
        .backend(notification_storage)
        .build_fn(handle_notification);

    // Build auto-archive cron worker
    let archive_schedule = Schedule::from_str(&config.archive_cron)
        .map_err(|e| WorkerError::InvalidCron(e.to_string()))?;

    let archive_worker = WorkerBuilder::new("mtchat-auto-archive")
        .data(ctx)
        .backend(CronStream::new(archive_schedule))
        .build_fn(handle_auto_archive);

    // Create monitor
    let monitor = Monitor::new()
        .register(notification_worker)
        .register(archive_worker);

    tracing::info!(
        notification_concurrency = config.notification_concurrency,
        archive_cron = %config.archive_cron,
        "Job workers configured"
    );

    Ok(monitor)
}

/// Errors that can occur when starting workers.
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("Redis connection error: {0}")]
    Redis(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorkerConfig::default();
        assert_eq!(config.archive_after_secs, 259200); // 3 days
        assert_eq!(config.notification_concurrency, 4);
    }

    #[test]
    fn test_valid_cron_schedule() {
        let cron = "0 */5 * * * *";
        assert!(Schedule::from_str(cron).is_ok());
    }
}
