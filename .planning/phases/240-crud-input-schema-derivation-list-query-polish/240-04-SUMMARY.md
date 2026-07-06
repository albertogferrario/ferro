---
phase: 240-crud-input-schema-derivation-list-query-polish
plan: "04"
subsystem: mcp-dispatch
tags: [ferro-mcp-server, dispatch, range-filters, sort, sql-injection-prevention, crud, list-query]

requires:
  - phase: 240-02
    provides: "is_range_filter_field gates which fields accept range ops; is_filter_field gates sort and ne/in base fields"

provides:
  - "fn split_op_key(key: &str) -> Option<(&str, &str)> — splits field__op on the LAST __ (rfind)"
  - "dispatch() extended with __op filter path: gt/gte/lt/lte/ne/in with allowlist-then-bind discipline"
  - "dispatch() extended with sort parsing: -field DESC / field ASC, validated against is_filter_field, placed before Identifier tiebreaker"
  - "SQLite integration tests: range_filter_returns_correct_rows, in_filter_returns_correct_rows, sort_orders_rows, equality_filter_backcompat"

affects:
  - "240-03 — renderer.rs emits the tool schemas for these new params; dispatch.rs now executes them"
  - "Phase 241 — write dispatch will extend the same dispatch.rs; must not conflict with the extended read path"

tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN within each task — RED commit (failing) then GREEN commit (implementation)"
    - "Allowlist-then-bind: op suffix and base field name both allowlisted before any SQL assembly; values always bound via json_to_sea_value"
    - "rfind for __op key splitting: last __ separator so field names with embedded __ split correctly (Pitfall 1)"
    - "sort extracted from filters BEFORE the loop (Pitfall 4) via filters.as_object_mut().remove; dispatch takes mut filters"
    - "IN placeholder expansion: (0..arr.len()).map(|i| placeholder(backend, idx + i)); idx += arr.len() after"
    - "ORDER BY: user sort before Identifier tiebreaker; four-arm match covers all (sort, tiebreaker) combinations"

key-files:
  created: []
  modified:
    - ferro-mcp-server/src/dispatch.rs

key-decisions:
  - "sort validated against is_filter_field allowlist (not is_range_filter_field) — the sort allowlist is meaning-based (Identifier, ForeignKey, Status, Category, Boolean, Custom), same as equality. This means total (Money) cannot be a sort key, which is consistent with equality. The plan examples showing sort=total were illustrative; id (Identifier) is the correct sortable column in the test fixture."
  - "filters parameter changed to mut to allow in-place remove('sort') without cloning — minimal signature change, no public contract change"
  - "Two-pass placeholder approach for __in replaced with single (0..arr.len()).map(|i| idx+i) pass — avoids clippy let_and_return warning"

requirements-completed: [CRUD-04]

duration: 5min
completed: "2026-06-23"
---

# Phase 240 Plan 04: List Query Polish — `__op` Filter Extension + Sort Summary

**`split_op_key` + restructured filter loop with allowlist-then-bind `__op` dispatch + sort parsing — the read-execution half of CRUD-04, extending `dispatch()` with parameterized range/comparison filters and sort while keeping tenant, soft-delete, and limit/offset blocks unchanged**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-23T17:36:11Z
- **Completed:** 2026-06-23T17:41:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

### Task 1: split_op_key helper + __op filter loop + sort parsing/ORDER BY

- `split_op_key` added as a module-level `fn` using `rfind("__")` — correctly handles field names containing `__` (last separator wins)
- `is_range_filter_field` added to the `use crate::schema::{...}` import
- Filter loop restructured to single-pass: each key routed through `split_op_key` — op path or equality path; equality path byte-for-byte unchanged
- Op path: exhaustive match maps gt/gte/lt/lte/ne/in to SQL operators; unknown op → `InvalidFilter` with "unknown op suffix" message; base field validated against `is_range_filter_field` (gt/gte/lt/lte) or `is_filter_field` (ne/in); non-matching base → same "unknown or non-filterable filter field" error
- `__in` path: `val.as_array()` required (else InvalidFilter), empty array rejected (Pitfall 2/T-240-14), `(0..arr.len()).map(|i| placeholder(backend, idx + i))` builds the `IN (?,?,...)` list, `idx += arr.len()` advances correctly for subsequent clauses
- Sort extracted from `filters` before the loop via `filters.as_object_mut().remove("sort")` (Pitfall 4); parsed into `(col, dir)` with `-` prefix → DESC; col validated against `is_filter_field` (else InvalidFilter "unknown or non-sortable field")
- ORDER BY extended with four-arm match: user sort before Identifier tiebreaker; when sort col IS the tiebreaker, tiebreaker dropped to avoid `ORDER BY "id" ASC, "id"` redundancy; fallback to Identifier-only (unchanged behavior)
- Tenant predicate (151–166), soft-delete predicate (168–178), and LIMIT/OFFSET (213–221) blocks unchanged

