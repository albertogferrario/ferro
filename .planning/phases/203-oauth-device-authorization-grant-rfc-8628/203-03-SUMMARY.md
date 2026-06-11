---
phase: 203-oauth-device-authorization-grant-rfc-8628
plan: "03"
subsystem: ferro-mcp-oauth
tags: [oauth, device-grant, rfc-8628, handlers, csrf, tenant-binding, login-resume]
dependency_graph:
  requires:
    - DeviceGrant/DeviceGrantStatus/cache-key helpers/code primitives (203-01)
  provides:
    - device_authorization handler (POST /device_authorization, public)
    - device_verification_get handler (GET /device, session+tenant group)
    - device_verification_post handler (POST /device, session+tenant group)
    - approve_device_grant(device_code, user_id, tenant_id) cache helper
    - deny_device_grant(device_code) cache helper
    - render_confirm_html / render_code_entry_form / render_terminal_page HTML helpers
    - device_authorization_body(device_code, display_user_code, app_url) pure fn
    - handlers re-export in lib.rs (device_authorization, device_verification_get, device_verification_post)
  affects:
    - ferro-mcp-oauth/src/device.rs (extended with handlers + helpers)
    - ferro-mcp-oauth/src/lib.rs (handlers re-export updated)
tech_stack:
  added: []
  patterns:
    - find_by_client_id for client validation (mirrors authorize.rs lines 105-120)
    - store_oauth_return_to + 302 /auth/login for unauthenticated redirect (mirrors authorize.rs lines 88-102)
    - get_csrf_token() + subtle::ConstantTimeEq ct_eq for CSRF (mirrors consent.rs lines 133-141)
    - Auth::id() + current_tenant() capture at approve time (mirrors consent.rs lines 191-201)
    - Cache::put overwrite for state transitions (mirrors consent.rs lines 219-232)
    - sanitized_app_url() for verification_uri construction (mirrors discovery.rs lines 44-45)
    - Raw HTML rendering with html_escape (mirrors consent.rs / authorize.rs)
key_files:
  created: []
  modified:
    - ferro-mcp-oauth/src/device.rs
    - ferro-mcp-oauth/src/lib.rs
decisions:
  - device_authorization_body factored as pure fn for testability without live DB (avoids needing registered client in unit test)
  - DeviceGrantError enum with Cache(FrameworkError) variant for approve/deny helpers (FrameworkError is correct cache error type, not ferro::CacheError which does not exist)
  - Code-entry POST uses PRG (redirect to GET /device?user_code=...) so browser re-fetches confirm page with fresh CSRF token
  - render_code_entry_form uses .to_string() on a string literal to satisfy clippy::useless_format
metrics:
  duration: "430s"
  completed: "2026-06-11"
  tasks_completed: 2
  files_created: 0
  files_modified: 2
---

# Phase 203 Plan 03: Device Authorization Handlers Summary

**One-liner:** RFC 8628 device flow HTTP handlers — `POST /device_authorization` with client validation + RFC §3.2 JSON, `GET /device` with login-resume + confirm+consent HTML, `POST /device` with CSRF `ct_eq` + `Auth::id()`/`current_tenant()` binding at approve time.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | device_authorization handler — client validation + code issuance + RFC §3.2 response | ee72407f | device.rs, lib.rs |
| 2 | device_verification_get + device_verification_post handlers (login-resume, CSRF, user/tenant binding) | ee72407f | device.rs |

Note: Tasks 1 and 2 committed together because both were implemented in a single pass on `device.rs`; all tests were green before the commit.

## What Was Built

### Task 1: `device_authorization` handler

`POST /device_authorization` (public endpoint, no session required):

1. Parses `client_id` from form body (`DeviceAuthRequest` struct).
2. Validates via `find_by_client_id` — unknown client → `400 {"error":"invalid_client"}` (T-203-INVALID-CLIENT).
3. Generates `device_code` via `generate_device_code()` (256-bit URL-safe random, identical to auth codes) and `display_user_code` via `generate_user_code()` (RFC §6.1 charset, `XXXX-XXXX`).
4. Stores `DeviceGrant{Pending, user_id:None, tenant_id:None}` under `mcp:device:{device_code}` with 600s TTL.
5. Stores `device_code` pointer under `mcp:usercode:{normalized_user_code}` with same TTL.
6. Returns JSON with exactly the 6 RFC §3.2 fields: `device_code`, `user_code`, `verification_uri` (`{app_url}/device`), `verification_uri_complete` (`{app_url}/device?user_code={encoded}`), `expires_in=600`, `interval=5`.

Helper `device_authorization_body(device_code, display_user_code, app_url) -> Value` is pure and unit-testable without a live DB row.

### Task 2: `device_verification_get` + `device_verification_post` handlers

**`GET /device`** (session+tenant group, `TenantFailureMode::Allow`):
- Unauthenticated: calls `crate::resume::store_oauth_return_to("/device?user_code={encoded}")`, redirects to `/auth/login` (T-203-OPEN-REDIRECT: return_url is handler-constructed, not from raw user input).
- Authenticated + valid `user_code` query param: normalizes → resolves `mcp:usercode:{normalized}` → reads `mcp:device:{dc}` → dispatches on grant status:
  - `None`/expired → error page "Code Expired".
  - `Approved`/`Denied` → terminal page "Code Already Used".
  - `Pending` → looks up client name via `find_by_client_id`, reads CSRF token via `get_csrf_token()`, renders confirm+consent HTML with hidden `_token`, hidden `device_code`, approve/deny buttons (T-203-XSS: `client_name` HTML-escaped).
