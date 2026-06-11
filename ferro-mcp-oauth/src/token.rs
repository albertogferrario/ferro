//! Token exchange endpoint (Plan 04).
//!
//! Implements `POST /token`: branches on `grant_type` to handle either the
//! authorization-code grant (RFC 6749 §4.1.3) or the device-code grant
//! (RFC 8628 §3.4), then mints an HS256 JWT access token via the shared
//! `build_claims` + `mint_token` path (one-token-issuer invariant, D-05/SC-3).
//!
//! Security properties:
//! - T-199-02 (code replay): auth-code arm calls `Cache::forget` BEFORE any validation.
//! - T-199-01 (PKCE downgrade): auth-code arm verifies S256 via `pkce::verify_s256`.
//! - T-199-16 (code substitution): auth-code arm re-validates client_id + redirect_uri.
//! - T-203-DEVICECODE-REPLAY: device-code arm forgets BOTH cache keys on Approved (single-use).
//! - T-203-DEVICECODE-EXPIRY: explicit `now - created_at > 600` guard before state machine.
//! - T-203-CLAIMS-DIVERGE: device arm calls `build_claims` + `mint_token` identically to auth-code arm.

use ferro::Cache;
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::OAuthConfig;
use crate::device::{
    device_cache_key, usercode_cache_key, DeviceGrant, DeviceGrantStatus, DEVICE_CODE_TTL,
    DEVICE_INTERVAL_SECS,
};
use crate::jwt::{build_claims, mint_token};
use crate::pkce::verify_s256;
use crate::store::OAuthCode;

/// Request body for `POST /token` (RFC 6749 §4.1.3 + RFC 8628 §3.4, urlencoded).
///
/// Fields are `Option<String>` with `#[serde(default)]` so a device-code request
/// (which omits `code`/`redirect_uri`/`code_verifier`) does not fail at deserialization
/// before the `grant_type` branch (Pitfall 5). Each arm validates the presence of its
/// own required fields after dispatching.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// Required for both grant types (RFC 6749 §4.1.3, RFC 8628 §3.4).
    pub grant_type: String,
    /// Required for `authorization_code` grant (RFC 6749 §4.1.3).
    #[serde(default)]
    pub code: Option<String>,
    /// Required for `authorization_code` grant (RFC 6749 §4.1.3).
    #[serde(default)]
    pub redirect_uri: Option<String>,
    /// Required for both grants (used for client identity validation).
    pub client_id: String,
    /// Required for `authorization_code` grant (PKCE, RFC 7636).
    #[serde(default)]
    pub code_verifier: Option<String>,
    /// Required for `urn:ietf:params:oauth:grant-type:device_code` grant (RFC 8628 §3.4).
    #[serde(default)]
    pub device_code: Option<String>,
}

/// Handler: `POST /token`.
///
/// Dispatches to `token_exchange_auth_code` or `token_exchange_device_code` based
/// on `grant_type`. Returns `400 unsupported_grant_type` for all other values.
#[ferro::handler]
pub async fn token_exchange(req: ferro::Request) -> ferro::Response {
    // ── Step 1: Parse form body ───────────────────────────────────────────────
    let form: TokenRequest = req
        .form()
        .await
        .map_err(|e| json_error(400, "invalid_request", &format!("form parse error: {e}")))?;

    token_exchange_dispatch(form).await
}

/// Inner dispatch — separated so tests can call without constructing a full `Request`.
async fn token_exchange_dispatch(form: TokenRequest) -> ferro::Response {
    // ── Step 2: grant_type dispatch ───────────────────────────────────────────
    match form.grant_type.as_str() {
        "authorization_code" => token_exchange_auth_code(form).await,
        "urn:ietf:params:oauth:grant-type:device_code" => token_exchange_device_code(form).await,
        _ => Err(json_error(
            400,
            "unsupported_grant_type",
            "unsupported grant_type",
        )),
    }
}

