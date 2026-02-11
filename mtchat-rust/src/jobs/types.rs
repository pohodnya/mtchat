//! Job types for background task processing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification job - sends webhook after delay if message not read.
///
/// Supports debouncing: if multiple messages are sent to the same recipient
/// in quick succession, only the last notification is sent.
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
    /// Unique job ID for debounce tracking
    pub job_id: String,
}

impl NotificationJob {
    /// Create a new notification job.
    pub fn new(dialog_id: Uuid, recipient_id: Uuid, message_id: Uuid, sender_id: Uuid) -> Self {
        Self {
            dialog_id,
            recipient_id,
            message_id,
            sender_id,
            job_id: Uuid::now_v7().to_string(),
        }
    }

    /// Get debounce key for this job.
    ///
    /// Jobs with the same debounce key will replace each other.
    pub fn debounce_key(&self) -> String {
        format!("{}:{}", self.dialog_id, self.recipient_id)
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
    fn test_notification_job_debounce_key() {
        let dialog_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        let job = NotificationJob::new(dialog_id, recipient_id, message_id, sender_id);

        assert_eq!(job.debounce_key(), format!("{}:{}", dialog_id, recipient_id));
    }

    #[test]
    fn test_notification_job_unique_job_id() {
        let dialog_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        let job1 = NotificationJob::new(dialog_id, recipient_id, message_id, sender_id);
        let job2 = NotificationJob::new(dialog_id, recipient_id, message_id, sender_id);

        // Each job gets unique ID
        assert_ne!(job1.job_id, job2.job_id);
    }

    #[test]
    fn test_notification_job_same_debounce_key_for_same_recipient() {
        let dialog_id = Uuid::new_v4();
        let recipient_id = Uuid::new_v4();

        let job1 = NotificationJob::new(dialog_id, recipient_id, Uuid::new_v4(), Uuid::new_v4());
        let job2 = NotificationJob::new(dialog_id, recipient_id, Uuid::new_v4(), Uuid::new_v4());

        // Same dialog + recipient = same debounce key
        assert_eq!(job1.debounce_key(), job2.debounce_key());
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
        assert_eq!(job.job_id, deserialized.job_id);
    }

    #[test]
    fn test_auto_archive_job_creation() {
        let job = AutoArchiveJob::new();
        assert!(!job.run_id.is_nil());
    }
}
