---
phase: 199-oauth-browser-login
verified: 2026-06-10T14:00:00Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 199: OAuth Browser Login — Verification Report

**Phase Goal:** A standard MCP client can discover the authorization server, dynamically register, complete a browser authorization-code + PKCE (S256) flow that reuses the application's existing session login, approve a consent screen, and exchange the code for an access token bound to `(user, tenant)` with the MCP endpoint as audience and a short expiry. The bearer-token validation on `POST /mcp` accepts valid tokens and rejects invalid or expired ones (401) and audience/tenant mismatches (403).
**Verified:** 2026-06-10T14:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `GET /.well-known/oauth-protected-resource` and `GET /.well-known/oauth-authorization-server` return spec-compliant discovery docs advertising authorization-code + PKCE (S256) | VERIFIED | `ferro-mcp-oauth/src/discovery.rs` implements both handlers; unit tests assert `resource`, `authorization_servers`, `issuer`, `authorization_endpoint`, `token_endpoint`, `registration_endpoint`, `response_types_supported=["code"]`, `grant_types_supported=["authorization_code"]`, `code_challenge_methods_supported=["S256"]`, `token_endpoint_auth_methods_supported=["none"]`; all 3 discovery tests pass |
| 2 | `POST /register` (RFC 7591 DCR) accepts a registration request and returns a `client_id` | VERIFIED | `ferro-mcp-oauth/src/register.rs` generates 16-byte URL-safe-base64 `client_id`, persists to `oauth_clients`, returns 201; rejects missing `redirect_uris` (400) and non-https/non-localhost schemes (400); 7 register tests pass |
| 3 | `GET /authorize` redirects to existing login when no session; after login presents consent; after approval redirects back with a PKCE authorization code | VERIFIED | `ferro-mcp-oauth/src/authorize.rs` calls `Auth::check()`, stores `oauth_return_to` in session, redirects to `/auth/login`; validates client + exact redirect_uri match; renders CSRF-protected HTML consent page via `consent::render_consent_html`; `POST /authorize` (consent.rs) validates CSRF via `ct_eq`, stores single-use code in `ferro::Cache` with 60s TTL, redirects with code+state; `app/src/controllers/auth_controller.rs` resumes via `oauth_return_to` on login success; 7 authorize + 5 consent tests pass |
| 4 | `POST /token` exchanges a valid code + PKCE verifier for an access token bound to `(user, tenant)`, MCP endpoint as audience, short expiry | VERIFIED | `ferro-mcp-oauth/src/token.rs` calls `Cache::forget` BEFORE any validation (single-use), verifies PKCE S256 via `pkce::verify_s256`, mints HS256 JWT via `jwt::build_claims` + `jwt::mint_token` with `aud=["{APP_URL}/mcp"]`, `exp=now+3600`; 9 token tests pass; full e2e test `full_pkce_flow` (flow_integration.rs) exercises the complete DCR→authorize→consent→token→validate chain and replay guard — passes |
| 5 | Invalid/expired bearer on `POST /mcp` → 401; audience or tenant mismatch → 403 | VERIFIED | `ferro-mcp-oauth/src/validate.rs` maps `InvalidAudience` → `BearerCheck::Forbidden` (403), all other decode errors → `BearerCheck::Invalid` (401), tenant mismatch post-decode → `BearerCheck::Forbidden` (403); `app/src/controllers/mcp.rs` maps `BearerCheck::Invalid` → 401 with `WWW-Authenticate: Bearer error="invalid_token"`, `BearerCheck::Forbidden` → 403 bare; 7 validate tests + 4 app mcp tests all pass |

**Score:** 5/5 truths verified

### Specific Load-Bearing Properties

