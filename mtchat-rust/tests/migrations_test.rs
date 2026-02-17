//! Migration tests for MTChat v3 architecture
//!
//! These tests verify that database migrations are applied correctly
//! and that the new schema works as expected.
//!
//! Run with: cargo test --test migrations_test
//! Requires: TEST_DATABASE_URL environment variable

use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

async fn get_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/mtchat_test".into());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn setup_test_db() -> PgPool {
    let pool = get_pool().await;

    // Run migrations once
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

// ============ Schema Tests ============

#[tokio::test]
async fn test_dialogs_table_has_new_columns() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    let id = Uuid::new_v4();
    let object_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    let result = sqlx::query(
        r#"INSERT INTO dialogs (id, object_id, object_type, title, created_by, created_at)
           VALUES ($1, $2, $3, $4, $5, NOW())
           RETURNING id, object_id, object_type, title, created_by"#,
    )
    .bind(id)
    .bind(object_id)
    .bind("tender")
    .bind("Test Dialog")
    .bind(created_by)
    .fetch_one(&mut *tx)
    .await;

    assert!(result.is_ok(), "Failed to insert dialog with new columns");

    let row = result.unwrap();
    assert_eq!(row.get::<Uuid, _>("id"), id);
    assert_eq!(row.get::<Uuid, _>("object_id"), object_id);
    assert_eq!(row.get::<String, _>("object_type"), "tender");
    assert_eq!(
        row.get::<Option<String>, _>("title"),
        Some("Test Dialog".to_string())
    );
    assert_eq!(row.get::<Option<Uuid>, _>("created_by"), Some(created_by));

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_dialogs_multiple_per_object() {
    let pool = setup_test_db().await;

    let object_id = Uuid::new_v4();
    let dialog1_id = Uuid::new_v4();
    let dialog2_id = Uuid::new_v4();
    let dialog3_id = Uuid::new_v4();

    // First insert should succeed
    let mut tx1 = pool.begin().await.unwrap();
    let result1 = sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog1_id)
    .bind(object_id)
    .bind("tender")
    .execute(&mut *tx1)
    .await;
    assert!(result1.is_ok(), "First insert should succeed");
    tx1.commit().await.unwrap();

    // Second insert with same object_id + object_type should also succeed (multiple dialogs per object allowed)
    let mut tx2 = pool.begin().await.unwrap();
    let result2 = sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog2_id)
    .bind(object_id)
    .bind("tender")
    .execute(&mut *tx2)
    .await;
    assert!(
        result2.is_ok(),
        "Multiple dialogs per object should be allowed"
    );
    tx2.commit().await.unwrap();

    // Same object_id but different object_type should also succeed
    let mut tx3 = pool.begin().await.unwrap();
    let result3 = sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog3_id)
    .bind(object_id)
    .bind("order") // Different type
    .execute(&mut *tx3)
    .await;
    assert!(
        result3.is_ok(),
        "Same object_id with different type should succeed"
    );
    tx3.commit().await.unwrap();

    // Cleanup
    sqlx::query("DELETE FROM dialogs WHERE id = ANY($1)")
        .bind(&vec![dialog1_id, dialog2_id, dialog3_id])
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_dialog_participants_table_structure() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    // Create a dialog first
    let dialog_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Insert participant with all new columns
    let user_id = Uuid::new_v4();
    let result = sqlx::query(
        r#"INSERT INTO dialog_participants
           (dialog_id, user_id, joined_as, notifications_enabled, joined_at)
           VALUES ($1, $2, $3, $4, NOW())
           RETURNING dialog_id, user_id, joined_as, notifications_enabled"#,
    )
    .bind(dialog_id)
    .bind(user_id)
    .bind("creator")
    .bind(true)
    .fetch_one(&mut *tx)
    .await;

    assert!(
        result.is_ok(),
        "Failed to insert participant with new columns"
    );

    let row = result.unwrap();
    assert_eq!(row.get::<Uuid, _>("dialog_id"), dialog_id);
    assert_eq!(row.get::<Uuid, _>("user_id"), user_id);
    assert_eq!(row.get::<String, _>("joined_as"), "creator");
    assert!(row.get::<bool, _>("notifications_enabled"));

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_dialog_participants_default_values() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    let dialog_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Insert with minimal fields - check defaults
    let user_id = Uuid::new_v4();
    let row = sqlx::query(
        r#"INSERT INTO dialog_participants (dialog_id, user_id, joined_at)
           VALUES ($1, $2, NOW())
           RETURNING joined_as, notifications_enabled"#,
    )
    .bind(dialog_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap();

    assert_eq!(row.get::<String, _>("joined_as"), "participant");
    assert!(row.get::<bool, _>("notifications_enabled"));

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_dialog_access_scopes_table() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    let dialog_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    let tenant_uid = Uuid::new_v4();
    let scope_level1 = vec!["dept_logistics", "dept_sales"];
    let scope_level2 = vec!["tender:manager", "tender:admin"];

    let row = sqlx::query(
        r#"INSERT INTO dialog_access_scopes
           (dialog_id, tenant_uid, scope_level1, scope_level2)
           VALUES ($1, $2, $3, $4)
           RETURNING id, dialog_id, tenant_uid, scope_level1, scope_level2"#,
    )
    .bind(dialog_id)
    .bind(tenant_uid)
    .bind(&scope_level1)
    .bind(&scope_level2)
    .fetch_one(&mut *tx)
    .await
    .unwrap();

    assert_eq!(row.get::<Uuid, _>("dialog_id"), dialog_id);
    assert_eq!(row.get::<Uuid, _>("tenant_uid"), tenant_uid);
    assert_eq!(
        row.get::<Vec<String>, _>("scope_level1"),
        vec!["dept_logistics", "dept_sales"]
    );
    assert_eq!(
        row.get::<Vec<String>, _>("scope_level2"),
        vec!["tender:manager", "tender:admin"]
    );

    tx.rollback().await.unwrap();
}