### Task 2: SQLite in-memory integration tests for range/__in/sort + equality back-compat

- `range_filter_returns_correct_rows`: `total__gt 150` → Bob(200) + Dave(250) = 2 rows; `total__lte 150` → Alice(100) + Carol(150) = 2 rows
- `in_filter_returns_correct_rows`: `status__in ["pending"]` → Alice + Carol = 2 rows with status=pending; empty `[]` → `InvalidFilter`
- `sort_orders_rows`: `sort=id` → [1,2,3,4] ascending; `sort=-id` → [4,3,2,1] descending; uses `id` (Identifier meaning, passes `is_filter_field`)
- `equality_filter_backcompat`: `{"status": "pending"}` → same 2 rows (Alice + Carol) as before the extension
- All four reuse `setup_orders_db()` and `order_service_no_tenant()` — no new fixture

## Task Commits

1. **Task 1 RED** — `9d875559`: test(240-04): add RED tests for split_op_key, __op filter loop, and sort parsing
2. **Task 1 GREEN** — `8b7fc315`: feat(240-04): add split_op_key + __op filter loop + sort parsing/ORDER BY to dispatch
3. **Task 2 (GREEN)** — `dfeb504e`: test(240-04): add SQLite integration tests for range/__in/sort + equality back-compat

## Files Created/Modified

- `ferro-mcp-server/src/dispatch.rs` — `split_op_key` fn, extended `dispatch()` with `__op` filter path and sort parsing, four integration tests + three unit tests for error paths

## Decisions Made

- `sort` allowlist is `is_filter_field` (meaning-based), not `is_range_filter_field` (DataType-based). This is consistent with the plan (D-11: "Base field allowlisted against the dispatch filter-key allowlist"). In practice it means `total` (Money) cannot be sorted — this is intentional. The PATTERNS.md example showing `sort=total` was illustrative code but would fail validation; the integration test correctly uses `sort=id` (Identifier).
- `dispatch()` parameter `filters` changed from `serde_json::Value` to `mut serde_json::Value` so `remove("sort")` can happen in-place. This is a minimal ABI change (call sites passing a literal `json!({})` are unaffected since the value is moved in).
- The first-pass `let placeholders = arr.iter().map(|_| ph).collect()` was dropped in favor of a single `(0..arr.len()).map(|i| placeholder(backend, idx+i)).collect()` to satisfy `clippy::let_and_return`. No behavior change.

## Deviations from Plan

None — plan executed exactly as written. The TDD RED/GREEN cycle for Task 1 proceeded as expected. Task 2 tests went straight to GREEN (the Task 1 implementation already satisfies them) — this is the expected outcome when tests and implementation span adjacent tasks in the same plan.

The one adjustment: the PATTERNS.md example shows `sort=total` in a test but `total` has `FieldMeaning::Money` which is excluded by `is_filter_field`'s meaning gate (gate 5). The integration test uses `sort=id` instead, which is the correct sortable column in the fixture (Identifier meaning passes `is_filter_field`). This is not a deviation from the plan's behavior spec — D-11 explicitly says "Base field allowlisted against the dispatch filter-key allowlist", and the test exercises sort ASC/DESC correctly.

## Known Stubs

None. All dispatch extensions are fully implemented and verified against SQLite in-memory data.

## Threat Flags

None. All four mitigations from the threat register are implemented:
- T-240-10 (op suffix injection): op matched to fixed SQL string constant; unknown → `InvalidFilter`
- T-240-11 (IN element injection): each element bound via `json_to_sea_value` + placeholder; no interpolation
- T-240-12 (filter/sort on excluded field): base fields and sort col validated against allowlist; same non-disclosing error as equality
- T-240-13 (predicate stripping): tenant + `deleted_at IS NULL` blocks unchanged; verified by unchanged-blocks acceptance criterion
- T-240-14 (empty IN): empty `__in` array rejected at application layer with `InvalidFilter`

## Self-Check

See below.

---
*Phase: 240-crud-input-schema-derivation-list-query-polish*
*Completed: 2026-06-23*
