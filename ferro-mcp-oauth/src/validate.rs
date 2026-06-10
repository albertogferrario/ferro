//! Bearer validation (Plan 03).
//!
//! `validate_bearer` verifies the JWT and returns a `BearerCheck` enum that
//! Plan 05 maps to `ferro_mcp_server::BearerOutcome` + HTTP status at the app seam.
//!
//! Validation order (D-07):
//! 1. Header absent / no `Bearer ` prefix → `Unauthenticated` (Plan 05 → 401 challenge).
//! 2. JWT signature + exp → `Invalid` on failure (401 `invalid_token`).
//! 3. Audience mismatch → `Forbidden` (403 `insufficient_scope`).
//! 4. Tenant mismatch → `Forbidden` (403).
//! 5. All checks pass → `Authenticated(principal)`.
//!
//! `ErrorKind` → HTTP status mapping:
//! | `ErrorKind`              | Status | `WWW-Authenticate`                            |
//! |--------------------------|--------|-----------------------------------------------|
//! | `ExpiredSignature`       | 401    | `Bearer error="invalid_token"` (token expired) |
//! | `InvalidSignature`       | 401    | `Bearer error="invalid_token"`                |
//! | `InvalidAudience`        | 403    | `Bearer error="insufficient_scope"` (RFC 6750 §3.1) |
//! | `InvalidToken`, `Base64`, `Json`, … | 401 | `Bearer error="invalid_token"` |
//! | tenant mismatch (post-decode) | 403 | — |
//!
//! `InvalidAudience` maps to 403, not 401, because the token is validly signed —
//! the bearer is authenticated but not scoped for this resource (RFC 6750 §3.1).

use crate::config::OAuthConfig;
use crate::jwt::decode_token;

/// Outcome of `validate_bearer`.
///
/// Plan 05 maps this to `ferro_mcp_server::BearerOutcome`:
/// - `Unauthenticated` | `Invalid` → 401 `WWW-Authenticate: Bearer error="invalid_token"`
/// - `Forbidden` → 403
/// - `Authenticated(principal)` → proceed with dispatch
#[derive(Debug)]
pub enum BearerCheck {
    /// No `Authorization` header or no `Bearer ` prefix. Plan 05 → 401 challenge.
    Unauthenticated,
    /// Bad signature, expired, or malformed token. Plan 05 → 401 `invalid_token`.
    Invalid,
    /// Valid signature but audience or tenant mismatch. Plan 05 → 403.
    Forbidden,
    /// Token validated. Principal is `json!({"sub": ..., "tenant_id": ...})`.
    Authenticated(serde_json::Value),
}

