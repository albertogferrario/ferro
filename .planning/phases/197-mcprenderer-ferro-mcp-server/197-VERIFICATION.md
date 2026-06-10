---
phase: 197-mcprenderer-ferro-mcp-server
verified: 2026-06-10T12:00:00Z
status: passed
score: 5/5
overrides_applied: 0
re_verification: false
---

# Phase 197: McpRenderer & ferro-mcp-server — Verification Report

**Phase Goal:** A `ServiceDef`-marked projection appears in an in-process `tools/list` call as exactly one MCP tool, with input JSON schema derived from the projection's filter and pagination fields and output derived from its read path. `ferro-projections` gains no renderer dependency.
**Verified:** 2026-06-10T12:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A projection with `mcp_exposed: true` appears in `tools/list`; a projection without it does not | ✓ VERIFIED | `render_exposed_tools` in `renderer.rs:63` filters on `s.mcp_exposed`; `test_mcp_exposed_filter` asserts exactly 1 tool from 2 services (1 exposed, 1 not); integration confirmed by 10/10 unit tests passing |
| 2 | The tool's `inputSchema` is derived solely from `ServiceDef` filter and pagination fields — no separately declared schema exists | ✓ VERIFIED | `build_input_schema` in `schema.rs` constructs the schema entirely from `service.fields.iter().filter(is_filter_field)`; `adding_field_changes_schema` test (renderer.rs:148) asserts adding a field increases property count; no static schema object anywhere in the codebase |
| 3 | Calling the tool's dispatch function executes the projection's existing read path and returns its rows as MCP structured content | ✓ VERIFIED | `dispatch()` in `dispatch.rs` runs `SELECT COUNT` + `SELECT *` via `Statement::from_sql_and_values`; 5 integration tests pass against SQLite in-memory: empty-filter returns all rows, status-filter returns matching rows, pagination returns subset with correct total |
| 4 | `ferro-projections` has no new dependency on `ferro-mcp-server`; the dependency direction is `ferro-mcp-server` → `ferro-projections` | ✓ VERIFIED | `grep -q 'ferro-mcp-server' ferro-projections/Cargo.toml` returns nothing; `cargo metadata` confirms no edge from ferro-projections to ferro-mcp-server |
| 5 | The new crate is registered in `.github/workflows/publish.yml` at the correct publish wave | ✓ VERIFIED | `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server"` in publish.yml; `ferro-mcp-server` also in workspace `members` in root `Cargo.toml` |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp-server/src/renderer.rs` | McpRenderer implementing Renderer trait, render_exposed_tools | ✓ VERIFIED | Contains `impl Renderer for McpRenderer`, `render_exposed_tools`, filter on `s.mcp_exposed`, `ToolAnnotations::new().read_only(true)` |
| `ferro-mcp-server/src/schema.rs` | build_input_schema, is_filter_field, data_type_to_json_schema | ✓ VERIFIED | All three functions present; 5-gate `is_filter_field` predicate documented and load-bearing |
| `ferro-mcp-server/src/dispatch.rs` | dispatch async fn, allowlist validation, parameterized SQL | ✓ VERIFIED | `Statement::from_sql_and_values`, `MAX_LIMIT = 100`, `limit.min(MAX_LIMIT)` clamp, `is_filter_field` eligibility check at line 124, deterministic `ORDER BY` |
| `ferro-mcp-server/src/error.rs` | Error enum + Result<T> alias | ✓ VERIFIED | `pub enum Error` with Render/Database/Serialization variants; `pub type Result<T>` |
| `ferro-mcp-server/src/lib.rs` | Module declarations + re-exports | ✓ VERIFIED | All 4 modules declared; `render_exposed_tools`, `McpContext`, `McpRenderer`, `dispatch`, `DispatchResult`, `Error`, `Result` re-exported |
| `ferro-mcp-server/Cargo.toml` | Crate manifest with ferro-projections path dep | ✓ VERIFIED | `ferro-projections = { path = "../ferro-projections", version = "0.2" }`; `default-features = false` on rmcp; `transport-io` absent |
| `ferro-mcp-server/tests/dispatch_integration.rs` | SQLite in-memory dispatch tests | ✓ VERIFIED | 5 integration tests: empty-filter, status-filter, limit-pagination, non-filterable-field-rejected, unknown-key-rejected |
| `ferro-projections/src/service.rs` | mcp_exposed bool field + builder method | ✓ VERIFIED | `#[serde(default)] pub mcp_exposed: bool` at line 84; `pub fn mcp_exposed(mut self, exposed: bool) -> Self` at line 117; `mcp_exposed: false` in `new()` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `renderer.rs` | `schema.rs` | `crate::schema::build_input_schema` call in `render()` | ✓ WIRED | Line 37: `crate::schema::build_input_schema(service)` — result feeds directly into tool's `input_schema` |
| `dispatch.rs` | `schema.rs` | `use crate::schema::is_filter_field` for allowlist check | ✓ WIRED | Line 1 import; line 124 usage in filter-key eligibility check — same predicate used in both schema and dispatch |
| `dispatch.rs` | `sea_orm::DatabaseConnection` | `Statement::from_sql_and_values` parameterized query | ✓ WIRED | Lines 145, 180; filter values bound as parameters, never string-interpolated |
| `ferro-mcp-server/Cargo.toml` | `ferro-projections` | path dependency | ✓ WIRED | `ferro-projections = { path = "../ferro-projections", version = "0.2" }` |
| `Cargo.toml` (workspace) | `ferro-mcp-server` | workspace members | ✓ WIRED | Line 16 of root Cargo.toml |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `dispatch.rs` | `rows: Vec<serde_json::Value>` | `db.query_all(data_stmt)` with `Statement::from_sql_and_values` | Yes — live DatabaseConnection query | ✓ FLOWING |
| `dispatch.rs` | `total: u64` | `db.query_one(count_stmt)` COUNT query | Yes — same database, same WHERE clause | ✓ FLOWING |
| `renderer.rs` | `Tool.input_schema` | `build_input_schema(service)` derived from `service.fields` | Yes — derives from real ServiceDef field list | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Unit tests: render, schema, filter | `cargo test -p ferro-mcp-server` (unit) | 10/10 passed | ✓ PASS |
| Integration tests: dispatch end-to-end | `cargo test -p ferro-mcp-server` (integration) | 5/5 passed | ✓ PASS |
| ferro-projections mcp_exposed tests | `cargo test -p ferro-projections mcp_exposed` | 2/2 passed | ✓ PASS |
| WR-01 regression: non-filterable field rejected | `dispatch_non_filterable_field_rejected` (Sensitive field) | PASS — returns Err | ✓ PASS |
| WR-02 regression: limit clamped | `MAX_LIMIT = 100`, `limit.min(MAX_LIMIT)` in dispatch | Code verified | ✓ PASS |
| WR-04 fix: ORDER BY present | `order_str` variable + `format!("...{order_str}{limit_str}")` | Code verified | ✓ PASS |

