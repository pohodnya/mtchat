//! Integration tests for Chat API
//!
//! Tests user-facing Chat API endpoints with scope-based access control.
//!
//! These tests require a running server. Run with:
//! ```
//! cargo test --test chat_api_test -- --ignored
//! ```

use base64::Engine;
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

fn encode_scope_config(tenant_uid: Uuid, scope_level1: &[&str], scope_level2: &[&str]) -> String {
    let config = json!({
        "tenant_uid": tenant_uid,
        "scope_level1": scope_level1,
        "scope_level2": scope_level2
    });
    base64::engine::general_purpose::STANDARD.encode(config.to_string().as_bytes())
}

// Helper to create a dialog for tests
async fn create_test_dialog(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    object_id: Uuid,
    object_type: &str,
    participants: &[Uuid],
    tenant_uid: Uuid,
    scope_level1: &[&str],
    scope_level2: &[&str],
) -> String {
    let resp = client
        .post(&format!("{}/api/v1/management/dialogs", base_url))
        .header("Authorization", auth_header)
        .json(&json!({
            "object_id": object_id,
            "object_type": object_type,
            "title": "Test Dialog",
            "participants": participants,
            "access_scopes": [{
                "tenant_uid": tenant_uid,
                "scope_level1": scope_level1,
                "scope_level2": scope_level2
            }]
        }))
        .send()
        .await
        .expect("Create dialog failed");

    let body: Value = resp.json().await.unwrap();
    body["data"]["id"].as_str().unwrap().to_string()
}

// Helper to delete a dialog
async fn delete_test_dialog(client: &Client, base_url: &str, auth_header: &str, dialog_id: &str) {
    client
        .delete(&format!(
            "{}/api/v1/management/dialogs/{}",
            base_url, dialog_id
        ))
        .header("Authorization", auth_header)
        .send()
        .await
        .ok();
}

