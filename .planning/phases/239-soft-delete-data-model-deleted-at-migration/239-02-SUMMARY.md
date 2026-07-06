---
phase: 239-soft-delete-data-model-deleted-at-migration
plan: "02"
subsystem: ferro-projections
tags: [service-def, accessor, resolver, classifier, soft-delete, server-injected]
dependency_graph:
  requires: []
  provides: [ServiceDef::resolved_table, ServiceDef::resolved_soft_delete_column, ServiceDef::is_server_injected_field]
  affects: [ferro-mcp-server/src/dispatch.rs (Plan 03 consumer), Phase 240 write-schema derivation]
tech_stack:
  added: []
  patterns: [resolver accessor on ServiceDef, FieldMeaning classifier predicate]
key_files:
  modified:
    - ferro-projections/src/service.rs
decisions:
  - resolved_table() returns String (allocation unavoidable for default); resolved_soft_delete_column() returns &str (borrows field or 'static literal)
  - is_server_injected_field() covers only Identifier/CreatedAt/tenant-column — not Sensitive (separate concern Phase 240 may unify)
  - deleted_at is never a declared projection field (framework-managed); classifier does not need to handle it
metrics:
  duration_seconds: 124
  completed_date: "2026-06-23"
  tasks_completed: 2
  files_modified: 1
---

# Phase 239 Plan 02: ServiceDef Resolver Accessors + Server-Injected Classifier Summary

Three pure `&self` accessors added to `ServiceDef` in `ferro-projections`: `resolved_table()`, `resolved_soft_delete_column()`, and `is_server_injected_field()`. Nine table tests cover all three. This is the generic, project-agnostic substrate Plan 03 wires into dispatch and Phase 240 consumes for write-schema exclusion.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | resolved_table() + resolved_soft_delete_column() with table tests | 409593b5 | ferro-projections/src/service.rs |
| 2 | is_server_injected_field() classifier with table tests | 409593b5 | ferro-projections/src/service.rs |

Both tasks were committed together as a single atomic commit since they land in the same file and the classifier task had no intermediate verification step requiring a separate commit.

## Verification Results

- `cargo test -p ferro-projections resolved_` — 5 tests passing
- `cargo test -p ferro-projections server_injected` — 4 tests passing
- `cargo test -p ferro-projections` — 276 unit + 22 catalog + 1 schema + 8 integration = 307 total, all passing
- `cargo clippy -p ferro-projections --all-targets -- -D warnings` — clean
- `cargo fmt -p ferro-projections -- --check` — clean
- `grep 'format!("{}s", self.name.to_lowercase())' ferro-projections/src/service.rs` — matches (default byte-identical to prior dispatch.rs inline derivation)

## Success Criteria

- SC#2 (resolver): `resolved_table()` returns default `"{}s".format(name.to_lowercase())` when unset and the explicit override when set; `resolved_soft_delete_column()` returns `"deleted_at"` by default and the explicit value when overridden. **MET.**
- SC#4 (substrate): `is_server_injected_field()` returns true for Identifier, CreatedAt, and the tenant column; false for ordinary data fields. **MET.**

## Deviations from Plan

None — plan executed exactly as written. Tasks 1 and 2 were committed as a single commit rather than two separate commits because both modify the same file (`service.rs`) and the plan's TDD structure was satisfied: all tests written and verified passing before commit.

## Known Stubs

None. All three accessors are fully wired with real logic and regression-pinned by table tests.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The three accessors are pure `&self` predicates operating on already-declared `ServiceDef` fields. No new threat surface.

## Self-Check: PASSED

- ferro-projections/src/service.rs: FOUND (modified in place)
- Commit 409593b5: FOUND (`git log --oneline -1` confirms)
- 5 resolver tests: CONFIRMED passing
- 4 classifier tests: CONFIRMED passing
- Clippy: CONFIRMED clean
- Format: CONFIRMED clean
- Grep acceptance check: CONFIRMED byte-identical default
