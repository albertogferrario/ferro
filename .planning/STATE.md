# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-09)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v3.0 JSON-UI — JSON-based UI rendering as alternative to Inertia (in progress)

## Current Position

Phase: 25 of 32 (Data Binding)
Plan: 2 of 2 in current phase
Status: Phase complete
Last activity: 2026-02-09 — Completed 25-02-PLAN.md

Progress: ███████░░░ 70% (v3.0)

## Milestone Summary

| Milestone | Phases | Plans | Status | Shipped |
|-----------|--------|-------|--------|---------|
| v1.0 DX Overhaul | 1-12 | 18 | ✅ Complete | 2026-01-16 |
| v2.0 Rebrand | 13-22 | 13 | ✅ Complete | 2026-01-16 |
| v2.0.1 Macro Fix | 22.1-22.3 | 6 | ✅ Complete | 2026-01-17 |
| v2.0.2 Type Generator Fixes | 22.4-22.9 | 6 | ✅ Complete | 2026-01-17 |
| v2.0.3 DO Apps Deploy | 22.10 | 1 | ✅ Complete | 2026-01-17 |
| v2.1 Inertia DX & Fixes | 33-34 | 4 | ✅ Complete | 2026-01-17 |
| v2.2 CLI Improvements | 35-37 | 5 | ✅ Complete | 2026-02-09 |
| v3.0 JSON-UI | 23-32 | 7/? | 🚧 In Progress | - |

## Accumulated Context

### Key Decisions (v3.0)

| Phase | Decision | Rationale |
|-------|----------|-----------|
| 23 | Serde tagged enum for Component (`type` field) | Clean JSON with `{"type": "Card", ...}` |
| 23 | Serde untagged enum for Visibility | Clean `{"and": [...]}` syntax without type field |
| 23 | ComponentNode wraps Component via flatten | Shared key/action/visibility without duplication |
| 23 | HttpMethod serializes UPPERCASE | Standard HTTP method format |
| 23 | Visibility aliased as JsonUiVisibility in framework | Avoids name collision with ferro-storage Visibility |
| 24 | ButtonVariant aligned to shadcn/ui (6 variants) | CVA pattern consistency with shadcn ecosystem |
| 24 | BadgeVariant aligned to shadcn/ui (4 variants) | Matches standard component library conventions |
| 24 | AlertVariant kept as Info/Success/Warning/Error | Pragmatic deviation from shadcn — richer for CRUD apps |
| 24 | Shared Size enum for cross-component sizing | Avoids variant sprawl across components |
| 24 | Checkbox/Switch identical props (visual distinction) | Frontend renderer handles visual difference |
| 24 | DescriptionItem reuses ColumnFormat from Table | Consistent formatting across data display components |
| 24 | Full re-export of all JSON-UI types from framework | All 20 component types available via `use ferro_rs::*` |
| 25 | Simple slash-separated paths (not full JSONPath) | Trivial implementation, easy path generation |
| 25 | data_path on form field components only | Table already has data_path; non-form components don't pre-fill |
| 25 | data field on JsonUiView after title, before components | Logical ordering: metadata then content |
| 25 | render_json explicit data wins over embedded | Explicit parameter is "live" handler data; embedded is for self-contained views |

### Pending Todos

None.

### Blockers/Concerns

**Pre-existing (unrelated to milestones):**
1. ferro-storage has unimplemented trait methods
2. Flaky shared state in test_different_methods_tracked_separately
3. test_globals_css_not_empty expects tailwind in CSS

### Roadmap Evolution

- v1.0 DX Overhaul complete: 12 phases, 18 plans (2026-01-15 to 2026-01-16)
- v2.0 Rebrand complete: 10 phases, 13 plans (2026-01-16)
- v2.0.1 Macro Fix complete: 3 phases (Phase 22.1-22.3) (2026-01-17)
- v2.0.2 Type Generator Fixes complete: 6 phases, 6 plans (Phase 22.4-22.9) (2026-01-17)
- v2.0.3 DO Apps Deploy complete: 1 phase, 1 plan (Phase 22.10) (2026-01-17)
- v2.1 Inertia DX & Fixes complete: 2 phases, 4 plans (Phase 33-34) (2026-01-17)
- v2.2 CLI Improvements complete: 3 phases, 5 plans (Phase 35-37) (2026-02-09)
- v3.0 JSON-UI: 10 phases planned (Phases 23-32)

## Session Continuity

Last session: 2026-02-09
Stopped at: Completed 25-02-PLAN.md — Phase 25 Data Binding complete
Resume file: None
