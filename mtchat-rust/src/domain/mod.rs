//! Domain models for MTChat
//!
//! Core business entities that represent the chat domain.

mod dialog;
mod participant;
mod access_scope;
mod message;
mod attachment;

pub use dialog::Dialog;
pub use participant::{DialogParticipant, JoinedAs};
pub use access_scope::DialogAccessScope;
pub use message::Message;
pub use attachment::{Attachment, AttachmentResponse, AttachmentInput, AttachmentType, limits as attachment_limits};
