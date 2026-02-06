//! Service layer for MTChat
//!
//! Contains business logic and external service integrations.

mod s3;
mod presence;

pub use s3::{S3Service, S3Config, S3Error};
pub use presence::PresenceService;
