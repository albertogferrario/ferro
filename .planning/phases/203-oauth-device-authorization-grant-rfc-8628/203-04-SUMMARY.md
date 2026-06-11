---
phase: 203-oauth-device-authorization-grant-rfc-8628
plan: "04"
subsystem: ferro-mcp-oauth
tags: [oauth, device-grant, rfc-8628, token, state-machine, jwt]
dependency_graph:
  requires:
    - DeviceGrant / DeviceGrantStatus / device_cache_key / usercode_cache_key (203-01)
  provides:
    - token_exchange grant_type dispatch (authorization_code + device_code arms)
    - token_exchange_device_code RFC 8628 §3.5 state machine
    - SC-3 device-arm unit tests (8 tests)
  affects:
    - ferro-mcp-oauth/src/token.rs (modified)
    - ferro-mcp-oauth/src/device.rs (fmt-only)
    - ferro-mcp-oauth/src/lib.rs (fmt-only)
tech_stack:
  added: []
  patterns:
    - grant_type match dispatch (authorization_code / device_code / unsupported)
    - RFC 8628 §3.5 state machine (expired_token / slow_down / authorization_pending / access_denied / access_token)
    - one-token-issuer invariant: build_claims + mint_token called identically in both arms
    - single-use via Cache::forget on both keys at Approved (T-199-02 discipline)
    - explicit created_at TTL guard (now - created_at > 600) independent of cache TTL
    - ENV_LOCK mutex scoped before await points (clippy::await_holding_lock)
key_files:
  created: []
  modified:
    - ferro-mcp-oauth/src/token.rs
decisions:
  - TokenRequest fields (code/redirect_uri/code_verifier/device_code) made Option<String> with #[serde(default)] so device requests don't fail deserialization before grant_type dispatch (Pitfall 5)
  - token_exchange_dispatch extracted as inner async fn so tests can call it without constructing a full ferro::Request
  - ENV vars (MCP_TOKEN_SECRET/APP_URL) set in a scoped block before first await to satisfy clippy::await_holding_lock
metrics:
  duration: "384s"
  completed: "2026-06-11"
  tasks_completed: 2
  files_created: 0
  files_modified: 1
---

# Phase 203 Plan 04: Device-Code Token Exchange + SC-3 State Machine Summary

**One-liner:** RFC 8628 §3.5 polling state machine in `POST /token` — `expired_token` / `slow_down` / `authorization_pending` / `access_denied` / minted JWT on `Approved` via the identical `build_claims + mint_token` call as the auth-code arm (one-token-issuer invariant).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | grant_type dispatch + optional TokenRequest fields | 0845a821 | token.rs |
| 2 | token_exchange_device_code state machine + identical-mint Approved arm | 0845a821 | token.rs |

Tasks 1 and 2 are in a single commit: the dispatch refactor and the device arm are inseparable — the stub required for Task 1 to compile is replaced by the full implementation in Task 2, and both together satisfy the plan's acceptance criteria. The TDD RED→GREEN→REFACTOR cycle was completed inline.

## What Was Built

`ferro-mcp-oauth/src/token.rs` received the following changes:

**TokenRequest struct** — `code`, `redirect_uri`, `code_verifier` changed from `String` to `Option<String>` with `#[serde(default)]`; `device_code: Option<String>` added with `#[serde(default)]`. `grant_type` and `client_id` remain required.

**token_exchange handler** — now delegates to `token_exchange_dispatch(form)` after parsing.

**token_exchange_dispatch** — `match form.grant_type.as_str()` branching to:
- `"authorization_code"` → `token_exchange_auth_code` (unchanged logic, extracted)
- `"urn:ietf:params:oauth:grant-type:device_code"` → `token_exchange_device_code`
- `_` → `400 unsupported_grant_type`

**token_exchange_auth_code** — the original auth-code body moved verbatim; optional fields unwrapped with `invalid_request` errors if absent.

**token_exchange_device_code** — RFC 8628 §3.5 state machine:
1. Require non-empty `device_code` field.
2. `Cache::get(device_cache_key)` — do NOT forget yet.
3. None → `expired_token`.
4. `now - grant.created_at > 600` → `expired_token` (explicit TTL guard, T-203-DEVICECODE-EXPIRY).
5. Match `grant.status`:
   - `Pending`: compute elapsed, update `last_polled_at` via `Cache::put`, then `elapsed < 5` → `slow_down`, else → `authorization_pending`.
   - `Denied` → `access_denied`.
   - `Approved`: `Cache::forget(device_cache_key)` + `Cache::forget(usercode_cache_key)` (both keys, T-203-DEVICECODE-REPLAY), then `build_claims(grant.user_id.expect(...), grant.tenant_id, &config.app_url, 3600)` + `mint_token` (identical call to auth-code arm, T-203-CLAIMS-DIVERGE).