/// Validates a bearer token against the OAuth config and an optional expected tenant.
///
/// `expected_tenant`:
/// - `None` — single-tenant or tenant-agnostic call site; tenant check skipped.
/// - `Some(t)` — multi-tenant: `claims.tenant_id` must equal `t` (both absent-when-expected → `Forbidden`).
///
/// This function is synchronous (JWT decode is synchronous in jsonwebtoken v9).
pub fn validate_bearer(
    authorization_header: Option<&str>,
    config: &OAuthConfig,
    expected_tenant: Option<i64>,
) -> BearerCheck {
    // Step 1: header presence + Bearer prefix
    let header = match authorization_header {
        None => return BearerCheck::Unauthenticated,
        Some(h) => h,
    };
    let token = match header.strip_prefix("Bearer ") {
        None | Some("") => return BearerCheck::Unauthenticated,
        Some(t) => t,
    };

    // Step 2 + 3: decode (validates signature, exp, and audience simultaneously)
    let expected_aud = format!("{}/mcp", config.app_url);
    let claims = match decode_token(token, &config.token_secret, &expected_aud) {
        Ok(c) => c,
        Err(e) => {
            return match e.kind() {
                // T-08: audience mismatch → 403 (token is authentic but not for this resource)
                jsonwebtoken::errors::ErrorKind::InvalidAudience => BearerCheck::Forbidden,
                // All other decode failures → 401 invalid_token
                _ => BearerCheck::Invalid,
            };
        }
    };

    // Step 4: tenant check (T-09)
    if let Some(expected) = expected_tenant {
        match claims.tenant_id {
            Some(claimed) if claimed == expected => {
                // tenant matches — fall through to Authenticated
            }
            // tenant_id present but wrong, or absent when expected → 403
            _ => return BearerCheck::Forbidden,
        }
    }

    // Step 5: authenticated
    BearerCheck::Authenticated(serde_json::json!({
        "sub": claims.sub,
        "tenant_id": claims.tenant_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::{build_claims, mint_token, McpTokenClaims};

    const SECRET: &[u8] = b"validate-test-secret-that-is-at-least-32-bytes!!";
    const APP_URL: &str = "https://app.example.com";

    fn config() -> OAuthConfig {
        OAuthConfig {
            app_name: "Test App".to_string(),
            app_url: APP_URL.to_string(),
            token_secret: SECRET.to_vec(),
        }
    }

    fn valid_token() -> String {
        let claims = build_claims(42, Some(7), APP_URL, 3600);
        mint_token(&claims, SECRET).expect("mint failed")
    }

    fn bearer(token: &str) -> String {
        format!("Bearer {token}")
    }

    /// Valid token → Authenticated principal with sub and tenant_id.
    #[test]
    fn valid_token_returns_authenticated() {
        let token = valid_token();
        let result = validate_bearer(Some(&bearer(&token)), &config(), None);
        match result {
            BearerCheck::Authenticated(principal) => {
                assert_eq!(principal["sub"], "42");
                assert_eq!(principal["tenant_id"], 7);
            }
            other => panic!("expected Authenticated, got {other:?}"),
        }
    }

    /// Expired token → Invalid (401-class).
    #[test]
    fn expired_token_returns_invalid() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expired = McpTokenClaims {
            sub: "42".to_string(),
            tenant_id: Some(7),
            aud: vec![format!("{APP_URL}/mcp")],
            iss: APP_URL.to_string(),
            iat: now - 7200,
            exp: now - 3600,
        };
        let token = mint_token(&expired, SECRET).expect("mint failed");
        let result = validate_bearer(Some(&bearer(&token)), &config(), None);
        assert!(
            matches!(result, BearerCheck::Invalid),
            "expected Invalid for expired token, got {result:?}"
        );
    }

    /// Wrong audience → Forbidden (403-class).
    #[test]
    fn wrong_audience_returns_forbidden() {
        // Mint a token for a different audience
        let other_claims = build_claims(42, Some(7), "https://other.example.com", 3600);
        let token = mint_token(&other_claims, SECRET).expect("mint failed");
        // Validate against this app's config (different audience)
        let result = validate_bearer(Some(&bearer(&token)), &config(), None);
        assert!(
            matches!(result, BearerCheck::Forbidden),
            "expected Forbidden for wrong audience, got {result:?}"
        );
    }

    /// Token with mismatched tenant_id → Forbidden (403-class).
    #[test]
    fn wrong_tenant_returns_forbidden() {
        let token = valid_token(); // tenant_id = Some(7)
                                   // Expected tenant is 99 — mismatch
        let result = validate_bearer(Some(&bearer(&token)), &config(), Some(99));
        assert!(
            matches!(result, BearerCheck::Forbidden),
            "expected Forbidden for wrong tenant, got {result:?}"
        );
    }

    /// No Authorization header → Unauthenticated.
    #[test]
    fn no_header_returns_unauthenticated() {
        let result = validate_bearer(None, &config(), None);
        assert!(
            matches!(result, BearerCheck::Unauthenticated),
            "expected Unauthenticated for no header, got {result:?}"
        );
    }

    /// `Authorization: something-without-bearer-prefix` → Unauthenticated.
    #[test]
    fn no_bearer_prefix_returns_unauthenticated() {
        let result = validate_bearer(Some("Token abc123"), &config(), None);
        assert!(
            matches!(result, BearerCheck::Unauthenticated),
            "expected Unauthenticated for non-Bearer prefix, got {result:?}"
        );
    }

    /// Token with tenant_id=None but expected_tenant=Some → Forbidden.
    #[test]
    fn absent_tenant_when_expected_returns_forbidden() {
        let claims = build_claims(42, None, APP_URL, 3600); // no tenant_id
        let token = mint_token(&claims, SECRET).expect("mint failed");
        let result = validate_bearer(Some(&bearer(&token)), &config(), Some(7));
        assert!(
            matches!(result, BearerCheck::Forbidden),
            "expected Forbidden when tenant absent but expected, got {result:?}"
        );
    }
}
