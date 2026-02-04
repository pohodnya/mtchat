//! Message entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A message in a dialog
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub dialog_id: Uuid,
    /// External user identifier (from JWT token)
    pub sender_id: Uuid,
    pub content: String,
    pub sent_at: DateTime<Utc>,
    pub last_edited_at: Option<DateTime<Utc>>,
    /// Reference to the message this is a reply to
    pub reply_to_id: Option<Uuid>,
}

impl Message {
    pub fn new(dialog_id: Uuid, sender_id: Uuid, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(), // Time-ordered UUID for efficient sorting
            dialog_id,
            sender_id,
            content: content.into(),
            sent_at: Utc::now(),
            last_edited_at: None,
            reply_to_id: None,
        }
    }

    pub fn with_reply(mut self, reply_to_id: Uuid) -> Self {
        self.reply_to_id = Some(reply_to_id);
        self
    }

    pub fn is_edited(&self) -> bool {
        self.last_edited_at.is_some()
    }
}
