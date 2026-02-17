//! Message entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Message type: user-sent or system-generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    User,
    System,
}

impl MessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::User => "user",
            MessageType::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "system" => MessageType::System,
            _ => MessageType::User,
        }
    }
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::User
    }
}

// Custom sqlx decode for MessageType from VARCHAR
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for MessageType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: &str = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(MessageType::from_str(s))
    }
}

impl sqlx::Type<sqlx::Postgres> for MessageType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        // Compatible with VARCHAR, TEXT, and other string types
        *ty == sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
            || *ty == sqlx::postgres::PgTypeInfo::with_name("TEXT")
            || *ty == <String as sqlx::Type<sqlx::Postgres>>::type_info()
            || *ty == <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

/// A message in a dialog
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub dialog_id: Uuid,
    /// External user identifier (from JWT token). NULL for system messages.
    pub sender_id: Option<Uuid>,
    pub content: String,
    pub sent_at: DateTime<Utc>,
    pub last_edited_at: Option<DateTime<Utc>>,
    /// Reference to the message this is a reply to
    pub reply_to_id: Option<Uuid>,
    /// Message type: 'user' or 'system'
    #[serde(default)]
    pub message_type: MessageType,
}

impl Message {
    /// Create a new user message
    pub fn new(dialog_id: Uuid, sender_id: Uuid, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(), // Time-ordered UUID for efficient sorting
            dialog_id,
            sender_id: Some(sender_id),
            content: content.into(),
            sent_at: Utc::now(),
            last_edited_at: None,
            reply_to_id: None,
            message_type: MessageType::User,
        }
    }

    /// Create a system message (no sender)
    pub fn system(dialog_id: Uuid, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            dialog_id,
            sender_id: None,
            content: content.into(),
            sent_at: Utc::now(),
            last_edited_at: None,
            reply_to_id: None,
            message_type: MessageType::System,
        }
    }

    pub fn with_reply(mut self, reply_to_id: Uuid) -> Self {
        self.reply_to_id = Some(reply_to_id);
        self
    }

    pub fn is_edited(&self) -> bool {
        self.last_edited_at.is_some()
    }

    pub fn is_system(&self) -> bool {
        self.message_type == MessageType::System
    }
}
