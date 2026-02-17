//! Service layer for MTChat
//!
//! Contains business logic and external service integrations.

mod presence;
mod s3;

pub use presence::PresenceService;
pub use s3::{S3Config, S3Error, S3Service};
