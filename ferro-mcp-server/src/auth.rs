//! Bearer-token extraction seam for the MCP endpoint.

/// Outcome of resolving a request's bearer credential.
///
/// Phase 199 fills the `Authenticated` variant with real PKCE/bearer
/// validation without changing this enum's shape or `extract_bearer`'s
/// signature.
pub enum BearerOutcome {
    /// No `Authorization` header, or token present but not recognized.
    Unauthenticated,
    /// Token validated; opaque principal attached. UNUSED in Phase 198.
    #[allow(dead_code)]
    Authenticated(serde_json::Value),
}

/// Resolve a request's `Authorization` header to a principal.
///
/// Phase 198: ALWAYS returns `Unauthenticated` — there is no valid-token
/// path yet. Phase 199 replaces this body without changing the signature.
pub fn extract_bearer(authorization_header: Option<&str>) -> BearerOutcome {
    let _ = authorization_header;
    BearerOutcome::Unauthenticated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_header_is_unauthenticated() {
        assert!(matches!(extract_bearer(None), BearerOutcome::Unauthenticated));
    }

    #[test]
    fn any_bearer_is_still_unauthenticated_in_phase_198() {
        assert!(matches!(
            extract_bearer(Some("Bearer eyJhbG...")),
            BearerOutcome::Unauthenticated
        ));
    }
}