**8 new tests** (all deterministic, no `sleep`):
- `token_exchange_unsupported_grant_returns_error`
- `device_grant_pending_returns_authorization_pending` (last_polled_at = now - 10s)
- `device_grant_slow_down_on_fast_poll` (last_polled_at = now)
- `device_grant_denied_returns_access_denied`
- `device_grant_expired_returns_expired_token` (created_at = now - 700)
- `device_grant_approved_returns_access_token` (asserts both cache keys forgotten)
- `device_grant_tenant_binding` (JWT decode confirms tenant_id = Some(7))
- `device_grant_token_claims_identical_to_auth_code` (one-issuer invariant assertion)

## Test Results

```
test token::tests::token_exchange_unsupported_grant_returns_error ... ok
test token::tests::device_grant_pending_returns_authorization_pending ... ok
test token::tests::device_grant_slow_down_on_fast_poll ... ok
test token::tests::device_grant_denied_returns_access_denied ... ok
test token::tests::device_grant_expired_returns_expired_token ... ok
test token::tests::device_grant_approved_returns_access_token ... ok
test token::tests::device_grant_tenant_binding ... ok
test token::tests::device_grant_token_claims_identical_to_auth_code ... ok
test result: ok. 77 passed; 0 failed; 0 ignored (full crate)
```

`cargo fmt --all -- --check` and `cargo clippy --all-targets -D warnings` both clean.

## Deviations from Plan

**1. [Rule 1 - Bug] `.body()` returns `&str` not `&[u8]`**
- **Found during:** Task 2 compilation
- **Issue:** Test code used `serde_json::from_slice(resp.body())` but `ferro::HttpResponse::body()` returns `&str`.
- **Fix:** Changed all test body-parsing calls to `serde_json::from_str(resp.body())`.
- **Files modified:** `ferro-mcp-oauth/src/token.rs`
- **Commit:** 0845a821 (fixed before commit)

**2. [Rule 1 - Bug] clippy::await_holding_lock in async tests**
- **Found during:** pre-commit clippy check
- **Issue:** `ENV_LOCK.lock()` guard held across `await` points in two async tests.
- **Fix:** Scoped the lock acquisition into a `{ }` block that drops the guard before the first `await`.
- **Files modified:** `ferro-mcp-oauth/src/token.rs`
- **Commit:** 0845a821 (fixed before commit)

**3. [Rule 2 - Missing] `token_exchange_dispatch` inner function for testability**
- **Found during:** Task 1 — plan specified testing dispatch behavior but `token_exchange` takes a `ferro::Request` making direct unit testing impractical.
- **Fix:** Extracted `async fn token_exchange_dispatch(form: TokenRequest) -> ferro::Response` as the inner testable unit; `token_exchange` parses then delegates.
- **Files modified:** `ferro-mcp-oauth/src/token.rs`
- **Commit:** 0845a821

## Threat Surface Scan

No new network endpoints added in this plan. The changes are internal to the existing `POST /token` handler:
- T-203-DEVICECODE-REPLAY: mitigated — `Cache::forget` on both keys before returning token.
- T-203-DEVICECODE-EXPIRY: mitigated — explicit `now - created_at > 600` guard.
- T-203-CLAIMS-DIVERGE: mitigated — `build_claims + mint_token` call is byte-identical to auth-code arm; asserted by `device_grant_token_claims_identical_to_auth_code`.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| ferro-mcp-oauth/src/token.rs | FOUND |
| commit 0845a821 | FOUND |
| `token_exchange_auth_code` in token.rs | FOUND |
| `token_exchange_device_code` in token.rs | FOUND |
| `urn:ietf:params:oauth:grant-type:device_code` in token.rs | FOUND |
| `Cache::forget` (both keys) in device arm | FOUND (8 occurrences) |
| `build_claims(` with `&config.app_url, 3600` | FOUND |
| All 8 device-arm tests green | PASSED |
| Full crate: 77/77 tests pass | PASSED |
