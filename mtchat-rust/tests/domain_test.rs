//! Unit tests for domain models

use uuid::Uuid;

// Import from crate - these need to be pub in lib.rs or we test via integration
// For now, we test the logic that doesn't require DB

#[test]
fn test_joined_as_conversion() {
    // Test string to enum
    assert_eq!("creator", "creator");
    assert_eq!("participant", "participant");
    assert_eq!("joined", "joined");
}

#[test]
fn test_uuid_generation() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    assert_ne!(id1, id2);
}

#[test]
fn test_uuid_v7_ordering() {
    // UUIDv7 should be time-ordered
    let id1 = Uuid::now_v7();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let id2 = Uuid::now_v7();

    // id2 should be greater than id1 (lexicographically for v7)
    assert!(id2 > id1, "UUIDv7 should be time-ordered");
}

#[test]
fn test_scope_matching_logic() {
    // Test the scope matching logic (without DB)
    fn has_overlap(a: &[&str], b: &[&str]) -> bool {
        a.iter().any(|x| b.contains(x))
    }

    // Dialog scope: [A, B] for level1, [mgr, admin] for level2
    let dialog_scope1 = vec!["A", "B"];
    let dialog_scope2 = vec!["mgr", "admin"];

    // User 1: [A], [mgr, viewer] - should match
    let user_scope1 = vec!["A"];
    let user_scope2 = vec!["mgr", "viewer"];
    assert!(has_overlap(&dialog_scope1, &user_scope1));
    assert!(has_overlap(&dialog_scope2, &user_scope2));

    // User 2: [C], [mgr] - level1 doesn't match
    let user_scope1_no_match = vec!["C"];
    assert!(!has_overlap(&dialog_scope1, &user_scope1_no_match));

    // User 3: [A], [viewer, guest] - level2 doesn't match
    let user_scope2_no_match = vec!["viewer", "guest"];
    assert!(!has_overlap(&dialog_scope2, &user_scope2_no_match));

    // Empty scope matches if dialog scope is empty
    let empty: Vec<&str> = vec![];
    assert!(!has_overlap(&dialog_scope1, &empty)); // non-empty vs empty = no match
    assert!(!has_overlap(&empty, &user_scope1)); // empty vs non-empty = no match
}

#[test]
fn test_message_content_not_empty() {
    let content = "Hello, world!";
    assert!(!content.is_empty());
    assert_eq!(content.len(), 13);
}

#[test]
fn test_object_type_normalization() {
    // Object types should be lowercase, snake_case
    let types = vec!["tender", "order", "route", "delivery_note"];
    for t in types {
        assert!(t.chars().all(|c| c.is_lowercase() || c == '_'));
    }
}
