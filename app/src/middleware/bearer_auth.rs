//! Bearer token authentication middleware for the MCP endpoint.
//!
//! Validates the JWT bearer token using ferro-mcp-oauth before TenantMiddleware
//! resolves the tenant context. Inserts the validated principal as
//! `serde_json::Value` so JwtClaimResolver can read it downstream.
//!
//! Ordering requirement (Phase 200 D-01):
//!   BearerAuthMiddleware → TenantMiddleware(JwtClaimResolver) → handler
//!
//! `expected_tenant` is `None` here because TenantMiddleware has not run yet —
//! the tenant context is established by TenantMiddleware reading the inserted claims.

use ferro::serde_json;
use ferro::{async_trait, HttpResponse, Middleware, Next, Request, Response};
use ferro_mcp_oauth::{validate_bearer, BearerCheck, OAuthConfig};
use ferro_mcp_server::McpServerConfig;

/// Middleware that validates the MCP bearer token and inserts claims into the request.
///
/// Must be mounted BEFORE `TenantMiddleware` on the `/mcp` route group.
/// On success inserts `serde_json::Value` (the principal) into request extensions
/// so `JwtClaimResolver` can extract `tenant_id` from it.
pub struct BearerAuthMiddleware {
    pub mcp_config: McpServerConfig,
}

#[async_trait]
impl Middleware for BearerAuthMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let auth_header = request.header("Authorization").map(|s| s.to_owned());

        let oauth_config =
            OAuthConfig::from_env().map_err(|_| challenge_response(&self.mcp_config))?;

        // expected_tenant: None — TenantMiddleware runs next and owns tenant validation.
        // Pitfall 1: passing current_tenant() here would always be None (middleware ordering).
        match validate_bearer(auth_header.as_deref(), &oauth_config, None) {
            BearerCheck::Unauthenticated => Err(challenge_response(&self.mcp_config)),
            BearerCheck::Invalid => Err(HttpResponse::new()
                .status(401)
                .header("WWW-Authenticate", "Bearer error=\"invalid_token\"")),
            BearerCheck::Forbidden => Err(HttpResponse::new().status(403)),
            BearerCheck::Authenticated(principal) => {
                // Insert claims as serde_json::Value — TypeId must match JwtClaimResolver
                // (framework/src/tenant/resolver.rs line 210: req.get::<serde_json::Value>()).
                // Pitfall 2: any other type (McpTokenClaims, Map<...>) would silently return None.
                request.insert::<serde_json::Value>(principal);
                next(request).await
            }
        }
    }
}

/// Build the RFC 9728 / RFC 6750 unauthenticated challenge response.
pub fn challenge_response(config: &McpServerConfig) -> HttpResponse {
    let challenge = format!(
        "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        config.app_url
    );
    HttpResponse::new()
        .status(401)
        .header("WWW-Authenticate", challenge)
}
