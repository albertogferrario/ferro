# Phase 202: Login-resume contract + magic-link sample app — Research

**Researched:** 2026-06-11
**Domain:** `ferro-mcp-oauth` session-contract helpers + sample app magic-link login
**Confidence:** HIGH (all findings verified from source files in this workspace)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Add a symmetric helper set to `ferro-mcp-oauth` owning the `oauth_return_to` session
  key end to end: a `store` helper (called by `/authorize`), a `take/consume` helper (called
  post-auth — reads and clears in one call), and an ergonomic `oauth_resume_redirect(default)`
  helper returning a 302 `HttpResponse`. The `"oauth_return_to"` string becomes a single
  crate-owned constant (or private behind helpers). Document that any login method that calls
  the take/redirect helper participates in the OAuth flow.

- **D-02:** Store the single-use, TTL-bounded magic-link token in `ferro-cache`, keyed by the
  high-entropy token, value = user identifier. TTL ~15 min; deleted on first successful verify.
  Token generated with `rand` (already a dep). Mirrors the authorization-code storage precedent
  (D-03 from Phase 199). App-local exemplar infrastructure — `ferro-mcp-oauth` gains no
  magic-link types.

- **D-03:** Gate on `Environment::is_development()` (true for `APP_ENV=local`, the default).
  In dev/test: do not send a real email — surface the magic link directly on the post-request
  confirmation page (JSON-UI) and via `tracing`. In non-dev: dispatch via
  `ferro-notifications` `Channel::Mail` (documented, not exercised by the test).

- **D-04:** `GET /auth/verify?token=...` is the verify handler. Absent/expired → re-render
  request-link page with error. Valid → delete token (single-use), `Auth::login(user_id)`,
  return D-01 resume redirect.

- **D-05:** Replace `src/views/login.json` with an email-only "send login link" form posting to
  `POST /auth/login` (which becomes the request-link handler), plus a confirmation state.
  Both states render through JSON-UI with `layout: "auth"` via `ThemeMiddleware`. Delete the
  old password `login_form`/`authenticate` path entirely. `register` handler is untouched.

### Claude's Discretion
- Exact helper names and whether the session-key constant is `pub` or private.
- Token length/encoding and the exact TTL value.
- Confirmation-page copy; whether the dev link renders as a clickable anchor or plain text.
- Module layout for the helper in `ferro-mcp-oauth` (new `resume.rs` vs extending `authorize.rs`).
- Whether the acceptance test lives in `app/src/tests/` or `ferro-mcp-oauth/tests/`.

### Deferred Ideas (OUT OF SCOPE)
- Cross-device / headless magic-link auth — Phase 203 (RFC 8628).
- Consumer `verify_magic_link` adoption — gestiscilo consumer phase.
- Rate-limiting / throttling magic-link requests.
- Magic-link for registration — only login is converted.
- Real-email path in CI.
</user_constraints>

---

## Summary

Phase 202 consolidates two existing inline duplications — the `oauth_return_to` session write
in `authorize.rs` and the corresponding read/forget in `auth_controller.rs` — into a
crate-owned helper set that any login method calls. That contract is then proven by converting
the sample app's password login to magic-link: a `POST /auth/login` request-link handler
issues a `ferro-cache`-backed single-use token, and a new `GET /auth/verify?token=` handler
authenticates and resumes via the helper.

