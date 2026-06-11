//! Full PKCE flow integration test.
//!
//! Proves the DCR→authorize→consent→token→validate chain in-process
//! with no external IdP. Drives the core logic functions directly
//! (store, cache, pkce, jwt, validate_bearer).
//!
//! Steps:
//! 1. Setup: in-memory SQLite + migration + in-memory cache bootstrap.
//! 2. DCR: insert_client with redirect_uris=["http://localhost:3000/callback"].
//! 3. PKCE pair: generate code_verifier + compute code_challenge (S256).
//! 4. Authorize (authenticated): validate client + redirect_uri, render consent HTML.
//! 5. Consent approve: mint single-use auth code, store in cache (60s TTL).
//! 6. Token: retrieve-then-forget code, verify PKCE, mint JWT.
//! 7. Validate: validate_bearer → Authenticated with correct sub + tenant_id.
//! 8. Replay guard: second token attempt with same code → invalid_grant (code gone).

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ferro::Cache;
use ferro_mcp_oauth::{
    cache_test_helpers::bootstrap_test_cache,
    config::OAuthConfig,
    consent::render_consent_html,
    jwt::{build_claims, mint_token},
    pkce::{generate_auth_code, verify_s256},
    store::{find_by_client_id, insert_client, OAuthCode},
    validate::{validate_bearer, BearerCheck},
    CreateOauthClientsTable,
};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Boot an in-memory SQLite database and apply the oauth_clients migration.
async fn fresh_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");

    struct TestMigrator;

    #[async_trait::async_trait]
    impl sea_orm_migration::MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(CreateOauthClientsTable)]
        }
    }

    TestMigrator::up(&conn, None)
        .await
        .expect("apply oauth_clients migration");

    conn
}

/// Set a deterministic test MCP_TOKEN_SECRET (>= 32 bytes) and return OAuthConfig.
fn test_oauth_config() -> OAuthConfig {
    std::env::set_var(
        "MCP_TOKEN_SECRET",
        "test_secret_that_is_at_least_32_bytes_long_for_hs256",
    );
    std::env::set_var("APP_URL", "http://localhost:8080");
    std::env::set_var("APP_NAME", "TestApp");
    OAuthConfig::from_env().expect("OAuthConfig::from_env should succeed with test secret")
}

/// Compute a PKCE S256 challenge from a verifier.
fn s256_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