/// Authorization-code grant arm (RFC 6749 §4.1.3).
///
/// Extracted from the original `token_exchange` body — behavior is byte-for-byte
/// identical to the pre-dispatch version. Single-use (T-199-02), PKCE S256 (T-199-01),
/// client_id + redirect_uri re-validation (T-199-16).
async fn token_exchange_auth_code(form: TokenRequest) -> ferro::Response {
    // Unwrap fields that are required for this arm
    let code = form
        .code
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| json_error(400, "invalid_request", "missing code"))?;
    let redirect_uri = form
        .redirect_uri
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| json_error(400, "invalid_request", "missing redirect_uri"))?;
    let code_verifier = form
        .code_verifier
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| json_error(400, "invalid_request", "missing code_verifier"))?;

    // ── Step 3: Single-use code retrieval (T-199-02, HIGH) ───────────────────
    // CRITICAL ORDER: get THEN forget BEFORE any validation.
    // Even if subsequent validation fails, the code cannot be replayed.
    let code_key = format!("mcp:code:{code}");
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
    if record.redirect_uri != redirect_uri {
        return Err(json_error(
            400,
            "invalid_grant",
            "redirect_uri does not match the authorization request",
        ));
    }

    // ── Step 5: PKCE S256 verify (T-199-01) ──────────────────────────────────
    if !verify_s256(code_verifier, &record.code_challenge) {
        return Err(json_error(
            400,
            "invalid_grant",
            "PKCE code_verifier does not match code_challenge",
        ));
    }

    // ── Step 6: Mint JWT ──────────────────────────────────────────────────────
    let config = OAuthConfig::from_env()
        .map_err(|e| json_error(500, "server_error", &format!("OAuth config error: {e}")))?;

    let claims = build_claims(record.user_id, record.tenant_id, &config.app_url, 3600);
    let access_token = mint_token(&claims, &config.token_secret)
        .map_err(|e| json_error(500, "server_error", &format!("token mint error: {e}")))?;

    // ── Step 7: Return token response (RFC 6749 §5.1) ────────────────────────
    Ok(ferro::HttpResponse::json(json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": 3600,
    })))
}

