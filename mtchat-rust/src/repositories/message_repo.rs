//! Message repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::Message;

pub struct MessageRepository {
    pool: PgPool,
}

impl MessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new message (user or system)
    pub async fn create(&self, message: &Message) -> Result<Message, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            r#"INSERT INTO messages (id, dialog_id, sender_id, content, sent_at, reply_to_id, message_type)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(message.id)
        .bind(message.dialog_id)
        .bind(message.sender_id)
        .bind(&message.content)
        .bind(message.sent_at)
        .bind(message.reply_to_id)
        .bind(message.message_type.as_str())
        .fetch_one(&self.pool)
        .await
    }

    /// Find message by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// Find message by ID and dialog (for access control)
    pub async fn find_by_id_and_dialog(
        &self,
        id: Uuid,
        dialog_id: Uuid,
    ) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE id = $1 AND dialog_id = $2",
        )
        .bind(id)
        .bind(dialog_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// List messages in a dialog with pagination
    pub async fn list_by_dialog(
        &self,
        dialog_id: Uuid,
        limit: i64,
        before: Option<Uuid>,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let messages = if let Some(before_id) = before {
            sqlx::query_as::<_, Message>(
                r#"SELECT * FROM messages
                   WHERE dialog_id = $1 AND id < $2
                   ORDER BY id DESC
                   LIMIT $3"#,
            )
            .bind(dialog_id)
            .bind(before_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Message>(
                r#"SELECT * FROM messages
                   WHERE dialog_id = $1
                   ORDER BY id DESC
                   LIMIT $2"#,
            )
            .bind(dialog_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        // Reverse to get chronological order
        Ok(messages.into_iter().rev().collect())
    }

    /// Save old content to edit history before updating
    pub async fn save_edit_history(
        &self,
        message_id: Uuid,
        old_content: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO message_edit_history (id, message_id, old_content, edited_at)
               VALUES ($1, $2, $3, NOW())"#,
        )
        .bind(Uuid::now_v7())
        .bind(message_id)
        .bind(old_content)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update message content
    pub async fn update_content(
        &self,
        id: Uuid,
        content: &str,
    ) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            r#"UPDATE messages
               SET content = $2, last_edited_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(content)
        .fetch_optional(&self.pool)
        .await
    }

    /// Delete message
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM messages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Count messages in a dialog
    pub async fn count_by_dialog(&self, dialog_id: Uuid) -> Result<i64, sqlx::Error> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM messages WHERE dialog_id = $1")
                .bind(dialog_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    /// Count unread messages for a user in a dialog
    pub async fn count_unread(
        &self,
        dialog_id: Uuid,
        last_read_message_id: Option<Uuid>,
    ) -> Result<i64, sqlx::Error> {
        let (count,): (i64,) = if let Some(last_read) = last_read_message_id {
            sqlx::query_as(
                "SELECT COUNT(*) FROM messages WHERE dialog_id = $1 AND id > $2",
            )
            .bind(dialog_id)
            .bind(last_read)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM messages WHERE dialog_id = $1")
                .bind(dialog_id)
                .fetch_one(&self.pool)
                .await?
        };
        Ok(count)
    }
}
