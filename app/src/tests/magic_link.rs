/// Unit tests for the magic-link token lifecycle.
///
/// Covers T-202-01 (replay/single-use), T-202-02 (expiry/absent-key),
/// and the dev-mode gate (D-03 branch selection).
///
/// None of these tests call `JsonUi::render_file` — they exercise only the
/// cache layer and environment detection (RESEARCH Pitfall 2: view files are
/// CWD-relative at request time; unit tests avoid that).
use ferro::Cache;
use ferro::config::env::Environment;
use ferro_mcp_oauth::cache_test_helpers::bootstrap_test_cache;
use std::time::Duration;

/// T-202-01: A magic-link token is single-use.
///
/// After `Cache::forget`, a second `Cache::get` must return `None`.
/// This mirrors the forget-before-validate invariant in `verify_magic_link`
/// (token.rs lines 62-64 pattern).
#[tokio::test]
async fn magic_link_single_use() {
    bootstrap_test_cache();

    let token = "test_token_unique_42";
    let key = format!("magic_link:{token}");
    let user_id: i64 = 42;

    // Put the token with a 15-minute TTL (same as production handler).
    Cache::put(&key, &user_id, Some(Duration::from_secs(15 * 60)))
        .await
        .expect("cache put should succeed");

    // First get: token is present.
    let first: Option<i64> = Cache::get(&key).await.ok().flatten();
    assert!(first.is_some(), "token should exist before forget");
    assert_eq!(first.unwrap(), 42);

    // Forget (single-use: always forget, regardless of validation result).
    let _ = Cache::forget(&key).await;

    // Second get: token must be gone (single-use invariant).
    let second: Option<i64> = Cache::get(&key).await.ok().flatten();
    assert!(
        second.is_none(),
        "token must be gone after forget — single-use invariant (T-202-01)"
    );
}

/// T-202-02: An absent or expired token maps to `None` from `Cache::get`.
///
/// When the key was never stored (simulates an expired or invalid token),
/// `Cache::get` returns `None`. The verify handler maps this to the error
/// re-render path. Tests the observable behavior of an expired token — the
/// InMemoryCache may not advance wall-clock TTL in tests, so we assert the
/// absent-key path directly.
#[tokio::test]
async fn magic_link_expired() {
    bootstrap_test_cache();

    let token = "token_that_was_never_stored_or_already_expired_unique_99";
    let key = format!("magic_link:{token}");

    // An absent key returns None.
    let result: Option<i64> = Cache::get(&key).await.ok().flatten();
    assert!(
        result.is_none(),
        "absent key must yield None — maps to error path in verify handler (T-202-02)"
    );

    // Forget on absent key is a no-op (idempotent — mirrors production handler).
    let _ = Cache::forget(&key).await;

    // Still None after forget.
    let after_forget: Option<i64> = Cache::get(&key).await.ok().flatten();
    assert!(
        after_forget.is_none(),
        "key must remain None after forget on absent key"
    );
}

/// D-03: In development (`APP_ENV=local`), `Environment::detect().is_development()` is true.
///
/// Proves that the dev branch (link surfaced on page, no real mail) is selected
/// in the default test environment. Restores the env var after assertion to avoid
/// leaking state to other tests.
#[test]
fn magic_link_dev_surface() {
    // APP_ENV=local is the default for development and tests.
    std::env::set_var("APP_ENV", "local");
    let is_dev = Environment::detect().is_development();
    // Restore to avoid env-state leakage across tests.
    std::env::remove_var("APP_ENV");

    assert!(
        is_dev,
        "Environment::detect().is_development() must be true for APP_ENV=local \
         so the dev branch (link surfaced, no real mail) is selected (D-03)"
    );
}
