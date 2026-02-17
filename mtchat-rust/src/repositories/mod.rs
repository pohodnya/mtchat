//! Repository layer for database access
//!
//! Each repository handles CRUD operations for a specific entity.

mod attachment_repo;
mod dialog_repo;
mod message_repo;
mod participant_repo;
mod scope_repo;

pub use attachment_repo::AttachmentRepository;
pub use dialog_repo::DialogRepository;
pub use message_repo::MessageRepository;
pub use participant_repo::ParticipantRepository;
pub use scope_repo::AccessScopeRepository;

use sqlx::PgPool;

/// Trait for repositories that need database access
pub trait Repository {
    fn pool(&self) -> &PgPool;
}
