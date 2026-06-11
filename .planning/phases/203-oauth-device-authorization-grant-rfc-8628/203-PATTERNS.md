# Phase 203: OAuth Device Authorization Grant (RFC 8628) - Pattern Map

**Mapped:** 2026-06-11
**Files analyzed:** 5 (1 new + 4 modified)
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp-oauth/src/device.rs` | handler + store type | request-response + CRUD | `ferro-mcp-oauth/src/consent.rs` + `store.rs` + `authorize.rs` | exact (multi-analog) |
| `ferro-mcp-oauth/src/token.rs` | handler | request-response | same file (existing `authorization_code` arm) | exact |
| `ferro-mcp-oauth/src/discovery.rs` | handler | request-response | same file (existing `authorization_server_metadata`) | exact |
| `ferro-mcp-oauth/src/lib.rs` | re-export | — | same file (existing `pub mod handlers` block) | exact |
| `app/src/routes.rs` | route config | request-response | same file (existing `/authorize` group + public `/token`) | exact |

---

## Pattern Assignments

### `ferro-mcp-oauth/src/device.rs` (new — store type + three handlers)

**Primary analogs:** `store.rs` (record shape), `consent.rs` (CSRF + auth capture + HTML render + `Cache::put`), `authorize.rs` (unauth redirect + `store_oauth_return_to`), `pkce.rs` (`generate_auth_code`)

---

#### `DeviceGrant` struct — copy shape from `OAuthCode` in `store.rs` lines 14-28

```rust
// ferro-mcp-oauth/src/store.rs lines 14-28
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCode {
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub user_id: i64,
    pub tenant_id: Option<i64>,
    pub created_at: i64,
}
```

`DeviceGrant` mirrors this shape. Differences from `OAuthCode`:
- No `redirect_uri` / `code_challenge` (device flow has no redirect)
- Add `status: DeviceGrantStatus` (Pending | Approved | Denied)
- `user_id: Option<i64>` (None until approval — vs mandatory in OAuthCode)
- `last_polled_at: Option<i64>` (for slow_down enforcement)
- `normalized_user_code: String` (needed by token handler to forget the usercode pointer key)

Cache key pattern from `consent.rs` line 221:
```rust
// ferro-mcp-oauth/src/consent.rs line 221
Cache::put(
    &format!("mcp:code:{code}"),
    &record,
    Some(Duration::from_secs(60)),
)
.await
```
Device keys follow the same `mcp:{prefix}:{value}` naming. Two keys per grant:
- `mcp:device:{device_code}` → full `DeviceGrant`
- `mcp:usercode:{normalized_user_code}` → `device_code` string (pointer only)

---

#### `generate_device_code` — copy directly from `pkce.rs` lines 16-19

```rust
// ferro-mcp-oauth/src/pkce.rs lines 16-19
pub fn generate_auth_code() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(bytes)
}
```

`generate_device_code` calls `crate::pkce::generate_auth_code()` directly — no reimplementation needed.

---

#### `device_authorization` handler — imports + `find_by_client_id` client validation

Import block from `consent.rs` lines 13-24 (the pattern to copy):
```rust
// ferro-mcp-oauth/src/consent.rs lines 13-24
use ferro::session::get_csrf_token;
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
```

Client validation from `authorize.rs` lines 105-120:
```rust
// ferro-mcp-oauth/src/authorize.rs lines 105-120
let db_conn = ferro::DB::connection()
    .map_err(|e| error_page(500, "server_error", &format!("db connection failed: {e}")))?;
let client = crate::store::find_by_client_id(db_conn.inner(), &client_id)
    .await
    .map_err(|e| error_page(500, "server_error", &format!("db error: {e}")))?;

