//! Authentication middleware helpers.

pub use ferro::{AuthMiddleware, GuestMiddleware};

/// Redirect unauthenticated users to the login page.
pub fn auth() -> AuthMiddleware {
    AuthMiddleware::redirect_to("/login")
}

/// Redirect already-authenticated users away from guest-only pages (to the map).
pub fn guest() -> GuestMiddleware {
    GuestMiddleware::redirect_to("/map")
}
