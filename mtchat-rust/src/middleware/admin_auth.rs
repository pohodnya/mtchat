//! Admin authentication middleware
//!
//! Validates admin tokens for Management API access.
//! Admin token is passed via `Authorization: Bearer <token>` header.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::env;

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
    let admin_token = env::var("ADMIN_API_TOKEN").ok();

    // If no admin token configured, allow all (dev mode)
    if admin_token.is_none() {
        return next.run(request).await;
    }

    let expected_token = admin_token.unwrap();

    // Get Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthError::unauthorized("Authorization header required")),
            )
                .into_response();
        }
        Some(header) => {
            // Expect "Bearer <token>" format
            if !header.starts_with("Bearer ") {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthError::unauthorized("Invalid authorization format. Use: Bearer <token>")),
                )
                    .into_response();
            }

            let token = &header[7..]; // Skip "Bearer "

            if token != expected_token {
                return (
                    StatusCode::FORBIDDEN,
                    Json(AuthError::forbidden("Invalid admin token")),
                )
                    .into_response();
            }
        }
    }

    // Token valid, proceed
    next.run(request).await
}

