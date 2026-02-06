//! Domain models for MTChat
//!
//! Core business entities that represent the chat domain.

mod dialog;
mod participant;
mod access_scope;
mod message;
mod attachment;
pub mod system_messages;
pub mod html_sanitize;

pub use dialog::Dialog;
pub use participant::{DialogParticipant, JoinedAs, ParticipantProfile};
pub use access_scope::DialogAccessScope;
pub use message::{Message, MessageType};
pub use attachment::{Attachment, AttachmentResponse, AttachmentInput, AttachmentType, limits as attachment_limits};
pub use html_sanitize::sanitize_html;
