//! Repository layer for database access
//!
//! Each repository handles CRUD operations for a specific entity.

mod dialog_repo;
mod participant_repo;
mod scope_repo;
mod message_repo;
mod attachment_repo;

pub use dialog_repo::DialogRepository;
pub use participant_repo::ParticipantRepository;
pub use scope_repo::AccessScopeRepository;
pub use message_repo::MessageRepository;
pub use attachment_repo::AttachmentRepository;

use sqlx::PgPool;

/// Trait for repositories that need database access
pub trait Repository {
    fn pool(&self) -> &PgPool;
}
