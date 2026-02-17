//! Unit tests for domain models
//!
//! Tests the actual domain model constructors, conversions, and business logic
//! using the library crate exports.

use multitenancy_chat_api::domain::{
    attachment_limits, Attachment, AttachmentType, Dialog, DialogAccessScope, DialogParticipant,
    JoinedAs, Message, MessageType, ParticipantProfile,
};
use uuid::Uuid;

// ============ Dialog ============

#[test]
fn test_dialog_new_sets_uuid_v7() {
    let object_id = Uuid::new_v4();
    let dialog = Dialog::new(object_id, "tender", Some("Test".into()), None, None);
    // UUIDv7 has version bits = 7
    assert_eq!(dialog.id.get_version_num(), 7);
}

#[test]
fn test_dialog_new_preserves_fields() {
    let object_id = Uuid::new_v4();
    let creator = Uuid::new_v4();
    let dialog = Dialog::new(
        object_id,
        "order",
        Some("Important Chat".into()),
        Some("https://example.com/order/1".into()),
        Some(creator),
    );
    assert_eq!(dialog.object_id, object_id);
    assert_eq!(dialog.object_type, "order");
    assert_eq!(dialog.title.as_deref(), Some("Important Chat"));
    assert_eq!(
        dialog.object_url.as_deref(),
        Some("https://example.com/order/1")
    );
    assert_eq!(dialog.created_by, Some(creator));
}

#[test]
fn test_dialog_new_optional_fields() {
    let dialog = Dialog::new(Uuid::new_v4(), "tender", None, None, None);
    assert!(dialog.title.is_none());
    assert!(dialog.object_url.is_none());
    assert!(dialog.created_by.is_none());
}

#[test]
fn test_dialog_ids_are_unique() {
    let id = Uuid::new_v4();
    let d1 = Dialog::new(id, "tender", None, None, None);
    let d2 = Dialog::new(id, "tender", None, None, None);
    assert_ne!(d1.id, d2.id);
}

#[test]
fn test_dialog_accepts_string_object_type() {
    let dialog = Dialog::new(
        Uuid::new_v4(),
        String::from("delivery_note"),
        None,
        None,
        None,
    );
    assert_eq!(dialog.object_type, "delivery_note");
}

// ============ JoinedAs ============

#[test]
fn test_joined_as_as_str() {
    assert_eq!(JoinedAs::Creator.as_str(), "creator");
    assert_eq!(JoinedAs::Participant.as_str(), "participant");
    assert_eq!(JoinedAs::Joined.as_str(), "joined");
}

#[test]
fn test_joined_as_from_string() {
    assert_eq!(JoinedAs::from("creator".to_string()), JoinedAs::Creator);
    assert_eq!(
        JoinedAs::from("participant".to_string()),
        JoinedAs::Participant
    );
    assert_eq!(JoinedAs::from("joined".to_string()), JoinedAs::Joined);
}

#[test]
fn test_joined_as_from_str() {
    assert_eq!(JoinedAs::from("creator"), JoinedAs::Creator);
    assert_eq!(JoinedAs::from("participant"), JoinedAs::Participant);
    assert_eq!(JoinedAs::from("joined"), JoinedAs::Joined);
}

#[test]
fn test_joined_as_unknown_defaults_to_participant() {
    assert_eq!(JoinedAs::from("unknown"), JoinedAs::Participant);
    assert_eq!(JoinedAs::from(""), JoinedAs::Participant);
    assert_eq!(JoinedAs::from("CREATOR"), JoinedAs::Participant); // case-sensitive
}

#[test]
fn test_joined_as_roundtrip() {
    for variant in [JoinedAs::Creator, JoinedAs::Participant, JoinedAs::Joined] {
        let s = variant.as_str();
        let recovered = JoinedAs::from(s);
        assert_eq!(recovered, variant);
    }
}

// ============ DialogParticipant ============

#[test]
fn test_participant_new_defaults() {
    let dialog_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let p = DialogParticipant::new(dialog_id, user_id, JoinedAs::Creator);

    assert_eq!(p.dialog_id, dialog_id);
    assert_eq!(p.user_id, user_id);
    assert_eq!(p.joined_as, "creator");
    assert!(p.notifications_enabled);
    assert!(p.last_read_message_id.is_none());
    assert_eq!(p.unread_count, 0);
    assert!(p.display_name.is_none());
    assert!(p.company.is_none());
    assert!(p.email.is_none());
    assert!(p.phone.is_none());
    assert!(!p.is_archived);
    assert!(!p.is_pinned);
}

