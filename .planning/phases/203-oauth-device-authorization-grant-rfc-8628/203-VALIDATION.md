---
phase: 203
slug: oauth-device-authorization-grant-rfc-8628
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-11
---

# Phase 203 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[tokio::test]` + inline `#[test]` (`tokio` full features) |
| **Config file** | `ferro-mcp-oauth/Cargo.toml` `[dev-dependencies]` (no new deps) |
| **Quick run command** | `cargo test -p ferro-mcp-oauth` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (crate) / longer for `--all-features` gate |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-oauth`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds (crate-scoped quick run)

---

## Per-Task Verification Map

> Task IDs are filled in by the planner; this maps each Success Criterion to its evidence.

| SC | Behavior | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|----|----------|------------|-----------------|-----------|-------------------|-------------|--------|
| SC-1 | `POST /device_authorization` returns exact RFC §3.2 fields | T-203-USERCODE-BRUTE | high-entropy `device_code`; user_code from §6.1 charset | unit | `cargo test -p ferro-mcp-oauth device_authorization` | ❌ W0 | ⬜ pending |
| SC-2 | Verification page code-entry + confirm+consent; unauth→login→resume; binds user+tenant | T-203-CSRF / T-203-TENANT-BYPASS | CSRF `ct_eq`; tenant from `current_tenant()` not form | unit | `cargo test -p ferro-mcp-oauth device_verification` | ❌ W0 | ⬜ pending |
| SC-3 | `POST /token` device-code arm returns `authorization_pending`/`slow_down`/`expired_token`/`access_denied`/`access_token` per §3.5 | T-203-CLAIMS-DIVERGE | single JWT mint path; claims-identical to auth-code | unit | `cargo test -p ferro-mcp-oauth token_exchange` | partial (add cases) | ⬜ pending |
| SC-3 | Device token audience-bound + tenant-scoped identically to auth-code flow | T-203-CLAIMS-DIVERGE | `build_claims` + `mint_token` same args | unit | `cargo test -p ferro-mcp-oauth device_grant_token_claims_identical` | ❌ W0 | ⬜ pending |
| SC-4 | Discovery advertises `device_authorization_endpoint` + device-code grant type | — | metadata read-only, public | unit | `cargo test -p ferro-mcp-oauth discovery` | partial (add assertions) | ⬜ pending |
| SC-4 | `device_code`/`user_code` single-use with TTL | T-203-DEVICECODE-REPLAY | get-then-forget on issue; TTL guard | unit | `cargo test -p ferro-mcp-oauth device_polling` | ❌ W0 | ⬜ pending |
| SC-5 | pending→approved polling, expiry, `slow_down` backoff, denied consent, tenant binding | T-203-DEVICECODE-REPLAY | full state-machine coverage | unit | `cargo test -p ferro-mcp-oauth device_polling` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-oauth/src/device.rs` (new module) — `DeviceGrant` record + handlers; covers SC-1, SC-2, SC-5 test stubs
- [ ] New test cases appended to `ferro-mcp-oauth/src/token.rs` — device-code grant arm (SC-3)
- [ ] New assertions in `ferro-mcp-oauth/src/discovery.rs` tests — SC-4
- [ ] Reuse `cache_test_helpers::bootstrap_test_cache()` (already exists) — no framework install needed

---

## Concrete Test List (SC-5 Matrix — from RESEARCH.md)

| Test Name | Scenario | Key Assertion |
|-----------|----------|----------------|
| `device_grant_pending_returns_authorization_pending` | Poll Pending grant within interval | `error == "authorization_pending"` |
| `device_grant_approved_returns_access_token` | Pending→Approved, poll | `access_token` present, `token_type == "Bearer"` |
| `device_grant_expired_returns_expired_token` | `Cache::get` → `None` (or `created_at` guard) | `error == "expired_token"` |
| `device_grant_slow_down_on_fast_poll` | `last_polled_at = now`, poll immediately | `error == "slow_down"` |
| `device_grant_denied_returns_access_denied` | Denied state, poll | `error == "access_denied"` |
| `device_grant_tenant_binding` | Approved grant `tenant_id = Some(7)` | Minted JWT carries `tenant_id = 7` |
| `device_grant_token_claims_identical_to_auth_code` | Device vs auth-code `McpTokenClaims` | Same `sub`/`aud`/`iss`/`tenant_id` shape |
| `device_authorization_response_fields` | Device-authorization handler | All 6 §3.2 fields present |
| `user_code_normalization_strips_hyphen_and_case` | `normalize_user_code("wdjb-mfxg")` | `== "WDJBMFXG"` |
| `user_code_format_is_xxxx_hyphen_xxxx` | `generate_user_code()` | len 9, idx-4 `-`, rest in `BCDFGHJKLMNPQRSTVWXZ` |
| `discovery_advertises_device_authorization_endpoint` | `authorization_server_metadata` | key `== {app_url}/device_authorization` |
| `discovery_advertises_device_grant_type` | `authorization_server_metadata` | `grant_types_supported` contains the device URN |

---

## Manual-Only Verifications

| Behavior | SC | Why Manual | Test Instructions |
|----------|----|-----------|--------------------|
| Real cross-device end-to-end (CLI requests code, user approves on phone, CLI receives token) | SC-2/SC-3 | Requires a live server + second device; not CI-automatable | Run sample app; start device flow from a CLI; open `verification_uri_complete` on a separate device; approve; confirm CLI poll receives `access_token` |

*All protocol-level behaviors have automated unit verification; only the live multi-device walkthrough is manual.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`device.rs`, token.rs cases, discovery assertions)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