// ============ List Dialogs Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_participating_dialogs() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog with user as participant
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "test",
        &[user_id],
        tenant_uid,
        &["dept"],
        &["role"],
    )
    .await;

    // List participating dialogs
    let resp = client
        .get(&format!(
            "{}/api/v1/dialogs?type=participating&user_id={}",
            base_url, user_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let dialogs = body["data"].as_array().unwrap();
    assert!(dialogs.iter().any(|d| d["id"] == dialog_id));

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_available_dialogs() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog without user as participant, but matching scope
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "tender",
        &[], // No direct participants
        tenant_uid,
        &["sales", "logistics"],
        &["manager", "admin"],
    )
    .await;

    // List available dialogs with matching scope
    let scope_header = encode_scope_config(tenant_uid, &["sales"], &["manager"]);
    let resp = client
        .get(&format!(
            "{}/api/v1/dialogs?type=available&user_id={}",
            base_url, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let dialogs = body["data"].as_array().unwrap();
    assert!(
        dialogs.iter().any(|d| d["id"] == dialog_id),
        "Dialog should be available"
    );

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_available_requires_scope_header() {
    let client = Client::new();
    let base_url = get_base_url();
    let user_id = Uuid::new_v4();

    // Request without X-Scope-Config header
    let resp = client
        .get(&format!(
            "{}/api/v1/dialogs?type=available&user_id={}",
            base_url, user_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_available_dialogs_scope_mismatch() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let dialog_tenant = Uuid::new_v4();
    let user_tenant = Uuid::new_v4(); // Different tenant
    let object_id = Uuid::new_v4();

    // Create dialog with specific tenant
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[],
        dialog_tenant,
        &["dept_a"],
        &["role_a"],
    )
    .await;

    // List with different tenant - should not see dialog
    let scope_header = encode_scope_config(user_tenant, &["dept_a"], &["role_a"]);
    let resp = client
        .get(&format!(
            "{}/api/v1/dialogs?type=available&user_id={}",
            base_url, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    let body: Value = resp.json().await.unwrap();
    let dialogs = body["data"].as_array().unwrap();
    assert!(
        !dialogs.iter().any(|d| d["id"] == dialog_id),
        "Dialog should NOT be visible to different tenant"
    );

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

// ============ Get Dialog by Object Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_get_dialog_by_object_as_participant() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog with user
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "route",
        &[user_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    // Get by object
    let resp = client
        .get(&format!(
            "{}/api/v1/dialogs/by-object/route/{}?user_id={}",
            base_url, object_id, user_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["id"], dialog_id);
    assert_eq!(body["data"]["i_am_participant"], true);
    assert_eq!(body["data"]["can_join"], false);

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_get_dialog_by_object_as_potential() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog without user
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "delivery",
        &[],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    // Get by object with matching scope
    let scope_header = encode_scope_config(tenant_uid, &["logistics"], &["driver"]);
    let resp = client
        .get(&format!(
            "{}/api/v1/dialogs/by-object/delivery/{}?user_id={}",
            base_url, object_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["id"], dialog_id);
    assert_eq!(body["data"]["i_am_participant"], false);
    assert_eq!(body["data"]["can_join"], true);

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_get_dialog_by_object_forbidden() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let dialog_tenant = Uuid::new_v4();
    let user_tenant = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "invoice",
        &[],
        dialog_tenant,
        &["finance"],
        &["accountant"],
    )
    .await;

    // Try to access with non-matching scope
    let scope_header = encode_scope_config(user_tenant, &["sales"], &["manager"]);
    let resp = client
        .get(&format!(
            "{}/api/v1/dialogs/by-object/invoice/{}?user_id={}",
            base_url, object_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

// ============ Join/Leave Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_join_dialog() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog without user
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "tender",
        &[],
        tenant_uid,
        &["procurement"],
        &["buyer"],
    )
    .await;

    // Join with matching scope
    let scope_header = encode_scope_config(tenant_uid, &["procurement"], &["buyer"]);
    let resp = client
        .post(&format!(
            "{}/api/v1/dialogs/{}/join?user_id={}",
            base_url, dialog_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "joined");

    // Verify now in participating list
    let list_resp = client
        .get(&format!(
            "{}/api/v1/dialogs?type=participating&user_id={}",
            base_url, user_id
        ))
        .send()
        .await
        .unwrap();

    let list_body: Value = list_resp.json().await.unwrap();
    let dialogs = list_body["data"].as_array().unwrap();
    assert!(dialogs.iter().any(|d| d["id"] == dialog_id));

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_join_dialog_forbidden() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let dialog_tenant = Uuid::new_v4();
    let user_tenant = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[],
        dialog_tenant,
        &["sales"],
        &["seller"],
    )
    .await;

    // Try to join with non-matching scope
    let scope_header = encode_scope_config(user_tenant, &["support"], &["agent"]);
    let resp = client
        .post(&format!(
            "{}/api/v1/dialogs/{}/join?user_id={}",
            base_url, dialog_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_join_already_participant() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog with user already in it
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "project",
        &[user_id],
        tenant_uid,
        &["dev"],
        &["engineer"],
    )
    .await;

    // Try to join again
    let scope_header = encode_scope_config(tenant_uid, &["dev"], &["engineer"]);
    let resp = client
        .post(&format!(
            "{}/api/v1/dialogs/{}/join?user_id={}",
            base_url, dialog_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_leave_dialog() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Create dialog with user
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "meeting",
        &[user_id],
        tenant_uid,
        &["team"],
        &["member"],
    )
    .await;

    // Leave
    let resp = client
        .post(&format!(
            "{}/api/v1/dialogs/{}/leave?user_id={}",
            base_url, dialog_id, user_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "left");

    // Verify no longer in participating list
    let list_resp = client
        .get(&format!(
            "{}/api/v1/dialogs?type=participating&user_id={}",
            base_url, user_id
        ))
        .send()
        .await
        .unwrap();

    let list_body: Value = list_resp.json().await.unwrap();
    let dialogs = list_body["data"].as_array().unwrap();
    assert!(!dialogs.iter().any(|d| d["id"] == dialog_id));

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

// ============ User ID Tests ============

#[tokio::test]
#[ignore] // Requires running server
async fn test_missing_user_id_returns_error() {
    let client = Client::new();
    let base_url = get_base_url();

    let resp = client
        .get(&format!("{}/api/v1/dialogs?type=participating", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
