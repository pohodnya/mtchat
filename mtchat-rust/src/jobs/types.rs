//! Job types for background task processing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification job - sends webhook after short delay if message not read.
///
/// The delay allows checking if user read the message while in chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationJob {
    /// Dialog where message was sent
    pub dialog_id: Uuid,
    /// User who should receive notification
    pub recipient_id: Uuid,
    /// Message that triggered the notification
    pub message_id: Uuid,
    /// User who sent the message
    pub sender_id: Uuid,
}

impl NotificationJob {
    /// Create a new notification job.
    pub fn new(dialog_id: Uuid, recipient_id: Uuid, message_id: Uuid, sender_id: Uuid) -> Self {
        Self {
            dialog_id,
            recipient_id,
            message_id,
            sender_id,
        }
    }
}

/// Auto-archive job - archives inactive dialogs.
///
/// Runs on a cron schedule, finds dialogs with no activity
/// for N days and archives them for all participants.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoArchiveJob {
    /// Unique run ID for logging
    pub run_id: Uuid,
    /// When this job was scheduled (used by cron)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<DateTime<Utc>>,
}

impl AutoArchiveJob {
    pub fn new() -> Self {
        Self {
            run_id: Uuid::now_v7(),
            scheduled_at: None,
        }
    }
}

/// Required by apalis-cron for scheduled job creation.
impl From<DateTime<Utc>> for AutoArchiveJob {
    fn from(scheduled_at: DateTime<Utc>) -> Self {
        Self {
            run_id: Uuid::now_v7(),
            scheduled_at: Some(scheduled_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_job_creation() {
        let dialog_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        let job = NotificationJob::new(dialog_id, recipient_id, message_id, sender_id);

        assert_eq!(job.dialog_id, dialog_id);
        assert_eq!(job.recipient_id, recipient_id);
        assert_eq!(job.message_id, message_id);
        assert_eq!(job.sender_id, sender_id);
    }

    #[test]
    fn test_notification_job_serialization() {
        let job = NotificationJob::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: NotificationJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.dialog_id, deserialized.dialog_id);
        assert_eq!(job.recipient_id, deserialized.recipient_id);
        assert_eq!(job.message_id, deserialized.message_id);
        assert_eq!(job.sender_id, deserialized.sender_id);
    }

    #[test]
    fn test_auto_archive_job_creation() {
        let job = AutoArchiveJob::new();
        assert!(!job.run_id.is_nil());
    }
}
