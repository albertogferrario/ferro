//! Application middleware.

pub mod authenticate;
mod logging;
mod share_inertia;

pub use logging::LoggingMiddleware;
pub use share_inertia::ShareInertiaData;
