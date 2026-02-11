//! Background job processing for MTChat.
//!
//! This module provides:
//! - Smart notifications with debounce (only notify if message not read)
//! - Auto-archiving of inactive dialogs
//!
//! # Architecture
//!
//! Jobs are processed using [apalis](https://docs.rs/apalis) with Redis backend.
//! Debounce is implemented by storing job_id in Redis and checking it before execution.
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
//!                         │  ┌─────────────────────────────────┐    │
//!                         │  │      handle_notification        │    │
//!                         │  │  1. Check debounce (Redis)      │    │
//!                         │  │  2. Check read status (DB)      │    │
//!                         │  │  3. Send webhook if unread      │    │
//!                         │  └─────────────────────────────────┘    │
//!                         └─────────────────────────────────────────┘
//! ```

pub mod handlers;
pub mod middleware;
pub mod producer;
pub mod types;
pub mod worker;

pub use handlers::JobContext;
pub use producer::JobProducer;
pub use types::NotificationJob;
pub use worker::{start_workers, WorkerConfig};
