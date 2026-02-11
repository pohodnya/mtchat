//! Participant repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{DialogParticipant, JoinedAs, ParticipantProfile};

pub struct ParticipantRepository {
    pool: PgPool,
}

impl ParticipantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add a participant to a dialog (without profile)
    pub async fn add(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        joined_as: JoinedAs,
    ) -> Result<DialogParticipant, sqlx::Error> {
        sqlx::query_as::<_, DialogParticipant>(
            r#"INSERT INTO dialog_participants (dialog_id, user_id, joined_as, joined_at)
               VALUES ($1, $2, $3, NOW())
               RETURNING *"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(joined_as.as_str())
        .fetch_one(&self.pool)
        .await
    }

    /// Add a participant with profile information
    pub async fn add_with_profile(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        joined_as: JoinedAs,
        profile: &ParticipantProfile,
    ) -> Result<DialogParticipant, sqlx::Error> {
        sqlx::query_as::<_, DialogParticipant>(
            r#"INSERT INTO dialog_participants
               (dialog_id, user_id, joined_as, joined_at, display_name, company, email, phone)
               VALUES ($1, $2, $3, NOW(), $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(joined_as.as_str())
        .bind(&profile.display_name)
        .bind(&profile.company)
        .bind(&profile.email)
        .bind(&profile.phone)
        .fetch_one(&self.pool)
        .await
    }

    /// Add participant, ignore if already exists
    pub async fn add_if_not_exists(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        joined_as: JoinedAs,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO dialog_participants (dialog_id, user_id, joined_as, joined_at)
               VALUES ($1, $2, $3, NOW())
               ON CONFLICT (dialog_id, user_id) DO NOTHING"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(joined_as.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Add participant with profile, ignore if already exists
    pub async fn add_with_profile_if_not_exists(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        joined_as: JoinedAs,
        profile: &ParticipantProfile,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO dialog_participants
               (dialog_id, user_id, joined_as, joined_at, display_name, company, email, phone)
               VALUES ($1, $2, $3, NOW(), $4, $5, $6, $7)
               ON CONFLICT (dialog_id, user_id) DO NOTHING"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(joined_as.as_str())
        .bind(&profile.display_name)
        .bind(&profile.company)
        .bind(&profile.email)
        .bind(&profile.phone)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Remove a participant from a dialog
    pub async fn remove(&self, dialog_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM dialog_participants WHERE dialog_id = $1 AND user_id = $2",
        )
        .bind(dialog_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Check if user is a participant
    pub async fn exists(&self, dialog_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result: Option<(i32,)> = sqlx::query_as(
            "SELECT 1 FROM dialog_participants WHERE dialog_id = $1 AND user_id = $2",
        )
        .bind(dialog_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.is_some())
    }

    /// Find participant by dialog and user
    pub async fn find(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<DialogParticipant>, sqlx::Error> {
        sqlx::query_as::<_, DialogParticipant>(
            "SELECT * FROM dialog_participants WHERE dialog_id = $1 AND user_id = $2",
        )
        .bind(dialog_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// List all participants of a dialog
    pub async fn list_by_dialog(
        &self,
        dialog_id: Uuid,
    ) -> Result<Vec<DialogParticipant>, sqlx::Error> {
        sqlx::query_as::<_, DialogParticipant>(
            "SELECT * FROM dialog_participants WHERE dialog_id = $1 ORDER BY joined_at",
        )
        .bind(dialog_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Update notifications setting
    pub async fn set_notifications(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        enabled: bool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET notifications_enabled = $3
               WHERE dialog_id = $1 AND user_id = $2"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(enabled)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Update last read message (legacy - use mark_as_read instead)
    pub async fn update_last_read(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        message_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET last_read_message_id = $3
               WHERE dialog_id = $1 AND user_id = $2"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(message_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Mark messages as read: reset unread_count and update last_read_message_id
    pub async fn mark_as_read(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        last_read_message_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET unread_count = 0, last_read_message_id = $3
               WHERE dialog_id = $1 AND user_id = $2"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(last_read_message_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Increment unread_count for all participants except the author
    pub async fn increment_unread(
        &self,
        dialog_id: Uuid,
        exclude_user_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET unread_count = unread_count + 1
               WHERE dialog_id = $1 AND user_id != $2"#,
        )
        .bind(dialog_id)
        .bind(exclude_user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Set unread_count to message count when user joins existing dialog
    pub async fn set_unread_count_from_messages(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET unread_count = (SELECT COUNT(*) FROM messages WHERE dialog_id = $1)
               WHERE dialog_id = $1 AND user_id = $2"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Archive or unarchive a dialog for a specific user
    pub async fn set_archived(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        archived: bool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET is_archived = $3
               WHERE dialog_id = $1 AND user_id = $2"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(archived)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Get all dialog IDs that a user participates in
    pub async fn get_user_dialogs(&self, user_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        sqlx::query_scalar("SELECT dialog_id FROM dialog_participants WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
    }

    /// Get all distinct user IDs who participate in any of the given dialogs
    pub async fn get_dialog_participants_user_ids(
        &self,
        dialog_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        if dialog_ids.is_empty() {
            return Ok(vec![]);
        }

        sqlx::query_scalar(
            "SELECT DISTINCT user_id FROM dialog_participants WHERE dialog_id = ANY($1)",
        )
        .bind(dialog_ids)
        .fetch_all(&self.pool)
        .await
    }

    /// Pin or unpin a dialog for a specific user
    pub async fn set_pinned(
        &self,
        dialog_id: Uuid,
        user_id: Uuid,
        pinned: bool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET is_pinned = $3
               WHERE dialog_id = $1 AND user_id = $2"#,
        )
        .bind(dialog_id)
        .bind(user_id)
        .bind(pinned)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Archive dialog for all participants.
    ///
    /// Used by auto-archive job. Returns number of participants affected.
    pub async fn archive_all_for_dialog(&self, dialog_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET is_archived = true
               WHERE dialog_id = $1 AND is_archived = false"#,
        )
        .bind(dialog_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Unarchive dialog for all participants.
    ///
    /// Used for auto-unarchive on new message. Returns number of participants affected.
    pub async fn unarchive_all_for_dialog(&self, dialog_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE dialog_participants
               SET is_archived = false
               WHERE dialog_id = $1 AND is_archived = true"#,
        )
        .bind(dialog_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
