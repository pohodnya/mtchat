//! Middleware utilities for job processing.

mod debounce;

pub use debounce::{cleanup_debounce_key, is_job_current, NOTIFICATION_DEBOUNCE_PREFIX};
