# Technical Concerns

**Analysis Date:** 2026-02-13

## Error Handling Risks

**Schedule Expression Unwraps:**

| File | Count | Nature |
|------|-------|--------|
| `framework/src/schedule/expression.rs` | 18 | Builder convenience methods passing known-valid cron strings |

15 of the 18 `.unwrap()` calls are in builder methods (`every_minute()`, `hourly()`, `daily()`, etc.) that pass hardcoded, known-valid cron format strings to `Self::parse()`. 1 is in a doc comment example, 2 are in tests. The risk of runtime panic is minimal since the inputs are compile-time constants.

**Priority:** P3 (Low effort, Low impact)

## Incomplete Features

**Active TODOs:**

| Location | Description | Priority |
|----------|-------------|----------|
| `app/src/middleware/share_inertia.rs:33` | Add authenticated user sharing to Inertia props | Low |
| `app/src/middleware/share_inertia.rs:42` | Add flash messages to Inertia props | Low |
| `framework/src/testing/http.rs:211` | Route TestClient requests through actual handlers | Low |

The sample app TODOs are non-blocking — the auth system works independently via `AuthUser<T>` extractor. The testing TODO documents a known limitation of `TestClient` when not using `with_router()`.

Template TODOs in `ferro-cli/src/templates/make.rs` and `scaffold.rs` are intentional user-facing placeholders in generated code, not incomplete features.

**Priority:** P3 (Low effort, Low impact — sample app and test infrastructure only)

## COMPONENT_CATALOG Duplication

The `COMPONENT_CATALOG` constant is duplicated in two locations:

| Location | Lines |
|----------|-------|
| `ferro-cli/src/ai.rs:14` | Used by `ferro make:json-view` AI generation |
| `ferro-mcp/src/tools/json_ui_generate.rs:67` | Used by MCP `json_ui_generate` tool |

**Drift detected:**
- CLI version: Text element has `div|section` in element options; Input lacks `step` parameter
- MCP version: Text element missing `div|section`; Input has `step (Option<String>)`

Cannot share code directly between workspace binary crates. Options: extract to shared crate, generate from source of truth at build time, or consolidate in `ferro-json-ui`.

**Priority:** P2 (Medium effort, Medium impact — drift causes inconsistent AI generation)

## Performance Considerations

**Database:**
- Connection pool size configurable via `DB_MAX_CONNECTIONS` (documented in `.env.example` template since Phase 54)
- Redis connection pooling via `deadpool-redis`

**Infrastructure (v4.0+):**
- Rate limiting backed by cache store with fail-open semantics
- WebSocket broadcasting with channel authorization and heartbeat
- Cache warming strategies not implemented (future recommendation)

## Testing Gaps

**Current State:**
- `TestClient::with_router()` enables router integration testing
- Factory patterns in `framework/src/testing/factory.rs` (1,281 lines)
- HTTP test helpers in `framework/src/testing/http.rs` (736 lines)

**Remaining Gaps:**
- No E2E test suite
- No load/stress testing framework
- Integration tests limited to utility functions

Acceptable for current project stage (pre-publication).

## Monitoring Recommendations

**Future additions:**
- Structured error logging with context
- Request tracing across service boundaries
- Database query performance metrics
- Cache hit/miss ratios

**Current State:**
- Basic tracing infrastructure exists
- Debug endpoints at `/_ferro/metrics`
- No external monitoring integration

## Priority Matrix

| Item | Effort | Impact | Priority |
|------|--------|--------|----------|
| COMPONENT_CATALOG consolidation | Medium | Medium | P2 |
| Schedule expression unwraps | Low | Low | P3 |
| Share inertia TODOs | Low | Low | P3 |
| TestClient handler routing TODO | Low | Low | P3 |

<details>
<summary>Resolved Concerns (click to expand)</summary>

### Error Handling — app/main.rs and bootstrap.rs

**Resolved:** No `.expect()` calls remain in `app/src/main.rs` or `app/src/bootstrap.rs`. Both files use proper `Result` propagation.

### Missing Configuration (.env.example)

**Resolved in Phase 54:** `ferro-cli/src/templates/files/root/env.example.tpl` provides all framework environment variables with documentation. Generated into new projects via `ferro new`.

### Code Quality — Large Template File

**Resolved in Phase 55:** `ferro-cli/src/templates/mod.rs` reduced from 2,713 to 831 lines, split into 7 focused submodules (`project.rs`, `make.rs`, `entity.rs`, `docker.rs`, `ai_boost.rs`, `scaffold.rs`, `auth.rs`).

### Code Quality — Testing Files

**Removed from concerns:** `framework/src/testing/factory.rs` (1,281 lines) and `framework/src/testing/http.rs` (736 lines) are acceptable sizes for test infrastructure modules.

### Security — Session Rotation

**Resolved in v4.0:** `regenerate_session_id()` in `framework/src/session/middleware.rs` is called on login via `framework/src/auth/guard.rs`. CSRF token regenerated on login/logout. `logout_and_invalidate()` provides complete session destruction.

### Input Validation

**Status:** Validation framework is comprehensive and actively used across all user input paths. Not an actionable concern.

### Dependency Security

**Status:** Ongoing practice. `cargo audit` recommended as part of CI. Not an actionable concern.

</details>

---

*Concern analysis: 2026-02-13*
*Review quarterly and update priorities*
