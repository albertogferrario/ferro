//! Application middleware
//!
//! Each middleware has its own dedicated file following the framework convention.

mod auth;
pub mod bearer_auth;
mod logging;
mod share_inertia;

pub use auth::AuthMiddleware;
pub use logging::LoggingMiddleware;
pub use share_inertia::ShareInertiaData;