// ============ Scope Matching Tests ============

#[tokio::test]
async fn test_scope_matching_with_array_overlap() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    // Setup: Create dialog with access scope
    let dialog_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Scope: tenant X, departments [A, B], permissions [mgr, admin]
    sqlx::query(
        r#"INSERT INTO dialog_access_scopes (dialog_id, tenant_uid, scope_level1, scope_level2)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(dialog_id)
    .bind(tenant_uid)
    .bind(&vec!["A", "B"])
    .bind(&vec!["mgr", "admin"])
    .execute(&mut *tx)
    .await
    .unwrap();

    // Test 1: User with matching scope should find the dialog
    let user_scope1 = vec!["A"]; // matches [A, B]
    let user_scope2 = vec!["mgr", "viewer"]; // matches [mgr, admin]

    let result = sqlx::query(
        r#"SELECT d.id FROM dialogs d
           INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
           WHERE s.tenant_uid = $1
             AND s.scope_level1 && $2
             AND s.scope_level2 && $3"#,
    )
    .bind(tenant_uid)
    .bind(&user_scope1)
    .bind(&user_scope2)
    .fetch_optional(&mut *tx)
    .await
    .unwrap();

    assert!(
        result.is_some(),
        "User with matching scope should find dialog"
    );

    // Test 2: User with non-matching scope_level1 should NOT find the dialog
    let non_matching_scope1 = vec!["C", "D"]; // doesn't match [A, B]
    let result = sqlx::query(
        r#"SELECT d.id FROM dialogs d
           INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
           WHERE s.tenant_uid = $1
             AND s.scope_level1 && $2
             AND s.scope_level2 && $3"#,
    )
    .bind(tenant_uid)
    .bind(&non_matching_scope1)
    .bind(&user_scope2)
    .fetch_optional(&mut *tx)
    .await
    .unwrap();

    assert!(
        result.is_none(),
        "User with non-matching scope_level1 should NOT find dialog"
    );

    // Test 3: User with non-matching scope_level2 should NOT find the dialog
    let non_matching_scope2 = vec!["viewer", "guest"]; // doesn't match [mgr, admin]
    let result = sqlx::query(
        r#"SELECT d.id FROM dialogs d
           INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
           WHERE s.tenant_uid = $1
             AND s.scope_level1 && $2
             AND s.scope_level2 && $3"#,
    )
    .bind(tenant_uid)
    .bind(&user_scope1)
    .bind(&non_matching_scope2)
    .fetch_optional(&mut *tx)
    .await
    .unwrap();

    assert!(
        result.is_none(),
        "User with non-matching scope_level2 should NOT find dialog"
    );

    // Test 4: Wrong tenant should NOT find the dialog
    let wrong_tenant = Uuid::new_v4();
    let result = sqlx::query(
        r#"SELECT d.id FROM dialogs d
           INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
           WHERE s.tenant_uid = $1
             AND s.scope_level1 && $2
             AND s.scope_level2 && $3"#,
    )
    .bind(wrong_tenant)
    .bind(&user_scope1)
    .bind(&user_scope2)
    .fetch_optional(&mut *tx)
    .await
    .unwrap();

    assert!(
        result.is_none(),
        "User with wrong tenant should NOT find dialog"
    );

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_available_dialogs_excludes_participants() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    let dialog_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Create dialog
    sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Create access scope
    sqlx::query(
        r#"INSERT INTO dialog_access_scopes (dialog_id, tenant_uid, scope_level1, scope_level2)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(dialog_id)
    .bind(tenant_uid)
    .bind(&vec!["dept"])
    .bind(&vec!["perm"])
    .execute(&mut *tx)
    .await
    .unwrap();

    // User is NOT a participant yet - should see as available
    let result = sqlx::query(
        r#"SELECT d.id FROM dialogs d
           INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
           WHERE s.tenant_uid = $1
             AND s.scope_level1 && $2
             AND s.scope_level2 && $3
             AND NOT EXISTS (
               SELECT 1 FROM dialog_participants dp
               WHERE dp.dialog_id = d.id AND dp.user_id = $4
             )"#,
    )
    .bind(tenant_uid)
    .bind(&vec!["dept"])
    .bind(&vec!["perm"])
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .unwrap();

    assert!(
        result.is_some(),
        "Non-participant should see dialog as available"
    );

    // Add user as participant
    sqlx::query(
        "INSERT INTO dialog_participants (dialog_id, user_id, joined_at) VALUES ($1, $2, NOW())",
    )
    .bind(dialog_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    // Now user should NOT see it as available
    let result = sqlx::query(
        r#"SELECT d.id FROM dialogs d
           INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
           WHERE s.tenant_uid = $1
             AND s.scope_level1 && $2
             AND s.scope_level2 && $3
             AND NOT EXISTS (
               SELECT 1 FROM dialog_participants dp
               WHERE dp.dialog_id = d.id AND dp.user_id = $4
             )"#,
    )
    .bind(tenant_uid)
    .bind(&vec!["dept"])
    .bind(&vec!["perm"])
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .unwrap();

    assert!(
        result.is_none(),
        "Participant should NOT see dialog as available"
    );

    tx.rollback().await.unwrap();
}

// ============ Messages Tests ============

#[tokio::test]
async fn test_messages_reply_to_column() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    let dialog_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    let sender_id = Uuid::new_v4();

    // Create original message
    let msg1_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at)
           VALUES ($1, $2, $3, $4, NOW())"#,
    )
    .bind(msg1_id)
    .bind(dialog_id)
    .bind(sender_id)
    .bind("Original message")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Create reply
    let msg2_id = Uuid::new_v4();
    let row = sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, reply_to_id)
           VALUES ($1, $2, $3, $4, NOW(), $5)
           RETURNING id, reply_to_id"#,
    )
    .bind(msg2_id)
    .bind(dialog_id)
    .bind(sender_id)
    .bind("Reply message")
    .bind(msg1_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap();

    assert_eq!(row.get::<Uuid, _>("id"), msg2_id);
    assert_eq!(row.get::<Option<Uuid>, _>("reply_to_id"), Some(msg1_id));

    // Verify we can query replies
    let replies = sqlx::query("SELECT id FROM messages WHERE reply_to_id = $1")
        .bind(msg1_id)
        .fetch_all(&mut *tx)
        .await
        .unwrap();

    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].get::<Uuid, _>("id"), msg2_id);

    tx.rollback().await.unwrap();
}

