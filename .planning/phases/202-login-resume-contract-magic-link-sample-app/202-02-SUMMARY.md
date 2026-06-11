---
phase: 202-login-resume-contract-magic-link-sample-app
plan: "02"
subsystem: app
tags: [magic-link, passwordless, oauth, cache, single-use-token, sample-app]
dependency_graph:
  requires:
    - 202-01 (ferro-mcp-oauth resume helpers: oauth_resume_redirect, store_oauth_return_to, take_oauth_return_to)
  provides:
    - app::controllers::auth_controller::login (magic-link request handler)
    - app::controllers::auth_controller::verify_magic_link (single-use token verify + resume)
    - app/src/tests/magic_link (single-use, expiry, dev-surface unit tests)
  affects:
    - app/src/views/login.json (email-only form, no password)
    - app/src/views/login_confirm.json (new confirmation view)
    - app/src/routes.rs (GET /auth/verify added to guest group)
tech_stack:
  added:
    - rand 0.8 (app/Cargo.toml)
    - base64 0.22 (app/Cargo.toml)
    - tracing 0.1 (app/Cargo.toml)
  patterns:
    - forget-before-validate single-use cache token (mirrors token.rs lines 62-64)
    - 256-bit URL-safe token via rand::thread_rng().gen::<[u8;32]>() + URL_SAFE_NO_PAD (mirrors pkce.rs)
    - Environment::is_development() dev-mode gate for link surfacing vs best-effort mail
    - best-effort non-dev mail via Notification + Notifiable inline impls, tracing::warn on error
key_files:
  created:
    - app/src/views/login_confirm.json
    - app/src/tests/magic_link.rs
  modified:
    - app/Cargo.toml
    - app/src/controllers/auth_controller.rs
    - app/src/routes.rs
    - app/src/views/login.json
    - app/src/tests/mod.rs
decisions:
  - "Password path (login_form + authenticate) deleted entirely — no deprecation, per architecture principles"
  - "Token stored in ferro-cache keyed magic_link:{token}, value i64 user_id, TTL 15min"
  - "forget-before-validate: Cache::forget called unconditionally before user_id is checked (T-202-01)"
  - "Non-dev mail dispatched via ferro-notifications Notification+Notifiable inline impls; any error is tracing::warn, never a hard failure"
  - "T-202-04 accepted flag: 'No account found for this email.' reveals registration status — acceptable for sample exemplar, noted for production hardening"
  - "verify_magic_link in guest group (GuestMiddleware): authenticated user clicking old link redirected to / — intentional per RESEARCH Pitfall 7"
  - "tracing added as direct dep (not re-exported by ferro); rand, base64, tracing all at versions matching ferro-mcp-oauth"
metrics:
  duration: ~8 minutes
  completed: "2026-06-11"
  tasks_completed: 2
  files_modified: 5
  files_created: 2
---

# Phase 202 Plan 02: Magic-link login handlers + tests Summary

Password login converted to single-use TTL-bounded magic-link: `POST /auth/login` issues a `ferro-cache` token; `GET /auth/verify?token=` consumes it (forget-before-validate), calls `Auth::login`, and resumes via `oauth_resume_redirect`.

## Completed Tasks

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Magic-link handlers, route, views, deleted password path | 748f6487 | app/Cargo.toml, auth_controller.rs, routes.rs, login.json, login_confirm.json |
| 2 | Unit tests: single-use, expiry, dev-surface | 2e2fe722 | magic_link.rs, tests/mod.rs |
| fmt | rustfmt on magic_link.rs | 16970348 | magic_link.rs |

## What Was Built

`app/src/controllers/auth_controller.rs` — converted:

- `login` is now the magic-link **request handler**: looks up user by email, generates a 256-bit URL-safe token via `rand::thread_rng().gen::<[u8;32]>()` + `URL_SAFE_NO_PAD`, stores in `ferro-cache` as `"magic_link:{token}"` with 15-minute TTL, branches on `Environment::is_development()` to surface the link on the confirmation view (dev) or dispatch via `ferro-notifications` best-effort (non-dev).
- `verify_magic_link` — new `GET /auth/verify?token=` handler: `Cache::get` → `Cache::forget` (unconditional, before validation) → `Auth::login(user_id)` → `return oauth_resume_redirect("/")`. Invalid/absent/expired token → re-render `login.json` with error at 422.
- `login_form` and `authenticate` deleted (password path gone).

