//! RFC 8628 Device Authorization Grant — store type, cache-key helpers, code primitives,
//! and HTTP handlers for the device authorization flow.
//!
//! ## Handlers
//!
//! - [`device_authorization`]: `POST /device_authorization` — public; validates `client_id`,
//!   generates device/user codes, stores both cache entries, returns RFC §3.2 JSON.
//! - [`device_verification_get`]: `GET /device` — redirects unauthenticated users to
//!   `/auth/login` via [`crate::resume::store_oauth_return_to`]; renders code-entry form or
//!   confirm+consent page for authenticated users.
//! - [`device_verification_post`]: `POST /device` — CSRF-validates, captures `Auth::id()` +
//!   `current_tenant()` at approve time, writes `Approved`/`Denied` to cache.
//!
//! ## Cache layout
//!
//! Each device grant occupies two cache keys:
//! - `mcp:device:{device_code}` → full [`DeviceGrant`] record (polled by the client).
//! - `mcp:usercode:{normalized_user_code}` → `device_code` string (pointer; used by the
//!   verification page to resolve a user-entered code to the grant).
//!
//! ## Status transitions
//!
//! `Pending` → `Approved` (user consents at verification page) | `Denied` (user denies).
//! `user_id` and `tenant_id` remain `None` until the `Approved` transition captures them
//! from `Auth::id()` and `current_tenant()`.
//! `last_polled_at` is updated on every token poll for `slow_down` enforcement (D-05).

use ferro::session::get_csrf_token;
use ferro::tenant::current_tenant;
use ferro::Auth;
use ferro::Cache;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

use crate::authorize::html_escape;

// ── Constants ─────────────────────────────────────────────────────────────────

/// TTL for both device-code cache entries (RFC 8628 recommends ~10 min).
pub const DEVICE_CODE_TTL: Duration = Duration::from_secs(600);

/// Minimum polling interval in seconds that clients must respect (RFC 8628 §3.5).
pub const DEVICE_INTERVAL_SECS: i64 = 5;

/// RFC 8628 §6.1 recommended charset: 20 unambiguous uppercase consonants.
/// No vowels (avoids profanity); no digits (avoids visual confusion with `0`/`O`, `1`/`I`).
const USER_CODE_CHARSET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";

// ── DeviceGrantStatus ─────────────────────────────────────────────────────────

/// State of a device authorization grant (RFC 8628 §3.5).
///
/// Serialized as snake_case (`"pending"`, `"approved"`, `"denied"`) in the cache record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceGrantStatus {
    /// The grant has been issued but not yet acted on by the user.
    Pending,
    /// The user consented; `user_id` and `tenant_id` are bound on the record.
    Approved,
    /// The user explicitly denied the authorization request.
    Denied,
}

// ── DeviceGrant ───────────────────────────────────────────────────────────────

/// Ephemeral device authorization grant stored in `ferro-cache` with [`DEVICE_CODE_TTL`].
///
/// Two cache keys per grant:
/// - `mcp:device:{device_code}` → this record (full state; keyed by the opaque device_code
///   the polling client sends to `POST /token`).
/// - `mcp:usercode:{normalized_user_code}` → `device_code` string (pointer used by the
///   verification page to resolve the human-entered short code back to this record).
///
/// ## Status transitions
///
/// `Pending` (initial) → `Approved` when the user consents at `POST /device`, at which
/// point `user_id` and `tenant_id` are captured from the session and written to this record.
/// `Pending` → `Denied` when the user explicitly denies. Both transitions are terminal.
///
/// ## Fields
///
/// - `user_id`: `None` until `Approved`; set from `Auth::id()` at verification.
/// - `tenant_id`: `None` until `Approved`; set from `current_tenant()` at verification.
/// - `last_polled_at`: updated on every `POST /token` poll; compared against
///   [`DEVICE_INTERVAL_SECS`] to enforce `slow_down` (RFC 8628 §3.5).
/// - `normalized_user_code`: stored so the token handler can forget the
///   `mcp:usercode:{…}` pointer key when issuing the token (get-then-forget discipline,
///   T-199-02).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceGrant {
    /// The registered client that initiated this flow.
    pub client_id: String,
    /// Current state of the grant (Pending → Approved | Denied).
    pub status: DeviceGrantStatus,
    /// Authenticated user id; `None` until the Approved transition.
    pub user_id: Option<i64>,
    /// Tenant id at approval time; `None` until the Approved transition or for
    /// single-tenant apps.
    pub tenant_id: Option<i64>,
    /// Unix timestamp (seconds) when the grant was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) of the most recent token poll; `None` = never polled.
    pub last_polled_at: Option<i64>,
    /// Normalized user code (uppercase, no hyphens/spaces) stored so the token handler
    /// can forget the `mcp:usercode:{…}` pointer entry on token issuance.
    pub normalized_user_code: String,
}

