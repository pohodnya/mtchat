//! Rate limiting middleware
//!
//! Uses governor crate for token bucket rate limiting.
//! Returns 429 Too Many Requests when limit is exceeded.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    RateLimiter,
};
use serde::Serialize;
use std::sync::Arc;

/// Shared rate limiter type
pub type SharedRateLimiter =
    Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>;

/// Error response for rate limit exceeded
#[derive(Debug, Serialize)]
struct RateLimitError {
    error: RateLimitErrorBody,
}

#[derive(Debug, Serialize)]
struct RateLimitErrorBody {
    code: &'static str,
    message: &'static str,
}

impl RateLimitError {
    fn too_many_requests() -> Self {
        Self {
            error: RateLimitErrorBody {
                code: "RATE_LIMIT_EXCEEDED",
                message: "Too many requests. Please slow down.",
            },
        }
    }
}

/// Rate limiting middleware
///
/// Checks if request is allowed by the rate limiter.
/// Returns 429 Too Many Requests if limit exceeded.
pub async fn rate_limit(
    request: Request,
    next: Next,
    limiter: Option<SharedRateLimiter>,
) -> Response {
    // If no limiter configured, allow all requests
    let Some(limiter) = limiter else {
        return next.run(request).await;
    };

    // Check if request is allowed
    match limiter.check() {
        Ok(_) => next.run(request).await,
        Err(_) => {
            tracing::warn!("Rate limit exceeded");
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(RateLimitError::too_many_requests()),
            )
                .into_response()
        }
    }
}
