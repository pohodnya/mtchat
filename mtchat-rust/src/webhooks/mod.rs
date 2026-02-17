//! Outgoing webhooks for MTChat
//!
//! Sends HTTP notifications to configured endpoints when events occur:
//! - `message.new` - New message sent
//! - `participant.joined` - User joined a dialog
//! - `participant.left` - User left a dialog
//!
//! Webhooks are signed with HMAC-SHA256 for verification.

mod events;
mod sender;

pub use events::{WebhookEvent, WebhookEventType, WebhookPayload};
pub use sender::{WebhookConfig, WebhookSender};
