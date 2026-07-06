//! Login-resume contract. Owns the `oauth_return_to` session key.
//!
//! Any login handler that calls [`oauth_resume_redirect`] (or
//! [`take_oauth_return_to`]) participates in the OAuth authorize flow; a handler
//! that does not will dead-end on its own default instead of resuming the
//! in-flight authorize request.
//!
//! # Contract
//!
//! When `ferro-mcp-oauth`'s `/authorize` endpoint receives an unauthenticated
//! request, it stores the in-flight authorize URL in the session via
//! [`store_oauth_return_to`] and redirects the user to the app login page.
//! After authentication the login handler must call [`oauth_resume_redirect`]
//! (or [`take_oauth_return_to`] directly) so the OAuth flow resumes and an
//! authorization code is issued.
//!
//! **Any login method** (synchronous password, asynchronous magic-link, future
//! SSO) must call [`oauth_resume_redirect`] or [`take_oauth_return_to`] after
//! establishing the session to participate in the OAuth flow. A handler that
//! redirects to a fixed dashboard instead will never resume the authorize
//! request.
//!
//! # Open-redirect invariant
//!
//! The stored value is written exclusively by the `/authorize` handler from a
//! URL it constructs itself (a validated, internal `/authorize?...` URL). The
//! caller-supplied `default` in [`oauth_resume_redirect`] is a static internal
//! path. This helper never reads user input and therefore never redirects to an
//! attacker-controlled URL (mitigates T-199-04).

use ferro::session::{session, session_mut};

/// Session key that holds the in-flight authorize URL.
///
/// This constant is the single owner of the key string in this crate.
/// No other module in `ferro-mcp-oauth` should reference the string
/// `"oauth_return_to"` directly.
const OAUTH_RETURN_TO_KEY: &str = "oauth_return_to";

/// Store the in-flight authorize URL so a later login handler can resume it.
///
/// Called by the `/authorize` handler when redirecting an unauthenticated user
/// to login. The stored URL originates from the authorize handler itself and is
/// never user-supplied.
pub fn store_oauth_return_to(url: String) {
    session_mut(|s| {
        s.put(OAUTH_RETURN_TO_KEY, url);
    });
}

/// Take the stored return URL, clearing it from the session (consume-on-read).
///
/// Returns `None` when no authorize flow is in progress (key absent or no
/// active session).
pub fn take_oauth_return_to() -> Option<String> {
    let url: Option<String> = session().and_then(|s| s.get(OAUTH_RETURN_TO_KEY));
    if url.is_some() {
        session_mut(|s| {
            s.forget(OAUTH_RETURN_TO_KEY);
        });
    }
    url
}

/// 302-redirect to the stored OAuth return URL, or to `default` when no
/// authorize flow is in progress. Consumes the stored key (consume-on-read).
///
/// Returns the success path of `ferro::Response` — callers use
/// `return oauth_resume_redirect("/")`, NOT `oauth_resume_redirect("/")?`.
///
/// # Open-redirect invariant
///
/// The stored value originates ONLY from the `/authorize` handler (a
/// validated, internal `/authorize?...` URL it constructs itself), never from
/// user input. `default` is a static internal path supplied by the caller.
/// The helper therefore never redirects to an attacker-controlled URL.
/// (Mitigates the carry-forward of T-199-04.)
pub fn oauth_resume_redirect(default: &str) -> ferro::Response {
    let dest = take_oauth_return_to().unwrap_or_else(|| default.to_string());
    Ok(ferro::HttpResponse::new()
        .status(302)
        .header("Location", dest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro::session::with_test_session;

    /// store then take returns the stored URL.
    #[tokio::test]
    async fn store_then_take_returns_stored_url() {
        with_test_session("t1", || async {
            store_oauth_return_to("/authorize?client_id=test&response_type=code".to_string());
            let url = take_oauth_return_to();
            assert_eq!(
                url,
                Some("/authorize?client_id=test&response_type=code".to_string())
            );
        })
        .await;
    }

    /// After take_oauth_return_to(), a second call returns None (consume-on-read).
    #[tokio::test]
    async fn take_clears_key_second_call_returns_none() {
        with_test_session("t2", || async {
            store_oauth_return_to("/authorize?client_id=test".to_string());
            let first = take_oauth_return_to();
            assert!(first.is_some(), "first take must return the stored URL");
            let second = take_oauth_return_to();
            assert_eq!(
                second, None,
                "second take must return None (consume-on-read)"
            );
        })
        .await;
    }

    /// take_oauth_return_to() with no key set returns None.
    #[tokio::test]
    async fn take_with_no_stored_key_returns_none() {
        with_test_session("t3", || async {
            let url = take_oauth_return_to();
            assert_eq!(url, None);
        })
        .await;
    }

    /// oauth_resume_redirect with a stored key returns 302 to the stored URL.
    #[tokio::test]
    async fn redirect_with_stored_key_returns_302_to_stored_url() {
        with_test_session("t4", || async {
            store_oauth_return_to("/authorize?client_id=abc&response_type=code".to_string());
            let resp = oauth_resume_redirect("/");
            let resp = resp.expect("oauth_resume_redirect returns Ok(...)");
            assert_eq!(resp.status_code(), 302);
            let location = resp
                .headers()
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("Location"))
                .map(|(_, v)| v.as_str());
            assert_eq!(
                location,
                Some("/authorize?client_id=abc&response_type=code"),
                "Location must be the stored authorize URL"
            );
        })
        .await;
    }

    /// oauth_resume_redirect with no stored key returns 302 to the default.
    #[tokio::test]
    async fn redirect_without_stored_key_returns_302_to_default() {
        with_test_session("t5", || async {
            let resp = oauth_resume_redirect("/dashboard");
            let resp = resp.expect("oauth_resume_redirect returns Ok(...)");
            assert_eq!(resp.status_code(), 302);
            let location = resp
                .headers()
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("Location"))
                .map(|(_, v)| v.as_str());
            assert_eq!(
                location,
                Some("/dashboard"),
                "Location must fall back to the default"
            );
        })
        .await;
    }
}
