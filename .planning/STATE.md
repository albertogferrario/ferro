# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-09)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v4.0 Production Readiness

## Current Position

Phase: 43 of 46 (Rate Limiting)
Plan: 1 of 3 in current phase
Status: In progress
Last activity: 2026-02-10 - Completed 43-01-PLAN.md

Progress: ██░░░░░░░░ 25%

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
| v3.0 JSON-UI | 23-32 | 24 | ✅ Complete | 2026-02-09 |
| v4.0 Production Readiness | 38-46 | 13/? | 🚧 In Progress | - |

## Accumulated Context

### Key Decisions

Archived to PROJECT.md and milestone archive files.

| Phase | Decision | Rationale |
|-------|----------|-----------|
| 38-01 | #[serial] over per-test MetricsStore refactor | Minimal change, same safety guarantee for global state tests |
| 38-01 | EnvGuard without unsafe blocks | Rust 2021 edition: env::set_var/remove_var are safe |
| 38-02 | S3 facade returns S3Driver that errors on use | Avoids panic at initialization; defers errors to actual usage |
| 38-02 | Remove Tailwind CDN entirely (not configurable) | Vite handles CSS; CDN was redundant and assumed Tailwind usage |
| 39-03 | Instructional output for provider/routes instead of auto-modify | Safer: user may have custom code in those files |
| 39-03 | ALTER TABLE migration approach for auth fields | Users table likely already exists in projects |
| 39-02 | Alias ferro::AuthMiddleware as SessionAuthMiddleware | Avoids name conflict with app's existing header-based AuthMiddleware |
| 40-01 | 401 via FrameworkError::domain not Unauthorized | Unauthorized is 403 (authorization); 401 is authentication failure |
| 40-01 | AuthUser counts as one FromRequest param per handler | Existing framework constraint: one body-consuming extractor per handler |
| 41-01 | Enable serde_json preserve_order feature | ResourceMap needs insertion-order field output; BTreeMap default sorts alphabetically |
| 41-01 | TCP loopback helper for Request in unit tests | hyper::body::Incoming has no public constructor; TCP loopback creates real Request |
| 41-02 | ferro:: prefix in generated macro code | Matches existing macro patterns (handler, model); uses framework re-exports |
| 41-02 | From<Model> copies all fields including skipped | Skip only affects JSON output; users can access skipped fields programmatically |
| 41-03 | Profile handler uses Auth::user_as instead of AuthUser extractor | AuthUser consumes Request; Auth::user_as allows access to both req and user |
| 41-03 | Skip MCP application_info update | No structured feature list exists; deferred to Phase 46 |
| 42-01 | Relative URLs for pagination links | Works behind reverse proxies without host configuration |
| 42-01 | form_urlencoded for pagination URL building | Already a framework dependency; proper encoding |
| 43-01 | eprintln! for rate limiter warnings | Consistent with framework pattern; no tracing dependency |
| 43-01 | OnceLock<DashMap> for limiter registry | Static, thread-safe, no initialization order dependency |
| 43-01 | Fail-open on cache errors and missing limiters | Availability over strictness; never block requests due to infra failure |

### Pending Todos

None.

### Blockers/Concerns

None (pre-existing blockers moved to Phase 38 scope).

### Roadmap Evolution

- v1.0 DX Overhaul complete: 12 phases, 18 plans (2026-01-15 to 2026-01-16)
- v2.0 Rebrand complete: 10 phases, 13 plans (2026-01-16)
- v2.0.1 Macro Fix complete: 3 phases (Phase 22.1-22.3) (2026-01-17)
- v2.0.2 Type Generator Fixes complete: 6 phases, 6 plans (Phase 22.4-22.9) (2026-01-17)
- v2.0.3 DO Apps Deploy complete: 1 phase, 1 plan (Phase 22.10) (2026-01-17)
- v2.1 Inertia DX & Fixes complete: 2 phases, 4 plans (Phase 33-34) (2026-01-17)
- v2.2 CLI Improvements complete: 3 phases, 5 plans (Phase 35-37) (2026-02-09)
- v3.0 JSON-UI complete: 10 phases, 24 plans (Phases 23-32) (2026-02-09)
- Milestone v4.0 Production Readiness created: auth, API resources, rate limiting, real-time, 9 phases (Phase 38-46)
- Phase 38 complete: 2 plans, 1 wave — test isolation fixes + storage/Inertia cleanup (2026-02-09)
- Phase 39 complete: 4 plans, 2 waves — user model/provider, auth controllers/routes, make:auth CLI, auth docs (2026-02-09)
- Phase 40 complete: 2 plans, 2 waves — AuthUser/OptionalUser extractors + sample app, templates, docs (2026-02-10)
- Phase 41 complete: 3 plans, 3 waves — Resource trait + ResourceMap, ApiResource derive macro, CLI + docs + sample app (2026-02-10)
- Phase 42 complete: 3 plans, 2 waves — PaginationMeta/ResourceCollection, when_loaded/collection(), docs + MCP templates (2026-02-10)

## Session Continuity

Last session: 2026-02-10
Stopped at: Completed 43-01-PLAN.md
Resume file: None
