//! Bearer-token outcome type for the MCP endpoint.

/// Outcome of resolving a request's bearer credential.
pub enum BearerOutcome {
    /// No `Authorization` header, or token present but not validated.
    Unauthenticated,
    /// Token validated; opaque principal attached.
    #[allow(dead_code)]
    Authenticated(serde_json::Value),
}