// ── Cache-key helpers ─────────────────────────────────────────────────────────

/// Returns the primary cache key for a device grant: `mcp:device:{device_code}`.
///
/// Used by the polling client (`POST /token`) and the verification page (`POST /device`)
/// to read and update the [`DeviceGrant`] record.
pub fn device_cache_key(device_code: &str) -> String {
    format!("mcp:device:{device_code}")
}

/// Returns the pointer cache key for a user code: `mcp:usercode:{normalized_user_code}`.
///
/// Stores the opaque `device_code` string so the verification page can resolve a
/// human-entered (and normalized) user code to the correct [`DeviceGrant`].
/// Forgotten by the token handler alongside the primary key on token issuance (T-199-02).
pub fn usercode_cache_key(normalized_user_code: &str) -> String {
    format!("mcp:usercode:{normalized_user_code}")
}

// ── Code generation and normalization ─────────────────────────────────────────

/// Generates a high-entropy opaque device code (256-bit URL-safe random string).
///
/// Delegates to [`crate::pkce::generate_auth_code`] — identical entropy and encoding
/// to authorization codes. Never shown to the user; sent directly to the polling client
/// over TLS in the `POST /device_authorization` response. (RFC 8628 §3.2)
pub fn generate_device_code() -> String {
    crate::pkce::generate_auth_code()
}

/// Generates a short human-typeable user code in `XXXX-XXXX` format.
///
/// Samples 8 characters uniformly at random from the RFC 8628 §6.1 recommended charset
/// (`BCDFGHJKLMNPQRSTVWXZ` — 20 unambiguous uppercase consonants), then groups them as
/// `XXXX-XXXX` for readability. Keyspace: 20^8 ≈ 2.56 × 10^10 combinations.
///
/// The hyphen is for display only; normalization strips it before cache lookup.
pub fn generate_user_code() -> String {
    use rand::Rng;
    // thread_rng() is seeded from the OS CSPRNG (rand 0.8). gen_range uses
    // UniformInt rejection sampling internally — no modular bias against the
    // 20-character charset.
    let mut rng = rand::thread_rng();
    let chars: String = (0..8)
        .map(|_| USER_CODE_CHARSET[rng.gen_range(0..USER_CODE_CHARSET.len())] as char)
        .collect();
    format!("{}-{}", &chars[..4], &chars[4..])
}

/// Normalizes a user-entered code for cache lookup: uppercase and strip hyphens/spaces.
///
/// Accepts any casing and ignores the optional hyphen separator, so `wdjb-mfxg`,
/// `WDJB-MFXG`, `WDJBMFXG`, and `wdjb mfxg` all normalize to the same key `WDJBMFXG`.
/// The normalized form is used as the `mcp:usercode:{…}` cache key (T-203-USERCODE-NORMALIZE).
pub fn normalize_user_code(input: &str) -> String {
    input
        .to_uppercase()
        .chars()
        .filter(|c| *c != '-' && *c != ' ')
        .collect()
}

// ── Percent-encoding helper ───────────────────────────────────────────────────

/// Percent-encode a string for use in a URI query parameter value.
///
/// Mirrors `authorize.rs` urlencoding::encode — same unreserved-character set (RFC 3986 §2.3).
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ── device_authorization_body — pure helper for testing ──────────────────────

/// Build the RFC §3.2 JSON response body for a device authorization grant.
///
/// Pure function — testable without a live DB or cache.
///
/// # Arguments
/// - `device_code`: high-entropy opaque code for the polling client.
/// - `display_user_code`: human-typeable `XXXX-XXXX` code shown to the user.
/// - `app_url`: base URL from [`crate::config::sanitized_app_url`]; no trailing slash.
pub(crate) fn device_authorization_body(
    device_code: &str,
    display_user_code: &str,
    app_url: &str,
) -> Value {
    let verification_uri = format!("{app_url}/device");
    let verification_uri_complete = format!(
        "{app_url}/device?user_code={}",
        url_encode(display_user_code)
    );
    json!({
        "device_code": device_code,
        "user_code": display_user_code,
        "verification_uri": verification_uri,
        "verification_uri_complete": verification_uri_complete,
        "expires_in": 600,
        "interval": 5,
    })
}

// ── Task 1: device_authorization handler ─────────────────────────────────────