**Note on full test gate:** `cargo test --all-features` is blocked by ENOSPC (disk full, ~1.9Gi free) — a known recurring environmental issue. `cargo clippy --all --all-targets -- -D warnings` passed for the whole workspace. Tests were run scoped: `cargo test -p ferro-mcp-server` (15 total: 10 unit + 5 integration) and `cargo test -p ferro-projections mcp_exposed` (2 tests). These constitute the test evidence for this phase's deliverables.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| AMCP-01 | 197-01, 197-02 | Exposed projection appears in tools/list; unmarked excluded | ✓ SATISFIED | `render_exposed_tools` filters on `mcp_exposed`; `test_mcp_exposed_filter` passes |
| AMCP-02 | 197-02 | inputSchema derived from ServiceDef, not declared separately | ✓ SATISFIED | `build_input_schema` derives from fields; `adding_field_changes_schema` guard test passes |
| AMCP-03 | 197-03 | dispatch() executes read path and returns rows | ✓ SATISFIED | Parameterized SQL via `from_sql_and_values`; 5 integration tests pass including filter and pagination |
| AMCP-04 | 197-01, 197-03 | McpRenderer in new output crate; ferro-projections gains no renderer dep | ✓ SATISFIED | Metadata edge absent; only `mcp_exposed: bool` added to ferro-projections |
| SC-5 | 197-03 | ferro-mcp-server in publish.yml Wave 2 | ✓ SATISFIED | `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server"` confirmed |

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ferro-projections/src/service.rs:83` | `#[serde(default)]` without `skip_serializing_if` on `mcp_exposed` | ℹ️ Info | Every serialized ServiceDef emits `"mcp_exposed": false` even when unused. Code-review item IN-01 noted this. Deserialization correctness is unaffected; purely cosmetic JSON bloat. Does not block goal. |
| `ferro-mcp-server/src/renderer.rs:64` | `derive_intents(s)` called but result passed as `_intents` (unused) | ℹ️ Info | Code-review item IN-02. Non-trivial computation discarded. Does not affect correctness. Future fix: pass `&[]` when intents are not needed. Does not block goal. |
| `ferro-mcp-server/src/dispatch.rs:109` | Table name derived as `service.name + "s"` with TODO comment | ℹ️ Info | Heuristic pluralization — works for standard names but breaks for irregular plurals. Explicitly tracked as a TODO. Not a blocker for the phase goal. |
| `ferro-mcp-server/src/dispatch.rs:47-85` | `rows_to_json` fallback to `Value::Null` for unmapped types (u32, Decimal, dates) | ⚠️ Warning | Code-review item WR-03. Type gaps produce silent nulls. Does not block the phase goal (SQLite integers map correctly via i64); only affects future deployment with richer column types. |

No stub implementations, no placeholder returns, no TODO-gated logic paths in the critical dispatch/render/schema surface. All four review warnings are acknowledged known gaps for future phases, not phase 197 blockers.

### Human Verification Required

No items require human verification. All success criteria are verifiable programmatically:
- Tool generation from ServiceDef is tested by unit tests
- Dispatch SQL correctness is tested by SQLite in-memory integration tests
- Dependency isolation is verified by cargo metadata
- Publish registration is verified by grep of publish.yml

### Gaps Summary

No gaps. All 5 roadmap success criteria are satisfied by real, substantive, wired, and data-flowing implementation.

**rmcp feature deviation noted (not a gap):** Plan 01 prescribed `features = ["schemars"]`; the actual Cargo.toml uses `features = ["server", "macros", "base64"]`. SUMMARY-01 documents the reason: the rmcp `model` module unconditionally imports `pastey` and `base64` regardless of feature flags, making `schemars`-only insufficient; `transport-io` (the stdio transport that would violate the renderer-only design) is explicitly absent. The plan's intent (no transport in a renderer crate) is preserved.

---

_Verified: 2026-06-10T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
