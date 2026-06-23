# Phase 239: Soft-delete data model + `deleted_at` migration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-23
**Phase:** 239-soft-delete-data-model-deleted-at-migration
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen)
**Areas discussed:** Migration scope · Migration form & portability · `deleted_at` enforcement point · field→column binding resolver · `created_at` + tenant server-injected contract

---

## Migration scope (target tables)

| Option | Description | Selected |
|--------|-------------|----------|
| `orders` only | Single CRUD pilot; flipped to `.deletable(true)` in Phase 243 | ✓ |
| All tenant-scoped tables | Speculatively add `deleted_at` everywhere now | |
| `orders` + `todos` | Pre-empt a second pilot | |

**User's choice:** `orders` only (recommended default).
**Notes:** Soft-delete is opt-in per projection; no other table is soft-deletable in v16.3.

---

## Migration form & backend portability

| Option | Description | Selected |
|--------|-------------|----------|
| New standalone additive migration | sea-orm `alter_table` + nullable `timestamp` column; append to `mod.rs` | ✓ |
| Edit the create_orders migration | Add the column inline to the shipped create | |
| Raw backend-specific SQL | Hand-write SQLite + Postgres DDL | |

**User's choice:** New standalone additive migration (recommended default).
**Notes:** Migrations are append-only. Nullable timestamp is backend-portable via sea-orm `ColumnDef`; existing rows default to `NULL` = "not deleted" (no backfill). Verify with `db:migrate` on SQLite and Postgres.

---

## `deleted_at IS NULL` enforcement point

| Option | Description | Selected |
|--------|-------------|----------|
| Read query builder (data layer) | Inject predicate in `dispatch.rs` WHERE assembly, mirroring tenant predicate | ✓ |
| Per-tool filtering | Each CRUD tool adds its own filter | |
| Model-level default scope | sea-orm entity default scope | |

**User's choice:** Read query builder, mirroring the tenant predicate (recommended default).
**Notes:** Success criterion #3 explicitly requires data-layer enforcement "not per-tool." Use the resolved column name so explicit overrides are honored.

---

## field→column binding resolver

| Option | Description | Selected |
|--------|-------------|----------|
| Resolver accessors on `ServiceDef` | `resolved_table()` / `resolved_soft_delete_column()` with defaults; wire into `dispatch.rs` | ✓ |
| Inline defaults at each call site | Repeat the pluralize / `"deleted_at"` fallback wherever needed | |
| Require explicit `.table()`/`.soft_delete_column()` | No defaults | |

**User's choice:** Resolver accessors with defaults (recommended default).
**Notes:** Defaults — table = `format!("{}s", name.to_lowercase())` (matches existing `dispatch.rs:122` TODO), soft-delete column = `"deleted_at"`. Replace the inline derivation + TODO in `dispatch.rs`. Table tests assert default vs explicit (success criterion #2).

---

## `created_at` + tenant server-injected contract (success criterion #4)

| Option | Description | Selected |
|--------|-------------|----------|
| DB default + `FieldMeaning` classification helper | `created_at` via column `DEFAULT current_timestamp`; classify Identifier/CreatedAt/tenant as non-input | ✓ |
| Enforce only in the future INSERT path | Defer entirely to Phase 241 | |
| Validation-rule at registration | Add a `validate()` rule | |

**User's choice:** DB default + classification helper (recommended default).
**Notes:** This phase fixes the *contract*: `created_at` defaulted in-DB (already true on `orders`); a classification predicate identifies server-injected/non-agent-input fields for Phase 240 to consume. Scope = predicate + tests, not schema emission.

## Claude's Discretion

- Migration filename/date stamp and `DeriveIden` enum shape.
- Resolver / classification helper naming and return shape (set vs predicate).

## Deferred Ideas

- Reusable `add_soft_delete_column` helper in `ferro-migration` (single-table need now — YAGNI).
- `deleted_at` on additional tables (opt-in per projection).
- `get_<svc>` tool, per-field `immutable()`/`read_only()` overrides (spec non-goals).
