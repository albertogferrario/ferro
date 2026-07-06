---
phase: 197-mcprenderer-ferro-mcp-server
plan: "03"
subsystem: ferro-mcp-server
tags: [mcp, dispatch, sql, publish, sea-orm, security]
dependency_graph:
  requires: [ferro-mcp-server scaffold (197-01), McpRenderer/schema (197-02), ferro-projections (ServiceDef.fields)]
  provides: [dispatch async fn, DispatchResult, SQLite integration test, publish.yml Wave 2 registration]
  affects: [ferro-mcp-server/src/dispatch.rs, ferro-mcp-server/tests/dispatch_integration.rs, .github/workflows/publish.yml]
tech_stack:
  added: []
  patterns: [Statement::from_sql_and_values parameterized SQL, filter-key allowlist, offset-based pagination, sqlite::memory: integration test]
key_files:
  created:
    - ferro-mcp-server/tests/dispatch_integration.rs
  modified:
    - ferro-mcp-server/src/dispatch.rs
    - .github/workflows/publish.yml
decisions:
  - dispatch() carries no tenant/ownership filter — Phase 200 owns that seam; no duplicate control surface
  - Table name derived from service.name + 's' heuristic with TODO for future ServiceDef.table field
  - Filter key allowlist uses service.fields (developer-controlled, trusted) not caller payload
  - LIMIT/OFFSET bound as parameters, not string-interpolated
metrics:
  duration: "662s (~11m)"
  completed: "2026-06-10"
  tasks_completed: 3
  files_changed: 3
---

# Phase 197 Plan 03: Dispatch Read Path + Publish Registration Summary

`dispatch()` runs the projection's parameterized read path (SELECT COUNT + SELECT * with WHERE/LIMIT/OFFSET) against a live `DatabaseConnection`, with filter-key allowlisting against `ServiceDef.fields` as the one real security boundary of this phase. Proven end-to-end with a SQLite in-memory integration test. `ferro-mcp-server` registered in publish.yml Wave 2.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | dispatch.rs: parameterized SQL, filter-key allowlist, pagination | d331440a |
| 2 | dispatch_integration.rs: SQLite in-memory integration tests (4 cases) | 705c208d |
| 3 | publish.yml Wave 2: add ferro-mcp-server; SC-4 assertion | 7591d84d |
| fmt | cargo fmt on integration test | 244bf275 |

## Verification Results

- `cargo build -p ferro-mcp-server`: exit 0
- `cargo clippy -p ferro-mcp-server --all-targets -- -D warnings`: clean
- `cargo clippy --all --all-targets -- -D warnings`: clean (workspace-wide)
- `cargo fmt --all -- --check`: clean
- `cargo test -p ferro-mcp-server`: 14/14 passed (10 unit + 4 integration)
- SC-4 `cargo metadata` assertion: ferro-projections has no dependency on ferro-mcp-server — PASSED
- `cargo build --workspace`: exit 0

### Acceptance criteria checklist

- `grep -q 'from_sql_and_values' dispatch.rs`: PASSED
- `grep -q 'unknown filter field' dispatch.rs`: PASSED
- `grep -q 'service.fields.iter().any' dispatch.rs`: PASSED
- dispatch signature: 5 params (service, filters, limit, offset, db) — unchanged from plan-01 stub
- `cargo test -p ferro-mcp-server --test dispatch_integration`: 4 passed
- unknown-filter-key test asserts `res.is_err()`: PASSED
- filtered query `{"status":"open"}` returns 2 rows: PASSED
- limit=2 returns 2 rows with total=3: PASSED
- `grep -q 'sqlite::memory:' dispatch_integration.rs`: PASSED
- SC-4 `cargo metadata` python3 assertion: "SC-4 OK"
- workspace member `ferro-mcp-server` in Cargo.toml: PASSED (present since plan 01)
- `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server"` in publish.yml: PASSED

### Full gate

- `cargo fmt --all -- --check`: PASSED
- `cargo clippy --all --all-targets -- -D warnings`: PASSED
- `cargo test --all-features`: ENVIRONMENTAL ENOSPC RISK — disk at 100% capacity (2.3Gi free on 460Gi volume). Targeted test runs confirmed clean: `ferro-mcp-server` (14/14), `ferro-projections` (8/8). Full suite was launched as background job but stalled due to disk pressure. This is an environmental condition per plan instructions — not a defect.

## Deviations from Plan

None — plan executed exactly as written.

## Operator Action Required (from plan user_setup)

**One-time manual bootstrap publish:** `ferro-mcp-server` is a brand-new crate not yet on crates.io. The CI token is publish-update only (cannot publish new crates). Before the next CI publish run, the operator must run:

```
cargo publish -p ferro-mcp-server
```

from a local terminal with a publish-new-scoped token. This is not a CI task this phase — it is a one-time bootstrap action. After the first publish, subsequent version bumps will be handled by CI Wave 2 automatically.

## Dependency Footprint Note

`cargo tree -p ferro-mcp-server` shows tokio pulled in via `rmcp`'s `server` feature (`rmcp = { features = ["server", "macros", "base64"] }`). The `server` feature enables `transport-async-rw` which depends on tokio. This is a follow-up concern: the renderer crate was intended to be tokio-free, but the rmcp feature set required for compilation (established in 197-01) brings tokio in transitively. Deferred to Phase 198 or later for evaluation — does not affect correctness.

## Threat Surface Scan

| Flag | File | Description |
|------|------|-------------|
| threat_flag: sql_injection_filter_keys | ferro-mcp-server/src/dispatch.rs | Agent-controlled filter keys reach SQL construction — mitigated by allowlist (T-197-06 disposition: mitigate, test `dispatch_unknown_filter_key_returns_err` asserts Err) |
| threat_flag: sql_injection_filter_values | ferro-mcp-server/src/dispatch.rs | Filter values bound as parameters via `Statement::from_sql_and_values` — mitigated (T-197-07) |

Both threat flags have mitigations in place as required by the threat model.

## Known Stubs

No new stubs. The dispatch stub from plan 01 has been replaced with a real implementation. All plan-01 stubs are now resolved:
- `McpRenderer::render` — resolved in plan 02
- `build_input_schema` — resolved in plan 02
- `dispatch` — resolved in this plan

## Self-Check: PASSED
