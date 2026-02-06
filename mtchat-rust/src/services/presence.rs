//! Presence service for tracking user online status
//!
//! Uses Redis to store online status with TTL-based expiration.

use fred::clients::Pool;
use fred::interfaces::KeysInterface;
use fred::error::Error as RedisError;
use std::sync::Arc;
use uuid::Uuid;

/// TTL for online status keys in seconds (60s)
const ONLINE_TTL: i64 = 60;

/// Service for managing user online presence via Redis
pub struct PresenceService {
    redis: Option<Arc<Pool>>,
}

impl PresenceService {
    /// Create a new presence service with Redis connection
    pub fn new(redis: Arc<Pool>) -> Self {
        Self { redis: Some(redis) }
    }

    /// Create a no-op presence service (when Redis is not configured)
    pub fn noop() -> Self {
        Self { redis: None }
    }

    /// Check if service is configured
    pub fn is_configured(&self) -> bool {
        self.redis.is_some()
    }

    /// Set user as online (with TTL)
    pub async fn set_online(&self, user_id: Uuid) -> Result<(), RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(());
        };

        let key = format!("online:{}", user_id);
        redis
            .set::<(), _, _>(
                &key,
                "1",
                Some(fred::types::Expiration::EX(ONLINE_TTL)),
                None,
                false,
            )
            .await?;
        Ok(())
    }

    /// Refresh online status TTL (called on ping)
    pub async fn refresh_online(&self, user_id: Uuid) -> Result<(), RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(());
        };

        let key = format!("online:{}", user_id);
        redis.expire::<(), _>(&key, ONLINE_TTL, None).await?;
        Ok(())
    }

    /// Set user as offline (remove key)
    pub async fn set_offline(&self, user_id: Uuid) -> Result<(), RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(());
        };

        let key = format!("online:{}", user_id);
        redis.del::<(), _>(&key).await?;
        Ok(())
    }

    /// Get list of online users from a list of user IDs (batch check)
    pub async fn get_online_users(&self, user_ids: &[Uuid]) -> Result<Vec<Uuid>, RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(vec![]);
        };

        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let keys: Vec<String> = user_ids
            .iter()
            .map(|id| format!("online:{}", id))
            .collect();

        let results: Vec<Option<String>> = redis.mget(keys).await?;

        let online_users = user_ids
            .iter()
            .zip(results)
            .filter_map(|(id, result)| result.map(|_| *id))
            .collect();

        Ok(online_users)
    }

    /// Check if a single user is online
    pub async fn is_online(&self, user_id: Uuid) -> Result<bool, RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(false);
        };

        let key = format!("online:{}", user_id);
        let result: Option<String> = redis.get(&key).await?;
        Ok(result.is_some())
    }
}
