# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-09)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v3.0 JSON-UI — JSON-based UI rendering as alternative to Inertia (in progress)

## Current Position

Phase: 30 of 32 (CLI Scaffolding)
Plan: 2 of 2 in current phase
Status: Phase complete
Last activity: 2026-02-09 — Completed 30-02-PLAN.md

Progress: █████████░ 93% (v3.0)

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
| v3.0 JSON-UI | 23-32 | 13/? | 🚧 In Progress | - |

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
| 26 | url field added directly to Action struct (Option<String>) | Simpler than separate ResolvedAction type, works for both HTML and JSON output |
| 26 | Callback-based resolver Fn(&str) -> Option<String> | Keeps ferro-json-ui decoupled from framework route registry |
| 26 | Clone view before resolution in render pipeline | Immutable API, caller's view never mutated |
| 26 | Non-strict resolve_actions in render pipeline | Missing routes produce url: None, handled downstream |
| 27 | Explicit component errors take priority over validation map | Do-not-overwrite rule: resolve_errors skips fields with existing error |
| 27 | resolve_errors_all joins with ". " separator | Readable concatenation of multiple validation messages |
| 27 | resolve_with_errors sets view.errors alongside field-level | Dual consumption: component-level + view-level for frontend |
| 27 | render_validation_error delegates via .all() | Single indirection from framework type to HashMap |
| 28 | GET actions wrap in `<a>`, non-GET render as-is | Only safe HTTP method for link navigation |
| 28 | Container components get basic SSR in Plan 01 | Full treatment deferred to Plan 02 |
| 28 | compute_page_range shows max 7 pages with ellipsis | Readable pagination for large datasets |
| 28 | Modal uses details/summary for no-JS progressive enhancement | Functional SSR without JavaScript |
| 28 | Tabs SSR renders only default_tab content | Tab switching requires JS, out of scope for Phase 28 |
| 28 | Framework pre dump replaced with render_to_html output | Real HTML pages instead of JSON placeholder |
| 29 | All layouts/partials/registry in single layout.rs module | Simpler than separate files for 3 small partial functions |
| 29 | html_escape made pub(crate) in render.rs | Cross-module reuse without duplication |
| 29 | AppLayout uses empty partials by default | Users create custom Layout impls with real NavItem data |
| 29 | Raw values to LayoutContext, layouts handle escaping | Avoids double-escaping since base_document/ferro_wrapper already escape |
| 29 | build_response helper for shared render logic | Eliminates duplication between render_with_config and render_with_errors_config |
| 30 | Blocking reqwest for Anthropic API in CLI | CLI main is synchronous; tokio only used for db commands |
| 30 | Component catalog as hardcoded const string | Fast, no file I/O, embedded in binary |
| 30 | Regex-based model scanning (not syn) | Speed and simplicity for AI context assembly |
| 30 | Graceful AI fallback chain | Missing API key or AI error silently uses static template |
| 30 | Sonnet default instead of Opus | ~5x cost reduction for code generation |
| 30 | Assistant prefill //! for code-only output | Eliminates strip_markdown_fences workaround |
| 30 | System/user prompt separation with cache_control | Cacheable static content, dynamic per-request context |

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
Stopped at: Completed 30-02-PLAN.md (Phase 30 complete)
Resume file: None
