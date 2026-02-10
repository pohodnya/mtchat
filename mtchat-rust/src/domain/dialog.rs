//! Dialog entity - object-bound chat room

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A dialog (chat room) bound to a specific business object.
///
/// Each dialog is uniquely identified by (object_id, object_type) pair.
/// For example: tender/123, order/456, route/789
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dialog {
    pub id: Uuid,
    /// External object ID this dialog is bound to
    pub object_id: Uuid,
    /// Type of the object: "tender", "order", "route", etc.
    pub object_type: String,
    /// Optional human-readable title
    pub title: Option<String>,
    /// URL to the object page in the host system
    pub object_url: Option<String>,
    /// User ID who created this dialog (external identifier)
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Dialog {
    /// Creates a new Dialog with generated ID and current timestamp
    pub fn new(
        object_id: Uuid,
        object_type: impl Into<String>,
        title: Option<String>,
        object_url: Option<String>,
        created_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            object_id,
            object_type: object_type.into(),
            title,
            object_url,
            created_by,
            created_at: Utc::now(),
        }
    }
}
