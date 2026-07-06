---
phase: 41-api-resources-basics
plan: 02
subsystem: api
tags: [proc-macro, derive, resource, api-response, codegen]

requires:
  - phase: 41-01
    provides: Resource trait, ResourceMap builder
provides:
  - "#[derive(ApiResource)] proc macro with field selection, rename, skip"
  - "Automatic From<Model> generation via #[resource(model = \"...\")]"
  - "ferro::ApiResource re-exported from framework public API"
affects: [41-03-cli-docs, 42-api-resources-advanced]

tech-stack:
  added: []
  patterns: [derive-macro-with-field-attrs, generated-resource-impl]

key-files:
  created:
    - ferro-macros/src/resource.rs
    - framework/tests/api_resource_derive.rs
  modified:
    - ferro-macros/src/lib.rs
    - framework/src/lib.rs

key-decisions:
  - "Use ferro:: prefix in generated code (matches existing macro patterns)"
  - "From<Model> copies all fields including skipped ones (skip only affects JSON output)"

patterns-established:
  - "ApiResource derive: struct annotations generate Resource trait + optional From<Model>"
  - "Field-level #[resource(skip/rename)] for controlling JSON serialization shape"

duration: 4min
completed: 2026-02-10
---

# Phase 41 Plan 02: ApiResource Derive Macro Summary

**#[derive(ApiResource)] proc macro generating Resource trait impls with field-level skip/rename and optional From<Model> via #[resource(model = "...")]**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-10T04:39:52Z
- **Completed:** 2026-02-10T04:44:12Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- ApiResource derive macro with struct-level `model` attribute and field-level `skip`/`rename` attributes
- Automatic `Resource` trait implementation generating `ResourceMap` builder chain
- Automatic `From<Model>` implementation when `model` attribute is specified
- Compile errors for invalid attributes, non-struct types, and tuple structs
- 5 integration tests covering simple resources, rename, skip, From<Model>, and field order

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement ApiResource derive macro** - `ce84637` (feat)
2. **Task 2: Export ApiResource and add integration tests** - `3e944fc` (test)

## Files Created/Modified
- `ferro-macros/src/resource.rs` - ApiResource derive macro implementation with attribute parsing
- `ferro-macros/src/lib.rs` - Module registration and derive macro entry point
- `framework/src/lib.rs` - Re-export ferro::ApiResource in public API
- `framework/tests/api_resource_derive.rs` - Integration tests for all attribute combinations

## Decisions Made
- **ferro:: prefix in generated code:** Matches existing macro patterns (handler.rs, model.rs) where generated code uses `ferro::` to reference framework types.
- **From<Model> copies all fields:** The `From` impl maps every field from model to resource (including skipped ones). Skip only affects `to_resource()` JSON output, not the struct itself. This lets users access skipped fields programmatically while controlling API exposure.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ApiResource derive macro is complete and exported as `ferro::ApiResource`
- Ready for Plan 03 (CLI make:resource + docs + sample app)
- All three resource attribute types (#[resource(model/rename/skip)]) are functional and tested

---
*Phase: 41-api-resources-basics*
*Completed: 2026-02-10*
