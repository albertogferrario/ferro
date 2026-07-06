# Phase 202: Login-resume contract + magic-link sample app - Pattern Map

**Mapped:** 2026-06-11
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-mcp-oauth/src/resume.rs` | helper | request-response | `ferro-mcp-oauth/src/authorize.rs` lines 88-103 | exact (same session API, same 302 pattern) |
| `ferro-mcp-oauth/src/authorize.rs` | handler | request-response | self (modify Step 3, lines 88-103) | self-refactor |
| `ferro-mcp-oauth/src/consent.rs` | handler | request-response | self (modify line 235-237) | self-refactor |
| `ferro-mcp-oauth/src/lib.rs` | config | — | self (add `pub mod resume` + `pub use`) | self-refactor |
| `app/src/controllers/auth_controller.rs` | handler | request-response | `ferro-mcp-oauth/src/token.rs` (Cache get/forget pattern) + self (login_form lines 175-195) | exact (token single-use) + self-refactor |
| `app/src/routes.rs` | route-registration | request-response | self lines 38-42 (guest group) | self-refactor |
| `app/src/views/login.json` | view | request-response | self (existing JSON-UI v2 structure) | self-refactor |
| `app/src/views/login_confirm.json` | view | request-response | `app/src/views/login.json` | exact |
| `app/Cargo.toml` | config | — | `ferro-mcp-oauth/Cargo.toml` (rand + base64 declarations) | role-match |
| `app/src/tests/oauth_magic_link_flow.rs` | test | request-response | `ferro-mcp-oauth/src/token.rs` tests (bootstrap_test_cache + Cache put/get/forget) | exact |

---

## Pattern Assignments

### `ferro-mcp-oauth/src/resume.rs` (helper, request-response)

**Analog:** `ferro-mcp-oauth/src/authorize.rs` lines 88-103 and `app/src/controllers/auth_controller.rs` lines 141-184

**Imports pattern** — copy from `authorize.rs` lines 14, `auth_controller.rs` lines 4:
```rust
use ferro::session::{session, session_mut};
use ferro::HttpResponse;
use ferro::Response;
```

**Session write pattern** (analog: `authorize.rs` lines 98-100):
```rust
session_mut(|s| {
    s.put("oauth_return_to", return_url.clone());
});
```

**Session read + forget pattern** (analog: `auth_controller.rs` lines 141-144):
```rust
let return_to: Option<String> = session().and_then(|s| s.get("oauth_return_to"));
session_mut(|s| {
    s.forget("oauth_return_to");
});
```

**302 redirect builder pattern** (analog: `authorize.rs` lines 101-103 and `auth_controller.rs` lines 183-184):
```rust
// Err(HttpResponse) is the redirect path in ferro::Response = Result<HttpResponse, HttpResponse>
return Err(ferro::HttpResponse::new()
    .status(302)
    .header("Location", "/auth/login"));
// — or for Ok path (non-error redirect used in login_form) —
return Ok(HttpResponse::new().status(302).header("Location", dest));
```

**Important:** `oauth_resume_redirect` should return `ferro::Response` (`Result<HttpResponse, HttpResponse>`). The login handler uses `return oauth_resume_redirect("/")` directly — NOT `oauth_resume_redirect("/")?`. The `Err(redirect)` path mirrors `authorize.rs` line 101-103; the `Ok(redirect)` path mirrors `auth_controller.rs` line 183-184. Choose `Ok` for post-login so the caller's `?` propagation behaves correctly — `login_form` line 184 uses `Ok(...)` for the success redirect.

**Unit test pattern** — inline `#[cfg(test)]` in `resume.rs`. Mirror the style from `authorize.rs` lines 229-308: simple `#[test]` (not `#[tokio::test]`) where no async needed, `#[tokio::test]` where session scope requires it.

---

### `ferro-mcp-oauth/src/authorize.rs` (modify Step 3, lines 88-103)

**Change:** Replace the inline session write and 302 return with a call to `store_oauth_return_to(return_url.clone())`.

