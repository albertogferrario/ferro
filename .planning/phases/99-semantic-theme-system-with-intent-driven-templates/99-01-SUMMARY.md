---
phase: 99-semantic-theme-system-with-intent-driven-templates
plan: "01"
subsystem: ui
tags: [ferro-theme, tailwind, css-tokens, intent-templates, theming, sdui]

# Dependency graph
requires: []
provides:
  - ferro-theme crate with Theme struct (css + templates), default embedded CSS, filesystem loader
  - ThemeError enum (Io/Json/NotFound) for loading failures
  - ThemeTemplates with 7 Optional intent fields for partial override deserialization
  - IntentModeTemplates and IntentSlotTemplate for slot-based intent layout schema
  - token module documenting 23 fixed semantic token slots (ferro-theme/v1 vocabulary)
  - Default Tailwind v4 @theme CSS with light + dark mode support (23 semantic token slots)
affects:
  - 99-02 (ferro-json-ui integration — will depend on ferro-theme for token types)
  - 99-03 (ferro-projections integration — will consume ThemeTemplates for intent rendering)
  - 99-04 (framework ThemeMiddleware — per-request resolution will use Theme struct)

# Tech tracking
tech-stack:
  added:
    - ferro-theme crate (new workspace member)
    - thiserror 2 (error enum derivation)
    - serde + serde_json (ThemeTemplates deserialization)
    - tempfile 3 (dev-dependency for loader tests)
  patterns:
    - Embedded const CSS asset pattern (matches ferro-json-ui FERRO_RUNTIME_JS)
    - Pure data + loader crate (mirrors ferro-lang Translator pattern)
    - Partial JSON deserialization with #[serde(default)] on Option fields
    - thiserror #[from] for transparent error conversion (Io, Json variants)

key-files:
  created:
    - ferro-theme/Cargo.toml
    - ferro-theme/src/lib.rs
    - ferro-theme/src/error.rs
    - ferro-theme/src/token.rs
    - ferro-theme/src/template.rs
    - ferro-theme/src/loader.rs
    - ferro-theme/assets/default.css
  modified:
    - Cargo.toml (added ferro-theme to workspace members)

key-decisions:
  - "thiserror = 2 for ferro-theme (matching ferro-lang, ferro-stripe convention for new leaf crates)"
  - "23 semantic token slots in ferro-theme/v1 vocabulary: 6 surface + 8 role + 4 radius + 3 shadow + 2 typography"
  - "ThemeTemplates uses #[serde(default)] on all 7 Option fields — partial JSON overrides work correctly"
  - "Theme::from_path() treats missing theme.json as empty ThemeTemplates (not an error)"
  - "token module documents fixed vocabulary as pub const TOKEN_* string constants"

patterns-established:
  - "embedded CSS asset: const DEFAULT_THEME_CSS: &str = include_str!(\"../assets/default.css\") in loader.rs"
  - "loader struct pattern: Theme { css: String, templates: ThemeTemplates } with default_theme() + from_path()"
  - "ThemeError follows LangError pattern: Io(#[from]), Json(#[from]), specific variant (NotFound)"

requirements-completed: [THEME-01, THEME-02, THEME-03]

# Metrics
duration: 6min
completed: 2026-03-12
---

# Phase 99 Plan 01: ferro-theme Crate Summary

**Standalone `ferro-theme` crate with 23-slot semantic token vocabulary, Tailwind v4 `@theme` default CSS, partial-override-capable ThemeTemplates, and filesystem loader — 17 tests all passing**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-12T02:51:07Z
- **Completed:** 2026-03-12T02:56:58Z
- **Tasks:** 1
- **Files modified:** 9 (7 created + 2 modified)

## Accomplishments
- Created `ferro-theme` workspace crate compiling cleanly with zero clippy warnings
- Embedded `default.css` with 23 semantic token slots in Tailwind v4 `@theme` format covering light + dark modes
- `ThemeTemplates` deserialization handles partial JSON (`{}` → all-None, any subset → only specified intents set)
- `Theme::from_path()` loads `tokens.css` + optional `theme.json`, with proper `ThemeError` variants
- 17 tests (16 unit + 1 doctest) covering all types, behaviors, and error conditions

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ferro-theme crate with types and default theme** - `5e30a4e` (feat)

**Plan metadata:** (see final docs commit)

## Files Created/Modified
- `Cargo.toml` - Added ferro-theme to workspace members list
- `ferro-theme/Cargo.toml` - Crate manifest with serde, serde_json, thiserror deps
- `ferro-theme/src/lib.rs` - Public re-exports: Theme, ThemeError, ThemeTemplates, IntentSlotTemplate, IntentModeTemplates
- `ferro-theme/src/error.rs` - ThemeError enum with Io, Json, NotFound variants
- `ferro-theme/src/token.rs` - 23 TOKEN_* string constants documenting ferro-theme/v1 vocabulary
- `ferro-theme/src/template.rs` - IntentSlotTemplate, IntentModeTemplates, ThemeTemplates with serde derives
- `ferro-theme/src/loader.rs` - Theme struct with from_path() and default_theme()
- `ferro-theme/assets/default.css` - Default theme: @theme syntax, 23 tokens, light + dark modes

## Decisions Made
- `thiserror = "2"` matching ferro-lang/ferro-stripe convention for new leaf crates
- 23 semantic slots: 6 surface (`background`, `surface`, `card`, `border`, `text`, `text-muted`) + 8 role (`primary`, `primary-foreground`, `secondary`, `secondary-foreground`, `accent`, `destructive`, `success`, `warning`) + 4 radius (`sm`, `md`, `lg`, `full`) + 3 shadow (`sm`, `md`, `lg`) + 2 typography (`font-family-sans`, `font-family-mono`)
- `#[serde(default)]` on all `Option<IntentModeTemplates>` fields enables partial JSON deserialization
- Missing `theme.json` is not an error — defaults to `ThemeTemplates::default()` (all-None)
- `token` module is `pub mod` (documentation/validation use) while `error`, `loader`, `template` are private `mod`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo fmt` required module declarations in alphabetical order within `lib.rs`: `mod template` before `pub mod token`. Fixed before commit.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `ferro-theme` is ready to be added as a dependency in `ferro-json-ui` and `ferro-projections`
- `Theme::default_theme()` provides the embedded CSS for injection into HTML `<head>` via layout system
- `ThemeTemplates` schema is ready for use in `JsonUiRenderer` to replace hardcoded intent layouts

## Self-Check: PASSED

- [x] `ferro-theme/Cargo.toml` exists
- [x] `ferro-theme/src/lib.rs` exists
- [x] `ferro-theme/src/error.rs` exists
- [x] `ferro-theme/src/token.rs` exists
- [x] `ferro-theme/src/template.rs` exists
- [x] `ferro-theme/src/loader.rs` exists
- [x] `ferro-theme/assets/default.css` exists
- [x] Commit 5e30a4e exists (feat(99-01): create ferro-theme crate)
- [x] 17 tests pass (16 unit + 1 doctest)
- [x] cargo clippy -p ferro-theme --all-targets -- -D warnings: clean
- [x] cargo fmt -p ferro-theme -- --check: clean

---
*Phase: 99-semantic-theme-system-with-intent-driven-templates*
*Completed: 2026-03-12*
