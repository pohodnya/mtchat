//! Admin authentication middleware
//!
//! Validates admin tokens for Management API access.
//! Admin token is passed via `Authorization: Bearer <token>` header.
//!
//! Security:
//! - Token is read once at startup (not per-request)
//! - Comparison uses SHA-256 digest to prevent timing attacks

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

/// Cached admin token digest, computed once at startup.
/// None = no token configured (dev mode, all requests allowed).
static ADMIN_TOKEN_DIGEST: OnceLock<Option<[u8; 32]>> = OnceLock::new();

/// Initialize the admin token from environment.
/// Must be called once during server startup.
pub fn init_admin_token() {
    ADMIN_TOKEN_DIGEST.get_or_init(|| match std::env::var("ADMIN_API_TOKEN") {
        Ok(token) if !token.is_empty() => {
            tracing::info!("Admin API token configured");
            Some(sha256_digest(token.as_bytes()))
        }
        _ => {
            tracing::warn!("ADMIN_API_TOKEN not set — Management API is unprotected (dev mode)");
            None
        }
    });
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Constant-time token comparison via SHA-256 digest.
/// Comparing fixed-length hash digests avoids timing side-channels
/// that exist in variable-length string comparison.
fn verify_token(provided: &str) -> bool {
    let expected = match ADMIN_TOKEN_DIGEST.get() {
        Some(Some(digest)) => digest,
        _ => return false,
    };
    let provided_digest = sha256_digest(provided.as_bytes());
    // Fixed-length array comparison — compiler emits constant-time code
    // for [u8; 32] equality (no early exit on mismatch).
    *expected == provided_digest
}

/// Error response for auth failures
#[derive(Debug, Serialize)]
struct AuthError {
    error: AuthErrorBody,
}

#[derive(Debug, Serialize)]
struct AuthErrorBody {
    code: String,
    message: String,
}

impl AuthError {
    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            error: AuthErrorBody {
                code: "UNAUTHORIZED".to_string(),
                message: message.into(),
            },
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            error: AuthErrorBody {
                code: "FORBIDDEN".to_string(),
                message: message.into(),
            },
        }
    }
}

/// Admin authentication middleware
///
/// Checks for valid admin token in Authorization header.
/// Token is configured via `ADMIN_API_TOKEN` environment variable.
///
/// If no token is configured, all requests are allowed (development mode).
pub async fn admin_auth(request: Request, next: Next) -> Response {
    let token_configured = matches!(ADMIN_TOKEN_DIGEST.get(), Some(Some(_)));

    // If no admin token configured, allow all (dev mode)
    if !token_configured {
        return next.run(request).await;
    }

    // Get Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        None => (
            StatusCode::UNAUTHORIZED,
            Json(AuthError::unauthorized("Authorization header required")),
        )
            .into_response(),
        Some(header) => {
            // Expect "Bearer <token>" format
            if !header.starts_with("Bearer ") {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthError::unauthorized(
                        "Invalid authorization format. Use: Bearer <token>",
                    )),
                )
                    .into_response();
            }

            let token = &header[7..]; // Skip "Bearer "

            if !verify_token(token) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(AuthError::forbidden("Invalid admin token")),
                )
                    .into_response();
            }

            // Token valid, proceed
            next.run(request).await
        }
    }
}
