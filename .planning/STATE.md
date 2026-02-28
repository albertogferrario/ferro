# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Agents can go from "I want an app that does X" to a working, deployed application with minimal friction.
**Current focus:** v8.0 Consumer MCP — OpenAPI Bridge

## Current Position

Phase: 79 (ferro-api-mcp Core)
Plan: 03 complete
Status: In progress
Last activity: 2026-02-28 — Plan 03 shipped (schema bridge, HTTP client)

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
| Security Hardening | 72-74 | 5 | Complete | 2026-02-26 |
| Type Generator Fix | 75 | 1 | Complete | 2026-02-27 |
| Default API Scaffold | 76 | 4 | Complete | 2026-02-27 |
| Validate & Fix API Scaffold | 77 | 3 | Complete | 2026-02-28 |
| Memory Leak Audit | 78 | 3 | Complete | 2026-02-28 |

## Accumulated Context

### Key Decisions

Archived to PROJECT.md and milestone archive files.

- Static file responses use hyper::Response<Full<Bytes>> directly (not HttpResponse) to preserve binary file integrity
- Static files checked before fallback handler in handle_request() to prevent SPA catch-all from intercepting asset requests
- HttpResponse body changed from String to Bytes; body() returns &str with from_utf8 fallback for backward compatibility
- bytes() constructor sets no default Content-Type; download() auto-detects from filename extension
- SecurityHeaders HSTS off by default to avoid breaking localhost over HTTP
- X-XSS-Protection set to 0 per OWASP (XSS Auditor can create vulnerabilities)
- CSP includes unsafe-inline/unsafe-eval for Inertia.js and Vite SPA compatibility
- SecurityHeaders placed after CSRF middleware in bootstrap so headers apply to all responses including CSRF rejections
- Nullable created_at for backward compat with existing sessions tables (NULL skips absolute check)
- destroy_for_user default trait impl returns error rather than panic
- Cookie max_age uses max(idle, absolute) so cookie outlives both server-side checks
- DatabaseSessionDriver with zero-duration lifetimes in logout_other_devices (destroy_for_user never reads them)
- DatabaseSessionDriver and SessionStore re-exported from framework root for admin flows
- Generated TypeScript files are fully self-contained (no shared.ts imports/re-exports) to prevent circular imports
- parse_shared_types kept for resolve_nested_types filtering (avoids regenerating user-defined types)
- MCP CRUD tools: column names derived from field names (SeaORM default), Postgres RETURNING / SQLite last_insert_rowid fallback
- Per-page capped at 100, created_at/updated_at skipped from required-field validation
- SHA-256 for API key hashing (not bcrypt) — correct for high-entropy random keys
- Constant-time comparison via subtle crate for API key hash verification
- OnceLock caching for OpenAPI spec and ReDoc HTML (generated once on first call)
- ApiKeyProvider resolved from service container via App::make (storage-agnostic)
- utoipa and utoipa-redoc re-exported from framework root for advanced customization
- make:api reuses syn AST visitor pattern from ferro-mcp list_models for model detection consistency
- Generated update handlers use conditional builder pattern (if let Some) for partial updates
- quote crate added as ferro-cli dependency for syn ToTokens trait
- DB::connection() errors in generated templates use map_err for consistent error handling
- Extracted normalize_page, normalize_per_page, find_missing_required_field as testable pure functions
- Per-page clamped to [1, 100] with .clamp() instead of .min(100) to prevent LIMIT 0
- Entity file stems singularized for correct model names (users.rs -> User, not Users)
- module_name separated from model name for correct crate::models:: import paths
- Generated resources use From<Model> (owned) for ApiResource derive macro compatibility
- Unmatched routes normalized to "UNMATCHED" bucket to prevent 404 path explosion DoS
- MAX_ROUTE_ENTRIES = 1000 cap as defense-in-depth for metrics HashMap
- InMemoryCache: moka::sync::Cache (not future) since CacheStore async_trait wraps synchronous operations
- Default capacity 10,000 entries matching ferro-cache MemoryStore convention
- CacheValue wrapper struct embeds optional TTL for Expiry trait per-entry reads
- ferro-cache: CacheValue wrapper with TTL for moka::future::Cache per-entry expiry via Expiry trait
- ferro-cache tags: HashSet instead of Vec prevents duplicates structurally
- moka 0.12 eviction listener fires on explicit removal/size eviction but not TTL expiry; lazy cleanup in tag_members as complement
- ferro-cache counters: bounded moka cache instead of unbounded DashMap
- ferro-api-mcp as standalone workspace binary (not library dependency of framework)
- ApiOperation::new() defaults input_schema to empty object schema for schema module to populate
- CLI uses EnvFilter with fallback to --log-level arg for tracing configuration
- Request body nested under "body" key in MCP input_schema to prevent name collisions with path/query params
- Pure helper functions (interpolate_path, build_query_params) for testability without HTTP mocking
- Missing path params leave {param} placeholder unchanged (graceful degradation)
- Non-JSON success responses wrapped as Value::String

### Roadmap Evolution

- All planned milestones v1.0–v7.1 complete (15 milestones, 150 plans shipped)
- Security hardening phases 72-74 added to roadmap
- Phase 75 complete: generate-types output made self-contained (no shared.ts imports/re-exports)
- Phase 76 complete: Default API scaffold with API key auth, OpenAPI, MCP CRUD, CLI make:api, and documentation
- Phase 77 complete: Validate & fix API scaffold — 5 template bugs fixed, 75 regression tests added
- Phase 78 complete: Memory leak audit — Plan 01 fixes metrics 404 explosion, Plan 02 replaces InMemoryCache with moka, Plan 03 fixes ferro-cache TTL/tags/counters
- Milestone v8.0 created: Consumer MCP — OpenAPI Bridge, 4 phases (79-82)
- Phase 79 Plan 01 complete: ferro-api-mcp crate scaffold with types, errors, and clap CLI
- Phase 79 Plan 03 complete: schema bridge (OpenAPI params to MCP JSON Schema) and HTTP client

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-02-28
Stopped at: Phase 79, Plan 03 complete — ready for Plan 04
Resume file: None
