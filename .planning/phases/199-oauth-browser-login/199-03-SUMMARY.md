---
phase: 199-oauth-browser-login
plan: "03"
subsystem: ferro-mcp-oauth
tags: [oauth, mcp, jwt, pkce, security, constant-time, hs256, bearer-validation]
dependency_graph:
  requires:
    - 199-01 (OAuthConfig, OAuthError, crate scaffold)
  provides:
    - pkce.rs: verify_s256 (constant-time S256) + generate_auth_code
    - jwt.rs: McpTokenClaims + mint_token + decode_token (alg-pinned HS256)
    - validate.rs: BearerCheck enum + validate_bearer (401/403/Authenticated)
  affects:
    - ferro-mcp-oauth/src/lib.rs (added McpTokenClaims + BearerCheck exports)
tech_stack:
  added: []
  patterns:
    - subtle::ConstantTimeEq for PKCE S256 challenge comparison (T-11)
    - jsonwebtoken v9 Validation with algorithm pin + set_audience (T-06, T-07, T-08)
    - Local BearerCheck enum decouples validate.rs from ferro-mcp-server (no new dep)
    - D-07 validation order: sig+exp→401, InvalidAudience→403, tenant mismatch→403
key_files:
  created: []
  modified:
    - ferro-mcp-oauth/src/pkce.rs
    - ferro-mcp-oauth/src/jwt.rs
    - ferro-mcp-oauth/src/validate.rs
    - ferro-mcp-oauth/src/lib.rs
decisions:
  - "validate_bearer returns BearerCheck (crate-local enum) not BearerOutcome — avoids ferro-mcp-server dep; Plan 05 maps at app seam"
  - "validate_bearer signature extended to expected_tenant: Option<i64> for post-decode tenant check"
  - "McpTokenClaims.tenant_id uses #[serde(skip_serializing_if=Option::is_none)] — single-tenant tokens omit the claim cleanly"
  - "BearerCheck and McpTokenClaims exported from lib.rs so Plan 05 can import without re-specifying module paths"
metrics:
  duration: "311s"
  completed_date: "2026-06-10"
  tasks_completed: 3
  files_created: 0
  files_modified: 4
---

# Phase 199 Plan 03: Crypto Core (PKCE + JWT + Bearer Validation) Summary

HS256 JWT mint/decode with pinned algorithm and audience binding, constant-time PKCE S256 verification, and `validate_bearer` mapping decode outcomes to `Authenticated` / `Invalid` (401) / `Forbidden` (403) / `Unauthenticated` — the security-critical crypto core for SC-4 and SC-5.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | PKCE S256 verification + auth-code generation (constant-time) | 0c2f4358 | ferro-mcp-oauth/src/pkce.rs |
| 2 | HS256 JWT mint + decode with pinned algorithm and audience | 392dd396 | ferro-mcp-oauth/src/jwt.rs |
| 3 | validate_bearer — 401/403/Authenticated mapping with tenant check | a7708bc3 | ferro-mcp-oauth/src/validate.rs, src/lib.rs |

## Verification Results

- `cargo test -p ferro-mcp-oauth pkce` exits 0 (3 tests)
- `cargo test -p ferro-mcp-oauth jwt` exits 0 (5 tests)
- `cargo test -p ferro-mcp-oauth validate` exits 0 (7 tests)
- `cargo clippy -p ferro-mcp-oauth --all-targets -- -D warnings` clean
- `grep -q 'ConstantTimeEq'` pkce.rs: FOUND
- `grep -q 'URL_SAFE_NO_PAD'` pkce.rs: FOUND
- `grep -q 'ct_eq'` pkce.rs: FOUND
- `grep -q 'validation.algorithms = vec![Algorithm::HS256]'` jwt.rs: FOUND
- `grep -q 'set_audience'` jwt.rs: FOUND
- `grep -q 'tenant_id'` jwt.rs, validate.rs: FOUND
- No `use ferro_mcp_server` in validate.rs: CONFIRMED

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Redundant guard pattern rejected by clippy -D warnings**
- **Found during:** Task 3, clippy run
- **Issue:** `Some(t) if t.is_empty()` is a redundant guard — clippy required `Some("")` pattern instead.
- **Fix:** Changed to `None | Some("") => return BearerCheck::Unauthenticated` (cleaner and idiomatic).
- **Files modified:** `ferro-mcp-oauth/src/validate.rs`
- **Commit:** a7708bc3

**2. [Rule 2 - Missing export] BearerCheck and McpTokenClaims not exported from lib.rs**
- **Found during:** Task 3
- **Issue:** Plan 05 needs to import `BearerCheck` and `McpTokenClaims` at the app seam. The stub lib.rs only exported `validate_bearer` — the return type was not re-exported.
- **Fix:** Added `pub use validate::{validate_bearer, BearerCheck}` and `pub use jwt::McpTokenClaims` to lib.rs.
- **Files modified:** `ferro-mcp-oauth/src/lib.rs`
- **Commit:** a7708bc3

## Security Properties Verified

| Threat | Mitigation | Location | Test |
|--------|------------|----------|------|
| T-06 alg confusion (alg=none) | `validation.algorithms = vec![Algorithm::HS256]` | jwt.rs decode_token | wrong_secret_returns_error |
| T-07 RS256→HS256 confusion | Same algorithm pin; no RS256 key exists | jwt.rs decode_token | — (structural) |
| T-08 audience confusion | `set_audience(&[expected_aud])` + InvalidAudience→Forbidden(403) | jwt.rs + validate.rs | wrong_audience_returns_forbidden |
| T-09 tenant confusion | Post-decode compare claims.tenant_id vs expected_tenant | validate.rs | wrong_tenant_returns_forbidden + absent_tenant_when_expected |
| T-11 PKCE timing oracle | `subtle::ConstantTimeEq` for S256 comparison | pkce.rs verify_s256 | correct_verifier + wrong_verifier |
| T-17 mix-up attack | iss and aud from same OAuthConfig | jwt.rs build_claims | mint_decode_round_trip |

## Known Stubs

None — all three files are fully implemented with tests.

The following module stubs from Plan 01 remain (filled by later plans):
- `ferro-mcp-oauth/src/discovery.rs` — Plan 02 (already done)
- `ferro-mcp-oauth/src/register.rs` — Plan 02 (already done)
- `ferro-mcp-oauth/src/authorize.rs` — Plan 04
- `ferro-mcp-oauth/src/consent.rs` — Plan 04
- `ferro-mcp-oauth/src/token.rs` — Plan 04
- `ferro-mcp-oauth/src/store.rs` — Plan 04

## Threat Surface Scan

No new network endpoints introduced. The three files are pure crypto/validation functions with no network boundary exposure. `validate_bearer` is called at the app seam by Plan 05. No threat flags.

## Self-Check: PASSED

- `ferro-mcp-oauth/src/pkce.rs` exists and contains `ConstantTimeEq`, `URL_SAFE_NO_PAD`, `ct_eq`
- `ferro-mcp-oauth/src/jwt.rs` exists and contains `Algorithm::HS256`, `validation.algorithms`, `set_audience`, `tenant_id`
- `ferro-mcp-oauth/src/validate.rs` exists and contains `BearerCheck`, `Forbidden`, `Authenticated`, `InvalidAudience`, `tenant_id`
- Commits 0c2f4358, 392dd396, a7708bc3 verified in git log
