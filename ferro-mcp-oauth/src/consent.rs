//! Consent page render + submit (Plan 04).
//!
//! Implements `POST /authorize`: validates CSRF token, issues auth code on
//! approve, redirects with error on deny.
//!
//! Security properties:
//! - T-199-10 / T-199-12 (CSRF): `_token` hidden field validated against session
//!   `get_csrf_token()` via constant-time `ct_eq` before processing.
//! - T-199-03 (code TTL): auth code stored with `Some(Duration::from_secs(60))`.
//! - T-199-16 (code substitution): client_id + redirect_uri re-validated at approve time.
//! - T-199-XSS: `client_name` HTML-escaped before embedding in page.

use ferro::session::{get_csrf_token, session_mut};
use ferro::tenant::current_tenant;
use ferro::Auth;
use ferro::Cache;
use serde::Deserialize;
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

use crate::authorize::html_escape;
use crate::pkce::generate_auth_code;
use crate::store::OAuthCode;

/// Content-Type for HTML consent responses.
///
/// `HttpResponse::text()` defaults to `text/plain`; callers must override with this value.
pub const CONSENT_CONTENT_TYPE: &str = "text/html; charset=utf-8";

/// Consent form fields submitted by the browser.
#[derive(Debug, Deserialize)]
pub struct ConsentForm {
    /// CSRF token (T-199-10): must match the session's `csrf_token`.
    #[serde(rename = "_token")]
    pub token: String,
    /// `"approve"` or `"deny"`.
    pub action: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    #[serde(default)]
    pub state: String,
    pub response_type: String,
}

/// Render the consent HTML page.
///
/// Embeds all OAuth params as hidden fields so the POST can reconstruct them.
/// `client_name` is HTML-escaped (T-199-XSS).
///
/// # Arguments
/// - `client_name`: human-readable name from the registered client.
/// - `client_id`, `redirect_uri`, `code_challenge`, `state`: echoed as hidden fields.
/// - `csrf_token`: embedded in `<input name="_token">` (T-199-10).
/// - `user_id`, `tenant_id`: captured at GET time (not re-embedded in the form;
///   re-read from session at POST time for security).
#[allow(clippy::too_many_arguments)]
pub fn render_consent_html(
    client_name: &str,
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
    csrf_token: &str,
    _user_id: i64,
    _tenant_id: Option<i64>,
) -> String {
    let safe_name = html_escape(client_name);
    let safe_client_id = html_escape(client_id);
    let safe_redirect_uri = html_escape(redirect_uri);
    let safe_code_challenge = html_escape(code_challenge);
    let safe_state = html_escape(state);
    let safe_csrf = html_escape(csrf_token);

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Authorize {safe_name}</title>
</head>
<body>
  <h1>Authorize Access</h1>
  <p><strong>{safe_name}</strong> is requesting access to your account.</p>
  <form method="POST" action="/authorize">
    <input type="hidden" name="_token" value="{safe_csrf}">
    <input type="hidden" name="client_id" value="{safe_client_id}">
    <input type="hidden" name="redirect_uri" value="{safe_redirect_uri}">
    <input type="hidden" name="code_challenge" value="{safe_code_challenge}">
    <input type="hidden" name="code_challenge_method" value="S256">
    <input type="hidden" name="state" value="{safe_state}">
    <input type="hidden" name="response_type" value="code">
    <button type="submit" name="action" value="approve">Approve</button>
    <button type="submit" name="action" value="deny">Deny</button>
  </form>
</body>
</html>"#
    )
}

