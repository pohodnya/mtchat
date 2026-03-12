use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::decode;
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::middleware::JwtClaims;
use crate::ws;

use super::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // Extract user_id - from JWT token if enabled, otherwise from query param
    let user_id = match extract_user_id(&params) {
        Ok(id) => id,
        Err(response) => return response,
    };

    ws.on_upgrade(move |socket| {
        ws::handle_socket(
            socket,
            state.connections,
            user_id,
            state.presence,
            state.participants,
        )
    })
    .into_response()
}

/// Extract user_id from JWT token (if enabled) or query parameter
#[allow(clippy::result_large_err)]
fn extract_user_id(params: &HashMap<String, String>) -> Result<Uuid, Response> {
    // If JWT auth is enabled, validate token and extract user_id from claims
    if let Some(config) = JwtConfig::get() {
        let token = params.get("token").ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "token query parameter required").into_response()
        })?;

        let token_data =
            decode::<JwtClaims>(token, &config.decoding_key, &config.validation).map_err(|e| {
                tracing::debug!("WebSocket JWT validation failed: {}", e);
                (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
            })?;

        return Ok(token_data.claims.sub);
    }

    // JWT disabled - use user_id query parameter
    params
        .get("user_id")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "Missing or invalid user_id query parameter",
            )
                .into_response()
        })
}