#[test]
fn test_participant_with_profile() {
    let dialog_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let profile = ParticipantProfile {
        display_name: "John Doe".to_string(),
        company: Some("Acme Inc".to_string()),
        email: Some("john@acme.com".to_string()),
        phone: Some("+1234567890".to_string()),
    };
    let p = DialogParticipant::with_profile(dialog_id, user_id, JoinedAs::Joined, profile);

    assert_eq!(p.display_name.as_deref(), Some("John Doe"));
    assert_eq!(p.company.as_deref(), Some("Acme Inc"));
    assert_eq!(p.email.as_deref(), Some("john@acme.com"));
    assert_eq!(p.phone.as_deref(), Some("+1234567890"));
    assert_eq!(p.joined_as, "joined");
}

#[test]
fn test_participant_with_profile_partial() {
    let profile = ParticipantProfile {
        display_name: "Anonymous".to_string(),
        company: None,
        email: None,
        phone: None,
    };
    let p =
        DialogParticipant::with_profile(Uuid::new_v4(), Uuid::new_v4(), JoinedAs::Joined, profile);

    assert_eq!(p.display_name.as_deref(), Some("Anonymous"));
    assert!(p.company.is_none());
    assert!(p.email.is_none());
    assert!(p.phone.is_none());
}

#[test]
fn test_participant_joined_as_enum() {
    let p = DialogParticipant::new(Uuid::new_v4(), Uuid::new_v4(), JoinedAs::Creator);
    assert_eq!(p.joined_as_enum(), JoinedAs::Creator);

    let p2 = DialogParticipant::new(Uuid::new_v4(), Uuid::new_v4(), JoinedAs::Joined);
    assert_eq!(p2.joined_as_enum(), JoinedAs::Joined);
}

// ============ MessageType ============

#[test]
fn test_message_type_as_str() {
    assert_eq!(MessageType::User.as_str(), "user");
    assert_eq!(MessageType::System.as_str(), "system");
}

#[test]
fn test_message_type_from_str() {
    assert_eq!(MessageType::from_str("user"), MessageType::User);
    assert_eq!(MessageType::from_str("system"), MessageType::System);
}

#[test]
fn test_message_type_from_str_unknown_defaults_to_user() {
    assert_eq!(MessageType::from_str("unknown"), MessageType::User);
    assert_eq!(MessageType::from_str(""), MessageType::User);
    assert_eq!(MessageType::from_str("SYSTEM"), MessageType::User); // case-sensitive
}

#[test]
fn test_message_type_default() {
    assert_eq!(MessageType::default(), MessageType::User);
}

#[test]
fn test_message_type_roundtrip() {
    for variant in [MessageType::User, MessageType::System] {
        let s = variant.as_str();
        let recovered = MessageType::from_str(s);
        assert_eq!(recovered, variant);
    }
}

// ============ Message ============

#[test]
fn test_message_new_user_message() {
    let dialog_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let msg = Message::new(dialog_id, sender_id, "Hello, world!");

    assert_eq!(msg.id.get_version_num(), 7);
    assert_eq!(msg.dialog_id, dialog_id);
    assert_eq!(msg.sender_id, Some(sender_id));
    assert_eq!(msg.content, "Hello, world!");
    assert_eq!(msg.message_type, MessageType::User);
    assert!(msg.reply_to_id.is_none());
    assert!(msg.last_edited_at.is_none());
    assert!(!msg.is_edited());
    assert!(!msg.is_system());
}

