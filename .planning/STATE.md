# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-10)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v6.0 ferro-lang — Localization

## Current Position

Phase: 66 of 66 (Tests & Polish)
Plan: 3 of 3 in current phase
Status: Phase complete
Last activity: 2026-02-13 — Completed 66-03-PLAN.md

Progress: ██████████ 100%

## Milestone Summary

| Milestone | Phases | Plans | Status | Shipped |
|-----------|--------|-------|--------|---------|
| v1.0 DX Overhaul | 1-12 | 18 | Complete | 2026-01-16 |
| v2.0 Rebrand | 13-22 | 13 | Complete | 2026-01-16 |
| v2.0.1 Macro Fix | 22.1-22.3 | 6 | Complete | 2026-01-17 |
| v2.0.2 Type Generator Fixes | 22.4-22.9 | 6 | Complete | 2026-01-17 |
| v2.0.3 DO Apps Deploy | 22.10 | 1 | Complete | 2026-01-17 |
| v2.1 Inertia DX & Fixes | 33-34 | 4 | Complete | 2026-01-17 |
| v2.2 CLI Improvements | 35-37 | 5 | Complete | 2026-02-09 |
| v3.0 JSON-UI | 23-32 | 24 | Complete | 2026-02-09 |
| v4.0 Production Readiness | 38-46 | 24 | Complete | 2026-02-10 |
| v5.0 Proximity — JSON-UI Field Test | 47-53 | 20 | Complete | 2026-02-10 |
| v5.1 Housekeeping | 54-57 | 5 | Complete | 2026-02-13 |
| v6.0 ferro-lang | 58-66 | 7/? | In Progress | - |

## Accumulated Context

### Key Decisions

Archived to PROJECT.md and milestone archive files.

| Phase | Decision | Rationale |
|-------|----------|-----------|
| 58 | Pre-merge fallback at load time | O(1) runtime lookup, no fallback chain per request |
| 58 | Normalize locales to lowercase+hyphens | Consistent lookup regardless of input format |
| 58 | Return key as-is when missing | Graceful degradation, no panics |
| 59 | LangConfig uses std::env::var directly | ferro-lang is standalone, no framework env helpers |
| 59 | ferro-lang re-exports use original names | LangError/LangConfig already unambiguous |
| 60 | locale() returns String not Option | Always has reasonable default via cascading fallback |
| 60 | LangMiddleware reads LangConfig from Config registry | No constructor params, consistent with framework pattern |
| 60 | Accept-Language: first tag only | Simple, sufficient; normalize_locale handles format |
| 61 | fn pointer over Box<dyn Fn> for TranslatorFn | No state capture needed, simpler type |
| 61 | OnceLock without RwLock | Translator set once at boot, never changes |
| 61 | pub(crate) translate_validation | Only validation rules call it, not external code |
| 62 | Nested keys for size rules (min.string/numeric/array) | Matches Laravel convention for type-specific messages |
| 62 | VALIDATION_TRANSLATOR pub(crate) | Enables integration tests within the crate |
| 63 | init() after config_fn() in Application::run() | User can override LangConfig before translator loads |
| 63 | Validation bridge registered inside init() | Automatic wiring, no separate registration step |
| 63 | lang_choice/lang_init aliases for re-exports | Avoid name collisions in ferro:: namespace |
| 64 | Locale validation: 2-letter base + optional hyphenated subtags | Locale codes differ from Rust identifiers |
| 64 | Template files via include_str! in make.rs | Consistent with all other make: commands |
| 65 | Lightweight .env parsing in MCP tool | No dependency added, consistent with other MCP tools |
| 65 | Coverage report compares against fallback locale | Single reference locale for missing key detection |

### Roadmap Evolution

- All planned milestones v1.0–v5.1 complete (11 milestones, 126 plans shipped)
- Milestone v6.0 created: ferro-lang localization, 9 phases (Phase 58-66)
- Phase 58 complete: ferro-lang crate with Translator, interpolation, pluralization
- Phase 59 complete: LangConfig, enriched LangError, framework integration
- Phase 60 complete: task_local locale context, LangMiddleware, locale()/set_locale()
- Phase 61 complete: OnceLock validation bridge with TranslatorFn callback
- Phase 62 complete: All 22 rules use translate_validation() with English fallback, default JSON
- Phase 63 complete: lang::init() with t()/trans()/choice() helpers, auto-boot, validation bridge wired
- Phase 64 complete: make:lang CLI command, ferro new templates with lang/en/ and locale env vars
- Phase 65 plan 01 complete: list_lang_files MCP tool, updated list_commands/application_info/MCP instructions
- Phase 65 plan 02 complete: Localization documentation page in docs/src/features/
- Phase 66 plan 01 complete: Loader, bridge, and init tests with tempfile
- Phase 66 plan 02 complete: Test coverage for all 23 validation rules
- Phase 66 plan 03 complete: is_valid_locale tests + sample app localization setup

### Pending Todos

None.

### Blockers/Concerns

None.

### Roadmap Evolution

v5.0 Proximity milestone created with 6 phases (47-52). First real-world test of JSON-UI and v4.0 features via a map-based social network app.

## Session Continuity

Last session: 2026-02-13
Stopped at: Completed 66-03-PLAN.md — Phase 66 complete (all 3 plans done)
Resume file: None
