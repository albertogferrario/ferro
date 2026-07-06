# Phase 217: Tenant Context + Per-Tenant API-Key Auth — Research

**Researched:** 2026-06-13
**Domain:** ferro-mcp-server + ferro-mcp-oauth — tenant context embedding, API-key validation branch, scope enforcement
**Confidence:** HIGH (all claims verified by direct source file reads)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** `validate_api_key` lives in `ferro-mcp-oauth/src/validate.rs`, parallel to `validate_bearer`. Returns `BearerCheck::Authenticated(principal)`.  `ferro-mcp-server/src/auth.rs` becomes a thin unifier (`resolve_tenant`) that branches on token shape and delegates to one of the two `ferro-mcp-oauth` validators.
- **D-02:** Branch detection on token shape: `ferro_`-prefix → `validate_api_key`; otherwise → `validate_bearer`. Absent header → `Unauthenticated` (401).
- **D-03:** SHA-256 hash lookup: `SELECT tenant_id, scope FROM api_keys WHERE key_hash = SHA256(key) AND revoked_at IS NULL`. Plaintext keys never stored. Uses `sha2 = "0.10"` (already present). `subtle` available for constant-time comparison.
- **D-04:** Framework helper to mint a `ferro_`-prefixed key (CSPRNG, base62/base64url entropy). Rotation is soft-revoke + new issuance. Confirm v8.1 `make:api-key` schema before designing new generator.
- **D-05:** Framework defines canonical `api_keys` schema (`id`, `tenant_id`, `key_hash`, `scope`, `revoked_at`/`active`, timestamps). Consumer runs migration. `ferro-mcp-oauth` ships lookup contract + generator helper only.
- **D-06:** API keys carry `scope: read | read_write` from the first migration. `tools/list` filters to scope. `tools/call` re-checks scope independently before dispatching a write tool. In 217 no write tools exist; scope-rejection path is wired and tested against the empty write-tool set.
- **D-07:** `McpContext { tenant_id: Option<i64>, evaluated_guards: HashMap<String, bool> }`, `#[derive(Debug, Clone, Default)]`. Constructed at top of request handler after auth. `evaluated_guards` is empty map in 217; field and its read sites exist for 218/219.
- **D-08:** Add `Auth(String)` variant to `ferro-mcp-server/src/error.rs`. Invalid/expired API key rejected before tool routing, same JSON-RPC error envelope as OAuth invalid-token path.
- **D-09:** Extend existing `app/src/tests/mcp_tenant_isolation.rs` in the same commit: authenticate via API key as tenant A, assert no tool listing or call surfaces tenant B data; assert tenant-A-key and tenant-A-JWT resolve the same `tenant_id`.

