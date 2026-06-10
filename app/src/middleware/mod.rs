//! Application middleware
//!
//! Each middleware has its own dedicated file following the framework convention.

pub mod bearer_auth;
mod auth;
mod logging;
mod share_inertia;

pub use auth::AuthMiddleware;
pub use bearer_auth::BearerAuthMiddleware;
pub use logging::LoggingMiddleware;
pub use share_inertia::ShareInertiaData;
