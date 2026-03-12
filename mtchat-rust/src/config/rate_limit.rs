//! Rate limiting configuration
//!
//! Configures request rate limits via environment variables.
//! Uses a token bucket algorithm (governor crate) for smooth rate limiting.

use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{num::NonZeroU32, sync::Arc, time::Duration};

/// Rate limiting configuration from environment variables
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    pub enabled: bool,
    /// Requests per second limit
    pub requests_per_second: u32,
    /// Burst capacity (allows short bursts above the rate)
    pub burst_size: u32,
}

impl RateLimitConfig {
    /// Create config from environment variables
    ///
    /// Environment variables:
    /// - `RATE_LIMIT_ENABLED` - Enable rate limiting (default: false)
    /// - `RATE_LIMIT_RPS` - Requests per second (default: 100)
    /// - `RATE_LIMIT_BURST` - Burst capacity (default: 50)
    pub fn from_env() -> Self {
        let enabled = std::env::var("RATE_LIMIT_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let requests_per_second = std::env::var("RATE_LIMIT_RPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let burst_size = std::env::var("RATE_LIMIT_BURST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);

        Self {
            enabled,
            requests_per_second,
            burst_size,
        }
    }

    /// Create a rate limiter from this config
    pub fn create_limiter(
        &self,
    ) -> Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>> {
        if !self.enabled {
            return None;
        }

        let rps =
            NonZeroU32::new(self.requests_per_second).unwrap_or(NonZeroU32::new(100).unwrap());
        let burst = NonZeroU32::new(self.burst_size).unwrap_or(NonZeroU32::new(50).unwrap());

        // Create quota: burst_size requests, replenishing at rps per second
        let quota = Quota::with_period(Duration::from_secs(1) / rps.get())
            .expect("valid quota")
            .allow_burst(burst);

        Some(Arc::new(RateLimiter::direct(quota)))
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: 100,
            burst_size: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.requests_per_second, 100);
        assert_eq!(config.burst_size, 50);
    }

    #[test]
    fn test_limiter_creation_disabled() {
        let config = RateLimitConfig::default();
        assert!(config.create_limiter().is_none());
    }

    #[test]
    fn test_limiter_creation_enabled() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_second: 10,
            burst_size: 5,
        };
        assert!(config.create_limiter().is_some());
    }
}
