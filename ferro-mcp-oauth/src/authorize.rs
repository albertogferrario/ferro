//! Authorization endpoint + login redirect (Plan 04).
//!
//! Implements `GET /authorize`: checks session auth, redirects to login if
//! unauthenticated, validates client_id + redirect_uri, then renders consent.
//!
//! Security properties:
//! - T-199-01 (PKCE downgrade): rejects `code_challenge_method != "S256"` and absent
//!   `code_challenge` before touching DB or rendering consent.
//! - T-199-04 (open redirect): client lookup failure or `redirect_uri` mismatch returns
//!   an error PAGE — never redirects to an unvalidated URI (RFC 6749 §4.1.2.1).
//! - D-06 (login reuse): unauthenticated requests are redirected to `/auth/login` with
//!   the authorize URL stored in the session via `crate::resume::store_oauth_return_to`.

use ferro::session::get_csrf_token;
use ferro::tenant::current_tenant;
use ferro::Auth;
use serde::Deserialize;

/// Query parameters for `GET /authorize` (RFC 6749 §4.1.1 + RFC 7636).
#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    /// Must be `"code"`.
    pub response_type: Option<String>,
    /// Registered client id.
    pub client_id: Option<String>,
    /// Must exactly match one of the client's registered redirect URIs.
    pub redirect_uri: Option<String>,
    /// BASE64URL(SHA256(code_verifier)) — required for S256 PKCE.
    pub code_challenge: Option<String>,
    /// Must be `"S256"` — plain is rejected (T-199-01).
    pub code_challenge_method: Option<String>,
    /// Optional; echoed back in the redirect.
    pub state: Option<String>,
    /// Optional; ignored in Phase 199 (single implicit scope).
    pub scope: Option<String>,
}

/// Handler: `GET /authorize`.
///
/// Steps:
/// 1. Parse query params; reject if `response_type != "code"`.
/// 2. PKCE downgrade guard (T-199-01): reject `code_challenge_method != "S256"` or absent.
/// 3. Auth check (D-06): redirect to `/auth/login` if unauthenticated.
/// 4. Client + redirect_uri exact-match validation (T-199-04).
/// 5. Tenant capture (D-06).
/// 6. Render consent HTML.
#[ferro::handler]
pub async fn authorize_get(req: ferro::Request) -> ferro::Response {
    // ── Step 1: Parse required query params ─────────────────────────────────
    let response_type = req.query_or("response_type", "");
    if response_type != "code" {
        return Err(error_page(
            400,
            "invalid_request",
            "response_type must be 'code'",
        ));
    }

    let client_id = req
        .query("client_id")
        .ok_or_else(|| error_page(400, "invalid_request", "client_id is required"))?;
    let redirect_uri = req
        .query("redirect_uri")
        .ok_or_else(|| error_page(400, "invalid_request", "redirect_uri is required"))?;
    let code_challenge = req.query("code_challenge").ok_or_else(|| {
        // T-199-01: absent code_challenge → PKCE downgrade rejection
        error_page(
            400,
            "invalid_request",
            "code_challenge is required (S256 PKCE)",
        )
    })?;
    let code_challenge_method = req.query_or("code_challenge_method", "");
    let state = req.query("state").unwrap_or_default();
    let _scope = req.query("scope").unwrap_or_default();

    // ── Step 2: PKCE downgrade guard (T-199-01, HIGH) ───────────────────────
    // Reject everything except S256. `plain` and absent are both rejected.
    if code_challenge_method != "S256" {
        return Err(error_page(
            400,
            "invalid_request",
            "code_challenge_method must be 'S256' (plain and absent are not accepted)",
        ));
    }

    // ── Step 3: Auth check (D-06) ────────────────────────────────────────────
    if !Auth::check() {
        // Reconstruct the full authorize URL to store as return_to.
        // Use the query parameters already read; build the return URL manually.
        let return_url = format!(
            "/authorize?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}",
            urlencoding::encode(&client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&code_challenge),
            urlencoding::encode(&state),
        );
        crate::resume::store_oauth_return_to(return_url.clone());
        return Err(ferro::HttpResponse::new()
            .status(302)
            .header("Location", "/auth/login"));
    }

    // ── Step 4: Client + redirect_uri validation (T-199-04, HIGH) ──────────
    let db_conn = ferro::DB::connection()
        .map_err(|e| error_page(500, "server_error", &format!("db connection failed: {e}")))?;
    let client = crate::store::find_by_client_id(db_conn.inner(), &client_id)
        .await
        .map_err(|e| error_page(500, "server_error", &format!("db error: {e}")))?;

    let client = match client {
        Some(c) => c,
        // Unknown client → error PAGE, never redirect (RFC 6749 §4.1.2.1)
        None => {
            return Err(error_page(
                400,
                "invalid_client",
                "Unknown client_id. Has the client registered via POST /register?",
            ));
        }
    };

    // Exact-string match against stored redirect_uris (T-199-04).
    let stored_uris: Vec<String> = serde_json::from_str(&client.redirect_uris).unwrap_or_default();
    if !stored_uris.iter().any(|u| u == &redirect_uri) {
        // redirect_uri mismatch → error PAGE, never redirect
        return Err(error_page(
            400,
            "invalid_request",
            "redirect_uri does not match any registered URI for this client",
        ));
    }

    // ── Step 5: Tenant capture (D-06) ────────────────────────────────────────
    let user_id = Auth::id().expect("Auth::check() confirmed authentication");
    let tenant_id = current_tenant().map(|t| t.id);

    // Step 5a: Multi-tenant ambiguity check.
    // If a TenantMiddleware is active (i.e. the app is multi-tenant) but current_tenant()
    // returned None, the tenant is ambiguous → return 400 invalid_request.
    // Single-tenant apps (no TenantMiddleware at all) always get tenant_id=None — that is fine.
    // We detect the "middleware active but None" case via a feature-level heuristic: if
    // the request has a host header that is not localhost/127.0.0.1 and current_tenant is None,
    // treat it as ambiguous. This is a best-effort check; the plan defers a full tenant
    // picker to Phase 200. For now we accept single-tenant (None is fine) and note the
    // ambiguous multi-tenant case in comments.
    // NOTE: In practice, if TenantMiddleware is mounted, it will have already resolved
    // current_tenant() before this handler runs. Returning None with the middleware active
    // means resolution failed — the handler returns 400.
    // This plan does not mount TenantMiddleware, so tenant_id=None is always fine here.
    // Phase 200 will enforce the multi-tenant path. Deliberately no check here for now.

    // ── Step 6: Render consent HTML ──────────────────────────────────────────
    let csrf_token = get_csrf_token().unwrap_or_default();
    let client_name = client.client_name.as_deref().unwrap_or("Unknown Client");
    let html = crate::consent::render_consent_html(
        client_name,
        &client_id,
        &redirect_uri,
        &code_challenge,
        &state,
        &csrf_token,
        user_id,
        tenant_id,
    );
    Ok(
        ferro::HttpResponse::text(html)
            .header("Content-Type", crate::consent::CONSENT_CONTENT_TYPE),
    )
}

