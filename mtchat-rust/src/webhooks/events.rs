//! Webhook event types and payloads

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Dialog, DialogParticipant, Message};

/// Webhook event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// New message was sent
    MessageNew,
    /// User joined a dialog
    ParticipantJoined,
    /// User left a dialog
    ParticipantLeft,
    /// Notification pending - message not read after delay
    NotificationPending,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MessageNew => "message.new",
            Self::ParticipantJoined => "participant.joined",
            Self::ParticipantLeft => "participant.left",
            Self::NotificationPending => "notification.pending",
        }
    }
}

impl std::fmt::Display for WebhookEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Webhook event wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Event type
    #[serde(rename = "type")]
    pub event_type: WebhookEventType,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Event payload
    pub payload: WebhookPayload,
}

impl WebhookEvent {
    /// Create a new webhook event
    pub fn new(event_type: WebhookEventType, payload: WebhookPayload) -> Self {
        Self {
            id: Uuid::now_v7(),
            event_type,
            timestamp: Utc::now(),
            payload,
        }
    }

    /// Create a message.new event
    pub fn message_new(dialog: &Dialog, message: &Message) -> Self {
        Self::new(
            WebhookEventType::MessageNew,
            WebhookPayload::MessageNew(MessageNewPayload {
                dialog_id: dialog.id,
                object_id: dialog.object_id,
                object_type: dialog.object_type.clone(),
                message: MessageData {
                    id: message.id,
                    sender_id: message.sender_id,
                    content: message.content.clone(),
                    reply_to: message.reply_to_id,
                    created_at: message.sent_at,
                    message_type: message.message_type.as_str().to_string(),
                },
            }),
        )
    }

    /// Create a participant.joined event
    pub fn participant_joined(dialog: &Dialog, participant: &DialogParticipant) -> Self {
        Self::new(
            WebhookEventType::ParticipantJoined,
            WebhookPayload::ParticipantJoined(ParticipantPayload {
                dialog_id: dialog.id,
                object_id: dialog.object_id,
                object_type: dialog.object_type.clone(),
                user_id: participant.user_id,
                joined_as: participant.joined_as.clone(),
                joined_at: participant.joined_at,
            }),
        )
    }

    /// Create a participant.left event
    pub fn participant_left(dialog: &Dialog, user_id: Uuid) -> Self {
        Self::new(
            WebhookEventType::ParticipantLeft,
            WebhookPayload::ParticipantLeft(ParticipantLeftPayload {
                dialog_id: dialog.id,
                object_id: dialog.object_id,
                object_type: dialog.object_type.clone(),
                user_id,
                left_at: Utc::now(),
            }),
        )
    }

    /// Create a notification.pending event (smart notifications)
    ///
    /// Sent when a message was not read after delay period.
    /// The receiving system should send a push notification to the user.
    pub fn notification_pending(dialog: &Dialog, message: &Message, recipient_id: Uuid) -> Self {
        Self::new(
            WebhookEventType::NotificationPending,
            WebhookPayload::NotificationPending(NotificationPendingPayload {
                dialog_id: dialog.id,
                object_id: dialog.object_id,
                object_type: dialog.object_type.clone(),
                recipient_id,
                message: MessageData {
                    id: message.id,
                    sender_id: message.sender_id,
                    content: message.content.clone(),
                    reply_to: message.reply_to_id,
                    created_at: message.sent_at,
                    message_type: message.message_type.as_str().to_string(),
                },
            }),
        )
    }
}

/// Event payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebhookPayload {
    MessageNew(MessageNewPayload),
    ParticipantJoined(ParticipantPayload),
    ParticipantLeft(ParticipantLeftPayload),
    NotificationPending(NotificationPendingPayload),
}

/// Payload for message.new events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageNewPayload {
    pub dialog_id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    pub message: MessageData,
}

/// Message data in webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<Uuid>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    /// Message type: 'user' or 'system'
    #[serde(default = "default_message_type")]
    pub message_type: String,
}

fn default_message_type() -> String {
    "user".to_string()
}

/// Payload for participant.joined events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPayload {
    pub dialog_id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    pub user_id: Uuid,
    pub joined_as: String,
    pub joined_at: DateTime<Utc>,
}

/// Payload for participant.left events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantLeftPayload {
    pub dialog_id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    pub user_id: Uuid,
    pub left_at: DateTime<Utc>,
}

/// Payload for notification.pending events (smart notifications)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPendingPayload {
    pub dialog_id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    /// User who should receive the notification
    pub recipient_id: Uuid,
    /// Message that triggered the notification
    pub message: MessageData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_serialization() {
        assert_eq!(WebhookEventType::MessageNew.as_str(), "message.new");
        assert_eq!(WebhookEventType::ParticipantJoined.as_str(), "participant.joined");
        assert_eq!(WebhookEventType::ParticipantLeft.as_str(), "participant.left");
    }

    #[test]
    fn test_event_json_serialization() {
        let event = WebhookEvent::new(
            WebhookEventType::MessageNew,
            WebhookPayload::MessageNew(MessageNewPayload {
                dialog_id: Uuid::nil(),
                object_id: Uuid::nil(),
                object_type: "tender".to_string(),
                message: MessageData {
                    id: Uuid::nil(),
                    sender_id: Some(Uuid::nil()),
                    content: "Hello".to_string(),
                    reply_to: None,
                    created_at: Utc::now(),
                    message_type: "user".to_string(),
                },
            }),
        );

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("message_new"));
        assert!(json.contains("tender"));
        assert!(json.contains("Hello"));
    }
}