| Property | Check | Result |
|----------|-------|--------|
| JWT tenant claim key is exactly `tenant_id` | `grep tenant_id ferro-mcp-oauth/src/jwt.rs` (field name in McpTokenClaims struct + test that asserts JSON key) | VERIFIED — `pub tenant_id: Option<i64>` in `McpTokenClaims`; test `tenant_claim_key_is_exactly_tenant_id` serializes and asserts `json.get("tenant_id")` is present and equals 7; matches `JwtClaimResolver` at `framework/src/tenant/resolver.rs` |
| JWT decode pins `validation.algorithms = vec![Algorithm::HS256]` | `grep "validation.algorithms" ferro-mcp-oauth/src/jwt.rs` | VERIFIED — line 89: `validation.algorithms = vec![Algorithm::HS256];` |
| `aud` mismatch → 403, expired/bad-signature → 401 (NOT swapped) | `ferro-mcp-oauth/src/validate.rs` `BearerCheck` mapping | VERIFIED — `InvalidAudience` arm returns `BearerCheck::Forbidden`; all other `Err` kinds return `BearerCheck::Invalid`; mcp.rs maps `Forbidden → 403`, `Invalid → 401` |
| Authorization code is single-use: `Cache::forget` precedes validation | `grep -n "Cache::forget" ferro-mcp-oauth/src/token.rs` | VERIFIED — line 64 `Cache::forget` called immediately after `Cache::get` (line 62), before any client_id/redirect_uri/PKCE validation |
| `redirect_uri` exact-match validation exists (open-redirect closed) | `authorize.rs` + `token.rs` | VERIFIED — `authorize.rs` line 127: `stored_uris.iter().any(|u| u == &redirect_uri)`; `token.rs` line 78: `record.redirect_uri != form.redirect_uri` |
| `ferro-mcp-server` has NO dependency on `ferro-mcp-oauth` | `cargo tree -p ferro-mcp-server | grep ferro-mcp-oauth` count == 0 | VERIFIED — returns 0 |
| `extract_bearer` deleted from `ferro-mcp-server/src/auth.rs`, `BearerOutcome` kept | `grep extract_bearer ferro-mcp-server/src/auth.rs` | VERIFIED — no match; `BearerOutcome` enum present |
| Full e2e PKCE integration test exists, is NOT `#[ignore]`'d, asserts replay guard | `ferro-mcp-oauth/tests/flow_integration.rs` fn `full_pkce_flow` | VERIFIED — no `#[ignore]` attribute; test drives 8-step flow including replay guard at Step 8; `cargo test -p ferro-mcp-oauth --test flow_integration` exits 0 |
| `OAuthConfig` fails closed when `MCP_TOKEN_SECRET` is unset or short | `ferro-mcp-oauth/src/config.rs` `from_env()` | VERIFIED — returns `Err(MissingSecret)` when unset, `Err(SecretTooShort)` when `< 32 bytes`; 5 config tests pass |

### Required Artifacts

