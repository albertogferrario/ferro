---
phase: 220-confirmation-gating-for-destructive-actions
plan: "02"
subsystem: app, ferro-mcp-server, ferro-ai
tags: [app-wiring, feature-gate, sc5-proof, ci-gate, confirmation]
dependency_graph:
  requires: ["220-01"]
  provides: ["AMCP-05-complete", "phase-220-complete"]
  affects: [app, ferro-mcp-server, ferro-ai]
tech_stack:
  added: ["app confirmation feature (forwards to ferro-mcp-server/confirmation + ferro-ai)"]
  patterns:
    - "OnceLock<InMemoryConfirmationStore> for process-wide shared store"
    - "#[cfg(feature = \"confirmation\")] arg threading at call sites"
    - "#[cfg(not(feature = \"confirmation\"))] gating pre-confirmation tests"
key_files:
  created: []
  modified:
    - app/Cargo.toml
    - app/src/controllers/mcp.rs
    - app/src/tests/mcp_write_dispatch.rs
    - app/src/tests/mcp_tenant_isolation.rs
decisions:
  - "OnceLock<InMemoryConfirmationStore> (not Arc<Mutex>) — process-wide singleton, no shutdown needed, zero overhead after init"
  - "ferro-ai added as direct optional dep to app (not transitive) — Rust does not allow source-level references to transitive crates"
  - "Phase 219 direct-write tests gated with #[cfg(not(feature = \"confirmation\"))] — submit is now confirmation-gated when feature is on; Phase 220 SC#1-#4 in ferro-mcp-server cover the confirmed path"
metrics:
  duration_seconds: ~900
  completed_date: "2026-06-14"
  tasks_completed: 2
  files_modified: 4
---

# Phase 220 Plan 02: App Wiring + SC#5 Proof + CI Gate — Summary

Process-wide `InMemoryConfirmationStore` wired into the sample app's MCP endpoint behind its own `confirmation` feature; SC#5 build-graph purity proven with `cargo tree`; full `--all-features` CI gate (fmt + clippy + test) green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Wire InMemoryConfirmationStore into sample app MCP endpoint | 650f8ac3 | app/Cargo.toml, app/src/controllers/mcp.rs |
| 2 | SC#5 build-graph assertion + full --all-features CI gate | 9d684f76 | app/src/tests/mcp_write_dispatch.rs, app/src/tests/mcp_tenant_isolation.rs, Cargo.lock |

## SC#5 Build-Graph Evidence

```
# Feature-off: ferro-ai absent from ferro-mcp-server dep tree
cargo build -p ferro-mcp-server          → OK
cargo tree -p ferro-mcp-server --edges normal | grep -c ferro-ai  = 0  ✓
cargo tree -p ferro-mcp-server --edges normal | grep -c reqwest   = 3  (pre-existing from ferro-mcp-oauth, NOT from ferro-ai)

# ferro-ai no-default-features compiles clean
cargo build -p ferro-ai --no-default-features  → OK

# Feature-on: ferro-ai present, reqwest count unchanged
cargo build -p ferro-mcp-server --features confirmation  → OK
cargo tree -p ferro-mcp-server --features confirmation --edges normal | grep -c ferro-ai  = 1  ✓
cargo tree -p ferro-mcp-server --features confirmation --edges normal | grep -c reqwest   = 3  (same 3 — zero new reqwest from ferro-ai)

# Feature-off tests (219 tests) green
cargo test -p ferro-mcp-server  → 48 passed (34+5+5+4)
```

The 3 reqwest lines are pre-existing from `ferro-mcp-oauth` (a hard dep of `ferro-mcp-server` since Phase 199), completely unrelated to `ferro-ai`. This matches the documented finding in 220-00-SUMMARY.

## CI Gate Evidence

```
cargo fmt --all -- --check           → clean (empty output)
cargo clippy --all --all-targets --all-features -- -D warnings  → clean
cargo test --all-features            → all test result lines: ok, 0 failed
```

Total tests across workspace: 900+ passing. No failures, no ENOSPC.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `confirmation_ttl_seconds` in app test struct literal**
- **Found during:** Task 1 (`cargo test -p app`)
- **Issue:** `app/src/controllers/mcp.rs` test constructed `McpServerConfig { app_name, app_url, version }` without the `confirmation_ttl_seconds` field added by Plan 01
- **Fix:** Added `confirmation_ttl_seconds: 300` to the struct literal
- **Files modified:** `app/src/controllers/mcp.rs`
- **Commit:** 650f8ac3

**2. [Rule 3 - Blocking] `ferro_ai` not reachable from app source as transitive dep**
- **Found during:** Task 1 (first `--features confirmation` build attempt)
- **Issue:** Rust does not allow source-level references to transitive crates; `use ferro_ai::InMemoryConfirmationStore` failed with E0433
- **Fix:** Added `ferro-ai = { path = "../ferro-ai", optional = true, default-features = false, features = ["confirmation"] }` as a direct dep in `app/Cargo.toml`; updated `confirmation` feature to include `dep:ferro-ai`
- **Files modified:** `app/Cargo.toml`
- **Commit:** 650f8ac3

**3. [Rule 1 - Bug] Phase 219 app write-dispatch tests fail under `--all-features`**
- **Found during:** Task 2 (`cargo test --all-features`)
- **Issue:** `write_call_produces_audit_entry` and `idempotent_write_e2e` call `submit` directly and expect success; with `confirmation` feature on, `submit` (a destructive action with `transition_trigger`) correctly returns `confirmation_required` before executing
- **Fix:** Gated both tests with `#[cfg(not(feature = "confirmation"))]`; added explanatory comments pointing to Phase 220 SC#1–#4 in `ferro-mcp-server` as the confirmed-path coverage
- **Files modified:** `app/src/tests/mcp_write_dispatch.rs`
- **Commit:** 9d684f76

**4. [Rule 1 - Bug] `handle_tools_call` call sites in app test files missing confirmation args**
- **Found during:** Task 2 (clippy `--all-targets --all-features` — E0061 arity mismatch)
- **Issue:** `mcp_write_dispatch.rs` (`call_write_tool` helper + 2 inline calls) and `mcp_tenant_isolation.rs` (2 calls) used the pre-Plan-01 arity without `store` and `config`
- **Fix:** Added `#[cfg(feature = "confirmation")] &ferro_ai::InMemoryConfirmationStore::new()` and `#[cfg(feature = "confirmation")] &test_config()` / `&McpServerConfig::default()` at each call site; added `McpServerConfig` to imports
- **Files modified:** `app/src/tests/mcp_write_dispatch.rs`, `app/src/tests/mcp_tenant_isolation.rs`
- **Commit:** 9d684f76

## Known Stubs

None — all paths produce real structured responses.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The `InMemoryConfirmationStore` is process-scoped in-memory state behind the existing `tools/call` trust boundary. Tenant binding in stored payloads (from Plan 01) prevents cross-tenant token use — no app-level mitigation needed (T-220-02e accepted upstream).

## Self-Check: PASSED