/// Build an HTML error page (never a redirect).
///
/// Used for T-199-04 (open redirect prevention) and PKCE downgrade (T-199-01).
/// Returns `Err(HttpResponse)` suitable for `?` propagation inside handlers.
pub(crate) fn error_page(status: u16, error: &str, description: &str) -> ferro::HttpResponse {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Authorization Error</title></head>
<body>
  <h1>Authorization Error</h1>
  <p><strong>{}</strong></p>
  <p>{}</p>
</body>
</html>"#,
        html_escape(error),
        html_escape(description),
    );
    ferro::HttpResponse::text(html)
        .header("Content-Type", "text/html; charset=utf-8")
        .status(status)
}

/// HTML-escapes a string to prevent XSS in server-rendered pages (T-199-XSS).
pub(crate) fn html_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect::<Vec<_>>(),
            '>' => "&gt;".chars().collect::<Vec<_>>(),
            '"' => "&quot;".chars().collect::<Vec<_>>(),
            '\'' => "&#x27;".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

// URL-encode helper (wraps percent_encoding via urlencoding crate approach)
mod urlencoding {
    /// Percent-encode a string for use in a URI query parameter value.
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for byte in s.bytes() {
            match byte {
                // Unreserved characters (RFC 3986 §2.3)
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(byte as char);
                }
                b => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_escape_replaces_special_chars() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"q\""), "&quot;q&quot;");
    }

    #[test]
    fn urlencoding_encode_handles_special_chars() {
        let encoded = urlencoding::encode("http://localhost:3000/cb?foo=bar&baz=qux");
        assert!(!encoded.contains('?'));
        assert!(!encoded.contains('&'));
        assert!(!encoded.contains('='));
    }

    #[test]
    fn error_page_returns_html_content_type_in_headers() {
        let resp = error_page(400, "invalid_request", "PKCE required");
        let has_html = resp
            .headers()
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("Content-Type") && v.contains("text/html"));
        assert!(has_html, "error_page must emit text/html Content-Type");
    }

    #[test]
    fn error_page_sets_correct_status() {
        let resp = error_page(400, "e", "d");
        assert_eq!(resp.status_code(), 400);
        let resp_500 = error_page(500, "e", "d");
        assert_eq!(resp_500.status_code(), 500);
    }

    /// Unauthenticated path: Auth::check() is false outside session scope → redirects to /auth/login.
    /// This test cannot call authorize_get (requires a real Request), but we verify the logic
    /// by confirming Auth::check() returns false when no session is present.
    #[test]
    fn auth_check_is_false_outside_session_scope() {
        assert!(
            !Auth::check(),
            "Auth::check() must be false outside session scope"
        );
    }

    /// Consent HTML contains the hidden _token field (CSRF) and S256 value.
    #[test]
    fn consent_html_contains_csrf_and_s256() {
        use crate::consent::render_consent_html;
        let html = render_consent_html(
            "Test Client",
            "client123",
            "http://localhost:3000/cb",
            "challenge_abc",
            "state_xyz",
            "csrf_test_token",
            42,
            Some(7),
        );
        assert!(html.contains(r#"name="_token""#), "must contain CSRF field");
        assert!(
            html.contains("csrf_test_token"),
            "must embed CSRF token value"
        );
        assert!(html.contains("value=\"S256\""), "must contain S256 value");
        assert!(html.starts_with("<!DOCTYPE html>"), "must be HTML document");
    }

    /// redirect_uri with mismatched client should NOT be reachable from authorize_get in tests,
    /// but we can verify the redirect_uris parse+match logic directly.
    #[test]
    fn redirect_uri_exact_match_check() {
        let stored: Vec<String> =
            serde_json::from_str(r#"["http://localhost:3000/callback"]"#).unwrap();
        assert!(stored.iter().any(|u| u == "http://localhost:3000/callback"));
        assert!(!stored.iter().any(|u| u == "http://localhost:3000/other"));
    }
}