#[test]
fn test_message_system() {
    let dialog_id = Uuid::new_v4();
    let msg = Message::system(dialog_id, r#"{"event":"chat_created"}"#);

    assert_eq!(msg.dialog_id, dialog_id);
    assert!(msg.sender_id.is_none());
    assert_eq!(msg.message_type, MessageType::System);
    assert!(msg.is_system());
}

#[test]
fn test_message_with_reply() {
    let dialog_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let reply_to = Uuid::new_v4();
    let msg = Message::new(dialog_id, sender_id, "Reply").with_reply(reply_to);

    assert_eq!(msg.reply_to_id, Some(reply_to));
    assert_eq!(msg.content, "Reply");
}

#[test]
fn test_message_accepts_string_content() {
    let msg = Message::new(Uuid::new_v4(), Uuid::new_v4(), String::from("owned string"));
    assert_eq!(msg.content, "owned string");
}

#[test]
fn test_message_ids_are_unique_and_ordered() {
    let dialog_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let m1 = Message::new(dialog_id, sender_id, "first");
    std::thread::sleep(std::time::Duration::from_millis(2));
    let m2 = Message::new(dialog_id, sender_id, "second");

    assert_ne!(m1.id, m2.id);
    assert!(m2.id > m1.id, "UUIDv7 messages should be time-ordered");
}

// ============ Attachment ============

#[test]
fn test_attachment_new() {
    let message_id = Uuid::new_v4();
    let att = Attachment::new(
        message_id,
        "photo.jpg",
        "image/jpeg",
        1024,
        "dialogs/x/photo.jpg",
    );

    assert_eq!(att.id.get_version_num(), 7);
    assert_eq!(att.message_id, message_id);
    assert_eq!(att.filename, "photo.jpg");
    assert_eq!(att.content_type, "image/jpeg");
    assert_eq!(att.size, 1024);
    assert_eq!(att.s3_key, "dialogs/x/photo.jpg");
    assert!(att.width.is_none());
    assert!(att.height.is_none());
    assert!(att.thumbnail_s3_key.is_none());
}

#[test]
fn test_attachment_with_image_metadata() {
    let att = Attachment::new(Uuid::new_v4(), "photo.jpg", "image/jpeg", 1024, "key")
        .with_image_metadata(800, 600, Some("thumb/key".to_string()));

    assert_eq!(att.width, Some(800));
    assert_eq!(att.height, Some(600));
    assert_eq!(att.thumbnail_s3_key.as_deref(), Some("thumb/key"));
}

#[test]
fn test_attachment_is_image() {
    let image = Attachment::new(Uuid::new_v4(), "f.jpg", "image/jpeg", 100, "k");
    assert!(image.is_image());
    assert!(!image.is_pdf());
    assert_eq!(image.attachment_type(), AttachmentType::Image);

    let png = Attachment::new(Uuid::new_v4(), "f.png", "image/png", 100, "k");
    assert!(png.is_image());

    let svg = Attachment::new(Uuid::new_v4(), "f.svg", "image/svg+xml", 100, "k");
    assert!(svg.is_image());
}

#[test]
fn test_attachment_is_pdf() {
    let pdf = Attachment::new(Uuid::new_v4(), "doc.pdf", "application/pdf", 100, "k");
    assert!(pdf.is_pdf());
    assert!(!pdf.is_image());
    assert_eq!(pdf.attachment_type(), AttachmentType::Pdf);
}

#[test]
fn test_attachment_type_other() {
    let zip = Attachment::new(Uuid::new_v4(), "archive.zip", "application/zip", 100, "k");
    assert!(!zip.is_image());
    assert!(!zip.is_pdf());
    assert_eq!(zip.attachment_type(), AttachmentType::Other);
}

// ============ Attachment Limits ============

#[test]
fn test_attachment_limits_max_file_size() {
    assert_eq!(attachment_limits::MAX_FILE_SIZE, 100 * 1024 * 1024);
}

#[test]
fn test_attachment_limits_max_per_message() {
    assert_eq!(attachment_limits::MAX_ATTACHMENTS_PER_MESSAGE, 10);
}

#[test]
fn test_attachment_limits_valid_size() {
    assert!(attachment_limits::is_valid_size(1));
    assert!(attachment_limits::is_valid_size(1024));
    assert!(attachment_limits::is_valid_size(100 * 1024 * 1024)); // exactly max
    assert!(!attachment_limits::is_valid_size(0));
    assert!(!attachment_limits::is_valid_size(-1));
    assert!(!attachment_limits::is_valid_size(100 * 1024 * 1024 + 1)); // over max
}

#[test]
fn test_attachment_limits_allowed_types() {
    assert!(attachment_limits::is_allowed_type("image/jpeg"));
    assert!(attachment_limits::is_allowed_type("image/png"));
    assert!(attachment_limits::is_allowed_type("application/pdf"));
    assert!(attachment_limits::is_allowed_type("video/mp4"));
    assert!(attachment_limits::is_allowed_type("audio/mpeg"));
    assert!(attachment_limits::is_allowed_type("application/zip"));
    assert!(attachment_limits::is_allowed_type("text/plain"));
}

#[test]
fn test_attachment_limits_disallowed_types() {
    assert!(!attachment_limits::is_allowed_type(
        "application/x-executable"
    ));
    assert!(!attachment_limits::is_allowed_type("application/x-sh"));
    assert!(!attachment_limits::is_allowed_type("text/javascript"));
}

#[test]
fn test_attachment_limits_empty_type_allowed() {
    // Empty content type is allowed (browser couldn't detect)
    assert!(attachment_limits::is_allowed_type(""));
}

// ============ DialogAccessScope ============

#[test]
fn test_access_scope_new() {
    let dialog_id = Uuid::new_v4();
    let tenant_uid = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        dialog_id,
        tenant_uid,
        vec!["dept_a".into(), "dept_b".into()],
        vec!["manager".into()],
    );

    assert_eq!(scope.id.get_version_num(), 7);
    assert_eq!(scope.dialog_id, dialog_id);
    assert_eq!(scope.tenant_uid, tenant_uid);
    assert_eq!(scope.scope_level1, vec!["dept_a", "dept_b"]);
    assert_eq!(scope.scope_level2, vec!["manager"]);
}

