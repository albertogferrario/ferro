---
phase: 217-tenant-context-per-tenant-api-key-auth
fixed_at: 2026-06-13T00:00:00Z
review_path: .planning/phases/217-tenant-context-per-tenant-api-key-auth/217-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 217: Code Review Fix Report

**Fixed at:** 2026-06-13
**Source review:** .planning/phases/217-tenant-context-per-tenant-api-key-auth/217-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (CR-01, CR-02, WR-01; IN-01 and IN-02 excluded per fix_scope=critical_warning)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### CR-01: `BearerAuthMiddleware` only handles JWT — API keys silently rejected at middleware

**Files modified:** `app/src/middleware/bearer_auth.rs`
**Commit:** 5495e812
**Applied fix:** Replaced the `validate_bearer`-only call with `ferro_mcp_server::resolve_tenant`, which branches on `ferro_` prefix internally. Added `ferro::DB::connection()` call inside `handle()` to obtain the DB handle needed for API-key lookup (fail closed on connection error — same pattern as the controller). Updated imports: removed `validate_bearer`, added `resolve_tenant`. Updated module doc comment to describe both JWT and API-key paths. The struct field list is unchanged (`mcp_config` only — DB is obtained at request time from the global registry, not stored in the middleware struct).

**Borrow note:** No borrow issues arose here; the `auth_header` is cloned to `String` before being passed as `Option<&str>`, matching the original pattern.

---

### CR-02: `McpContext::scope` never populated — scope gate always treats callers as `read_write`

**Files modified:** `app/src/controllers/mcp.rs`
**Commit:** f99c2b9e
**Applied fix:** Extracted `key_scope: Option<String>` from `principal` immediately after `user_id` is parsed — before `req.json()` consumes `req` (the borrow checker requires this ordering because `principal` borrows `req`). Both `tools/list` and `tools/call` arms now build a real `McpContext` with the resolved `scope` and `tenant_id` instead of `McpContext::default()`. The `tools/list` arm clones `key_scope` since the `tools/call` arm also needs it. For JWT principals (no `scope` field), `key_scope` is `None`, which `jsonrpc.rs` maps to `"read_write"` — the OAuth path is unaffected by design.

---

### WR-01: SQL injection risk in test seed helpers (test-only, but pattern concern)

**Files modified:** `ferro-mcp-oauth/src/validate.rs`, `ferro-mcp-server/tests/mcp_tenant_isolation.rs`
**Commit:** 7cccad03
**Applied fix:** Converted both `seed_key` (in `validate.rs` `#[cfg(test)]` block) and `seed_api_key` (in `mcp_tenant_isolation.rs`) from `Statement::from_string` with `format!()` interpolation to `Statement::from_sql_and_values` with `[Value::BigInt, Value::String, Value::String, Value::String]` bind parameters. The `revoked_at` NULL case uses `Value::String(None)` (parameterized NULL) instead of the bare SQL keyword `NULL`. The unused `hash_mcp_api_key` import in `mcp_tenant_isolation.rs` was removed along with the `let _ = hash_mcp_api_key;` suppression line.

---

## Verification Results

All fixes were verified with the following commands (run sequentially):

```
cargo build -p app -p ferro-mcp-server -p ferro-mcp-oauth
```
Result: `Finished dev profile` — clean build, no errors.

```
cargo test -p ferro-mcp-server -p ferro-mcp-oauth
```
Result: 111 tests passed (84 ferro-mcp-oauth unit tests + 1 flow integration + 17 ferro-mcp-server unit tests + 5 dispatch integration + 5 jsonrpc integration + 4 mcp_tenant_isolation integration). 0 failures.

```
cargo test -p app
```
Result: 16 tests passed (including all 3 `mcp_tenant_isolation` integration tests and all `controllers::mcp` unit tests). 0 failures.

```
cargo clippy -p app -p ferro-mcp-server --all-targets -- -D warnings
```
Result: `Finished dev profile` — no warnings, no errors.

---

_Fixed: 2026-06-13_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
