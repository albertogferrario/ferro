//! Session middleware for Ferro framework

use crate::http::cookie::{Cookie, SameSite};
use crate::http::Response;
use crate::middleware::{Middleware, Next};
use crate::Request;
use async_trait::async_trait;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::SessionConfig;
use super::driver::DatabaseSessionDriver;
use super::store::{SessionData, SessionStore};

// Task-local session context using tokio's task_local macro
// This is async-safe unlike thread_local which can lose data across await points
tokio::task_local! {
    pub(crate) static SESSION_CONTEXT: Arc<RwLock<Option<SessionData>>>;
    // Set true the first time `session()` / `session_mut()` is called within a
    // request. Lets the middleware persist + set the session cookie ONLY when the
    // handler actually used the session — anonymous requests that never touch it
    // (e.g. static asset fetches) get no `Set-Cookie`, so a CDN/Cloudflare can
    // cache them, and no needless session-store write happens per asset.
    pub(crate) static SESSION_ACCESSED: Arc<AtomicBool>;
}

/// Flag the current session as accessed (read or written) so the middleware
/// persists it and emits the cookie. No-op outside a request scope.
fn mark_session_accessed() {
    let _ = SESSION_ACCESSED.try_with(|flag| flag.store(true, Ordering::Relaxed));
}

/// Get the current session (read-only)
///
/// Returns a clone of the current session data if available.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::session::session;
///
/// if let Some(session) = session() {
///     let name: Option<String> = session.get("name");
/// }
/// ```
pub fn session() -> Option<SessionData> {
    mark_session_accessed();
    SESSION_CONTEXT
        .try_with(|ctx| {
            // Use try_read to avoid blocking - if locked, return None
            ctx.try_read().ok().and_then(|guard| guard.clone())
        })
        .ok()
        .flatten()
}

/// Get the current session and modify it
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::session::session_mut;
///
/// session_mut(|session| {
///     session.put("name", "John");
/// });
/// ```
pub fn session_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut SessionData) -> R,
{
    mark_session_accessed();
    SESSION_CONTEXT
        .try_with(|ctx| {
            // Use try_write to avoid blocking
            ctx.try_write()
                .ok()
                .and_then(|mut guard| guard.as_mut().map(f))
        })
        .ok()
        .flatten()
}

/// Take the session out of the context (for saving)
fn take_session_internal(ctx: &Arc<RwLock<Option<SessionData>>>) -> Option<SessionData> {
    ctx.try_write().ok().and_then(|mut guard| guard.take())
}

/// Generate a cryptographically secure session ID
///
/// Generates a 40-character alphanumeric string.
pub fn generate_session_id() -> String {
    let mut rng = rand::thread_rng();
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

    (0..40)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a CSRF token
///
/// Same format as session ID for consistency.
pub fn generate_csrf_token() -> String {
    generate_session_id()
}

/// Session middleware
///
/// Handles session lifecycle:
/// 1. Reads session ID from cookie
/// 2. Loads session data from storage
/// 3. Makes session available during request
/// 4. Saves session after request
/// 5. Sets session cookie on response
pub struct SessionMiddleware {
    config: SessionConfig,
    store: Arc<dyn SessionStore>,
}

impl SessionMiddleware {
    /// Create a new session middleware with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        let store = Arc::new(DatabaseSessionDriver::new(
            config.idle_lifetime,
            config.absolute_lifetime,
        ));
        Self { config, store }
    }

    /// Create session middleware with a custom store
    pub fn with_store(config: SessionConfig, store: Arc<dyn SessionStore>) -> Self {
        Self { config, store }
    }

    fn create_session_cookie(&self, session_id: &str) -> Cookie {
        let mut cookie = Cookie::new(&self.config.cookie_name, session_id)
            .http_only(self.config.cookie_http_only)
            .secure(self.config.cookie_secure)
            .path(&self.config.cookie_path)
            .max_age(std::cmp::max(
                self.config.idle_lifetime,
                self.config.absolute_lifetime,
            ));

        cookie = match self.config.cookie_same_site.to_lowercase().as_str() {
            "strict" => cookie.same_site(SameSite::Strict),
            "none" => cookie.same_site(SameSite::None),
            _ => cookie.same_site(SameSite::Lax),
        };

        cookie
    }
}