let client = match client {
    Some(c) => c,
    None => {
        return Err(error_page(
            400,
            "invalid_client",
            "Unknown client_id. Has the client registered via POST /register?",
        ));
    }
};
```

Error response shape from `token.rs` lines 112-117:
```rust
// ferro-mcp-oauth/src/token.rs lines 112-117
fn json_error(status: u16, error: &str, description: &str) -> ferro::HttpResponse {
    ferro::HttpResponse::json(json!({
        "error": error,
        "error_description": description,
    }))
    .status(status)
}
```

The `device_authorization` response body (all 6 RFC §3.2 fields) is JSON via `ferro::HttpResponse::json(json!({...}))`.

---

#### `device_verification_get` handler — unauth redirect (copy from `authorize.rs` lines 88-102)

```rust
// ferro-mcp-oauth/src/authorize.rs lines 88-102
if !Auth::check() {
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
```

For the verification page the return URL is `format!("/device?user_code={}", urlencoding::encode(&user_code_param))`.

After login `take_oauth_return_to` / `oauth_resume_redirect` (from `resume.rs`) brings the user back. The contract is already wired in `auth_controller::verify_magic_link` — no changes needed there.

HTML render pattern from `authorize.rs` lines 166-169:
```rust
// ferro-mcp-oauth/src/authorize.rs lines 166-169
Ok(
    ferro::HttpResponse::text(html)
        .header("Content-Type", crate::consent::CONSENT_CONTENT_TYPE),
)
```

---

#### `device_verification_post` handler — CSRF + tenant capture (copy from `consent.rs`)

CSRF validation from `consent.rs` lines 124-141:
```rust
// ferro-mcp-oauth/src/consent.rs lines 124-141
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
```

User + tenant capture from `consent.rs` lines 191-201:
```rust
// ferro-mcp-oauth/src/consent.rs lines 191-201
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
```

State-transition via `Cache::put` overwrite (same key) from `consent.rs` lines 219-232:
```rust
// ferro-mcp-oauth/src/consent.rs lines 219-232
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
```

Hidden fields required in the confirm+consent form HTML:
- `_token` (CSRF — matches `consent.rs` line 88)
- `action` (value `"approve"` / `"deny"` — same button-value pattern as `consent.rs` lines 95-96)
- `device_code` (opaque; needed by POST handler to update `mcp:device:{device_code}`)

---

### `ferro-mcp-oauth/src/token.rs` (modify — add device-code grant arm)

**Analog:** existing `authorization_code` arm in the same file

Current grant-type gate at lines 50-56:
```rust
// ferro-mcp-oauth/src/token.rs lines 50-56
if form.grant_type != "authorization_code" {
    return Err(json_error(
        400,
        "unsupported_grant_type",
        "grant_type must be 'authorization_code'",
    ));
}
```

Replace with a `match` that dispatches to an inner function per grant type. The `TokenRequest` struct (lines 24-30) currently has all fields required:
```rust
// ferro-mcp-oauth/src/token.rs lines 24-30
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub code_verifier: String,
}
```

All five fields must become `Option<String>` with `#[serde(default)]` so a device-code request (which sends `device_code` instead of `code`/`redirect_uri`/`code_verifier`) does not fail at deserialization before the `grant_type` branch.

JWT mint call from lines 96-101 — the device arm must be call-identical:
```rust
// ferro-mcp-oauth/src/token.rs lines 96-101
let config = OAuthConfig::from_env()
    .map_err(|e| json_error(500, "server_error", &format!("OAuth config error: {e}")))?;

let claims = build_claims(record.user_id, record.tenant_id, &config.app_url, 3600);
let access_token = mint_token(&claims, &config.token_secret)
    .map_err(|e| json_error(500, "server_error", &format!("token mint error: {e}")))?;
```

Single-use get-then-forget discipline from lines 61-64:
```rust
// ferro-mcp-oauth/src/token.rs lines 61-64
let code_key = format!("mcp:code:{}", form.code);
let record: Option<OAuthCode> = Cache::get(&code_key).await.ok().flatten();
// forget() regardless of whether the code was found — idempotent no-op if absent
let _ = Cache::forget(&code_key).await;
```

Device arm differs: forget is deferred until `Approved` state (not at get-time), because pending polls must re-read the same record. Forget BOTH keys on `Approved`:
1. `Cache::forget(&format!("mcp:device:{device_code}"))` — T-199-02
2. `Cache::forget(&format!("mcp:usercode:{}", grant.normalized_user_code))` — cleanup pointer

Token response shape from lines 104-108:
```rust
// ferro-mcp-oauth/src/token.rs lines 104-108
Ok(ferro::HttpResponse::json(json!({
    "access_token": access_token,
    "token_type": "Bearer",
    "expires_in": 3600,
})))
```

Test helper pattern from lines 122-130:
```rust
// ferro-mcp-oauth/src/token.rs lines 122-130
use crate::cache_test_helpers::bootstrap_test_cache;
use ferro::Cache;
// ...
let _cache = bootstrap_test_cache();
```

---

### `ferro-mcp-oauth/src/discovery.rs` (modify — add device fields)

**Analog:** `authorization_server_metadata` function at lines 26-37

```rust
// ferro-mcp-oauth/src/discovery.rs lines 26-37
pub(crate) fn authorization_server_metadata(app_url: &str) -> Value {
    json!({
        "issuer": app_url,
        "authorization_endpoint": format!("{}/authorize", app_url),
        "token_endpoint": format!("{}/token", app_url),
        "registration_endpoint": format!("{}/register", app_url),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
    })
}
```

Add two fields to the `json!({...})` literal:
```rust
"device_authorization_endpoint": format!("{}/device_authorization", app_url),
"grant_types_supported": ["authorization_code", "urn:ietf:params:oauth:grant-type:device_code"],
```

