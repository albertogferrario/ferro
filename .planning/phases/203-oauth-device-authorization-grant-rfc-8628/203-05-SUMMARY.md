---
phase: 203-oauth-device-authorization-grant-rfc-8628
plan: "05"
subsystem: ferro-mcp-oauth
tags: [oauth, device-grant, rfc-8628, routes, docs, gate]
dependency_graph:
  requires:
    - device_authorization / device_verification_get / device_verification_post handlers (203-03)
    - token_exchange_device_code state machine + SC-3 tests (203-04)
    - discovery device endpoint + grant type + tests (203-02)
    - DeviceGrant / DeviceGrantStatus / primitives (203-01)
  provides:
    - POST /device_authorization mounted public in app/src/routes.rs
    - GET+POST /device mounted under SessionUserTenantResolver + TenantFailureMode::Allow
    - MCP OAuth Authorization Server feature docs (docs/src/features/mcp-oauth.md)
    - All 12 SC-5 matrix tests green
    - Full workspace gate (fmt + clippy --all-targets + test --all-features) green
  affects:
    - app/src/routes.rs (device routes mounted)
    - docs/src/features/mcp-oauth.md (new feature doc)
    - docs/src/SUMMARY.md (new entry)
tech_stack:
  added: []
  patterns:
    - public endpoint mount pattern (like /register and /token — no session)
    - session+tenant group pattern (like /authorize — TenantFailureMode::Allow)
    - ferro_mcp_oauth::handlers import extension
key_files:
  created:
    - docs/src/features/mcp-oauth.md
  modified:
    - app/src/routes.rs
    - docs/src/SUMMARY.md
decisions:
  - All 12 SC-5 matrix tests already existed from Plans 01/03/04 — no new tests needed
  - lib.rs handlers re-exports already present from Plan 03 — no lib.rs changes needed
  - discovery.rs device fields already present from Plan 02 — no discovery.rs changes needed
metrics:
  duration: "673s"
  completed: "2026-06-11"
  tasks_completed: 3
  files_created: 1
  files_modified: 2
---

# Phase 203 Plan 05: Framework Wiring + Full Gate Summary

**One-liner:** Device routes mounted public (`POST /device_authorization`) and session-tenant (`GET`/`POST /device` under `SessionUserTenantResolver` + `TenantFailureMode::Allow`), MCP OAuth feature doc created, all 12 SC-5 matrix tests confirmed present and green, workspace gate (fmt + clippy `--all-targets` + test `--all-features`) passes clean.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Export device handlers + mount routes + docs | f297ef05 | app/src/routes.rs, docs/src/features/mcp-oauth.md, docs/src/SUMMARY.md |
| 2 | Verify full SC-5 polling test matrix (12 named tests) | — (no new tests needed) | ferro-mcp-oauth/src/token.rs, device.rs, discovery.rs |
| 3 | BLOCKING workspace gate (fmt + clippy --all-targets + test --all-features) | — (gate-only, no source edits) | — |

## What Was Built

### Task 1: Route mounts + docs

**`app/src/routes.rs`** extended with:
- Import: `device_authorization`, `device_verification_get`, `device_verification_post` added to the `ferro_mcp_oauth::handlers::{...}` import block.
- `post!("/device_authorization", device_authorization)` — public, no session (T-203-MOUNT-PUBLIC accepted: client_id validated in-handler per D-06).
- `group!("/", { get!("/device", ...), post!("/device", ...) }).middleware(TenantMiddleware::new().resolver(SessionUserTenantResolver::new()).on_failure(TenantFailureMode::Allow))` — session+tenant group (T-203-MOUNT-TENANT mitigated: `current_tenant()` is real at approve time, parity with `/authorize`).

**`docs/src/features/mcp-oauth.md`** — new feature doc covering:
- Quick-start mount example for both flows.
- Authorization-code flow (5-step summary).
- Device Authorization Grant (RFC 8628): Step 1 request/response with all 6 §3.2 fields, Step 2 user verification flow, Step 3 polling table with all 5 §3.5 outcomes.
- Token identity: one-issuer invariant documented.
- Discovery metadata: full JSON example with `device_authorization_endpoint` and device grant type.
- `validate_bearer` usage.
- Security notes (CSRF, tenant binding, single-use codes, entropy, rate limiting deferral).

