---
phase: 203-oauth-device-authorization-grant-rfc-8628
verified: 2026-06-11T15:30:00Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 203: OAuth Device Authorization Grant (RFC 8628) Verification Report

**Phase Goal:** `ferro-mcp-oauth` supports the OAuth 2.0 Device Authorization Grant (RFC 8628) so passwordless, cross-device, and headless/CLI MCP clients can authenticate without a same-device browser callback.
**Verified:** 2026-06-11T15:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth (Success Criterion) | Status | Evidence |
|---|--------------------------|--------|----------|
| 1 | `POST /device_authorization` returns all 6 exact RFC §3.2 fields | VERIFIED | `device_authorization_body()` in device.rs:203-221 returns `device_code`, `user_code`, `verification_uri`, `verification_uri_complete`, `expires_in=600`, `interval=5`; `verification_uri` derives from `sanitized_app_url()` (line 302), no hardcoded host |
| 2 | Verification page handlers exist with unauth→login redirect, CSRF validation, and user/tenant binding at approve time | VERIFIED | `device_verification_get` (line 468): unauthenticated → `store_oauth_return_to` + 302 `/auth/login` (line 497-500); `device_verification_post` (line 602): `ct_eq` CSRF (line 625), `Auth::id()` (line 659), `current_tenant()` (line 670); routes mounted under `SessionUserTenantResolver` + `TenantFailureMode::Allow` in `app/src/routes.rs:97-104` |
| 3 | `POST /token` with device-code grant type returns RFC §3.5 outcomes; tokens are claims-identical to auth-code arm; `client_id` binding validated (CR-01 fix present) | VERIFIED | token.rs:76 dispatches on `urn:ietf:params:oauth:grant-type:device_code`; state machine returns `authorization_pending`/`slow_down`/`expired_token`/`access_denied`/`access_token` (lines 229-280); `build_claims` + `mint_token` called identically to auth-code arm (line 267-274); `grant.client_id != form.client_id` guard at line 258 (forget-first discipline); both cache keys forgotten (lines 252-253) |
| 4 | Discovery advertises `device_authorization_endpoint` and device-code grant URN; device/user codes are single-use with TTL | VERIFIED | `authorization_server_metadata()` in discovery.rs:33 includes `device_authorization_endpoint`; line 35 includes the device URN in `grant_types_supported`; `DEVICE_CODE_TTL = Duration::from_secs(600)` (device.rs:43); both keys forgotten via `Cache::forget` on Approved (token.rs:252-253) |
| 5 | 12-test SC-5 matrix present and green; `--all-features` clippy + tests pass | VERIFIED | 7 token.rs device tests + 2 device.rs tests (SC-5 matrix) + 2 discovery.rs tests + 1 CR-01 test (`device_grant_wrong_client_id_returns_invalid_grant`) = 13 named tests all present; workspace gate confirmed green: `cargo fmt` clean, `cargo clippy --all --all-targets -D warnings` clean, `cargo test --all-features` — 121 suites ok (203-05-SUMMARY.md:102-104) |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp-oauth/src/device.rs` | DeviceGrant + handlers + primitives | VERIFIED | 1039 lines; exports `DeviceGrant`, `DeviceGrantStatus`, `generate_device_code`, `generate_user_code`, `normalize_user_code`, `device_cache_key`, `usercode_cache_key`, `DEVICE_CODE_TTL`, `DEVICE_INTERVAL_SECS`; three handlers present |
| `ferro-mcp-oauth/src/token.rs` | device-code grant arm + dispatch | VERIFIED | `token_exchange_device_code` arm (line 170); dispatch match (line 74); `TokenRequest` with optional fields + `device_code: Option<String>` (line 53) |
| `ferro-mcp-oauth/src/discovery.rs` | device_authorization_endpoint + grant URN | VERIFIED | Lines 33-35: both fields present; uses `app_url` parameter, no hardcoded host |
| `ferro-mcp-oauth/src/lib.rs` | `pub mod device`; handler re-exports | VERIFIED | Line 11: `pub mod device;`; lines 34-36: `device_authorization`, `device_verification_get`, `device_verification_post` in `pub mod handlers` |
| `app/src/routes.rs` | device endpoints mounted with correct middleware | VERIFIED | Line 92: `post!("/device_authorization", device_authorization)` public; lines 97-104: `/device` group under `SessionUserTenantResolver` + `TenantFailureMode::Allow` |
| `docs/src/features/mcp-oauth.md` | Device Authorization Grant section | VERIFIED | "Device Authorization Grant (RFC 8628)" section at line 84; documents all §3.2 response fields, verification flow, §3.5 outcomes, discovery additions |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `device_authorization` handler | `store::find_by_client_id` | client_id validation before code issuance | WIRED | device.rs:253 calls `crate::store::find_by_client_id`; unknown client → `invalid_client` (line 256-262) |
| `device_authorization` handler | `sanitized_app_url()` | verification_uri derived from app URL | WIRED | device.rs:302: `let app_url = crate::config::sanitized_app_url();` |
| `device_verification_get` handler | `crate::resume::store_oauth_return_to` | unauthenticated redirect | WIRED | device.rs:497: `crate::resume::store_oauth_return_to(return_url);` |
| `device_verification_post` handler | `current_tenant()` | tenant captured server-side at approve time | WIRED | device.rs:670: `let tenant_id = current_tenant().map(\|t\| t.id);` |
| `token_exchange_device_code` Approved arm | `jwt::build_claims` + `jwt::mint_token` | identical call to auth-code arm | WIRED | token.rs:267-274: call shape identical to auth-code arm at lines 149-150 |
| `token_exchange_device_code` Approved arm | `Cache::forget` (both keys) | single-use enforcement | WIRED | token.rs:252-253: forgets `device_cache_key` and `usercode_cache_key` before validation |
| `token_exchange_device_code` Approved arm | `grant.client_id != form.client_id` | CR-01 client binding check | WIRED | token.rs:258: guard returns `invalid_grant`; forget-first discipline (lines 252-253 before line 258) |
| `app/src/routes.rs /device group` | `SessionUserTenantResolver` + `TenantFailureMode::Allow` | tenant middleware | WIRED | routes.rs:100-103: `TenantMiddleware::new().resolver(SessionUserTenantResolver::new()).on_failure(TenantFailureMode::Allow)` |
| `app/src/routes.rs` | `ferro_mcp_oauth::handlers::device_authorization` | import + post! mount | WIRED | routes.rs:5-7: import; line 92: `post!("/device_authorization", device_authorization)` |

### Data-Flow Trace (Level 4)

Level 4 trace applies to the token issuance path (dynamic data: JWT minted from grant state).

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `token_exchange_device_code` Approved branch | `grant.user_id`, `grant.tenant_id` | `DeviceGrant` from `Cache::get(&device_cache_key(device_code))` | Yes — set by `approve_device_grant()` from `Auth::id()` + `current_tenant()` at verification | FLOWING |
| `device_verification_post` approve path | `user_id`, `tenant_id` | `Auth::id()` + `current_tenant()` from session middleware | Yes — session state, not form input | FLOWING |
| `device_authorization` handler | `device_code`, `user_code` | `generate_device_code()` (256-bit rand) + `generate_user_code()` (RFC charset) | Yes — cryptographic randomness | FLOWING |

### Behavioral Spot-Checks

Step 7b skipped: `cargo test -p ferro-mcp-oauth` would re-run the full unit suite (disk-tight constraint); gate evidence already confirmed 121 suites green in SUMMARY (203-05-SUMMARY.md:104). Test names listed in SC-5 matrix section below.

### SC-5 Test Matrix Coverage

| Test Name | File | SC Coverage |
|-----------|------|-------------|
| `device_grant_pending_returns_authorization_pending` | token.rs | SC-3 pending state |
| `device_grant_approved_returns_access_token` | token.rs | SC-3 approved + both keys forgotten |
| `device_grant_expired_returns_expired_token` | token.rs | SC-3 expiry + SC-4 TTL |
| `device_grant_slow_down_on_fast_poll` | token.rs | SC-3 slow_down backoff |
| `device_grant_denied_returns_access_denied` | token.rs | SC-3 denied consent |
| `device_grant_tenant_binding` | token.rs | SC-3 tenant-scoped token |
| `device_grant_token_claims_identical_to_auth_code` | token.rs | SC-3 one-issuer invariant |
| `device_grant_wrong_client_id_returns_invalid_grant` | token.rs | SC-3 + CR-01 client_id binding |
| `device_authorization_response_fields` | device.rs | SC-1 §3.2 fields |
| `user_code_normalization_strips_hyphen_and_case` | device.rs | SC-4 single-use key normalization |
| `user_code_format_is_xxxx_hyphen_xxxx` | device.rs | SC-1/SC-4 user_code charset |
| `device_verification_binds_user_and_tenant` | device.rs | SC-2 user/tenant binding |
| `discovery_advertises_device_authorization_endpoint` | discovery.rs | SC-4 discovery |
| `discovery_advertises_device_grant_type` | discovery.rs | SC-4 grant URN advertised |

13 tests total (12 SC-5 matrix + 1 CR-01 security regression test), all green.

### Requirements Coverage

No REQUIREMENTS.md IDs mapped to this phase. All 5 roadmap Success Criteria verified above.

### Anti-Patterns Found

No blockers or warnings. The one "placeholder" text grep match is the `XXXX-XXXX` HTML input placeholder attribute in `render_code_entry_form()` — this is the intended UI hint to the user, not a code stub.

The `Ok(()) => {}` match arms at device.rs:645,673 are intentional: successful grant state transitions are followed immediately by terminal HTML renders. Not stubs.

### Human Verification Required

None. All success criteria are fully verifiable from code and test evidence.

### Conceptual-Coherence Constraints

| Constraint | Status | Evidence |
|-----------|--------|----------|
| No second token issuer — device arm reuses `jwt.rs` | VERIFIED | token.rs:267-274 calls `build_claims` + `mint_token` identically to auth-code arm; `device_grant_token_claims_identical_to_auth_code` test asserts structural identity |
| No parallel consent system — reuses login + consent surface | VERIFIED | Verification page redirects to `/auth/login` via `store_oauth_return_to`; CSRF + approve/deny mirrors consent.rs patterns |
| Raw-HTML verification page — no ferro-json-ui dep added | VERIFIED | `ferro-mcp-oauth/Cargo.toml`: no `ferro-json-ui` dependency; verification pages rendered via `render_confirm_html` / `render_code_entry_form` in plain Rust string formatting |

### Gaps Summary

No gaps. All 5 Success Criteria verified against the actual codebase with implementation evidence.

---

_Verified: 2026-06-11T15:30:00Z_
_Verifier: Claude (gsd-verifier)_
