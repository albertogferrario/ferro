---
phase: 199-oauth-browser-login
plan: "02"
subsystem: ferro-mcp-oauth
tags: [oauth, discovery, dcr, rfc8414, rfc9728, rfc7591, pkce, mcp]
dependency_graph:
  requires:
    - ferro-mcp-oauth crate scaffold (Plan 01)
    - OAuthConfig.sanitized_app_url (Plan 01, config.rs)
    - oauth_clients migration (Plan 01, migration.rs)
  provides:
    - protected_resource_metadata handler (RFC 9728 §2)
    - authorization_server_metadata handler (RFC 8414 §2)
    - register_client DCR handler (RFC 7591)
    - OAuthClient SeaORM entity + insert_client/find_by_client_id helpers
    - OAuthCode struct (cache serialization shape for Plan 04)
  affects:
    - ferro-mcp-oauth/src/discovery.rs (filled from stub)
    - ferro-mcp-oauth/src/register.rs (filled from stub)
    - ferro-mcp-oauth/src/store.rs (filled from stub)
    - ferro-mcp-oauth/src/config.rs (ENV_LOCK fix + dead_code removal)
    - ferro-mcp-oauth/Cargo.toml (chrono dep, ferro alias)
tech_stack:
  added:
    - chrono = "0.4" with serde feature (DateTimeUtc in SeaORM entity)
    - ferro dependency aliased as 'ferro' (package = "ferro-rs") to satisfy #[handler] macro's ::ferro path emission
  patterns:
    - Pure helper functions (protected_resource_metadata, authorization_server_metadata) for testable JSON construction — handlers are thin wrappers
    - SeaORM entity with exec_with_returning for insert + returning
    - URL-safe-base64 random client_id (16 bytes via rand::RngCore::fill_bytes) — no sequential integer (T-199-DCR)
    - Scheme allowlist via starts_with match (http://localhost, https://)
    - ENV_LOCK static Mutex in config tests to serialize env-var-mutating tests
key_files:
  created: []
  modified:
    - ferro-mcp-oauth/src/discovery.rs
    - ferro-mcp-oauth/src/register.rs
    - ferro-mcp-oauth/src/store.rs
    - ferro-mcp-oauth/src/config.rs
    - ferro-mcp-oauth/Cargo.toml
decisions:
  - "ferro dependency aliased as 'ferro' (not ferro_rs) so the #[handler] macro's ::ferro path resolves correctly in the crate"
  - "Discovery handlers read only APP_URL via sanitized_app_url() — never MCP_TOKEN_SECRET — so they serve pre-auth requests even when the secret is unset (T-199-DISC)"
  - "client_id is 16-byte URL-safe-base64 (rand fill_bytes) not UUID: no extra uuid crate needed, equivalent entropy (128-bit), satisfies T-199-DCR non-sequential requirement"
  - "ENV_LOCK Mutex added to config.rs tests to fix pre-existing env-var race between parallel test threads (Rule 1 auto-fix)"
metrics:
  duration: "~420s"
  completed_date: "2026-06-10"
  tasks_completed: 2
  files_created: 0
  files_modified: 5
---

# Phase 199 Plan 02: Discovery Metadata + Dynamic Client Registration Summary

RFC 8414 / RFC 9728 discovery handlers and RFC 7591 DCR endpoint — MCP clients can now discover the OAuth server, dynamically register, and receive a `client_id` for the PKCE browser flow.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Discovery metadata handlers (RFC 8414 + RFC 9728) | d5d6a30a | ferro-mcp-oauth/src/discovery.rs, Cargo.toml, Cargo.lock |
| 2 | Dynamic Client Registration + OAuthClient model + env-var race fix | 80aebced | ferro-mcp-oauth/src/register.rs, store.rs, config.rs, Cargo.toml |

## Verification Results

- `cargo test -p ferro-mcp-oauth discovery` exits 0 (3 tests)
- `cargo test -p ferro-mcp-oauth register` exits 0 (7 tests)
- `cargo test -p ferro-mcp-oauth` exits 0 (18 unit + 1 integration, parallel execution)
- `cargo clippy -p ferro-mcp-oauth --all-targets -- -D warnings` clean
- Discovery JSON field names match RFC 8414/9728 exactly (`resource`, `authorization_servers`, `code_challenge_methods_supported`, `token_endpoint_auth_methods_supported`)
- DCR returns `client_id`, rejects missing `redirect_uris` (400), rejects `javascript:`/`data:`/`http://evil.com` schemes (400)
- `client_id` verified random and non-sequential (base64 URL-safe, no `+`/`/`/`=` chars)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ferro dependency must be aliased as 'ferro' for #[handler] macro**
- **Found during:** Task 1 compile
- **Issue:** `ferro_macros::handler` emits `::ferro` in generated code. The `ferro-mcp-oauth` Cargo.toml used the default `ferro-rs` package name (resolves as `ferro_rs` in Rust), causing `could not find 'ferro' in the list of imported crates`.
- **Fix:** Changed `ferro-rs = { path = "../framework" }` to `ferro = { path = "../framework", package = "ferro-rs" }` in Cargo.toml.
- **Files modified:** `ferro-mcp-oauth/Cargo.toml`
- **Commit:** d5d6a30a

**2. [Rule 3 - Blocking] chrono not directly available in ferro-mcp-oauth**
- **Found during:** Task 2 compile (store.rs `DateTimeUtc`)
- **Issue:** `chrono` is a framework dep (not re-exported into the crate's namespace). `DateTimeUtc` requires chrono directly.
- **Fix:** Added `chrono = { version = "0.4", features = ["serde"] }` to Cargo.toml.
- **Files modified:** `ferro-mcp-oauth/Cargo.toml`
- **Commit:** 80aebced

**3. [Rule 1 - Bug] Pre-existing env-var race in config tests**
- **Found during:** Task 2 full test run (parallel threads, `short_secret_returns_err` flaked)
- **Issue:** Config tests mutate `MCP_TOKEN_SECRET` / `APP_URL` without synchronization. With `-j` parallelism, `set_var` in one test interferes with `remove_var` in another.
- **Fix:** Added `static ENV_LOCK: Mutex<()>` in `config::tests`; all env-mutating tests acquire `_g = ENV_LOCK.lock()` before touching env vars.
- **Files modified:** `ferro-mcp-oauth/src/config.rs`
- **Commit:** 80aebced

## Known Stubs

The following stub files from Plan 01 remain unfilled:

| File | Stub Type | Plan that fills it |
|------|-----------|-------------------|
| `ferro-mcp-oauth/src/authorize.rs` | Empty module | Plan 04 |
| `ferro-mcp-oauth/src/consent.rs` | Empty module | Plan 04 |
| `ferro-mcp-oauth/src/token.rs` | Empty module | Plan 04 |
| `ferro-mcp-oauth/src/pkce.rs` | Empty module | Plan 03 |
| `ferro-mcp-oauth/src/jwt.rs` | Empty module | Plan 03 |
| `ferro-mcp-oauth/src/validate.rs` | Returns `Option<serde_json::Value>` | Plan 03 |
| `ferro-mcp-oauth/tests/flow_integration.rs` | Harness skeleton only | Plan 04 |

`store.rs` now provides `OAuthCode` struct (Plan 04 cache serialization) and `find_by_client_id` (Plan 04 `/authorize` validation seam) — those stubs are filled by this plan.

## Threat Surface Scan

This plan fills the `POST /register` and `GET /.well-known/*` endpoints. Both are already enumerated in the plan's `<threat_model>`:

- T-199-05 (redirect_uri scheme): mitigated — `is_redirect_uri_allowed` enforces `https://` or `http://localhost` only; `javascript:`, `data:`, custom schemes, and arbitrary HTTP hosts rejected with 400.
- T-199-DCR (client_id enumeration): mitigated — 16-byte random URL-safe base64 (`rand::RngCore::fill_bytes`), equivalent to UUIDv4 entropy.
- T-199-04a (redirect_uris verbatim storage): mitigated — stored as JSON-array text via `serde_json::to_string`; Plan 04 exact-match seam reads this verbatim.
- T-199-DISC (discovery leak): accept — discovery docs expose only endpoint URLs derived from `APP_URL`, no secret; `sanitized_app_url()` deliberately skips `MCP_TOKEN_SECRET`.

No new threat surface introduced beyond what the plan's threat register covers.

## Self-Check: PASSED

- `ferro-mcp-oauth/src/discovery.rs` exists and contains `protected_resource_metadata`, `authorization_server_metadata`, `protected_resource_handler`, `authorization_server_handler`
- `ferro-mcp-oauth/src/register.rs` exists and contains `RegisterInput`, `is_redirect_uri_allowed`, `validate_redirect_uris`, `generate_client_id`, `register_client`
- `ferro-mcp-oauth/src/store.rs` exists and contains `OAuthCode`, `Model`, `OAuthClient`, `insert_client`, `find_by_client_id`
- `ferro-mcp-oauth/src/config.rs` contains `ENV_LOCK`, `sanitized_app_url` (no longer `#[allow(dead_code)]`)
- Commits d5d6a30a and 80aebced exist in git log
- `cargo test -p ferro-mcp-oauth` passes (18 + 1 tests)
- `cargo clippy -p ferro-mcp-oauth --all-targets -- -D warnings` clean
