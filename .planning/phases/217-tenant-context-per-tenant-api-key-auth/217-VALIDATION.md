---
phase: 217
slug: tenant-context-per-tenant-api-key-auth
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 217 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test runner (`cargo test`) + `tokio::test` for async DB lookups |
| **Config file** | None — `[dev-dependencies]` in `ferro-mcp-server` / `ferro-mcp-oauth` Cargo.toml |
| **Quick run command** | `cargo test -p ferro-mcp-server -p ferro-mcp-oauth` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (workspace build dominated) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-server -p ferro-mcp-oauth`
- **After every plan wave:** Run `cargo test --all-features` + `cargo clippy --all --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + clippy must be green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 217-00-01 | 00 | 0 | AMCP-01/02 | — | RED tests exist and fail before impl | unit/integration | `cargo test -p ferro-mcp-server -p ferro-mcp-oauth` | ❌ W0 | ⬜ pending |
| 217-01-01 | 01 | 1 | AMCP-02 | T-217-04 (scope creep) | API key + JWT yield same `BearerCheck::Authenticated` tenant_id | unit | `cargo test -p ferro-mcp-oauth validate_api_key` | ❌ W0 | ⬜ pending |
| 217-01-02 | 01 | 1 | AMCP-02 | T-217-04 | Invalid/revoked/expired key → not `Authenticated` | unit | `cargo test -p ferro-mcp-oauth validate::tests::invalid_api_key_rejected` | ❌ W0 | ⬜ pending |
| 217-01-03 | 01 | 1 | AMCP-02 | — | `generate_mcp_api_key` returns `ferro_`-prefixed plaintext + matching SHA-256 hash | unit | `cargo test -p ferro-mcp-oauth generate_mcp_api_key` | ❌ W0 | ⬜ pending |
| 217-02-01 | 02 | 2 | AMCP-01 | T-217-01 (cross-tenant leak) | `McpContext` carries `tenant_id` + `evaluated_guards`; threaded into dispatch | unit | `cargo test -p ferro-mcp-server` | ❌ W0 | ⬜ pending |
| 217-02-02 | 02 | 2 | AMCP-02 | T-217-04 | read-scoped key rejected on write-tool call; allowed on read-tool call | unit | `cargo test -p ferro-mcp-server read_scope` | ❌ W0 | ⬜ pending |
| 217-02-03 | 02 | 2 | AMCP-01 | T-217-01 | tenant A key surfaces no tenant B data in list or call | integration | `cargo test -p ferro-mcp-server mcp_tenant_isolation::api_key_cross_tenant_isolation` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — NEW file (does not exist today); covers SC#2, SC#3, SC#4, SC#5 via raw-SQL SQLite fixture (mirrors `dispatch_integration.rs` harness)
- [ ] `ferro-mcp-oauth/src/validate.rs` — RED unit tests for `validate_api_key` (parity, invalid, revoked, expired)
- [ ] `ferro-mcp-oauth` key-generation RED tests for `generate_mcp_api_key` (prefix + hash round-trip)
- [ ] Add `ferro-mcp-oauth` to `ferro-mcp-server/Cargo.toml` dependencies (prerequisite for the cross-crate tests to compile)
- [ ] Update `ferro-mcp-server/tests/jsonrpc_integration.rs` for the changed `handle_tools_list(ctx, ...)` signature

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | — | All phase behaviors have automated coverage | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