/// Handler: `POST /authorize` (consent submit).
///
/// Steps:
/// 1. Parse form body.
/// 2. CSRF validation (T-199-10, T-199-12): constant-time compare `_token` vs session.
/// 3. Deny path: redirect with `error=access_denied`.
/// 4. Approve path:
///    a. Re-validate `code_challenge_method == "S256"` (T-199-01).
///    b. Re-validate client_id + redirect_uri exact-match (T-199-16).
///    c. Mint single-use auth code, store in cache with 60s TTL (T-199-03).
///    d. 302 redirect with `code` + `state`.
#[ferro::handler]
pub async fn authorize_post(req: ferro::Request) -> ferro::Response {
    let form: ConsentForm = req.form().await.map_err(|e| {
        ferro::HttpResponse::json(json!({
            "error": "invalid_request",
            "error_description": format!("form parse error: {}", e),
        }))
        .status(400)
    })?;

    // ── Step 2: CSRF validation (T-199-10, T-199-12) ─────────────────────────
    let session_csrf = get_csrf_token().ok_or_else(|| {
        ferro::HttpResponse::json(json!({
            "error": "invalid_request",
            "error_description": "no CSRF token in session",
        }))
        .status(400)
    })?;

    // Constant-time compare (T-199-12 timing oracle prevention)
    let csrf_ok: bool = form.token.as_bytes().ct_eq(session_csrf.as_bytes()).into();
    if !csrf_ok {
        return Err(ferro::HttpResponse::json(json!({
            "error": "invalid_request",
            "error_description": "CSRF token mismatch",
        }))
        .status(400));
    }

    // ── Step 3: Deny path ─────────────────────────────────────────────────────
    if form.action == "deny" {
        let location = if form.state.is_empty() {
            format!("{}?error=access_denied", form.redirect_uri)
        } else {
            format!(
                "{}?error=access_denied&state={}",
                form.redirect_uri, form.state
            )
        };
        return Err(ferro::HttpResponse::new()
            .status(302)
            .header("Location", location));
    }

    // ── Step 4a: Re-validate PKCE method (T-199-01) ──────────────────────────
    if form.code_challenge_method != "S256" {
        return Err(ferry_error_page(
            400,
            "invalid_request",
            "code_challenge_method must be 'S256'",
        ));
    }

    // ── Step 4b: Re-validate client_id + redirect_uri (T-199-16) ─────────────
    let db_conn = ferro::DB::connection()
        .map_err(|e| ferry_error_page(500, "server_error", &format!("db error: {e}")))?;
    let client = crate::store::find_by_client_id(db_conn.inner(), &form.client_id)
        .await
        .map_err(|e| ferry_error_page(500, "server_error", &format!("db error: {e}")))?;

    let client = match client {
        Some(c) => c,
        None => {
            return Err(ferry_error_page(400, "invalid_client", "Unknown client_id"));
        }
    };

    let stored_uris: Vec<String> = serde_json::from_str(&client.redirect_uris).unwrap_or_default();
    if !stored_uris.iter().any(|u| u == &form.redirect_uri) {
        return Err(ferry_error_page(
            400,
            "invalid_request",
            "redirect_uri mismatch",
        ));
    }

    // ── Step 4c: Capture user/tenant + mint code ──────────────────────────────
    let user_id = match Auth::id() {
        Some(id) => id,
        None => {
            return Err(ferry_error_page(
                401,
                "unauthorized",
                "session expired; please log in again",
            ));
        }
    };
    let tenant_id = current_tenant().map(|t| t.id);

    let code = generate_auth_code();

    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let record = OAuthCode {
        client_id: form.client_id.clone(),
        redirect_uri: form.redirect_uri.clone(),
        code_challenge: form.code_challenge.clone(),
        user_id,
        tenant_id,
        created_at: now_unix,
    };

    // Store with 60s TTL (T-199-03, D-03)
    Cache::put(
        &format!("mcp:code:{code}"),
        &record,
        Some(Duration::from_secs(60)),
    )
    .await
    .map_err(|e| {
        ferro::HttpResponse::json(json!({
            "error": "server_error",
            "error_description": format!("cache error: {}", e),
        }))
        .status(500)
    })?;

    // Clear the oauth_return_to session key now that we've reached the consent step
    session_mut(|s| {
        s.forget("oauth_return_to");
    });

    // ── Step 4d: 302 redirect with code + state ───────────────────────────────
    let location = if form.state.is_empty() {
        format!("{}?code={}", form.redirect_uri, code)
    } else {
        format!("{}?code={}&state={}", form.redirect_uri, code, form.state)
    };

    Err(ferro::HttpResponse::new()
        .status(302)
        .header("Location", location))
}

/// Build an HTML error response for the consent post handler.
fn ferry_error_page(status: u16, error: &str, description: &str) -> ferro::HttpResponse {
    crate::authorize::error_page(status, error, description)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_consent_html_contains_csrf_field() {
        let html = render_consent_html(
            "MyApp",
            "client-abc",
            "http://localhost:3000/cb",
            "chall_xyz",
            "state123",
            "my_csrf_token",
            42,
            Some(7),
        );
        assert!(
            html.contains(r#"name="_token""#),
            "must have CSRF hidden input: {html}"
        );
        assert!(
            html.contains("my_csrf_token"),
            "must embed CSRF token value: {html}"
        );
    }

    #[test]
    fn render_consent_html_contains_s256_and_code_challenge_method() {
        let html = render_consent_html(
            "Client",
            "cid",
            "http://localhost/cb",
            "challenge",
            "st",
            "tok",
            1,
            None,
        );
        assert!(
            html.contains("value=\"S256\""),
            "code_challenge_method must be S256: {html}"
        );
    }

    #[test]
    fn render_consent_html_escapes_client_name_xss() {
        let html = render_consent_html(
            "<script>alert(1)</script>",
            "cid",
            "http://localhost/cb",
            "challenge",
            "st",
            "tok",
            1,
            None,
        );
        assert!(
            !html.contains("<script>"),
            "raw <script> must not appear in output: {html}"
        );
        assert!(
            html.contains("&lt;script&gt;"),
            "escaped form must appear: {html}"
        );
    }

    #[test]
    fn render_consent_html_contains_text_html_doctype() {
        let html = render_consent_html(
            "App",
            "cid",
            "http://localhost/cb",
            "chall",
            "state",
            "tok",
            1,
            None,
        );
        assert!(html.starts_with("<!DOCTYPE html>"), "must be HTML document");
    }
}