#[async_trait]
impl Middleware for SessionMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Get session ID from cookie or generate new one
        let session_id = request
            .cookie(&self.config.cookie_name)
            .unwrap_or_else(generate_session_id);

        // Load session from store
        let mut session = match self.store.read(&session_id).await {
            Ok(Some(s)) => s,
            Ok(None) => {
                // Create new session
                SessionData::new(session_id.clone(), generate_csrf_token())
            }
            Err(e) => {
                eprintln!("Session read error: {e}");
                SessionData::new(session_id.clone(), generate_csrf_token())
            }
        };

        // Age flash data from previous request
        session.age_flash_data();

        // Create task-local context and store session in it
        let ctx = Arc::new(RwLock::new(Some(session)));

        // Process the request within the task-local scopes.
        // SESSION_CONTEXT makes session()/session_mut() work across await points;
        // SESSION_ACCESSED records whether the handler actually touched the session.
        let accessed_flag = Arc::new(AtomicBool::new(false));
        let response = SESSION_ACCESSED
            .scope(
                accessed_flag.clone(),
                SESSION_CONTEXT.scope(ctx.clone(), async move { next(request).await }),
            )
            .await;

        // Get the potentially modified session from the context
        let session = take_session_internal(&ctx);

        // Lazy persistence: only write the store and emit the Set-Cookie when the
        // session was actually used — accessed via session()/session_mut(), or left
        // dirty by a mutation. A request that never touches the session (e.g. a
        // static asset fetch) returns NO Set-Cookie, so Cloudflare / a CDN can edge-
        // cache it, and no per-request session-store write is incurred. Auth flows
        // (login/logout/CSRF/is_authenticated) all go through session()/session_mut(),
        // so they continue to persist and refresh the cookie exactly as before.
        let used = accessed_flag.load(Ordering::Relaxed)
            || session.as_ref().map(SessionData::is_dirty).unwrap_or(false);

        match session {
            Some(session) if used => {
                if let Err(e) = self.store.write(&session).await {
                    eprintln!("Session write error: {e}");
                }
                let cookie = self.create_session_cookie(&session.id);
                match response {
                    Ok(res) => Ok(res.cookie(cookie)),
                    Err(res) => Err(res.cookie(cookie)),
                }
            }
            _ => response,
        }
    }
}

/// Regenerate the session ID (for security after login)
///
/// This creates a new session ID while preserving session data,
/// which helps prevent session fixation attacks.
pub fn regenerate_session_id() {
    session_mut(|session| {
        session.id = generate_session_id();
        session.dirty = true;
    });
}

/// Invalidate the current session (clear all data)
pub fn invalidate_session() {
    session_mut(|session| {
        session.flush();
        session.csrf_token = generate_csrf_token();
    });
}

/// Helper to get the CSRF token from current session
pub fn get_csrf_token() -> Option<String> {
    session().map(|s| s.csrf_token)
}

/// Helper to check if user is authenticated
pub fn is_authenticated() -> bool {
    session().map(|s| s.user_id.is_some()).unwrap_or(false)
}

/// Helper to get the authenticated user ID
pub fn auth_user_id() -> Option<i64> {
    session().and_then(|s| s.user_id)
}

/// Helper to set the authenticated user
pub fn set_auth_user(user_id: i64) {
    session_mut(|session| {
        session.user_id = Some(user_id);
        session.dirty = true;
    });
}

/// Helper to clear the authenticated user (logout)
pub fn clear_auth_user() {
    session_mut(|session| {
        session.user_id = None;
        session.dirty = true;
    });
}

/// Destroy all sessions for a user, with optional exception for the current session.
///
/// Uses the session store's `destroy_for_user` method for direct DB deletion.
/// This is auth-method-agnostic — works for password-based, OAuth, or any auth.
///
/// # Arguments
/// * `store` - The session store to use
/// * `user_id` - The user whose sessions to destroy
/// * `except_session_id` - Optional session ID to preserve (current session)
pub async fn invalidate_all_for_user(
    store: &dyn super::store::SessionStore,
    user_id: i64,
    except_session_id: Option<&str>,
) -> Result<u64, crate::error::FrameworkError> {
    store.destroy_for_user(user_id, except_session_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> Arc<RwLock<Option<SessionData>>> {
        Arc::new(RwLock::new(Some(SessionData::new(
            "sid".into(),
            "csrf".into(),
        ))))
    }

    // The middleware persists + sets the cookie iff this flag (or `dirty`) is set.
    // These prove the flag is set ONLY when the handler touches the session — so a
    // request that never calls session()/session_mut() (a static asset) yields no
    // Set-Cookie and stays CDN-cacheable.
    #[tokio::test]
    async fn untouched_session_is_not_marked_accessed() {
        let flag = Arc::new(AtomicBool::new(false));
        SESSION_ACCESSED
            .scope(
                flag.clone(),
                SESSION_CONTEXT.scope(scope(), async {
                    // simulate a static-asset request: never touches the session
                }),
            )
            .await;
        assert!(
            !flag.load(Ordering::Relaxed),
            "a request that never touches the session must not be marked accessed"
        );
    }

    #[tokio::test]
    async fn session_read_marks_accessed() {
        let flag = Arc::new(AtomicBool::new(false));
        SESSION_ACCESSED
            .scope(
                flag.clone(),
                SESSION_CONTEXT.scope(scope(), async {
                    let _ = session();
                }),
            )
            .await;
        assert!(
            flag.load(Ordering::Relaxed),
            "session() must mark the session accessed"
        );
    }

    #[tokio::test]
    async fn session_mut_marks_accessed_and_dirties() {
        let ctx = scope();
        let flag = Arc::new(AtomicBool::new(false));
        SESSION_ACCESSED
            .scope(
                flag.clone(),
                SESSION_CONTEXT.scope(ctx.clone(), async {
                    session_mut(|s| s.put("k", "v"));
                }),
            )
            .await;
        assert!(
            flag.load(Ordering::Relaxed),
            "session_mut must mark accessed"
        );
        let data = take_session_internal(&ctx).expect("session present");
        assert!(data.is_dirty(), "put() must dirty the session");
    }

    #[test]
    fn mark_session_accessed_outside_scope_is_noop() {
        // Must not panic when called with no active request scope.
        mark_session_accessed();
    }
}
