---
phase: 197
slug: mcprenderer-ferro-mcp-server
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 197 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + `cargo test` (sea-orm SQLite in-memory for dispatch) |
| **Config file** | None (workspace-level) |
| **Quick run command** | `cargo test -p ferro-mcp-server` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick: <30s for the new crate; full suite: minutes |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-mcp-server && cargo test -p ferro-projections`
- **After every plan wave:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green
- **Max feedback latency:** ~30 seconds (crate-scoped quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 197-A | TBD | 1 | AMCP-04 (scaffold) | structural | `grep ferro-mcp-server Cargo.toml .github/workflows/publish.yml` | ❌ W0 | ⬜ pending |
| 197-B | TBD | 1 | AMCP-01 | unit | `cargo test -p ferro-projections mcp_exposed_default` | ❌ W0 | ⬜ pending |
| 197-C | TBD | 2 | AMCP-01 | unit | `cargo test -p ferro-mcp-server test_mcp_exposed_filter` | ❌ W0 | ⬜ pending |
| 197-D | TBD | 2 | AMCP-02 | unit | `cargo test -p ferro-mcp-server test_input_schema_derivation` | ❌ W0 | ⬜ pending |
| 197-E | TBD | 2 | AMCP-02 | unit | `cargo test -p ferro-mcp-server test_sensitive_field_excluded` | ❌ W0 | ⬜ pending |
| 197-F | TBD | 2 | AMCP-02 | unit | `cargo test -p ferro-mcp-server test_write_only_excluded` | ❌ W0 | ⬜ pending |
| 197-G | TBD | 2 | AMCP-02 | unit | `cargo test -p ferro-mcp-server test_pagination_params_in_schema` | ❌ W0 | ⬜ pending |
| 197-H | TBD | 3 | AMCP-03 | integration | `cargo test -p ferro-mcp-server test_dispatch_sqlite` | ❌ W0 | ⬜ pending |
| 197-I | TBD | 3 | AMCP-04 | structural | `cargo metadata` — `ferro-projections` has no dep on `ferro-mcp-server` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Single-source-of-truth guard (AMCP-02):** a test that adds a filter field to a `ServiceDef` and asserts the rendered tool's `inputSchema` property count increases — proving the schema is derived, not separately declared.

---

## Wave 0 Requirements

- [ ] `ferro-mcp-server/Cargo.toml` — manifest mirroring `ferro-json-ui` (deps: ferro-projections path v0.2, serde, serde_json, schemars v1, `rmcp = { version = "0.12", default-features = false, features = ["schemars"] }`, thiserror, tracing)
- [ ] `ferro-mcp-server/src/` — crate scaffold (`lib.rs`, `renderer.rs`, `schema.rs`, `dispatch.rs`, `error.rs`)
- [ ] `ferro-mcp-server/tests/dispatch_integration.rs` — SQLite in-memory dispatch test
- [ ] `ferro-projections/src/service.rs` — `mcp_exposed: bool` field (`#[serde(default)]`) + builder + default test
- [ ] Root `Cargo.toml` workspace `members` + `.github/workflows/publish.yml` Wave 2 registration

*Framework already installed — `cargo test` available.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| New-crate first publish bootstrap | SC-5 | CI publish token is publish-update only; a brand-new crate needs a one-time manual `cargo publish` from a local terminal | Note in SUMMARY/STATE as an operator action; not a CI-runnable check this phase |

---

*Phase: 197-mcprenderer-ferro-mcp-server*
*Validation strategy created: 2026-06-10*
