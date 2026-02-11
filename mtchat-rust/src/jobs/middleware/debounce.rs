//! Debounce utilities for job deduplication.
//!
//! When multiple jobs with the same debounce key are enqueued,
//! only the most recent one is executed. Earlier jobs are skipped.

use std::sync::Arc;

use fred::clients::Pool as RedisPool;
use fred::prelude::*;

/// Check if a job is still current (not superseded by newer job).
///
/// Returns `true` if this job should be executed, `false` if it was debounced.
pub async fn is_job_current(
    redis: &Arc<RedisPool>,
    prefix: &str,
    debounce_key: &str,
    job_id: &str,
) -> bool {
    let redis_key = format!("{}:{}", prefix, debounce_key);

    // Check if this job is still current
    let stored_job_id: Option<String> = match redis.get(&redis_key).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(
                error = %e,
                debounce_key = %debounce_key,
                "Failed to check debounce key, proceeding with execution"
            );
            // On Redis error, proceed with execution to avoid losing jobs
            return true;
        }
    };

    match stored_job_id {
        Some(ref stored_id) if stored_id != job_id => {
            // Job has been superseded by a newer one
            tracing::debug!(
                debounce_key = %debounce_key,
                current_job_id = %job_id,
                stored_job_id = %stored_id,
                "Job debounced (superseded by newer job)"
            );
            false
        }
        Some(_) => {
            // This is the current job
            tracing::debug!(
                debounce_key = %debounce_key,
                job_id = %job_id,
                "Job is current, executing"
            );
            true
        }
        None => {
            // No debounce key found (expired or never set)
            tracing::debug!(
                debounce_key = %debounce_key,
                "No debounce key found, executing job"
            );
            true
        }
    }
}

/// Clean up debounce key after job processing.
pub async fn cleanup_debounce_key(redis: &Arc<RedisPool>, prefix: &str, debounce_key: &str) {
    let redis_key = format!("{}:{}", prefix, debounce_key);
    if let Err(e) = redis.del::<(), _>(&redis_key).await {
        tracing::warn!(
            error = %e,
            debounce_key = %debounce_key,
            "Failed to delete debounce key"
        );
    }
}

/// Debounce key prefix for notification jobs.
pub const NOTIFICATION_DEBOUNCE_PREFIX: &str = "mtchat:debounce:notifications";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debounce_prefix() {
        assert_eq!(NOTIFICATION_DEBOUNCE_PREFIX, "mtchat:debounce:notifications");
    }
}
