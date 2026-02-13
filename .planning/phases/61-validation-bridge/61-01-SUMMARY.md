---
phase: 61-validation-bridge
plan: 01
subsystem: validation
tags: [oncelock, fn-pointer, validation, translation, bridge]

# Dependency graph
requires:
  - phase: 60-locale-context
    provides: locale()/set_locale() for per-request locale detection
provides:
  - TranslatorFn type alias for validation message callbacks
  - register_validation_translator() public API
  - translate_validation() pub(crate) helper for rules
affects: [62-validation-rules-update, 63-framework-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "OnceLock<fn> callback bridge for cross-module decoupling"

key-files:
  created:
    - framework/src/validation/bridge.rs
  modified:
    - framework/src/validation/mod.rs
    - framework/src/lib.rs

key-decisions:
  - "fn pointer over Box<dyn Fn> — no state capture needed, simpler"
  - "OnceLock without RwLock — translator set once at boot, never changes"
  - "pub(crate) translate_validation — only rules within validation call it"

patterns-established:
  - "Validation bridge: OnceLock<TranslatorFn> with None-fallback for optional localization"

# Metrics
duration: 3min
completed: 2026-02-13
---

# Phase 61 Plan 01: Validation Bridge Summary

**OnceLock-based TranslatorFn callback bridge in validation module, decoupled from ferro-lang, with None-fallback for English defaults**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-13T16:39:01Z
- **Completed:** 2026-02-13T16:42:11Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Created validation bridge module with `TranslatorFn` type alias, `OnceLock` static, registration function, and lookup helper
- Wired bridge into `validation/mod.rs` and `framework/src/lib.rs` re-exports
- Added tests proving None fallback and fn pointer signature compatibility
- Zero new dependencies added

## Task Commits

Each task was committed atomically:

1. **Task 1: Create validation bridge module** - `ea84220` (feat)
2. **Task 2: Add bridge tests and format exports** - `221f13c` (test)

## Files Created/Modified
- `framework/src/validation/bridge.rs` - Translation bridge with OnceLock, TranslatorFn, register/lookup functions, tests
- `framework/src/validation/mod.rs` - Added bridge module declaration and pub use exports
- `framework/src/lib.rs` - Added register_validation_translator and TranslatorFn to validation re-exports

## Decisions Made
- Used `fn` pointer (`TranslatorFn`) over `Box<dyn Fn>` — the translator is a static function, no closure state capture needed
- Used `OnceLock` alone without `RwLock` — translator is set once at boot and never changes
- Made `translate_validation()` `pub(crate)` — only validation rules within the crate call it
- Tests included in bridge.rs alongside implementation (standard Rust convention)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Expected `unused` warnings for `translate_validation` since no rules call it yet (Phase 62 work). These are warnings, not errors.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Bridge mechanism complete and exported from `ferro::register_validation_translator` and `ferro::TranslatorFn`
- Ready for Phase 62 to update 21 validation rules to call `translate_validation()`
- No blockers or concerns

---
*Phase: 61-validation-bridge*
*Completed: 2026-02-13*