**Before** (lines 98-103):
```rust
session_mut(|s| {
    s.put("oauth_return_to", return_url.clone());
});
return Err(ferro::HttpResponse::new()
    .status(302)
    .header("Location", "/auth/login"));
```

**After:**
```rust
crate::resume::store_oauth_return_to(return_url.clone());
return Err(ferro::HttpResponse::new()
    .status(302)
    .header("Location", "/auth/login"));
```

The 302 to `/auth/login` stays in `authorize.rs` — only the session write moves to the helper. The const `"oauth_return_to"` lives in `resume.rs` as a private (or pub) constant; `authorize.rs` no longer references the string.

---

### `ferro-mcp-oauth/src/consent.rs` (modify line 235-237)

**Change:** Replace the inline `s.forget("oauth_return_to")` with a call to `crate::resume::take_oauth_return_to()` (discarding the return value), so the key string has one owner.

**Before** (lines 234-237):
```rust
session_mut(|s| {
    s.forget("oauth_return_to");
});
```

**After:**
```rust
let _ = crate::resume::take_oauth_return_to();
```

No other changes to `consent.rs`.

---

### `ferro-mcp-oauth/src/lib.rs` (add module + re-exports)

**Analog:** existing module declarations lines 8-19 and `pub use` lines 21-25, `pub mod handlers` lines 28-34.

**Pattern to copy** (lines 8-34):
```rust
// Add alongside existing mod declarations:
pub mod resume;

// Add alongside existing pub use lines:
pub use resume::{oauth_resume_redirect, store_oauth_return_to, take_oauth_return_to};
```

The naming convention is: `pub mod foo;` then `pub use foo::bar;` at the top-level for the crate's public surface. `resume` helpers are user-facing (any login handler calls them), so they belong in the top-level `pub use` block alongside `OAuthConfig`, `OAuthError`, etc.

---

### `app/src/controllers/auth_controller.rs` (convert + add verify handler)

**Analog (token single-use pattern):** `ferro-mcp-oauth/src/token.rs` lines 58-72

The exact security invariant — `Cache::get` then `Cache::forget` BEFORE any validation:
```rust
// Source: ferro-mcp-oauth/src/token.rs lines 61-72
let code_key = format!("mcp:code:{}", form.code);
let record: Option<OAuthCode> = Cache::get(&code_key).await.ok().flatten();
// forget() regardless of whether the code was found — idempotent no-op if absent
let _ = Cache::forget(&code_key).await;

let record = record.ok_or_else(|| {
    json_error(400, "invalid_grant", "authorization code expired or already used")
})?;
```

Mirror this exactly for magic-link verify:
```rust
let key = format!("magic_link:{token}");
let user_id: Option<i64> = Cache::get(&key).await.ok().flatten();
let _ = Cache::forget(&key).await;  // single-use: forget before validation

let user_id = user_id.ok_or_else(|| /* re-render login with error */)?;
```

**Analog (Auth::login call):** `app/src/controllers/auth_controller.rs` line 79 (`register` handler):
```rust
Auth::login(user.id as i64);
```

**Analog (resume redirect):** `auth_controller.rs` lines 179-184 (`login_form`):
```rust
let return_to: Option<String> = session().and_then(|s| s.get("oauth_return_to"));
session_mut(|s| {
    s.forget("oauth_return_to");
});
let dest = return_to.unwrap_or_else(|| "/".to_string());
return Ok(HttpResponse::new().status(302).header("Location", dest));
```

After introducing the resume helper, this becomes:
```rust
return ferro_mcp_oauth::oauth_resume_redirect("/");
```

**Analog (JsonUi::render_file with error data):** `auth_controller.rs` lines 187-194 (existing `login_form` failure path):
```rust
JsonUi::render_file(
    "src/views/login.json",
    json!({
        "email": input.email,
        "error": "These credentials do not match our records.",
    }),
)
.map(|resp| resp.status(422))
```

Mirror for verify handler failure:
```rust
JsonUi::render_file(
    "src/views/login.json",
    json!({ "error": "This login link has expired or already been used." }),
)
.map(|resp| resp.status(422))
```