`app/src/views/login.json` — replaced with email-only form (`email` Input + `Send login link` Button); `password` element removed.

`app/src/views/login_confirm.json` — new confirmation view with `dev_link` Button (visible only when `dev_mode: true`).

`app/src/routes.rs` — `GET /auth/verify` added to the guest group as `auth.verify`.

`app/src/tests/magic_link.rs` — three unit tests:
- `magic_link_single_use`: put → get Some → forget → get None (T-202-01)
- `magic_link_expired`: absent key → None; idempotent forget (T-202-02)
- `magic_link_dev_surface`: `APP_ENV=local` → `is_development()` true (D-03)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing dependency] Added tracing as direct app dep**
- **Found during:** Task 1 compilation
- **Issue:** `tracing::info!` and `tracing::warn!` macros in `auth_controller.rs` require `tracing` to be a direct dep — it was only transitively available via framework but not usable without an explicit declaration.
- **Fix:** Added `tracing = "0.1"` to `app/Cargo.toml`.
- **Files modified:** `app/Cargo.toml`
- **Commit:** 748f6487

**2. [Rule 1 - Bug] Fixed notification_type return type**
- **Found during:** Task 1 compilation
- **Issue:** `Notification::notification_type` returns `&'static str` (verified from trait source), not `String`. The initial impl used `String`.
- **Fix:** Changed `fn notification_type(&self) -> String` to `fn notification_type(&self) -> &'static str { "MagicLink" }`.
- **Files modified:** `app/src/controllers/auth_controller.rs`
- **Commit:** 748f6487

**3. [Rule 1 - Bug] Used req.query("token") instead of req.query::<VerifyQuery>()**
- **Found during:** Task 1 — verified from framework/src/http/request.rs
- **Issue:** The plan's pseudocode showed `req.query::<VerifyQuery>()` but the actual framework API uses `req.query("name") -> Option<String>` (no deserialize method). The `VerifyQuery` struct was not needed.
- **Fix:** Used `req.query("token")` returning `Option<String>` directly, without a `VerifyQuery` struct.
- **Files modified:** `app/src/controllers/auth_controller.rs`
- **Commit:** 748f6487

## Threat Surface Scan

No new network endpoints beyond `GET /auth/verify` (already in the plan's threat model). The `verify_magic_link` handler is gated behind `GuestMiddleware` in the guest group — authenticated users are redirected to `/` before reaching it (intentional: old magic links clicked after login are harmlessly discarded).

The non-dev `send_magic_link_mail_best_effort` function constructs an HTTP request to the configured mail provider. No new trust boundary — this path is behind `!env.is_development()` and only reached in production (non-test) environments.

## Known Stubs

None. The magic-link request and verify handlers are fully wired. The non-dev mail path is documented best-effort (not a stub — it dispatches via the real `ferro-notifications` dispatcher; it is simply not exercised in CI tests per T-202-MAIL).

**T-202-04 hardening note (accepted flag):** The unknown-email error message "No account found for this email." reveals registration status. Acceptable for the sample exemplar. Production consumers should return a generic "If an account exists, a link was sent." message to prevent email enumeration.

## Self-Check: PASSED

- `app/src/views/login_confirm.json`: EXISTS
- `app/src/tests/magic_link.rs`: EXISTS
- No `"oauth_return_to"` literal in `app/src/`: CONFIRMED
- `fn login_form` and `fn authenticate` absent: CONFIRMED
- Commit 748f6487: FOUND in git log
- Commit 2e2fe722: FOUND in git log
- `cargo test -p app magic_link`: 3 passed, 0 failed
- `cargo clippy -p app --all-targets -- -D warnings`: clean
- `cargo fmt --all -- --check`: clean
