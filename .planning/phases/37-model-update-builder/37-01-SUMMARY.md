---
phase: 37-model-update-builder
plan: 01
subsystem: database
tags: [sea-orm, derive-macro, builder-pattern, proc-macro]

requires:
  - phase: 2-model-boilerplate
    provides: FerroModel derive macro with create builder
provides:
  - UpdateBuilder struct with selective field tracking
  - set_*() methods for all non-id, non-timestamp fields
  - clear_*() methods for Option fields
  - save() that only sends modified fields to database
affects: [37-02, docs]

tech-stack:
  added: []
  patterns:
    - "UpdateBuilder pattern: model.update().set_field(v).save().await"
    - "Selective field tracking via Option wrapping in builder"

key-files:
  created: []
  modified:
    - ferro-macros/src/model.rs
    - ferro-macros/src/lib.rs

key-decisions:
  - "Consume model in update() rather than borrow (simpler ownership, matches create pattern)"
  - "Option<Option<T>> for nullable fields: None=unchanged, Some(None)=clear, Some(Some(v))=set"

patterns-established:
  - "UpdateBuilder: typed builder for selective model updates with NotSet/Set tracking"

duration: 3min
completed: 2026-02-09
---

# Phase 37 Plan 01: Model Update Builder Summary

**FerroModel derive macro generates typed UpdateBuilder with selective field tracking, replacing verbose ActiveModel manipulation with `model.update().set_field(v).save().await`**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-09T05:15:34Z
- **Completed:** 2026-02-09T05:18:55Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Generated `{Model}UpdateBuilder` struct alongside existing create builder in FerroModel derive macro
- Selective field tracking: only modified fields sent to database via `Set`/`NotSet`
- `set_*()` methods for all non-id, non-timestamp fields with `impl Into<String>` ergonomics
- `clear_*()` methods for `Option` fields to explicitly NULL columns
- Auto-set `updated_at` on save when field is present (handles both `String` and `DateTimeUtc`)
- Removed old model setters and `to_active_model()` in favor of UpdateBuilder

## Task Commits

Each task was committed atomically:

1. **Task 1: Generate UpdateBuilder struct** - `8d6b57c` (feat)
2. **Task 2: Verify compilation and update existing model code** - verification only, no code changes needed

## Files Created/Modified

- `ferro-macros/src/model.rs` - Core macro implementation: added UpdateBuilder generation, removed old model setters and to_active_model
- `ferro-macros/src/lib.rs` - Updated FerroModel docstring to reflect new update builder API

## Decisions Made

- Consume model in `update()` (takes `self`) rather than borrowing -- simpler ownership model and matches the create builder pattern
- Use `Option<Option<T>>` for nullable fields in UpdateBuilder: `None` = unchanged, `Some(None)` = clear to NULL, `Some(Some(v))` = set to value
- Extract id type dynamically from struct fields to support `i32`, `i64`, `Uuid`, etc.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 37-02-PLAN.md (documentation updates)
- UpdateBuilder API is complete and all tests pass

---
*Phase: 37-model-update-builder*
*Completed: 2026-02-09*
