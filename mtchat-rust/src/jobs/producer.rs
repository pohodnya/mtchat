//! Job producer for enqueueing background tasks.

use std::sync::Arc;
use std::time::Duration;

use apalis::prelude::Storage;
use apalis_redis::RedisStorage;
use fred::clients::Pool as RedisPool;
use fred::prelude::*;

use super::types::NotificationJob;

/// Job producer for enqueueing background tasks.
#[derive(Clone)]
pub struct JobProducer {
    notifications: Option<RedisStorage<NotificationJob>>,
    redis: Option<Arc<RedisPool>>,
    delay: Duration,
}

impl JobProducer {
    /// Create a new job producer.
    pub fn new(
        notifications: RedisStorage<NotificationJob>,
        redis: Arc<RedisPool>,
        delay_seconds: u64,
    ) -> Self {
        Self {
            notifications: Some(notifications),
            redis: Some(redis),
            delay: Duration::from_secs(delay_seconds),
        }
    }

    /// Create a no-op producer (when job queue is disabled).
    pub fn noop() -> Self {
        Self {
            notifications: None,
            redis: None,
            delay: Duration::from_secs(30),
        }
    }

    /// Check if producer is enabled.
    pub fn is_enabled(&self) -> bool {
        self.notifications.is_some()
    }

    /// Enqueue a notification job with debounce.
    ///
    /// If a job with the same debounce key (dialog_id:recipient_id) already exists,
    /// the new job will replace it. Only the latest job will be executed.
    pub async fn enqueue_notification(&self, job: NotificationJob) -> Result<(), JobProducerError> {
        let notifications = match &self.notifications {
            Some(n) => n,
            None => {
                tracing::debug!("Job queue disabled, skipping notification");
                return Ok(());
            }
        };

        let redis = self.redis.as_ref().ok_or(JobProducerError::NotConfigured)?;

        let debounce_key = format!("mtchat:debounce:notifications:{}", job.debounce_key());
        let job_id = job.job_id.clone();

        // Store debounce key -> job_id mapping
        // This allows the worker to check if the job is still current
        let ttl = self.delay.as_secs() as i64 + 60; // delay + buffer
        redis
            .set::<(), _, _>(
                &debounce_key,
                &job_id,
                Some(Expiration::EX(ttl)),
                None,
                false,
            )
            .await
            .map_err(|e| JobProducerError::Redis(e.to_string()))?;

        // Schedule the job with delay (seconds)
        notifications
            .clone()
            .schedule(job, self.delay.as_secs() as i64)
            .await
            .map_err(|e| JobProducerError::Apalis(e.to_string()))?;

        tracing::debug!(
            debounce_key = %debounce_key,
            job_id = %job_id,
            delay_secs = self.delay.as_secs(),
            "Notification job enqueued"
        );

        Ok(())
    }
}

/// Errors that can occur when producing jobs.
#[derive(Debug, thiserror::Error)]
pub enum JobProducerError {
    #[error("Job queue not configured")]
    NotConfigured,

    #[error("Redis error: {0}")]
    Redis(String),

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