#[tokio::test]
async fn full_pkce_flow() {
    // ── Step 1: Setup ─────────────────────────────────────────────────────────
    let db = fresh_db().await;
    let config = test_oauth_config();
    let _cache = bootstrap_test_cache();

    // ── Step 2: DCR — register a client ──────────────────────────────────────
    let redirect_uri = "http://localhost:3000/callback".to_string();
    let redirect_uris_json = serde_json::to_string(&[&redirect_uri]).unwrap();

    // Generate a client_id using the same helper as register.rs
    let client_id = ferro_mcp_oauth::register::generate_client_id();

    insert_client(
        &db,
        client_id.clone(),
        Some("Test MCP Client".to_string()),
        redirect_uris_json,
    )
    .await
    .expect("DCR insert should succeed");

    // Verify the client is findable
    let client = find_by_client_id(&db, &client_id)
        .await
        .expect("find should not error")
        .expect("client must exist after DCR");
    assert_eq!(client.client_id, client_id);

    // ── Step 3: PKCE pair ─────────────────────────────────────────────────────
    let code_verifier = "test_code_verifier_that_is_long_enough_for_pkce_requirements_abc123";
    let code_challenge = s256_challenge(code_verifier);

    // Sanity check: verify_s256 should pass with the correct verifier
    assert!(
        verify_s256(code_verifier, &code_challenge),
        "S256 sanity: correct verifier must match challenge"
    );

    // ── Step 4: Authorize (authenticated) ────────────────────────────────────
    // Simulate what authorize_get does after Auth::check() succeeds:
    // - Validate client exists (done above)
    // - Validate redirect_uri exact-match
    let stored_uris: Vec<String> = serde_json::from_str(&client.redirect_uris).unwrap_or_default();
    assert!(
        stored_uris.iter().any(|u| u == &redirect_uri),
        "redirect_uri must match registered URIs"
    );

    // Render consent HTML (proves SC-3 first half: consent page is produced)
    let test_user_id: i64 = 42;
    let test_tenant_id: Option<i64> = Some(7);
    let csrf_token = "test_csrf_token_for_integration_test";

    let consent_html = render_consent_html(
        &client.client_name.clone().unwrap_or_default(),
        &client_id,
        &redirect_uri,
        &code_challenge,
        "test_state",
        csrf_token,
        test_user_id,
        test_tenant_id,
    );

    // Assert consent page contains required fields
    assert!(
        consent_html.contains(r#"name="_token""#),
        "consent HTML must have CSRF field"
    );
    assert!(
        consent_html.contains(csrf_token),
        "consent HTML must embed CSRF token"
    );
    assert!(
        consent_html.contains("value=\"S256\""),
        "consent HTML must have S256 method"
    );
    assert!(
        consent_html.contains(&code_challenge),
        "consent HTML must embed code_challenge"
    );

    // ── Step 5: Consent approve — mint single-use auth code ──────────────────
    // Simulate what authorize_post does after CSRF validation and re-validation:
    let code = generate_auth_code();
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let oauth_code = OAuthCode {
        client_id: client_id.clone(),
        redirect_uri: redirect_uri.clone(),
        code_challenge: code_challenge.clone(),
        user_id: test_user_id,
        tenant_id: test_tenant_id,
        created_at: now_unix,
    };

    // Store with 60s TTL (T-199-03)
    let code_key = format!("mcp:code:{code}");
    Cache::put(&code_key, &oauth_code, Some(Duration::from_secs(60)))
        .await
        .expect("cache put should succeed (T-199-03)");

    // ── Step 6: Token — forget-before-validate (T-199-02) ────────────────────
    // Retrieve THEN forget BEFORE any validation — single-use guarantee
    let retrieved: Option<OAuthCode> = Cache::get(&code_key).await.ok().flatten();
    let _ = Cache::forget(&code_key).await; // single-use: forget regardless of validation outcome

    let record = retrieved.expect("code must exist on first retrieval");

    // Validate client_id + redirect_uri (T-199-16)
    assert_eq!(record.client_id, client_id, "client_id must match");
    assert_eq!(record.redirect_uri, redirect_uri, "redirect_uri must match");

    // PKCE verify (T-199-01)
    assert!(
        verify_s256(code_verifier, &record.code_challenge),
        "PKCE S256 must pass with correct verifier"
    );

    // Mint JWT
    let claims = build_claims(record.user_id, record.tenant_id, &config.app_url, 3600);
    let access_token =
        mint_token(&claims, &config.token_secret).expect("token mint should succeed");
    assert!(!access_token.is_empty(), "access_token must be non-empty");

    // ── Step 7: Validate bearer → Authenticated ───────────────────────────────
    let auth_header = format!("Bearer {access_token}");
    let result = validate_bearer(Some(&auth_header), &config, None);

    match result {
        BearerCheck::Authenticated(principal) => {
            assert_eq!(
                principal["sub"],
                serde_json::json!(test_user_id.to_string()),
                "sub must match test user_id"
            );
            assert_eq!(
                principal["tenant_id"],
                serde_json::json!(7_i64),
                "tenant_id must match test tenant"
            );
        }
        other => panic!("expected BearerCheck::Authenticated, got {other:?}"),
    }

    // ── Step 8: Replay guard (T-199-02) ──────────────────────────────────────
    // The code was already forgotten in Step 6. A second attempt must return None.
    let replay_attempt: Option<OAuthCode> = Cache::get(&code_key).await.ok().flatten();
    assert!(
        replay_attempt.is_none(),
        "replayed code must return None — forget-before-validate ensures single-use"
    );
    // Even if someone calls forget again, it's a no-op (idempotent)
    let _ = Cache::forget(&code_key).await;
    let replay_attempt2: Option<OAuthCode> = Cache::get(&code_key).await.ok().flatten();
    assert!(
        replay_attempt2.is_none(),
        "second replay attempt must also return None"
    );
}
