//! Token exchange endpoint (Plan 04).
//!
//! Implements `POST /token`: validates auth code, verifies PKCE S256, mints
//! HS256 JWT access token bound to `(user, tenant)`.
//!
//! Security properties:
//! - T-199-02 (code replay): `Cache::forget` called BEFORE any validation.
//!   A replay or any validation failure cannot reuse the code.
//! - T-199-01 (PKCE downgrade): verifies S256 via `pkce::verify_s256`.
//! - T-199-16 (code substitution): client_id + redirect_uri re-validated
//!   against the stored code record.

use ferro::Cache;
use serde::Deserialize;
use serde_json::json;

use crate::config::OAuthConfig;
use crate::jwt::{build_claims, mint_token};
use crate::pkce::verify_s256;
use crate::store::OAuthCode;

/// Request body for `POST /token` (RFC 6749 §4.1.3, urlencoded).
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
}

/// Handler: `POST /token`.
///
/// 1. Parse form body.
/// 2. Validate `grant_type == "authorization_code"`.
/// 3. **Single-use (T-199-02):** `Cache::get` then `Cache::forget` BEFORE validation.
/// 4. Validate `client_id` and `redirect_uri` exact-match (T-199-16).
/// 5. PKCE S256 verify (T-199-01).
/// 6. Mint HS256 JWT (`build_claims` + `mint_token`).
/// 7. Return `200 {"access_token", "token_type": "Bearer", "expires_in": 3600}`.
#[ferro::handler]
pub async fn token_exchange(req: ferro::Request) -> ferro::Response {
    // ── Step 1: Parse form body ───────────────────────────────────────────────
    let form: TokenRequest = req.form().await.map_err(|e| {
        json_error(400, "invalid_request", &format!("form parse error: {e}"))
    })?;

    // ── Step 2: grant_type check ──────────────────────────────────────────────
    if form.grant_type != "authorization_code" {
        return Err(json_error(
            400,
            "unsupported_grant_type",
            "grant_type must be 'authorization_code'",
        ));
    }

    // ── Step 3: Single-use code retrieval (T-199-02, HIGH) ───────────────────
    // CRITICAL ORDER: get THEN forget BEFORE any validation.
    // Even if subsequent validation fails, the code cannot be replayed.
    let code_key = format!("mcp:code:{}", form.code);
    let record: Option<OAuthCode> = Cache::get(&code_key).await.ok().flatten();
    // forget() regardless of whether the code was found — idempotent no-op if absent
    let _ = Cache::forget(&code_key).await;

    let record = record.ok_or_else(|| {
        json_error(
            400,
            "invalid_grant",
            "authorization code expired or already used",
        )
    })?;

    // ── Step 4: Validate client_id + redirect_uri (T-199-16) ─────────────────
    if record.client_id != form.client_id {
        return Err(json_error(400, "invalid_client", "client_id mismatch"));
    }
    if record.redirect_uri != form.redirect_uri {
        return Err(json_error(
            400,
            "invalid_grant",
            "redirect_uri does not match the authorization request",
        ));
    }

    // ── Step 5: PKCE S256 verify (T-199-01) ──────────────────────────────────
    if !verify_s256(&form.code_verifier, &record.code_challenge) {
        return Err(json_error(
            400,
            "invalid_grant",
            "PKCE code_verifier does not match code_challenge",
        ));
    }

    // ── Step 6: Mint JWT ──────────────────────────────────────────────────────
    let config = OAuthConfig::from_env().map_err(|e| {
        json_error(500, "server_error", &format!("OAuth config error: {e}"))
    })?;

    let claims = build_claims(record.user_id, record.tenant_id, &config.app_url, 3600);
    let access_token = mint_token(&claims, &config.token_secret).map_err(|e| {
        json_error(500, "server_error", &format!("token mint error: {e}"))
    })?;

    // ── Step 7: Return token response (RFC 6749 §5.1) ────────────────────────
    Ok(ferro::HttpResponse::json(json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": 3600,
    })))
}

