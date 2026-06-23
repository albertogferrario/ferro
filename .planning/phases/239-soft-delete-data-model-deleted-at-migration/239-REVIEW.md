---
phase: 239-soft-delete-data-model-deleted-at-migration
reviewed: 2026-06-23T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - app/src/bootstrap.rs
  - app/src/migrations/m20260623_add_deleted_at_to_orders.rs
  - app/src/migrations/mod.rs
  - app/src/models/entities/orders.rs
  - app/src/tests/mcp_tenant_isolation.rs
  - app/src/tests/mcp_write_dispatch.rs
  - app/src/tests/single_source.rs
  - app/src/tests/visual_action.rs
  - ferro-mcp-server/src/dispatch.rs
  - ferro-projections/src/service.rs
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 239: Code Review Report

**Reviewed:** 2026-06-23T00:00:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 239 adds a soft-delete substrate: a nullable `deleted_at` column on the `orders` table, three resolver accessors on `ServiceDef` (`resolved_table`, `resolved_soft_delete_column`, `is_server_injected_field`), and a `deleted_at IS NULL` predicate injected into the dispatch read path.

The core correctness and security properties are sound:

- The soft-delete predicate is gated strictly on `service.soft_delete_column.is_some()` (dispatch.rs:173), pushes no bound value, and does NOT increment `idx` — so LIMIT/OFFSET placeholder indices on Postgres are unaffected.
- The predicate is assembled into `where_clauses` before `where_str` is built, so it applies to BOTH the COUNT and DATA queries through the shared `{where_str}` substitution.
- `resolved_table()` replaces the inline `format!` with byte-identical logic; no behavior change.
- `is_server_injected_field` reads `self.tenant_column` dynamically — no hardcoded strings.
- Filter keys are allowlisted before any SQL assembly; values are bound parameters.
- The migration is additive (nullable column, no non-constant DEFAULT), append-only, and correctly appended to the migration vector.
- `deleted_at: Set(None)` in bootstrap.rs and all test seed helpers is correct usage of SeaORM's `ActiveValue::Set` for a nullable column.
- The `soft_delete_excluded` test in dispatch.rs uses its own inline DB — it does not mutate `setup_orders_db()`.

Two warnings and three info items follow.

## Warnings

### WR-01: `resolved_soft_delete_column` returns `"deleted_at"` when `soft_delete_column.is_none()`, but the predicate is only injected when `is_some()`

**File:** `ferro-projections/src/service.rs:223-225`
**Issue:** `resolved_soft_delete_column()` has an implicit default (`unwrap_or("deleted_at")`), but in `dispatch.rs` the predicate is only added when `service.soft_delete_column.is_some()`. The result accessor is therefore reachable only through the `is_some()` guard — its fallback branch is dead by construction.

The risk is future callers invoking `resolved_soft_delete_column()` without first checking `soft_delete_column.is_some()`, silently injecting an unwanted `deleted_at IS NULL` predicate on tables that do not have the column (producing a SQL error) or that have it but were not intended to soft-delete. The method's doc comment says "defaults to `deleted_at`" which implies the default is live.

**Fix:** Either make the method return `Option<&str>` (consistent with the optional nature of the field), or rename it to `soft_delete_column_name_or_default()` and add a loud doc comment that callers MUST check `soft_delete_column.is_some()` first. The cleanest option:

```rust
/// Returns the declared soft-delete column name, or `None` if the
/// projection has no soft-delete column. Callers should check
/// `soft_delete_column.is_some()` before using this.
pub fn resolved_soft_delete_column(&self) -> Option<&str> {
    self.soft_delete_column.as_deref()
}
```

Then update `dispatch.rs` to:
```rust
if let Some(col) = service.resolved_soft_delete_column() {
    where_clauses.push(format!("\"{col}\" IS NULL"));
}
```

### WR-02: `single_source.rs` calls `handle_tools_call` without the `#[cfg(feature = "confirmation")]` extra arguments present in sibling test files

**File:** `app/src/tests/single_source.rs:252-262`
**Issue:** The `drive_mcp` helper in `single_source.rs` calls `handle_tools_call` with 6 positional arguments:

