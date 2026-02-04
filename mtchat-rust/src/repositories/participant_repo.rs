//! Participant repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{DialogParticipant, JoinedAs};

pub struct ParticipantRepository {
    pool: PgPool,
}

impl ParticipantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add a participant to a dialog
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

    /// Update last read message
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
}