All the infrastructure needed exists and is already wired: `ferro::Cache` (static facade,
`Option<Duration>` TTL, `put`/`get`/`forget`), `session()` / `session_mut()` (the session
API the helper wraps), `rand` and `base64` (already in `ferro-mcp-oauth`'s `Cargo.toml`),
`Environment::is_development()` (Local and Development both return true), and
`ferro-notifications` `Channel::Mail` for the documented non-dev path. The sample app's
`bootstrap.rs` already mounts `ThemeMiddleware::new().default_theme(Theme::default_theme())`
(the CWD-independent form, fixed in commit `10263291`). View files still resolve via
`fs::canonicalize` at request time, which is CWD-relative — not a startup panic, but a
runtime concern for the test harness.

**Primary recommendation:** Implement the helpers in a new `ferro-mcp-oauth/src/resume.rs`
module, export from `lib.rs`, refactor `authorize.rs` Step 3 to call `store_oauth_return_to`,
and replace the inline reads in `auth_controller.rs` with `oauth_resume_redirect("/")`. The
magic-link token, handlers, and views are app-local in `app/src/`; no `ferro-mcp-oauth` API
surface changes beyond the resume helpers.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Session key ownership (`oauth_return_to`) | `ferro-mcp-oauth` crate | — | The authorize endpoint writes it; the crate must own the constant and helpers |
| Login-resume redirect | Login handler (app) | `ferro-mcp-oauth` (helper) | The handler calls the helper; the helper encapsulates session read+clear+redirect |
| Magic-link token generation and storage | App (`auth_controller`) | `ferro::Cache` | App-local exemplar; no new crate surface |
| Token verification and session establishment | App (`auth_controller`) | `ferro::Auth` | `Auth::login(user_id)` is already the session-establishment call |
| Dev-mode link surfacing | App (confirmation view) | `tracing` | `is_development()` branch in the request-link handler |
| Non-dev email dispatch | App (`auth_controller`) | `ferro-notifications` | Handler calls `NotificationDispatcher::send`; no new crate needed |
| View rendering | JSON-UI layer | `ThemeMiddleware` | Already mounted globally in bootstrap |
| Acceptance test session continuity | `app/src/tests/` | — | Same pattern as `mcp_tenant_isolation.rs` |

---

## Standard Stack

### Core (all already in workspace — no new dependencies)

| Library / API | Version | Purpose | Location |
|---------------|---------|---------|----------|
| `ferro::Cache` (static facade) | workspace | Token store (`put`/`get`/`forget`, `Option<Duration>` TTL) | `framework/src/cache/mod.rs` |
| `ferro::session::{session, session_mut}` | workspace | Session read / write wrapping for helpers | `framework/src/session/mod.rs` |
| `ferro::Auth` | workspace | `Auth::check()`, `Auth::login(user_id)` | framework crate |
| `ferro::config::env::Environment` | workspace | `Environment::detect()`, `is_development()` | `framework/src/config/env.rs` |
| `rand 0.8` | `ferro-mcp-oauth/Cargo.toml` | High-entropy token generation | already a dep |
| `base64 0.22` | `ferro-mcp-oauth/Cargo.toml` | URL-safe token encoding | already a dep |
| `ferro::JsonUi::render_file` | workspace | Auth view rendering | framework crate |
| `ferro-notifications` `NotificationDispatcher` | workspace | Non-dev mail dispatch | `ferro-notifications/src/dispatcher.rs` |

### Supporting

| Library | Purpose | When to Use |
|---------|---------|-------------|
| `tracing::info!` | Log dev-mode magic link | Dev surfacing alongside view |
| `ferro_mcp_oauth::cache_test_helpers::bootstrap_test_cache()` | In-memory cache for tests | Every test that calls `Cache::put/get/forget` |
| `pkce::generate_auth_code()` | URL-safe 32-byte random (43 chars) | Reuse for magic-link token generation |

**No new crate dependencies required.** All libraries are already present in the workspace.

---

## Architecture Patterns

### System Architecture Diagram

```
GET /authorize (unauthenticated)
        │
        ▼
  authorize.rs Step 3
  store_oauth_return_to(session, url)  ← resume.rs helper
        │
        ▼ 302 /auth/login
        │
GET /auth/login ─────────────────────► login_page handler
                                              │ JsonUi::render_file("src/views/login.json")
                                              ▼ email-only form
POST /auth/login (email) ─────────────► request_link handler (replaces login)
                                              │ issue token → Cache::put(key, user_id, Some(15min))
                                              │ is_development? → surface link on confirmation view
                                              │            else → NotificationDispatcher::send(mail)
                                              ▼ JsonUi confirmation page

GET /auth/verify?token=XXX ───────────► verify handler (new route in guest group)
                                              │ Cache::get(key)? → absent → error view
                                              │ Cache::forget(key)  [single-use, before auth]
                                              │ Auth::login(user_id)
                                              ▼
                                     oauth_resume_redirect("/")   ← resume.rs helper
                                              │ reads + clears oauth_return_to from session
                                              ▼ 302 to stored /authorize URL  (or "/" if absent)

GET /authorize (now authenticated)
        │
        ▼ consent page rendered
```

### Recommended Project Structure Changes

```
ferro-mcp-oauth/src/
├── lib.rs              # + pub mod resume; pub use resume::{store_oauth_return_to, oauth_resume_redirect};
├── resume.rs           # NEW: const OAUTH_RETURN_TO_KEY, store/take/redirect helpers
└── authorize.rs        # Step 3: calls store_oauth_return_to() instead of inline put

app/src/
├── controllers/
│   └── auth_controller.rs  # login → request_link handler; new verify handler; delete login_form/authenticate
├── views/
│   ├── login.json          # Replace: email-only form + confirmation state (one file, $data-driven)
│   └── login_confirm.json  # OR: separate confirmation file (see Pitfall 1)
└── routes.rs               # + get!("/auth/verify", verify_magic_link) in guest group
```

### Pattern 1: Resume Helper Set

**What:** Three functions in `ferro-mcp-oauth/src/resume.rs` that own the session key lifecycle.

**When to use:** Any login handler that must resume an in-flight OAuth flow after authentication.

```rust
// Source: synthesized from authorize.rs Step 3 + auth_controller.rs login/login_form

const OAUTH_RETURN_TO_KEY: &str = "oauth_return_to";

/// Store the authorize URL so the login handler can resume it after authentication.
pub fn store_oauth_return_to(url: String) {
    session_mut(|s| {
        s.put(OAUTH_RETURN_TO_KEY, url);
    });
}

/// Take the stored return URL, clearing it from the session (consume-on-read).
pub fn take_oauth_return_to() -> Option<String> {
    let url: Option<String> = session().and_then(|s| s.get(OAUTH_RETURN_TO_KEY));
    if url.is_some() {
        session_mut(|s| { s.forget(OAUTH_RETURN_TO_KEY); });
    }
    url
}

/// 302 redirect to the stored OAuth return URL, or to `default` when absent.
pub fn oauth_resume_redirect(default: &str) -> ferro::Response {
    let dest = take_oauth_return_to().unwrap_or_else(|| default.to_string());
    Ok(ferro::HttpResponse::new()
        .status(302)
        .header("Location", dest))
}
```

**Important:** `oauth_resume_redirect` returns `ferro::Response` (`Result<HttpResponse, HttpResponse>`). Callers use `return oauth_resume_redirect("/")` directly. The pattern mirrors the existing pattern in `auth_controller.rs` lines 180-184.

### Pattern 2: Cache::put Signature

**What:** The framework static `Cache` facade takes `Option<Duration>` for TTL (not `Duration`).

```rust
// Source: framework/src/cache/mod.rs line 145-154
// `None` means use default TTL (or no expiration).
Cache::put(
    &format!("magic_link:{token}"),
    &user_id,  // i64
    Some(Duration::from_secs(15 * 60)),
).await?;

// Single-use: forget BEFORE checking validity (mirrors T-199-02 in token.rs)
let user_id: Option<i64> = Cache::get(&key).await.ok().flatten();
let _ = Cache::forget(&key).await;  // always forget, even if None
let user_id = user_id.ok_or_else(|| /* re-render with error */)?;
```

**Key difference from ferro-cache crate:** The framework's `Cache::put` takes `Option<Duration>`, while `ferro-cache::Cache::put` takes `Duration`. The `ferro-mcp-oauth` token store uses the framework static facade (`use ferro::Cache`), not `ferro-cache` directly.

### Pattern 3: Magic-Link Token Generation

Reuse `pkce::generate_auth_code()` from within `ferro-mcp-oauth`, or replicate it in `auth_controller.rs`:

```rust
// Source: ferro-mcp-oauth/src/pkce.rs lines 16-18
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;

fn generate_magic_link_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(bytes)  // 43 chars, URL-safe, 256-bit entropy
}
```

**Note:** `rand` and `base64` are already in `ferro-mcp-oauth/Cargo.toml`; they are NOT declared in the `app` crate's `Cargo.toml`. The magic-link handler lives in the app, so the app's `Cargo.toml` will need these dependencies added OR the app can call a public function from `ferro-mcp-oauth` if `generate_auth_code` is re-exported. The simplest path: add `rand` and `base64` to `app/Cargo.toml` and implement the token generator locally.

### Pattern 4: Dev-Mode Branch

```rust
// Source: framework/src/config/env.rs line 51-53
// is_development() returns true for Local and Development.
use ferro::config::env::Environment;

let env = Environment::detect();
if env.is_development() {
    // Surface link on confirmation page data; log via tracing
    tracing::info!(magic_link = %verify_url, "Magic-link generated (dev mode)");
    // Pass verify_url as data to JsonUi confirmation view
} else {
    // NotificationDispatcher::send (documented, not tested in CI)
}
```

### Pattern 5: JSON-UI Email-Only Login View

The existing `login.json` test (`login_view_is_valid_and_posts_to_login`) asserts the presence of `password` and `email` fields. The test MUST be updated when `login.json` is converted.

New `login.json` structure:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Sign in",
  "layout": "auth",
  "root": "card",
  "elements": {
    "card": { "type": "Card", "props": { "title": "Sign in", "variant": "elevated" }, "children": ["form"] },
    "form": {
      "type": "Form",
      "props": { "action": { "handler": "/auth/login", "method": "POST" }, "max_width": "narrow" },
      "children": ["email", "submit"]
    },
    "email": {
      "type": "Input",
      "props": { "field": "email", "label": "Email", "input_type": "email", "required": true,
                 "data_path": "/email", "error": { "$data": "/error" } }
    },
    "submit": {
      "type": "Button",
      "props": { "label": "Send login link", "button_type": "submit", "variant": "default" }
    }
  }
}
```

A separate `login_confirm.json` (or an embedded `$if` in the same file) renders the confirmation state with the dev link. Two files is the simpler approach given the current `$if` expression support.

### Anti-Patterns to Avoid

- **Calling `take_oauth_return_to` without immediately redirecting:** The session key is gone after the call. Extract the URL and use it in one step, or use `oauth_resume_redirect` directly.
- **Running `Cache::get` without `Cache::forget` on the same request:** Leaves the token reusable. Always forget before (or immediately after) reading.
- **Panic-on-startup from view file CWD-relative paths:** `JsonUi::render_file("src/views/login.json", ...)` calls `fs::canonicalize` at request time. This fails at request time (not boot) when the working directory differs. This is distinct from the Theme `from_path` panic fixed in commit `10263291`. See Pitfall 2.
- **Adding magic-link types to `ferro-mcp-oauth`:** The token is app-local exemplar infrastructure. The `ferro-mcp-oauth` surface gains only the resume helpers (D-01 scope).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| High-entropy URL-safe token | Custom PRNG/encoding | `rand::thread_rng().gen::<[u8; 32]>()` + `URL_SAFE_NO_PAD.encode()` | Already used in `pkce::generate_auth_code()`; proven pattern |
| Single-use token revocation | DB table with TTL reaper | `ferro::Cache` + `Cache::forget` | TTL-native, no reaper job; mirrors Phase 199 authorization-code precedent |
| Session key ownership | Ad-hoc string literals | `const OAUTH_RETURN_TO_KEY` + helper functions | One source of truth; prevents the duplicate-literal drift that prompted this phase |
| Mail notification | SMTP hand-roll | `ferro-notifications` `NotificationDispatcher::send` | Already in workspace; configured via `MailConfig::from_env()` |

---

## Common Pitfalls

### Pitfall 1: Single vs. Two JSON-UI View Files for Login/Confirmation States
**What goes wrong:** Attempting to express both the "send link" form and the "check your email" confirmation in a single `login.json` via `$if` expressions becomes complex when the dev link (a dynamic URL string) must conditionally appear.
**Why it happens:** `$if` works well for showing/hiding static elements but gets awkward for rendering dynamic string content from handler data in the same layout root.
**How to avoid:** Use two separate files: `login.json` (the email form) and `login_confirm.json` (the confirmation + optional dev link). The `login_page` handler renders `login.json`; the request-link handler renders `login_confirm.json` on success with `json!({"dev_link": verify_url})` or `json!({})` in production.
**Warning signs:** A complex `$if`/`$data` structure in a single file that the planner finds difficult to describe in a task action.

### Pitfall 2: View Files Remain CWD-Relative (Runtime, Not Startup)
**What goes wrong:** `JsonUi::render_file("src/views/login.json", ...)` calls `fs::canonicalize(path)` which is relative to the process CWD. This is a request-time error, not a startup panic.
**Why it happens:** The Theme CWD-panic was fixed in commit `10263291` by switching to `Theme::default_theme()` (embedded). View files are not embedded — they are read from disk at request time.
**How to avoid:** The existing app runs with CWD = the `app/` directory during normal operation. The test harness in `app/src/tests/` does not call handlers directly via HTTP, so view files are not exercised in those unit tests. The acceptance test (SC-3) must be structured so it does not call `render_file` directly (or must set CWD appropriately). Integration tests that exercise view rendering should cd to the `app/` directory before calling handlers, or the acceptance test should avoid asserting on rendered HTML when testing the redirect chain.
**Warning signs:** `Failed to load spec: No such file or directory` in test logs when running from a different working directory.

### Pitfall 3: Forgetting to Delete the Token Before Validation
**What goes wrong:** Token replay is possible if `Cache::get` succeeds but `Cache::forget` is called only after validation passes. A network error between get and forget leaves a used token consumable.
**Why it happens:** Natural ordering: get → validate → forget. But forget-before-validate is the security invariant (mirrors T-199-02 in `token.rs`).
**How to avoid:** Pattern in `token.rs` lines 62-64: `get` then `forget` BEFORE any validation. The verify handler must do: get → forget → validate.
**Warning signs:** Test `replay_code_returns_none_after_forget` in `token.rs` documents this invariant; mirror this test for the magic-link verify handler.

### Pitfall 4: `oauth_resume_redirect` Returns `ferro::Response` — Cannot Be Used With `?`
**What goes wrong:** `oauth_resume_redirect` returns `Result<HttpResponse, HttpResponse>` (a `Response`). Callers who try to use `?` on it expecting to propagate errors will get unexpected behavior since `Ok(redirect)` is already the success path.
**How to avoid:** Use `return oauth_resume_redirect(default)` directly, not `oauth_resume_redirect(default)?`. This mirrors the existing `login_form` pattern at lines 180-184 of `auth_controller.rs`.

### Pitfall 5: `consent.rs` Also Clears `oauth_return_to`
**What goes wrong:** `consent.rs` line 235 calls `session_mut(|s| { s.forget("oauth_return_to"); })` inline. After the resume helper is introduced, this becomes a second site that references the session key — not as a duplicate write, but as an explicit cleanup at the consent step.
**Why it happens:** The consent handler clears the key when the user has already reached the consent page (so a back-navigation from consent to login doesn't re-redirect).
**How to avoid:** Either (a) update `consent.rs` to call `take_oauth_return_to()` (discarding the result) so the key constant is used consistently, or (b) leave it as an inline forget since the key name is now a public constant. The planner must include this site in the refactor scope.

### Pitfall 6: `rand` and `base64` Are Not App Dependencies
**What goes wrong:** The magic-link token generation code in `auth_controller.rs` uses `rand` and `base64`, but these are declared in `ferro-mcp-oauth/Cargo.toml`, not `app/Cargo.toml`.
**How to avoid:** Add `rand = "0.8"` and `base64 = "0.22"` to `app/Cargo.toml`. Alternatively, expose `pub fn generate_token() -> String` from `ferro-mcp-oauth/src/pkce.rs` (rename or re-export `generate_auth_code`) and call it from the app.

### Pitfall 7: GuestMiddleware Blocks the Verify Handler for Authenticated Users
**What goes wrong:** `GET /auth/verify?token=` added to the guest group (`.middleware(GuestMiddleware::redirect_to("/"))`) would redirect already-authenticated users away before they can verify. This is actually correct behavior (an already-authenticated user clicking a magic link should be redirected to `/` not re-verified), but it means the acceptance test cannot use an authenticated session to simulate the verify step.
**How to avoid:** The verify handler should be in the guest group — this is intentional. The acceptance test must use a fresh unauthenticated session for the full flow. Confirm the GuestMiddleware behavior is acceptable for re-authentication edge cases.

---

## Research Areas: Specific Findings

### Finding 1: Session API (D-01)

**Verified from:** `framework/src/session/mod.rs` and existing usage in `authorize.rs`/`auth_controller.rs`.

The session API available:
- `session() -> Option<&SessionData>` — read-only access; returns `None` if no session active
- `session_mut(|s| { ... })` — mutable closure; `s.put(key, value)`, `s.forget(key)`, `s.get::<T>(key) -> Option<T>`
- `get_csrf_token() -> Option<String>` — not needed for the resume helper

The exact existing patterns are:
- Write (authorize.rs line 98-100): `session_mut(|s| { s.put("oauth_return_to", return_url.clone()); });`
- Read (auth_controller.rs line 141): `let return_to: Option<String> = session().and_then(|s| s.get("oauth_return_to"));`
- Clear (auth_controller.rs line 142-144): `session_mut(|s| { s.forget("oauth_return_to"); });`

The `take_oauth_return_to()` helper combines the read and clear into one call. The read must happen before the forget (not in the same closure) because the session API separates read and mutable access.

**302 redirect builder pattern** (confirmed from `authorize.rs` line 101-103):
```rust
ferro::HttpResponse::new()
    .status(302)
    .header("Location", dest)