**`docs/src/SUMMARY.md`** — "MCP OAuth Authorization Server" entry added under Authentication.

### Task 2: SC-5 matrix verification

All 12 SC-5 matrix tests were already written in prior plans:

| Test | Location | Status |
|------|----------|--------|
| `device_grant_pending_returns_authorization_pending` | token.rs | green |
| `device_grant_approved_returns_access_token` | token.rs | green |
| `device_grant_expired_returns_expired_token` | token.rs | green |
| `device_grant_slow_down_on_fast_poll` | token.rs | green |
| `device_grant_denied_returns_access_denied` | token.rs | green |
| `device_grant_tenant_binding` | token.rs | green |
| `device_grant_token_claims_identical_to_auth_code` | token.rs | green |
| `device_authorization_response_fields` | device.rs | green |
| `user_code_normalization_strips_hyphen_and_case` | device.rs | green |
| `user_code_format_is_xxxx_hyphen_xxxx` | device.rs | green |
| `discovery_advertises_device_authorization_endpoint` | discovery.rs | green |
| `discovery_advertises_device_grant_type` | discovery.rs | green |

No new test code required; all 12 pass under `cargo test -p ferro-mcp-oauth`.

### Task 3: Workspace gate

The exact CLAUDE.md gate command passed end to end:
- `cargo fmt --all -- --check` — clean (no diffs)
- `cargo clippy --all --all-targets -- -D warnings` — clean (no warnings)
- `cargo test --all-features` — 121 test suites, all `ok`

Post-gate: `docs/protocol/schemas/*.json` churn from the Phase 94 export test was discarded via `git checkout -- docs/protocol/schemas/` (not folded into any phase commit).

## Deviations from Plan

**1. lib.rs handlers re-exports already present (no change needed)**
- **Found during:** Initial read of `ferro-mcp-oauth/src/lib.rs`
- **Note:** Plan 03's SUMMARY shows lib.rs handlers block was updated in commit `ee72407f`. Plan 05's task description stated "add three re-exports" but they were already present. Verified and no action needed.
- **Impact:** Zero — acceptance criteria satisfied by prior work.

**2. discovery.rs device fields already present (no change needed)**
- **Found during:** Initial read of `ferro-mcp-oauth/src/discovery.rs`
- **Note:** Plan 02 already added `device_authorization_endpoint` and device grant type to the discovery metadata, along with both discovery tests.
- **Impact:** Zero — acceptance criteria satisfied by prior work.

No regressions. Plan executed with all work already performed by prior-wave plans.

## Threat Surface Scan

New network endpoints mounted:
- `POST /device_authorization`: public, client_id validated in-handler (T-203-MOUNT-PUBLIC accepted per plan threat register).
- `GET /device` + `POST /device`: under `SessionUserTenantResolver` + `TenantFailureMode::Allow` — `current_tenant()` real at approve time (T-203-MOUNT-TENANT mitigated).

No new threat surface beyond what was analyzed in the plan's threat register. No additional threat flags.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| app/src/routes.rs | FOUND |
| docs/src/features/mcp-oauth.md | FOUND |
| docs/src/SUMMARY.md | FOUND |
| commit f297ef05 | FOUND |
| `device_authorization` in routes.rs | FOUND |
| `post!("/device_authorization"` in routes.rs | FOUND |
| `get!("/device"` in routes.rs | FOUND |
| `post!("/device"` in routes.rs | FOUND |
| `TenantFailureMode::Allow` in /device group | FOUND |
| `device_authorization` in lib.rs handlers | FOUND |
| `device_authorization_endpoint` in discovery.rs | FOUND |
| All 12 SC-5 matrix tests pass | VERIFIED |
| Workspace gate (fmt + clippy + test) | GREEN |