Existing test at lines 80-110 uses index-based assertion `grant_types[0]`. The updated test must use `.iter().any()` — see Pitfall 6 in RESEARCH.md. Pattern for new assertions (mirrors existing test style):
```rust
// ferro-mcp-oauth/src/discovery.rs lines 80-110 (test pattern)
let val = authorization_server_metadata("https://app.example.com");
assert_eq!(val["issuer"].as_str().unwrap(), "https://app.example.com");
// New:
assert_eq!(
    val["device_authorization_endpoint"].as_str().unwrap(),
    "https://app.example.com/device_authorization"
);
let grant_types = val["grant_types_supported"].as_array().unwrap();
assert!(grant_types.iter().any(|v| v.as_str() == Some("authorization_code")));
assert!(grant_types.iter().any(|v| v.as_str() == Some("urn:ietf:params:oauth:grant-type:device_code")));
```

---

### `ferro-mcp-oauth/src/lib.rs` (modify — export device handlers)

**Analog:** existing `pub mod handlers` block at lines 30-36

```rust
// ferro-mcp-oauth/src/lib.rs lines 30-36
pub mod handlers {
    pub use crate::authorize::authorize_get;
    pub use crate::consent::authorize_post;
    pub use crate::discovery::{authorization_server_handler, protected_resource_handler};
    pub use crate::register::register_client;
    pub use crate::token::token_exchange;
}
```

Add one `pub mod device;` to the top-level module list (alongside the other `pub mod` lines 8-20) and three re-exports inside `handlers`:
```rust
pub use crate::device::device_authorization;
pub use crate::device::device_verification_get;
pub use crate::device::device_verification_post;
```

---

### `app/src/routes.rs` (modify — mount device endpoints)

**Analogs:** the `/authorize` session group (lines 71-78) and the public `/register`/`/token` mounts (lines 85-88)

Public mount pattern (lines 85-88):
```rust
// app/src/routes.rs lines 85-88
// Dynamic Client Registration (public)
post!("/register", register_client),

// Token exchange (public, no session needed)
post!("/token", token_exchange),
```

Session + tenant group pattern (lines 71-78):
```rust
// app/src/routes.rs lines 71-78
group!("/", {
    get!("/authorize", authorize_get),
    post!("/authorize", authorize_post),
}).middleware(
    TenantMiddleware::new()
        .resolver(SessionUserTenantResolver::new())
        .on_failure(TenantFailureMode::Allow),
),
```

Import addition needed at lines 4-7:
```rust
// app/src/routes.rs lines 4-7
use ferro_mcp_oauth::handlers::{
    authorization_server_handler, authorize_get, authorize_post, protected_resource_handler,
    register_client, token_exchange,
    // add:
    device_authorization, device_verification_get, device_verification_post,
};
```

New mounts to add (after the `/token` line, before or after the OAuth discovery group — order does not matter for public routes):
```rust
// public — no session, like /register and /token
post!("/device_authorization", device_authorization),

// session + tenant — like the /authorize group (TenantFailureMode::Allow so
// unauthenticated visitors reach the handler for the login-redirect path)
group!("/", {
    get!("/device", device_verification_get),
    post!("/device", device_verification_post),
}).middleware(
    TenantMiddleware::new()
        .resolver(SessionUserTenantResolver::new())
        .on_failure(TenantFailureMode::Allow),
),
```

---

## Shared Patterns

### Cache put/get/forget
**Source:** `ferro-mcp-oauth/src/consent.rs` lines 219-232 (put), `ferro-mcp-oauth/src/token.rs` lines 61-64 (get+forget)
**Apply to:** all three handlers in `device.rs` + the new device-code arm in `token.rs`

```rust
// Put (consent.rs lines 219-232):
Cache::put(&format!("mcp:code:{code}"), &record, Some(Duration::from_secs(60))).await?;

// Get + forget before validation (token.rs lines 61-64):
let record: Option<OAuthCode> = Cache::get(&code_key).await.ok().flatten();
let _ = Cache::forget(&code_key).await;
```

### CSRF constant-time comparison
**Source:** `ferro-mcp-oauth/src/consent.rs` lines 133-141
**Apply to:** `device_verification_post` handler

```rust
// consent.rs lines 133-141
let csrf_ok: bool = form.token.as_bytes().ct_eq(session_csrf.as_bytes()).into();
if !csrf_ok {
    return Err(ferro::HttpResponse::json(json!({
        "error": "invalid_request",
        "error_description": "CSRF token mismatch",
    }))
    .status(400));
}
```

### Auth + tenant capture
**Source:** `ferro-mcp-oauth/src/consent.rs` lines 191-201
**Apply to:** `device_verification_post` (approve path) in `device.rs`

