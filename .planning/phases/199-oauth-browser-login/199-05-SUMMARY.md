---
phase: 199-oauth-browser-login
plan: "05"
subsystem: app + ferro-mcp-server + ferro-mcp-oauth
tags: [oauth, mcp, bearer-validation, origin-check, jwt, routes, seam-wiring, return-to]
dependency_graph:
  requires:
    - 199-01 (crate scaffold, OAuthConfig)
    - 199-02 (DCR handlers, oauth_clients migration)
    - 199-03 (validate_bearer, BearerCheck, jwt/pkce)
    - 199-04 (authorize/consent/token handlers)
  provides:
    - app/src/routes.rs: six OAuth route mounts (discovery x2, DCR, authorize x2, token)
    - app/src/controllers/mcp.rs: real bearer validation seam + Origin check
    - app/src/controllers/auth_controller.rs: oauth_return_to post-login redirect
    - ferro-mcp-server/src/auth.rs: BearerOutcome only (extract_bearer deleted)
    - ferro-mcp-oauth/src/lib.rs: handlers module re-export
  affects:
    - ferro-mcp-server/src/lib.rs (extract_bearer removed from re-export)
    - app/Cargo.toml (ferro-mcp-oauth dep added)
tech_stack:
  added: []
  patterns:
    - validate_bearer(header, config, expected_tenant) at /mcp seam (D-07 order: sig+exp→401, aud→403, tenant→403)
    - Origin guard before body read (present+mismatched→403, absent→allowed, T-15)
    - OAuthConfig::from_env() fail-closed: Err → 401 challenge (T-199-13b)
    - session().and_then(s.get) + session_mut(s.forget) for oauth_return_to (D-06)
    - ferro_mcp_oauth::handlers module for clean route import
key_files:
  created: []
  modified:
    - ferro-mcp-server/src/auth.rs
    - ferro-mcp-server/src/lib.rs
    - ferro-mcp-oauth/src/lib.rs
    - app/Cargo.toml
    - app/src/routes.rs
    - app/src/controllers/mcp.rs
    - app/src/controllers/auth_controller.rs
decisions:
  - "extract_bearer deleted from ferro-mcp-server (not replaced in-place); app/src/controllers/mcp.rs calls ferro_mcp_oauth::validate_bearer directly — ferro-mcp-server gains no new dependency"
  - "BearerCheck::Invalid → 401 with WWW-Authenticate: Bearer error=\"invalid_token\" (RFC 6750); BearerCheck::Forbidden → 403 bare (aud/tenant mismatch)"
  - "handlers module added to ferro-mcp-oauth/src/lib.rs to provide clean import path for routes.rs"
  - "oauth_return_to uses session().get + session_mut().forget (not remove — the actual SessionData method)"
metrics:
  duration: "672s"
  completed_date: "2026-06-10"
  tasks_completed: 2
  files_created: 0
  files_modified: 7
---

# Phase 199 Plan 05: Seam Wiring — OAuth Routes, Bearer Validation, Return-To Summary

