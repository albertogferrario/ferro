//! Bearer validation (Plan 03).
//!
//! `validate_bearer` verifies the JWT signature + expiry (→ 401), audience
//! match (→ 403), and tenant claim (→ 403). Plan 03 finalizes the real
//! signature using `ferro_mcp_server::BearerOutcome`.

use crate::config::OAuthConfig;

/// Stub bearer validation.
///
/// Returns `Some(claims)` if the bearer token is valid, `None` otherwise.
/// Plan 03 replaces this stub with the real HS256 + audience + tenant checks
/// and returns `BearerOutcome` from `ferro-mcp-server`.
pub fn validate_bearer(
    _authorization_header: Option<&str>,
    _config: &OAuthConfig,
) -> Option<serde_json::Value> {
    None
}
