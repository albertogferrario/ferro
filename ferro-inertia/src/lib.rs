//! # Ferro Inertia
//!
//! Server-side [Inertia.js](https://inertiajs.com) adapter for Rust web frameworks.
//!
//! This crate provides the core functionality for building Inertia.js responses
//! in Rust. It is framework-agnostic and can be integrated with any Rust web
//! framework (Axum, Actix-web, Rocket, etc.).
//!
//! ## Features
//!
//! - Framework-agnostic design via traits
//! - Async-safe (no thread-local storage)
//! - Partial reload support
//! - Shared props for auth, flash messages, CSRF tokens
//! - Version conflict detection (409 responses)
//! - Development mode with Vite HMR support
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ferro_inertia::{Inertia, InertiaConfig, InertiaRequest};
//!
//! // Implement InertiaRequest for your framework's request type
//! impl InertiaRequest for MyRequest {
//!     fn inertia_header(&self, name: &str) -> Option<&str> {
//!         self.headers().get(name).and_then(|v| v.to_str().ok())
//!     }
//!     fn path(&self) -> &str {
//!         self.uri().path()
//!     }
//! }
//!
//! // In your handler
//! async fn index(req: MyRequest) -> MyResponse {
//!     let response = Inertia::render(&req, "Home", serde_json::json!({
//!         "title": "Welcome"
//!     }));
//!
//!     // Convert InertiaHttpResponse to your framework's response type
//!     response.into()
//! }
//! ```
//!
//! ## Framework Integrations
//!
//! See the examples directory for integrations with popular frameworks:
//! - Axum
//! - Actix-web
//! - Hyper

mod config;
mod manifest;
mod request;
mod response;
mod shared;

pub use config::InertiaConfig;
pub use request::InertiaRequest;
pub use response::{Inertia, InertiaHttpResponse, InertiaResponse};
pub use shared::InertiaShared;

// Re-export serde_json for convenience
pub use serde_json;