**Analog (token generation):** `ferro-mcp-oauth/src/pkce.rs` lines 16-18:
```rust
pub fn generate_auth_code() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(bytes)  // 43 chars, URL-safe, 256-bit entropy
}
```

Replicate this locally in `auth_controller.rs` or call it via `ferro_mcp_oauth::pkce::generate_auth_code()` if re-exported.

**Imports to add** (derive from existing imports at lines 1-12 plus new deps):
```rust
use ferro::Cache;
use ferro::config::env::Environment;
use ferro_mcp_oauth::{oauth_resume_redirect, store_oauth_return_to};
use std::time::Duration;
```

**Structs to change:** `LoginInput` currently has `email` and `password` fields (line 25-28). Replace with a single-field struct:
```rust
#[derive(Deserialize)]
struct RequestLinkInput {
    email: String,
}
```

**Functions to DELETE** (D-05):
- `async fn login_form(req: Request) -> Response` — lines 175-195
- `async fn authenticate(email: &str, password: &str) -> Result<bool, HttpResponse>` — lines 198-213

**Functions to CONVERT:**
- `login` (lines 111-170) — becomes the request-link handler: accepts `email`, issues token, renders confirmation

**Functions to ADD:**
- `verify_magic_link` — `GET /auth/verify?token=...` handler

**Test to update** (`login_view_is_valid_and_posts_to_login`, lines 249-264): remove `assert_eq!(v["elements"]["password"]["props"]["field"], "password")` and `assert_eq!(v["elements"]["password"]["props"]["input_type"], "password")`. Add assertion that `password` key does NOT exist.

---

### `app/src/routes.rs` (add GET /auth/verify to guest group)

**Analog:** existing guest group lines 38-42:
```rust
group!("/auth", {
    get!("/login", controllers::auth_controller::login_page).name("auth.login.page"),
    post!("/register", controllers::auth_controller::register).name("auth.register"),
    post!("/login", controllers::auth_controller::login).name("auth.login"),
}).middleware(GuestMiddleware::redirect_to("/")),
```

**After (add one line):**
```rust
group!("/auth", {
    get!("/login", controllers::auth_controller::login_page).name("auth.login.page"),
    get!("/verify", controllers::auth_controller::verify_magic_link).name("auth.verify"),
    post!("/register", controllers::auth_controller::register).name("auth.register"),
    post!("/login", controllers::auth_controller::login).name("auth.login"),
}).middleware(GuestMiddleware::redirect_to("/")),
```

`GuestMiddleware::redirect_to("/")` on this group is intentional for `GET /auth/verify`: an already-authenticated user clicking an old magic link gets redirected to `/`, which is the correct behavior.

---

### `app/src/views/login.json` (replace with email-only form)

**Analog:** the file itself (current structure lines 1-37) — keep all structural keys, remove `password` element, update children list and button label.

**Current structure to mirror:**
```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Sign in",
  "layout": "auth",
  "root": "card",
  "elements": {
    "card": { "type": "Card", "props": { "title": "...", "description": "...", "variant": "elevated" }, "children": ["form"] },
    "form": { "type": "Form", "props": { "action": { "handler": "/auth/login", "method": "POST" }, "method": "POST", "max_width": "narrow" }, "children": ["email", "submit"] },
    "email": { "type": "Input", "props": { "field": "email", "label": "Email", "input_type": "email", "required": true, "data_path": "/email", "error": { "$data": "/error" } } },
    "submit": { "type": "Button", "props": { "label": "Send login link", "button_type": "submit", "variant": "default" } }
  }
}
```

Key changes from current file:
- `form.children`: `["email", "password", "submit"]` → `["email", "submit"]`
- Remove the `"password"` element entirely
- `submit.props.label`: `"Continue"` → `"Send login link"`

The `data_path: "/email"` and `error: { "$data": "/error" }` on the email input stay — they enable pre-fill on failure and error display from handler data, matching the existing pattern.

---

### `app/src/views/login_confirm.json` (new — confirmation state)

**Analog:** `app/src/views/login.json` (same schema, layout, root, Card/Form shape)

