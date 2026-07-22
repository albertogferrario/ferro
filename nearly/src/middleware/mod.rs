//! Application middleware.

pub mod authenticate;
mod logging;

pub use logging::LoggingMiddleware;