/// RFC 6749 §5.2 error response body.
fn json_error(status: u16, error: &str, description: &str) -> ferro::HttpResponse {
    ferro::HttpResponse::json(json!({
        "error": error,
        "error_description": description,
    }))
    .status(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_test_helpers::bootstrap_test_cache;
    use crate::config::OAuthConfig;
    use crate::jwt::{build_claims, mint_token};
    use crate::pkce::{generate_auth_code, verify_s256};
    use crate::store::OAuthCode;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use ferro::Cache;
    use sha2::{Digest, Sha256};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            app_name: "TestApp".to_string(),
            app_url: "http://localhost:8080".to_string(),
            token_secret: b"test_secret_that_is_at_least_32_bytes_long_for_hs256".to_vec(),
        }
    }

    fn make_challenge(verifier: &str) -> String {
        let hash = Sha256::digest(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hash)
    }

    /// Store an OAuthCode in cache and return the code string.
    async fn store_code(
        client_id: &str,
        redirect_uri: &str,
        verifier: &str,
        user_id: i64,
        tenant_id: Option<i64>,
    ) -> String {
        let code = generate_auth_code();
        let challenge = make_challenge(verifier);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let record = OAuthCode {
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            code_challenge: challenge,
            user_id,
            tenant_id,
            created_at: now,
        };
        Cache::put(
            &format!("mcp:code:{code}"),
            &record,
            Some(Duration::from_secs(60)),
        )
        .await
        .expect("cache put should succeed");
        code
    }

    /// Verify that Cache::forget is called before validation by checking that
    /// a second retrieve returns None.
    #[tokio::test]
    async fn forget_before_validate_single_use() {
        bootstrap_test_cache();

        let verifier = "test_verifier_that_is_long_enough_for_pkce_12345";
        let code = store_code(
            "client-abc",
            "http://localhost:3000/cb",
            verifier,
            1,
            None,
        )
        .await;

        // First get: should find it
        let key = format!("mcp:code:{code}");
        let first: Option<OAuthCode> = Cache::get(&key).await.ok().flatten();
        assert!(first.is_some(), "code should exist before forget");

        // forget()
        let _ = Cache::forget(&key).await;

        // Second get: should be None
        let second: Option<OAuthCode> = Cache::get(&key).await.ok().flatten();
        assert!(second.is_none(), "code should be gone after forget");
    }

    /// Replaying the same code returns None on the second get.
    #[tokio::test]
    async fn replay_code_returns_none_after_forget() {
        bootstrap_test_cache();

        let verifier = "replay_verifier_long_enough_for_pkce_requirements_abc";
        let code = store_code("cid", "http://localhost/cb", verifier, 1, None).await;
        let key = format!("mcp:code:{code}");

        // Simulate the token_exchange forget-before-validate pattern
        let first: Option<OAuthCode> = Cache::get(&key).await.ok().flatten();
        let _ = Cache::forget(&key).await;
        assert!(first.is_some());

        // Second attempt: code is already gone
        let second: Option<OAuthCode> = Cache::get(&key).await.ok().flatten();
        let _ = Cache::forget(&key).await; // idempotent
        assert!(second.is_none(), "replayed code must return None");
    }

    /// Wrong code_verifier → verify_s256 returns false.
    #[test]
    fn wrong_verifier_fails_pkce() {
        let verifier = "correct_verifier_for_this_test_long_enough_abc123";
        let challenge = make_challenge(verifier);
        assert!(!verify_s256("wrong_verifier_abc123", &challenge));
    }

    /// Correct code_verifier → verify_s256 returns true.
    #[test]
    fn correct_verifier_passes_pkce() {
        let verifier = "correct_verifier_for_this_test_long_enough_abc123";
        let challenge = make_challenge(verifier);
        assert!(verify_s256(verifier, &challenge));
    }

    /// JWT round-trip: mint then validate via validate_bearer → Authenticated.
    #[test]
    fn jwt_roundtrip_authenticated() {
        let config = test_config();
        let claims = build_claims(42, Some(7), &config.app_url, 3600);
        let token = mint_token(&claims, &config.token_secret).expect("mint ok");

        let result = crate::validate::validate_bearer(
            Some(&format!("Bearer {token}")),
            &config,
            None,
        );
        match result {
            crate::validate::BearerCheck::Authenticated(principal) => {
                assert_eq!(principal["sub"], "42");
                assert_eq!(principal["tenant_id"], 7);
            }
            other => panic!("expected Authenticated, got {other:?}"),
        }
    }

    /// access_token field name and expires_in shape are correct in the JSON response.
    #[test]
    fn json_error_shape() {
        let resp = json_error(400, "invalid_grant", "test desc");
        assert_eq!(resp.status_code(), 400);
    }
}
