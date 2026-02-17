use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;

use crate::ws;

use super::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let user_id = match params.get("user_id").and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Missing or invalid user_id query parameter",
            ).into_response();
        }
    };

    ws.on_upgrade(move |socket| {
        ws::handle_socket(
            socket,
            state.connections,
            user_id,
            state.presence,
            state.participants,
        )
    }).into_response()
}
