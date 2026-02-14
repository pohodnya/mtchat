//! Background job processing for MTChat.
//!
//! This module provides:
//! - Smart notifications (only notify if message not read after 1 second)
//! - Auto-archiving of inactive dialogs
//!
//! # Architecture
//!
//! Jobs are processed using [apalis](https://docs.rs/apalis) with Redis backend.
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │   API Handler   │────▶│   JobProducer   │────▶│      Redis      │
//! │  (send_message) │     │   (enqueue)     │     │    (storage)    │
//! └─────────────────┘     └─────────────────┘     └────────┬────────┘
//!                                                          │
//!                                                          ▼
//!                         ┌─────────────────────────────────────────┐
//!                         │              Worker                      │
//!                         │  ┌─────────────────────────────────────┐ │
//!                         │  │      handle_notification            │ │
//!                         │  │  1. Check read status (DB)          │ │
//!                         │  │  2. Send webhook if unread          │ │
//!                         │  └─────────────────────────────────────┘ │
//!                         └─────────────────────────────────────────┘
//! ```

pub mod handlers;
pub mod producer;
pub mod types;
pub mod worker;

pub use handlers::JobContext;
pub use producer::JobProducer;
pub use types::NotificationJob;
pub use worker::{start_workers, WorkerConfig};
