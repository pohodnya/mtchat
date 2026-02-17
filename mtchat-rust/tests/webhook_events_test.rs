//! Tests for webhook event construction from domain models
//!
//! Verifies that WebhookEvent factory methods correctly map
//! domain entities to event payloads.

use multitenancy_chat_api::domain::{Dialog, DialogParticipant, JoinedAs, Message};
use multitenancy_chat_api::webhooks::{WebhookEvent, WebhookEventType, WebhookPayload};
use uuid::Uuid;

fn make_dialog() -> Dialog {
    Dialog::new(
        Uuid::new_v4(),
        "tender",
        Some("Test Dialog".into()),
        None,
        Some(Uuid::new_v4()),
    )
}

#[test]
fn test_message_new_event() {
    let dialog = make_dialog();
    let sender_id = Uuid::new_v4();
    let message = Message::new(dialog.id, sender_id, "Hello world");

    let event = WebhookEvent::message_new(&dialog, &message);

    assert_eq!(event.event_type, WebhookEventType::MessageNew);
    assert_eq!(event.id.get_version_num(), 7);

    if let WebhookPayload::MessageNew(payload) = &event.payload {
        assert_eq!(payload.dialog_id, dialog.id);
        assert_eq!(payload.object_id, dialog.object_id);
        assert_eq!(payload.object_type, "tender");
        assert_eq!(payload.message.id, message.id);
        assert_eq!(payload.message.sender_id, Some(sender_id));
        assert_eq!(payload.message.content, "Hello world");
        assert!(payload.message.reply_to.is_none());
        assert_eq!(payload.message.message_type, "user");
    } else {
        panic!("Expected MessageNew payload");
    }
}

#[test]
fn test_message_new_event_system_message() {
    let dialog = make_dialog();
    let message = Message::system(dialog.id, r#"{"event":"chat_created"}"#);

    let event = WebhookEvent::message_new(&dialog, &message);

    if let WebhookPayload::MessageNew(payload) = &event.payload {
        assert!(payload.message.sender_id.is_none());
        assert_eq!(payload.message.message_type, "system");
    } else {
        panic!("Expected MessageNew payload");
    }
}

#[test]
fn test_message_new_event_with_reply() {
    let dialog = make_dialog();
    let reply_to = Uuid::new_v4();
    let message = Message::new(dialog.id, Uuid::new_v4(), "Reply text").with_reply(reply_to);

    let event = WebhookEvent::message_new(&dialog, &message);

    if let WebhookPayload::MessageNew(payload) = &event.payload {
        assert_eq!(payload.message.reply_to, Some(reply_to));
    } else {
        panic!("Expected MessageNew payload");
    }
}

#[test]
fn test_participant_joined_event() {
    let dialog = make_dialog();
    let user_id = Uuid::new_v4();
    let participant = DialogParticipant::new(dialog.id, user_id, JoinedAs::Joined);

    let event = WebhookEvent::participant_joined(&dialog, &participant);

    assert_eq!(event.event_type, WebhookEventType::ParticipantJoined);

    if let WebhookPayload::ParticipantJoined(payload) = &event.payload {
        assert_eq!(payload.dialog_id, dialog.id);
        assert_eq!(payload.object_id, dialog.object_id);
        assert_eq!(payload.object_type, "tender");
        assert_eq!(payload.user_id, user_id);
        assert_eq!(payload.joined_as, "joined");
    } else {
        panic!("Expected ParticipantJoined payload");
    }
}

#[test]
fn test_participant_left_event() {
    let dialog = make_dialog();
    let user_id = Uuid::new_v4();

    let event = WebhookEvent::participant_left(&dialog, user_id);

    assert_eq!(event.event_type, WebhookEventType::ParticipantLeft);

    if let WebhookPayload::ParticipantLeft(payload) = &event.payload {
        assert_eq!(payload.dialog_id, dialog.id);
        assert_eq!(payload.object_id, dialog.object_id);
        assert_eq!(payload.user_id, user_id);
    } else {
        panic!("Expected ParticipantLeft payload");
    }
}

#[test]
fn test_notification_pending_event() {
    let dialog = make_dialog();
    let sender_id = Uuid::new_v4();
    let recipient_id = Uuid::new_v4();
    let message = Message::new(dialog.id, sender_id, "Unread message");

    let event = WebhookEvent::notification_pending(&dialog, &message, recipient_id);

    assert_eq!(event.event_type, WebhookEventType::NotificationPending);

    if let WebhookPayload::NotificationPending(payload) = &event.payload {
        assert_eq!(payload.dialog_id, dialog.id);
        assert_eq!(payload.recipient_id, recipient_id);
        assert_eq!(payload.message.id, message.id);
        assert_eq!(payload.message.sender_id, Some(sender_id));
        assert_eq!(payload.message.content, "Unread message");
    } else {
        panic!("Expected NotificationPending payload");
    }
}

#[test]
fn test_event_serialization_roundtrip() {
    let dialog = make_dialog();
    let message = Message::new(dialog.id, Uuid::new_v4(), "Test");
    let event = WebhookEvent::message_new(&dialog, &message);

    let json = serde_json::to_string(&event).expect("serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");

    assert_eq!(parsed["type"], "message_new");
    assert!(parsed["id"].is_string());
    assert!(parsed["timestamp"].is_string());
    assert_eq!(parsed["payload"]["object_type"], "tender");
    assert_eq!(parsed["payload"]["message"]["content"], "Test");
}

#[test]
fn test_event_type_display() {
    assert_eq!(WebhookEventType::MessageNew.to_string(), "message.new");
    assert_eq!(
        WebhookEventType::ParticipantJoined.to_string(),
        "participant.joined"
    );
    assert_eq!(
        WebhookEventType::ParticipantLeft.to_string(),
        "participant.left"
    );
    assert_eq!(
        WebhookEventType::NotificationPending.to_string(),
        "notification.pending"
    );
}
