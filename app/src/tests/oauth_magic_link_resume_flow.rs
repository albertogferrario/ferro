//! SC-3 acceptance test: OAuth async-flow resume contract.
//!
//! Stages the full logical sequence of an unauthenticated user completing a
//! magic-link login during an in-flight OAuth authorize request:
//!
//! 1. `/authorize` stores the return URL in the session (Step 1).
//! 2. `POST /auth/login` issues a single-use token into the cache (Step 2).
//! 3. `GET /auth/verify?token=` consumes the token (Step 3).
//! 4. `oauth_resume_redirect("/")` resumes to the stored `/authorize` URL (Step 4).
//! 5. With no stored key, `oauth_resume_redirect("/")` falls back to `"/"` (Step 5).
//!
//! ## Offline guarantee
//!
//! This test uses in-memory cache (`bootstrap_test_cache`) and session
//! (`with_test_session`) only. No network calls, no live HTTP server, no SMTP,
//! no CWD-relative view rendering. CI is always offline-green.

use ferro::Cache;
use ferro::session::with_test_session;
use ferro_mcp_oauth::{oauth_resume_redirect, store_oauth_return_to};
use ferro_mcp_oauth::cache_test_helpers::bootstrap_test_cache;
use std::time::Duration;

/// SC-3: Full async OAuth resume flow — store → issue → consume → resume.
///
/// Walks the four logical steps of the async magic-link flow within a shared
/// session scope so that the `oauth_return_to` key stored in Step 1 is visible
/// to the redirect in Step 4.
#[tokio::test]
async fn oauth_magic_link_resume_flow() {
    bootstrap_test_cache();

    with_test_session("sc3_flow", || async {
        // ── SC-3 Step 1: unauthenticated /authorize stored the return target ──
        //
        // The authorize handler calls `store_oauth_return_to` when it detects an
        // unauthenticated request and redirects the browser to /auth/login.
        let authorize_url = "/authorize?client_id=test&redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback&response_type=code&state=abc";
        store_oauth_return_to(authorize_url.to_string());

        // ── SC-3 Step 2: request-link handler issued a token into the cache ──
        //
        // POST /auth/login generates a high-entropy token and stores it as
        // "magic_link:{token}" in ferro-cache with a 15-minute TTL.
        let token = "flow_token_unique_sc3_202_04";
        let user_id: i64 = 7;
        let key = format!("magic_link:{token}");

        Cache::put(&key, &user_id, Some(Duration::from_secs(15 * 60)))
            .await
            .expect("cache put must succeed — Step 2 token issued");

        // Token is present immediately after issue.
        let present = Cache::get::<i64>(&key)
            .await
            .expect("cache get must not error")
            .is_some();
        assert!(present, "token must be present in cache after issue (Step 2)");

        // ── SC-3 Step 3: verify consumes the token (single-use) ──
        //
        // GET /auth/verify?token= reads the token then deletes it unconditionally
        // (forget-before-validate, mirrors token.rs lines 62-64). A second read
        // must return None, proving replay is not possible (T-202-01).
        let looked_up: Option<i64> = Cache::get(&key)
            .await
            .ok()
            .flatten();
        let _ = Cache::forget(&key).await;

        assert_eq!(looked_up, Some(user_id), "verify must retrieve the user_id from cache (Step 3)");

        let after_forget: Option<i64> = Cache::get(&key)
            .await
            .expect("cache get must not error");
        assert!(
            after_forget.is_none(),
            "token must be gone after forget — single-use invariant (T-202-01, Step 3)"
        );

        // ── SC-3 Step 4: resume redirect targets the stored /authorize URL ──
        //
        // After Auth::login, the verify handler calls `oauth_resume_redirect("/")`.
        // With the return_to key present it must redirect to the stored authorize URL,
        // not to the fallback "/".
        let resp = oauth_resume_redirect("/").expect("oauth_resume_redirect must return Ok(...)");

        assert_eq!(resp.status_code(), 302, "resume redirect must be 302 (Step 4)");

        let location = resp
            .headers()
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("Location"))
            .map(|(_, v)| v.as_str());

        assert_eq!(
            location,
            Some(authorize_url),
            "Location must be the stored /authorize URL, not the fallback (Step 4)"
        );
    })
    .await;
}

/// SC-3 Step 5 (no-key default): when no authorize flow is in progress,
/// `oauth_resume_redirect` falls back to the caller-supplied default.
///
/// This proves the helper is safe to call from any login handler — handlers
/// that are not initiated from an OAuth flow are redirected to the application
/// dashboard instead of erroring.
#[tokio::test]
async fn oauth_magic_link_resume_flow_no_key_falls_back_to_default() {
    // Fresh session: no oauth_return_to key present.
    with_test_session("sc3_nokey", || async {
        let resp = oauth_resume_redirect("/")
            .expect("oauth_resume_redirect must return Ok(...) even with no stored key");

        assert_eq!(resp.status_code(), 302, "fallback redirect must be 302 (Step 5)");

        let location = resp
            .headers()
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("Location"))
            .map(|(_, v)| v.as_str());

        assert_eq!(
            location,
            Some("/"),
            "Location must be the default '/' when no oauth_return_to is stored (Step 5)"
        );
    })
    .await;
}