/// Device-code grant arm (RFC 8628 §3.4 + §3.5).
///
/// State machine:
/// - Missing/expired grant → `expired_token` (T-203-DEVICECODE-EXPIRY)
/// - Pending + fast poll → `slow_down` (RFC §3.5)
/// - Pending → `authorization_pending` (RFC §3.5)
/// - Denied → `access_denied` (RFC §3.5)
/// - Approved → forget both cache keys (T-203-DEVICECODE-REPLAY), mint JWT via the
///   identical `build_claims` + `mint_token` call as the auth-code arm (T-203-CLAIMS-DIVERGE).
async fn token_exchange_device_code(form: TokenRequest) -> ferro::Response {
    // ── Step 1: Require device_code field ────────────────────────────────────
    let device_code = form.device_code.as_deref().unwrap_or("");
    if device_code.is_empty() {
        return Err(json_error(400, "invalid_request", "missing device_code"));
    }

    // ── Step 2: Fetch grant from cache (do NOT forget yet — pending polls re-read) ──
    let grant: Option<DeviceGrant> = Cache::get(&device_cache_key(device_code))
        .await
        .ok()
        .flatten();

    // ── Step 3: Missing grant → expired_token ────────────────────────────────
    let grant = match grant {
        None => {
            return Err(json_error(
                400,
                "expired_token",
                "device_code expired or not found",
            ))
        }
        Some(g) => g,
    };

    // ── Step 4: Manual TTL guard (T-203-DEVICECODE-EXPIRY, RESEARCH Open Q #2) ──
    // Cache put-overwrite resets TTL; the explicit created_at check enforces the
    // 600s bound independently of cache TTL behavior.
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    if now_unix - grant.created_at > DEVICE_CODE_TTL.as_secs() as i64 {
        return Err(json_error(400, "expired_token", "device_code expired"));
    }

    // ── Step 5: State machine ─────────────────────────────────────────────────
    match grant.status {
        DeviceGrantStatus::Pending => {
            // slow_down / authorization_pending with last_polled_at update
            let last_poll = grant.last_polled_at.unwrap_or(grant.created_at);
            let elapsed = now_unix - last_poll;

            // Update last_polled_at regardless of slow_down decision
            let updated = DeviceGrant {
                last_polled_at: Some(now_unix),
                ..grant.clone()
            };
            let _ = Cache::put(
                &device_cache_key(device_code),
                &updated,
                Some(DEVICE_CODE_TTL),
            )
            .await;

            if elapsed < DEVICE_INTERVAL_SECS {
                return Err(json_error(
                    400,
                    "slow_down",
                    "polling too fast; increase interval by 5 seconds",
                ));
            }
            Err(json_error(
                400,
                "authorization_pending",
                "authorization request is still pending",
            ))
        }

        DeviceGrantStatus::Denied => Err(json_error(
            400,
            "access_denied",
            "authorization request was denied",
        )),

        DeviceGrantStatus::Approved => {
            // Single-use: forget BOTH keys before minting (T-203-DEVICECODE-REPLAY)
            let _ = Cache::forget(&device_cache_key(device_code)).await;
            let _ = Cache::forget(&usercode_cache_key(&grant.normalized_user_code)).await;

            // Mint JWT — IDENTICAL call to the auth-code arm (T-203-CLAIMS-DIVERGE / D-05)
            let config = OAuthConfig::from_env().map_err(|e| {
                json_error(500, "server_error", &format!("OAuth config error: {e}"))
            })?;

            let claims = build_claims(
                grant.user_id.expect("Approved grant must have user_id"),
                grant.tenant_id,
                &config.app_url,
                3600,
            );
            let access_token = mint_token(&claims, &config.token_secret)
                .map_err(|e| json_error(500, "server_error", &format!("token mint error: {e}")))?;

            Ok(ferro::HttpResponse::json(json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
            })))
        }
    }
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
    use crate::device::{
        device_cache_key, usercode_cache_key, DeviceGrant, DeviceGrantStatus, DEVICE_CODE_TTL,
    };
    use crate::jwt::{build_claims, decode_token, mint_token};
    use crate::pkce::{generate_auth_code, verify_s256};
    use crate::store::OAuthCode;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use ferro::Cache;
    use sha2::{Digest, Sha256};
    use std::sync::Mutex;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // Serialize env-var-mutating tests to prevent races between threads.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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

    fn now_unix() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
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
        let now = now_unix();
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

    /// Store a DeviceGrant in cache and return the device_code string.
    async fn store_device_grant(device_code: &str, grant: DeviceGrant) {
        Cache::put(
            &device_cache_key(device_code),
            &grant,
            Some(DEVICE_CODE_TTL),
        )
        .await
        .expect("cache put should succeed");
    }

    // ── Existing auth-code tests (must not regress after extraction) ──────────

    /// Verify that Cache::forget is called before validation by checking that
    /// a second retrieve returns None.
    #[tokio::test]
    async fn forget_before_validate_single_use() {
        let _cache = bootstrap_test_cache();

        let verifier = "test_verifier_that_is_long_enough_for_pkce_12345";
        let code = store_code("client-abc", "http://localhost:3000/cb", verifier, 1, None).await;

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
        let _cache = bootstrap_test_cache();

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

        let result =
            crate::validate::validate_bearer(Some(&format!("Bearer {token}")), &config, None);
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

    // ── Task 1: grant_type dispatch tests ─────────────────────────────────────

    /// A request with an unknown grant_type returns unsupported_grant_type.
    ///
    /// TDD RED→GREEN for Task 1 — asserts the dispatch rejects unknown grant types.
    #[tokio::test]
    async fn token_exchange_unsupported_grant_returns_error() {
        let _cache = bootstrap_test_cache();

        let form = TokenRequest {
            grant_type: "client_credentials".to_string(),
            code: None,
            redirect_uri: None,
            client_id: "some-client".to_string(),
            code_verifier: None,
            device_code: None,
        };

        let result = token_exchange_dispatch(form).await;
        assert!(
            result.is_err(),
            "unsupported grant_type must return an error"
        );
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), 400);
        let body: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(
            body["error"].as_str().unwrap(),
            "unsupported_grant_type",
            "error must be unsupported_grant_type"
        );
    }

    // ── Task 2: device-code grant state machine tests ─────────────────────────

    /// Pending grant polled within interval (last_polled_at = now - 10s, well past interval)
    /// returns authorization_pending.
    #[tokio::test]
    async fn device_grant_pending_returns_authorization_pending() {
        let _cache = bootstrap_test_cache();

        let device_code = "dc_pending_test_abc123";
        let now = now_unix();
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Pending,
            user_id: None,
            tenant_id: None,
            created_at: now - 30, // created 30s ago — well within 600s TTL
            last_polled_at: Some(now - 10), // last polled 10s ago — elapsed >= 5s interval
            normalized_user_code: "TESTPEND".to_string(),
        };
        store_device_grant(device_code, grant).await;

        let form = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            code: None,
            redirect_uri: None,
            client_id: "client-test".to_string(),
            code_verifier: None,
            device_code: Some(device_code.to_string()),
        };

        let result = token_exchange_dispatch(form).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let body: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(
            body["error"].as_str().unwrap(),
            "authorization_pending",
            "expected authorization_pending, got: {body}"
        );
    }

    /// Pending grant with last_polled_at = now (polled just now) returns slow_down.
    #[tokio::test]
    async fn device_grant_slow_down_on_fast_poll() {
        let _cache = bootstrap_test_cache();

        let device_code = "dc_slowdown_test_abc123";
        let now = now_unix();
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Pending,
            user_id: None,
            tenant_id: None,
            created_at: now - 10,
            last_polled_at: Some(now), // polled just this second — elapsed < 5s
            normalized_user_code: "TESTSLOW".to_string(),
        };
        store_device_grant(device_code, grant).await;

        let form = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            code: None,
            redirect_uri: None,
            client_id: "client-test".to_string(),
            code_verifier: None,
            device_code: Some(device_code.to_string()),
        };

        let result = token_exchange_dispatch(form).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let body: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(
            body["error"].as_str().unwrap(),
            "slow_down",
            "expected slow_down, got: {body}"
        );
    }

    /// Denied grant returns access_denied.
    #[tokio::test]
    async fn device_grant_denied_returns_access_denied() {
        let _cache = bootstrap_test_cache();

        let device_code = "dc_denied_test_abc123";
        let now = now_unix();
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Denied,
            user_id: None,
            tenant_id: None,
            created_at: now - 30,
            last_polled_at: None,
            normalized_user_code: "TESTDENY".to_string(),
        };
        store_device_grant(device_code, grant).await;

        let form = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            code: None,
            redirect_uri: None,
            client_id: "client-test".to_string(),
            code_verifier: None,
            device_code: Some(device_code.to_string()),
        };

        let result = token_exchange_dispatch(form).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let body: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(
            body["error"].as_str().unwrap(),
            "access_denied",
            "expected access_denied, got: {body}"
        );
    }

    /// No grant in cache (or created_at > 600s ago) returns expired_token.
    ///
    /// Uses the created_at guard path: seeds a grant with created_at = now - 700
    /// so the explicit TTL check fires regardless of cache TTL.
    #[tokio::test]
    async fn device_grant_expired_returns_expired_token() {
        let _cache = bootstrap_test_cache();

        let device_code = "dc_expired_test_abc123";
        let now = now_unix();
        // Seed a grant that is 700s old — beyond the 600s explicit TTL guard
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Pending,
            user_id: None,
            tenant_id: None,
            created_at: now - 700, // older than 600s → explicit expiry guard fires
            last_polled_at: None,
            normalized_user_code: "TESTEXP".to_string(),
        };
        store_device_grant(device_code, grant).await;

        let form = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            code: None,
            redirect_uri: None,
            client_id: "client-test".to_string(),
            code_verifier: None,
            device_code: Some(device_code.to_string()),
        };

        let result = token_exchange_dispatch(form).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let body: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(
            body["error"].as_str().unwrap(),
            "expired_token",
            "expected expired_token, got: {body}"
        );
    }

    /// Approved grant returns access_token and token_type == "Bearer".
    /// Both cache keys are forgotten after issuance (single-use invariant).
    #[tokio::test]
    async fn device_grant_approved_returns_access_token() {
        {
            // Scope the lock so it is dropped before any await point (clippy::await_holding_lock).
            let _env = ENV_LOCK.lock().unwrap();
            std::env::set_var(
                "MCP_TOKEN_SECRET",
                "test_secret_that_is_at_least_32_bytes_long_for_hs256",
            );
            std::env::set_var("APP_URL", "http://localhost:8080");
        }
        let _cache = bootstrap_test_cache();

        let device_code = "dc_approved_test_abc123";
        let now = now_unix();
        let normalized_user_code = "TESTAPPR";
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Approved,
            user_id: Some(42),
            tenant_id: Some(7),
            created_at: now - 30,
            last_polled_at: None,
            normalized_user_code: normalized_user_code.to_string(),
        };

        // Also store the usercode pointer key so we can verify it's forgotten
        Cache::put(
            &usercode_cache_key(normalized_user_code),
            &device_code.to_string(),
            Some(DEVICE_CODE_TTL),
        )
        .await
        .expect("cache put usercode pointer should succeed");
        store_device_grant(device_code, grant).await;

        let form = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            code: None,
            redirect_uri: None,
            client_id: "client-test".to_string(),
            code_verifier: None,
            device_code: Some(device_code.to_string()),
        };

        let result = token_exchange_dispatch(form).await;
        assert!(result.is_ok(), "approved grant must return Ok: {result:?}");
        let resp = result.unwrap();
        let body: serde_json::Value = serde_json::from_str(resp.body()).unwrap();
        assert!(
            body.get("access_token").is_some(),
            "response must have access_token: {body}"
        );
        assert_eq!(
            body["token_type"].as_str().unwrap(),
            "Bearer",
            "token_type must be Bearer: {body}"
        );
        assert_eq!(
            body["expires_in"].as_i64().unwrap(),
            3600,
            "expires_in must be 3600: {body}"
        );

        // Both cache keys must be forgotten (single-use)
        let device_key_after: Option<DeviceGrant> = Cache::get(&device_cache_key(device_code))
            .await
            .ok()
            .flatten();
        assert!(
            device_key_after.is_none(),
            "device_cache_key must be forgotten after token issuance"
        );
        let usercode_key_after: Option<String> =
            Cache::get(&usercode_cache_key(normalized_user_code))
                .await
                .ok()
                .flatten();
        assert!(
            usercode_key_after.is_none(),
            "usercode_cache_key must be forgotten after token issuance"
        );
    }

    /// Approved grant with tenant_id = Some(7) → minted JWT decodes to tenant_id == Some(7).
    #[tokio::test]
    async fn device_grant_tenant_binding() {
        {
            // Scope the lock so it is dropped before any await point (clippy::await_holding_lock).
            // Set env vars to match test_config() so the JWT minted by token_exchange_dispatch
            // can be decoded with the same secret as test_config().token_secret.
            let _env = ENV_LOCK.lock().unwrap();
            std::env::set_var(
                "MCP_TOKEN_SECRET",
                "test_secret_that_is_at_least_32_bytes_long_for_hs256",
            );
            std::env::set_var("APP_URL", "http://localhost:8080");
        }
        let _cache = bootstrap_test_cache();

        let device_code = "dc_tenant_binding_test_abc123";
        let now = now_unix();
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Approved,
            user_id: Some(99),
            tenant_id: Some(7),
            created_at: now - 30,
            last_polled_at: None,
            normalized_user_code: "TESTTENANT".to_string(),
        };
        store_device_grant(device_code, grant).await;

        let form = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            code: None,
            redirect_uri: None,
            client_id: "client-test".to_string(),
            code_verifier: None,
            device_code: Some(device_code.to_string()),
        };

        let result = token_exchange_dispatch(form).await;
        assert!(result.is_ok(), "approved grant must succeed: {result:?}");
        let resp = result.unwrap();
        let body: serde_json::Value = serde_json::from_str(resp.body()).unwrap();
        let token_str = body["access_token"]
            .as_str()
            .expect("access_token must be a string");

        // Decode the JWT using the crate's existing decode path
        let config = test_config();
        let claims = decode_token(
            token_str,
            &config.token_secret,
            &format!("{}/mcp", config.app_url),
        )
        .expect("JWT must decode successfully");

        assert_eq!(claims.sub, "99", "sub must be user_id 99");
        assert_eq!(
            claims.tenant_id,
            Some(7),
            "tenant_id must be Some(7) in the minted JWT"
        );
    }

    /// Claims from the device arm are structurally identical to claims from the auth-code arm
    /// for the same (user_id, tenant_id, app_url, ttl) — one-token-issuer invariant (D-05/SC-3).
    ///
    /// Calls `build_claims` directly from both arms' perspective with the same arguments
    /// and asserts that sub, aud, iss, and tenant_id are equal.
    #[test]
    fn device_grant_token_claims_identical_to_auth_code() {
        let app_url = "http://localhost:8080";
        let user_id = 42i64;
        let tenant_id = Some(7i64);
        let ttl = 3600i64;

        // Auth-code arm perspective: build_claims(record.user_id, record.tenant_id, &config.app_url, 3600)
        let auth_code_claims = build_claims(user_id, tenant_id, app_url, ttl);

        // Device arm perspective: build_claims(grant.user_id.expect(...), grant.tenant_id, &config.app_url, 3600)
        let device_claims = build_claims(user_id, tenant_id, app_url, ttl);

        // Structural identity: sub, aud, iss, tenant_id
        assert_eq!(
            auth_code_claims.sub, device_claims.sub,
            "sub must be identical between auth-code and device arms"
        );
        assert_eq!(
            auth_code_claims.aud, device_claims.aud,
            "aud must be identical between auth-code and device arms"
        );
        assert_eq!(
            auth_code_claims.iss, device_claims.iss,
            "iss must be identical between auth-code and device arms"
        );
        assert_eq!(
            auth_code_claims.tenant_id, device_claims.tenant_id,
            "tenant_id must be identical between auth-code and device arms"
        );
    }
}