| Artifact | Status | Details |
|----------|--------|---------|
| `ferro-mcp-oauth/Cargo.toml` | VERIFIED | Crate manifest with `ferro` (package="ferro-rs") dep; no `ferro-mcp-server` dep |
| `ferro-mcp-oauth/src/lib.rs` | VERIFIED | Declares all 12 modules; re-exports `OAuthConfig`, `OAuthConfigError`, `OAuthError`, `McpTokenClaims`, `CreateOauthClientsTable`, `validate_bearer`, `BearerCheck`; `handlers` pub module for route imports |
| `ferro-mcp-oauth/src/config.rs` | VERIFIED | `OAuthConfig::from_env()` fail-closed; `sanitized_app_url()` secret-free; `sanitize_identity` strips CRLF |
| `ferro-mcp-oauth/src/discovery.rs` | VERIFIED | Both RFC 9728 + RFC 8414 handlers; reads only `APP_URL` via `sanitized_app_url()`; all required JSON fields present |
| `ferro-mcp-oauth/src/register.rs` | VERIFIED | RFC 7591 DCR; 16-byte random `client_id`; scheme allowlist; no `client_secret` |
| `ferro-mcp-oauth/src/pkce.rs` | VERIFIED | `verify_s256` constant-time via `subtle::ConstantTimeEq`; `generate_auth_code` 256-bit URL-safe |
| `ferro-mcp-oauth/src/jwt.rs` | VERIFIED | `McpTokenClaims` with `tenant_id`; `mint_token` HS256; `decode_token` with pinned algorithm + `set_audience` + `leeway=0` |
| `ferro-mcp-oauth/src/validate.rs` | VERIFIED | `BearerCheck` enum; `validate_bearer` with D-07 validation order (sig+exp→401, aud→403, tenant→403) |
| `ferro-mcp-oauth/src/authorize.rs` | VERIFIED | `authorize_get` with PKCE S256 guard, auth redirect, exact redirect_uri match, tenant capture, consent render |
| `ferro-mcp-oauth/src/consent.rs` | VERIFIED | `render_consent_html` with CSRF field; `authorize_post` with `ct_eq` CSRF check, single-use code mint; `CONSENT_CONTENT_TYPE` constant |
| `ferro-mcp-oauth/src/token.rs` | VERIFIED | `token_exchange` with forget-before-validate, PKCE verify, JWT mint, RFC 6749 §5.1 response |
| `ferro-mcp-oauth/src/store.rs` | VERIFIED | `OAuthClient` SeaORM entity; `OAuthCode` cache struct; `insert_client` + `find_by_client_id` helpers |
| `ferro-mcp-oauth/src/migration.rs` | VERIFIED | `Migration` (re-exported as `CreateOauthClientsTable`); crate-shipped migration helper |
| `ferro-mcp-oauth/tests/flow_integration.rs` | VERIFIED | Full 8-step e2e PKCE flow; no `#[ignore]`; replay guard asserted; passes |
| `app/src/migrations/m20260611_create_oauth_clients_table.rs` | VERIFIED | `OauthClients` enum; `idx_oauth_clients_client_id` unique index |
| `app/src/migrations/mod.rs` | VERIFIED | `m20260611_create_oauth_clients_table` registered |
| `app/src/routes.rs` | VERIFIED | All six OAuth routes mounted: `/.well-known/oauth-protected-resource`, `/.well-known/oauth-authorization-server`, `/register`, `/authorize` (GET+POST), `/token` |
| `app/src/controllers/mcp.rs` | VERIFIED | `validate_bearer` called; `BearerCheck` → 401/403/proceed mapping; Origin guard; no `extract_bearer` import |
| `app/src/controllers/auth_controller.rs` | VERIFIED | `oauth_return_to` session read + `s.forget` + 302 redirect on login success |
| `ferro-mcp-server/src/auth.rs` | VERIFIED | `BearerOutcome` enum only; `extract_bearer` deleted |
| `.github/workflows/publish.yml` | VERIFIED | Line 274: `WAVE2_CRATES="ferro-rs ferro-mcp ferro-mcp-server ferro-mcp-oauth"` |
| `Cargo.toml` (workspace) | VERIFIED | `ferro-mcp-oauth` in members array |

### Key Link Verification

