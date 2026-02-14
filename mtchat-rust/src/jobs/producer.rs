//! Job producer for enqueueing background tasks.

use apalis::prelude::Storage;
use apalis_redis::RedisStorage;

use super::types::NotificationJob;

/// Notification delay in seconds (check if message was read).
const NOTIFICATION_DELAY_SECS: i64 = 1;

/// Job producer for enqueueing background tasks.
#[derive(Clone)]
pub struct JobProducer {
    notifications: Option<RedisStorage<NotificationJob>>,
}

impl JobProducer {
    /// Create a new job producer.
    pub fn new(notifications: RedisStorage<NotificationJob>) -> Self {
        Self {
            notifications: Some(notifications),
        }
    }

    /// Create a no-op producer (when job queue is disabled).
    pub fn noop() -> Self {
        Self {
            notifications: None,
        }
    }

    /// Check if producer is enabled.
    pub fn is_enabled(&self) -> bool {
        self.notifications.is_some()
    }

    /// Enqueue a notification job with 1 second delay.
    ///
    /// The delay allows checking if user read the message while in chat.
    pub async fn enqueue_notification(&self, job: NotificationJob) -> Result<(), JobProducerError> {
        let notifications = match &self.notifications {
            Some(n) => n,
            None => {
                tracing::debug!("Job queue disabled, skipping notification");
                return Ok(());
            }
        };

        // Schedule the job with 1 second delay
        notifications
            .clone()
            .schedule(job, NOTIFICATION_DELAY_SECS)
            .await
            .map_err(|e| JobProducerError::Apalis(e.to_string()))?;

        tracing::debug!("Notification job enqueued with {}s delay", NOTIFICATION_DELAY_SECS);

        Ok(())
    }
}

/// Errors that can occur when producing jobs.
#[derive(Debug, thiserror::Error)]
pub enum JobProducerError {
    #[error("Job queue not configured")]
    NotConfigured,

    #[error("Apalis error: {0}")]
    Apalis(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_producer_is_not_enabled() {
        let producer = JobProducer::noop();
        assert!(!producer.is_enabled());
    }
}
