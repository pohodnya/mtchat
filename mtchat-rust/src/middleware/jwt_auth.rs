//! JWT Authentication Middleware for Chat API
//!
//! Validates JWT tokens and extracts user_id from the `sub` claim.
//! When JWT auth is disabled, falls back to query parameter extraction.

use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::decode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::JwtConfig;

/// JWT claims structure
/// Only `sub` (subject = user_id) is used, other claims are ignored
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID (subject claim)
    pub sub: Uuid,
}

/// Middleware for JWT authentication on Chat API routes
///
/// If JWT auth is enabled:
/// - Validates the Bearer token from Authorization header
/// - Returns 401 if token is missing or invalid
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

    // Validate token signature
    match decode::<JwtClaims>(token, &config.decoding_key, &config.validation) {
        Ok(_) => next.run(request).await,
        Err(e) => {
            tracing::debug!("JWT validation failed: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
        }
    }
}

/// Extractor for getting user_id from JWT token or query parameter
///
/// When JWT auth is enabled:
/// - Extracts user_id from JWT `sub` claim
///
/// When JWT auth is disabled:
/// - Falls back to `user_id` or `sender_id` query parameter
pub struct JwtUserId(pub Uuid);

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

        // Decode and extract user_id from claims
        match decode::<JwtClaims>(token, &config.decoding_key, &config.validation) {
            Ok(data) => Ok(JwtUserId(data.claims.sub)),
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
                return value.parse().map(JwtUserId).map_err(|_| {
                    (StatusCode::BAD_REQUEST, "Invalid user_id format").into_response()
                });
            }
        }
    }

    Err((StatusCode::BAD_REQUEST, "user_id query parameter required").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header};

    fn create_test_token(user_id: Uuid, secret: &str) -> String {
        let claims = JwtClaims { sub: user_id };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_valid_jwt_token() {
        let secret = "test-secret-key-32-characters-long";
        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, secret);

        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        let decoded = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .expect("Token should be valid");

        assert_eq!(decoded.claims.sub, user_id);
    }

    #[test]
    fn test_invalid_signature() {
        let user_id = Uuid::new_v4();
        let token = create_test_token(user_id, "secret1");

        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        let result = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret("secret2".as_bytes()),
            &validation,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token_accepted() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let secret = "test-secret";
        let user_id = Uuid::new_v4();

        // Create expired token
        #[derive(Serialize)]
        struct ExpiredClaims {
            sub: Uuid,
            exp: u64,
        }

        let claims = ExpiredClaims {
            sub: user_id,
            // Expired 1 hour ago
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 3600,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // With validate_exp = false, expired token should be accepted
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        let result = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap().claims.sub, user_id);
    }
}