```

### Finding 2: Cache API Exact Signature (D-02)

**Verified from:** `framework/src/cache/mod.rs`.

```rust
// Put with explicit TTL
Cache::put(key: &str, value: &T, ttl: Option<Duration>) -> Result<(), FrameworkError>

// Get (returns deserialized value or None if expired/absent)
Cache::get::<T>(key: &str) -> Result<Option<T>, FrameworkError>

// Remove (returns true if key existed)
Cache::forget(key: &str) -> Result<bool, FrameworkError>
```

TTL argument is `Option<Duration>`. `None` uses the configured default TTL (or no expiration).
For 15-minute magic-link token: `Some(Duration::from_secs(15 * 60))`.

Cache key convention mirrors existing `"mcp:code:{code}"` — use `"magic_link:{token}"` for the magic-link tokens to keep the namespace distinct.

Test bootstrap: `ferro_mcp_oauth::cache_test_helpers::bootstrap_test_cache()` binds an `InMemoryCache` into the App container. The acceptance test in `app/src/tests/` must call this (or an equivalent) before any `Cache::put/get/forget` operations.

### Finding 3: Auth Controller Delete Scope (D-05)

**Verified from:** `app/src/controllers/auth_controller.rs` full read.

Functions to DELETE (password path, D-05):
- `async fn login_form(req: Request) -> Response` (private function, lines 175-195) — the browser-form password handler
- `async fn authenticate(email: &str, password: &str) -> Result<bool, HttpResponse>` (lines 198-213) — password verifier

Functions to CONVERT:
- `login_page` → rename or keep as is (renders the new email-only view)
- `login` → becomes the request-link handler (takes email, issues token, renders confirmation)

Functions to ADD:
- `verify_magic_link` — `GET /auth/verify?token=...` handler

Functions to KEEP UNCHANGED:
- `register` — uses `Auth::login(user.id as i64)` directly without `authenticate()`
- `logout`, `profile`

The `LoginInput` struct currently has `email` and `password` fields. The new handler only needs `email`. Either create `RequestLinkInput { email: String }` or rename the struct.

**Test that must be updated:** `login_view_is_valid_and_posts_to_login` in `auth_controller.rs` (lines 248-264) asserts `v["elements"]["password"]` exists. This will fail once `login.json` removes the password field. The plan must include updating this test.

### Finding 4: Routes Change (D-04, D-05)

**Verified from:** `app/src/routes.rs`.

Current guest group (lines 38-42):
```rust
group!("/auth", {
    get!("/login", controllers::auth_controller::login_page).name("auth.login.page"),
    post!("/register", controllers::auth_controller::register).name("auth.register"),
    post!("/login", controllers::auth_controller::login).name("auth.login"),
}).middleware(GuestMiddleware::redirect_to("/")),
```

Required changes:
- Add `get!("/verify", controllers::auth_controller::verify_magic_link).name("auth.verify")` to the guest group
- The `POST /auth/login` route name and handler reference stay the same (handler is repurposed, not renamed at the route level)

### Finding 5: ThemeMiddleware and Bootstrap (SC-4)

**Verified from:** `app/src/bootstrap.rs` lines 75.

```rust
global_middleware!(ThemeMiddleware::new().default_theme(Theme::default_theme()));
```

The CWD-independent embedded default theme is already in place (commit `10263291`). The confirmation view will inherit the same theme automatically since it goes through `JsonUi::render_file`. No changes needed to bootstrap.

### Finding 6: ferro-notifications API (D-03)

**Verified from:** `ferro-notifications/src/lib.rs` and `dispatcher.rs`.

The non-dev mail path requires:
1. Implementing `Notification` trait on a `MagicLinkNotification` struct
2. Implementing `Notifiable` trait on something with the user's email
3. Calling `NotificationDispatcher::send(&notifiable, notification).await`

The `NotificationDispatcher` uses a global `CONFIG` (OnceLock). In the sample app, `MailConfig::from_env()` provides the SMTP config. The dispatcher must be configured before sending. Since this path is non-dev only (not exercised by tests), the app must either configure the dispatcher in `bootstrap.rs` or do a lazy check. The safest approach: only call `send` when `!env.is_development()`, and wrap with a `tracing::warn` on error so a missing SMTP config does not crash the app.

**CI safety:** The non-dev branch is never reached in `APP_ENV=local` (tests), so no SMTP dependency is introduced in `cargo test`.

### Finding 7: Acceptance Test Harness (SC-3)

**Verified from:** `app/src/tests/mcp_tenant_isolation.rs` and `tests/mod.rs`.

The existing test harness:
- Opens in-memory SQLite, runs `Migrator::up`
- Seeds data via direct SeaORM inserts
- Builds test-local middleware and calls `middleware.handle(req, next)` directly
- Calls `ferro_mcp_server::handle_tools_call` for business logic
- Does NOT boot a full HTTP server for request/response

For the SC-3 async OAuth flow acceptance test, the full sequence involves session state persisting across multiple HTTP requests. This is the key challenge: the existing harness does not test multi-request session continuity.

**Options:**
1. **Unit-level test:** Test each step independently by injecting session state manually (call store/take helpers directly, assert Cache state after token issue, assert redirect target after verify). This does not test session continuity end-to-end but is tractable with the existing harness.
2. **Full HTTP integration test:** Boot a real server on a random port, use a cookie-jar HTTP client (e.g., `reqwest` with cookie store) to carry session cookies across requests. This truly tests SC-3 but requires adding `reqwest` as a dev-dependency and a live server.

**Recommendation (Claude's Discretion):** A multi-step unit test that drives the logical sequence is sufficient for SC-3 if each step is clearly labeled. The resume helper can be tested by directly calling it in a session context. The magic-link token single-use can be tested via `Cache::put`/`get`/`forget`. The test location should be `app/src/tests/oauth_magic_link_flow.rs` (parallel to `mcp_tenant_isolation.rs`) — add a `pub mod oauth_magic_link_flow;` to `tests/mod.rs`.

**Session continuity across requests in unit tests:** Session state is thread-local in the framework. Each test request runs in its own scope. To test the full flow, the test must either (a) use actual HTTP requests with a shared cookie jar, or (b) test each handler step in isolation asserting the intermediate state (Cache contents, session contents after each step). Approach (b) is more robust for CI.

### Finding 8: Publish Wiring (Phase 203 forward-compatibility)

**Verified from:** `.github/workflows/publish.yml` grep output.

```
WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server ferro-mcp-oauth"
```

`ferro-mcp-oauth` is already in Wave 2 of the publish workflow. No new publish wiring needed for Phase 202.

The resume helper is a pure same-device browser redirect mechanism. It reads a session key and issues a 302. The Phase 203 device grant uses a `device_code`/`user_code` pair and a separate polling endpoint — it does not use `oauth_return_to` at all. The resume helper does NOT bake in authorization-code-loopback assumptions: it simply reads an arbitrary URL from the session and redirects to it. Any login front door (password, magic-link, future SSO, device grant user-code verification) can call `oauth_resume_redirect("/")` independently.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust's built-in `#[test]` / `#[tokio::test]` |
| Config file | none (cargo workspace) |
| Quick run command | `cargo test -p app -- --test-output immediate 2>&1` |
| Full suite command | `cargo test --all-features 2>&1` |

