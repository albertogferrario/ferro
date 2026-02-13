---
phase: 63-framework-integration
plan: 01
subsystem: lang
tags: [localization, translator, oncelock, validation-bridge, t-helper]

# Dependency graph
requires:
  - phase: 62-validation-rules-update
    provides: translate_validation() bridge and English JSON defaults
  - phase: 61-validation-bridge
    provides: OnceLock TranslatorFn callback mechanism
  - phase: 60-locale-context
    provides: locale()/set_locale() task-local context
provides:
  - Global Translator initialization via lang::init()
  - t()/trans()/choice() convenience helpers from ferro:: namespace
  - Automatic validation bridge registration at boot
affects: [64-cli-scaffolding, 65-mcp-documentation, 66-tests-polish]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "OnceLock<Translator> for global singleton set at boot"
    - "Graceful degradation: t() returns key as-is when no translator"
    - "Validation bridge auto-registered during lang::init()"

key-files:
  created:
    - framework/src/lang/init.rs
  modified:
    - framework/src/lang/mod.rs
    - framework/src/app.rs
    - framework/src/lib.rs

key-decisions:
  - "init() called after config_fn() so user can override LangConfig before translator loads"
  - "Validation bridge registered inside init() on successful load, not separately"
  - "choice() aliased as lang_choice to avoid name collisions in ferro:: namespace"

patterns-established:
  - "Framework auto-initialization: lang::init() called in Application::run() boot sequence"

# Metrics
duration: 4min
completed: 2026-02-13
---

# Phase 63 Plan 01: Framework Integration Summary

**OnceLock Translator with t()/trans()/choice() helpers wired into Application boot, validation bridge auto-registered**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-13T17:12:08Z
- **Completed:** 2026-02-13T17:16:41Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Global `OnceLock<Translator>` storage with `init()` that loads from `LangConfig` and auto-registers the validation bridge
- `t()`, `trans()`, and `choice()` convenience helpers available from `ferro::` namespace
- `lang::init()` called automatically during `Application::run()` after config, before bootstrap
- Graceful degradation: all helpers return key as-is when no translator loaded (no panics)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create lang::init() and t()/trans() helpers** - `9ea91f3` (feat)
2. **Task 2: Wire lang::init() into Application boot and add re-exports** - `4d14b0c` (feat)
3. **Task 3: Add tests and verify full build** - No separate commit (tests included in Task 1, verification passed)

## Files Created/Modified

- `framework/src/lang/init.rs` - Global Translator storage, init(), t()/trans()/choice(), validation bridge fn
- `framework/src/lang/mod.rs` - Added init module and re-exports for lang_init, t, trans, lang_choice
- `framework/src/app.rs` - Call lang::init::init() in Application::run() boot sequence
- `framework/src/lib.rs` - Re-export lang_init, t, trans, lang_choice from ferro:: namespace

## Decisions Made

- `init()` placed after `config_fn()` so users can override `LangConfig` before translator loads
- Validation bridge registered inside `init()` on successful load, keeping wiring automatic
- Used `lang_choice` and `lang_init` aliases in re-exports to avoid name collisions

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Framework integration complete: `t()`, `trans()`, `lang_choice()` available from `ferro::` namespace
- `lang_init()` auto-called during `Application::run()` boot after Config::init()
- Validation bridge auto-wired when Translator loads successfully
- Ready for Phase 64 (CLI Scaffolding)

---
*Phase: 63-framework-integration*
*Completed: 2026-02-13*
