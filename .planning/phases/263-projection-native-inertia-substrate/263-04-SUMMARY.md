---
phase: 263-projection-native-inertia-substrate
plan: "04"
subsystem: framework/inertia
tags: [inertia, delivery, framework, projection, writes, subst-03, subst-04]
requirements: [SUBST-03, SUBST-04]

dependency_graph:
  requires:
    - 263-01  # schema_contract in ferro-projections
    - 263-02  # permitted_actions in framework
    - 263-03  # projection_read::dispatch in framework
  provides:
    - framework::Inertia::from_projection (SUBST-03 delivery)
    - framework::ProjectionQuery
    - ferro::ProjectionQuery (facade re-export)
    - SUBST-04 write reuse confirmed (POST /{service}/{action} → dispatch_write(channel="web"))
  affects:
    - framework/src/inertia/ (new projection module)
    - framework/src/lib.rs (ProjectionQuery re-export)

tech_stack:
  added: []
  patterns:
    - "Inertia::from_projection delegates to Inertia::render after assembling six-key props"
    - "ProjectionQuery: consuming builder (mut self -> Self), Default { filters={}, limit=25, offset=0 }"
    - "Error path: returns rendered Inertia page with {error: msg}, never panics"
    - "#[cfg(feature = \"projections\")] gate on projection module in inertia/mod.rs"
    - "combined feature gate all(inertia, projections) for ProjectionQuery in facade"

key_files:
  created:
    - framework/src/inertia/projection.rs
  modified:
    - framework/src/inertia/mod.rs
    - framework/src/lib.rs

decisions:
  - "from_projection placed on framework::Inertia at framework/src/inertia/projection.rs (not ferro_inertia::Inertia) — cycle-free; framework already depends on ferro-inertia, so ferro-inertia→framework is a hard Cargo cycle (confirmed by Task 0 cargo tree self-check)"
  - "ProjectionQuery re-exported from ferro facade under all(inertia, projections) feature gate so callers do not need to reference the internal inertia:: path"
  - "projection module itself is #[cfg(feature = projections)] — users without projections feature do not pay the compile cost"

metrics:
  duration: "339s (~5m)"
  completed_date: "2026-07-27"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 2
---

# Phase 263 Plan 04: Inertia Delivery Helper + Write Reuse Confirmation

`Inertia::from_projection` on the framework-side Inertia facade assembles six-key props from three derivation cores and delegates to `Inertia::render`; write path confirmed reused with exactly one `dispatch_write` call site.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| Task 0 | Cycle-direction self-check (autonomous) | no code change | read-only verification |
| Task 1 | `Inertia::from_projection` + `ProjectionQuery` | `75df2b08`, `e5d5dc0d` | `framework/src/inertia/projection.rs` (new), `mod.rs`, `lib.rs` |
| Task 2 | Confirm Inertia write reuse | no code change | read-only verification |

## Final `from_projection` Signature and Module Path

**Module:** `framework/src/inertia/projection.rs`
**Method on:** `framework::Inertia` (the framework-side facade, NOT `ferro_inertia::Inertia`)

```rust
pub async fn from_projection(
    req: &Request,
    component: &str,
    service: &ServiceDef,
    query: ProjectionQuery,
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    evaluated_guards: &HashMap<String, bool>,
) -> Response
```

## Task 0: Cargo Tree Acyclicity Evidence

Both checks passed autonomously (no operator prompt):

```
PASS: ferro-inertia does NOT depend on ferro-mcp-server or ferro-rs
PASS: framework -> ferro-inertia confirmed
```

Exact commands:
- `! cargo tree -p ferro-inertia -e normal | grep -qE "ferro-mcp-server|ferro-rs"` → exit 0
- `cargo tree -p framework -e normal --features inertia | grep -q ferro-inertia` → exit 0

This confirms:
- `framework → ferro-inertia` (one-directional)
- `ferro-inertia` is clean of both `framework`/`ferro-rs` and `ferro-mcp-server`
- Placement at `framework/src/inertia/projection.rs` is the only cycle-free home

## Props Key Set

`from_projection` assembles and serializes exactly six keys:

| Key | Source | Type |
|-----|--------|------|
| `schema` | `schema_contract(service)` (Plan 01) | `SchemaContract` |
| `data` | `dispatch(...).rows` (Plan 03) | `Vec<Value>` |
| `permitted_actions` | `permitted_actions(service, evaluated_guards)` (Plan 02) | `Vec<String>` |
| `total` | `dispatch(...).total` | `u64` |
| `limit` | `dispatch(...).limit` (clamped to `MAX_LIMIT=100`) | `u64` |
| `offset` | `dispatch(...).offset` | `u64` |

## Task 2: Write Route Reuse Confirmation (SUBST-04)

The existing `POST /{service}/{action}` route (app/src/routes.rs:127) inside `TenantMiddleware(on_failure=Forbidden)` already routes through `controllers::visual_action::handle` → `dispatch_write(.., channel="web")`.

**Exactly one `dispatch_write` call site** in `visual_action.rs` (line 71):
```rust
let outcome = ferro::write::dispatch_write(
    action, &inputs, tenant_id, db.inner(), &dispatcher,
    transition_guard, "web", ...
)
.await;
```