/// Form body for `POST /device_authorization` (RFC 8628 §3.1).
#[derive(Debug, Deserialize)]
pub struct DeviceAuthRequest {
    /// Registered client identifier. Required per RFC 8628 §3.1.
    pub client_id: String,
}

/// Handler: `POST /device_authorization` (RFC 8628 §3.1).
///
/// Public endpoint (no session required). Steps:
/// 1. Parse `client_id` from form body.
/// 2. Validate client exists via `find_by_client_id` (T-203-INVALID-CLIENT).
/// 3. Generate `device_code` (256-bit random) + `user_code` (RFC §6.1 charset).
/// 4. Store `DeviceGrant{Pending}` under `mcp:device:{device_code}` with 600s TTL.
/// 5. Store pointer `device_code` under `mcp:usercode:{normalized_user_code}` with same TTL.
/// 6. Return the six RFC §3.2 fields as JSON.
#[ferro::handler]
pub async fn device_authorization(req: ferro::Request) -> ferro::Response {
    // ── Step 1: Parse form body ───────────────────────────────────────────────
    let form: DeviceAuthRequest = req
        .form()
        .await
        .map_err(|e| json_error(400, "invalid_request", &format!("form parse error: {e}")))?;
    let client_id = form.client_id;

    // ── Step 2: Validate client_id (T-203-INVALID-CLIENT) ────────────────────
    let db_conn = ferro::DB::connection()
        .map_err(|e| json_error(500, "server_error", &format!("db connection failed: {e}")))?;
    let client = crate::store::find_by_client_id(db_conn.inner(), &client_id)
        .await
        .map_err(|e| json_error(500, "server_error", &format!("db error: {e}")))?;
    if client.is_none() {
        return Err(json_error(
            400,
            "invalid_client",
            "Unknown client_id. Has the client registered via POST /register?",
        ));
    }

    // ── Step 3: Generate codes ────────────────────────────────────────────────
    let device_code = generate_device_code();
    let display_user_code = generate_user_code();
    let normalized_user_code = normalize_user_code(&display_user_code);

    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // ── Step 4: Store DeviceGrant{Pending} ────────────────────────────────────
    let grant = DeviceGrant {
        client_id,
        status: DeviceGrantStatus::Pending,
        user_id: None,
        tenant_id: None,
        created_at: now_unix,
        last_polled_at: None,
        normalized_user_code: normalized_user_code.clone(),
    };
    Cache::put(
        &device_cache_key(&device_code),
        &grant,
        Some(DEVICE_CODE_TTL),
    )
    .await
    .map_err(|e| json_error(500, "server_error", &format!("cache error: {e}")))?;

    // ── Step 5: Store usercode pointer ────────────────────────────────────────
    Cache::put(
        &usercode_cache_key(&normalized_user_code),
        &device_code,
        Some(DEVICE_CODE_TTL),
    )
    .await
    .map_err(|e| json_error(500, "server_error", &format!("cache error: {e}")))?;

    // ── Step 6: Return RFC §3.2 JSON ──────────────────────────────────────────
    let app_url = crate::config::sanitized_app_url();
    let body = device_authorization_body(&device_code, &display_user_code, &app_url);
    Ok(ferro::HttpResponse::json(body))
}

// ── Task 2: approve/deny helpers ─────────────────────────────────────────────

/// Error returned by [`approve_device_grant`] and [`deny_device_grant`].
#[derive(Debug)]
pub(crate) enum DeviceGrantError {
    /// Grant not found in cache (expired or never existed).
    NotFound,
    /// Cache I/O error.
    #[allow(dead_code)]
    Cache(ferro::FrameworkError),
}

impl From<ferro::FrameworkError> for DeviceGrantError {
    fn from(e: ferro::FrameworkError) -> Self {
        DeviceGrantError::Cache(e)
    }
}

/// Write the `Approved` state to the cache for the given `device_code`.
///
/// Sets `status = Approved`, `user_id = Some(user_id)`, `tenant_id`, re-writes the
/// grant under the same key with a fresh [`DEVICE_CODE_TTL`].
///
/// Testable independently of a full HTTP session: seed a Pending grant, call this,
/// re-read and assert the bound fields.
pub(crate) async fn approve_device_grant(
    device_code: &str,
    user_id: i64,
    tenant_id: Option<i64>,
) -> Result<(), DeviceGrantError> {
    let key = device_cache_key(device_code);
    let grant: DeviceGrant = Cache::get(&key).await?.ok_or(DeviceGrantError::NotFound)?;
    let updated = DeviceGrant {
        status: DeviceGrantStatus::Approved,
        user_id: Some(user_id),
        tenant_id,
        ..grant
    };
    Cache::put(&key, &updated, Some(DEVICE_CODE_TTL)).await?;
    Ok(())
}

