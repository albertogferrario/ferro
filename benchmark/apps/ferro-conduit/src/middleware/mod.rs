//! Custom middleware (JWT auth). Task 3 fills jwt_auth / optional_jwt.

/// Authenticated user id, inserted into the request extension map by the JWT
/// middlewares. Handlers read it via `req.get::<UserId>()` (never `AuthUser<T>`,
/// which is session-bound — RESEARCH Pitfall 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(pub i64);
