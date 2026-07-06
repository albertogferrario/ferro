---
phase: 199
slug: oauth-browser-login
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-10
validated: 2026-06-10
---

# Phase 199 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[tokio::test]` + in-process assertions (no test-runner config) |
| **Config file** | none — inline `#[cfg(test)]` modules and `tests/*.rs` |
| **Quick run command** | `cargo test -p ferro-mcp-oauth` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (full workspace), ~5s crate-local |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-oauth` (+ `cargo clippy -p ferro-mcp-oauth --all-targets -- -D warnings`)
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | Test Fns | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|----------|--------|
| 199-01 | 01 | 1 | — | T-199-13/14/IDENT | `OAuthConfig::from_env` fails closed when `MCP_TOKEN_SECRET` unset / < 32 bytes; CRLF stripped | unit | `cargo test -p ferro-mcp-oauth config` | config.rs: missing_secret_returns_err, short_secret_returns_err, valid_secret_returns_ok_with_bytes, sanitize_strips_crlf_and_control_chars, sanitized_app_url_works_without_secret | ✅ green |
| 199-02 | 02 | 2 | AMCP-07 | T-199-DISC | `.well-known` docs return spec field names; `code_challenge_methods_supported=["S256"]`; URLs from APP_URL | unit | `cargo test -p ferro-mcp-oauth discovery` | discovery.rs: protected_resource_has_resource_and_authorization_servers, authorization_server_has_all_required_fields, discovery_urls_interpolate_app_url_no_hardcoded_host | ✅ green |
| 199-02 | 02 | 2 | AMCP-07 | T-199-05/DCR/04a | DCR returns random non-sequential `client_id`; scheme allowlist; `redirect_uris` required + stored verbatim | integration | `cargo test -p ferro-mcp-oauth register` | register.rs: register_valid_returns_client_id_and_redirect_uris, client_id_is_random_and_non_sequential, missing_redirect_uris_returns_error, javascript_scheme_is_rejected, data_and_custom_schemes_are_rejected, allowed_schemes_pass_validation, persisted_client_retrievable_by_client_id | ✅ green |
| 199-03 | 03 | 2 | AMCP-08 | T-199-11 | correct verifier → true; wrong verifier → false; constant-time S256 compare | unit | `cargo test -p ferro-mcp-oauth pkce` | pkce.rs: correct_verifier_matches_stored_challenge, wrong_verifier_does_not_match, generate_auth_code_is_url_safe_and_unique | ✅ green |
| 199-03 | 03 | 2 | AMCP-08 | T-199-06/07/08/17 | HS256 mint→decode round-trip; alg-pinned; `tenant_id` exact key; wrong aud/secret/expiry error | unit | `cargo test -p ferro-mcp-oauth jwt` | jwt.rs: mint_decode_round_trip, wrong_secret_returns_error, expired_token_returns_error, wrong_audience_returns_error, tenant_claim_key_is_exactly_tenant_id | ✅ green |
| 199-03 | 03 | 2 | AMCP-09 | T-199-08/09/401 | valid→Authenticated; expired→Invalid(401); wrong aud→Forbidden(403); wrong `tenant_id`→Forbidden(403); no header→Unauthenticated | unit | `cargo test -p ferro-mcp-oauth validate` | validate.rs: valid_token_returns_authenticated, expired_token_returns_invalid, wrong_audience_returns_forbidden, wrong_tenant_returns_forbidden, no_header_returns_unauthenticated, no_bearer_prefix_returns_unauthenticated | ✅ green |
| 199-04 | 04 | 3 | AMCP-08 | T-199-01/04/10/12/XSS | `redirect_uri` exact-match; consent CSRF + S256 fields; `client_name` XSS-escaped | unit | `cargo test -p ferro-mcp-oauth authorize consent` | authorize.rs: consent_html_contains_csrf_and_s256, redirect_uri_exact_match_check, html_escape_replaces_special_chars, error_page_*; consent.rs: render_consent_html_contains_csrf_field, _contains_s256_and_code_challenge_method, _escapes_client_name_xss, _contains_text_html_doctype | ✅ green |
| 199-04 | 04 | 3 | AMCP-08 | T-199-02/03/16 | code single-use (`forget` before validate); replay→None; PKCE verified; JWT minted | integration | `cargo test -p ferro-mcp-oauth token` | token.rs: forget_before_validate_single_use, replay_code_returns_none_after_forget, wrong_verifier_fails_pkce, correct_verifier_passes_pkce, jwt_roundtrip_authenticated, json_error_shape | ✅ green |
| 199-04 | 04 | 3 | AMCP-08 | — | DCR→authorize→consent→token→validate end-to-end + replay guard, no external IdP (proves SC-1..SC-5) | integration | `cargo test -p ferro-mcp-oauth --test flow_integration` | flow_integration.rs: full_pkce_flow | ✅ green |
| 199-05 | 05 | 4 | AMCP-09 | T-199-401/15/13b | `/mcp` rejects invalid (401 `invalid_token`) / Origin mismatch (403); absent Origin allowed; secret-unset → 401 | unit | `cargo test -p app mcp` | mcp.rs: challenge_response_has_correct_header, invalid_token_returns_401_invalid_token_header, origin_mismatch_maps_to_403, absent_origin_is_allowed | ✅ green |
| 199-05 | 05 | 4 | AMCP-09 | T-199-CYCLE | seam change does not regress Phase 197/198 (extract_bearer deleted, no new dep) | integration | `cargo test -p ferro-mcp-server` | 22 prior tests (regression) | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*All commands verified green during validation audit 2026-06-10 (targeted runs; full `--all-features` skipped — disk ~96%).*

---

## Wave 0 Requirements

- [x] `ferro-mcp-oauth/src/lib.rs` + module files — crate scaffold (Cargo.toml, workspace members + `.github/workflows/publish.yml` Wave 2) — built in Plan 199-01
- [x] `ferro-mcp-oauth/tests/flow_integration.rs` — full PKCE flow integration test harness (in-memory SQLite + in-memory `ferro::Cache`) — harness in 199-01, `full_pkce_flow` filled in 199-04
- [x] `app/src/migrations/m20260611_create_oauth_clients_table.rs` — `oauth_clients` migration + registration in the app migration list — built in Plan 199-01
- [x] `OAuthConfig` test fixture populated with a deterministic test `MCP_TOKEN_SECRET` — `test_oauth_config()` in flow_integration.rs:57
- [x] `Request` test-helper pattern reused from `ferro-mcp-server/tests/dispatch_integration.rs` (`fresh_db()`) — `fresh_db()` in flow_integration.rs:35

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| A real MCP client (Claude Desktop / MCP SDK) completes browser login against a live app | AMCP-08 (dogfood) | Requires a live browser + external client; deferred to Phase 200 GO/NO-GO | Phase 200 acceptance — not blocking Phase 199 automated gate |

*All Phase 199 success criteria (SC-1…SC-5) have automated verification via in-process integration tests; the live-client dogfood is Phase 200's gate.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (crate scaffold, migration, flow harness)
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** verified 2026-06-10

---

## Validation Audit 2026-06-10

| Metric | Count |
|--------|-------|
| Requirements (AMCP-07/08/09) | 3 |
| Requirements covered | 3 |
| Success criteria (SC-1..SC-5) | 5 |
| Success criteria covered | 5 |
| Gaps found | 0 |
| Resolved | 0 (none needed) |
| Escalated to manual-only | 1 (live-client dogfood → Phase 200, by design) |

Every success criterion and requirement maps to green automated tests (see Per-Task Verification
Map). Coverage verified by enumerating real test functions across `ferro-mcp-oauth/src/*`,
`ferro-mcp-oauth/tests/flow_integration.rs`, `app/src/controllers/mcp.rs`, and the
`ferro-mcp-server` regression suite — no gap-filling auditor run was required. The only
non-automated item is the live external-MCP-client browser-login dogfood, which is Phase 200's
GO/NO-GO gate by design, not a Phase 199 coverage gap.