Six OAuth routes mounted in the app, `/mcp` bearer seam filled with real JWT validation mapping `BearerCheck` → 401/403/proceed, DNS-rebinding Origin guard added, and the post-login `oauth_return_to` redirect closing the browser OAuth loop — all without adding any new dependency to `ferro-mcp-server`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2 | Delete extract_bearer; mount OAuth routes; real bearer seam + Origin check; return-to | cc783049 | ferro-mcp-server/src/auth.rs, ferro-mcp-server/src/lib.rs, ferro-mcp-oauth/src/lib.rs, app/Cargo.toml, app/src/routes.rs, app/src/controllers/mcp.rs, app/src/controllers/auth_controller.rs |
| fmt | cargo fmt across ferro-mcp-oauth + app (pre-existing style issues) | d2c9b876 | ferro-mcp-oauth/src/*.rs, app/src/controllers/mcp.rs, flow_integration.rs |

## Verification Results

- `cargo build -p ferro-mcp-server -p app` exits 0: CONFIRMED
- `cargo clippy --all --all-targets -- -D warnings` clean: CONFIRMED
- `cargo fmt --all -- --check` clean: CONFIRMED
- `cargo test -p app mcp` exits 0 (4 tests): CONFIRMED
- `grep -q 'BearerOutcome' ferro-mcp-server/src/auth.rs`: FOUND
- `! grep -q 'pub fn extract_bearer' ferro-mcp-server/src/auth.rs`: CONFIRMED
- `ferro-mcp-server/src/lib.rs` no longer re-exports `extract_bearer`: CONFIRMED
- `grep -q '/.well-known/oauth-protected-resource' app/src/routes.rs`: FOUND
- `grep -q 'ferro-mcp-oauth' app/Cargo.toml`: FOUND
- All six routes present in routes.rs: CONFIRMED
- `grep -q 'validate_bearer' app/src/controllers/mcp.rs`: FOUND
- `! grep -q 'extract_bearer' app/src/controllers/mcp.rs`: CONFIRMED
- `grep -q 'oauth_return_to' app/src/controllers/auth_controller.rs`: FOUND
- `cargo tree -p ferro-mcp-server | grep -c ferro-mcp-oauth` == 0: CONFIRMED (no new dep)

## Security Properties Verified

| Threat | Mitigation | Location | Test |
|--------|------------|----------|------|
| T-199-09 Tenant confusion | `expected_tenant = current_tenant().map(t.id)` → Forbidden(403) | mcp.rs validate_bearer call | validate.rs wrong_tenant_returns_forbidden (Plan 03) |
| T-199-08 Audience confusion | `aud` mismatch → BearerCheck::Forbidden → 403 | mcp.rs validate_bearer call | validate.rs wrong_audience_returns_forbidden (Plan 03) |
| T-199-401 Invalid/expired token | BearerCheck::Invalid → 401 + WWW-Authenticate: Bearer error="invalid_token" | mcp.rs | invalid_token_returns_401_invalid_token_header |
| T-199-15 DNS-rebinding (Origin) | Present+mismatched Origin → 403; absent → allowed | mcp.rs Origin guard | origin_mismatch_maps_to_403, absent_origin_is_allowed |
| T-199-13b Unconfigured secret | OAuthConfig::from_env() Err → 401 challenge (fail-closed) | mcp.rs OAuthConfig::from_env() | (structural: config.rs missing_secret_returns_err from Plan 01) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Task 1 and Task 2 could not be committed separately**
- **Found during:** Task 1 build verification
- **Issue:** `app/src/controllers/mcp.rs` still imported `extract_bearer` from `ferro-mcp-server`, causing a compile error after the re-export was removed. The two tasks had a compile-time dependency: Task 1 removed `extract_bearer` from `ferro-mcp-server`, Task 2 removed the import from `mcp.rs`. They were committed atomically.
- **Fix:** Executed both tasks before verifying, then committed as one unit.
- **Files modified:** all 7 task files in one commit (cc783049)
- **Commit:** cc783049

**2. [Rule 1 - Bug] Pre-existing cargo fmt violations in ferro-mcp-oauth**
- **Found during:** `cargo fmt --all -- --check` phase gate
- **Issue:** Several files in `ferro-mcp-oauth` (written in Plan 04) had formatting issues that `rustfmt` would reformat: multi-line `assert_eq!` collapsed to one line, multi-line `let` bindings collapsed, comment alignment.
- **Fix:** `cargo fmt --all` applied; no logic changes.
- **Files modified:** ferro-mcp-oauth/src/{authorize,config,consent,discovery,jwt,pkce,register,store,token,validate}.rs, tests/flow_integration.rs, app/src/controllers/mcp.rs, app/src/migrations/m20260611_create_oauth_clients_table.rs
- **Commit:** d2c9b876

**3. [Rule 2 - Missing export] handlers module needed in ferro-mcp-oauth**
- **Found during:** Task 1 implementation
- **Issue:** The PLAN and PATTERNS referenced `ferro_mcp_oauth::handlers::*` but no `handlers` module existed in `ferro-mcp-oauth/src/lib.rs`. The actual handler functions (`protected_resource_handler`, `authorization_server_handler`, `authorize_get`, `authorize_post`, `register_client`, `token_exchange`) were pub in their respective modules but not re-exported under a `handlers` namespace.
- **Fix:** Added `pub mod handlers { ... }` to `ferro-mcp-oauth/src/lib.rs` re-exporting all six handler functions.
- **Files modified:** `ferro-mcp-oauth/src/lib.rs`
- **Commit:** cc783049

**4. [Rule 1 - Bug] SessionData method is `forget`, not `remove`**
- **Found during:** Task 2 implementation of auth_controller.rs
- **Issue:** PATTERNS.md snippet used `s.remove("oauth_return_to")` but `SessionData` has no `remove` method — the correct method is `s.forget("oauth_return_to")`.
- **Fix:** Used `s.forget("oauth_return_to")` in the `session_mut` closure.
- **Files modified:** `app/src/controllers/auth_controller.rs`
- **Commit:** cc783049

## Known Stubs

None. All implemented features are wired end-to-end:
- `/mcp` bearer validation: calls `validate_bearer` with real JWT decode
- Six OAuth routes: mounted and dispatching to fully-implemented handlers (Plans 02+04)
- `oauth_return_to`: session-based redirect on login success
- `BearerOutcome` in `ferro-mcp-server`: kept for type contract (Phase 200 may extend it)

The `BearerCheck::Authenticated(_principal)` arm in `mcp.rs` discards the principal for now — Phase 200 inserts it into request extensions for `JwtClaimResolver`. This is intentional (the plan documents it with a comment) and not a stub: the flow proceeds to dispatch, which is the correct behavior.

## Threat Surface Scan

No new network endpoints introduced beyond what the plan's threat model covers. The six OAuth routes (`/.well-known/*`, `/register`, `/authorize` x2, `/token`) are all enumerated in Plans 02+04's threat registers. The `/mcp` endpoint's surface is unchanged — it narrows (validates rather than always-challenges).

No new threat flags.

## Self-Check: PASSED

- `ferro-mcp-server/src/auth.rs` exists, contains `BearerOutcome`, does NOT contain `extract_bearer`
- `ferro-mcp-server/src/lib.rs` does NOT re-export `extract_bearer`
- `ferro-mcp-oauth/src/lib.rs` contains `handlers` module
- `app/Cargo.toml` contains `ferro-mcp-oauth`
- `app/src/routes.rs` contains all six OAuth routes
- `app/src/controllers/mcp.rs` contains `validate_bearer`, `BearerCheck`, Origin guard, does NOT contain `extract_bearer`
- `app/src/controllers/auth_controller.rs` contains `oauth_return_to`
- Commits cc783049, d2c9b876 verified in git log
- `cargo test -p app mcp` passes 4 tests
- `cargo clippy --all --all-targets -- -D warnings` clean
- `cargo fmt --all -- --check` clean
