---
phase: 202-login-resume-contract-magic-link-sample-app
reviewed: 2026-06-11T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - ferro-mcp-oauth/src/resume.rs
  - ferro-mcp-oauth/src/authorize.rs
  - ferro-mcp-oauth/src/consent.rs
  - ferro-mcp-oauth/src/lib.rs
  - ferro-mcp-oauth/src/token.rs
  - ferro-mcp-oauth/tests/flow_integration.rs
  - app/src/controllers/auth_controller.rs
  - app/src/routes.rs
  - app/src/views/login.json
  - app/src/views/login_confirm.json
  - app/src/tests/magic_link.rs
  - app/src/tests/oauth_magic_link_resume_flow.rs
  - app/src/tests/mod.rs
  - app/Cargo.toml
  - framework/src/session/mod.rs
  - framework/src/lib.rs
  - docs/src/features/authentication.md
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 202: Code Review Report

**Reviewed:** 2026-06-11
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

Phase 202 delivers the magic-link passwordless login flow and the OAuth login-resume contract for the sample app, plus the `with_test_session` test helper in `framework/src/session/mod.rs`. The security-critical path — token single-use (forget-before-validate), open-redirect invariant, PKCE S256 enforcement, CSRF constant-time comparison, and code replay prevention — is implemented correctly and is well-tested. The `rand::thread_rng()` RNG used across `pkce.rs`, `register.rs`, and `auth_controller.rs` is seeded from `OsRng` (ChaCha12-backed via `rand` 0.8's `ReseedingRng`) and is a CSPRNG: no issue there.

Four warnings and three info items were found, none of them on the critical authentication path.

---

## Warnings

### WR-01: `state` value not percent-encoded in deny and approve redirect `Location` headers

**File:** `ferro-mcp-oauth/src/consent.rs:149-150`, `241`

**Issue:** `form.state` is interpolated raw (unencoded) into the redirect `Location` header in both the deny and approve paths:

```rust
// deny (line 149-150)
format!("{}?error=access_denied&state={}", form.redirect_uri, form.state)
// approve (line 241)
format!("{}?code={}&state={}", form.redirect_uri, code, form.state)
```

If `state` contains `&`, `=`, `#`, or space characters, the Location URL is malformed and the receiving client will misparse the query string. RFC 6749 §4.1.2 requires the `state` value be passed unmodified; "unmodified" means the bytes must be preserved, which requires percent-encoding when embedding in a URL. OAuth clients are supposed to use URL-safe state values but the server cannot assume they do.

**Fix:** Percent-encode `form.state` before interpolation. The local `urlencoding::encode` helper already exists in `authorize.rs` — expose it or duplicate the same logic:

```rust
// deny
format!("{}?error=access_denied&state={}", form.redirect_uri,
    urlencoding::encode(&form.state))
// approve
format!("{}?code={}&state={}", form.redirect_uri, code,
    urlencoding::encode(&form.state))
```

---

### WR-02: `GET /auth/verify` under `GuestMiddleware` — authenticated users cannot resume an OAuth flow via a re-clicked magic link

**File:** `app/src/routes.rs:40-43`

**Issue:** The `/auth/verify` endpoint is grouped under `GuestMiddleware::redirect_to("/")`. An already-authenticated user who clicks a magic link (e.g., a link received before their previous session expired, or opened in a second tab) is silently redirected to `/` instead of consuming the token and running `oauth_resume_redirect`. In the OAuth flow this means: if a user is authenticated in a stale session but the token was issued for a new OAuth authorize request, clicking the verify link bounces them to `/` and the `oauth_return_to` stored in the _new_ session is never consumed.

The token itself is not leaked (it stays in cache until expiry), but the OAuth flow is abandoned silently rather than resumed. In a production deployment this would be a confusing UX failure that appears as a broken login from the MCP client's perspective.

**Fix:** Move `GET /auth/verify` out of the `GuestMiddleware` group so authenticated users can also land on the verify handler (which is a no-op for replay since the token was already consumed). The handler calls `Auth::login(user_id)` followed by `oauth_resume_redirect("/")`; when the user is already authenticated, calling `Auth::login` again is benign (session is regenerated), and `oauth_resume_redirect` will still redirect to the stored authorize URL if one is present.

```rust
// In routes.rs: move verify out of the GuestMiddleware group
get!("/auth/verify", controllers::auth_controller::verify_magic_link).name("auth.verify"),

group!("/auth", {
    get!("/login", controllers::auth_controller::login_page).name("auth.login.page"),
    post!("/register", controllers::auth_controller::register).name("auth.register"),
    post!("/login", controllers::auth_controller::login).name("auth.login"),
}).middleware(GuestMiddleware::redirect_to("/")),
```

---

### WR-03: `POST /auth/register` does not call `oauth_resume_redirect` — OAuth flow is abandoned on first-time registration

**File:** `app/src/controllers/auth_controller.rs:83`

**Issue:** The register handler calls `Auth::login(user.id as i64)` and immediately returns a 201 JSON response without consuming or clearing `oauth_return_to`. If an MCP client triggers an OAuth flow for a user who does not yet have an account, the authorize handler redirects them to `/auth/login`, but if the user instead navigates to POST `/auth/register` to create an account, the OAuth flow is silently abandoned and the MCP client never receives an authorization code.

The docs explicitly state: "Any login method — synchronous password, asynchronous magic-link, future SSO — must call `oauth_resume_redirect` after establishing the session." The register handler is a login-equivalent path that violates this contract.

**Fix:** After `Auth::login(user.id as i64)` in the register handler, call `oauth_resume_redirect` (or at minimum call `take_oauth_return_to()` to clear the session key) and return an appropriate redirect. Since this is currently a JSON endpoint (used by the sample app for API-style registration), one option is to return a 201 with a `Location: /` header and leave the MCP flow note in a comment, deferring the full fix if registration-during-OAuth-flow is out of scope. At minimum the `oauth_return_to` key should be cleared so it does not persist in the session:

```rust
// After Auth::login(user.id as i64):
// Clear any in-flight OAuth flow (see login-resume contract in ferro-mcp-oauth).
// Registration during an OAuth flow is not currently supported; clear the key
// so it does not linger in the session if the client later attempts a fresh flow.
let _ = ferro_mcp_oauth::take_oauth_return_to();
```

---

### WR-04: `scope` parameter silently dropped from the reconstructed `oauth_return_to` URL

**File:** `ferro-mcp-oauth/src/authorize.rs:91-97`

**Issue:** When the `/authorize` handler redirects an unauthenticated user to login, it reconstructs the authorize URL for `store_oauth_return_to` from the parsed parameters. The `scope` query parameter is read into `_scope` (line 75) but is not included in the reconstructed URL:

```rust
let return_url = format!(
    "/authorize?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}",
    // scope is missing
    ...
);
```

After login, the resumed `/authorize` request will have no `scope` parameter. Phase 199 treats scope as "single implicit scope" and the comment on line 34 acknowledges this, but the omission is silent: a future phase that introduces multi-scope support will observe that scope was already being dropped at the resume point and will get a subtle regression. The `_scope` naming convention signals "intentionally unused" but the drop has cross-request consequences that differ from a normal unused variable.

**Fix:** Either include `scope` in the reconstructed URL (safe even when it is currently ignored), or rename `_scope` to `scope` and add an explicit comment:

```rust
// Include scope in return_url so a resumed /authorize request has the
// original parameter. Currently ignored (Phase 199 single implicit scope)
// but omitting it would cause silent regressions when multi-scope is added.
let return_url = format!(
    "/authorize?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}&scope={}",
    urlencoding::encode(&client_id),
    urlencoding::encode(&redirect_uri),
    urlencoding::encode(&code_challenge),
    urlencoding::encode(&state),
    urlencoding::encode(&scope),
);
```

---

## Info

### IN-01: Email enumeration accepted flag (T-202-04) — note for documentation / production consumers

**File:** `app/src/controllers/auth_controller.rs:125-133`

**Issue:** The magic-link request handler returns a distinct error message "No account found for this email." when the submitted email is not registered. This reveals account existence to an unauthenticated caller. The code comment on line 115-116 acknowledges this as "T-202-04 accepted flag — reveals account existence; acceptable for the sample exemplar." This is correctly flagged in the code, but the `docs/src/features/authentication.md` does not mention it. A developer adopting the sample pattern in a production app may not notice this design choice and may unintentionally ship enumerable registration state.

**Fix:** Add a note in `docs/src/features/authentication.md` in the magic-link section indicating that the sample app intentionally reveals account existence and that production apps should consider returning a uniform "If an account exists, a link has been sent" response.

---

### IN-02: `std::env::set_var` in integration test without cleanup may affect parallel test workers

**File:** `ferro-mcp-oauth/tests/flow_integration.rs:58-64`

**Issue:** `test_oauth_config()` calls `std::env::set_var` for `MCP_TOKEN_SECRET`, `APP_URL`, and `APP_NAME` without restoring them afterward. Rust integration tests under `tokio::test` run in the same process. If the test binary runs other tests that read these env vars and expect different values, this mutation could cause non-deterministic failures. The `app/src/tests/magic_link.rs` counterpart correctly calls `std::env::remove_var` after its `APP_ENV` mutation (line 91).

**Fix:** Wrap the env var mutations in a scoped restore, or use `std::env::set_var` only within the test function and restore with `std::env::remove_var` in a finally-equivalent pattern after the assertion:

```rust
fn test_oauth_config() -> OAuthConfig {
    std::env::set_var("MCP_TOKEN_SECRET", "...");
    std::env::set_var("APP_URL", "http://localhost:8080");
    std::env::set_var("APP_NAME", "TestApp");
    let config = OAuthConfig::from_env().expect("...");
    // These env vars are benign for the test binary but document the intent:
    // restore them so other tests that check for absence of these vars are unaffected.
    config
}
```

The risk is low because the test binary for `ferro-mcp-oauth` likely only contains this one integration test, but the pattern is inconsistent with the cleanup in `magic_link.rs`.

---

### IN-03: `#[allow(dead_code)]` on `RegisterInput` is misleading — struct is actively used

**File:** `app/src/controllers/auth_controller.rs:20-26`

**Issue:** `RegisterInput` is annotated `#[allow(dead_code)]` but all four fields (`name`, `email`, `password`, `password_confirmation`) are read in the `register` handler via `req.json().await?`. The annotation suppresses a warning that is not actually emitted (the struct and its fields are not dead). The annotation is either a leftover from an earlier draft where the struct was not yet wired up, or was added defensively and should be removed.

**Fix:** Remove `#[allow(dead_code)]` from `RegisterInput`.

---

_Reviewed: 2026-06-11_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
