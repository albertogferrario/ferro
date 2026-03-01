---
phase: 92-mcp-introspection-cli
plan: 01
subsystem: cli
tags: [syn, sea-orm, code-generation, service-projections]

# Dependency graph
requires:
  - phase: 91-framework-integration
    provides: make:projection command, ServiceDef builder API
  - phase: 84-service-projections
    provides: FieldMeaning, DataType, ServiceDef types
provides:
  - Model-aware projection scaffolding via --from-model flag
  - Rust type to DataType mapping logic
  - Field meaning inference for code generation
affects: [92-mcp-introspection-cli, 93-protocol-docs]

# Tech tracking
tech-stack:
  added: []
  patterns: [model-scanning-via-syn, type-mapping-for-codegen, field-meaning-inference]

key-files:
  modified:
    - ferro-cli/src/commands/make_projection.rs
    - ferro-cli/src/main.rs

key-decisions:
  - "Self-contained model scanning in make_projection.rs (not imported from make_api) to avoid coupling"
  - "Replicated infer_meaning logic inline instead of adding ferro-projections as dependency"
  - "Sensitive fields excluded entirely from generated output"
  - "FK fields generate both read_only_field and belongs_to calls"

patterns-established:
  - "rust_type_to_data_type: maps Rust/SeaORM types to DataType enum string for codegen"
  - "infer_meaning: field name to FieldMeaning variant string for codegen"
  - "model_aware_template: generates complete ServiceDef builder chain from model fields"

# Metrics
duration: 12min
completed: 2026-03-01
---

# Phase 92 Plan 01: Model-Aware Projection Scaffolding Summary

**`ferro make:projection --from-model` reads SeaORM model fields and generates populated ServiceDef with type mapping, meaning inference, FK detection, and sensitive field exclusion**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-01
- **Completed:** 2026-03-01
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `--from-model` flag to `ferro make:projection` that scans `src/models/` for matching SeaORM models
- Rust type to DataType mapping covering 20+ types (String, integers, floats, DateTime variants, Uuid, Binary, Json)
- Field meaning inference replicating ferro-projections logic (exact matches, suffix/prefix patterns, sensitive detection)
- Automatic belongs_to relationship generation from FK fields
- Sensitive fields (password, secret, token, api_key, hashed_key) excluded from generated output
- 6 new tests covering all model-aware generation paths

## Task Commits

Each task was committed atomically:

1. **Task 1: Add --from-model flag and model-aware template generation** - `283d05c` (feat)
2. **Task 2: Add tests for model-aware projection generation** - included in `283d05c` (tests co-located with implementation in same file)

## Files Created/Modified
- `ferro-cli/src/commands/make_projection.rs` - Added model scanning, type mapping, meaning inference, model-aware template generation, and 6 tests
- `ferro-cli/src/main.rs` - Added `--from-model` flag to MakeProjection variant (committed in prior phase 92-02)

## Decisions Made
- Self-contained ModelField/ModelVisitor types in make_projection.rs rather than importing from make_api (avoids coupling, make_api types are private)
- Replicated infer_meaning logic as string-returning function rather than adding ferro-projections dependency (plan explicitly prohibits this)
- Unknown Rust types fall back to DataType::String rather than erroring
- Optional FK fields (e.g., `Option<i32>` with `_id` suffix) still get read_only_field + belongs_to since FK semantics take precedence over nullability

## Deviations from Plan

None - plan executed as specified.

## Issues Encountered
- main.rs changes (--from-model flag) were already committed by a prior agent in `2fced76` as part of the projection:check work. No conflict; the changes were correct.
- `cargo clippy --all --all-targets` fails without `--all-features` due to feature-gated `ProjectionCheck` variant. Pre-existing issue, not introduced by this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Model-aware projection scaffolding complete and tested
- Ready for Plan 02 (projection validation CLI + MCP tools) or Plan 03 (service discovery)

---
*Phase: 92-mcp-introspection-cli*
*Completed: 2026-03-01*
