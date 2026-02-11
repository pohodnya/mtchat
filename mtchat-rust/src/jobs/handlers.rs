//! Job handlers for background task processing.

use std::sync::Arc;

use apalis::prelude::*;
use chrono::{Duration, Utc};
use fred::clients::Pool as RedisPool;
use sqlx::PgPool;

use super::middleware::{cleanup_debounce_key, is_job_current, NOTIFICATION_DEBOUNCE_PREFIX};
use super::types::{AutoArchiveJob, NotificationJob};
use crate::repositories::{DialogRepository, MessageRepository, ParticipantRepository};
use crate::webhooks::{WebhookEvent, WebhookSender};

/// Shared context for job handlers.
#[derive(Clone)]
pub struct JobContext {
    pub db: PgPool,
    pub redis: Arc<RedisPool>,
    pub dialogs: Arc<DialogRepository>,
    pub participants: Arc<ParticipantRepository>,
    pub messages: Arc<MessageRepository>,
    pub webhooks: WebhookSender,
    /// Seconds of inactivity before auto-archive (default: 259200 = 3 days)
    pub archive_after_secs: i64,
}

/// Handle notification job.
///
/// Checks if the message has been read by the recipient.
/// If not read and notifications are enabled, sends a webhook.
pub async fn handle_notification(job: NotificationJob, ctx: Data<JobContext>) -> Result<(), Error> {
    tracing::debug!(
        dialog_id = %job.dialog_id,
        recipient_id = %job.recipient_id,
        message_id = %job.message_id,
        job_id = %job.job_id,
        "Processing notification job"
    );

    // Check debounce - is this job still current?
    let debounce_key = job.debounce_key();
    if !is_job_current(&ctx.redis, NOTIFICATION_DEBOUNCE_PREFIX, &debounce_key, &job.job_id).await {
        // Job was superseded by newer one, skip
        return Ok(());
    }

    // Clean up debounce key
    cleanup_debounce_key(&ctx.redis, NOTIFICATION_DEBOUNCE_PREFIX, &debounce_key).await;

    // Get participant to check read status
    let participant = match ctx.participants.find(job.dialog_id, job.recipient_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            tracing::debug!(
                recipient_id = %job.recipient_id,
                "Recipient no longer participant, skipping notification"
            );
            return Ok(());
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get participant");
            return Err(Error::Failed(Arc::new(Box::new(e))));
        }
    };

    // Check if notifications are enabled for this user
    if !participant.notifications_enabled {
        tracing::debug!(
            recipient_id = %job.recipient_id,
            "Notifications disabled for user, skipping"
        );
        return Ok(());
    }

    // Check if message has been read
    // If unread_count is 0, message was read
    if participant.unread_count == 0 {
        tracing::debug!(
            recipient_id = %job.recipient_id,
            message_id = %job.message_id,
            "Message already read, skipping notification"
        );
        return Ok(());
    }

    // Message not read - send notification webhook
    tracing::info!(
        recipient_id = %job.recipient_id,
        message_id = %job.message_id,
        "Sending notification webhook"
    );

    // Load dialog and message for webhook payload
    let dialog = match ctx.dialogs.find_by_id(job.dialog_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            tracing::warn!(dialog_id = %job.dialog_id, "Dialog not found");
            return Ok(());
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get dialog");
            return Err(Error::Failed(Arc::new(Box::new(e))));
        }
    };

    let message = match ctx.messages.find_by_id(job.message_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            tracing::warn!(message_id = %job.message_id, "Message not found");
            return Ok(());
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to get message");
            return Err(Error::Failed(Arc::new(Box::new(e))));
        }
    };

    // Send webhook with notification info
    ctx.webhooks
        .send(WebhookEvent::notification_pending(
            &dialog,
            &message,
            job.recipient_id,
        ))
        .await;

    Ok(())
}

/// Handle auto-archive job.
///
/// Finds dialogs with no activity for N seconds and archives them.
pub async fn handle_auto_archive(job: AutoArchiveJob, ctx: Data<JobContext>) -> Result<(), Error> {
    tracing::info!(run_id = %job.run_id, "Starting auto-archive job");

    let cutoff = Utc::now() - Duration::seconds(ctx.archive_after_secs);

    // Find inactive dialogs
    let inactive_dialogs = match ctx.dialogs.find_inactive_since(cutoff).await {
        Ok(dialogs) => dialogs,
        Err(e) => {
            tracing::error!(error = %e, "Failed to find inactive dialogs");
            return Err(Error::Failed(Arc::new(Box::new(e))));
        }
    };

    if inactive_dialogs.is_empty() {
        tracing::info!(run_id = %job.run_id, "No inactive dialogs found");
        return Ok(());
    }

    tracing::info!(
        run_id = %job.run_id,
        count = inactive_dialogs.len(),
        "Found inactive dialogs to archive"
    );

    let mut archived_count = 0;
    for dialog_id in inactive_dialogs {
        match ctx.participants.archive_all_for_dialog(dialog_id).await {
            Ok(count) => {
                archived_count += count;
                tracing::debug!(dialog_id = %dialog_id, participants = count, "Archived dialog");
            }
            Err(e) => {
                tracing::warn!(dialog_id = %dialog_id, error = %e, "Failed to archive dialog");
            }
        }
    }

    tracing::info!(
        run_id = %job.run_id,
        archived_participants = archived_count,
        "Auto-archive job completed"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    // Tests require database fixtures - see integration tests
}