### Claude's Discretion
- Exact column names/types in the `api_keys` migration (within D-05's shape).
- Whether `read`/`read_write` scope is a SeaORM enum, `TEXT` check-constrained column, or small int.
- Token entropy length and exact `ferro_` prefix format, provided it is unambiguously distinguishable from a JWT.
- Whether `validate_api_key` takes `expected_tenant: Option<i64>` (mirroring `validate_bearer`'s signature) or resolves tenant purely from the row.

### Deferred Ideas (OUT OF SCOPE)
- Write-tool rendering from `ActionDef`, guard filtering, `destructiveHint` annotations — Phase 218.
- `dispatch_write()` + server-side guard re-evaluation at execution + idempotency keys + audit log — Phase 219.
- `ferro-ai` confirmation gating for destructive actions — Phase 220.
- Inbound NL intent loop + replay/smoke CI path — Phase 221.
- Fine-grained `abilities[]` per-action scoping on the key (beyond `read`/`read_write`) — future v15.x.
- DB-backed confirmation store, per-call audit trail / key-usage logging — production hardening, deferred.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AMCP-01 | The MCP endpoint resolves the calling tenant and that tenant's evaluated guards into the render/call context, so every tool listing and tool call is tenant- and permission-scoped. (`McpContext` embeds `BaseContext` — `tenant_id` + `evaluated_guards`; today it is an empty struct.) | McpContext is currently `struct McpContext;` at `ferro-mcp-server/src/renderer.rs:10`. Extension is a struct field addition. `handle_tools_list` and `handle_tools_call` in `jsonrpc.rs` both take `services` + `db` — `McpContext` is constructed at the top of the JSON-RPC handler and threaded through. |
| AMCP-02 | A tenant authenticates to the MCP endpoint with a per-tenant API key (alongside the existing OAuth path), and the resolved principal scopes both the visible tool set and all data access to that tenant. | `BearerCheck` in `ferro-mcp-oauth/src/validate.rs` is the unifying outcome type. `validate_bearer` is the template. The `api_keys` table (new migration, consumer-side) is the storage. `validate_api_key` (new function in `validate.rs`) does the SHA-256 lookup. The unifier `resolve_tenant` in `ferro-mcp-server/src/auth.rs` (replaces the stub `BearerOutcome`) routes to the correct validator. |
</phase_requirements>

---

## Summary

Phase 217 has two non-interleaved concerns: (1) extend `McpContext` from an empty struct to carry `tenant_id` + `evaluated_guards` and wire it through the existing JSON-RPC handler, and (2) add a second auth validation branch for API keys that produces the same `BearerCheck::Authenticated(principal)` the OAuth path already produces.

The read paths through `tools/list` and `tools/call` already accept `tenant_id: Option<i64>` in `handle_tools_call`; the `McpContext` extension is purely additive. The `auth.rs` stub `BearerOutcome` enum (2 variants, one `#[allow(dead_code)]`) is replaced by a real `resolve_tenant` unifier that delegates to `ferro-mcp-oauth`.

**Primary recommendation:** Implement `validate_api_key` in `ferro-mcp-oauth/src/validate.rs` using `hash_api_key` from the existing `framework/src/api/api_key.rs` pattern (SHA-256 hex). Define the `api_keys` migration schema in `ferro-mcp-oauth/src/migration.rs` (alongside `CreateOauthClientsTable`) using columns `id, tenant_id, key_hash, scope, revoked_at, created_at`. Wire the unifier in `ferro-mcp-server/src/auth.rs` and extend `McpContext`. Add `Auth(String)` to `error.rs`. Create `ferro-mcp-server/tests/mcp_tenant_isolation.rs` (it does not yet exist there; the existing isolation tests live in `app/src/tests/mcp_tenant_isolation.rs` which uses the consumer app's full migrations and models — the ferro-mcp-server tests directory has only `dispatch_integration.rs` and `jsonrpc_integration.rs`).

**Critical pre-planning finding:** The `ferro-mcp-server/tests/mcp_tenant_isolation.rs` file cited in CONTEXT.md D-09 does NOT exist. The isolation tests for Phase 200 live in `app/src/tests/mcp_tenant_isolation.rs`. Phase 217 must create a new file at `ferro-mcp-server/tests/mcp_tenant_isolation.rs` using the simpler in-process SQLite fixture pattern (like `dispatch_integration.rs`), not the full app model stack. This is a Wave 0 gap.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Bearer token validation (JWT + API key) | `ferro-mcp-oauth` | — | Auth outcome types (`BearerCheck`) owned here; `validate_bearer` template already here |
| Auth unifier / branch detection | `ferro-mcp-server/src/auth.rs` | — | Server seam — routes `ferro_`-prefix tokens to api-key path, JWTs to OAuth path |
| `McpContext` struct | `ferro-mcp-server/src/renderer.rs` | — | Already defined here as the `Renderer::Context` associated type |
| `McpContext` threading | `ferro-mcp-server/src/jsonrpc.rs` | — | `handle_tools_list` and `handle_tools_call` consume the context |
| `api_keys` schema definition | `ferro-mcp-oauth/src/migration.rs` | Consumer app | Framework defines canonical schema; consumer runs it |
| Scope enforcement (`read` vs `read_write`) | `ferro-mcp-server/src/jsonrpc.rs` | — | Re-check at `tools/call` time, independent of listing filter |
| Error variants | `ferro-mcp-server/src/error.rs` | — | `Auth(String)` variant added here |
| Cross-tenant isolation tests | `ferro-mcp-server/tests/mcp_tenant_isolation.rs` | — | New file (D-09); must use in-process SQLite, not app model stack |

---

## Standard Stack

### Core (no new external dependencies required)

| Library | Version | Purpose | Location |
|---------|---------|---------|----------|
| `sha2` | `"0.10"` | SHA-256 hash of raw API key | `ferro-mcp-oauth/Cargo.toml` — already present [VERIFIED: Cargo.toml read] |
| `subtle` | `"2.5"` | Constant-time comparison of hex digests | `ferro-mcp-oauth/Cargo.toml` — already present [VERIFIED: Cargo.toml read] |
| `rand` | `"0.8"` | CSPRNG for key generation (already used in `framework/src/api/api_key.rs`) | `ferro-mcp-oauth/Cargo.toml` — already present [VERIFIED: Cargo.toml read] |
| `sea-orm` | `"1.0"` | DB lookup in `api_keys` table | `ferro-mcp-oauth/Cargo.toml` — already present [VERIFIED: Cargo.toml read] |
| `thiserror` | `"1"` | `Auth(String)` error variant | `ferro-mcp-server/Cargo.toml` — already present [VERIFIED: Cargo.toml read] |

No new dependencies are required for Phase 217 in any crate.

### Dependency Gap (important)

`ferro-mcp-server` does NOT currently depend on `ferro-mcp-oauth`. [VERIFIED: grep returned no output.] This is intentional per `ferro-mcp-oauth/src/lib.rs` module docs: "Consumers mount the routes and call `validate_bearer` directly — `ferro-mcp-server` gains no new dependency." The unifier in `ferro-mcp-server/src/auth.rs` therefore CANNOT import `BearerCheck` from `ferro-mcp-oauth` directly.

**Resolution options (Claude's Discretion):**

Option A — Add `ferro-mcp-oauth` as a dependency of `ferro-mcp-server`. This is the cleanest approach for D-01 ("same `BearerCheck::Authenticated(principal)` outcome type") but does add a new crate-to-crate dependency. The dependency graph becomes: `ferro-mcp-server → ferro-mcp-oauth → ferro`.

Option B — Re-export `BearerCheck` from `ferro-mcp-server/src/auth.rs` as a local type alias, and call `ferro_mcp_oauth::validate_bearer`/`validate_api_key` via an `Arc<dyn Fn...>` registered at startup. Complex and creates a parallel type.

Option C — Move `BearerCheck` to a shared primitive crate (overkill for Phase 217).

**Recommendation:** Option A. Add `ferro-mcp-oauth = { path = "../ferro-mcp-oauth", version = "0.2" }` to `ferro-mcp-server/Cargo.toml`. This is one line. The `lib.rs` comment ("gains no new dependency") described the v12.6 state before the write-path phases; the v15.0 auth unifier legitimately requires it. The planner must decide and add it to the Wave 0 task.

---

## Key Code Touchpoints

### 1. `BearerCheck` and `validate_bearer` — `ferro-mcp-oauth/src/validate.rs`

**Current state** [VERIFIED: file read]:

```rust
// Line 35-44
pub enum BearerCheck {
    Unauthenticated,
    Invalid,
    Forbidden,
    Authenticated(serde_json::Value),  // principal = json!({"sub": ..., "tenant_id": ...})
}

// Line 53-57 — signature to parallel
pub fn validate_bearer(
    authorization_header: Option<&str>,
    config: &OAuthConfig,
    expected_tenant: Option<i64>,
) -> BearerCheck
```

Branch detection inside `validate_bearer` (line 63-66):
```rust
let token = match header.strip_prefix("Bearer ") {
    None | Some("") => return BearerCheck::Unauthenticated,
    Some(t) => t,
};
```

The authenticated principal shape (line 94-97):
```rust
BearerCheck::Authenticated(serde_json::json!({
    "sub": claims.sub,
    "tenant_id": claims.tenant_id,
}))
```

**What `validate_api_key` must produce:** The same `BearerCheck::Authenticated(json!({"sub": <user_id_string_or_key_id>, "tenant_id": <i64>}))` shape. For API keys there is no OAuth `sub` (user ID) — the key IS the credential. Convention: use the key's row `id` as `sub` (as a string), or a synthetic `"api_key:<id>"` string. The planner must decide; both work since downstream code only reads `tenant_id` from the principal.

**Recommended `validate_api_key` signature** (mirrors `validate_bearer`, returns `BearerCheck` — no `async fn` in `validate_bearer` but DB lookup IS async):

```rust
// ferro-mcp-oauth/src/validate.rs (new function)
pub async fn validate_api_key(
    authorization_header: Option<&str>,
    db: &sea_orm::DatabaseConnection,
    expected_tenant: Option<i64>,  // mirrors validate_bearer; enables future per-endpoint scoping
) -> BearerCheck
```

Implementation sketch:
1. Strip `"Bearer "` prefix → `Unauthenticated` if absent.
2. Check `token.starts_with("ferro_")` → if not, return `Unauthenticated` (caller should have routed to `validate_bearer`; defensive fallback).
3. Hash the raw token: `hash_api_key(token)` → `String` (SHA-256 hex).
4. `SELECT id, tenant_id, scope, revoked_at FROM api_keys WHERE key_hash = ?` via SeaORM raw SQL or entity.
5. Row not found → `Invalid`.
6. `revoked_at IS NOT NULL` → `Invalid`.
7. If `expected_tenant = Some(t)` and `row.tenant_id != t` → `Forbidden`.
8. Return `BearerCheck::Authenticated(json!({"sub": row.id.to_string(), "tenant_id": row.tenant_id, "scope": row.scope}))`.

Note: `validate_bearer` is synchronous (JWT decode is sync); `validate_api_key` is async (DB lookup). This asymmetry means `auth.rs::resolve_tenant` must be `async fn`.

### 2. `McpTokenClaims` — `ferro-mcp-oauth/src/jwt.rs`

**Current state** [VERIFIED: file read]:

```rust
// Line 22-37
pub struct McpTokenClaims {
    pub sub: String,           // user ID as string
    pub tenant_id: Option<i64>, // LOAD-BEARING: must be "tenant_id" for JwtClaimResolver
    pub aud: Vec<String>,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
}
```

The principal shape `json!({"sub": ..., "tenant_id": ...})` in `validate_bearer` line 94-97 is derived from `McpTokenClaims`. `validate_api_key` produces the same shape (no `aud`/`iss`/`iat`/`exp` in the principal — only `sub` and `tenant_id`). This is correct: both paths produce the same principal dict, just via different validation paths.

### 3. `McpContext` — `ferro-mcp-server/src/renderer.rs`

**Current state** [VERIFIED: file read, lines 9-10]:

```rust
/// Context for MCP rendering. Carries no state in Phase 197;
/// Phase 200 will extend with tenant/policy context.
#[derive(Debug, Clone, Default)]
pub struct McpContext;
```

**Phase 217 target:**

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct McpContext {
    pub tenant_id: Option<i64>,
    pub evaluated_guards: HashMap<String, bool>,
}
```

`render_exposed_tools` signature (line 56-58) takes `ctx: &McpContext` — already correct; no signature change needed. Internal callers that pass `&McpContext` (the unit-constructed default) will continue to compile because `McpContext` derives `Default`.

`handle_tools_list` in `jsonrpc.rs` (line 33-38) currently passes `&McpContext` (the zero-value struct). After the extension, it must pass a fully constructed `McpContext` with the resolved `tenant_id`. This requires `handle_tools_list` to accept `tenant_id: Option<i64>` or a pre-built `McpContext`. The planner must choose the approach.

### 4. `dispatch()` — `ferro-mcp-server/src/dispatch.rs`

**Current state** [VERIFIED: file read, lines 108-115]:

```rust
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,      // ← already present
) -> crate::Result<DispatchResult>
```

`tenant_id` is already a typed parameter. The fail-closed guarantee is at lines 152-166:
```rust
None => {
    return Err(crate::Error::InvalidFilter(
        "tenant context required but not present".to_string(),
    ));
}
```

**No changes to `dispatch()` are required for Phase 217.** The `McpContext.tenant_id` simply needs to be extracted and passed to `dispatch` (already happens in `handle_tools_call` at line 84: `dispatch(service, filters, limit, offset, db, tenant_id).await`).

### 5. `BearerOutcome` stub and `auth.rs` — `ferro-mcp-server/src/auth.rs`

**Current state** [VERIFIED: file read, lines 1-10]:

```rust
//! Bearer-token outcome type for the MCP endpoint.
pub enum BearerOutcome {
    Unauthenticated,
    #[allow(dead_code)]
    Authenticated(serde_json::Value),
}
```

This stub is entirely replaced in Phase 217. The replacement is:

```rust
// ferro-mcp-server/src/auth.rs (Phase 217 replacement)
use ferro_mcp_oauth::BearerCheck;  // requires adding ferro-mcp-oauth as dep
use sea_orm::DatabaseConnection;

/// Resolve the calling tenant from the Authorization header.
/// Branches on token shape: `ferro_`-prefix → validate_api_key, else → validate_bearer.
pub async fn resolve_tenant(
    authorization_header: Option<&str>,
    db: &DatabaseConnection,
    oauth_config: &ferro_mcp_oauth::OAuthConfig,
) -> BearerCheck {
    let token = match authorization_header.and_then(|h| h.strip_prefix("Bearer ")) {
        None | Some("") => return BearerCheck::Unauthenticated,
        Some(t) => t,
    };
    if token.starts_with("ferro_") {
        ferro_mcp_oauth::validate::validate_api_key(
            authorization_header, db, None
        ).await
    } else {
        ferro_mcp_oauth::validate_bearer(authorization_header, oauth_config, None)
    }
}
```

`BearerCheck` is re-exported from `ferro-mcp-server/src/lib.rs` so callers in the consumer app don't need to import `ferro-mcp-oauth` directly.

### 6. JSON-RPC handler — `ferro-mcp-server/src/jsonrpc.rs`

**Current state** [VERIFIED: file read]:

`handle_tools_list` (line 33): `pub async fn handle_tools_list(services: &[ServiceDef], _config: &McpServerConfig) -> Value` — passes `&McpContext` (zero-value). Must be updated to accept `ctx: &McpContext` (or `tenant_id`) and pass the real context.

`handle_tools_call` (line 49): `pub async fn handle_tools_call(call_params: Value, services: &[ServiceDef], db: &sea_orm::DatabaseConnection, tenant_id: Option<i64>) -> Value` — already accepts `tenant_id`. No signature change needed here. The `tenant_id` is threaded into `dispatch()` correctly.

The HTTP adapter (consumer app `app/src/routes.rs`) is where `resolve_tenant` is called, the `McpContext` is built, and the appropriate `handle_*` function is invoked. This is NOT inside `ferro-mcp-server` — it is at the app seam. Phase 217 adds `resolve_tenant` to `ferro-mcp-server/src/auth.rs` as the callable function; the consumer app calls it.

**Signature change needed for `handle_tools_list`:** To carry `tenant_id` into `render_exposed_tools`, `handle_tools_list` needs to accept either a `McpContext` parameter or a `tenant_id: Option<i64>`. The cleanest approach (consistent with D-07): accept `ctx: &McpContext`:

```rust
pub async fn handle_tools_list(
    services: &[ServiceDef],
    ctx: &McpContext,
    _config: &McpServerConfig,
) -> Value
```

This is a breaking change to the function signature. All test call sites in `jsonrpc_integration.rs` and the consumer app need to be updated to pass `&McpContext::default()` (which carries `tenant_id: None`).

### 7. `ferro-mcp-server/src/error.rs`

**Current state** [VERIFIED: file read]:

```rust
pub enum Error {
    Render(String),
    InvalidFilter(String),
    Database(String),
    Serialization(#[from] serde_json::Error),
}
```

**Phase 217 addition:**

```rust
#[error("authentication error: {0}")]
Auth(String),
```

This maps to the same JSON-RPC error code as `Invalid` / `Forbidden` from the OAuth path (the CONTEXT.md D-08 says "identical JSON-RPC error envelope"). The convention in `jsonrpc.rs` is `-32603` for internal errors. For auth, the HTTP layer returns a 401/403, not a JSON-RPC error code. The mapping depends on where auth is checked:

- If auth is checked in the HTTP adapter (consumer app), auth errors never reach the JSON-RPC layer — they are HTTP-level rejections. The `Auth(String)` variant in `ferro-mcp-server/src/error.rs` is then used only for cases where auth state is checked INSIDE a tool handler (e.g., scope check on `tools/call`).
- The planner must clarify: is `Auth(String)` needed at the `ferro-mcp-server` level for SC#3 scope enforcement, or is it only needed in the consumer app? Answer: YES — the scope check at `tools/call` time (SC#3) happens inside `ferro-mcp-server/src/jsonrpc.rs::handle_tools_call`, so it needs an `Auth` variant to return a scope-rejection response.

---

## Critical Finding: api_keys Schema

**The `api_keys` table does NOT currently exist as a migration or SeaORM entity anywhere in the ferro workspace.** [VERIFIED: grep across framework/, ferro-cli/, ferro-mcp-oauth/, ferro-mcp-server/]

What DOES exist:

1. **`framework/src/api/api_key.rs`** — A general-purpose REST API key system (not MCP-specific). Uses `fe_live_`/`fe_test_` prefix, prefix (first 16 chars) + `hashed_key` (SHA-256) columns. Includes `generate_api_key()`, `hash_api_key()`, `verify_api_key_hash()` (constant-time via `subtle`), and `ApiKeyMiddleware`. The SQL template in the CLI output (line 84) is: `INSERT INTO api_keys (name, prefix, hashed_key, created_at)` — no `tenant_id`, no `scope`.

2. **`ferro-cli/src/commands/make_api_key.rs`** — The `ferro make:api-key` CLI command. Generates `fe_live_`/`fe_test_` keys, outputs SQL for `api_keys (name, prefix, hashed_key, created_at)`. No `tenant_id`, no `scope`.

**Conclusion:** The existing `api_keys` infrastructure uses a different schema than what Phase 217 requires. The v8.1 `make:api-key` generates general REST API keys without `tenant_id` or `scope`. Phase 217 needs MCP-specific per-tenant API keys with a different schema.

**Resolution (D-05 + D-04 reconciled):**

The framework defines a NEW canonical MCP API key schema as a `ferro-mcp-oauth` migration (placed alongside `CreateOauthClientsTable` in `ferro-mcp-oauth/src/migration.rs`). The consumer runs both migrations. The existing `framework/src/api/api_key.rs` is for general REST API key auth and is UNRELATED to MCP; do not reuse its schema.

**Recommended `mcp_api_keys` table schema** (using `mcp_api_keys` to avoid collision with the existing general `api_keys` table):

```sql
CREATE TABLE mcp_api_keys (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL,
    key_hash    TEXT NOT NULL UNIQUE,   -- SHA-256 hex of the raw `ferro_` key
    scope       TEXT NOT NULL DEFAULT 'read',  -- 'read' | 'read_write'
    revoked_at  TEXT,                  -- NULL = active; set to revoke (soft delete)
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_mcp_api_keys_key_hash ON mcp_api_keys(key_hash);
CREATE INDEX idx_mcp_api_keys_tenant_id ON mcp_api_keys(tenant_id);
```

The `CHECK (scope IN ('read', 'read_write'))` constraint can be added for SQLite; Postgres uses an enum type. Claude's Discretion per D-05/D-06.

**Key generation helper** — reuse the pattern from `framework/src/api/api_key.rs` but adapted for MCP:
- Prefix: `ferro_` (unambiguous JWT prefix distinction since JWTs start with `eyJ`)
- Entropy: 32 bytes from `rand::thread_rng()` (CSPRNG), base62-encoded = 43 chars
- Full key: `ferro_<43 chars>` = 49 chars total
- Hash: SHA-256 hex (64 chars)
- The `fe_live_`/`fe_test_` prefix from `make_api_key.rs` is for general REST keys; MCP keys use `ferro_` only (per D-02, it must distinguish from a JWT which always starts `eyJ`)

**MCP key generator function in `ferro-mcp-oauth/src/validate.rs` (or a new `ferro-mcp-oauth/src/api_key.rs`):**

```rust
use sha2::{Digest, Sha256};

/// Generate a new MCP API key. Returns `(raw_key, key_hash)`.
/// raw_key is shown once; key_hash is stored.
pub fn generate_mcp_api_key() -> (String, String) {
    use rand::Rng;
    const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    let random: String = (0..43).map(|_| BASE62[rng.gen_range(0..62)] as char).collect();
    let raw_key = format!("ferro_{random}");
    let key_hash = {
        let mut h = Sha256::new();
        h.update(raw_key.as_bytes());
        format!("{:x}", h.finalize())
    };
    (raw_key, key_hash)
}
```

---

## Architecture Patterns

### System Architecture Diagram (Phase 217 scope)

```
Consumer App HTTP handler (/mcp)
        |
        | Authorization: Bearer <token>
        v
[ferro-mcp-server/src/auth.rs]
  resolve_tenant(header, db, oauth_config)
        |
        +-- token starts with "ferro_" ──────────────────────────────┐
        |                                                             v
        |                                            [ferro-mcp-oauth/src/validate.rs]
        |                                              validate_api_key(header, db, None)
        |                                                SQL: SELECT id, tenant_id, scope
        |                                                     FROM mcp_api_keys
        |                                                     WHERE key_hash = SHA256(token)
        |                                                       AND revoked_at IS NULL
        |                                              → BearerCheck::Authenticated(principal)
        |
        +-- otherwise (JWT, starts with "eyJ") ──────────────────────┐
                                                                      v
                                                [ferro-mcp-oauth/src/validate.rs]
                                                  validate_bearer(header, oauth_config, None)
                                                  → BearerCheck::Authenticated(principal)

Both paths produce: BearerCheck::Authenticated(json!({"sub":..., "tenant_id":..., "scope":...}))
        |
        v
[Consumer App handler]
  extract tenant_id from principal
  construct McpContext { tenant_id: Some(tid), evaluated_guards: HashMap::new() }
        |
        v
[ferro-mcp-server/src/jsonrpc.rs]
  handle_tools_list(services, &ctx, config)       → render_exposed_tools(services, &ctx)
  handle_tools_call(params, services, db, tid)    → dispatch(service, filters, limit, offset, db, tid)
        |                                                    |
        | scope check (SC#3):                               v
        | if write tool && scope == "read"           SQL + tenant predicate injection
        |   → Auth error                             (already fail-closed in dispatch.rs)
        v
  CallToolResult::structured(payload)
```

### Recommended Project Structure (Phase 217 additions)

```
ferro-mcp-oauth/src/
├── validate.rs          # MODIFIED: add validate_api_key() + generate_mcp_api_key()
├── migration.rs         # MODIFIED: add CreateMcpApiKeysTable migration
└── lib.rs               # MODIFIED: pub use validate::validate_api_key; pub use api_key migration

ferro-mcp-server/src/
├── auth.rs              # REPLACED: BearerOutcome → resolve_tenant()
├── renderer.rs          # MODIFIED: McpContext struct extended
├── error.rs             # MODIFIED: Auth(String) variant added
├── jsonrpc.rs           # MODIFIED: handle_tools_list gets ctx param; scope check added
└── lib.rs               # MODIFIED: pub use new exports

ferro-mcp-server/tests/
└── mcp_tenant_isolation.rs  # NEW: cross-tenant + scope + auth-parity tests (Wave 0)
```

### Anti-Patterns to Avoid

- **Checking scope only at `tools/list`:** A `read`-scoped key's scope must be re-checked at `tools/call` time (D-06 / PITFALLS §4). The listing filter is not the auth gate.
- **Sourcing `tenant_id` from tool call arguments:** The security invariant from `dispatch.rs:104` must hold: tenant always from auth token, never from payload.
- **Adding a separate MCP endpoint for API keys:** One endpoint, two validation branches.
- **Using `expected_tenant` in `resolve_tenant`:** The MCP endpoint resolves tenant FROM the key, not the other way around. `expected_tenant` in `validate_bearer`'s signature is for the case where the caller already knows which tenant is expected (route-level scope). For the MCP auth unifier, `expected_tenant = None` is correct — the tenant is derived from the credential.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| SHA-256 hashing of API keys | Custom hash | `sha2::Sha256` (already in `ferro-mcp-oauth`) |
| Constant-time comparison | `==` on strings | `subtle::ConstantTimeEq` (already in `ferro-mcp-oauth`) — use on the hex digests |
| CSPRNG for key generation | `rand::random()` | `rand::thread_rng().gen_range()` with `BASE62` (pattern from `framework/src/api/api_key.rs:117-120`) |
| Token-shape branch detection | Complex parsing | `token.starts_with("ferro_")` — JWTs always start with `eyJ` (base64url of `{"alg":...}`) |

---

## Scope Enforcement (SC#3) — No Write Tools Yet

**The situation:** Phase 217 must wire the scope-rejection path for `tools/call` on write tools (D-06), but Phase 218 adds write tools. In Phase 217 the write-tool set is empty.

**The cleanest approach (Claude's Discretion):**

Add the scope-check gate in `handle_tools_call` that rejects non-`list_`-prefixed tool names when `scope == "read"`. In Phase 217 the only tools are `list_*` tools (read-only), so a `read` key will succeed (all tools pass the scope check). A `read_write` key also succeeds. The gate is:

```rust
// In handle_tools_call, after resolving the service, before dispatch:
let is_write_tool = !tool_name.starts_with("list_");
let key_scope = ctx.scope.as_deref().unwrap_or("read_write"); // from principal
if is_write_tool && key_scope == "read" {
    return json!({ "error": { "code": -32603, "message": "Scope insufficient: read key cannot call write tools" } });
}
```

This gate is wired now and will reject write tools the moment Phase 218 adds them. To make it testable in Phase 217 without write tools, add a unit test that directly calls the scope-check logic with a synthetic write tool name (e.g., `"create_order"`) and a `read` scope and asserts rejection. No production code path needs a real write tool for this test.

**McpContext and scope:** The `scope` from the API key principal must be accessible in `handle_tools_call`. Since `McpContext` is the context object, the planner should decide: add `scope: Option<String>` to `McpContext`, or pass scope separately. Adding it to `McpContext` is consistent with D-07. Note `validated_guards` is also in `McpContext` per D-07; scope is a separate axis (D-06 explicitly states scope governs the credential's permission, `mcp_ability` governs the tenant's ability).

---

## Common Pitfalls

### Pitfall 1: `ferro-mcp-server` has no `ferro-mcp-oauth` dependency

`ferro-mcp-server/Cargo.toml` does NOT list `ferro-mcp-oauth`. The `auth.rs` stub `BearerOutcome` is a local type that does NOT import from `ferro-mcp-oauth`. Adding the dependency (Option A above) is required to share `BearerCheck`. The planner must add this to Wave 0.

### Pitfall 2: `handle_tools_list` signature breaks tests

`handle_tools_list` in `jsonrpc.rs` currently takes `(services, _config)`. Adding a `ctx: &McpContext` parameter is a breaking change that requires updating `jsonrpc_integration.rs` and the consumer app route. Wave 0 must include this update.

### Pitfall 3: `mcp_tenant_isolation.rs` does NOT exist in `ferro-mcp-server/tests/`

The file is at `app/src/tests/mcp_tenant_isolation.rs` and uses the full consumer app models and migrations (Sea ORM entities for tenants, users, orders). The `ferro-mcp-server/tests/` directory contains only `dispatch_integration.rs` and `jsonrpc_integration.rs`, both using a minimal in-memory SQLite fixture. Phase 217's D-09 test must be created from scratch at `ferro-mcp-server/tests/mcp_tenant_isolation.rs` using the simpler pattern (no app model imports) — it seeds a minimal `mcp_api_keys` table and an `orders` table directly in SQLite.

### Pitfall 4: `validate_bearer` is synchronous; `validate_api_key` must be async

`validate_bearer` is a synchronous fn (JWT decode is sync). `validate_api_key` requires an async DB lookup. The unifier `resolve_tenant` in `auth.rs` must be `async fn`. All call sites must `await` it.

### Pitfall 5: Prefix collision — `ferro_` vs `eyJ`

JWTs always begin with `eyJ` (base64url of `{`). The `ferro_` prefix is unambiguous. However, the branch detection must happen before stripping the `Bearer ` prefix is propagated, to avoid double-stripping. The token after `strip_prefix("Bearer ")` is what gets checked with `starts_with("ferro_")`.

### Pitfall 6: `scope` field from JWT principal vs API key principal

The OAuth JWT path (`validate_bearer`) does NOT include a `scope` field in the principal. The API key path (`validate_api_key`) DOES (from the `mcp_api_keys.scope` column). The scope check in `handle_tools_call` must handle the absent `scope` case gracefully. Convention: absent `scope` field = full access (`read_write`), to not break the existing OAuth path which has no scope concept at this layer.

---

## Test Plan

### Wave 0 — Before Implementation

Create `ferro-mcp-server/tests/mcp_tenant_isolation.rs` with the following test structure:

```rust
// Fixture: in-memory SQLite with mcp_api_keys table + orders table
// No consumer app models — only raw SQL inserts
async fn setup_isolation_db() -> sea_orm::DatabaseConnection {
    // CREATE TABLE mcp_api_keys (id, tenant_id, key_hash, scope, revoked_at, ...)
    // CREATE TABLE orders (id, customer_name, total, status, tenant_id)
    // INSERT two tenants' worth of orders
    // INSERT two API keys: tenant_1_key (read), tenant_2_key (read_write)
}
```

### SC#2 — Auth parity: API key and JWT resolve same `tenant_id`

```rust
#[tokio::test]
async fn api_key_and_jwt_produce_same_tenant_id() {
    // Generate a test API key for tenant_id=1, hash it, insert into mcp_api_keys
    // Call validate_api_key("Bearer ferro_<key>", &db, None)
    // Assert BearerCheck::Authenticated(p) where p["tenant_id"] == 1
    // Call validate_bearer with a minted JWT for tenant_id=1
    // Assert BearerCheck::Authenticated(p) where p["tenant_id"] == 1
    // Assert both tenant_ids are equal
}
```

### SC#3 — Scope enforcement: read key rejected on write tool call

```rust
#[tokio::test]
async fn read_scope_key_rejected_on_write_tool() {
    // Build McpContext with scope: Some("read".to_string())
    // Call handle_tools_call with tool_name="create_order" (hypothetical write tool)
    // Assert response["error"]["code"] is set (not a result)
    // The scope-gate rejects before dispatch; no DB query needed for this test
}

#[tokio::test]
async fn read_scope_key_allowed_on_read_tool() {
    // Build McpContext with scope: Some("read".to_string())
    // Call handle_tools_call with tool_name="list_order"
    // Assert response["result"] is present (passes scope gate)
}
```

### SC#4 — Invalid/expired key rejected before tool routing

```rust
#[tokio::test]
async fn invalid_api_key_rejected_before_dispatch() {
    // Call validate_api_key("Bearer ferro_badkey", &db, None)
    // Assert BearerCheck::Invalid (key_hash not in db)
}

#[tokio::test]
async fn revoked_api_key_rejected() {
    // Insert a key with revoked_at set
    // Call validate_api_key with that key
    // Assert BearerCheck::Invalid
}
```

### SC#5 — Cross-tenant isolation: tenant A key does not surface tenant B data

```rust
#[tokio::test]
async fn api_key_cross_tenant_isolation() {
    // Insert tenant_1_key → tenant_id=1
    // Call handle_tools_call("list_order", ..., tenant_id=1)
    // Assert all returned rows have tenant_id == 1
    // Assert no row has tenant_id == 2
}
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (`cargo test`) + `tokio::test` for async |
| Config file | None (Cargo.toml `[dev-dependencies]` in `ferro-mcp-server`) |
| Quick run command | `cargo test -p ferro-mcp-server mcp_tenant_isolation` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AMCP-01 (SC#1) | McpContext carries tenant_id after auth | Unit | `cargo test -p ferro-mcp-server` | ❌ Wave 0 |
| AMCP-01 (SC#1) | tools/call returns only tenant A's data when authenticated as tenant A | Integration | `cargo test -p ferro-mcp-server mcp_tenant_isolation::api_key_cross_tenant_isolation` | ❌ Wave 0 |
| AMCP-02 (SC#2) | API key and JWT produce same BearerCheck::Authenticated(principal) tenant_id | Unit | `cargo test -p ferro-mcp-oauth validate_api_key` | ❌ Wave 0 |
| AMCP-02 (SC#3) | read-scoped key rejected on write tool call | Unit | `cargo test -p ferro-mcp-server mcp_tenant_isolation::read_scope_key_rejected_on_write_tool` | ❌ Wave 0 |
| AMCP-02 (SC#3) | read-scoped key allowed on read tool call | Unit | `cargo test -p ferro-mcp-server mcp_tenant_isolation::read_scope_key_allowed_on_read_tool` | ❌ Wave 0 |
| AMCP-02 (SC#4) | Invalid key rejected before dispatch | Unit | `cargo test -p ferro-mcp-oauth validate::tests::invalid_api_key_rejected` | ❌ Wave 0 |
| AMCP-02 (SC#4) | Revoked key rejected | Unit | `cargo test -p ferro-mcp-oauth validate::tests::revoked_api_key_rejected` | ❌ Wave 0 |
| AMCP-01 (SC#5) | Tenant A API key returns only tenant A data | Integration | `cargo test -p ferro-mcp-server mcp_tenant_isolation::api_key_cross_tenant_isolation` | ❌ Wave 0 |

### Wave 0 Gaps

- [ ] `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — covers SC#2, SC#3, SC#4, SC#5
- [ ] Add `ferry-mcp-oauth` dependency to `ferro-mcp-server/Cargo.toml`
- [ ] `validate_api_key` unit tests in `ferro-mcp-oauth/src/validate.rs`
- [ ] `generate_mcp_api_key` function + unit tests in `ferro-mcp-oauth`

---

## Contradictions Between CONTEXT.md, ARCHITECTURE.md, and Actual Code

### Contradiction 1: Validator placement (resolved in CONTEXT.md D-01)

**ARCHITECTURE.md §"Build order Phase 1"** (line 363-369): Describes `resolve_tenant_from_bearer()` and `resolve_tenant_from_api_key(raw_key_prefix, db)` both living in `ferro-mcp-server/src/auth.rs`.

**CONTEXT.md D-01**: `validate_api_key` lives in `ferro-mcp-oauth/src/validate.rs`; `auth.rs` is a thin unifier.

**Code reality**: `auth.rs` is a 10-line stub. `validate_bearer` is in `ferro-mcp-oauth/src/validate.rs`. `ferro-mcp-server` has NO dependency on `ferro-mcp-oauth`.

**Resolution**: CONTEXT.md D-01 is correct and supersedes ARCHITECTURE.md. The validator goes in `ferro-mcp-oauth`. The consequence: add `ferro-mcp-oauth` as a `ferro-mcp-server` dependency. The planner must include this in Wave 0.

### Contradiction 2: `mcp_tenant_isolation.rs` location

**CONTEXT.md D-09**: "Extend the existing non-ignored integration test `ferro-mcp-server/tests/mcp_tenant_isolation.rs`."

**Code reality**: No such file exists in `ferro-mcp-server/tests/`. The existing isolation test is at `app/src/tests/mcp_tenant_isolation.rs` (consumer app).

**Resolution**: Create `ferro-mcp-server/tests/mcp_tenant_isolation.rs` from scratch using the simple in-process SQLite fixture pattern. It cannot use the app's `Migrator` or `crate::models` — it must use raw SQL `CREATE TABLE` + `INSERT` statements like `dispatch_integration.rs` does.

### Contradiction 3: STACK.md §(b) "reuse whatever v8.1 `make:api-key` generates"

**STACK.md**: "The `api_keys` schema should reuse whatever the v8.1 `ferro make:api-key` command generates."

**Code reality**: The v8.1 `make:api-key` generates a schema with `(name, prefix, hashed_key, created_at)` — no `tenant_id`, no `scope`. This schema cannot be reused for MCP per-tenant API keys.

**Resolution**: CONTEXT.md D-05 is the authority. The existing general `api_keys` table (for REST API key middleware) is a different concern. Phase 217 creates a NEW `mcp_api_keys` table in `ferro-mcp-oauth`'s migration with the MCP-specific schema. The existing `framework/src/api/api_key.rs` `hash_api_key()` and `generate_api_key()` pattern IS reused (the hashing logic is identical), but the table is new.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 217 is code-only changes in the ferro workspace. All required tools (Rust, cargo, SQLite via sea-orm sqlx feature) are confirmed available by the existing test suite running in CI.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | JWTs always begin with `eyJ`, making `starts_with("ferro_")` an unambiguous branch discriminator | Branch detection, auth.rs | If a non-JWT bearer token exists in the system that also starts with `ferro_` or a JWT that doesn't start with `eyJ`, the branch would misroute — LOW risk given standard JWT encoding |
| A2 | Adding `ferro-mcp-oauth` as a `ferro-mcp-server` dependency creates no circular dependency | Dependency Gap section | If `ferro-mcp-server` were already imported by `ferro-mcp-oauth` there would be a cycle — VERIFIED: `ferro-mcp-oauth/Cargo.toml` lists `ferro` (framework) but not `ferro-mcp-server` |
| A3 | `scope` absent from principal (OAuth path) = `read_write` (full access) | Scope enforcement section | If the planner decides absent scope = deny, existing OAuth JWT users would lose access — this must be decided before implementation |

---

## Open Questions (RESOLVED)

1. **McpContext.scope field:** Should `McpContext` carry `scope: Option<String>` directly (populated from principal), or should `handle_tools_call` extract scope from the raw principal value? Adding it to `McpContext` is consistent with D-07 and avoids passing the full principal JSON through the call chain.
   **RESOLVED: `scope: Option<String>` added to `McpContext` (Plan 00 Task 1).**

2. **`mcp_api_keys` vs `api_keys` table name:** The existing general REST key table is `api_keys`. The new MCP key table should be `mcp_api_keys` to avoid collision, OR the Phase 217 migration can extend the general `api_keys` table by adding `tenant_id` and `scope` columns. The latter is NOT recommended because the general REST key table is app-managed (consumer's schema), not framework-managed. Using `mcp_api_keys` is cleaner.
   **RESOLVED: using the `mcp_api_keys` table throughout (Plan 01 Task 1).**

3. **`sub` field in API key principal:** The OAuth JWT `sub` is a user ID. For API keys, there is no user — the key IS the credential. Using `row.id.to_string()` as `sub` is a reasonable convention. Downstream code (`app/src/tests/mcp_tenant_isolation.rs` line 238) uses `principal["sub"]` for user identification — ensure the API key `sub` does not accidentally resolve to a user via `JwtClaimResolver` in the middleware chain.
   **RESOLVED: `row.id.to_string()` used as `sub` (Plan 01 Task 2).**

---

## Sources

### Primary (HIGH confidence — direct source file reads)

- `ferro-mcp-oauth/src/validate.rs` — `BearerCheck` enum (lines 35-44), `validate_bearer` signature + implementation (lines 53-98), principal shape (lines 94-97)
- `ferro-mcp-oauth/src/jwt.rs` — `McpTokenClaims` struct (lines 22-37), `build_claims` / `mint_token` / `decode_token`
- `ferro-mcp-oauth/Cargo.toml` — confirmed `sha2 = "0.10"`, `subtle = "2.5"`, `rand = "0.8"`, `sea-orm = "1.0"` all present
- `ferro-mcp-server/src/renderer.rs` — `McpContext` stub (lines 9-10), `McpRenderer` + `render_exposed_tools`
- `ferro-mcp-server/src/auth.rs` — `BearerOutcome` stub (lines 1-10)
- `ferro-mcp-server/src/dispatch.rs` — `dispatch()` signature (lines 108-115), fail-closed guarantee (lines 152-166), `tenant_id` parameter confirmed
- `ferro-mcp-server/src/error.rs` — current `Error` enum (4 variants, no `Auth` variant)
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_list` (line 33), `handle_tools_call` (lines 49-54), `tenant_id` threading (line 84)
- `ferro-mcp-server/src/lib.rs` — public API exports; NO `ferro-mcp-oauth` in imports
- `ferro-mcp-server/Cargo.toml` — confirmed NO `ferro-mcp-oauth` dependency
- `ferro-mcp-server/tests/` — confirmed only `dispatch_integration.rs` and `jsonrpc_integration.rs` exist; NO `mcp_tenant_isolation.rs`
- `framework/src/api/api_key.rs` — existing general REST API key infrastructure: `fe_live_`/`fe_test_` prefix, `generate_api_key()`, `hash_api_key()`, `verify_api_key_hash()`, `ApiKeyMiddleware`; schema `(name, prefix, hashed_key, created_at)` — NO `tenant_id`, NO `scope`
- `ferro-cli/src/commands/make_api_key.rs` — confirmed SQL output: `INSERT INTO api_keys (name, prefix, hashed_key, created_at)` (line 83-84); no tenant_id, no scope
- `app/src/tests/mcp_tenant_isolation.rs` — existing consumer-app tenant isolation tests; uses app `Migrator` and model entities; confirmed NOT in `ferro-mcp-server/tests/`
- `ferro-mcp-server/tests/common/mod.rs` — `setup_db()` pattern for new test file reference

### Secondary (MEDIUM confidence)

- `.planning/research/ARCHITECTURE.md` — system architecture, build order, component boundaries
- `.planning/research/STACK.md` — dependency confirmation, validator placement recommendation
- `.planning/research/PITFALLS.md` — §1 cross-tenant leak, §2 guard bypass, §4 scope creep
- `.planning/research/FEATURES.md` — per-tenant API-key auth and anti-patterns

---

## Metadata

**Research date:** 2026-06-13
**Valid until:** 2026-07-13 (stable Rust + sea-orm + rmcp, 30-day window)

**Confidence breakdown:**
- Code touchpoints (current state): HIGH — all files read directly
- api_keys schema question: HIGH — confirmed absence of MCP-specific table
- dependency gap (ferro-mcp-server ↔ ferro-mcp-oauth): HIGH — grep confirmed no dependency
- mcp_tenant_isolation.rs gap: HIGH — `ls` confirmed file does not exist in tests/
- Validate_api_key implementation sketch: MEDIUM — pattern is clear, exact SeaORM query syntax may need adjustment for entity vs raw SQL choice

---

## RESEARCH COMPLETE

**Phase:** 217 - Tenant Context + Per-Tenant API-Key Auth
**Confidence:** HIGH

### Key Findings

1. **`api_keys` schema gap — define new.** The existing `make:api-key` CLI and `framework/src/api/api_key.rs` generate a general REST API key schema `(name, prefix, hashed_key, created_at)` with no `tenant_id` and no `scope`. Phase 217 must create a new `mcp_api_keys` migration in `ferro-mcp-oauth` with the canonical MCP schema. The hashing primitives (`sha2`, `subtle`, `rand`) from the existing system are directly reusable.

2. **`ferro-mcp-server` has no dependency on `ferro-mcp-oauth`.** D-01 (validator in `ferro-mcp-oauth`) requires adding `ferro-mcp-oauth` as a `ferro-mcp-server` Cargo.toml dependency. This is one Cargo.toml line with no circular dependency risk.

3. **`mcp_tenant_isolation.rs` must be created from scratch** at `ferro-mcp-server/tests/mcp_tenant_isolation.rs`. The existing consumer-app file (`app/src/tests/mcp_tenant_isolation.rs`) uses the full app migration stack and model entities — not importable from `ferro-mcp-server`. The new file uses the minimal in-process SQLite pattern from `dispatch_integration.rs`.

4. **`validate_api_key` is async; `validate_bearer` is sync.** The unifier `resolve_tenant` in `auth.rs` must be `async fn`. The planner must update all call sites.

5. **`handle_tools_list` signature must change** to accept `ctx: &McpContext` (or `tenant_id`) to carry the resolved tenant_id into `render_exposed_tools`. This breaks the existing `jsonrpc_integration.rs` test which passes `&McpContext` (zero-value). Wave 0 must include the test update.

6. **Scope enforcement (SC#3) is wired now, tested against empty write-tool set.** The scope-rejection gate in `handle_tools_call` (`!tool_name.starts_with("list_") && scope == "read" → reject`) can be tested with a synthetic write-tool name in a unit test. No real write tool is needed for Phase 217's scope gate to be correct and tested.

### Contradictions Resolved

| Contradiction | Resolution |
|---------------|------------|
| ARCHITECTURE.md puts API-key lookup in `ferro-mcp-server`; CONTEXT.md D-01 puts it in `ferro-mcp-oauth` | CONTEXT.md D-01 wins; consequence: add crate dependency |
| CONTEXT.md D-09 says extend "existing" `ferro-mcp-server/tests/mcp_tenant_isolation.rs` | File does not exist; create new |
| STACK.md says reuse v8.1 `make:api-key` schema | Schema incompatible (no tenant_id, no scope); define new `mcp_api_keys` table |

### Ready for Planning

Research complete. Planner can now create PLAN.md files.
