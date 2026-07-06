---
phase: 203-oauth-device-authorization-grant-rfc-8628
plan: "01"
subsystem: ferro-mcp-oauth
tags: [oauth, device-grant, rfc-8628, cache, primitives]
dependency_graph:
  requires: []
  provides:
    - DeviceGrant ephemeral cache record (ferro-mcp-oauth/src/device.rs)
    - DeviceGrantStatus enum (Pending/Approved/Denied, snake_case serde)
    - device_cache_key / usercode_cache_key helpers
    - generate_device_code / generate_user_code / normalize_user_code primitives
    - DEVICE_CODE_TTL / DEVICE_INTERVAL_SECS constants
  affects:
    - ferro-mcp-oauth/src/lib.rs (pub mod device added)
tech_stack:
  added: []
  patterns:
    - OAuthCode analog pattern (store type mirrors store::OAuthCode)
    - pkce::generate_auth_code delegation for device_code
    - RFC 8628 §6.1 charset (BCDFGHJKLMNPQRSTVWXZ) for user_code generation
key_files:
  created:
    - ferro-mcp-oauth/src/device.rs
  modified:
    - ferro-mcp-oauth/src/lib.rs
decisions:
  - D-01: DeviceGrant stored in ferro-cache (not DB); two cache keys per grant
  - D-02: user_code from RFC 8628 §6.1 charset, 8-char XXXX-XXXX format
  - normalized_user_code field on DeviceGrant so token handler can forget usercode pointer key
metrics:
  duration: "188s"
  completed: "2026-06-11"
  tasks_completed: 2
  files_created: 1
  files_modified: 1
---

# Phase 203 Plan 01: DeviceGrant Foundation Module Summary

**One-liner:** RFC 8628 device grant substrate — `DeviceGrant` cache record with two-key layout, `XXXX-XXXX` user code from the 20-char consonant charset, and `generate_device_code` delegating to the existing 256-bit PKCE entropy path.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | DeviceGrant record + DeviceGrantStatus enum + cache-key constants | 0b49f5f5 | device.rs (created), lib.rs (modified) |
| 2 | user_code / device_code generation + normalization primitives | 0b49f5f5 | device.rs |

Note: Tasks 1 and 2 are implemented in a single commit because the file was written in one pass with both the struct/constants and the generation functions; the TDD RED→GREEN cycle was completed inline.

## What Was Built

`ferro-mcp-oauth/src/device.rs` is the substrate module for RFC 8628. It provides:

- **`DeviceGrantStatus`** — `#[serde(rename_all = "snake_case")]` enum with `Pending`, `Approved`, `Denied`.
- **`DeviceGrant`** — cache record mirroring `OAuthCode`: `client_id`, `status`, `user_id: Option<i64>`, `tenant_id: Option<i64>`, `created_at: i64`, `last_polled_at: Option<i64>`, `normalized_user_code: String`. Rustdoc explains the two-key cache layout, status transitions, and the `normalized_user_code` field purpose.
- **`DEVICE_CODE_TTL`** (`Duration::from_secs(600)`) and **`DEVICE_INTERVAL_SECS`** (`5i64`).
- **`USER_CODE_CHARSET`** — `b"BCDFGHJKLMNPQRSTVWXZ"` (RFC 8628 §6.1, private constant).
- **`device_cache_key(device_code)`** → `"mcp:device:{device_code}"`.
- **`usercode_cache_key(normalized_user_code)`** → `"mcp:usercode:{normalized_user_code}"`.
- **`generate_device_code()`** — delegates to `crate::pkce::generate_auth_code()` (256-bit URL-safe random); no reimplementation (T-203-DEVICECODE-ENTROPY mitigated).
- **`generate_user_code()`** — samples 8 chars uniformly from `USER_CODE_CHARSET` via `rand::thread_rng().gen_range`, formats as `XXXX-XXXX`.
- **`normalize_user_code(input)`** — `input.to_uppercase().chars().filter(|c| *c != '-' && *c != ' ').collect()`.

`lib.rs` gains `pub mod device;` in the existing module block.

## Test Results

```
test device::tests::device_grant_serde_roundtrip ... ok
test device::tests::device_grant_status_serializes_snake_case ... ok
test device::tests::user_code_format_is_xxxx_hyphen_xxxx ... ok
test device::tests::user_code_normalization_strips_hyphen_and_case ... ok
test device::tests::device_code_is_url_safe_nonempty ... ok
test result: ok. 60 passed; 0 failed; 0 ignored
```

All 60 `ferro-mcp-oauth` tests pass. `cargo fmt --check` and `cargo clippy --all-targets -D warnings` both clean.

## Deviations from Plan

**1. [Rule 1 - Bug] Fixed clippy `uninlined_format_args` errors in test assertion messages**
- **Found during:** Post-implementation clippy check
- **Issue:** Four `assert!` calls in the test module used `{:?}, variable` format string style; clippy `-D warnings` rejects this in favor of inline `{variable:?}`.
- **Fix:** Rewrote all four assertion messages to use inline format args.
- **Files modified:** `ferro-mcp-oauth/src/device.rs`
- **Commit:** 0b49f5f5 (fixed before commit)

No other deviations. Plan executed as written.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced in this plan. `device.rs` is a pure data/utility module — no I/O, no HTTP handlers, no cache calls. The threat mitigations T-203-USERCODE-BRUTE, T-203-DEVICECODE-ENTROPY, and T-203-USERCODE-NORMALIZE are structurally satisfied by the charset constant, the `generate_auth_code()` delegation, and the single normalization function respectively.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| ferro-mcp-oauth/src/device.rs | FOUND |
| ferro-mcp-oauth/src/lib.rs | FOUND |
| commit 0b49f5f5 | FOUND |
