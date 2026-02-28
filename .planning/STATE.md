# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v9.0 Service Projections

## Current Position

Phase: 85 (State Machines)
Plan: Not started
Status: Ready to plan
Last activity: 2026-02-28 — Phase 84 complete (2 plans, 8 commits)

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
| v6.0 ferro-lang — Localization | 58-66 | 11 | Complete | 2026-02-13 |
| v6.1 Fix Known Issues | 67 | 1 | Complete | 2026-02-24 |
| v7.0 Resend Integration | 68 | 3 | Complete | 2026-02-25 |
| v7.1 Static File Serving | 69 | 1 | Complete | 2026-02-25 |
| v7.4 Security Hardening | 72-74 | 5 | Complete | 2026-02-26 |
| v7.5 Type Generator Fix | 75 | 1 | Complete | 2026-02-27 |
| v7.6 Default API Scaffold | 76 | 4 | Complete | 2026-02-27 |
| v7.7 Validate & Fix API Scaffold | 77 | 3 | Complete | 2026-02-28 |
| v7.8 Memory Leak Fixes | 78 | 3 | Complete | 2026-02-28 |
| v8.0 Consumer MCP — OpenAPI Bridge | 79-82 | 11 | Complete | 2026-02-28 |
| v8.1 API DX Polish | 83 | 5 | Complete | 2026-02-28 |
| v9.0 Service Projections | 84-93 | 4/25 | In Progress | - |

## Accumulated Context

### Key Decisions

Archived to PROJECT.md and milestone archive files.

**Phase 84-01:**
- Consuming builder (`mut self -> Self`) for ServiceDef, matching workspace convention
- 18 known FieldMeaning variants + Custom(String) untagged fallback
- 10 DataType variants (abstract categories, not database types)
- infer_meaning() with 7 rules from existing CLI patterns

### Roadmap Evolution

- 21 milestones shipped, 184 plans total
- v9.0 created: Service Projections — ServiceDef→IntentGraph→Renderer architecture, 10 phases (Phase 84-93)

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-28
Stopped at: Phase 84 complete — ferro-projections crate with all types + 23 tests, ready for Phase 85
Resume file: None
