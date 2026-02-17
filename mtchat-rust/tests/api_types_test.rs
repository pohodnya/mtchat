//! Tests for API types: error responses, response wrappers, and DTO serialization

use multitenancy_chat_api::api::{ApiError, ApiResponse, ErrorResponse, ErrorBody};
use axum::response::IntoResponse;
use axum::http::StatusCode;

// ============ ApiError ============

#[tokio::test]
async fn test_api_error_not_found() {
    let error = ApiError::NotFound("Dialog not found".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "NOT_FOUND");
    assert_eq!(json["error"]["message"], "Dialog not found");
}

#[tokio::test]
async fn test_api_error_bad_request() {
    let error = ApiError::BadRequest("Invalid input".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "BAD_REQUEST");
    assert_eq!(json["error"]["message"], "Invalid input");
}

#[tokio::test]
async fn test_api_error_forbidden() {
    let error = ApiError::Forbidden("Not a participant".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "FORBIDDEN");
    assert_eq!(json["error"]["message"], "Not a participant");
}

#[tokio::test]
async fn test_api_error_internal_hides_details() {
    let error = ApiError::Internal("connection refused: db pool exhausted".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "INTERNAL_ERROR");
    // Should NOT expose internal details
    assert_eq!(json["error"]["message"], "Internal server error");
    assert!(!json["error"]["message"].as_str().unwrap().contains("connection refused"));
}

#[tokio::test]
async fn test_api_error_from_sqlx_error() {
    // Simulate a sqlx error by creating an ApiError::Internal
    // (sqlx::Error -> ApiError::Internal via From impl)
    let error = ApiError::Internal("sqlx: no rows returned".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============ ApiResponse ============

#[test]
fn test_api_response_serialization() {
    let response = ApiResponse { data: "hello" };
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["data"], "hello");
}

#[test]
fn test_api_response_with_struct() {
    #[derive(serde::Serialize)]
    struct Item {
        id: u32,
        name: String,
    }

    let response = ApiResponse {
        data: Item { id: 1, name: "test".into() },
    };
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["data"]["id"], 1);
    assert_eq!(json["data"]["name"], "test");
}

#[test]
fn test_api_response_with_vec() {
    let response = ApiResponse { data: vec![1, 2, 3] };
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["data"], serde_json::json!([1, 2, 3]));
}

// ============ ErrorResponse ============

#[test]
fn test_error_response_serialization() {
    let response = ErrorResponse {
        error: ErrorBody {
            code: "NOT_FOUND".to_string(),
            message: "Dialog not found".to_string(),
        },
    };
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["error"]["code"], "NOT_FOUND");
    assert_eq!(json["error"]["message"], "Dialog not found");
}
