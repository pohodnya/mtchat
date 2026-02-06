//! Dialog repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::Dialog;

pub struct DialogRepository {
    pool: PgPool,
}

impl DialogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new dialog
    pub async fn create(&self, dialog: &Dialog) -> Result<Dialog, sqlx::Error> {
        sqlx::query_as::<_, Dialog>(
            r#"INSERT INTO dialogs (id, object_id, object_type, title, created_by, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(dialog.id)
        .bind(dialog.object_id)
        .bind(&dialog.object_type)
        .bind(&dialog.title)
        .bind(dialog.created_by)
        .bind(dialog.created_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Find dialog by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Dialog>, sqlx::Error> {
        sqlx::query_as::<_, Dialog>("SELECT * FROM dialogs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// Find dialog by object (type + id)
    pub async fn find_by_object(
        &self,
        object_type: &str,
        object_id: Uuid,
    ) -> Result<Option<Dialog>, sqlx::Error> {
        sqlx::query_as::<_, Dialog>(
            "SELECT * FROM dialogs WHERE object_type = $1 AND object_id = $2",
        )
        .bind(object_type)
        .bind(object_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Find dialogs where user is a direct participant
    ///
    /// - archived: None = all, Some(true) = only archived, Some(false) = only active
    pub async fn find_participating(
        &self,
        user_id: Uuid,
        search: Option<&str>,
        archived: Option<bool>,
    ) -> Result<Vec<Dialog>, sqlx::Error> {
        sqlx::query_as::<_, Dialog>(
            r#"SELECT d.* FROM dialogs d
               INNER JOIN dialog_participants dp ON dp.dialog_id = d.id
               WHERE dp.user_id = $1
                 AND ($2::text IS NULL OR d.title ILIKE '%' || $2 || '%')
                 AND ($3::boolean IS NULL OR dp.is_archived = $3)
               ORDER BY d.created_at DESC"#,
        )
        .bind(user_id)
        .bind(search)
        .bind(archived)
        .fetch_all(&self.pool)
        .await
    }

    /// Find dialogs available to user via scope (not yet participating)
    ///
    /// Matching logic:
    /// - tenant_uid must match exactly
    /// - scope_level1: empty array in DB = wildcard (match all), otherwise requires overlap
    /// - scope_level2: empty array in DB = wildcard (match all), otherwise requires overlap
    pub async fn find_available(
        &self,
        user_id: Uuid,
        tenant_uid: Uuid,
        scope_level1: &[String],
        scope_level2: &[String],
        search: Option<&str>,
    ) -> Result<Vec<Dialog>, sqlx::Error> {
        sqlx::query_as::<_, Dialog>(
            r#"SELECT DISTINCT d.* FROM dialogs d
               INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
               WHERE s.tenant_uid = $1
                 AND (s.scope_level1 = '{}' OR s.scope_level1 && $2)
                 AND (s.scope_level2 = '{}' OR s.scope_level2 && $3)
                 AND NOT EXISTS (
                   SELECT 1 FROM dialog_participants dp
                   WHERE dp.dialog_id = d.id AND dp.user_id = $4
                 )
                 AND ($5::text IS NULL OR d.title ILIKE '%' || $5 || '%')
               ORDER BY d.created_at DESC"#,
        )
        .bind(tenant_uid)
        .bind(scope_level1)
        .bind(scope_level2)
        .bind(user_id)
        .bind(search)
        .fetch_all(&self.pool)
        .await
    }

    /// Delete dialog by ID
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dialogs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Count participants in a dialog
    pub async fn count_participants(&self, dialog_id: Uuid) -> Result<i64, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM dialog_participants WHERE dialog_id = $1",
        )
        .bind(dialog_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    /// Get last message timestamp for a dialog
    pub async fn get_last_message_at(
        &self,
        dialog_id: Uuid,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, sqlx::Error> {
        let result: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
            "SELECT sent_at FROM messages WHERE dialog_id = $1 ORDER BY sent_at DESC LIMIT 1",
        )
        .bind(dialog_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.map(|(ts,)| ts))
    }

    /// Get last message timestamps for multiple dialogs in one query
    pub async fn get_last_message_at_batch(
        &self,
        dialog_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, chrono::DateTime<chrono::Utc>>, sqlx::Error> {
        if dialog_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let rows: Vec<(Uuid, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            r#"SELECT DISTINCT ON (dialog_id) dialog_id, sent_at
               FROM messages
               WHERE dialog_id = ANY($1)
               ORDER BY dialog_id, sent_at DESC"#,
        )
        .bind(dialog_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().collect())
    }
}
