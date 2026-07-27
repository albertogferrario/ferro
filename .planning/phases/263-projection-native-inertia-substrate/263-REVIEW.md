---
phase: 263-projection-native-inertia-substrate
reviewed: 2026-07-27T00:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - ferro-projections/src/schema_contract.rs
  - ferro-projections/src/lib.rs
  - ferro-projections/tests/schema_contract.rs
  - framework/src/permitted_actions.rs
  - framework/src/projection_read.rs
  - framework/src/inertia/projection.rs
  - framework/src/inertia/mod.rs
  - framework/src/lib.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/dispatch.rs
  - ferro-mcp-server/src/schema.rs
  - app/src/tests/permitted_actions_parity.rs
  - app/src/tests/data_tenant_scoping.rs
  - app/src/tests/single_source.rs
  - app/src/tests/mod.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: clean
---

# Phase 263: Code Review Report

**Reviewed:** 2026-07-27
**Depth:** standard
**Files Reviewed:** 15
**Status:** clean

## Summary

Phase 263 ("projection-native inertia substrate") derives `{schema, data,
permitted_actions, total, limit, offset}` from a single `ServiceDef` and delivers
it via a new `Inertia::from_projection` facade, while lifting the guard-visibility
filter into `framework::permitted_actions` and relocating the tenant-scoped read
kernel into `framework::projection_read`. This is a security-sensitive phase; the
review focused on the four risk areas called out in the brief: visibility-vs-authz
semantics, tenant scoping / SQL-injection surface, dependency-cycle avoidance, and
the behavior-neutrality of the relocation.

All four hold. No critical or warning findings. Three informational items are noted
below; none block the phase.

### Visibility filter, not authz — verified

`framework::permitted_actions` (permitted_actions.rs:18-33) is a pure list-time
visibility filter with the correct deny-semantics: absent key = allow
(`evaluated_guards.get(p) == Some(&false)` only excludes on an explicit `false`),
`Some(false)` = deny, `Some(true)` = allow. It is exercised at exactly the sites the
brief requires and never gates execution:

- MCP `render_action_tool` (renderer.rs:230), `render_request_confirm_tool`
  (renderer.rs:365), and `render_confirm_tool` (renderer.rs:416) all call
  `permitted_actions` as a visibility filter only. Their doc comments explicitly
  disclaim authz ("VISIBILITY filter, NOT an authorization gate").
- `Inertia::from_projection` (projection.rs:111) uses the result as advisory
  display props only; the doc comment (projection.rs:85-88) states
  "`permitted_actions` in props is **advisory display data only**, not an
  authorization grant." Write enforcement remains at `dispatch_write` via the live
  `GuardEvaluatorFn`, and `McpContext::write_authorized` (renderer.rs:23-34) stays a
  dedicated, separate authz signal. The `permitted_actions_parity` test
  (permitted_actions_parity.rs) pins that the Inertia and MCP surfaces expose the
  identical action set for the same guard map.

### Tenant scoping / injection surface — verified

`framework::projection_read::dispatch` (projection_read.rs:207-442) is sound:

- The tenant predicate is a bound parameter (`"{col}" = {placeholder}` with
  `sea_orm::Value::BigInt(Some(tid))`, projection_read.rs:357-358). `tenant_id` is a
  function parameter, never read from `filters`. Fail-closed on
  `tenant_column = Some` + `tenant_id = None` (projection_read.rs:361-366), proven by
  `tenant_fail_closed` and `cross_tenant_id_not_found`.
- `MAX_LIMIT = 100` clamp (projection_read.rs:219) and `MAX_OFFSET = i64::MAX`
  clamp (projection_read.rs:220) both guard the `u64 -> i64` cast against negative
  wrap; enforced regardless of the caller-supplied value.
- Filter KEYS are allowlisted before any SQL assembly: equality keys must pass
  `is_filter_field` (projection_read.rs:337-344), range/`ne`/`in` keys must pass the
  op-appropriate predicate (projection_read.rs:282-295), and `sort` must name a
  field passing `is_filter_field || is_range_filter_field` (projection_read.rs:245-254).
  Unknown keys return `Err` and are never interpolated. Filter VALUES are always
  bound via `Statement::from_sql_and_values`.
