---
phase: 202
slug: login-resume-contract-magic-link-sample-app
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-11
---

# Phase 202 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (built-in `#[test]` / `#[tokio::test]`) |
| **Config file** | none — workspace `Cargo.toml`; cache bootstrapped via `ferro_mcp_oauth::cache_test_helpers::bootstrap_test_cache()` |
| **Quick run command** | `cargo test -p ferro-mcp-oauth && cargo test -p app` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~120–300 seconds (workspace build dominates; see disk-full caveat) |

> Per project memory: `cargo test --all-features` can ENOSPC-fail on link/fingerprint — check `df` and clean `target/` before the full gate; that is not a real defect. Serialize CPU-heavy cargo runs (one at a time).

---

## Sampling Rate

- **After every task commit:** Run the per-crate quick command for the crate touched (`cargo test -p ferro-mcp-oauth` or `cargo test -p app`).
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings` + the quick test command.
- **Before `/gsd-verify-work`:** `cargo clippy --all --all-targets --all-features -- -D warnings` + `cargo test --all-features` green (SC-5).
- **Max feedback latency:** ~120 seconds (incremental per-crate build).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 202-01-xx | 01 | 1 | SC-1 (resume helper) | — | `take_oauth_return_to` reads then CLEARS the session key (no replay of return target) | unit | `cargo test -p ferro-mcp-oauth resume` | ❌ W0 | ⬜ pending |
| 202-01-xx | 01 | 1 | SC-1 (single key owner) | — | `authorize.rs` + `consent.rs` use the crate `store`/key, no inline `"oauth_return_to"` literal remains | unit/grep | `! grep -rn '"oauth_return_to"' ferro-mcp-oauth/src/authorize.rs ferro-mcp-oauth/src/consent.rs` | ✅ | ⬜ pending |
| 202-02-xx | 02 | 2 | SC-2 (token single-use) | T-202-01 (link replay) | magic-link token deleted on first verify; second verify fails | unit | `cargo test -p app magic_link_single_use` | ❌ W0 | ⬜ pending |
| 202-02-xx | 02 | 2 | SC-2 (TTL bound) | T-202-02 (stale link) | token absent after TTL → verify re-renders request page with error | unit | `cargo test -p app magic_link_expired` | ❌ W0 | ⬜ pending |
| 202-02-xx | 02 | 2 | SC-2 (dev surfacing) | — | `is_development()` → link on page + log, NO real mail send; non-dev → `Channel::Mail` path (not exercised) | unit | `cargo test -p app magic_link_dev_surface` | ❌ W0 | ⬜ pending |
| 202-03-xx | 03 | 3 | SC-4 (JSON-UI views) | T-202-XSS | request form + confirmation render valid `ferro-json-ui/v2`, `layout:"auth"`; updated `login_view_*` test green | unit | `cargo test -p app login_view` | ✅ (update) | ⬜ pending |
| 202-04-xx | 04 | 3 | SC-3 (async flow) | — | unauth `/authorize` → 302 `/auth/login` → request link → `verify` (return_to in session) → 302 resume `/authorize` → consent rendered | integration | `cargo test -p app oauth_magic_link_resume_flow` | ❌ W0 | ⬜ pending |
| 202-05-xx | 05 | 4 | SC-5 (gates + boot) | — | clippy/test green; app boots from any CWD (no request-time view-path panic in test) | command | `cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-oauth/src/resume.rs` (or `authorize.rs`) `#[cfg(test)]` module — `take_oauth_return_to` read-then-clear assertions for SC-1.
- [ ] `app/src/tests/magic_link.rs` — single-use, expiry, and dev-surface unit tests for SC-2 (uses `bootstrap_test_cache()`).
- [ ] `app/src/tests/oauth_magic_link_resume_flow.rs` — the SC-3 async-flow integration test (session continuity across requests; avoid direct view-rendering handler calls per RESEARCH finding 6).
- [ ] Update existing `login_view_is_valid_and_posts_to_login` in `auth_controller.rs` to the magic-link contract (email-only, no password field).

*Existing infrastructure (`cargo test`, `cache_test_helpers`) covers the rest — no new framework install.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real email send via `Channel::Mail` in non-dev | SC-2 (non-dev path) | Requires live SMTP/Resend config; deliberately not wired into CI to keep `cargo test` offline-green | Set `APP_ENV=production` + `MAIL_*`, request a link, confirm email arrives — done by operator, not CI |
| End-to-end browser MCP login with a real magic-link app | SC-2/SC-3 (golden path) | Full browser OAuth round-trip with email client | Optional: run sample app, point an MCP client at `/mcp`, complete login via the surfaced dev link |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s (per-crate incremental)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
