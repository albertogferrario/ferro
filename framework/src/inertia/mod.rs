//! Inertia.js integration for Ferro framework.
//!
//! This module provides the integration between the framework-agnostic
//! `ferro-inertia` crate and the Ferro framework's HTTP types.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{Inertia, Request, Response, InertiaProps};
//!
//! #[derive(InertiaProps)]
//! pub struct HomeProps {
//!     pub title: String,
//! }
//!
//! pub async fn index(req: Request) -> Response {
//!     Inertia::render(&req, "Home", HomeProps {
//!         title: "Welcome".into(),
//!     })
//! }
//! ```

mod config;
mod context;
pub(crate) mod global;
mod response;

pub use config::InertiaConfig;
pub use context::{Inertia, InertiaShared, SavedInertiaContext};
pub use global::{get_inertia_config, set_inertia_config};
pub use response::InertiaResponse;

// Re-export core types from ferro-inertia for advanced usage
pub use ferro_inertia::{InertiaHttpResponse, InertiaRequest as InertiaRequestTrait};

// Deprecated exports for backward compatibility
#[allow(deprecated)]
pub use context::{InertiaContext, InertiaContextData};
