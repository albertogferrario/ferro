//! Broadcasting integration for channel authorization.
//!
//! Provides the [`broadcasting_auth`] handler that bridges session-based
//! authentication with channel authorization for private and presence channels.

pub mod auth;

pub use auth::broadcasting_auth;
