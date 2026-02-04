//! Dialog participant entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// How the participant joined the dialog
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinedAs {
    /// Created the dialog
    Creator,
    /// Was added when dialog was created
    Participant,
    /// Joined via scope access
    Joined,
}

impl JoinedAs {
    pub fn as_str(&self) -> &'static str {
        match self {
            JoinedAs::Creator => "creator",
            JoinedAs::Participant => "participant",
            JoinedAs::Joined => "joined",
        }
    }
}

impl From<String> for JoinedAs {
    fn from(s: String) -> Self {
        match s.as_str() {
            "creator" => JoinedAs::Creator,
            "participant" => JoinedAs::Participant,
            "joined" => JoinedAs::Joined,
            _ => JoinedAs::Participant,
        }
    }
}

impl From<&str> for JoinedAs {
    fn from(s: &str) -> Self {
        JoinedAs::from(s.to_string())
    }
}

/// A direct participant of a dialog.
///
/// Direct participants can:
/// - See the dialog in "My Chats" list
/// - Send and receive messages
/// - Receive notifications (if enabled)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DialogParticipant {
    pub dialog_id: Uuid,
    /// External user identifier (from JWT token)
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    /// How the user joined: "creator", "participant", "joined"
    pub joined_as: String,
    pub notifications_enabled: bool,
    /// Last message the user has read
    pub last_read_message_id: Option<Uuid>,
}

impl DialogParticipant {
    pub fn new(dialog_id: Uuid, user_id: Uuid, joined_as: JoinedAs) -> Self {
        Self {
            dialog_id,
            user_id,
            joined_at: Utc::now(),
            joined_as: joined_as.as_str().to_string(),
            notifications_enabled: true,
            last_read_message_id: None,
        }
    }

    pub fn joined_as_enum(&self) -> JoinedAs {
        JoinedAs::from(self.joined_as.as_str())
    }
}
