//! Session management for Ferro framework
//!
//! Provides Laravel-like session handling with database storage.
//!
//! # Features
//!
//! - Secure session cookies (HttpOnly, Secure, SameSite)
//! - Database-backed storage for scalability
//! - CSRF token generation per session
//! - Flash messages for one-time notifications
//! - Session data stored as JSON
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::session::{session, session_mut};
//!
//! // Read from session
//! if let Some(s) = session() {
//!     let name: Option<String> = s.get("name");
//! }
//!
//! // Write to session
//! session_mut(|s| {
//!     s.put("name", "John");
//!     s.flash("success", "Item saved!");
//! });
//! ```
//!
//! # Setup
//!
//! Add the `SessionMiddleware` to your bootstrap:
//!
//! ```rust,ignore
//! use ferro_rs::{global_middleware, SessionMiddleware, SessionConfig};
//!
//! pub async fn register() {
//!     let config = SessionConfig::from_env();
//!     global_middleware!(SessionMiddleware::new(config));
//! }
//! ```

pub mod config;
pub mod driver;
pub mod middleware;
pub mod store;

pub use config::SessionConfig;
pub use driver::DatabaseSessionDriver;
pub use middleware::{
    auth_user_id, clear_auth_user, generate_csrf_token, generate_session_id, get_csrf_token,
    invalidate_all_for_user, invalidate_session, is_authenticated, regenerate_session_id, session,
    session_mut, set_auth_user, SessionMiddleware,
};
pub use store::{SessionData, SessionStore};

/// Run an async closure inside a fresh in-memory session scope.
///
/// Constructs a [`SessionData`] with the given `id` and an empty CSRF token,
/// wraps it in the task-local `SESSION_CONTEXT`, and runs `f` within that
/// scope.  [`session()`] and [`session_mut()`] work normally inside `f`.
///
/// Intended for unit tests in downstream crates that need session access
/// without a full HTTP request cycle.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::session::with_test_session;
/// use ferro::session::{session, session_mut};
///
/// #[tokio::test]
/// async fn my_test() {
///     with_test_session("test", || async {
///         session_mut(|s| { s.put("key", "value"); });
///         let v: Option<String> = session().and_then(|s| s.get("key"));
///         assert_eq!(v, Some("value".to_string()));
///     }).await;
/// }
/// ```
pub async fn with_test_session<F, Fut>(id: &str, f: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    use middleware::SESSION_CONTEXT;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let session = SessionData::new(id.to_string(), String::new());
    let ctx = Arc::new(RwLock::new(Some(session)));
    SESSION_CONTEXT.scope(ctx, f()).await;
}
