---
phase: 198
slug: streamable-http-endpoint-unauthenticated-challenge
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 198 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` + `tokio` (async) |
| **Config file** | `ferro-mcp-server/Cargo.toml` (`[dev-dependencies] tokio = { version = "1", features = ["full", "macros"] }`) |
| **Quick run command** | `cargo test -p ferro-mcp-server` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (full clippy+test on this workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-server`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 198-pure-dispatch | jsonrpc | 1 | AMCP-05 (SC-1a) | — | `initialize` returns `protocolVersion: "2025-03-26"` + `capabilities.tools` | unit | `cargo test -p ferro-mcp-server jsonrpc` | ❌ W0 | ⬜ pending |
| 198-tools-list | jsonrpc | 1 | AMCP-05 (SC-1b) | — | `tools/list` returns exactly the `mcp_exposed` projections | unit | `cargo test -p ferro-mcp-server jsonrpc` | ❌ W0 | ⬜ pending |
| 198-tools-call | jsonrpc | 1 | AMCP-05 (SC-1c) | T-198 input | `tools/call` returns dispatch rows; strips `list_` prefix; allowlist + limit clamp retained (197 WR-01/WR-02) | integration (in-memory DB) | `cargo test -p ferro-mcp-server jsonrpc` | ❌ W0 | ⬜ pending |
| 198-bearer-seam | auth | 1 | AMCP-06 (SC-2) | T-198 authn | `extract_bearer` has NO path returning `Authenticated` in Phase 198 | unit | `cargo test -p ferro-mcp-server auth` | ❌ W0 | ⬜ pending |
| 198-401-challenge | handler | 2 | AMCP-06 (SC-2) | T-198 authn | unauthenticated → `401` + `WWW-Authenticate: Bearer resource_metadata="{APP_URL}/.well-known/oauth-protected-resource"` | unit | `cargo test -p app --lib mcp` | ❌ W0 | ⬜ pending |
| 198-route-mount | handler | 2 | AMCP-05 (SC-3) | — | `post!("/mcp")` registered in app router via same middleware stack | compile-time | `cargo build -p app` | ❌ W0 | ⬜ pending |
| 198-no-oauth | jsonrpc | 1 | AMCP-06 (SC-4) | — | all four paths exercised without a live server/OAuth | integration (no server) | `cargo test -p ferro-mcp-server` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-server/tests/common/mod.rs` — extract `setup_db()`/`fresh_db()` in-memory SQLite fixture for cross-file reuse (Open Question 2)
- [ ] `ferro-mcp-server/tests/jsonrpc_integration.rs` — covers AMCP-05 SC-1a/b/c + SC-4
- [ ] `ferro-mcp-server/src/auth.rs` — `extract_bearer` unit tests inline (SC-2 seam-always-unauthenticated)
- [ ] `tokio` dev-dependency present in `ferro-mcp-server/Cargo.toml` with `macros` feature

*Existing dispatch_integration.rs covers the read path (197); new files cover the JSON-RPC + auth surface.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real MCP client follows the `WWW-Authenticate` challenge | AMCP-06 | Requires a live server + an MCP client; out of Phase 198 scope (deferred to Phase 199/200 dogfood) | Deferred — Phase 200 dogfood gate exercises this end-to-end |

*Phase 198's automated tests fully cover SC-1..SC-4 without a live server.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`tests/common/mod.rs`, `jsonrpc_integration.rs`)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
