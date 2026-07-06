---
phase: 76-default-api-scaffold
plan: 01
subsystem: api
tags: [api-key, openapi, utoipa, sha2, redoc, middleware]

# Dependency graph
requires: []
provides:
  - API key generation, hashing, and constant-time verification
  - ApiKeyProvider trait and ApiKeyMiddleware for route protection
  - OpenAPI spec builder from route metadata
  - ReDoc HTML and JSON spec handlers with OnceLock caching
  - utoipa and utoipa-redoc re-exported from framework
affects: [76-default-api-scaffold]

# Tech tracking
tech-stack:
  added: [sha2 0.10, subtle 2.5, utoipa 5.4, utoipa-redoc 6.0]
  patterns: [prefix-based API key lookup, constant-time hash comparison, auto-generated OpenAPI from route metadata]

key-files:
  created:
    - framework/src/api/mod.rs
    - framework/src/api/api_key.rs
    - framework/src/api/openapi.rs
  modified:
    - framework/Cargo.toml
    - framework/src/lib.rs

key-decisions:
  - "SHA-256 for API key hashing (not bcrypt) — correct for high-entropy random keys, avoids per-request bottleneck"
  - "Constant-time comparison via subtle crate — prevents timing attacks on hash verification"
  - "OnceLock caching for OpenAPI spec and ReDoc HTML — generated once on first call, zero per-request cost"
  - "ApiKeyProvider trait resolved from service container — storage-agnostic, testable via container faking"
  - "Naive singularization (strip trailing 's') for auto-summaries — sufficient for CRUD resource naming"
  - "rand 0.8 API (thread_rng + gen_range) — matches existing framework dependency version"

patterns-established:
  - "API key format: fe_{env}_{43 random base62 chars} with 16-char prefix for DB lookup"
  - "ApiKeyInfo stored in request extensions for downstream handler access"
  - "OpenAPI spec builder consumes RouteInfo array and filters by api_prefix"

# Metrics
duration: 18min
completed: 2026-02-27
---

# Phase 76, Plan 01: API Key + OpenAPI Foundation Summary

**API key auth with SHA-256/constant-time verification and OpenAPI spec builder from route metadata using utoipa**

## Performance

- **Duration:** 18 min
- **Started:** 2026-02-27T00:00:00Z
- **Completed:** 2026-02-27T00:18:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- API key generation in `fe_{env}_{random}` format with prefix-based lookup and SHA-256 hashing
- ApiKeyMiddleware with scope checking, ApiKeyProvider trait for storage-agnostic verification
- OpenAPI spec builder that auto-generates operations from Ferro route metadata
- Auto-summary generation (GET collection -> "List X", GET item -> "Get X", etc.)
- ReDoc HTML documentation handler and JSON spec handler with OnceLock caching
- utoipa and utoipa-redoc re-exported for advanced user customization

## Task Commits

Each task was committed atomically:

1. **Task 1: API key module** - `dc4ff5f` (feat)
2. **Task 2: OpenAPI spec builder** - `3529940` (feat)

**Formatting cleanup:** `3070650` (chore: apply formatting and update lockfile)

## Files Created/Modified
- `framework/src/api/mod.rs` - API module declaration
- `framework/src/api/api_key.rs` - Key generation, hashing, verification, provider trait, middleware
- `framework/src/api/openapi.rs` - Spec builder, auto-summary, tag extraction, handler helpers
- `framework/Cargo.toml` - Added sha2, subtle, utoipa, utoipa-redoc dependencies
- `framework/src/lib.rs` - Re-exports for all new public types

## Decisions Made
- Used SHA-256 (not bcrypt) for API key hashing — correct for high-entropy random keys
- Used `subtle::ConstantTimeEq` for hash comparison to prevent timing attacks
- Used `OnceLock` for spec caching (not per-request regeneration)
- ApiKeyProvider resolved from service container via `App::make::<dyn ApiKeyProvider>()`
- Used rand 0.8 API (thread_rng + gen_range) to match existing framework dependency

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- API key and OpenAPI modules ready for CLI scaffold (Plan 03) to wire into generated code
- Plan 02 (MCP CRUD tools) can proceed independently
- All types exported from framework root for user access

---
*Phase: 76-default-api-scaffold*
*Completed: 2026-02-27*
