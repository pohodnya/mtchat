//! Domain models for MTChat
//!
//! Core business entities that represent the chat domain.

mod access_scope;
mod attachment;
mod dialog;
pub mod html_sanitize;
mod message;
mod participant;
pub mod system_messages;

pub use access_scope::DialogAccessScope;
pub use attachment::{
    limits as attachment_limits, Attachment, AttachmentInput, AttachmentResponse, AttachmentType,
};
pub use dialog::Dialog;
pub use html_sanitize::sanitize_html;
pub use message::{Message, MessageType};
pub use participant::{DialogParticipant, JoinedAs, ParticipantProfile};