// ============ Cascade Delete Tests ============

#[tokio::test]
async fn test_cascade_delete_dialog() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    let dialog_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();

    // Create dialog
    sqlx::query(
        "INSERT INTO dialogs (id, object_id, object_type, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Add participant
    sqlx::query(
        "INSERT INTO dialog_participants (dialog_id, user_id, joined_at) VALUES ($1, $2, NOW())",
    )
    .bind(dialog_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .unwrap();

    // Add access scope
    sqlx::query(
        r#"INSERT INTO dialog_access_scopes (dialog_id, tenant_uid, scope_level1, scope_level2)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(dialog_id)
    .bind(tenant_uid)
    .bind(&vec!["dept"])
    .bind(&vec!["perm"])
    .execute(&mut *tx)
    .await
    .unwrap();

    // Add message
    sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at)
           VALUES ($1, $2, $3, $4, NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(dialog_id)
    .bind(user_id)
    .bind("Test message")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Delete dialog
    sqlx::query("DELETE FROM dialogs WHERE id = $1")
        .bind(dialog_id)
        .execute(&mut *tx)
        .await
        .unwrap();

    // Verify all related data is deleted
    let participants: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM dialog_participants WHERE dialog_id = $1")
            .bind(dialog_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(participants.0, 0, "Participants should be cascade deleted");

    let scopes: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM dialog_access_scopes WHERE dialog_id = $1")
            .bind(dialog_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(scopes.0, 0, "Access scopes should be cascade deleted");

    let messages: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE dialog_id = $1")
        .bind(dialog_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(messages.0, 0, "Messages should be cascade deleted");

    tx.rollback().await.unwrap();
}

// ============ Index Tests ============

#[tokio::test]
async fn test_gin_indexes_exist() {
    let pool = setup_test_db().await;

    // Check GIN indexes on scope arrays
    let result = sqlx::query(
        r#"SELECT indexname FROM pg_indexes
           WHERE tablename = 'dialog_access_scopes'
           AND indexdef LIKE '%gin%'"#,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let index_names: Vec<String> = result.iter().map(|r| r.get("indexname")).collect();

    assert!(
        index_names.iter().any(|n| n.contains("level1")),
        "GIN index on scope_level1 should exist"
    );
    assert!(
        index_names.iter().any(|n| n.contains("level2")),
        "GIN index on scope_level2 should exist"
    );
}

#[tokio::test]
async fn test_index_on_object() {
    let pool = setup_test_db().await;

    // Index should exist
    let result = sqlx::query(
        r#"SELECT indexname, indexdef FROM pg_indexes
           WHERE tablename = 'dialogs'
           AND indexname = 'idx_dialogs_object'"#,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(
        !result.is_empty(),
        "Index on (object_id, object_type) should exist"
    );

    // Index should NOT be unique (multiple dialogs per object allowed)
    let indexdef: String = result[0].get("indexdef");
    assert!(
        !indexdef.to_uppercase().contains("UNIQUE"),
        "Index should NOT be unique (multiple dialogs per object allowed)"
    );
}

// ============ Legacy Tables Removal Tests ============

#[tokio::test]
async fn test_legacy_tables_removed() {
    let pool = setup_test_db().await;

    // Check that tenants table does NOT exist
    let tenants_exists = sqlx::query(
        r#"SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = 'tenants'
        ) as exists"#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        !tenants_exists.get::<bool, _>("exists"),
        "Table 'tenants' should be removed"
    );

    // Check that employees table does NOT exist
    let employees_exists = sqlx::query(
        r#"SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = 'employees'
        ) as exists"#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        !employees_exists.get::<bool, _>("exists"),
        "Table 'employees' should be removed"
    );
}

#[tokio::test]
async fn test_no_fk_to_legacy_tables() {
    let pool = setup_test_db().await;

    // Check no FK references to employees or tenants
    let fk_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM information_schema.table_constraints tc
           JOIN information_schema.referential_constraints rc
             ON tc.constraint_name = rc.constraint_name
           JOIN information_schema.constraint_column_usage ccu
             ON rc.unique_constraint_name = ccu.constraint_name
           WHERE ccu.table_name IN ('employees', 'tenants')
             AND tc.constraint_type = 'FOREIGN KEY'"#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        fk_count.0, 0,
        "No FK constraints should reference employees or tenants tables"
    );
}

#[tokio::test]
async fn test_system_messages_support() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();

    // Create dialog
    let dialog_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO dialogs (id, object_id, object_type, created_at)
           VALUES ($1, $2, $3, NOW())"#,
    )
    .bind(dialog_id)
    .bind(Uuid::new_v4())
    .bind("tender")
    .execute(&mut *tx)
    .await
    .unwrap();

    // Test 1: Verify message_type column exists with default 'user'
    let user_msg_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at)
           VALUES ($1, $2, $3, $4, NOW())"#,
    )
    .bind(user_msg_id)
    .bind(dialog_id)
    .bind(sender_id)
    .bind("User message")
    .execute(&mut *tx)
    .await
    .unwrap();

    let row = sqlx::query("SELECT message_type FROM messages WHERE id = $1")
        .bind(user_msg_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    let msg_type: String = row.get("message_type");
    assert_eq!(msg_type, "user", "Default message_type should be 'user'");

    // Test 2: Create system message (sender_id = NULL, message_type = 'system')
    let sys_msg_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, message_type)
           VALUES ($1, $2, NULL, $3, NOW(), 'system')"#,
    )
    .bind(sys_msg_id)
    .bind(dialog_id)
    .bind(r#"{"event":"participant_joined","name":"Test User"}"#)
    .execute(&mut *tx)
    .await
    .unwrap();

    let row = sqlx::query("SELECT sender_id, message_type FROM messages WHERE id = $1")
        .bind(sys_msg_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    let sys_sender_id: Option<Uuid> = row.get("sender_id");
    let sys_msg_type: String = row.get("message_type");
    assert!(
        sys_sender_id.is_none(),
        "System message should have NULL sender_id"
    );
    assert_eq!(
        sys_msg_type, "system",
        "System message should have type 'system'"
    );

    // Test 3: Verify constraint - user message must have sender_id
    let result = sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, message_type)
           VALUES ($1, $2, NULL, $3, NOW(), 'user')"#,
    )
    .bind(Uuid::new_v4())
    .bind(dialog_id)
    .bind("Should fail")
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_err(),
        "User message with NULL sender_id should fail constraint"
    );

    // Test 4: Verify constraint - system message must have NULL sender_id
    let result = sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, message_type)
           VALUES ($1, $2, $3, $4, NOW(), 'system')"#,
    )
    .bind(Uuid::new_v4())
    .bind(dialog_id)
    .bind(sender_id)
    .bind("Should fail")
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_err(),
        "System message with sender_id should fail constraint"
    );

    // Test 5: Verify invalid message_type is rejected
    let result = sqlx::query(
        r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, message_type)
           VALUES ($1, $2, $3, $4, NOW(), 'invalid')"#,
    )
    .bind(Uuid::new_v4())
    .bind(dialog_id)
    .bind(sender_id)
    .bind("Should fail")
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_err(),
        "Invalid message_type should fail constraint"
    );

    tx.rollback().await.unwrap();
}
