---
phase: 217-tenant-context-per-tenant-api-key-auth
reviewed: 2026-06-13T00:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - ferro-mcp-oauth/src/validate.rs
  - ferro-mcp-oauth/src/migration.rs
  - ferro-mcp-oauth/src/lib.rs
  - ferro-mcp-server/src/auth.rs
  - ferro-mcp-server/src/error.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/lib.rs
  - ferro-mcp-server/tests/mcp_tenant_isolation.rs
  - ferro-mcp-server/tests/common/mod.rs
  - ferro-mcp-server/tests/jsonrpc_integration.rs
  - app/src/controllers/mcp.rs
  - app/src/tests/mcp_tenant_isolation.rs
findings:
  critical: 2
  warning: 1
  info: 2
  total: 5
status: issues_found
---

# Phase 217: Code Review Report

**Reviewed:** 2026-06-13
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

This phase adds per-tenant API-key auth (`ferro_`-prefixed keys, SHA-256 stored, scope `read`/`read_write`) alongside the existing JWT path. The `ferro-mcp-oauth` layer (`validate.rs`) is implemented correctly: hash-only storage, parameterized SQL lookup, fail-closed on DB error, revocation check, tenant mismatch → `Forbidden`. The `dispatch.rs` tenant predicate injection is correct and fail-closed (tenant_column=Some + tenant_id=None → Err).

Two critical wiring gaps exist at the HTTP adapter layer (`app/src/controllers/mcp.rs`). Together they mean: API keys are silently rejected at the middleware layer (CR-01), and if that is fixed, the scope enforcement is bypassed because `McpContext::scope` is never populated from the resolved principal (CR-02). The underlying library implementations are sound; the issues are in how the controller wires them.

## Critical Issues

### CR-01: `BearerAuthMiddleware` only handles JWT — API keys silently rejected at middleware

**File:** `app/src/middleware/bearer_auth.rs:37`, `app/src/routes.rs:60-65`

**Issue:** `BearerAuthMiddleware::handle` calls `validate_bearer` (JWT-only). A `ferro_`-prefixed API key token hits `decode_token`, which attempts to parse it as a JWT, fails, and returns `BearerCheck::Invalid` → 401. The key never reaches the handler. The `resolve_tenant` function in `ferro-mcp-server/src/auth.rs` (which branches on `ferro_` prefix) is exported but never called anywhere in the live request path. The API-key auth path is dead code at the HTTP level despite `validate_api_key` being fully implemented in `ferro-mcp-oauth`.

**Fix:** `BearerAuthMiddleware` needs to branch on token shape, mirroring `resolve_tenant`. It also needs a database handle to call `validate_api_key`. The cleanest fix is to use `resolve_tenant` directly from the middleware:

```rust
// bearer_auth.rs — add db: DatabaseConnection field
pub struct BearerAuthMiddleware {
    pub mcp_config: McpServerConfig,
    pub db: DatabaseConnection,
}

#[async_trait]
impl Middleware for BearerAuthMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let auth_header = request.header("Authorization").map(|s| s.to_owned());
        let oauth_config =
            OAuthConfig::from_env().map_err(|_| challenge_response(&self.mcp_config))?;

        // Branches on ferro_ prefix internally.
        let check = ferro_mcp_server::resolve_tenant(
            auth_header.as_deref(),
            &self.db,
            &oauth_config,
        )
        .await;

        match check {
            BearerCheck::Unauthenticated => Err(challenge_response(&self.mcp_config)),
            BearerCheck::Invalid => Err(HttpResponse::new()
                .status(401)
                .header("WWW-Authenticate", "Bearer error=\"invalid_token\"")),
            BearerCheck::Forbidden => Err(HttpResponse::new().status(403)),
            BearerCheck::Authenticated(principal) => {
                request.insert::<serde_json::Value>(principal);
                next(request).await
            }
        }
    }
}
```

Note: `resolve_tenant` currently passes `expected_tenant: None` to both paths, which is correct here — `TenantMiddleware` handles the tenant-context establishment downstream.

---

### CR-02: `McpContext::scope` never populated — scope gate always treats callers as `read_write`

**File:** `app/src/controllers/mcp.rs:158`, `ferro-mcp-server/src/jsonrpc.rs:68`

**Issue:** `handle_tools_call` is called with `&McpContext::default()` (line 158 of the controller). `McpContext::default()` sets `scope: None`. In `jsonrpc.rs:68`, `None` maps to `"read_write"` via `ctx.scope.as_deref().unwrap_or("read_write")`. This means a `read`-scoped API key can call any write tool — the scope enforcement code exists and is correct in isolation, but is bypassed because the scope from the validated principal is never threaded into `McpContext`.

