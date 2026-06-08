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
#[allow(clippy::too_many_arguments)]
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
        .post(format!("{}/api/v1/management/dialogs", base_url))
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

// Helper to archive a dialog for a given user
async fn archive_test_dialog(client: &Client, base_url: &str, dialog_id: &str, user_id: Uuid) {
    client
        .post(format!(
            "{}/api/v1/dialogs/{}/archive?user_id={}",
            base_url, dialog_id, user_id
        ))
        .send()
        .await
        .expect("Archive dialog failed");
}

// Helper to delete a dialog
async fn delete_test_dialog(client: &Client, base_url: &str, auth_header: &str, dialog_id: &str) {
    client
        .delete(format!(
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
        .get(format!(
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
        .get(format!(
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
        .get(format!(
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
        .get(format!(
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
        .get(format!(
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
        .get(format!(
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
        .get(format!(
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

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_dialogs_by_object() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Two dialogs for the same object: one where the user is a participant,
    // one the user can only join via scope.
    let participating_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[user_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;
    let available_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    // List all dialogs for the object with matching scope
    let scope_header = encode_scope_config(tenant_uid, &["logistics"], &["driver"]);
    let resp = client
        .get(format!(
            "{}/api/v1/dialogs/by-object/order/{}/list?user_id={}",
            base_url, object_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let dialogs = body["data"].as_array().unwrap();

    // Both dialogs are returned
    assert!(dialogs.iter().any(|d| d["id"] == participating_id));
    assert!(dialogs.iter().any(|d| d["id"] == available_id));

    // Participant dialog: i_am_participant=true, can_join=false
    let participating = dialogs
        .iter()
        .find(|d| d["id"] == participating_id)
        .unwrap();
    assert_eq!(participating["i_am_participant"], true);
    assert_eq!(participating["can_join"], false);

    // Available dialog: i_am_participant=false, can_join=true
    let available = dialogs.iter().find(|d| d["id"] == available_id).unwrap();
    assert_eq!(available["i_am_participant"], false);
    assert_eq!(available["can_join"], true);

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &participating_id).await;
    delete_test_dialog(&client, &base_url, &auth_header, &available_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_dialogs_by_object_archived_filter() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Three dialogs for the same object:
    // - active_id:   user is a participant, NOT archived
    // - archived_id: user is a participant, archived for this user
    // - available_id: user is not a participant, can join via scope
    let active_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[user_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;
    let archived_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[user_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;
    let available_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    archive_test_dialog(&client, &base_url, &archived_id, user_id).await;

    let scope_header = encode_scope_config(tenant_uid, &["logistics"], &["driver"]);

    let list = |archived: Option<bool>| {
        let client = client.clone();
        let base_url = base_url.clone();
        let scope_header = scope_header.clone();
        async move {
            let mut url = format!(
                "{}/api/v1/dialogs/by-object/order/{}/list?user_id={}",
                base_url, object_id, user_id
            );
            if let Some(a) = archived {
                url.push_str(&format!("&archived={}", a));
            }
            let resp = client
                .get(url)
                .header("X-Scope-Config", &scope_header)
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body: Value = resp.json().await.unwrap();
            body["data"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| d["id"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        }
    };

    // Omitted: all accessible dialogs returned (current behavior).
    let all = list(None).await;
    assert!(all.contains(&active_id), "omitted should include active");
    assert!(
        all.contains(&archived_id),
        "omitted should include archived"
    );
    assert!(
        all.contains(&available_id),
        "omitted should include available"
    );

    // archived=true: only archived participant dialog, plus all potential.
    let only_archived = list(Some(true)).await;
    assert!(
        !only_archived.contains(&active_id),
        "archived=true should exclude active participant dialog"
    );
    assert!(
        only_archived.contains(&archived_id),
        "archived=true should include archived participant dialog"
    );
    assert!(
        only_archived.contains(&available_id),
        "archived=true should still include potential dialogs"
    );

    // archived=false: only active participant dialog, plus all potential.
    let only_active = list(Some(false)).await;
    assert!(
        only_active.contains(&active_id),
        "archived=false should include active participant dialog"
    );
    assert!(
        !only_active.contains(&archived_id),
        "archived=false should exclude archived participant dialog"
    );
    assert!(
        only_active.contains(&available_id),
        "archived=false should still include potential dialogs"
    );

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &active_id).await;
    delete_test_dialog(&client, &base_url, &auth_header, &archived_id).await;
    delete_test_dialog(&client, &base_url, &auth_header, &available_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_dialogs_by_object_type_filter() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // One dialog the user participates in, one only joinable via scope.
    let participating_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[user_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;
    let available_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    let scope_header = encode_scope_config(tenant_uid, &["logistics"], &["driver"]);

    let list = |type_filter: Option<&'static str>| {
        let client = client.clone();
        let base_url = base_url.clone();
        let scope_header = scope_header.clone();
        async move {
            let mut url = format!(
                "{}/api/v1/dialogs/by-object/order/{}/list?user_id={}",
                base_url, object_id, user_id
            );
            if let Some(t) = type_filter {
                url.push_str(&format!("&type={}", t));
            }
            let resp = client
                .get(url)
                .header("X-Scope-Config", &scope_header)
                .send()
                .await
                .unwrap();
            (resp.status(), resp)
        }
    };

    let ids = |body: &Value| -> Vec<String> {
        body["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["id"].as_str().unwrap().to_string())
            .collect()
    };

    // Omitted → both branches (default behavior).
    let (status, resp) = list(None).await;
    assert_eq!(status, StatusCode::OK);
    let all = ids(&resp.json().await.unwrap());
    assert!(
        all.contains(&participating_id),
        "omitted should include participating"
    );
    assert!(
        all.contains(&available_id),
        "omitted should include available"
    );

    // type=participating → only the participant dialog.
    let (status, resp) = list(Some("participating")).await;
    assert_eq!(status, StatusCode::OK);
    let only_part = ids(&resp.json().await.unwrap());
    assert!(only_part.contains(&participating_id));
    assert!(
        !only_part.contains(&available_id),
        "type=participating should exclude available dialogs"
    );

    // type=available → only the joinable dialog.
    let (status, resp) = list(Some("available")).await;
    assert_eq!(status, StatusCode::OK);
    let only_avail = ids(&resp.json().await.unwrap());
    assert!(only_avail.contains(&available_id));
    assert!(
        !only_avail.contains(&participating_id),
        "type=available should exclude participant dialogs"
    );

    // Invalid type → 400.
    let (status, _) = list(Some("bogus")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Cleanup
    delete_test_dialog(&client, &base_url, &auth_header, &participating_id).await;
    delete_test_dialog(&client, &base_url, &auth_header, &available_id).await;
}

// ============ Synced List Interface Tests (last_message, participants, search) ============

// Helper: send a message as a user, returning the created message id
async fn send_test_message(
    client: &Client,
    base_url: &str,
    dialog_id: &str,
    user_id: Uuid,
    content: &str,
) -> String {
    let resp = client
        .post(format!(
            "{}/api/v1/dialogs/{}/messages?user_id={}",
            base_url, dialog_id, user_id
        ))
        .json(&json!({ "content": content }))
        .send()
        .await
        .expect("Send message failed");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    body["data"]["id"].as_str().unwrap().to_string()
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_dialogs_by_object_includes_last_message_and_participants() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[user_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    // Send a message so there is a last_message.
    let msg_id = send_test_message(&client, &base_url, &dialog_id, user_id, "hello world").await;

    let scope_header = encode_scope_config(tenant_uid, &["logistics"], &["driver"]);
    let resp = client
        .get(format!(
            "{}/api/v1/dialogs/by-object/order/{}/list?user_id={}",
            base_url, object_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let dialogs = body["data"].as_array().unwrap();
    let dialog = dialogs.iter().find(|d| d["id"] == dialog_id).unwrap();

    // last_message is the full message object
    assert_eq!(dialog["last_message"]["id"], msg_id);
    assert_eq!(dialog["last_message"]["content"], "hello world");
    assert_eq!(dialog["last_message"]["message_type"], "user");
    assert_eq!(dialog["last_message"]["sender_id"], user_id.to_string());

    // participants is the full list (at least the one participant)
    let participants = dialog["participants"].as_array().unwrap();
    assert!(participants
        .iter()
        .any(|p| p["user_id"] == user_id.to_string()));

    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_can_join_dialog_hides_last_message_but_shows_participants() {
    // Security: a user who can only join (not yet a participant) must NOT see
    // message content (last_message), but the participant list is not sensitive
    // and is still returned. Mirrors the v0.3.7 "no reading before join" rule.
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let member_id = Uuid::new_v4();
    let observer_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // Dialog where member_id participates; observer_id only matches by scope.
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[member_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    // A message exists in the dialog.
    send_test_message(&client, &base_url, &dialog_id, member_id, "secret content").await;

    // observer_id lists the object's dialogs with matching scope (can_join, not participant).
    let scope_header = encode_scope_config(tenant_uid, &["logistics"], &["driver"]);
    let resp = client
        .get(format!(
            "{}/api/v1/dialogs/by-object/order/{}/list?user_id={}",
            base_url, object_id, observer_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let dialogs = body["data"].as_array().unwrap();
    let dialog = dialogs.iter().find(|d| d["id"] == dialog_id).unwrap();

    // Observer can join but is not a participant.
    assert_eq!(dialog["can_join"], true);
    assert_eq!(dialog["i_am_participant"], false);

    // last_message must be absent (no message content leaked before join).
    assert!(
        dialog["last_message"].is_null(),
        "last_message must be hidden for can_join dialogs, got: {}",
        dialog["last_message"]
    );

    // participants is still returned (membership is not sensitive).
    let participants = dialog["participants"].as_array().unwrap();
    assert!(participants
        .iter()
        .any(|p| p["user_id"] == member_id.to_string()));

    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_participating_includes_last_message_and_participants() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

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

    send_test_message(&client, &base_url, &dialog_id, user_id, "ping").await;

    let resp = client
        .get(format!(
            "{}/api/v1/dialogs?type=participating&user_id={}",
            base_url, user_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let dialogs = body["data"].as_array().unwrap();
    let dialog = dialogs.iter().find(|d| d["id"] == dialog_id).unwrap();

    assert_eq!(dialog["last_message"]["content"], "ping");
    let participants = dialog["participants"].as_array().unwrap();
    assert!(participants
        .iter()
        .any(|p| p["user_id"] == user_id.to_string()));

    delete_test_dialog(&client, &base_url, &auth_header, &dialog_id).await;
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_list_dialogs_by_object_search_filter() {
    let client = Client::new();
    let base_url = get_base_url();
    let auth_header = get_admin_token()
        .map(|t| format!("Bearer {}", t))
        .unwrap_or_default();

    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let object_id = Uuid::new_v4();

    // create_test_dialog always uses title "Test Dialog".
    let dialog_id = create_test_dialog(
        &client,
        &base_url,
        &auth_header,
        object_id,
        "order",
        &[user_id],
        tenant_uid,
        &["logistics"],
        &["driver"],
    )
    .await;

    let scope_header = encode_scope_config(tenant_uid, &["logistics"], &["driver"]);

    // Matching search returns the dialog.
    let resp = client
        .get(format!(
            "{}/api/v1/dialogs/by-object/order/{}/list?user_id={}&search=Test",
            base_url, object_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["id"] == dialog_id));

    // Non-matching search excludes the dialog.
    let resp = client
        .get(format!(
            "{}/api/v1/dialogs/by-object/order/{}/list?user_id={}&search=zzz_no_match_zzz",
            base_url, object_id, user_id
        ))
        .header("X-Scope-Config", &scope_header)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(!body["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["id"] == dialog_id));

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
        .post(format!(
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
        .get(format!(
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
        .post(format!(
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
        .post(format!(
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
        .post(format!(
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
        .get(format!(
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
        .get(format!("{}/api/v1/dialogs?type=participating", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
