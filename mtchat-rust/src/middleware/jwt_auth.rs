//! JWT Authentication Middleware for Chat API
//!
//! Validates JWT tokens and extracts the user identifier from a configurable
//! claim (default `sub`). When JWT auth is disabled, falls back to query
//! parameter extraction.

use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::decode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::config::JwtConfig;

/// JWT claims as a flexible map.
///
/// The token payload is deserialized as an arbitrary JSON object so that the
/// user identifier can be read from any configured claim name (`sub`,
/// `user_id`, `userId`, `id`, ...). String and numeric values are accepted —
/// numbers are stringified to keep MTChat's `String`-typed identifiers.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JwtClaims(pub Map<String, Value>);

impl JwtClaims {
    /// Extract the user identifier from the named claim.
    /// Returns `None` if the claim is missing or its value is not string/number.
    pub fn user_id(&self, claim: &str) -> Option<String> {
        match self.0.get(claim)? {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }
}

/// Middleware for JWT authentication on Chat API routes
///
/// If JWT auth is enabled:
/// - Validates the Bearer token from Authorization header
/// - Verifies the configured user-id claim is present and is a string/number
/// - Returns 401 if token is missing, invalid, or claim is absent
///
/// If JWT auth is disabled:
/// - Passes request through without validation
pub async fn jwt_auth(request: Request, next: Next) -> Response {
    // If JWT auth disabled - pass through
    let config = match JwtConfig::get() {
        Some(c) => c,
        None => return next.run(request).await,
    };

    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return (StatusCode::UNAUTHORIZED, "Authorization header required").into_response();
        }
    };

    // Validate signature and ensure the configured user-id claim exists
    match decode::<JwtClaims>(token, &config.decoding_key, &config.validation) {
        Ok(data) => {
            if data.claims.user_id(&config.user_id_claim).is_none() {
                tracing::debug!(
                    "JWT validation failed: missing or non-string claim '{}'",
                    config.user_id_claim
                );
                return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
            }
            next.run(request).await
        }
        Err(e) => {
            tracing::debug!("JWT validation failed: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
        }
    }
}

/// Extractor for getting user_id from JWT token or query parameter
///
/// When JWT auth is enabled:
/// - Extracts user_id from the configured JWT claim (default `sub`)
///
/// When JWT auth is disabled:
/// - Falls back to `user_id` or `sender_id` query parameter
pub struct JwtUserId(pub String);

#[allow(clippy::result_large_err)]
impl<S> FromRequestParts<S> for JwtUserId
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // If JWT auth disabled - fallback to query parameter
        let config = match JwtConfig::get() {
            Some(c) => c,
            None => return extract_user_id_from_query(parts),
        };

        // Extract token from Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = match auth_header {
            Some(h) if h.starts_with("Bearer ") => &h[7..],
            _ => {
                return Err((StatusCode::UNAUTHORIZED, "Missing token").into_response());
            }
        };

        // Decode and extract user_id from the configured claim
        match decode::<JwtClaims>(token, &config.decoding_key, &config.validation) {
            Ok(data) => match data.claims.user_id(&config.user_id_claim) {
                Some(id) => Ok(JwtUserId(id)),
                None => Err((StatusCode::UNAUTHORIZED, "Invalid token").into_response()),
            },
            Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid token").into_response()),
        }
    }
}

/// Extract user_id from query parameters (fallback when JWT disabled)
#[allow(clippy::result_large_err)]
fn extract_user_id_from_query(parts: &Parts) -> Result<JwtUserId, Response> {
    let query = parts.uri.query().unwrap_or("");

    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            if key == "user_id" || key == "sender_id" {
                // URL decode the value
                let decoded = urlencoding::decode(value).unwrap_or_else(|_| value.into());
                if decoded.is_empty() {
                    return Err(
                        (StatusCode::BAD_REQUEST, "user_id cannot be empty").into_response()
                    );
                }
                return Ok(JwtUserId(decoded.into_owned()));
            }
        }
    }

    Err((StatusCode::BAD_REQUEST, "user_id query parameter required").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header};
    use serde_json::json;

    fn build_token(payload: Value, secret: &str) -> String {
        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn permissive_validation() -> jsonwebtoken::Validation {
        let mut v = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        v.validate_exp = false;
        v.required_spec_claims.clear();
        v
    }

    #[test]
    fn test_valid_jwt_token() {
        let secret = "test-secret-key-32-characters-long";
        let user_id = "user-123";
        let token = build_token(json!({ "sub": user_id }), secret);

        let decoded = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &permissive_validation(),
        )
        .expect("Token should be valid");

        assert_eq!(decoded.claims.user_id("sub").as_deref(), Some(user_id));
    }

    #[test]
    fn test_invalid_signature() {
        let token = build_token(json!({ "sub": "user-456" }), "secret1");

        let result = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret("secret2".as_bytes()),
            &permissive_validation(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token_accepted() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let secret = "test-secret";
        let user_id = "user-789";
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 3600;
        let token = build_token(json!({ "sub": user_id, "exp": exp }), secret);

        let result = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &permissive_validation(),
        );

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().claims.user_id("sub").as_deref(),
            Some(user_id)
        );
    }

    #[test]
    fn test_custom_claim_name() {
        let secret = "test-secret-key-32-characters-long";
        let user_id = "794a8653-53ce-4b32-865a-4118a8038e3f";
        let token = build_token(json!({ "user_id": user_id }), secret);

        let decoded = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &permissive_validation(),
        )
        .expect("Token should be valid");

        assert_eq!(decoded.claims.user_id("user_id").as_deref(), Some(user_id));
        // Default `sub` lookup must miss when ID lives under a custom claim.
        assert!(decoded.claims.user_id("sub").is_none());
    }

    #[test]
    fn test_numeric_claim_stringified() {
        let secret = "test-secret-key-32-characters-long";
        let token = build_token(json!({ "id": 42 }), secret);

        let decoded = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &permissive_validation(),
        )
        .expect("Token should be valid");

        assert_eq!(decoded.claims.user_id("id").as_deref(), Some("42"));
    }

    #[test]
    fn test_missing_claim_returns_none() {
        let secret = "test-secret-key-32-characters-long";
        let token = build_token(json!({ "sub": "user-1" }), secret);

        let decoded = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &permissive_validation(),
        )
        .expect("Token should decode");

        assert!(decoded.claims.user_id("user_id").is_none());
    }
}
