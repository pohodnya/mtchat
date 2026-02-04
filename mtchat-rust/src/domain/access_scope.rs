//! Dialog access scope entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Access scope rule for potential participants.
///
/// Defines who can see and join a dialog based on:
/// - tenant_uid: must match exactly
/// - scope_level1: user must have at least one matching value (e.g., departments)
/// - scope_level2: user must have at least one matching value (e.g., permissions)
///
/// Matching logic: `tenant AND (ANY scope1) AND (ANY scope2)`
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DialogAccessScope {
    pub id: Uuid,
    pub dialog_id: Uuid,
    /// Tenant identifier that this scope applies to
    pub tenant_uid: Uuid,
    /// First scope level (e.g., departments): ["dept_logistics", "dept_sales"]
    pub scope_level1: Vec<String>,
    /// Second scope level (e.g., permissions): ["tender:manager", "tender:admin"]
    pub scope_level2: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl DialogAccessScope {
    pub fn new(
        dialog_id: Uuid,
        tenant_uid: Uuid,
        scope_level1: Vec<String>,
        scope_level2: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            dialog_id,
            tenant_uid,
            scope_level1,
            scope_level2,
            created_at: Utc::now(),
        }
    }

    /// Check if user scope matches this access scope
    pub fn matches(&self, user_tenant: Uuid, user_scope1: &[String], user_scope2: &[String]) -> bool {
        if self.tenant_uid != user_tenant {
            return false;
        }

        let scope1_match = self.scope_level1.is_empty()
            || self.scope_level1.iter().any(|s| user_scope1.contains(s));

        let scope2_match = self.scope_level2.is_empty()
            || self.scope_level2.iter().any(|s| user_scope2.contains(s));

        scope1_match && scope2_match
    }
}

/// User's scope configuration (from JWT token or header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScopeConfig {
    pub tenant_uid: Uuid,
    #[serde(default)]
    pub scope_level1: Vec<String>,
    #[serde(default)]
    pub scope_level2: Vec<String>,
}