The principal inserted by the middleware (`BearerCheck::Authenticated(principal)`) carries `scope` only for the API-key path (validate.rs:197). The controller reads `principal["sub"]` (line 76) and `current_tenant()` (line 157) but never reads `principal["scope"]` to construct a non-default `McpContext`.

**Fix:** Read `scope` from the principal (only present for API-key path; absent for JWT path) and build `McpContext` with it:

```rust
// app/src/controllers/mcp.rs — inside tools/call branch, after resolving tenant_id

// principal is already in scope from step 2.
let key_scope = principal.get("scope")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string()); // None for JWT path → full access in jsonrpc.rs

let ctx = McpContext {
    tenant_id,
    scope: key_scope,
    ..Default::default()
};

handle_tools_call(params, &services, db.inner(), tenant_id, &ctx).await
```

The `None` → `"read_write"` default in `jsonrpc.rs` is intentional for the JWT path (where no `scope` field is present); this fix preserves that semantics while making the API-key path actually pass through its scope.

---

## Warnings

### WR-01: SQL injection risk in test seed helpers (test-only, but pattern concern)

**File:** `ferro-mcp-oauth/src/validate.rs:364-369`, `ferro-mcp-server/tests/mcp_tenant_isolation.rs:100-108`

**Issue:** The `seed_key` / `seed_api_key` test helpers build INSERT statements via `format!()` string interpolation, including `{key_hash}` (a SHA-256 hex string) and `{scope}` (caller-controlled string literal in tests). The `key_hash` value is a 64-character hex string and cannot inject SQL in practice. However, `scope` is a `&str` parameter that callers could pass with embedded SQL. In the current tests, scope is always `"read"` or `"read_write"`, so there is no actual injection — but the pattern is unsafe and conflicts with the codebase's own stated convention (the comment in `tests/common/mod.rs:15-17` explicitly flags this pattern as test-only and warns not to copy it to production code).

The concern is that `seed_key` is a private helper inside `validate.rs`'s `#[cfg(test)]` block, making accidental promotion unlikely — but the same `format!` pattern is duplicated in `mcp_tenant_isolation.rs` where it is not inside `#[cfg(test)]` at the module level.

**Fix:** Use parameterized statements in seed helpers to make them safe regardless of input, and remove the copy in the integration test in favour of a shared helper:

```rust
// Use Statement::from_sql_and_values even in test fixtures
db.execute(Statement::from_sql_and_values(
    DatabaseBackend::Sqlite,
    "INSERT INTO mcp_api_keys (tenant_id, key_hash, scope, revoked_at) \
     VALUES (?, ?, ?, ?)",
    [
        Value::BigInt(Some(tenant_id)),
        Value::String(Some(Box::new(key_hash))),
        Value::String(Some(Box::new(scope.to_string()))),
        revoked_at_value,
    ],
))
```

---

## Info

### IN-01: `resolve_tenant` is exported but unused in the live request path

**File:** `ferro-mcp-server/src/auth.rs`, `ferro-mcp-server/src/lib.rs:14`

**Issue:** `resolve_tenant` is the intended API-key/JWT unifier. It is exported from `ferro-mcp-server` and re-exported from `lib.rs`. It is not called anywhere in the current live request path (the controller goes through `BearerAuthMiddleware` → `validate_bearer`-only). Once CR-01 is fixed, `resolve_tenant` should be the function called from the middleware, making this export meaningful. Until then, Clippy may flag it as dead code depending on feature configuration.

**Fix:** No code change needed beyond the CR-01 fix, which will make this path live. If the dead-code warning surfaces before the fix, add `#[allow(dead_code)]` with a comment referencing the open CR-01 item, not a permanent suppress.

---

### IN-02: `tools/list` passes `McpContext::default()` — write tool names visible to read-scoped keys

**File:** `app/src/controllers/mcp.rs:93`

**Issue:** The `tools/list` dispatch uses `&McpContext::default()`, so `scope` is `None` and no write-tool filtering is applied to the listed tool names. Currently all MCP tools are `list_`-prefixed (read-only) due to the current `McpRenderer` design, making this a non-issue for the current projection set. If a non-`list_` tool is ever added to `exposed_services()`, read-scoped keys would see its name in the tool list even though calling it is blocked by the scope gate.

This is lower priority than CR-02 (the actual scope enforcement gap) but should be addressed when non-read tools are introduced.

**Fix:** Once CR-02 is fixed, build `McpContext` with the resolved scope before the `match method` dispatch block, and pass the same context to `handle_tools_list`:

```rust
// Build ctx once, reuse for both tools/list and tools/call
let ctx = McpContext {
    tenant_id: ferro::current_tenant().map(|t| t.id),
    scope: principal.get("scope").and_then(|v| v.as_str()).map(str::to_string),
    ..Default::default()
};
// then use ctx in all match arms
```

---

_Reviewed: 2026-06-13_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