The second occurrence of `dispatch_write` in that file is in a doc comment (line 15 — `//! Guards are re-evaluated server-side inside the shared dispatch_write`), not an actual call site. **No new write path was added.**

The Inertia frontend posts forms to this same route. Guard re-evaluation, persistence, and audit already exist in the kernel — this plan adds READ derivation only.

Visual_action tests: **16/16 passed** (`cargo test -p app visual_action`).

## A1 Research-Defect Correction (for phase retrospective)

The 263-RESEARCH.md assumption A1 stated that `ferro-inertia → framework` would be acyclic. This is **false**: `framework/Cargo.toml` already declares `ferro-inertia` as an optional dependency under the `inertia` feature. Adding a `ferro-inertia → framework (ferro-rs)` edge would create a hard Cargo cycle regardless of feature gating (mirrors the Phase 261 `ferro-bundle` cycle class).

The reconciled operator-approved decision: `Inertia::from_projection` lives on the **framework-side** `Inertia` facade at `framework/src/inertia/projection.rs`, not the `ferro-inertia` crate. The renderer-location rule's intent is honored — the helper is NOT in the pure `ferro-projections` crate; it is in the framework's Inertia delivery layer. `ferro-inertia` gained zero new dependencies.

This correction was documented in 263-CONTEXT.md and 263-PATTERNS.md before plan execution and is verified by the Task 0 `cargo tree` self-check.

## ProjectionQuery

```rust
/// Default: filters = {}, limit = 25, offset = 0
pub struct ProjectionQuery {
    pub filters: Value,
    pub limit: u64,
    pub offset: u64,
}

impl Default for ProjectionQuery { ... }

// Consuming builder methods (mut self -> Self):
impl ProjectionQuery {
    pub fn filters(mut self, f: Value) -> Self { ... }
    pub fn limit(mut self, n: u64) -> Self { ... }
    pub fn offset(mut self, n: u64) -> Self { ... }
}
```

Re-exported from the facade as `ferro::ProjectionQuery` under `all(feature = "inertia", feature = "projections")`.

## Deviations from Plan

### Auto-additions (Rule 2)

**1. [Rule 2 - Missing critical functionality] Re-export ProjectionQuery from ferro facade**
- **Found during:** Task 1 completion
- **Issue:** `from_projection` takes `ProjectionQuery` as a parameter but callers using `ferro::Inertia::from_projection` would need to reach into `ferro::inertia::ProjectionQuery` without a facade re-export — a usability gap.
- **Fix:** Added `#[cfg(all(feature = "inertia", feature = "projections"))] pub use inertia::ProjectionQuery;` to `framework/src/lib.rs`
- **Files modified:** `framework/src/lib.rs`
- **Commit:** `e5d5dc0d`

### Other

None — plan executed exactly as written (modulo the re-export addition above).

## Known Stubs

None. All six props keys are wired to real derivation cores (Plans 01-03). No placeholder data flows to the component.

## Threat Flags

No new threat surface introduced. The security properties analyzed in the plan's threat model are all covered:

| Threat | Status |
|--------|--------|
| T-263-10 (cross-tenant disclosure) | Mitigated — `dispatch` receives caller-supplied `tenant_id`, never from request body |
| T-263-11 (authz bypass via `permitted_actions`) | Mitigated — documented advisory-only; writes go through existing `dispatch_write` with live guard re-evaluation; Task 2 confirms exactly one call site |
| T-263-12 (dependency cycle / wrong placement) | Mitigated — Task 0 self-check passes; `ferro-inertia` gained no framework dep |
| T-263-13 (DoS via unbounded limit) | Mitigated — `dispatch` enforces `MAX_LIMIT=100` (Plan 03); `ProjectionQuery.limit` default is 25 |

## Test Results

| Test Suite | Command | Result |
|-----------|---------|--------|
| `from_projection` unit tests | `cargo test --manifest-path framework/Cargo.toml --features "projections inertia" "inertia::projection"` | 4/4 PASSED |
| `visual_action` confirmation | `cargo test --manifest-path app/Cargo.toml visual_action` | 16/16 PASSED |

## Self-Check: PASSED

Created files exist:
- `framework/src/inertia/projection.rs` — EXISTS (223 lines, >60 line minimum met)

Commits exist:
- `75df2b08` — EXISTS (Task 1: projection.rs + mod.rs)
- `e5d5dc0d` — EXISTS (Task 1b: lib.rs ProjectionQuery re-export)

All acceptance criteria met:
- `pub async fn from_projection` present in projection.rs
- `dispatch(` call in projection.rs
- `permitted_actions(service, evaluated_guards)` call in projection.rs
- `schema_contract(service)` call in projection.rs
- No `ferro_mcp_server` import in projection.rs
- `ferro-inertia/Cargo.toml` gained no framework/ferro-rs dependency
- `cargo test ... inertia::projection` exits 0 (4/4)
- `cargo test ... visual_action` exits 0 (16/16)
- `ferro-inertia` does NOT depend on `ferro-mcp-server` or `ferro-rs` (Task 0)
- `framework → ferro-inertia` confirmed one-directional (Task 0)
