//! Presence service for tracking user online status
//!
//! Uses Redis to store online status with TTL-based expiration.
//!
//! A user may be connected to several API replicas at once, so every
//! connection is tracked in a per-user sorted set (`presence:conns:<user>`,
//! member = connection id, score = expiry timestamp). The user goes offline
//! only when the set is empty; members of a replica that died without closing
//! its connections expire by score.

use fred::clients::Pool;
use fred::error::{Error as RedisError, ErrorKind as RedisErrorKind};
use fred::interfaces::{KeysInterface, LuaInterface};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// TTL for online status keys in seconds (120s)
/// Heartbeat is 30s, but browsers throttle timers in background tabs to
/// about once a minute, so the TTL leaves room for a throttled ping.
const ONLINE_TTL: i64 = 120;

/// TTL left on the online flag of a user whose last connection was on a
/// stopping replica: a client that reconnects to another replica within it
/// refreshes the flag, otherwise the user goes offline when it expires.
const RELEASED_ONLINE_TTL: i64 = 30;

/// Outer cap on a presence script, kept well above fred's command timeout
/// (`REDIS_COMMAND_TIMEOUT` in main.rs) so fred's fires first: fred marks a
/// command that timed out and never sends it after a reconnect, while a future
/// dropped by this cap leaves the command buffered, to run later and leave a
/// phantom connection in the set.
pub const SCRIPT_TIMEOUT: Duration = Duration::from_secs(3);

/// Adds or refreshes a connection and keeps the user online. Returns the
/// number of live connections before the call, the refreshed one included
/// only if it had not expired: 0 means the user was offline.
/// KEYS: conns set, online flag. ARGV: conn id, TTL.
const CONNECT_SCRIPT: &str = r#"
local now = tonumber(redis.call('TIME')[1])
local ttl = tonumber(ARGV[2])
redis.call('ZREMRANGEBYSCORE', KEYS[1], '-inf', now)
local before = redis.call('ZCARD', KEYS[1])
redis.call('ZADD', KEYS[1], now + ttl, ARGV[1])
redis.call('EXPIRE', KEYS[1], ttl)
redis.call('SET', KEYS[2], '1', 'EX', ttl)
return before
"#;

/// Removes a connection and returns the number of live connections left;
/// clears the online flag when none is left. KEYS: conns set, online flag. ARGV: conn id.
const DISCONNECT_SCRIPT: &str = r#"
local now = tonumber(redis.call('TIME')[1])
redis.call('ZREM', KEYS[1], ARGV[1])
redis.call('ZREMRANGEBYSCORE', KEYS[1], '-inf', now)
local left = redis.call('ZCARD', KEYS[1])
if left == 0 then
  redis.call('DEL', KEYS[1], KEYS[2])
end
return left
"#;

/// Removes a connection of a stopping replica and returns the number of live
/// connections left; when none is left the online flag gets a short TTL
/// instead of being removed. KEYS: conns set, online flag. ARGV: conn id, TTL.
const RELEASE_SCRIPT: &str = r#"
local now = tonumber(redis.call('TIME')[1])
redis.call('ZREM', KEYS[1], ARGV[1])
redis.call('ZREMRANGEBYSCORE', KEYS[1], '-inf', now)
local left = redis.call('ZCARD', KEYS[1])
if left == 0 then
  redis.call('DEL', KEYS[1])
  redis.call('EXPIRE', KEYS[2], tonumber(ARGV[2]))
end
return left
"#;

fn presence_keys(user_id: &str) -> Vec<String> {
    vec![
        format!("presence:conns:{}", user_id),
        format!("online:{}", user_id),
    ]
}

/// Run a presence script with a hard timeout, so a stalled Redis cannot hang
/// the WebSocket handshake or the pong; callers fall back to local tracking.
async fn eval_script(
    redis: &Pool,
    script: &str,
    user_id: &str,
    args: Vec<String>,
) -> Result<i64, RedisError> {
    match tokio::time::timeout(
        SCRIPT_TIMEOUT,
        redis.eval(script, presence_keys(user_id), args),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(RedisError::new(
            RedisErrorKind::Timeout,
            "presence script timed out",
        )),
    }
}

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

    /// Register a connection and mark the user online.
    /// Returns whether it is the user's first live connection across all
    /// replicas, or None when Redis is not configured.
    pub async fn connect(&self, user_id: &str, conn_id: Uuid) -> Result<Option<bool>, RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(None);
        };

        let before = eval_script(
            redis,
            CONNECT_SCRIPT,
            user_id,
            vec![conn_id.to_string(), ONLINE_TTL.to_string()],
        )
        .await?;
        Ok(Some(before == 0))
    }

    /// Refresh the connection and online status TTL (called on ping).
    /// Returns whether the user was offline, i.e. the connection had expired
    /// and no other one was alive, or None when Redis is not configured.
    pub async fn refresh(&self, user_id: &str, conn_id: Uuid) -> Result<Option<bool>, RedisError> {
        self.connect(user_id, conn_id).await
    }

    /// Unregister a connection; the user goes offline when no live connection
    /// is left on any replica. Returns whether it was the last one, or None
    /// when Redis is not configured.
    pub async fn disconnect(
        &self,
        user_id: &str,
        conn_id: Uuid,
    ) -> Result<Option<bool>, RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(None);
        };

        let left =
            eval_script(redis, DISCONNECT_SCRIPT, user_id, vec![conn_id.to_string()]).await?;
        Ok(Some(left == 0))
    }

    /// Unregister a connection of a stopping replica without taking the user
    /// offline right away: when it was the last connection, the online flag
    /// expires after a short TTL unless the client reconnects elsewhere.
    pub async fn release(&self, user_id: &str, conn_id: Uuid) -> Result<(), RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(());
        };

        eval_script(
            redis,
            RELEASE_SCRIPT,
            user_id,
            vec![conn_id.to_string(), RELEASED_ONLINE_TTL.to_string()],
        )
        .await?;
        Ok(())
    }

    /// Get list of online users from a list of user IDs (batch check)
    pub async fn get_online_users(&self, user_ids: &[String]) -> Result<Vec<String>, RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(vec![]);
        };

        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let keys: Vec<String> = user_ids.iter().map(|id| format!("online:{}", id)).collect();

        // Guard the Redis round-trip with a hard timeout. A silently-dropped
        // TCP connection makes `mget` block indefinitely, which surfaced as
        // gateway 504s on the only endpoint that touches Redis. On timeout we
        // degrade gracefully to "no one is online" rather than hanging.
        let results: Vec<Option<String>> =
            match tokio::time::timeout(std::time::Duration::from_secs(2), redis.mget(keys)).await {
                Ok(res) => res?,
                Err(_) => {
                    tracing::warn!("Redis mget timed out in get_online_users; assuming offline");
                    return Ok(vec![]);
                }
            };

        let online_users = user_ids
            .iter()
            .zip(results)
            .filter_map(|(id, result)| result.map(|_| id.clone()))
            .collect();

        Ok(online_users)
    }

    /// Check if a single user is online
    pub async fn is_online(&self, user_id: &str) -> Result<bool, RedisError> {
        let Some(redis) = &self.redis else {
            return Ok(false);
        };

        let key = format!("online:{}", user_id);
        let result: Option<String> = redis.get(&key).await?;
        Ok(result.is_some())
    }
}