```rust
// consent.rs lines 191-201
let user_id = match Auth::id() {
    Some(id) => id,
    None => { return Err(ferry_error_page(401, "unauthorized", "session expired")); }
};
let tenant_id = current_tenant().map(|t| t.id);
```

### Unauthenticated redirect + resume
**Source:** `ferro-mcp-oauth/src/authorize.rs` lines 88-102
**Apply to:** `device_verification_get` handler in `device.rs`

```rust
// authorize.rs lines 88-102
if !Auth::check() {
    crate::resume::store_oauth_return_to(return_url.clone());
    return Err(ferro::HttpResponse::new()
        .status(302)
        .header("Location", "/auth/login"));
}
```

### HTML error page (never redirect)
**Source:** `ferro-mcp-oauth/src/authorize.rs` lines 176-193
**Apply to:** all error paths in `device.rs` that must not redirect

```rust
// authorize.rs lines 176-193
pub(crate) fn error_page(status: u16, error: &str, description: &str) -> ferro::HttpResponse {
    let html = format!(r#"<!DOCTYPE html>..."#, html_escape(error), html_escape(description));
    ferro::HttpResponse::text(html)
        .header("Content-Type", "text/html; charset=utf-8")
        .status(status)
}
```

### HTML escape + URL encode
**Source:** `ferro-mcp-oauth/src/authorize.rs` lines 196-225
**Apply to:** all HTML renders in `device.rs` (T-199-XSS)

```rust
// authorize.rs lines 196-207
pub(crate) fn html_escape(s: &str) -> String { ... }
// authorize.rs lines 209-225
mod urlencoding { pub fn encode(s: &str) -> String { ... } }
```

Call `crate::authorize::html_escape(value)` for every untrusted string embedded in HTML. For URL parameters use the same `urlencoding::encode` as `authorize.rs` uses internally (it is module-private; copy the logic or make it pub — planner's discretion call).

### JSON error body shape
**Source:** `ferro-mcp-oauth/src/token.rs` lines 112-117
**Apply to:** `device_authorization` handler (public JSON endpoint) + device-code arm in `token.rs`

```rust
// token.rs lines 112-117
fn json_error(status: u16, error: &str, description: &str) -> ferro::HttpResponse {
    ferro::HttpResponse::json(json!({
        "error": error,
        "error_description": description,
    }))
    .status(status)
}
```

### `build_claims` + `mint_token` call shape (load-bearing invariant)
**Source:** `ferro-mcp-oauth/src/token.rs` lines 96-101, `ferro-mcp-oauth/src/jwt.rs` lines 43-61
**Apply to:** device-code `Approved` arm in `token.rs`

```rust
// token.rs lines 96-101  ← auth-code arm; device arm must be identical
let config = OAuthConfig::from_env()
    .map_err(|e| json_error(500, "server_error", &format!("OAuth config error: {e}")))?;
let claims = build_claims(record.user_id, record.tenant_id, &config.app_url, 3600);
let access_token = mint_token(&claims, &config.token_secret)
    .map_err(|e| json_error(500, "server_error", &format!("token mint error: {e}")))?;
```

Device arm replaces `record.user_id` with `grant.user_id.expect("Approved grant must have user_id")` and `record.tenant_id` with `grant.tenant_id`. Every other argument is the same.

### Cache test bootstrap
**Source:** `ferro-mcp-oauth/src/lib.rs` lines 63-67, `ferro-mcp-oauth/src/token.rs` line 183
**Apply to:** all `#[tokio::test]` functions in `device.rs` tests + new `token.rs` device-code tests

```rust
// token.rs line 183 (test body pattern)
let _cache = bootstrap_test_cache();
```

`bootstrap_test_cache()` returns a `TestContainerGuard` that must be held (`let _cache = ...`) for the entire test — drop causes teardown.

### `sanitized_app_url` for URL construction
**Source:** `ferro-mcp-oauth/src/discovery.rs` lines 44-45
**Apply to:** `device_authorization` handler (constructs `verification_uri` + `verification_uri_complete`)

```rust
// discovery.rs lines 44-45
let url = crate::config::sanitized_app_url();
Ok(ferro::HttpResponse::json(protected_resource_metadata(&url)))
```

`verification_uri = format!("{}/device", crate::config::sanitized_app_url())` — no hardcoded host.

---

## No Analog Found

All files have close analogs. No new dependencies are required.

---

## Metadata

**Analog search scope:** `ferro-mcp-oauth/src/`, `app/src/routes.rs`
**Files read:** `store.rs`, `consent.rs`, `token.rs`, `authorize.rs`, `discovery.rs`, `lib.rs`, `pkce.rs`, `jwt.rs`, `resume.rs`, `app/src/routes.rs`
**Pattern extraction date:** 2026-06-11
