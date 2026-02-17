//! Attachment repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::Attachment;

pub struct AttachmentRepository {
    pool: PgPool,
}

impl AttachmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new attachment
    pub async fn create(&self, attachment: &Attachment) -> Result<Attachment, sqlx::Error> {
        sqlx::query_as::<_, Attachment>(
            r#"INSERT INTO attachments (id, message_id, filename, content_type, size, s3_key, width, height, thumbnail_s3_key, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#,
        )
        .bind(attachment.id)
        .bind(attachment.message_id)
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size)
        .bind(&attachment.s3_key)
        .bind(attachment.width)
        .bind(attachment.height)
        .bind(&attachment.thumbnail_s3_key)
        .bind(attachment.created_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Create multiple attachments in a single transaction
    pub async fn create_many(
        &self,
        attachments: &[Attachment],
    ) -> Result<Vec<Attachment>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::with_capacity(attachments.len());

        for attachment in attachments {
            let result = sqlx::query_as::<_, Attachment>(
                r#"INSERT INTO attachments (id, message_id, filename, content_type, size, s3_key, width, height, thumbnail_s3_key, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   RETURNING *"#,
            )
            .bind(attachment.id)
            .bind(attachment.message_id)
            .bind(&attachment.filename)
            .bind(&attachment.content_type)
            .bind(attachment.size)
            .bind(&attachment.s3_key)
            .bind(attachment.width)
            .bind(attachment.height)
            .bind(&attachment.thumbnail_s3_key)
            .bind(attachment.created_at)
            .fetch_one(&mut *tx)
            .await?;

            created.push(result);
        }

        tx.commit().await?;
        Ok(created)
    }

    /// Find attachment by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Attachment>, sqlx::Error> {
        sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// Find attachment by S3 key
    pub async fn find_by_s3_key(&self, s3_key: &str) -> Result<Option<Attachment>, sqlx::Error> {
        sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE s3_key = $1")
            .bind(s3_key)
            .fetch_optional(&self.pool)
            .await
    }

    /// List attachments by message ID
    pub async fn list_by_message(&self, message_id: Uuid) -> Result<Vec<Attachment>, sqlx::Error> {
        sqlx::query_as::<_, Attachment>(
            "SELECT * FROM attachments WHERE message_id = $1 ORDER BY created_at ASC",
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await
    }

    /// List attachments for multiple messages (batch fetch)
    pub async fn list_by_messages(
        &self,
        message_ids: &[Uuid],
    ) -> Result<Vec<Attachment>, sqlx::Error> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        sqlx::query_as::<_, Attachment>(
            "SELECT * FROM attachments WHERE message_id = ANY($1) ORDER BY message_id, created_at ASC",
        )
        .bind(message_ids)
        .fetch_all(&self.pool)
        .await
    }

    /// Update attachment with image metadata
    pub async fn update_image_metadata(
        &self,
        id: Uuid,
        width: i32,
        height: i32,
        thumbnail_s3_key: Option<&str>,
    ) -> Result<Option<Attachment>, sqlx::Error> {
        sqlx::query_as::<_, Attachment>(
            r#"UPDATE attachments
               SET width = $2, height = $3, thumbnail_s3_key = $4
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(width)
        .bind(height)
        .bind(thumbnail_s3_key)
        .fetch_optional(&self.pool)
        .await
    }

    /// Delete attachment
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM attachments WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Delete attachments by message ID
    pub async fn delete_by_message(&self, message_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM attachments WHERE message_id = $1")
            .bind(message_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Count attachments by message
    pub async fn count_by_message(&self, message_id: Uuid) -> Result<i64, sqlx::Error> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM attachments WHERE message_id = $1")
                .bind(message_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    /// Check if s3_key exists (for validation)
    pub async fn s3_key_exists(&self, s3_key: &str) -> Result<bool, sqlx::Error> {
        let (exists,): (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM attachments WHERE s3_key = $1)")
                .bind(s3_key)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }
}
