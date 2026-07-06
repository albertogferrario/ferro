---
phase: 64-cli-scaffolding
plan: 01
subsystem: cli
tags: [cli, localization, scaffolding, templates, make-command]

# Dependency graph
requires:
  - phase: 63-framework-integration
    provides: lang::init(), t()/trans()/choice() helpers, validation bridge
provides:
  - ferro make:lang {locale} command for scaffolding translation files
  - Updated ferro new templates with lang/en/ and locale env vars
affects: [65-mcp-documentation, 66-tests-polish]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "make:lang follows same pattern as make:middleware for CLI commands"
    - "Template files in ferro-cli/src/templates/files/lang/ served via include_str!"

key-files:
  created:
    - ferro-cli/src/commands/make_lang.rs
    - ferro-cli/src/templates/files/lang/validation.json.tpl
    - ferro-cli/src/templates/files/lang/app.json.tpl
  modified:
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
    - ferro-cli/src/templates/make.rs
    - ferro-cli/src/templates/files/root/env.example.tpl
    - ferro-cli/src/templates/files/root/env.tpl
    - ferro-cli/src/commands/new.rs

key-decisions:
  - "Locale validation: 2-letter base + optional hyphenated subtags (en, pt-br, zh-hans)"
  - "Template files in ferro-cli/src/templates/files/lang/ using include_str! pattern"

patterns-established:
  - "make:lang locale validation differs from make:middleware (locale codes, not Rust identifiers)"

# Metrics
duration: 3min
completed: 2026-02-13
---

# Phase 64 Plan 01: CLI Scaffolding Summary

**ferro make:lang command and updated ferro new templates with localization defaults (lang/en/, .env locale vars)**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-13T18:01:35Z
- **Completed:** 2026-02-13T18:04:06Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- `ferro make:lang {locale}` command scaffolds lang/{locale}/ with validation.json and app.json
- Locale validation accepts standard codes (en, fr, pt-br, zh-hans) and rejects invalid formats
- `ferro new` creates lang/en/ with English starter translations out of the box
- .env.example documents LOCALIZATION section with APP_LOCALE, APP_FALLBACK_LOCALE, LANG_PATH
- .env includes sensible locale defaults (APP_LOCALE=en, APP_FALLBACK_LOCALE=en)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create make:lang command and templates** - `c3ecd4d` (feat)
2. **Task 2: Update ferro new templates with localization defaults** - `3711fec` (feat)

## Files Created/Modified
- `ferro-cli/src/commands/make_lang.rs` - make:lang command implementation with locale validation
- `ferro-cli/src/templates/files/lang/validation.json.tpl` - English validation messages template
- `ferro-cli/src/templates/files/lang/app.json.tpl` - Starter app translations template
- `ferro-cli/src/commands/mod.rs` - Added make_lang module
- `ferro-cli/src/main.rs` - Added MakeLang variant to Commands enum and match arm
- `ferro-cli/src/templates/make.rs` - Added lang_validation_json() and lang_app_json() functions
- `ferro-cli/src/templates/files/root/env.example.tpl` - Added LOCALIZATION section
- `ferro-cli/src/templates/files/root/env.tpl` - Added APP_LOCALE and APP_FALLBACK_LOCALE
- `ferro-cli/src/commands/new.rs` - Added lang/en/ directory and file creation

## Decisions Made
- Locale validation uses 2-letter base + optional hyphenated subtags (not Rust identifiers) since locale codes like "pt-br" are valid
- Template files stored in ferro-cli/src/templates/files/lang/ using the established include_str! pattern

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 64 complete, ready for Phase 65 (MCP & Documentation)
- make:lang command and updated project templates are available
- All existing tests pass (184/184)

---
*Phase: 64-cli-scaffolding*
*Completed: 2026-02-13*
