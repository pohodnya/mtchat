//! JWT Configuration for Chat API authentication
//!
//! When enabled (JWT_AUTH_ENABLED=true), validates JWT tokens on Chat API requests.
//! The token is expected to be signed with HS256 algorithm.
//! Expiration is NOT checked - the token is validated only for signature correctness.
//!
//! Several signing secrets may be configured at once (`JWT_SECRETS`). A token is
//! accepted when its signature verifies against *any* of them. This lets one
//! MTChat instance serve host applications that sign with different keys (e.g. a
//! staging and a demo stand sharing a single chat deployment).

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::sync::OnceLock;

use crate::middleware::JwtClaims;

static JWT_CONFIG: OnceLock<Option<JwtConfig>> = OnceLock::new();

/// JWT configuration loaded from environment variables
pub struct JwtConfig {
    /// Decoding keys for JWT signature verification. A token is accepted when it
    /// verifies against any one of them.
    pub decoding_keys: Vec<DecodingKey>,
    /// Validation settings (HS256, no expiration check)
    pub validation: Validation,
    /// JWT claim name that holds the user identifier (default: "sub")
    pub user_id_claim: String,
}

impl JwtConfig {
    /// Initialize JWT configuration from environment variables.
    /// Must be called once at startup.
    ///
    /// Environment variables:
    /// - `JWT_AUTH_ENABLED`: Set to "true" or "1" to enable JWT auth
    /// - `JWT_SECRETS`: Comma-separated HS256 secrets; a token is accepted when it
    ///   verifies against any of them. Use when several host applications sign
    ///   with different keys.
    /// - `JWT_SECRET`: Single HS256 secret. Alias for a one-entry `JWT_SECRETS`.
    ///   Either this or `JWT_SECRETS` is required when auth is enabled.
    /// - `JWT_USER_ID_CLAIM`: Claim name to read user ID from (default: `sub`).
    ///   Use this when your host application encodes the user identifier under
    ///   a non-standard claim like `user_id`, `userId`, `id`, etc.
    pub fn init() {
        JWT_CONFIG.get_or_init(|| {
            let enabled = std::env::var("JWT_AUTH_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);

            if !enabled {
                tracing::info!("JWT auth disabled (JWT_AUTH_ENABLED not set)");
                return None;
            }

            let raw_secrets = std::env::var("JWT_SECRETS")
                .or_else(|_| std::env::var("JWT_SECRET"))
                .expect("JWT_SECRETS or JWT_SECRET required when JWT_AUTH_ENABLED=true");

            let secrets = parse_secrets(&raw_secrets);
            assert!(
                !secrets.is_empty(),
                "JWT_SECRETS must contain at least one non-empty secret"
            );

            let user_id_claim =
                std::env::var("JWT_USER_ID_CLAIM").unwrap_or_else(|_| "sub".to_string());

            let mut validation = Validation::new(Algorithm::HS256);
            // Don't validate expiration - token is reused from host application
            validation.validate_exp = false;
            // Don't require any specific claims
            validation.required_spec_claims.clear();

            tracing::info!(
                "JWT auth enabled for Chat API (user_id claim: {}, {} signing key(s))",
                user_id_claim,
                secrets.len()
            );
            Some(JwtConfig {
                decoding_keys: secrets
                    .iter()
                    .map(|s| DecodingKey::from_secret(s.as_bytes()))
                    .collect(),
                validation,
                user_id_claim,
            })
        });
    }

    /// Decode a token against the configured keys, returning its claims.
    ///
    /// Each key is tried in order; the first successful verification wins.
    /// Returns `None` when the signature matches none of them.
    pub fn decode_claims(&self, token: &str) -> Option<JwtClaims> {
        self.decoding_keys
            .iter()
            .find_map(|key| decode::<JwtClaims>(token, key, &self.validation).ok())
            .map(|data| data.claims)
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

/// Split a comma-separated secret list, trimming whitespace and dropping empties.
fn parse_secrets(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde_json::json;

    fn config_with(secrets: &[&str]) -> JwtConfig {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        JwtConfig {
            decoding_keys: secrets
                .iter()
                .map(|s| DecodingKey::from_secret(s.as_bytes()))
                .collect(),
            validation,
            user_id_claim: "user_id".to_string(),
        }
    }

    fn token_signed_with(secret: &str, user_id: &str) -> String {
        encode(
            &Header::default(),
            &json!({ "user_id": user_id }),
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_parse_secrets_splits_and_trims() {
        assert_eq!(parse_secrets("a,b"), vec!["a", "b"]);
        assert_eq!(parse_secrets(" a , b "), vec!["a", "b"]);
        assert_eq!(parse_secrets("a,,b,"), vec!["a", "b"]);
        assert_eq!(parse_secrets("solo"), vec!["solo"]);
        assert!(parse_secrets("  ,  ").is_empty());
    }

    #[test]
    fn test_accepts_token_from_any_configured_secret() {
        let config = config_with(&["stage-secret-key", "demo-secret-key"]);

        // Token from the first (stage) signer.
        let stage = config
            .decode_claims(&token_signed_with("stage-secret-key", "stage-user"))
            .expect("stage token should verify");
        assert_eq!(stage.user_id("user_id").as_deref(), Some("stage-user"));

        // Token from the second (demo) signer — this is the case that used to 401.
        let demo = config
            .decode_claims(&token_signed_with("demo-secret-key", "demo-user"))
            .expect("demo token should verify");
        assert_eq!(demo.user_id("user_id").as_deref(), Some("demo-user"));
    }

    #[test]
    fn test_rejects_token_signed_with_unknown_secret() {
        let config = config_with(&["stage-secret-key", "demo-secret-key"]);

        assert!(config
            .decode_claims(&token_signed_with("attacker-secret", "someone"))
            .is_none());
    }

    #[test]
    fn test_rejects_unsigned_and_malformed_tokens() {
        let config = config_with(&["stage-secret-key"]);

        assert!(config.decode_claims("not-a-jwt").is_none());
        // alg=none must not bypass verification
        assert!(config
            .decode_claims("eyJhbGciOiJub25lIn0.eyJ1c2VyX2lkIjoiaGFjayJ9.")
            .is_none());
    }

    #[test]
    fn test_single_secret_still_works() {
        let config = config_with(&["only-secret"]);

        let claims = config
            .decode_claims(&token_signed_with("only-secret", "u1"))
            .expect("token should verify");
        assert_eq!(claims.user_id("user_id").as_deref(), Some("u1"));
    }
}
