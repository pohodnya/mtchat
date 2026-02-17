//! Integration tests for Management API
//!
//! Tests admin authentication and all Management API endpoints.
//!
//! These tests require a running server. Run with:
//! ```
//! cargo test --test management_api_test -- --ignored
//! ```

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::env;
use uuid::Uuid;

fn get_base_url() -> String {
    env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn get_admin_token() -> Option<String> {
    env::var("ADMIN_API_TOKEN").ok()
}

// ============ Admin Auth Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_admin_auth_rejects_missing_header() {
    let client = Client::new();
    let base_url = get_base_url();

    // Set admin token to enable auth (skip if not configured)
    if get_admin_token().is_none() {
        println!("Skipping: ADMIN_API_TOKEN not configured (dev mode allows all)");
        return;
    }

    let resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .json(&json!({
            "object_id": Uuid::new_v4(),
            "object_type": "test",
            "participants": []
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_admin_auth_rejects_invalid_token() {
    let client = Client::new();
    let base_url = get_base_url();

    if get_admin_token().is_none() {
        println!("Skipping: ADMIN_API_TOKEN not configured");
        return;
    }

    let resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", "Bearer invalid-token")
        .json(&json!({
            "object_id": Uuid::new_v4(),
            "object_type": "test",
            "participants": []
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_admin_auth_accepts_valid_token() {
    let client = Client::new();
    let base_url = get_base_url();

    let token = match get_admin_token() {
        Some(t) => t,
        None => {
            println!("Skipping: ADMIN_API_TOKEN not configured");
            return;
        }
    };

    let resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "object_id": Uuid::new_v4(),
            "object_type": "test",
            "participants": []
        }))
        .send()
        .await
        .expect("Request failed");

    // Should not be auth error (201 or other success/client error)
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(resp.status(), StatusCode::FORBIDDEN);
}

// ============ Dialog CRUD Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_create_dialog_with_participants() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let object_id = Uuid::new_v4();
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();

    let resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", &auth_header)
        .json(&json!({
            "object_id": object_id,
            "object_type": "tender",
            "title": "Test Dialog",
            "participants": [user1, user2],
            "access_scopes": [{
                "tenant_uid": tenant_uid,
                "scope_level1": ["dept_a", "dept_b"],
                "scope_level2": ["manager", "admin"]
            }]
        }))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = resp.json().await.unwrap();
    let dialog_id = body["data"]["id"].as_str().unwrap();

    assert_eq!(body["data"]["object_id"], object_id.to_string());
    assert_eq!(body["data"]["object_type"], "tender");
    assert_eq!(body["data"]["title"], "Test Dialog");

    // Cleanup
    client
        .delete(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_get_dialog_with_details() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let object_id = Uuid::new_v4();
    let user1 = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();

    // Create dialog
    let create_resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", &auth_header)
        .json(&json!({
            "object_id": object_id,
            "object_type": "order",
            "title": "Order Chat",
            "participants": [user1],
            "access_scopes": [{
                "tenant_uid": tenant_uid,
                "scope_level1": ["sales"],
                "scope_level2": ["viewer"]
            }]
        }))
        .send()
        .await
        .unwrap();

    let create_body: Value = create_resp.json().await.unwrap();
    let dialog_id = create_body["data"]["id"].as_str().unwrap();

    // Get dialog with details
    let get_resp = client
        .get(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::OK);

    let body: Value = get_resp.json().await.unwrap();
    assert_eq!(body["data"]["object_type"], "order");
    assert!(body["data"]["participants"].as_array().unwrap().len() >= 1);
    assert!(body["data"]["access_scopes"].as_array().unwrap().len() >= 1);

    // Cleanup
    client
        .delete(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_delete_dialog() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    // Create dialog
    let create_resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", &auth_header)
        .json(&json!({
            "object_id": Uuid::new_v4(),
            "object_type": "test",
            "participants": []
        }))
        .send()
        .await
        .unwrap();

    let create_body: Value = create_resp.json().await.unwrap();
    let dialog_id = create_body["data"]["id"].as_str().unwrap();

    // Delete dialog
    let delete_resp = client
        .delete(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();

    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let get_resp = client
        .get(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

// ============ Participant Management Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_add_and_remove_participant() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    // Create dialog
    let create_resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", &auth_header)
        .json(&json!({
            "object_id": Uuid::new_v4(),
            "object_type": "test",
            "participants": []
        }))
        .send()
        .await
        .unwrap();

    let create_body: Value = create_resp.json().await.unwrap();
    let dialog_id = create_body["data"]["id"].as_str().unwrap();
    let new_user = Uuid::new_v4();

    // Add participant
    let add_resp = client
        .post(&format!(
            "{}/api/v1/management/dialogs/{}/participants",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .json(&json!({ "user_id": new_user }))
        .send()
        .await
        .unwrap();

    assert_eq!(add_resp.status(), StatusCode::CREATED);

    // Verify added
    let get_resp = client
        .get(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();

    let body: Value = get_resp.json().await.unwrap();
    let participants = body["data"]["participants"].as_array().unwrap();
    assert!(participants
        .iter()
        .any(|p| p["user_id"] == new_user.to_string()));

    // Remove participant
    let remove_resp = client
        .delete(&format!(
            "{}/api/v1/management/dialogs/{}/participants/{}",
            base_url, dialog_id, new_user
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();

    assert_eq!(remove_resp.status(), StatusCode::NO_CONTENT);

    // Cleanup
    client
        .delete(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();
}

// ============ Access Scopes Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_update_access_scopes() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    // Create dialog with initial scopes
    let create_resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", &auth_header)
        .json(&json!({
            "object_id": Uuid::new_v4(),
            "object_type": "test",
            "participants": [],
            "access_scopes": [{
                "tenant_uid": tenant1,
                "scope_level1": ["old_dept"],
                "scope_level2": ["old_role"]
            }]
        }))
        .send()
        .await
        .unwrap();

    let create_body: Value = create_resp.json().await.unwrap();
    let dialog_id = create_body["data"]["id"].as_str().unwrap();

    // Update access scopes (replace all)
    let update_resp = client
        .put(&format!(
            "{}/api/v1/management/dialogs/{}/access-scopes",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .json(&json!({
            "access_scopes": [
                {
                    "tenant_uid": tenant2,
                    "scope_level1": ["new_dept1", "new_dept2"],
                    "scope_level2": ["manager", "admin"]
                }
            ]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: Value = update_resp.json().await.unwrap();
    let scopes = update_body["data"].as_array().unwrap();
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0]["tenant_uid"], tenant2.to_string());

    // Verify via GET
    let get_resp = client
        .get(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();

    let body: Value = get_resp.json().await.unwrap();
    let access_scopes = body["data"]["access_scopes"].as_array().unwrap();
    assert_eq!(access_scopes.len(), 1);
    assert_eq!(access_scopes[0]["tenant_uid"], tenant2.to_string());

    // Old scope should be gone
    assert!(!access_scopes
        .iter()
        .any(|s| s["tenant_uid"] == tenant1.to_string()));

    // Cleanup
    client
        .delete(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();
}

// ============ Error Handling Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_get_nonexistent_dialog_returns_404() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let fake_id = Uuid::new_v4();

    let resp = client
        .get(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, fake_id
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_add_participant_to_nonexistent_dialog() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let fake_dialog_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let resp = client
        .post(&format!(
            "{}/api/v1/management/dialogs/{}/participants",
            base_url, fake_dialog_id
        ))
        .header("Authorization", &auth_header)
        .json(&json!({ "user_id": user_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