### SC → Test Map

| SC | Behavior | Test Type | Automated Command | File |
|----|----------|-----------|-------------------|------|
| SC-1 (resume helper) | `store_oauth_return_to` writes the session key; `take_oauth_return_to` reads and clears it; absent session returns `None` | unit | `cargo test -p ferro-mcp-oauth -- resume` | `ferro-mcp-oauth/src/resume.rs` (inline `#[cfg(test)]`) |
| SC-1 (take clears) | After `take_oauth_return_to()`, a second call returns `None` | unit | same | same |
| SC-1 (`oauth_resume_redirect`) | With stored key returns 302 to that URL; without returns 302 to default | unit | same | same |
| SC-2 (token single-use) | `Cache::get` → `Cache::forget` → second `Cache::get` returns `None` | unit | `cargo test -p app -- verify` | `app/src/tests/oauth_magic_link_flow.rs` ❌ Wave 0 |
| SC-2 (token expiry) | Expired token returns `None` from `Cache::get` | unit | same | same |
| SC-2 (dev link surfacing) | `is_development()` returns true for `APP_ENV=local` | unit | `cargo test -p framework -- environment` | `framework/src/config/env.rs` (existing) |
| SC-3 (async OAuth flow) | Unauth `/authorize` → session stores key → `POST /auth/login` → token in cache → `GET /auth/verify?token` → resume redirect | integration | `cargo test -p app -- oauth_magic_link_flow` | `app/src/tests/oauth_magic_link_flow.rs` ❌ Wave 0 |
| SC-4 (view validity) | New `login.json` is valid JSON-UI v2 schema; posts to `/auth/login`; email field present; no password field | unit | `cargo test -p app -- login_view` | `app/src/controllers/auth_controller.rs` (update existing test) |
| SC-5 (clippy/test green) | `cargo clippy --all-targets --all-features -- -D warnings` + `cargo test --all-features` pass | command gate | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | CI gate |
| SC-5 (no CWD panic at boot) | ThemeMiddleware uses embedded default (already fixed); confirm no new `from_path` or CWD-sensitive startup code introduced | manual review | code review during plan execution | — |