| From | To | Via | Status |
|------|----|-----|--------|
| `app/src/controllers/mcp.rs` | `ferro_mcp_oauth::validate_bearer` | Direct import + call | WIRED |
| `app/src/routes.rs` | `ferro_mcp_oauth::handlers::*` | six `get!/post!` mounts | WIRED |
| `app/src/controllers/auth_controller.rs` | `session.oauth_return_to` | `session().and_then(s.get)` + `session_mut(s.forget)` | WIRED |
| `token.rs` | `ferro::Cache` forget+get | single-use code redemption; forget before validation | WIRED |
| `consent.rs` | `ferro::Cache::put` | OAuthCode 60s TTL | WIRED |
| `token.rs` | `jwt::mint_token` | code → access token | WIRED |
| `jwt.rs decode_token` | `Validation.algorithms` | `validation.algorithms = vec![Algorithm::HS256]` | WIRED |
| `validate.rs` | `jwt::decode_token` | aud + tenant_id checks | WIRED |
| `ferro-mcp-server/src/auth.rs` | (no ferro-mcp-oauth dep) | dependency direction preserved | VERIFIED |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `app/src/controllers/mcp.rs` | `validate_bearer(authorization...)` | `ferro_mcp_oauth::validate_bearer` → `decode_token` → real JWT decode | Yes — decodes live JWT against real secret | FLOWING |
| `ferro-mcp-oauth/src/token.rs` | `access_token` | `build_claims` + `mint_token` with real `OAuthConfig.token_secret` | Yes — HS256 signed with env-sourced secret | FLOWING |
| `ferro-mcp-oauth/src/discovery.rs` | JSON response | `sanitized_app_url()` reads `APP_URL` env | Yes — derives from real env config | FLOWING |
| `ferro-mcp-oauth/src/register.rs` | `client_id` | `rand::thread_rng().fill_bytes()` | Yes — 128-bit random, non-sequential | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All ferro-mcp-oauth unit + integration tests | `cargo test -p ferro-mcp-oauth -- --test-threads=1` | 50 unit tests + 1 integration test pass; 0 failed | PASS |
| app mcp controller tests | `cargo test -p app mcp` | 4 tests pass; 0 failed | PASS |
| ferro-mcp-server has no ferro-mcp-oauth dependency | `cargo tree -p ferro-mcp-server | grep -c ferro-mcp-oauth` | 0 | PASS |
| extract_bearer fully removed | `grep extract_bearer ferro-mcp-server/src/auth.rs ferro-mcp-server/src/lib.rs app/src/controllers/mcp.rs` | no matches | PASS |
| Algorithm pin present in jwt.rs | `grep "validation.algorithms = vec!\[Algorithm::HS256\]" ferro-mcp-oauth/src/jwt.rs` | line 89 found | PASS |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AMCP-07 | Plans 01, 02 | OAuth discovery metadata + DCR endpoint advertising authorization-code + PKCE S256 | SATISFIED | Both `.well-known` handlers return spec-exact JSON; DCR persists clients with random `client_id`; scheme allowlist enforced |
| AMCP-08 | Plans 01, 02, 03, 04 | Browser authorization-code + PKCE flow reusing existing login + consent; token bound to `(user, tenant)` | SATISFIED | Full e2e test proves DCR→authorize→consent→token chain; `auth_controller.rs` closes browser loop via `oauth_return_to`; token carries `sub` + `tenant_id` + `aud` |
| AMCP-09 | Plans 03, 05 | MCP endpoint validates bearer; invalid/expired → 401; aud/tenant mismatch → 403 | SATISFIED | `validate_bearer` implements D-07 order; `mcp.rs` maps `Invalid→401`, `Forbidden→403`; 4 app mcp tests confirm rejection paths |

### Anti-Patterns Found

No blockers or stubs. All module files are fully implemented. The only intentional deferred item is the `BearerCheck::Authenticated(_principal)` arm in `mcp.rs` discarding the principal — documented in the summary as intentional (Phase 200 inserts it into request extensions for `JwtClaimResolver`). The dispatch path proceeds correctly.

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `app/src/controllers/mcp.rs` | 63 | `BearerCheck::Authenticated(_principal)` — principal discarded | Info | Intentional; Phase 200 will wire it to `JwtClaimResolver`; dispatch path is correct |

### Human Verification Required

One item requires live-client testing, noted in the phase VALIDATION.md and explicitly deferred to Phase 200's GO/NO-GO:

**A real MCP client (Claude Desktop / MCP SDK) completes browser login against a live app**

- **Test:** Configure a running ferro app with `MCP_TOKEN_SECRET` set, point an MCP client at the app's `/mcp` endpoint, complete the browser authorization flow.
- **Expected:** Client discovers authorization server, registers, gets authorization code after login + consent, exchanges for access token, successfully calls `tools/list`.
- **Why human:** Requires a live browser, running server, and external MCP client. All automated success criteria (SC-1..SC-5) have been proven in-process. This is Phase 200's acceptance gate per VALIDATION.md.

This item does not affect automated gate status. Phase 199's SC-1..SC-5 are fully verified in-process.

### Gaps Summary

No gaps. All five success criteria are verified. All load-bearing security properties are confirmed in the actual source files. The test suite (50 unit + 1 integration for `ferro-mcp-oauth`, 4 for `app mcp`) passes. The no-cycle dependency constraint between `ferro-mcp-server` and `ferro-mcp-oauth` holds. Requirements AMCP-07, AMCP-08, and AMCP-09 are all satisfied.

---

_Verified: 2026-06-10T14:00:00Z_
_Verifier: Claude (gsd-verifier)_
