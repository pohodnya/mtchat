//! Scope config extractor for Chat API
//!
//! Extracts user scope configuration from X-Scope-Config header.
//! The header value is base64-encoded JSON:
//! ```json
//! {
//!   "scope_level0": ["tenant-a", "tenant-b"],
//!   "scope_level1": ["dept_a", "dept_b"],
//!   "scope_level2": ["manager", "admin"]
//! }
//! ```

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine;
use serde::{Deserialize, Serialize};

/// User scope configuration extracted from X-Scope-Config header
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeConfig {
    /// Top scope level (e.g., tenants/organizations)
    #[serde(default)]
    pub scope_level0: Vec<String>,
    #[serde(default)]
    pub scope_level1: Vec<String>,
    #[serde(default)]
    pub scope_level2: Vec<String>,
}

/// Error response for scope config extraction failures
#[derive(Debug, Serialize)]
struct ScopeError {
    error: ScopeErrorBody,
}

#[derive(Debug, Serialize)]
struct ScopeErrorBody {
    code: String,
    message: String,
}

impl ScopeError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            error: ScopeErrorBody {
                code: "BAD_REQUEST".to_string(),
                message: message.into(),
            },
        }
    }
}

/// Extractor that requires X-Scope-Config header
///
/// Use this when scope config is mandatory:
/// ```ignore
/// async fn handler(scope: ScopeConfig, ...) { ... }
/// ```
impl<S> FromRequestParts<S> for ScopeConfig
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get("X-Scope-Config")
            .and_then(|v| v.to_str().ok());

        match header_value {
            None => Err((
                StatusCode::BAD_REQUEST,
                Json(ScopeError::bad_request("X-Scope-Config header required")),
            )
                .into_response()),
            Some(encoded) => {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(encoded)
                    .map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ScopeError::bad_request(
                                "Invalid X-Scope-Config: not valid base64",
                            )),
                        )
                            .into_response()
                    })?;

                serde_json::from_slice(&decoded).map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ScopeError::bad_request(
                            "Invalid X-Scope-Config: not valid JSON",
                        )),
                    )
                        .into_response()
                })
            }
        }
    }
}

/// Optional scope config extractor
///
/// Use this when scope config is optional:
/// ```ignore
/// async fn handler(scope: OptionalScopeConfig, ...) { ... }
/// ```
#[derive(Debug, Clone)]
pub struct OptionalScopeConfig(pub Option<ScopeConfig>);

impl<S> FromRequestParts<S> for OptionalScopeConfig
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get("X-Scope-Config")
            .and_then(|v| v.to_str().ok());

        match header_value {
            None => Ok(OptionalScopeConfig(None)),
            Some(encoded) => {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(encoded)
                    .map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ScopeError::bad_request(
                                "Invalid X-Scope-Config: not valid base64",
                            )),
                        )
                            .into_response()
                    })?;

                let config: ScopeConfig = serde_json::from_slice(&decoded).map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ScopeError::bad_request(
                            "Invalid X-Scope-Config: not valid JSON",
                        )),
                    )
                        .into_response()
                })?;

                Ok(OptionalScopeConfig(Some(config)))
            }
        }
    }
}

/// User ID extractor from query parameters
///
/// Extracts user_id from ?user_id= or ?sender_id= query parameter
#[derive(Debug, Clone)]
pub struct UserId(pub String);

impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");

        // Simple query parsing
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                if key == "user_id" || key == "sender_id" {
                    // URL decode the value
                    let decoded = urlencoding::decode(value).unwrap_or_else(|_| value.into());
                    if decoded.is_empty() {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(ScopeError::bad_request("user_id cannot be empty")),
                        )
                            .into_response());
                    }
                    return Ok(UserId(decoded.into_owned()));
                }
            }
        }

        Err((
            StatusCode::BAD_REQUEST,
            Json(ScopeError::bad_request("user_id query parameter required")),
        )
            .into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_config_deserialize() {
        let json =
            r#"{"scope_level0": ["tenant-123"], "scope_level1": ["a"], "scope_level2": ["b"]}"#;
        let config: ScopeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.scope_level0, vec!["tenant-123"]);
        assert_eq!(config.scope_level1, vec!["a"]);
        assert_eq!(config.scope_level2, vec!["b"]);
    }

    #[test]
    fn test_scope_config_defaults() {
        let json = r#"{}"#;
        let config: ScopeConfig = serde_json::from_str(json).unwrap();
        assert!(config.scope_level0.is_empty());
        assert!(config.scope_level1.is_empty());
        assert!(config.scope_level2.is_empty());
    }

    #[test]
    fn test_scope_config_multi_tenant() {
        // Multiple tenants in scope_level0
        let json = r#"{"scope_level0": ["tenant-a", "tenant-b"], "scope_level1": ["dept"]}"#;
        let config: ScopeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.scope_level0, vec!["tenant-a", "tenant-b"]);
        assert_eq!(config.scope_level1, vec!["dept"]);
        assert!(config.scope_level2.is_empty());
    }
}