- Interpolated column names (`sort` column, tiebreaker order column, tenant column,
  soft-delete column, table name) are all either validated against `service.fields`
  or read from developer-controlled `ServiceDef` fields — none originate from the
  call payload.
- Soft-delete predicate is `IS NULL` with no bound value and does not consume a
  placeholder index, so the subsequent `LIMIT`/`OFFSET` placeholders keep correct
  1-based indices on Postgres (projection_read.rs:375-380, 421-425). Verified by
  `soft_delete_excluded`.

### No dependency cycle — verified

`Inertia::from_projection` lives in `framework/src/inertia/projection.rs`, gated
behind `#[cfg(feature = "projections")]` (inertia/mod.rs:26-27, 33-34). It imports
only `ferro_projections`, `crate::*`, and `sea_orm`/`serde` — no `ferro-inertia ->
framework` edge and no `ferro-mcp-server` import (confirmed: projection.rs contains
no `mcp` reference; `framework/Cargo.toml` has no `ferro-mcp-server` dependency). On
a `dispatch` error it returns a rendered Inertia error page
(`{ "error": "<message>" }`, projection.rs:124-131), never a panic.

### Relocation is behavior-neutral — verified

`ferro-mcp-server::dispatch` (dispatch.rs) is now a thin wrapper that delegates to
`ferro_rs::projection_read::dispatch` and maps `ProjectionReadError` back 1:1
(`InvalidFilter -> Error::InvalidFilter`, `Database -> Error::Database`,
dispatch.rs:22-25), matching the `WriteError` mapping pattern in
`ferro-mcp-server/src/error.rs`. `is_filter_field` / `is_range_filter_field` are
re-exported from the framework's canonical definition (schema.rs:6), so the schema
builder and the data query share one source of truth. The `single_source` and
`permitted_actions_parity` tests pin cross-surface identity.

## Info

### IN-01: `rows_to_json` type-probe order may mis-type edge-case columns

**File:** `framework/src/projection_read.rs:162-189`
**Issue:** Column values are decoded by trying `String`, then `i64`, then `f64`,
then `bool`, taking the first that succeeds. For SQLite this relies on the driver
rejecting a `try_get_by::<String>` on an INTEGER/REAL column so the probe falls
through. Column affinity edge cases (e.g. a numeric value stored in a TEXT-affinity
column, or a boolean stored as `0`/`1` integer that will decode as `i64` before
`bool` is tried) can yield a JSON type that differs from the field's declared
`DataType`. This is a relocated pre-existing behavior, not introduced by this phase,
and the phase goal is behavior-neutrality — so it is out of scope to fix here.
**Fix (future, optional):** Drive decoding from the projection's declared
`FieldDef.data_type` per column rather than a first-success probe, so the emitted
JSON type is deterministic and matches the schema contract. Track separately from
263 since changing it is a behavior change, not a relocation.

### IN-02: `total` count uses `as u64` cast without an explicit guard

**File:** `framework/src/projection_read.rs:395-397`
**Issue:** `let total: u64 = count_row...try_get_by::<i64,_>("cnt")...unwrap_or(0)
as u64`. `COUNT(*)` is always non-negative so the `i64 -> u64` cast is safe in
practice, but the cast is unguarded and the intent ("count is non-negative") is
implicit. Not a defect.
**Fix (optional):** `.map(|c| c.max(0) as u64)` or a comment noting COUNT(*) is
non-negative, mirroring the explicit-clamp discipline already applied to
`MAX_LIMIT`/`MAX_OFFSET`.

### IN-03: `_base_field` bound purely for its allowlist side effect

**File:** `framework/src/projection_read.rs:282-295`
**Issue:** In the op-path, the field allowlist check binds `let _base_field = match
... { Some(f) ... => f, _ => return Err(...) }`. The `f` is never used after the
match — only the `Err` branch matters. The leading underscore signals this
intentionally, but the value binding is dead. Readability nit only.
**Fix (optional):** Replace with a boolean guard that returns `Err` on failure
without binding the field, e.g. `if !service.fields.iter().any(|f| f.name == base &&
op_matches(f, op)) { return Err(...) }`, to make the "validate-then-discard" intent
explicit.

---

_Reviewed: 2026-07-27_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