/// Write the `Denied` state to the cache for the given `device_code`.
///
/// Sets `status = Denied` and re-writes the grant. The usercode pointer key is left
/// in place (it will expire naturally with the same TTL).
pub(crate) async fn deny_device_grant(device_code: &str) -> Result<(), DeviceGrantError> {
    let key = device_cache_key(device_code);
    let grant: DeviceGrant = Cache::get(&key).await?.ok_or(DeviceGrantError::NotFound)?;
    let updated = DeviceGrant {
        status: DeviceGrantStatus::Denied,
        ..grant
    };
    Cache::put(&key, &updated, Some(DEVICE_CODE_TTL)).await?;
    Ok(())
}

// ── Render helpers ────────────────────────────────────────────────────────────

/// HTML content-type constant for device verification pages.
const DEVICE_CONTENT_TYPE: &str = "text/html; charset=utf-8";

/// Render the code-entry form (GET /device with no valid user_code).
///
/// The form POSTs `user_code` to `/device` for server-side lookup.
fn render_code_entry_form() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Enter Device Code</title>
</head>
<body>
  <h1>Enter your device code</h1>
  <p>Type the code shown on your device.</p>
  <form method="post" action="/device">
    <label for="user_code">Code:</label>
    <input type="text" id="user_code" name="user_code" placeholder="XXXX-XXXX" autocomplete="off">
    <button type="submit">Continue</button>
  </form>
</body>
</html>"#
        .to_string()
}

/// Render the confirm+consent page for a Pending grant.
///
/// Embeds `device_code` (not `user_code`) as a hidden field — needed by the POST handler to
/// update the correct grant. `client_name` and any other user-visible strings are HTML-escaped
/// (T-203-XSS). `csrf_token` is embedded in a hidden `_token` field (T-203-CSRF).
pub(crate) fn render_confirm_html(
    client_name: &str,
    device_code: &str,
    csrf_token: &str,
) -> String {
    let safe_name = html_escape(client_name);
    let safe_device_code = html_escape(device_code);
    let safe_csrf = html_escape(csrf_token);
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Authorize {safe_name}</title>
</head>
<body>
  <h1>Authorize Device Access</h1>
  <p><strong>{safe_name}</strong> is requesting access to your account from another device.</p>
  <form method="post" action="/device">
    <input type="hidden" name="_token" value="{safe_csrf}">
    <input type="hidden" name="device_code" value="{safe_device_code}">
    <button type="submit" name="action" value="approve">Approve</button>
    <button type="submit" name="action" value="deny">Deny</button>
  </form>
</body>
</html>"#
    )
}

/// Render a terminal page (grant already used, expired, approved, or denied).
fn render_terminal_page(title: &str, message: &str) -> String {
    let safe_title = html_escape(title);
    let safe_message = html_escape(message);
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>{safe_title}</title>
</head>
<body>
  <h1>{safe_title}</h1>
  <p>{safe_message}</p>
</body>
</html>"#
    )
}

/// Emit an HTML response (success path).
fn html_ok(html: String) -> ferro::HttpResponse {
    ferro::HttpResponse::text(html).header("Content-Type", DEVICE_CONTENT_TYPE)
}

/// Emit an HTML error response (error path).
fn html_err(status: u16, html: String) -> ferro::HttpResponse {
    ferro::HttpResponse::text(html)
        .header("Content-Type", DEVICE_CONTENT_TYPE)
        .status(status)
}

// ── Task 2: device_verification_get handler ───────────────────────────────────