### Sampling Rate
- **Per task commit:** `cargo clippy --all-targets -- -D warnings`
- **Per wave merge:** `cargo test -p app -p ferro-mcp-oauth`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`

### Wave 0 Gaps
- [ ] `app/src/tests/oauth_magic_link_flow.rs` — covers SC-2 (token single-use) and SC-3 (async flow)
- [ ] `app/src/tests/mod.rs` — add `pub mod oauth_magic_link_flow;`
- [ ] `ferro-mcp-oauth/src/resume.rs` — unit tests embedded in the new module

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `rand` and `base64` are NOT in `app/Cargo.toml`; they are only in `ferro-mcp-oauth/Cargo.toml` | Finding 6 | If app already has them, no action needed; low risk |
| A2 | The confirmation view can safely be a second `.json` file (not embedded in `login.json`) — `JsonUi::render_file` supports any file path | Pitfall 1 | Two-file approach is the standard pattern; no known restriction |
| A3 | `GuestMiddleware` redirects authenticated users away from `/auth/verify` — this is acceptable behavior | Pitfall 7 | If re-authentication use case requires verify to work for authenticated users, route placement must change |
| A4 | The `NotificationDispatcher` can be called without pre-configuration in `bootstrap.rs` (returns an error rather than panicking when unconfigured) | Finding 6 | If it panics on unconfigured send, bootstrap must configure it. Mitigated by wrapping in `if !env.is_development()` |

---

## Open Questions

1. **Acceptance test session continuity mechanism**
   - What we know: The existing test harness uses direct middleware/dispatch calls, not a full HTTP round-trip. Session state is thread-local.
   - What's unclear: Whether the acceptance test should use actual HTTP requests (adds `reqwest` as dev-dep) or unit-style staged verification.
   - Recommendation: Use unit-style staged verification in `app/src/tests/oauth_magic_link_flow.rs` (test each step with `bootstrap_test_cache()` + direct helper calls). If the planner considers this insufficient for SC-3, add an HTTP round-trip test.

2. **`take_oauth_return_to` read-then-forget atomicity**
   - What we know: Session read and mutable access are two separate calls (cannot read and forget in the same `session_mut` closure because `session_mut` takes `&mut SessionData`, not `SessionData`).
   - What's unclear: Whether there are race conditions in concurrent requests for the same session.
   - Recommendation: Not a concern for browser-based OAuth flow (single user, sequential requests). The helper reads, checks, then forgets — this is the same two-call pattern already in the codebase.

---

## Environment Availability

All dependencies are available in the workspace. No external services required for the core implementation or tests.

| Dependency | Required By | Available | Notes |
|------------|------------|-----------|-------|
| `ferro::Cache` (InMemoryCache) | Token storage + tests | ✓ | Bootstrapped via `bootstrap_test_cache()` |
| `ferro::Auth` | Session establishment | ✓ | Framework crate |
| `ferro::Environment::is_development()` | Dev-mode gate | ✓ | Framework crate |
| `ferro-notifications` | Non-dev mail | ✓ | Workspace crate |
| SMTP server | Non-dev mail dispatch | not tested | Guarded by `is_development()`; not required for CI |

---

## Sources

### Primary (HIGH confidence — verified from source files)
- `ferro-mcp-oauth/src/authorize.rs` — exact Step 3 inline session write (line 98-100, 101-103)
- `ferro-mcp-oauth/src/token.rs` — `Cache::get`/`Cache::forget` pattern (lines 62-64); `Cache::put` with `Some(Duration)` (line 168)
- `ferro-mcp-oauth/src/lib.rs` — public export shape; `cache_test_helpers::bootstrap_test_cache`
- `ferro-mcp-oauth/src/pkce.rs` — `generate_auth_code()` (line 16-18); `rand` and `base64` deps confirmed
- `ferro-mcp-oauth/Cargo.toml` — `rand = "0.8"`, `base64 = "0.22"` confirmed present
- `app/src/controllers/auth_controller.rs` — full login/login_form/authenticate code; inline `oauth_return_to` reads; test `login_view_is_valid_and_posts_to_login`
- `app/src/views/login.json` — current view structure
- `app/src/routes.rs` — guest group structure; GuestMiddleware
- `app/src/bootstrap.rs` — `ThemeMiddleware::new().default_theme(Theme::default_theme())` confirmed CWD-independent
- `framework/src/cache/mod.rs` — `Cache::put(key, value, Option<Duration>)` exact signature (line 145-154)
- `framework/src/session/mod.rs` — exported session API
- `framework/src/config/env.rs` — `Environment::is_development()` returns true for Local|Development
- `framework/src/json_ui/mod.rs` — `render_file` calls `load_cached` via `fs::canonicalize` (CWD-relative)
- `ferro-json-ui/src/loader.rs` — `fs::canonicalize(path)` at line 123 (CWD-relative path resolution)
- `ferro-notifications/src/lib.rs` + `dispatcher.rs` — `NotificationDispatcher::send` API
- `app/src/config/mail.rs` — `MailConfig::from_env()` fields
- `app/src/tests/mcp_tenant_isolation.rs` — integration test harness pattern; session/cache setup
- `.github/workflows/publish.yml` — `ferro-mcp-oauth` already in `WAVE2_CRATES` (confirmed)
- `git show 10263291` — confirms the Theme CWD-panic was fixed; view files remain CWD-relative

### Tertiary (LOW confidence — see Assumptions Log)
- A1: `app/Cargo.toml` rand/base64 absence (not read; inferred from grep of ferry-mcp-oauth's Cargo.toml showing them there)

---

## Metadata

**Confidence breakdown:**
- Resume helper API: HIGH — exact session/HttpResponse patterns verified from source
- Cache API: HIGH — exact `Option<Duration>` signature verified from `framework/src/cache/mod.rs`
- Magic-link flow: HIGH — mirrors existing `token.rs` single-use pattern exactly
- View conversion scope: HIGH — exact fields/tests verified from `auth_controller.rs` and `login.json`
- Acceptance test structure: MEDIUM — harness pattern verified; session continuity design is a recommendation, not verified as the only path

**Research date:** 2026-06-11
**Valid until:** 2026-07-11 (stable framework codebase; no fast-moving external deps)