**Pattern to copy and adapt:**
```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Check your email",
  "layout": "auth",
  "root": "card",
  "elements": {
    "card": {
      "type": "Card",
      "props": { "title": "Check your email", "description": "A login link has been sent.", "variant": "elevated" },
      "children": ["message"]
    },
    "message": {
      "type": "Text",
      "props": { "content": { "$data": "/dev_link" } }
    }
  }
}
```

The `dev_link` data path is populated by the handler with the full verify URL when `is_development()`, or with an empty string / absent when not. The handler passes `json!({"dev_link": verify_url})` in dev mode and `json!({})` in production. The view may conditionally show/hide via a `$if` on `dev_link`, or simply display nothing when the value is empty — use the simpler approach given current `$if` support.

Two-file approach (this file + `login.json`) is preferred over a single-file `$if` approach (RESEARCH Pitfall 1).

---

### `app/Cargo.toml` (add rand + base64 deps)

**Analog:** `ferro-mcp-oauth/Cargo.toml` lines 18-19:
```toml
rand = "0.8"
base64 = "0.22"
```

Add these to `app/Cargo.toml` under `[dependencies]`. The app crate currently does NOT declare them (RESEARCH Finding 6 / Assumption A1 — confirmed by reading `app/Cargo.toml` lines 1-29: neither `rand` nor `base64` appears).

---

### `app/src/tests/oauth_magic_link_flow.rs` (new test file)

**Analog:** `ferro-mcp-oauth/src/token.rs` tests (lines 120-259) — the `bootstrap_test_cache()` + `Cache::put/get/forget` unit-test harness is the exact model.

**Imports pattern** (copy from `token.rs` lines 122-131):
```rust
use crate::migrations::Migrator;  // only if DB needed for user lookup
use ferro_mcp_oauth::cache_test_helpers::bootstrap_test_cache;
use ferro::Cache;
use std::time::Duration;
```

**bootstrap_test_cache call pattern** (analog: `token.rs` line 183 and 203):
```rust
#[tokio::test]
async fn token_single_use() {
    bootstrap_test_cache();
    // put → get → forget → get returns None
    let token = "test_token_abc123_unique";
    let user_id: i64 = 42;
    Cache::put(
        &format!("magic_link:{token}"),
        &user_id,
        Some(Duration::from_secs(15 * 60)),
    ).await.expect("cache put should succeed");

    let first: Option<i64> = Cache::get(&format!("magic_link:{token}")).await.ok().flatten();
    assert!(first.is_some(), "token should exist before forget");

    let _ = Cache::forget(&format!("magic_link:{token}")).await;

    let second: Option<i64> = Cache::get(&format!("magic_link:{token}")).await.ok().flatten();
    assert!(second.is_none(), "token must be gone after forget (single-use)");
}
```

**Test file registration:** Add `pub mod oauth_magic_link_flow;` to `app/src/tests/mod.rs` (currently line 1 only has `pub mod mcp_tenant_isolation;`).

**SC-3 async OAuth flow test strategy** (RESEARCH Finding 7, unit-style staged verification):
Test each logical step by calling helpers and cache operations directly, asserting intermediate state. Do NOT attempt full HTTP round-trip (no `reqwest` dep needed). Structure:

```
Step 1: call store_oauth_return_to("...") → assert session key is set (via take check)
Step 2: put token in Cache::put → assert Cache::get returns Some
Step 3: get+forget token → simulate verify handler → assert Cache::get returns None (single-use)
Step 4: call oauth_resume_redirect with stored key present → assert returns 302 to stored URL
Step 5: call oauth_resume_redirect without stored key → assert returns 302 to default "/"
```

Each step is a separate `#[tokio::test]` function. `bootstrap_test_cache()` at the start of each cache test.

---

## Shared Patterns

### Session API
**Source:** `framework/src/session/mod.rs` (verified usage in `authorize.rs` lines 14, 98-100 and `auth_controller.rs` lines 4, 141-144)
**Apply to:** `resume.rs` (wraps both), `authorize.rs` (call store helper), `consent.rs` (call take helper), `auth_controller.rs` (replace inline reads)

