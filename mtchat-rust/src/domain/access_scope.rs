//! Dialog access scope entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Access scope rule for potential participants.
///
/// Defines who can see and join a dialog based on:
/// - scope_level0: user must have at least one matching value (e.g., tenants/organizations)
/// - scope_level1: user must have at least one matching value (e.g., departments)
/// - scope_level2: user must have at least one matching value (e.g., permissions)
///
/// Matching logic: `(ANY scope0) AND (ANY scope1) AND (ANY scope2)`
/// Empty array at any level = wildcard (matches all values)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DialogAccessScope {
    pub id: Uuid,
    pub dialog_id: Uuid,
    /// Top scope level (e.g., tenants/organizations): ["tenant-a", "tenant-b"]
    /// Empty array = wildcard (matches all)
    pub scope_level0: Vec<String>,
    /// First scope level (e.g., departments): ["dept_logistics", "dept_sales"]
    pub scope_level1: Vec<String>,
    /// Second scope level (e.g., permissions): ["tender:manager", "tender:admin"]
    pub scope_level2: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl DialogAccessScope {
    pub fn new(
        dialog_id: Uuid,
        scope_level0: Vec<String>,
        scope_level1: Vec<String>,
        scope_level2: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            dialog_id,
            scope_level0,
            scope_level1,
            scope_level2,
            created_at: Utc::now(),
        }
    }

    /// Check if user scope matches this access scope.
    ///
    /// For each level, empty array in DB = wildcard (matches any user value).
    /// Otherwise, requires at least one overlapping value.
    pub fn matches(
        &self,
        user_scope0: &[String],
        user_scope1: &[String],
        user_scope2: &[String],
    ) -> bool {
        let scope0_match = self.scope_level0.is_empty()
            || self.scope_level0.iter().any(|s| user_scope0.contains(s));

        let scope1_match = self.scope_level1.is_empty()
            || self.scope_level1.iter().any(|s| user_scope1.contains(s));

        let scope2_match = self.scope_level2.is_empty()
            || self.scope_level2.iter().any(|s| user_scope2.contains(s));

        scope0_match && scope1_match && scope2_match
    }
}

/// User's scope configuration (from JWT token or header)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScopeConfig {
    #[serde(default)]
    pub scope_level0: Vec<String>,
    #[serde(default)]
    pub scope_level1: Vec<String>,
    #[serde(default)]
    pub scope_level2: Vec<String>,
}
