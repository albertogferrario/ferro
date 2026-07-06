---
phase: 113-pattern-coherence
plan: 01
subsystem: docs
tags: [documentation, imports, rust, ferro, pattern-coherence]

requires:
  - phase: 111-documentation-coverage
    provides: 23 documentation files covering all framework features

provides:
  - Consistent import style across all 23 doc files (explicit crate-root imports)
  - Zero glob imports (use ferro::*) in any doc file
  - Zero sub-module path imports (use ferro::module::Type) except documented exceptions
  - All route handler examples have #[handler] attribute
  - All .unwrap() calls in non-test examples replaced with ? or .expect()

affects:
  - future documentation additions
  - developer onboarding from docs

tech-stack:
  added: []
  patterns:
    - "All ferro imports use explicit crate-root exports: use ferro::{TypeA, TypeB}"
    - "Known exceptions kept as sub-module paths with comment: ferro::testing::{FactoryTraits, DatabaseFactory, Expect}"
    - "Error propagation: ? in handler context, .expect() in setup context, .unwrap() acceptable in test context"
    - "rules! macro needs no import (#[macro_export])"

key-files:
  created: []
  modified:
    - docs/src/introduction.md
    - docs/src/getting-started/quickstart.md
    - docs/src/the-basics/routing.md
    - docs/src/the-basics/controllers.md
    - docs/src/json-ui/components.md
    - docs/src/json-ui/plugins.md
    - docs/src/json-ui/data-binding.md
    - docs/src/json-ui/getting-started.md
    - docs/src/features/api.md
    - docs/src/features/api-mcp.md
    - docs/src/features/inertia.md
    - docs/src/features/validation.md
    - docs/src/features/database.md
    - docs/src/features/rate-limiting.md
    - docs/src/features/authentication.md
    - docs/src/features/broadcasting.md
    - docs/src/features/testing.md
    - docs/src/features/ai.md
    - docs/src/features/projections.md
    - docs/src/features/stripe.md
    - docs/src/features/themes.md
    - docs/src/reference/cli.md

key-decisions:
  - "ferro::testing::{FactoryTraits, DatabaseFactory, Expect} kept as sub-module path imports with comment since not re-exported at crate root"
  - "Test context .unwrap() calls left as-is (acceptable pattern in test functions)"
  - "migration-guide BEFORE examples left unchanged (intentionally showing old patterns)"
  - "rules! macro has no import — #[macro_export] makes it available globally, comment added to validation.md"

patterns-established:
  - "Import style: use ferro::{ExplicitType, AnotherType} — one use statement per crate per code block where practical"
  - "Error propagation: ok_or_else(|| HttpResponse::bad_request())? in handlers, .expect(reason) in setup code"

requirements-completed:
  - COH-01
  - COH-02
  - COH-03

duration: 85min
completed: 2026-03-27
---

# Phase 113 Plan 01: Pattern Coherence — Imports, Handlers, and Error Propagation Summary

**Standardized 22 documentation files to use explicit crate-root imports, #[handler] attributes, and ? / .expect() error propagation instead of glob imports and .unwrap()**

## Performance

- **Duration:** ~85 min (continuing from previous session)
- **Started:** 2026-03-26 (previous session)
- **Completed:** 2026-03-27T01:25:00Z
- **Tasks:** 2
- **Files modified:** 22

## Accomplishments

- Replaced all glob imports (`use ferro::*`) in 6 doc files with explicit crate-root imports
- Converted all sub-module path imports (`use ferro::module::Type`) to crate-root in 13+ doc files
- Fixed 3 missing `#[handler]` attributes in `inertia.md` (login, logout, store handlers)
- Replaced all `.unwrap()` calls in non-test code examples with `?` or `.expect("reason")`
- Documented known exceptions: `ferro::testing::{FactoryTraits, DatabaseFactory, Expect}` with inline comments
- Fixed `ferro::routing::*` glob in `api-mcp.md`
- Fixed `ferro::http::FormRequest` sub-module import in `validation.md`
- Fixed `ferro::scheduling::` and `ferro::database::Seeder` sub-module imports in `cli.md`

## Task Commits

1. **Tasks 1 + 2: Import style, handler macros, error propagation** - `d9a16f6c` (docs)

## Files Created/Modified

- `docs/src/introduction.md` - glob import → explicit
- `docs/src/getting-started/quickstart.md` - glob imports → explicit
- `docs/src/the-basics/routing.md` - glob import → explicit
- `docs/src/the-basics/controllers.md` - glob imports → explicit
- `docs/src/json-ui/components.md` - 28 glob imports → per-component explicit imports
- `docs/src/json-ui/plugins.md` - glob imports → explicit
- `docs/src/json-ui/data-binding.md` - sub-module import → crate root
- `docs/src/json-ui/getting-started.md` - sub-module import → crate root
- `docs/src/features/api.md` - sub-module imports, .unwrap() fixes
- `docs/src/features/api-mcp.md` - glob sub-module import → explicit
- `docs/src/features/inertia.md` - multiple sub-module imports, missing #[handler], .unwrap() cleanup
- `docs/src/features/validation.md` - sub-module imports, glob rules import removed, .unwrap() fix
- `docs/src/features/database.md` - sub-module imports fixed
- `docs/src/features/rate-limiting.md` - middleware sub-module imports → crate root
- `docs/src/features/authentication.md` - auth/session sub-module imports, .unwrap() fixes
- `docs/src/features/broadcasting.md` - container::App → App, .unwrap() fixes
- `docs/src/features/testing.md` - ferro::testing:: sub-modules split into crate-root + exceptions
- `docs/src/features/ai.md` - .unwrap() fixes
- `docs/src/features/projections.md` - .unwrap() fixes
- `docs/src/features/stripe.md` - .unwrap() fix
- `docs/src/features/themes.md` - .unwrap() fixes
- `docs/src/reference/cli.md` - scheduling and database sub-module imports → crate root

## Decisions Made

- `ferro::testing::{FactoryTraits, DatabaseFactory, Expect}` are not re-exported at crate root — kept as sub-module path with `// not re-exported at crate root` comment
- Test context `.unwrap()` calls (inside `#[tokio::test]` functions) left as-is — acceptable pattern
- `rules!` macro comment in `validation.md` explains it needs no import (it's `#[macro_export]`)
- `migration-guide.md` BEFORE examples preserved unchanged

## Deviations from Plan

None — plan executed exactly as written. All import fixes, handler macro additions, and unwrap replacements were in-scope as planned.

## Issues Encountered

None.

## User Setup Required

None - documentation-only changes.

## Next Phase Readiness

- Phase 113 Plan 01 complete
- All documentation now uses consistent import style
- Developers reading docs will see idiomatic, compilable examples
- Ready for Phase 113 Plan 02 if it exists

---
*Phase: 113-pattern-coherence*
*Completed: 2026-03-27*
