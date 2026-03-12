//! Middleware for authentication and authorization

pub mod admin_auth;
pub mod jwt_auth;
pub mod scope_config;

pub use admin_auth::init_admin_token;
pub use jwt_auth::{jwt_auth, JwtClaims, JwtUserId};
pub use scope_config::{OptionalScopeConfig, ScopeConfig, UserId};