```rust
// Read-only (returns Option<&SessionData>)
use ferro::session::session;
let val: Option<String> = session().and_then(|s| s.get("key"));

// Mutable (closure form)
use ferro::session::session_mut;
session_mut(|s| {
    s.put("key", value);        // write
    s.forget("key");            // delete
    let _: Option<T> = s.get("key");  // read (in mut closure, but prefer session() for reads)
});
```

Note: read (`session()`) and forget (`session_mut`) must be two separate calls — they cannot be combined in one closure because `session_mut` takes `&mut SessionData`.

### Cache Single-Use Pattern
**Source:** `ferro-mcp-oauth/src/token.rs` lines 61-72
**Apply to:** `auth_controller.rs` verify_magic_link handler, `app/src/tests/oauth_magic_link_flow.rs`

```rust
use ferro::Cache;
use std::time::Duration;

// Store with TTL
Cache::put(&key, &value, Some(Duration::from_secs(15 * 60))).await?;

// Single-use retrieve: get THEN forget BEFORE any validation
let value: Option<T> = Cache::get(&key).await.ok().flatten();
let _ = Cache::forget(&key).await;  // idempotent — forget even if None

let value = value.ok_or_else(|| /* error response */)?;
```

### 302 Redirect Builder
**Source:** `authorize.rs` lines 101-103, `auth_controller.rs` line 183-184
**Apply to:** `resume.rs` (`oauth_resume_redirect`), `auth_controller.rs` (verify success path)

```rust
// Error-path redirect (inside Err()):
return Err(ferro::HttpResponse::new()
    .status(302)
    .header("Location", dest));

// Success-path redirect (inside Ok()):
return Ok(HttpResponse::new().status(302).header("Location", dest));
```

`oauth_resume_redirect("/")` returns `ferro::Response` — callers use `return oauth_resume_redirect("/")`, NOT `oauth_resume_redirect("/")?`.

### Environment Dev-Mode Gate
**Source:** `framework/src/config/env.rs` lines 51-53
**Apply to:** `auth_controller.rs` request_link handler (D-03 branch)

```rust
use ferro::config::env::Environment;

let env = Environment::detect();
if env.is_development() {
    // is_development() returns true for Local (APP_ENV=local) and Development
    tracing::info!(magic_link = %verify_url, "Magic-link generated (dev mode)");
    JsonUi::render_file("src/views/login_confirm.json", json!({ "dev_link": verify_url }))
} else {
    // NotificationDispatcher::send path (non-dev, not exercised in CI)
    JsonUi::render_file("src/views/login_confirm.json", json!({}))
}
```

### ferro-mcp-oauth Module Export Shape
**Source:** `ferro-mcp-oauth/src/lib.rs` lines 8-34
**Apply to:** `lib.rs` additions for `resume.rs`

```rust
// Module declarations (lines 8-19 pattern):
pub mod resume;

// Top-level pub use (lines 21-25 pattern):
pub use resume::{oauth_resume_redirect, store_oauth_return_to, take_oauth_return_to};

// Note: helpers are NOT added to pub mod handlers {} — that block is for route handlers only
```

### Test Bootstrap Pattern
**Source:** `ferro-mcp-oauth/src/token.rs` lines 122-131 and usage at lines 182, 202
**Apply to:** `app/src/tests/oauth_magic_link_flow.rs`

```rust
use ferro_mcp_oauth::cache_test_helpers::bootstrap_test_cache;

#[tokio::test]
async fn my_cache_test() {
    bootstrap_test_cache();  // binds InMemoryCache into App container
    // now Cache::put/get/forget work
}
```

`bootstrap_test_cache()` is idempotent for re-binding; call at the top of each test that uses `Cache`.

---

## No Analog Found

All files have close analogs in the codebase. No entries.

---

## Metadata

**Analog search scope:** `ferro-mcp-oauth/src/`, `app/src/controllers/`, `app/src/views/`, `app/src/tests/`, `framework/src/cache/`, `framework/src/config/`
**Files scanned:** 12 source files read directly
**Pattern extraction date:** 2026-06-11
