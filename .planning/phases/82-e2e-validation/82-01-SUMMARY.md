---
phase: 82-e2e-validation
plan: 01
subsystem: api
tags: [openapi, api-key, crud, sea-orm, middleware]

# Dependency graph
requires:
  - phase: 76-default-api-scaffold
    provides: make:api templates, ApiKeyMiddleware, OpenApiConfig
  - phase: 77-validate-fix-api-scaffold
    provides: validated make:api template patterns
provides:
  - Sample app with /api/v1/users CRUD protected by ApiKeyMiddleware
  - OpenAPI spec served at /api/openapi.json
  - ReDoc documentation at /api/docs
  - API key infrastructure (migration, model, provider)
affects: [82-e2e-validation]

# Tech tracking
tech-stack:
  added: []
  patterns: [api-key-auth, openapi-docs, crud-handlers]

key-files:
  created:
    - app/src/api/mod.rs
    - app/src/api/docs.rs
    - app/src/api/routes.rs
    - app/src/api/user_api.rs
    - app/src/migrations/m20260228_create_api_keys_table.rs
    - app/src/models/api_key.rs
    - app/src/models/entities/api_keys.rs
    - app/src/providers/api_key_provider.rs
  modified:
    - app/src/bootstrap.rs
    - app/src/main.rs
    - app/src/routes.rs
    - app/src/migrations/mod.rs
    - app/src/models/mod.rs
    - app/src/models/entities/mod.rs
    - app/src/providers/mod.rs
    - framework/src/lib.rs

key-decisions:
  - "Entity file separate from model file: api_keys entity in entities/ with model re-export and trait impls in models/api_key.rs"
  - "Request types defined inline in user_api.rs rather than separate requests module (plan scope limited to listed files)"
  - "openapi_docs_response and openapi_json_response re-exported from framework root (were defined but not public)"
  - "Old /api group route removed and replaced by api_routes() function call in routes! macro"

patterns-established:
  - "API module structure: api/mod.rs, api/routes.rs, api/docs.rs, api/{model}_api.rs"
  - "GroupDef functions registered directly in routes! macro (no .into() needed)"

# Metrics
duration: 12min
completed: 2026-02-28
---

# Phase 82 Plan 01: API Layer for Sample App

**User CRUD at /api/v1/users with API key auth, OpenAPI spec at /api/openapi.json, and ReDoc at /api/docs**

## Performance

- **Duration:** 12 min
- **Tasks:** 2
- **Files created:** 8
- **Files modified:** 8

## Accomplishments
- API key infrastructure: migration, SeaORM entity, model with query builder, database-backed provider with constant-time hash verification
- User CRUD handlers (index with pagination, show, store, update, destroy) following make:api template patterns
- OpenAPI JSON spec endpoint and ReDoc HTML documentation
- API routes protected by ApiKeyMiddleware with Throttle rate limiting
- ApiKeyProvider registered as service in bootstrap

## Task Commits

Each task was committed atomically:

1. **Task 1: Add API key infrastructure** - `7ea0faf` (feat)
2. **Task 2: Add API CRUD handlers and OpenAPI routes** - `348df85` (feat)

**Formatting fix:** `ae492e7` (style: cargo fmt)
**Framework export fix:** `0318289` (feat: re-export openapi response helpers)

## Files Created/Modified
- `app/src/api/mod.rs` - API module declaration
- `app/src/api/docs.rs` - OpenAPI JSON and ReDoc handlers
- `app/src/api/routes.rs` - API route group with ApiKeyMiddleware and Throttle
- `app/src/api/user_api.rs` - User CRUD handlers with request types
- `app/src/migrations/m20260228_create_api_keys_table.rs` - API keys table migration
- `app/src/models/api_key.rs` - API key model with query builder
- `app/src/models/entities/api_keys.rs` - SeaORM entity for api_keys
- `app/src/providers/api_key_provider.rs` - Database-backed ApiKeyProvider
- `app/src/bootstrap.rs` - Added ApiKeyProvider service binding
- `app/src/main.rs` - Added `mod api` declaration
- `app/src/routes.rs` - Replaced old /api group with api_routes() and docs_routes()
- `framework/src/lib.rs` - Re-exported openapi_docs_response and openapi_json_response

## Decisions Made
- Entity/model split: entity in entities/api_keys.rs (auto-generated pattern), model in models/api_key.rs (custom code pattern)
- Request types inline in user_api.rs to stay within plan scope (no separate requests module)
- Re-exported openapi response helpers from framework root since they were defined but not publicly accessible

## Deviations from Plan

### Auto-fixed Issues

**1. [Blocking] Re-exported missing openapi response helpers**
- **Found during:** Task 2 (docs.rs compilation)
- **Issue:** `openapi_docs_response` and `openapi_json_response` were defined in framework but not re-exported from `lib.rs`
- **Fix:** Added them to the `pub use api::openapi::` re-export block
- **Files modified:** `framework/src/lib.rs`
- **Verification:** `cargo check -p app` compiles cleanly
- **Committed in:** `0318289`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Essential for API docs to compile. No scope creep.

## Issues Encountered
- `GroupDef.into()` in routes! macro fails type inference; resolved by calling `api_routes()` directly (GroupDef implements `.register()` natively)

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Sample app now serves a valid API target for ferro-api-mcp E2E testing
- API key must be generated and inserted into api_keys table before E2E tests can authenticate
- Next plan can test ferro-api-mcp against the running sample app

---
*Phase: 82-e2e-validation*
*Completed: 2026-02-28*
