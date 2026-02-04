//! Middleware for authentication and authorization

pub mod admin_auth;
pub mod scope_config;

pub use scope_config::{ScopeConfig, OptionalScopeConfig, UserId};