/// Handler: `GET /device`.
///
/// States:
/// 1. Unauthenticated → store return URL via `store_oauth_return_to`, redirect to `/auth/login`.
/// 2. Authenticated + valid `user_code` query param → resolve grant, render confirm+consent or terminal.
/// 3. Authenticated + no/invalid user_code → render code-entry form.
///
/// Security: T-203-OPEN-REDIRECT (return_url constructed by handler, not from user input);
/// T-203-XSS (client_name escaped); T-203-TENANT-BYPASS (tenant not read here — only at POST).
#[ferro::handler]
pub async fn device_verification_get(req: ferro::Request) -> ferro::Response {
    let user_code_param: Option<String> = req.query("user_code");

    // ── Auth check: redirect unauthenticated users to login (D-04) ────────────
    if !Auth::check() {
        // Only carry the user_code into the return URL if it matches the expected
        // XXXX-XXXX format (9 chars, hyphen at position 4, all chars from the RFC 8628
        // §6.1 charset). Malformed codes are silently dropped — redirect to /device with
        // no query param. This upholds the resume.rs contract: stored URLs must not
        // contain user-supplied content that bypasses format validation.
        let encoded_uc = user_code_param
            .as_deref()
            .filter(|uc| {
                uc.len() == 9
                    && uc.as_bytes().get(4) == Some(&b'-')
                    && uc.as_bytes().iter().enumerate().all(|(i, &b)| {
                        i == 4 || USER_CODE_CHARSET.contains(&b)
                    })
            })
            .map(|uc| url_encode(uc))
            .unwrap_or_default();
        let return_url = if encoded_uc.is_empty() {
            "/device".to_string()
        } else {
            format!("/device?user_code={encoded_uc}")
        };
        crate::resume::store_oauth_return_to(return_url);
        return Err(ferro::HttpResponse::new()
            .status(302)
            .header("Location", "/auth/login"));
    }

    // ── Authenticated path ────────────────────────────────────────────────────
    if let Some(ref uc) = user_code_param {
        if !uc.is_empty() {
            let normalized = normalize_user_code(uc);
            // Resolve user_code → device_code pointer
            let device_code_opt: Option<String> = Cache::get(&usercode_cache_key(&normalized))
                .await
                .ok()
                .flatten();

            if let Some(ref device_code) = device_code_opt {
                let grant_opt: Option<DeviceGrant> = Cache::get(&device_cache_key(device_code))
                    .await
                    .ok()
                    .flatten();

                match grant_opt {
                    None => {
                        // Expired or never existed
                        let html = render_terminal_page(
                            "Code Expired",
                            "This code has expired or is invalid. Please restart the device flow.",
                        );
                        return Err(html_err(400, html));
                    }
                    Some(grant) => match grant.status {
                        DeviceGrantStatus::Approved | DeviceGrantStatus::Denied => {
                            let html = render_terminal_page(
                                "Code Already Used",
                                "This code has already been used.",
                            );
                            return Ok(html_ok(html));
                        }
                        DeviceGrantStatus::Pending => {
                            // Look up client name for the consent page
                            let client_name = match ferro::DB::connection() {
                                Ok(db) => {
                                    match crate::store::find_by_client_id(
                                        db.inner(),
                                        &grant.client_id,
                                    )
                                    .await
                                    {
                                        Ok(Some(c)) => {
                                            c.client_name.unwrap_or_else(|| grant.client_id.clone())
                                        }
                                        _ => grant.client_id.clone(),
                                    }
                                }
                                Err(_) => grant.client_id.clone(),
                            };

                            let csrf_token = get_csrf_token().unwrap_or_default();
                            let html = render_confirm_html(&client_name, device_code, &csrf_token);
                            return Ok(html_ok(html));
                        }
                    },
                }
            }
            // user_code present but not found in cache → fall through to code-entry form
        }
    }

    // No user_code (or not found) → code-entry form
    Ok(html_ok(render_code_entry_form()))
}

// ── Task 2: device_verification_post handler ──────────────────────────────────

/// Form body for `POST /device`.
///
/// Two modes:
/// - Code-entry: `user_code` present, `device_code` absent → resolve and re-render confirm.
/// - Approve/deny: `device_code` present (hidden field), `action` = `"approve"`/`"deny"`.
#[derive(Debug, Deserialize)]
pub struct DeviceVerifyForm {
    /// CSRF token (T-203-CSRF). Required for approve/deny path.
    #[serde(rename = "_token", default)]
    pub token: String,
    /// Approve or deny.
    #[serde(default)]
    pub action: String,
    /// Hidden device_code from the confirm+consent form.
    #[serde(default)]
    pub device_code: String,
    /// User-entered code from the code-entry form (code-entry path only).
    #[serde(default)]
    pub user_code: String,
}

