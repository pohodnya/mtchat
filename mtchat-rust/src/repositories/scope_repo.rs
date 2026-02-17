//! Access scope repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::DialogAccessScope;

pub struct AccessScopeRepository {
    pool: PgPool,
}

impl AccessScopeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new access scope
    pub async fn create(
        &self,
        scope: &DialogAccessScope,
    ) -> Result<DialogAccessScope, sqlx::Error> {
        sqlx::query_as::<_, DialogAccessScope>(
            r#"INSERT INTO dialog_access_scopes (id, dialog_id, tenant_uid, scope_level1, scope_level2, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(scope.id)
        .bind(scope.dialog_id)
        .bind(scope.tenant_uid)
        .bind(&scope.scope_level1)
        .bind(&scope.scope_level2)
        .bind(scope.created_at)
        .fetch_one(&self.pool)
        .await
    }

    /// Find all scopes for a dialog
    pub async fn find_by_dialog(
        &self,
        dialog_id: Uuid,
    ) -> Result<Vec<DialogAccessScope>, sqlx::Error> {
        sqlx::query_as::<_, DialogAccessScope>(
            "SELECT * FROM dialog_access_scopes WHERE dialog_id = $1",
        )
        .bind(dialog_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Check if user has scope access to a dialog
    ///
    /// Matching logic:
    /// - tenant_uid must match exactly
    /// - scope_level1: empty array in DB = wildcard (match all), otherwise requires overlap
    /// - scope_level2: empty array in DB = wildcard (match all), otherwise requires overlap
    pub async fn check_access(
        &self,
        dialog_id: Uuid,
        tenant_uid: Uuid,
        scope_level1: &[String],
        scope_level2: &[String],
    ) -> Result<bool, sqlx::Error> {
        let result: Option<(i32,)> = sqlx::query_as(
            r#"SELECT 1 FROM dialog_access_scopes
               WHERE dialog_id = $1
                 AND tenant_uid = $2
                 AND (scope_level1 = '{}' OR scope_level1 && $3)
                 AND (scope_level2 = '{}' OR scope_level2 && $4)"#,
        )
        .bind(dialog_id)
        .bind(tenant_uid)
        .bind(scope_level1)
        .bind(scope_level2)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.is_some())
    }

    /// Delete all scopes for a dialog
    pub async fn delete_by_dialog(&self, dialog_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dialog_access_scopes WHERE dialog_id = $1")
            .bind(dialog_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Replace all scopes for a dialog (delete + insert) atomically
    pub async fn replace_for_dialog(
        &self,
        dialog_id: Uuid,
        scopes: Vec<DialogAccessScope>,
    ) -> Result<Vec<DialogAccessScope>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Delete existing
        sqlx::query("DELETE FROM dialog_access_scopes WHERE dialog_id = $1")
            .bind(dialog_id)
            .execute(&mut *tx)
            .await?;

        // Insert new
        let mut result = Vec::new();
        for scope in scopes {
            let created = sqlx::query_as::<_, DialogAccessScope>(
                r#"INSERT INTO dialog_access_scopes (id, dialog_id, tenant_uid, scope_level1, scope_level2, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   RETURNING *"#,
            )
            .bind(scope.id)
            .bind(scope.dialog_id)
            .bind(scope.tenant_uid)
            .bind(&scope.scope_level1)
            .bind(&scope.scope_level2)
            .bind(scope.created_at)
            .fetch_one(&mut *tx)
            .await?;
            result.push(created);
        }

        tx.commit().await?;
        Ok(result)
    }

    /// Delete a specific scope by ID
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM dialog_access_scopes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
