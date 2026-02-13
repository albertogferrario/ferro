---
phase: 58-core-translator
plan: 01
subsystem: i18n
tags: [localization, i18n, json, interpolation, pluralization, ferro-lang]

requires:
  - phase: none
    provides: standalone crate, no prior phase dependencies

provides:
  - ferro-lang crate with Translator struct
  - JSON translation loading with dot-notation flattening
  - :param/:Param/:PARAM interpolation
  - pipe-separated pluralization with range syntax
  - locale fallback pre-merging

affects: [59-config-error-types, 60-locale-context, 61-validation-bridge, 62-validation-rules, 63-framework-integration]

tech-stack:
  added: [ferro-lang]
  patterns: [translator-singleton, fallback-pre-merge, dot-notation-keys]

key-files:
  created:
    - ferro-lang/Cargo.toml
    - ferro-lang/src/lib.rs
    - ferro-lang/src/error.rs
    - ferro-lang/src/loader.rs
    - ferro-lang/src/interpolation.rs
    - ferro-lang/src/pluralization.rs
    - ferro-lang/src/translator.rs
  modified:
    - Cargo.toml

key-decisions:
  - "Pre-merge fallback at load time for O(1) runtime lookup"
  - "Normalize all locale identifiers to lowercase with hyphens"
  - "Return key as-is when translation missing (no Option, no panic)"

patterns-established:
  - "Translator::load() reads {path}/{locale}/*.json at startup"
  - "Interpolation processes longer keys first to avoid partial replacement"
  - "Pluralization supports both simple pipe and {N}/[N,M]/[N,*] range syntax"

duration: 5min
completed: 2026-02-13
---

# Phase 58 Plan 01: Core Translator Summary

**ferro-lang crate with JSON translation loading, :param interpolation, pipe-separated pluralization with range syntax, and locale fallback pre-merging**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-13T14:47:02Z
- **Completed:** 2026-02-13T14:52:11Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- New ferro-lang workspace crate following ferro-cache patterns
- Translator loads JSON translations from filesystem with automatic locale discovery and normalization
- Fallback translations pre-merged at load time for O(1) runtime lookup
- Interpolation supports :param, :Param (ucfirst), :NAME (uppercase) with length-priority processing
- Pluralization handles simple pipe syntax and explicit {N}/[N,M]/[N,*] range syntax
- 28 comprehensive unit tests covering all modules

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ferro-lang crate scaffold and core types** - `d99fbcd` (feat)
2. **Task 2: Implement JSON loader and Translator struct** - `710035e` (feat)
3. **Task 3: Implement interpolation, pluralization, and tests** - `d6a8bf9` (feat)

## Files Created/Modified

- `Cargo.toml` - Added ferro-lang to workspace members
- `ferro-lang/Cargo.toml` - Crate manifest with serde, serde_json, thiserror, tracing deps
- `ferro-lang/src/lib.rs` - Public API re-exports (Translator, LangError)
- `ferro-lang/src/error.rs` - LangError enum (IoError, JsonError, NoTranslationsLoaded)
- `ferro-lang/src/loader.rs` - JSON file loading with locale normalization, flattening, fallback pre-merge
- `ferro-lang/src/interpolation.rs` - :param replacement with case variants and length-priority
- `ferro-lang/src/pluralization.rs` - Plural form selection with pipe and range syntax
- `ferro-lang/src/translator.rs` - Translator struct with load/get/choice/has/locales and 12 integration tests

## Decisions Made

- Pre-merge fallback translations at load time so runtime lookup is a single HashMap::get
- Normalize all locale identifiers to lowercase with hyphens (en_US -> en-us)
- Return key as-is when translation is missing (no Option, no panic, tracing::warn logged)
- Process interpolation parameters longest-key-first to avoid partial replacement

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ferro-lang crate is ready for Phase 59 (Config & Error Types)
- Translator API is stable: load(), get(), choice(), has(), locales()
- No blockers or concerns

---
*Phase: 58-core-translator*
*Completed: 2026-02-13*
