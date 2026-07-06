---
phase: 198-streamable-http-endpoint-unauthenticated-challenge
reviewed: 2026-06-10T00:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - app/Cargo.toml
  - app/src/controllers/mcp.rs
  - app/src/controllers/mod.rs
  - app/src/projections/order.rs
  - app/src/routes.rs
  - ferro-mcp-server/src/auth.rs
  - ferro-mcp-server/src/config.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - ferro-mcp-server/src/lib.rs
  - ferro-mcp-server/tests/common/mod.rs
  - ferro-mcp-server/tests/dispatch_integration.rs
  - ferro-mcp-server/tests/jsonrpc_integration.rs
findings:
  critical: 1
  warning: 2
  info: 1
  total: 4
status: resolved
resolved: 2026-06-10
resolution: >
  CR-01 fixed (sanitize_identity strips ASCII control chars incl. CRLF from
  APP_URL/APP_NAME at the from_env trust boundary; 2 tests). WR-01 fixed (offset
  clamped to MAX_OFFSET before the as-i64 cast in dispatch.rs). WR-02 fixed (new
  Error::InvalidFilter variant → JSON-RPC -32602; DB/internal errors → -32603;
  integration test added). IN-01 addressed (clarifying comment on the test
  fixture's hardcoded-literal SQL). fmt + clippy(-D warnings) + scoped tests green.
---

# Phase 198: Code Review Report

**Reviewed:** 2026-06-10
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Phase 198 introduces a Streamable HTTP MCP endpoint with an unauthenticated 401 challenge seam. The bearer seam, JSON-RPC dispatch wiring, and the separation between the HTTP adapter (`mcp.rs`) and the pure dispatch layer (`jsonrpc.rs` / `dispatch.rs`) are architecturally sound. The Phase 197 security work (filter-key allowlist, `MAX_LIMIT` clamp, parameterized binding, deterministic `ORDER BY`) is present and correct.

One critical issue: the `WWW-Authenticate` challenge header is constructed by interpolating `APP_URL` directly from an environment variable into a response header string, with no CRLF stripping. Any newline character in that env value produces a header injection. Two warnings: `offset` lacks the same overflow guard that `limit` has via `MAX_LIMIT`, and `dispatch` errors are uniformly mapped to JSON-RPC `-32602` (Invalid params) regardless of whether the failure was a bad-input or an internal server error.

## Critical Issues

### CR-01: Header injection via `APP_URL` in `WWW-Authenticate`

**File:** `app/src/controllers/mcp.rs:26-29`

**Issue:** `challenge_response` builds the `WWW-Authenticate` value by directly interpolating `config.app_url` (sourced from the `APP_URL` environment variable, `config.rs:22`) into an HTTP header string using `format!`. If `APP_URL` contains a CRLF sequence (`\r\n`) — possible through a mishandled `.env` file, a CI/CD secret injection, or a deployment config error — the injected newline terminates the current header and begins a new one. This is a classic HTTP response-splitting / header-injection vulnerability. The attacker controls the injected header content (and can append an arbitrary response body) wherever a proxy or client parses headers line-by-line.

The same construction path exists for any ferro-* crate that builds an HTTP header from `app_url` without sanitization.

**Fix:** Strip CRLF characters from `app_url` before interpolating it into any header value, or reject the value at config-load time:

```rust
// In config.rs or at the call site, sanitize before use in headers:
fn sanitize_url_for_header(url: &str) -> String {
    url.replace(['\r', '\n'], "")
}

fn challenge_response(config: &McpServerConfig) -> HttpResponse {
    let safe_url = sanitize_url_for_header(&config.app_url);
    let challenge = format!(
        "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        safe_url
    );
    HttpResponse::new()
        .status(401)
        .header("WWW-Authenticate", challenge)
}
```

Alternatively, validate at `McpServerConfig::from_env()` that `APP_URL` does not contain CR or LF and return an `Err` (fail-fast on startup) rather than silently using a potentially poisoned value.

## Warnings

### WR-01: `offset` cast from `u64` to `i64` without an upper bound clamp

**File:** `ferro-mcp-server/src/dispatch.rs:177`

**Issue:** `limit` is clamped to `MAX_LIMIT` (100) at line 107, with an explicit comment explaining that values beyond `i64::MAX` would wrap negative on `as i64`. The same risk exists for `offset`: it is passed in as `u64` from `handle_tools_call` (via `Value::as_u64()`, jsonrpc.rs:72), and cast to `i64` at dispatch.rs:177 with no upper-bound guard. An offset of `u64::MAX` casts to `-1_i64`. Most databases will either reject a negative offset or silently skip all rows; neither outcome is correct, and the inconsistency relative to `limit`'s treatment is a latent bug when `offset` is elevated to user-visible pagination.

**Fix:** Apply a matching constant or a hard ceiling for offset, mirroring the limit guard:

```rust
// At the top of dispatch.rs, alongside MAX_LIMIT:
const MAX_OFFSET: u64 = i64::MAX as u64;

// In the dispatch function, after the existing limit clamp:
let limit = limit.min(MAX_LIMIT);
let offset = offset.min(MAX_OFFSET);
```

`i64::MAX` is large enough to never be a practical constraint on real pagination, and it closes the cast-wraps-negative path.

### WR-02: All `dispatch` errors mapped to JSON-RPC `-32602` (Invalid params)

**File:** `ferro-mcp-server/src/jsonrpc.rs:91`

**Issue:** The `Err(e)` arm of the `dispatch(...)` match maps every failure to error code `-32602` (Invalid params). JSON-RPC 2.0 reserves `-32602` for "the JSON sent is not a valid Request object" / malformed method params — i.e., a client error. Database failures (connection errors, query errors from `dispatch.rs` lines 148-149 and 182-183) are internal server errors that belong to `-32603` (Internal error). Using `-32602` for a DB connection failure tells the client its parameters were wrong, which is incorrect and misleads retry logic.

Only the filter-key-not-allowed path (`unknown or non-filterable filter field`) in `dispatch` is genuinely a parameter problem; all other `dispatch` errors are internal.

**Fix:** Distinguish the error kinds when mapping:

```rust
match dispatch(service, filters, limit, offset, db).await {
    Ok(result) => json!({
        "result": {
            "content": result.rows,
            "total": result.total,
            "limit": result.limit,
            "offset": result.offset
        }
    }),
    Err(crate::Error::Database(msg)) if msg.starts_with("unknown or non-filterable") => {
        json!({ "error": { "code": -32602, "message": msg } })
    }
    Err(e) => json!({ "error": { "code": -32603, "message": e.to_string() } }),
}
```

Alternatively, add an `InvalidParam` variant to `crate::Error` so the dispatch layer can signal the distinction without string matching at the jsonrpc layer.

## Info

### IN-01: Test fixture uses string-interpolated SQL inserts

**File:** `ferro-mcp-server/tests/common/mod.rs:13-16`

**Issue:** The `setup_db` helper builds `INSERT` statements via `format!` with directly interpolated values rather than parameterized binding. The values are hardcoded string and integer literals so there is no injection risk in practice, but the pattern is inconsistent with the parameterized production code in `dispatch.rs`. If the fixture is extended to accept externally supplied test parameters, the unsafe pattern will already be established.

**Fix:** Use `Statement::from_sql_and_values` with bound parameters (matching the production pattern), or add a comment marking the values as hardcoded constants to signal that the pattern is intentionally not repeated with variable input.

---

_Reviewed: 2026-06-10_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
