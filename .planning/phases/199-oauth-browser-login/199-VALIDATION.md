---
phase: 199
slug: oauth-browser-login
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 199 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[tokio::test]` + in-process assertions (no test-runner config) |
| **Config file** | none — inline `#[cfg(test)]` modules and `tests/*.rs` |
| **Quick run command** | `cargo test -p ferro-mcp-oauth` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (full workspace), ~5s crate-local |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-oauth` (+ `cargo clippy -p ferro-mcp-oauth --all-targets -- -D warnings`)
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 199-WW-XX | scaffold | 0 | — | — | crate compiles; `OAuthConfig::from_env` fails closed when `MCP_TOKEN_SECRET` unset (non-debug) | unit | `cargo test -p ferro-mcp-oauth config` | ❌ W0 | ⬜ pending |
| 199-WW-XX | discovery | 1 | AMCP-07 | — | `.well-known` docs return spec field names; `code_challenge_methods_supported=["S256"]` | unit | `cargo test -p ferro-mcp-oauth discovery` | ❌ W0 | ⬜ pending |
| 199-WW-XX | register | 1 | AMCP-07 | T-redirect-exact, T-client-enum | DCR returns `client_id` (UUIDv4); `redirect_uris` required + stored | integration | `cargo test -p ferro-mcp-oauth register` | ❌ W0 | ⬜ pending |
| 199-WW-XX | pkce | 1 | AMCP-08 | T-pkce-downgrade | correct verifier → true; wrong verifier → false; constant-time compare | unit | `cargo test -p ferro-mcp-oauth pkce` | ❌ W0 | ⬜ pending |
| 199-WW-XX | jwt-mint | 1 | AMCP-08 | T-alg-confusion | HS256 mint→decode round-trip; claims `sub`,`tenant_id`,`aud`,`iss`,`exp` | unit | `cargo test -p ferro-mcp-oauth jwt` | ❌ W0 | ⬜ pending |
| 199-WW-XX | authorize | 2 | AMCP-08 | T-open-redirect, T-csrf-consent | unauth → redirect `/auth/login`; `redirect_uri` exact-match; consent CSRF enforced | integration | `cargo test -p ferro-mcp-oauth authorize` | ❌ W0 | ⬜ pending |
| 199-WW-XX | token | 2 | AMCP-08 | T-code-replay, T-code-ttl | code single-use (`forget` before validate); TTL ~60s; PKCE verified | integration | `cargo test -p ferro-mcp-oauth token` | ❌ W0 | ⬜ pending |
| 199-WW-XX | flow-e2e | 2 | AMCP-08 | — | DCR→authorize→consent→token→validate end-to-end, no external IdP | integration | `cargo test -p ferro-mcp-oauth flow` | ❌ W0 | ⬜ pending |
| 199-WW-XX | validate-bearer | 2 | AMCP-09 | T-aud-confusion, T-tenant-confusion | valid→Authenticated; expired→401; wrong aud→403; wrong `tenant_id`→403; no header→Unauthenticated | unit | `cargo test -p ferro-mcp-oauth validate` | ❌ W0 | ⬜ pending |
| 199-WW-XX | seam-wire | 3 | AMCP-09 | T-origin | `/mcp` accepts valid bearer, rejects invalid (401) / mismatch (403); Origin present-but-mismatched rejected | integration | `cargo test -p ferro-mcp-server` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Task IDs are placeholders (199-WW-XX) — the planner assigns concrete `{plan}-{task}` IDs.*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-oauth/src/lib.rs` + module files — crate scaffold (Cargo.toml, added to workspace members + `.github/workflows/publish.yml` Wave 2)
- [ ] `ferro-mcp-oauth/tests/flow_integration.rs` — full PKCE flow integration test harness (in-memory SQLite + in-memory `ferro::Cache`)
- [ ] `app/src/migrations/m20260611_create_oauth_clients_table.rs` — `oauth_clients` migration + registration in the app migration list
- [ ] `OAuthConfig` test fixture populated with a deterministic test `MCP_TOKEN_SECRET`
- [ ] `Request` test-helper pattern reused from `ferro-mcp-server/tests/dispatch_integration.rs` (`fresh_db()`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| A real MCP client (Claude Desktop / MCP SDK) completes browser login against a live app | AMCP-08 (dogfood) | Requires a live browser + external client; deferred to Phase 200 GO/NO-GO | Phase 200 acceptance — not blocking Phase 199 automated gate |

*All Phase 199 success criteria (SC-1…SC-5) have automated verification via in-process integration tests; the live-client dogfood is Phase 200's gate.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (crate scaffold, migration, flow harness)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
