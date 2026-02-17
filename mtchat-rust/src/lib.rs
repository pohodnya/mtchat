//! MTChat API - Embeddable Chat Service Backend
//!
//! Object-bound chat service with direct and potential participants.
//! This library crate exposes internal modules for integration testing.

pub mod api;
pub mod domain;
pub mod jobs;
pub mod middleware;
pub mod repositories;
pub mod services;
pub mod webhooks;
pub mod ws;
