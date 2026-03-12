//! JWT Configuration for Chat API authentication
//!
//! When enabled (JWT_AUTH_ENABLED=true), validates JWT tokens on Chat API requests.
//! The token is expected to be signed with HS256 algorithm.
//! Expiration is NOT checked - the token is validated only for signature correctness.

use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use std::sync::OnceLock;

static JWT_CONFIG: OnceLock<Option<JwtConfig>> = OnceLock::new();

/// JWT configuration loaded from environment variables
pub struct JwtConfig {
    /// Decoding key for JWT signature verification
    pub decoding_key: DecodingKey,
    /// Validation settings (HS256, no expiration check)
    pub validation: Validation,
}

impl JwtConfig {
    /// Initialize JWT configuration from environment variables.
    /// Must be called once at startup.
    ///
    /// Environment variables:
    /// - `JWT_AUTH_ENABLED`: Set to "true" or "1" to enable JWT auth
    /// - `JWT_SECRET`: Secret key for HS256 signature verification (required if enabled)
    pub fn init() {
        JWT_CONFIG.get_or_init(|| {
            let enabled = std::env::var("JWT_AUTH_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);

            if !enabled {
                tracing::info!("JWT auth disabled (JWT_AUTH_ENABLED not set)");
                return None;
            }

            let secret = std::env::var("JWT_SECRET")
                .expect("JWT_SECRET required when JWT_AUTH_ENABLED=true");

            let mut validation = Validation::new(Algorithm::HS256);
            // Don't validate expiration - token is reused from host application
            validation.validate_exp = false;
            // Don't require any specific claims
            validation.required_spec_claims.clear();

            tracing::info!("JWT auth enabled for Chat API");
            Some(JwtConfig {
                decoding_key: DecodingKey::from_secret(secret.as_bytes()),
                validation,
            })
        });
    }

    /// Get the JWT configuration (if enabled)
    pub fn get() -> Option<&'static JwtConfig> {
        JWT_CONFIG.get().and_then(|c| c.as_ref())
    }

    /// Check if JWT auth is enabled
    pub fn is_enabled() -> bool {
        JWT_CONFIG.get().map(|c| c.is_some()).unwrap_or(false)
    }
}