- No/invalid `user_code` → code-entry form (`<form method="post" action="/device">`, `name="user_code"` input).

**`POST /device`** (`DeviceVerifyForm` with `_token`, `action`, `device_code`, `user_code`):
- Code-entry path (`user_code` present, `device_code` absent): PRG redirect to `GET /device?user_code={encoded}`.
- Approve/deny path:
  - CSRF: `get_csrf_token()` → `ct_eq` constant-time compare → 400 on mismatch (T-203-CSRF).
  - `action == "deny"`: calls `deny_device_grant(device_code)` → cache overwrite with `status=Denied` → terminal "Access Denied" page.
  - `action == "approve"`: `user_id = Auth::id()` (401 if absent), `tenant_id = current_tenant().map(|t| t.id)` — both from session, never form (T-203-TENANT-BYPASS) → calls `approve_device_grant(device_code, user_id, tenant_id)` → cache overwrite with `status=Approved, user_id, tenant_id` → terminal "Access Approved" page.

**`approve_device_grant` / `deny_device_grant`**: `pub(crate)` async helpers that do cache get → modify → put overwrite. Return `DeviceGrantError::NotFound` when grant is absent/expired. Testable without a session.

## Test Results

```
test device::tests::device_authorization_response_fields ... ok
test device::tests::device_authorization_body_uri_complete_encodes_user_code ... ok
test device::tests::device_verification_confirm_html_contains_required_fields ... ok
test device::tests::device_verification_code_entry_form_structure ... ok
test device::tests::device_verification_confirm_html_escapes_client_name ... ok
test device::tests::device_verification_binds_user_and_tenant ... ok
test device::tests::device_verification_deny_sets_denied_status ... ok
test result: ok. 69 passed; 0 failed; 0 ignored (all ferro-mcp-oauth unit tests)
test full_pkce_flow ... ok (integration test)
```

`cargo fmt --check` and `cargo clippy --all-targets -D warnings` both clean.

## Deviations from Plan

**1. [Rule 1 - Bug] `ferro::CacheError` does not exist — used `ferro::FrameworkError`**
- **Found during:** Build attempt after writing `approve_device_grant`
- **Issue:** Plan referenced `ferro::CacheError::Miss` but the correct type is `ferro::FrameworkError` (returned by `Cache::get`/`Cache::put`). Cache miss is `Ok(None)`, not an `Err`.
- **Fix:** Defined `DeviceGrantError { NotFound, Cache(FrameworkError) }` with `From<FrameworkError>` impl. `None` from `Cache::get` maps to `DeviceGrantError::NotFound`.
- **Files modified:** `ferro-mcp-oauth/src/device.rs`
- **Commit:** ee72407f (fixed before commit)

**2. [Rule 1 - Bug] clippy::useless_format on `render_code_entry_form`**
- **Found during:** `cargo clippy -D warnings` after formatting
- **Issue:** `format!(r#"..."#)` with no interpolation is a useless format call.
- **Fix:** Changed to `r#"..."#.to_string()`.
- **Files modified:** `ferro-mcp-oauth/src/device.rs`
- **Commit:** ee72407f (fixed before commit)

No other deviations. Plan executed as written.

## Threat Surface Scan

New network endpoints introduced:
- `POST /device_authorization` (public): client_id validation via `find_by_client_id` (T-203-INVALID-CLIENT mitigated).
- `GET /device` (session+tenant group): unauthenticated path uses `store_oauth_return_to` with handler-constructed URL (T-203-OPEN-REDIRECT mitigated); HTML renders escape untrusted strings (T-203-XSS mitigated).
- `POST /device` (session+tenant group): CSRF `ct_eq` (T-203-CSRF mitigated); tenant from session (T-203-TENANT-BYPASS mitigated).

All threats from the plan's threat register are addressed:
| Threat | Status |
|--------|--------|
| T-203-INVALID-CLIENT | Mitigated — `find_by_client_id` before code issuance |
| T-203-CSRF | Mitigated — `get_csrf_token()` + `ct_eq` on POST |
| T-203-TENANT-BYPASS | Mitigated — `current_tenant()` from session, never form |
| T-203-OPEN-REDIRECT | Mitigated — return_url constructed by handler |
| T-203-XSS | Mitigated — `html_escape` on `client_name` and all untrusted strings |
| T-203-DEVICECODE-FORM-LEAK | Accepted — device_code already transmitted to polling device over TLS |

## Self-Check: PASSED

| Item | Status |
|------|--------|
| ferro-mcp-oauth/src/device.rs | FOUND |
| ferro-mcp-oauth/src/lib.rs | FOUND |
| commit ee72407f | FOUND |
| `device_authorization` in device.rs | FOUND |
| `verification_uri_complete` in device.rs | FOUND |
| `find_by_client_id` in device.rs | FOUND |
| `invalid_client` in device.rs | FOUND |
| `sanitized_app_url` in device.rs | FOUND |
| `store_oauth_return_to` in device.rs | FOUND |
| `/auth/login` in device.rs | FOUND |
| `get_csrf_token` in device.rs | FOUND |
| `ct_eq` in device.rs | FOUND |
| `current_tenant()` in device.rs | FOUND |
| `Auth::id()` in device.rs | FOUND |
| `approve_device_grant` in device.rs | FOUND |
| `DeviceGrantStatus::Approved` in device.rs | FOUND |
| `name="device_code"` hidden field in confirm HTML | FOUND |
| 69 unit tests pass | VERIFIED |
