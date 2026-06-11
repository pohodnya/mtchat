//! Dialog repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Dialog, Message};

/// Type alias for external user identifier
type UserId = str;

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
            r#"INSERT INTO dialogs (id, object_id, object_type, title, object_url, created_by, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(dialog.id)
        .bind(&dialog.object_id)
        .bind(&dialog.object_type)
        .bind(&dialog.title)
        .bind(&dialog.object_url)
        .bind(&dialog.created_by)
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

    /// Find the most recent dialog by object (type + id) that the caller can access.
    ///
    /// Multiple dialogs can exist per object (one per access scope, e.g. one chat
    /// per counterparty). This returns the newest dialog the caller can access —
    /// either as a direct participant, or via scope matching (same OR-across-levels
    /// logic as `find_all_by_object_for_user`; empty array in DB = wildcard).
    ///
    /// When `scope` is `None`, only dialogs where the user is a participant are
    /// considered. Returns `None` when no accessible dialog exists — which is
    /// indistinguishable, by design, from "no dialog exists for this object", so
    /// the endpoint never leaks the existence of another tenant's dialog.
    pub async fn find_by_object_for_user(
        &self,
        object_type: &str,
        object_id: &str,
        user_id: &UserId,
        scope: Option<(&[String], &[String], &[String])>,
    ) -> Result<Option<Dialog>, sqlx::Error> {
        let (has_scope, scope_level0, scope_level1, scope_level2) = match scope {
            Some((s0, s1, s2)) => (true, s0, s1, s2),
            None => (false, &[][..], &[][..], &[][..]),
        };

        sqlx::query_as::<_, Dialog>(
            r#"SELECT d.* FROM dialogs d
               WHERE d.object_type = $1 AND d.object_id = $2
                 AND (
                   EXISTS (
                     SELECT 1 FROM dialog_participants dp
                     WHERE dp.dialog_id = d.id AND dp.user_id = $3
                   )
                   OR ($4 AND EXISTS (
                     SELECT 1 FROM dialog_access_scopes s
                     WHERE s.dialog_id = d.id
                       AND (s.scope_level0 = '{}' OR s.scope_level0 && $5)
                       AND (s.scope_level1 = '{}' OR s.scope_level1 && $6)
                       AND (s.scope_level2 = '{}' OR s.scope_level2 && $7)
                   ))
                 )
               ORDER BY d.created_at DESC
               LIMIT 1"#,
        )
        .bind(object_type)
        .bind(object_id)
        .bind(user_id)
        .bind(has_scope)
        .bind(scope_level0)
        .bind(scope_level1)
        .bind(scope_level2)
        .fetch_optional(&self.pool)
        .await
    }

    /// Find all dialogs for an object (type + id), newest first.
    ///
    /// Used by host systems for idempotent "find-or-create": the caller
    /// narrows candidates further by matching access scopes.
    pub async fn find_all_by_object(
        &self,
        object_type: &str,
        object_id: &str,
    ) -> Result<Vec<Dialog>, sqlx::Error> {
        sqlx::query_as::<_, Dialog>(
            "SELECT * FROM dialogs WHERE object_type = $1 AND object_id = $2 ORDER BY created_at DESC",
        )
        .bind(object_type)
        .bind(object_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Find dialogs where user is a direct participant
    ///
    /// - archived: None = all, Some(true) = only archived, Some(false) = only active
    /// - search: searches in dialog title AND participant company names
    /// - limit/offset: pagination parameters
    pub async fn find_participating(
        &self,
        user_id: &UserId,
        search: Option<&str>,
        archived: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Dialog>, sqlx::Error> {
        sqlx::query_as::<_, Dialog>(
            r#"SELECT d.* FROM dialogs d
               INNER JOIN dialog_participants dp ON dp.dialog_id = d.id
               WHERE dp.user_id = $1
                 AND ($2::text IS NULL OR (
                   d.title ILIKE '%' || $2 || '%'
                   OR EXISTS (
                     SELECT 1 FROM dialog_participants p
                     WHERE p.dialog_id = d.id
                       AND p.company ILIKE '%' || $2 || '%'
                   )
                 ))
                 AND ($3::boolean IS NULL OR dp.is_archived = $3)
               ORDER BY d.created_at DESC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(user_id)
        .bind(search)
        .bind(archived)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    /// Find dialogs available to user via scope (not yet participating)
    ///
    /// Matching logic (consistent OR across all levels):
    /// - scope_level0: empty array in DB = wildcard (match all), otherwise requires overlap
    /// - scope_level1: empty array in DB = wildcard (match all), otherwise requires overlap
    /// - scope_level2: empty array in DB = wildcard (match all), otherwise requires overlap
    /// - search: searches in dialog title AND participant company names
    /// - limit/offset: pagination parameters
    #[allow(clippy::too_many_arguments)]
    pub async fn find_available(
        &self,
        user_id: &UserId,
        scope_level0: &[String],
        scope_level1: &[String],
        scope_level2: &[String],
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Dialog>, sqlx::Error> {
        sqlx::query_as::<_, Dialog>(
            r#"SELECT DISTINCT d.* FROM dialogs d
               INNER JOIN dialog_access_scopes s ON s.dialog_id = d.id
               WHERE (s.scope_level0 = '{}' OR s.scope_level0 && $1)
                 AND (s.scope_level1 = '{}' OR s.scope_level1 && $2)
                 AND (s.scope_level2 = '{}' OR s.scope_level2 && $3)
                 AND NOT EXISTS (
                   SELECT 1 FROM dialog_participants dp
                   WHERE dp.dialog_id = d.id AND dp.user_id = $4
                 )
                 AND ($5::text IS NULL OR (
                   d.title ILIKE '%' || $5 || '%'
                   OR EXISTS (
                     SELECT 1 FROM dialog_participants p
                     WHERE p.dialog_id = d.id
                       AND p.company ILIKE '%' || $5 || '%'
                   )
                 ))
               ORDER BY d.created_at DESC
               LIMIT $6 OFFSET $7"#,
        )
        .bind(scope_level0)
        .bind(scope_level1)
        .bind(scope_level2)
        .bind(user_id)
        .bind(search)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    /// Find all dialogs for an object that the user can access
    ///
    /// Returns every dialog for `(object_type, object_id)` where the user is either:
    /// - a direct participant, OR
    /// - able to join via scope matching (same OR-across-levels logic as
    ///   `find_available`; empty array in DB = wildcard).
    ///
    /// When `scope` is `None`, the scope branch is not evaluated and only dialogs
    /// where the user is a participant are returned — consistent with
    /// `get_dialog_by_object`, where `can_join` is `false` without a scope header.
    ///
    /// `archived` filters the participant branch only (`None` = all,
    /// `Some(true)` = only archived, `Some(false)` = only active). Potential
    /// (scope-matched) dialogs have no per-user archived state and are never
    /// filtered by it — consistent with available dialogs never being archived.
    ///
    /// `search` (when `Some`) filters by dialog title OR any participant's company name (case-insensitive `ILIKE`), matching `find_participating`.
    ///
    /// Ordered by creation time (newest first). Returns an empty vec when no
    /// accessible dialogs exist.
    /// List all dialogs for an object the user can access.
    ///
    /// `dialog_type` mirrors `GET /api/v1/dialogs`:
    /// - `Some("participating")` → only dialogs the user is a direct participant of
    /// - `Some("available")`     → only potential (can-join via scope) dialogs
    /// - `None`                  → both branches (default)
    ///
    /// The participant branch honours the `archived` filter; the potential
    /// branch has no per-user archived state and is unaffected by it.
    #[allow(clippy::too_many_arguments)]
    pub async fn find_all_by_object_for_user(
        &self,
        object_type: &str,
        object_id: &str,
        user_id: &UserId,
        scope: Option<(&[String], &[String], &[String])>,
        archived: Option<bool>,
        search: Option<&str>,
        dialog_type: Option<&str>,
    ) -> Result<Vec<Dialog>, sqlx::Error> {
        let (has_scope, scope_level0, scope_level1, scope_level2) = match scope {
            Some((s0, s1, s2)) => (true, s0, s1, s2),
            None => (false, &[][..], &[][..], &[][..]),
        };

        // Which branches of the OR are enabled. `None` enables both.
        let include_participant = matches!(dialog_type, None | Some("participating"));
        let include_potential = matches!(dialog_type, None | Some("available"));

        sqlx::query_as::<_, Dialog>(
            r#"SELECT d.* FROM dialogs d
               WHERE d.object_type = $1 AND d.object_id = $2
                 AND (
                   ($10 AND EXISTS (
                     SELECT 1 FROM dialog_participants dp
                     WHERE dp.dialog_id = d.id AND dp.user_id = $3
                       AND ($8::boolean IS NULL OR dp.is_archived = $8)
                   ))
                   OR ($11 AND $4 AND EXISTS (
                     SELECT 1 FROM dialog_access_scopes s
                     WHERE s.dialog_id = d.id
                       AND (s.scope_level0 = '{}' OR s.scope_level0 && $5)
                       AND (s.scope_level1 = '{}' OR s.scope_level1 && $6)
                       AND (s.scope_level2 = '{}' OR s.scope_level2 && $7)
                   ))
                 )
                 AND ($9::text IS NULL OR (
                   d.title ILIKE '%' || $9 || '%'
                   OR EXISTS (
                     SELECT 1 FROM dialog_participants p
                     WHERE p.dialog_id = d.id
                       AND p.company ILIKE '%' || $9 || '%'
                   )
                 ))
               ORDER BY d.created_at DESC"#,
        )
        .bind(object_type)
        .bind(object_id)
        .bind(user_id)
        .bind(has_scope)
        .bind(scope_level0)
        .bind(scope_level1)
        .bind(scope_level2)
        .bind(archived)
        .bind(search)
        .bind(include_participant)
        .bind(include_potential)
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
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM dialog_participants WHERE dialog_id = $1")
                .bind(dialog_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    /// Count participants for multiple dialogs in one query
    pub async fn count_participants_batch(
        &self,
        dialog_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, i64>, sqlx::Error> {
        if dialog_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let rows: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"SELECT dialog_id, COUNT(*) as cnt
               FROM dialog_participants
               WHERE dialog_id = ANY($1)
               GROUP BY dialog_id"#,
        )
        .bind(dialog_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().collect())
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

    /// Get the full last message for multiple dialogs in one query.
    ///
    /// Returns a map of dialog_id -> latest Message (by sent_at). Dialogs with no
    /// messages are absent from the map. One query, no N+1.
    pub async fn get_last_message_batch(
        &self,
        dialog_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Message>, sqlx::Error> {
        if dialog_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let rows: Vec<Message> = sqlx::query_as::<_, Message>(
            r#"SELECT DISTINCT ON (dialog_id) *
               FROM messages
               WHERE dialog_id = ANY($1)
               ORDER BY dialog_id, sent_at DESC"#,
        )
        .bind(dialog_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|m| (m.dialog_id, m)).collect())
    }

    /// Find dialogs with no messages since the cutoff date.
    ///
    /// Used by auto-archive job to find inactive dialogs.
    pub async fn find_inactive_since(
        &self,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        // Find dialogs where:
        // - No messages exist OR latest message is older than cutoff
        // - At least one participant is not archived (to avoid re-processing)
        sqlx::query_scalar(
            r#"SELECT d.id FROM dialogs d
               WHERE (
                   NOT EXISTS (SELECT 1 FROM messages m WHERE m.dialog_id = d.id)
                   OR (
                       SELECT MAX(m.sent_at) FROM messages m WHERE m.dialog_id = d.id
                   ) < $1
               )
               AND EXISTS (
                   SELECT 1 FROM dialog_participants dp
                   WHERE dp.dialog_id = d.id AND dp.is_archived = false
               )"#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await
    }
}