```rust
let result = handle_tools_call(
    json!({ "name": action_name, "arguments": inputs }),
    &services,
    db,
    Some(tenant_id),
    &ctx,
    &disp,
)
.await;
```

The sibling files `mcp_tenant_isolation.rs` and `mcp_write_dispatch.rs` both include `#[cfg(feature = "confirmation")]` conditional arguments (confirmation store and server config). `single_source.rs` omits them entirely — no `#[cfg(feature = "confirmation")]` guard around additional args. If the `confirmation` feature is enabled in CI, this call site has the wrong arity and will fail to compile.

The file itself is gated `#[cfg(all(test, not(feature = "confirmation")))]` at the module level (line 24), which means the entire module is compiled away when `confirmation` is active. So the immediate compilation failure is suppressed by the module gate. However, this means the `single_source` tests are NEVER run with the `confirmation` feature on, whereas their stated invariant (single dispatch kernel, both surfaces) is equally true in that configuration. This is a correctness gap in the test coverage, not a compilation bug.

**Fix:** Mirror the pattern from `mcp_write_dispatch.rs` — move the `not(feature = "confirmation")` gate from the module level to the individual test functions that exercise destructive actions, and add `#[cfg(feature = "confirmation")]` arguments to `handle_tools_call` in `drive_mcp`. The structural tests (`single_source_guard_rejects_both`) can be ungated.

## Info

### IN-01: `setup_orders_db()` in dispatch.rs tests does not include a `deleted_at` column

**File:** `ferro-mcp-server/src/dispatch.rs:251-280`
**Issue:** The shared `setup_orders_db()` fixture creates an `orders` table without `deleted_at`. The existing tenant-scoping tests (`tenant_scoping`, `tenant_isolation`, `tenant_fail_closed`, `non_tenant_unscoped`) use this fixture, and the new `soft_delete_excluded` test correctly uses its own inline DB. This is fine for isolation.

However, if a future test tries to call `dispatch` with a `ServiceDef` that has `soft_delete_column` set using `setup_orders_db()`, it will get a SQL error on the missing column. The fixture should either add the column for completeness or carry a comment noting its intentional omission.

**Fix:** Add a comment to `setup_orders_db()`:

```rust
// Note: this fixture omits `deleted_at`. Tests that exercise soft-delete
// must create their own inline DB (see `soft_delete_excluded`).
```

### IN-02: `orders.rs` entity model uses `Option<String>` for `deleted_at` where the migration declares `TIMESTAMP`

**File:** `app/src/models/entities/orders.rs:19-21`
**Issue:** The migration adds `deleted_at` as a `TIMESTAMP NULL` column. The entity model maps it as `Option<String>`. This is consistent with how `created_at` is modeled in the same entity (also `String`), so it is not a new inconsistency introduced by this phase. It does mean that on Postgres the ORM will attempt string deserialization of a `TIMESTAMPTZ` column, which can silently produce `None` if the type mapping fails rather than a hard error.

This is pre-existing debt, not introduced by Phase 239 — flagged here because Phase 239 adds the column and is the right time to track it.

**Fix:** If this is SQLite-only, `String` is fine. If Postgres is in scope, change to `Option<chrono::NaiveDateTime>` or `Option<time::PrimitiveDateTime>` depending on the ORM feature in use, and add the appropriate `sea_orm(column_type = "TimestampWithTimeZone")` annotation.

### IN-03: `resolved_table` doc comment says "Matches the inline derivation previously at dispatch.rs:123" — the line number will drift

**File:** `ferro-projections/src/service.rs:213-219`
**Issue:** The doc comment `"Matches the inline derivation previously at dispatch.rs:123"` embeds a concrete line number that will become stale as `dispatch.rs` evolves. Line numbers in doc comments are rarely maintained and become misleading noise.

**Fix:** Remove the line number reference:

```rust
/// Returns the backing table name: explicit `.table()` value or the
/// pluralized, lowercased service name (e.g. "order" → "orders").
///
/// The default derivation is byte-identical to the previous inline format at
/// the dispatch call site — existing projections must not see a behavior change.
pub fn resolved_table(&self) -> String {
```

---

_Reviewed: 2026-06-23T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