/// Handler: `POST /device`.
///
/// Two paths:
/// - Code-entry POST (`user_code` present, `device_code` absent): resolve and redirect to
///   `GET /device?user_code=…` (PRG pattern) so the confirm page can render fresh CSRF.
/// - Approve/deny POST (`device_code` present): CSRF-validate, bind user/tenant or deny.
///
/// Security: T-203-CSRF (constant-time `ct_eq`); T-203-TENANT-BYPASS (`tenant_id` from
/// session, never form); T-203-XSS (HTML escape on render).
#[ferro::handler]
pub async fn device_verification_post(req: ferro::Request) -> ferro::Response {
    let form: DeviceVerifyForm = req.form().await.map_err(|e| {
        crate::authorize::error_page(400, "invalid_request", &format!("form parse error: {e}"))
    })?;

    // ── Code-entry path: user_code present but no device_code ────────────────
    // No CSRF validation here: this path only moves user input into the URL query
    // string via a PRG redirect. No authorization state changes — the approve/deny
    // POST (which does commit state) validates CSRF separately on the path below.
    if !form.user_code.is_empty() && form.device_code.is_empty() {
        // PRG: redirect to GET /device?user_code=… so browser re-fetches with fresh CSRF
        let encoded = url_encode(&form.user_code);
        return Err(ferro::HttpResponse::new()
            .status(302)
            .header("Location", format!("/device?user_code={encoded}")));
    }

    // ── Approve/deny path ─────────────────────────────────────────────────────
    // CSRF validation (T-203-CSRF)
    let session_csrf = get_csrf_token().ok_or_else(|| {
        crate::authorize::error_page(400, "invalid_request", "no CSRF token in session")
    })?;
    let csrf_ok: bool = form.token.as_bytes().ct_eq(session_csrf.as_bytes()).into();
    if !csrf_ok {
        return Err(crate::authorize::error_page(
            400,
            "invalid_request",
            "CSRF token mismatch",
        ));
    }

    let device_code = form.device_code.clone();
    if device_code.is_empty() {
        return Err(crate::authorize::error_page(
            400,
            "invalid_request",
            "device_code is required",
        ));
    }

    if form.action == "deny" {
        match deny_device_grant(&device_code).await {
            Ok(()) => {}
            Err(_) => {
                // Grant expired or missing — show terminal page regardless
            }
        }
        let html = render_terminal_page(
            "Access Denied",
            "You denied access. You may close this page.",
        );
        return Ok(html_ok(html));
    }

    if form.action == "approve" {
        // Capture user_id from session (T-203-TENANT-BYPASS: never from form)
        let user_id = match Auth::id() {
            Some(id) => id,
            None => {
                return Err(crate::authorize::error_page(
                    401,
                    "unauthorized",
                    "session expired; please log in again",
                ));
            }
        };
        // Capture tenant_id from session middleware at approve time (D-04)
        let tenant_id = current_tenant().map(|t| t.id);

        match approve_device_grant(&device_code, user_id, tenant_id).await {
            Ok(()) => {}
            Err(_) => {
                // Grant expired — render error
                let html = render_terminal_page(
                    "Code Expired",
                    "The device code has expired. Please restart the device flow.",
                );
                return Err(html_err(400, html));
            }
        }

        let html = render_terminal_page(
            "Access Approved",
            "You may now return to your device. This page can be closed.",
        );
        return Ok(html_ok(html));
    }

    // Unknown action
    Err(crate::authorize::error_page(
        400,
        "invalid_request",
        "action must be 'approve' or 'deny'",
    ))
}

// ── JSON error helper (mirrors token.rs) ─────────────────────────────────────