#[test]
fn test_scope_matches_exact_match() {
    let tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        Uuid::new_v4(),
        tenant,
        vec!["dept_a".into(), "dept_b".into()],
        vec!["manager".into(), "admin".into()],
    );

    let user_scope1 = vec!["dept_a".into()];
    let user_scope2 = vec!["manager".into()];
    assert!(scope.matches(tenant, &user_scope1, &user_scope2));
}

#[test]
fn test_scope_matches_partial_overlap() {
    let tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        Uuid::new_v4(),
        tenant,
        vec!["dept_a".into(), "dept_b".into()],
        vec!["manager".into(), "admin".into()],
    );

    // User has dept_a (matches), and viewer+manager (manager matches)
    let user_scope1 = vec!["dept_a".into(), "dept_c".into()];
    let user_scope2 = vec!["viewer".into(), "manager".into()];
    assert!(scope.matches(tenant, &user_scope1, &user_scope2));
}

#[test]
fn test_scope_no_match_wrong_tenant() {
    let tenant = Uuid::new_v4();
    let other_tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        Uuid::new_v4(),
        tenant,
        vec!["dept_a".into()],
        vec!["manager".into()],
    );

    let user_scope1 = vec!["dept_a".into()];
    let user_scope2 = vec!["manager".into()];
    assert!(!scope.matches(other_tenant, &user_scope1, &user_scope2));
}

#[test]
fn test_scope_no_match_level1() {
    let tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        Uuid::new_v4(),
        tenant,
        vec!["dept_a".into(), "dept_b".into()],
        vec!["manager".into()],
    );

    let user_scope1 = vec!["dept_c".into()]; // no overlap with [dept_a, dept_b]
    let user_scope2 = vec!["manager".into()];
    assert!(!scope.matches(tenant, &user_scope1, &user_scope2));
}

#[test]
fn test_scope_no_match_level2() {
    let tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        Uuid::new_v4(),
        tenant,
        vec!["dept_a".into()],
        vec!["manager".into(), "admin".into()],
    );

    let user_scope1 = vec!["dept_a".into()];
    let user_scope2 = vec!["viewer".into(), "guest".into()]; // no overlap
    assert!(!scope.matches(tenant, &user_scope1, &user_scope2));
}

#[test]
fn test_scope_empty_dialog_scope_matches_any() {
    let tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        Uuid::new_v4(),
        tenant,
        vec![], // empty = matches any
        vec![], // empty = matches any
    );

    let user_scope1 = vec!["anything".into()];
    let user_scope2 = vec!["anything".into()];
    assert!(scope.matches(tenant, &user_scope1, &user_scope2));
}

#[test]
fn test_scope_empty_dialog_scope_matches_empty_user() {
    let tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(Uuid::new_v4(), tenant, vec![], vec![]);

    let empty: Vec<String> = vec![];
    assert!(scope.matches(tenant, &empty, &empty));
}

#[test]
fn test_scope_nonempty_vs_empty_user_no_match() {
    let tenant = Uuid::new_v4();
    let scope = DialogAccessScope::new(
        Uuid::new_v4(),
        tenant,
        vec!["dept_a".into()],
        vec!["manager".into()],
    );

    let empty: Vec<String> = vec![];
    // Non-empty dialog scope vs empty user scope = no match
    assert!(!scope.matches(tenant, &empty, &empty));
}

#[test]
fn test_scope_mixed_empty_levels() {
    let tenant = Uuid::new_v4();

    // level1 is empty (matches any), level2 is not
    let scope1 = DialogAccessScope::new(Uuid::new_v4(), tenant, vec![], vec!["admin".into()]);
    assert!(scope1.matches(tenant, &vec!["anything".into()], &vec!["admin".into()]));
    assert!(!scope1.matches(tenant, &vec!["anything".into()], &vec!["viewer".into()]));

    // level1 is not empty, level2 is empty (matches any)
    let scope2 = DialogAccessScope::new(Uuid::new_v4(), tenant, vec!["dept_a".into()], vec![]);
    assert!(scope2.matches(tenant, &vec!["dept_a".into()], &vec![]));
    assert!(!scope2.matches(tenant, &vec!["dept_b".into()], &vec![]));
}
