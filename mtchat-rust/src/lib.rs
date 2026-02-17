//! MTChat API - Embeddable Chat Service Backend
//!
//! Object-bound chat service with direct and potential participants.
//! This library crate exposes internal modules for integration testing.

pub mod api;
pub mod ws;
pub mod domain;
pub mod repositories;
pub mod middleware;
pub mod webhooks;
pub mod services;
pub mod jobs;
