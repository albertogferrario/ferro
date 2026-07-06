//! Share Inertia Data middleware
//!
//! Adds shared props (auth, flash, csrf) to every Inertia response.

use ferro::{async_trait, csrf_token, InertiaShared, Middleware, Next, Request, Response};

/// Middleware that shares common data with all Inertia responses
///
/// This middleware adds:
/// - `auth` - Current authenticated user (if any)
/// - `flash` - Flash messages from the session
/// - `csrf` - CSRF token for forms
///
/// # Example
///
/// Register in `bootstrap.rs`:
/// ```rust,ignore
/// global_middleware!(ShareInertiaData);
/// ```
pub struct ShareInertiaData;

#[async_trait]
impl Middleware for ShareInertiaData {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        // Build shared props
        let mut shared = InertiaShared::new();

        // Add CSRF token
        if let Some(token) = csrf_token() {
            shared = shared.csrf(token);
        }

        // TODO: Add authenticated user when Auth is properly integrated
        // if let Some(user) = Auth::user::<User>().await {
        //     shared = shared.auth(serde_json::json!({
        //         "id": user.id,
        //         "name": user.name,
        //         "email": user.email,
        //     }));
        // }

        // TODO: Add flash messages when session flash is implemented
        // if let Some(flash) = session::flash() {
        //     shared = shared.flash(flash);
        // }

        // Insert shared props into request extensions
        request.insert(shared);

        next(request).await
    }
}