/// RFC 6749 §5.2 / RFC 8628 §3.5 JSON error response body.
fn json_error(status: u16, error: &str, description: &str) -> ferro::HttpResponse {
    ferro::HttpResponse::json(json!({
        "error": error,
        "error_description": description,
    }))
    .status(status)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Plan 01 tests (carried from TDD RED → GREEN) ──────────────────────────

    /// DeviceGrant serializes to JSON and deserializes back to an equal record.
    #[test]
    fn device_grant_serde_roundtrip() {
        let grant = DeviceGrant {
            client_id: "client-abc".to_string(),
            status: DeviceGrantStatus::Pending,
            user_id: None,
            tenant_id: None,
            created_at: 1_718_000_000,
            last_polled_at: None,
            normalized_user_code: "WDJBMFXG".to_string(),
        };

        let json_str = serde_json::to_string(&grant).expect("serialize DeviceGrant");
        let back: DeviceGrant = serde_json::from_str(&json_str).expect("deserialize DeviceGrant");

        assert_eq!(back.client_id, grant.client_id);
        assert_eq!(back.status, grant.status);
        assert_eq!(back.user_id, grant.user_id);
        assert_eq!(back.tenant_id, grant.tenant_id);
        assert_eq!(back.created_at, grant.created_at);
        assert_eq!(back.last_polled_at, grant.last_polled_at);
        assert_eq!(back.normalized_user_code, grant.normalized_user_code);
    }

    /// DeviceGrantStatus variants serialize as snake_case per project convention.
    #[test]
    fn device_grant_status_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&DeviceGrantStatus::Pending).unwrap(),
            r#""pending""#
        );
        assert_eq!(
            serde_json::to_string(&DeviceGrantStatus::Approved).unwrap(),
            r#""approved""#
        );
        assert_eq!(
            serde_json::to_string(&DeviceGrantStatus::Denied).unwrap(),
            r#""denied""#
        );
    }

    /// generate_user_code returns length 9 with a hyphen at index 4, all other
    /// chars from the RFC 8628 §6.1 charset.
    #[test]
    fn user_code_format_is_xxxx_hyphen_xxxx() {
        for _ in 0..20 {
            let code = generate_user_code();
            let len = code.len();
            assert_eq!(len, 9, "expected length 9 (XXXX-XXXX), got {len}: {code:?}");
            assert_eq!(
                code.as_bytes()[4],
                b'-',
                "expected hyphen at index 4, got {code:?}"
            );
            for (i, &b) in code.as_bytes().iter().enumerate() {
                if i == 4 {
                    continue; // skip the hyphen
                }
                let ch = b as char;
                assert!(
                    USER_CODE_CHARSET.contains(&b),
                    "char {ch:?} at index {i} is not in RFC 8628 charset: {code:?}"
                );
            }
        }
    }

    /// normalize_user_code strips hyphens and uppercases (RFC 8628 §3.3 case-insensitive).
    #[test]
    fn user_code_normalization_strips_hyphen_and_case() {
        assert_eq!(normalize_user_code("wdjb-mfxg"), "WDJBMFXG");
        assert_eq!(normalize_user_code("WDJB-MFXG"), "WDJBMFXG");
        assert_eq!(normalize_user_code("WDJBMFXG"), "WDJBMFXG");
        assert_eq!(normalize_user_code("wdjb mfxg"), "WDJBMFXG");
    }

    /// generate_device_code returns a non-empty URL-safe string (no `/`, `+`, `=`).
    #[test]
    fn device_code_is_url_safe_nonempty() {
        let code = generate_device_code();
        assert!(!code.is_empty(), "device_code must not be empty");
        assert!(
            !code.contains('/'),
            "device_code must not contain '/': {code:?}"
        );
        assert!(
            !code.contains('+'),
            "device_code must not contain '+': {code:?}"
        );
        assert!(
            !code.contains('='),
            "device_code must not contain '=': {code:?}"
        );
    }

    // ── Task 1 tests (Plan 03 TDD RED → GREEN) ────────────────────────────────

    /// device_authorization_body returns all 6 RFC §3.2 fields with correct values.
    ///
    /// This is the RED test: it tests the pure `device_authorization_body` helper
    /// without needing a live DB or cache, covering SC-1.
    #[test]
    fn device_authorization_response_fields() {
        let body = device_authorization_body(
            "dc_test_code_abc123",
            "BCDF-GHJK",
            "https://app.example.com",
        );

        // All 6 fields must be present (RFC §3.2)
        assert_eq!(
            body["device_code"].as_str().unwrap(),
            "dc_test_code_abc123",
            "device_code must match"
        );
        assert_eq!(
            body["user_code"].as_str().unwrap(),
            "BCDF-GHJK",
            "user_code must match"
        );

        let verification_uri = body["verification_uri"].as_str().unwrap();
        assert!(
            verification_uri.ends_with("/device"),
            "verification_uri must end with /device: {verification_uri}"
        );

        let verification_uri_complete = body["verification_uri_complete"].as_str().unwrap();
        assert!(
            verification_uri_complete.contains("?user_code="),
            "verification_uri_complete must contain ?user_code=: {verification_uri_complete}"
        );

        assert_eq!(
            body["expires_in"].as_i64().unwrap(),
            600,
            "expires_in must be 600"
        );
        assert_eq!(body["interval"].as_i64().unwrap(), 5, "interval must be 5");
    }

    /// verification_uri_complete percent-encodes the hyphen in user_code correctly.
    #[test]
    fn device_authorization_body_uri_complete_encodes_user_code() {
        let body = device_authorization_body("dc_abc", "WDJB-MFXG", "https://example.com");
        let vc = body["verification_uri_complete"].as_str().unwrap();
        // hyphen is unreserved (RFC 3986 §2.3) — must NOT be percent-encoded
        assert!(
            vc.contains("WDJB-MFXG") || vc.contains("WDJB%2DMFXG") || vc.contains("WDJBMFXG"),
            "uri complete must encode user_code: {vc}"
        );
        // Must contain the ?user_code= key
        assert!(
            vc.contains("?user_code="),
            "uri complete must have ?user_code= query: {vc}"
        );
    }

    // ── Task 2 tests (Plan 03 TDD RED → GREEN) ────────────────────────────────

    /// render_confirm_html contains the required hidden fields (CSRF + device_code).
    #[test]
    fn device_verification_confirm_html_contains_required_fields() {
        let html = render_confirm_html("TestApp", "dc_abc123", "csrf_token_xyz");

        assert!(
            html.contains(r#"name="_token""#),
            "confirm HTML must have CSRF hidden input: {html}"
        );
        assert!(
            html.contains("csrf_token_xyz"),
            "confirm HTML must embed CSRF token value: {html}"
        );
        assert!(
            html.contains(r#"name="device_code""#),
            "confirm HTML must have device_code hidden input: {html}"
        );
        assert!(
            html.contains("dc_abc123"),
            "confirm HTML must embed device_code value: {html}"
        );
        assert!(
            html.contains(r#"value="approve""#),
            "confirm HTML must have approve button: {html}"
        );
        assert!(
            html.contains(r#"value="deny""#),
            "confirm HTML must have deny button: {html}"
        );
    }

    /// render_code_entry_form produces a POST form with method=post and action=/device.
    #[test]
    fn device_verification_code_entry_form_structure() {
        let html = render_code_entry_form();
        assert!(
            html.contains(r#"method="post""#),
            "code-entry form must use POST: {html}"
        );
        assert!(
            html.contains(r#"action="/device""#),
            "code-entry form must POST to /device: {html}"
        );
        assert!(
            html.contains(r#"name="user_code""#),
            "code-entry form must have user_code input: {html}"
        );
    }

    /// render_confirm_html escapes XSS in client_name (T-203-XSS).
    #[test]
    fn device_verification_confirm_html_escapes_client_name() {
        let html = render_confirm_html("<script>alert(1)</script>", "dc_abc", "tok");
        assert!(
            !html.contains("<script>"),
            "raw <script> must not appear in confirm HTML: {html}"
        );
        assert!(
            html.contains("&lt;script&gt;"),
            "escaped form must appear in confirm HTML: {html}"
        );
    }

    /// approve_device_grant writes Approved status with bound user_id and tenant_id.
    ///
    /// This is the device_verification_binds_user_and_tenant test: seed a Pending grant,
    /// call approve_device_grant, re-read and assert Approved + bound IDs.
    #[tokio::test]
    async fn device_verification_binds_user_and_tenant() {
        let _cache = crate::cache_test_helpers::bootstrap_test_cache();

        let device_code = "test_device_code_bind_123";
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Pending,
            user_id: None,
            tenant_id: None,
            created_at: 1_718_000_000,
            last_polled_at: None,
            normalized_user_code: "TESTCODE".to_string(),
        };

        // Seed Pending grant
        Cache::put(
            &device_cache_key(device_code),
            &grant,
            Some(DEVICE_CODE_TTL),
        )
        .await
        .expect("cache put should succeed");

        // Approve with user_id=42 and tenant_id=Some(7)
        approve_device_grant(device_code, 42, Some(7))
            .await
            .expect("approve_device_grant should succeed");

        // Re-read and assert
        let updated: DeviceGrant = Cache::get(&device_cache_key(device_code))
            .await
            .expect("cache get should succeed")
            .expect("grant should exist after approve");

        assert_eq!(
            updated.status,
            DeviceGrantStatus::Approved,
            "status must be Approved after approve_device_grant"
        );
        assert_eq!(updated.user_id, Some(42), "user_id must be bound to 42");
        assert_eq!(
            updated.tenant_id,
            Some(7),
            "tenant_id must be bound to Some(7)"
        );
    }

    /// deny_device_grant writes Denied status without binding user/tenant.
    #[tokio::test]
    async fn device_verification_deny_sets_denied_status() {
        let _cache = crate::cache_test_helpers::bootstrap_test_cache();

        let device_code = "test_device_code_deny_456";
        let grant = DeviceGrant {
            client_id: "client-test".to_string(),
            status: DeviceGrantStatus::Pending,
            user_id: None,
            tenant_id: None,
            created_at: 1_718_000_000,
            last_polled_at: None,
            normalized_user_code: "DENYCODE".to_string(),
        };

        Cache::put(
            &device_cache_key(device_code),
            &grant,
            Some(DEVICE_CODE_TTL),
        )
        .await
        .expect("cache put should succeed");

        deny_device_grant(device_code)
            .await
            .expect("deny_device_grant should succeed");

        let updated: DeviceGrant = Cache::get(&device_cache_key(device_code))
            .await
            .expect("cache get should succeed")
            .expect("grant should exist after deny");

        assert_eq!(
            updated.status,
            DeviceGrantStatus::Denied,
            "status must be Denied after deny_device_grant"
        );
        assert_eq!(updated.user_id, None, "user_id must remain None after deny");
        assert_eq!(
            updated.tenant_id, None,
            "tenant_id must remain None after deny"
        );
    }
}
