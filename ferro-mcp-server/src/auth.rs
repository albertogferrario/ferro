//! Auth unifier for the MCP endpoint (Phase 217).
//!
//! `resolve_tenant` branches on token shape and delegates to:
//! - `ferro_mcp_oauth::validate_api_key` — `ferro_`-prefixed tokens (async DB lookup)
//! - `ferro_mcp_oauth::validate_bearer` — JWT tokens (sync decode)
//!
//! Both paths return `BearerCheck` from `ferro-mcp-oauth`.

use ferro_mcp_oauth::{validate_api_key, validate_bearer, BearerCheck, OAuthConfig};
use sea_orm::DatabaseConnection;

/// Resolve the calling tenant from the Authorization header.
///
/// Branches on token shape: `ferro_`-prefix → `validate_api_key` (async DB lookup),
/// otherwise → `validate_bearer` (sync JWT decode, wrapped in async fn for uniform call site).
pub async fn resolve_tenant(
    authorization_header: Option<&str>,
    db: &DatabaseConnection,
    oauth_config: &OAuthConfig,
) -> BearerCheck {
    let token = match authorization_header.and_then(|h| h.strip_prefix("Bearer ")) {
        None | Some("") => return BearerCheck::Unauthenticated,
        Some(t) => t,
    };
    if token.starts_with("ferro_") {
        validate_api_key(authorization_header, db, None).await
    } else {
        validate_bearer(authorization_header, oauth_config, None)
    }
}
