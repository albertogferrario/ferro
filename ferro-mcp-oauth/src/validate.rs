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
use sea_orm::{ConnectionTrait, Statement};
use sha2::{Digest, Sha256};

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

/// Hash a raw MCP API key to SHA-256 hex for storage lookup.
pub fn hash_mcp_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a new MCP API key. Returns `(raw_key, key_hash)`.
///
/// `raw_key` starts with `ferro_` followed by 43 base62 characters (49 chars total).
/// `key_hash` is the SHA-256 hex of `raw_key`. Store only the hash; show raw_key once.
///
/// SKELETON — Plan 01 replaces the body with real CSPRNG generation.
pub fn generate_mcp_api_key() -> (String, String) {
    // Placeholder: returns a fixed non-prefixed value so the prefix/round-trip test fails (RED).
    (String::from("STUB"), String::from("STUB"))
}

/// Validate an MCP API key against the `mcp_api_keys` table.
///
/// Branches: header absent → `Unauthenticated`; hash not found or revoked → `Invalid`;
/// tenant mismatch (when `expected_tenant` is `Some`) → `Forbidden`; valid → `Authenticated`.
///
/// SKELETON — Plan 01 replaces the body with a real SHA-256 lookup.
pub async fn validate_api_key(
    authorization_header: Option<&str>,
    db: &sea_orm::DatabaseConnection,
    expected_tenant: Option<i64>,
) -> BearerCheck {
    // Step 1: header presence + Bearer prefix (mirrors validate_bearer)
    let header = match authorization_header {
        None => return BearerCheck::Unauthenticated,
        Some(h) => h,
    };
    let token = match header.strip_prefix("Bearer ") {
        None | Some("") => return BearerCheck::Unauthenticated,
        Some(t) => t,
    };
    // Step 2: ferro_ prefix guard (defensive — caller should have routed here already)
    if !token.starts_with("ferro_") {
        return BearerCheck::Unauthenticated;
    }

    // SKELETON: real SHA-256 lookup deferred to Plan 01.
    // Hash the key and query the DB so the structure compiles; always returns Invalid
    // until Plan 01 wires the real row-match and revocation check.
    let key_hash = hash_mcp_api_key(token);
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT id, tenant_id, scope, revoked_at FROM mcp_api_keys WHERE key_hash = ?",
        [sea_orm::Value::String(Some(Box::new(key_hash)))],
    );
    let _row = match db.query_one(stmt).await {
        Ok(r) => r,
        Err(_) => return BearerCheck::Invalid,
    };
    // Placeholder: row found path not yet implemented — RED for valid-key tests.
    let _ = expected_tenant;
    BearerCheck::Invalid
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

    // ── API key tests (Phase 217 RED suite) ──────────────────────────────────

    async fn setup_api_keys_db() -> sea_orm::DatabaseConnection {
        use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("sqlite connect");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE mcp_api_keys (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant_id  INTEGER NOT NULL,
                key_hash   TEXT NOT NULL UNIQUE,
                scope      TEXT NOT NULL DEFAULT 'read',
                revoked_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
            .to_string(),
        ))
        .await
        .expect("create mcp_api_keys");
        db
    }

    async fn seed_key(
        db: &sea_orm::DatabaseConnection,
        tenant_id: i64,
        scope: &str,
        revoked: bool,
    ) -> String {
        use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
        let (raw_key, key_hash) = generate_mcp_api_key();
        let revoked_at = if revoked {
            "'2020-01-01T00:00:00Z'"
        } else {
            "NULL"
        };
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            format!(
                "INSERT INTO mcp_api_keys (tenant_id, key_hash, scope, revoked_at) \
                 VALUES ({tenant_id}, '{key_hash}', '{scope}', {revoked_at})"
            ),
        ))
        .await
        .expect("seed key");
        raw_key
    }

    /// RED: stub returns ("STUB","STUB") — prefix and length checks will fail.
    #[tokio::test]
    async fn generate_mcp_api_key_is_prefixed_and_hash_matches() {
        let (raw_key, key_hash) = generate_mcp_api_key();
        assert!(
            raw_key.starts_with("ferro_"),
            "raw_key must start with ferro_, got {raw_key:?}"
        );
        assert_eq!(
            raw_key.len(),
            49,
            "raw_key must be 49 chars (ferro_ + 43 base62), got {}",
            raw_key.len()
        );
        assert_eq!(
            key_hash,
            hash_mcp_api_key(&raw_key),
            "key_hash must equal SHA-256 of raw_key"
        );
    }

    /// RED: skeleton returns Invalid — will pass once Plan 01 wires real lookup.
    #[tokio::test]
    async fn valid_api_key_returns_authenticated() {
        let db = setup_api_keys_db().await;
        let raw_key = seed_key(&db, 1, "read", false).await;
        let header = format!("Bearer {raw_key}");
        let result = validate_api_key(Some(&header), &db, None).await;
        match result {
            BearerCheck::Authenticated(principal) => {
                assert_eq!(principal["tenant_id"], 1, "tenant_id must be 1");
                assert_eq!(principal["scope"], "read", "scope must be read");
            }
            other => panic!("expected Authenticated, got {other:?}"),
        }
    }

    /// Trivially passes against stub (stub always returns Invalid for unknown keys too).
    #[tokio::test]
    async fn unknown_api_key_returns_invalid() {
        let db = setup_api_keys_db().await;
        let result = validate_api_key(Some("Bearer ferro_unknownkey123"), &db, None).await;
        assert!(
            matches!(result, BearerCheck::Invalid),
            "expected Invalid for unknown key, got {result:?}"
        );
    }

    /// Trivially passes against stub (stub returns Invalid for all keys including revoked).
    #[tokio::test]
    async fn revoked_api_key_returns_invalid() {
        let db = setup_api_keys_db().await;
        let raw_key = seed_key(&db, 1, "read", true).await;
        let header = format!("Bearer {raw_key}");
        let result = validate_api_key(Some(&header), &db, None).await;
        assert!(
            matches!(result, BearerCheck::Invalid),
            "expected Invalid for revoked key, got {result:?}"
        );
    }

    /// RED: skeleton ignores expected_tenant — will return Forbidden once Plan 01 wires real lookup.
    #[tokio::test]
    async fn wrong_expected_tenant_returns_forbidden() {
        let db = setup_api_keys_db().await;
        let raw_key = seed_key(&db, 1, "read", false).await;
        let header = format!("Bearer {raw_key}");
        let result = validate_api_key(Some(&header), &db, Some(2)).await;
        assert!(
            matches!(result, BearerCheck::Forbidden),
            "expected Forbidden for wrong expected_tenant, got {result:?}"
        );
    }
}
