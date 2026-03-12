//! Database configuration
//!
//! Configures PostgreSQL connection pool via environment variables.

use std::time::Duration;

/// Database connection pool configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection acquire timeout
    pub acquire_timeout: Duration,
    /// Idle connection timeout
    pub idle_timeout: Duration,
    /// Maximum connection lifetime
    pub max_lifetime: Duration,
}

impl DatabaseConfig {
    /// Create config from environment variables
    ///
    /// Environment variables:
    /// - `DATABASE_URL` - PostgreSQL connection URL (required)
    /// - `DATABASE_MAX_CONNECTIONS` - Maximum pool size (default: 20)
    /// - `DATABASE_MIN_CONNECTIONS` - Minimum pool size (default: 5)
    /// - `DATABASE_ACQUIRE_TIMEOUT_SECS` - Acquire timeout in seconds (default: 30)
    /// - `DATABASE_IDLE_TIMEOUT_SECS` - Idle timeout in seconds (default: 600)
    /// - `DATABASE_MAX_LIFETIME_SECS` - Max lifetime in seconds (default: 1800)
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/multitenancy_chat".into()
        });

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        let acquire_timeout = std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30));

        let idle_timeout = std::env::var("DATABASE_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(600));

        let max_lifetime = std::env::var("DATABASE_MAX_LIFETIME_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(1800));

        Self {
            url,
            max_connections,
            min_connections,
            acquire_timeout,
            idle_timeout,
            max_lifetime,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/multitenancy_chat".into(),
            max_connections: 20,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.acquire_timeout, Duration::from_secs(30));
    }
}
