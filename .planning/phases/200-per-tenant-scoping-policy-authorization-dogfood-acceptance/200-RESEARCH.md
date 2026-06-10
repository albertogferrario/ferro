# Phase 200: Per-Tenant Scoping, Policy Authorization & Dogfood Acceptance — Research

**Researched:** 2026-06-10
**Domain:** Rust / Ferro framework — tenant scoping, gate-based authorization, SQL predicate injection, MCP tool-error shaping, multi-tenant fixture bootstrapping
**Confidence:** HIGH (all load-bearing claims verified against source files in this session)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 — Tenant context establishment for `/mcp`:**
Mount `[BearerAuthMiddleware, TenantMiddleware(JwtClaimResolver("tenant_id", lookup))]` in that order on the `/mcp` route. Bearer middleware inserts `serde_json::Value` claims into request extensions; `TenantMiddleware` reads them to populate `current_tenant()`.

**D-02 — Tenant predicate injection in dispatch:**
`dispatch` appends `AND "{tenant_col}" = ?` (bound parameter, never string-interpolated, never from call payload) to both COUNT and SELECT WHERE clauses when the projection is tenant-scoped and `current_tenant()` is `Some`. Column name comes from `ServiceDef.tenant_column: Option<String>`. Value comes from `current_tenant().id`.

**D-03 — Tenant claim name alignment:**
JWT claim is `tenant_id` (integer). Must match `JwtClaimResolver::new("tenant_id", …)`.

**D-04 — Policy gating mechanism:**
`ServiceDef.mcp_ability: Option<String>` holds the ability name. App `/mcp` handler loads the concrete `User` from `sub`, calls `Gate::authorize_for(&user, ability, None)`. Deny → MCP tool error (D-09). If `mcp_ability` is `None` on an `mcp_exposed` projection → deny (fail-closed).

**D-05 — Division of responsibility:**
Framework (`ferro-projections` `ServiceDef`): `tenant_column`, `mcp_ability` as plain metadata.
Framework (`ferro-mcp-server` `dispatch`): reads `current_tenant()`, injects predicate.
App (`/mcp` handler glue): loads concrete `User`, calls `Gate::authorize_for`, maps deny to tool error.

**D-06 — Fail-closed on missing tenant:**
Tenant-scoped projection + `current_tenant() == None` → deny / return zero rows. Never unscoped SELECT.

**D-07 — Dogfood multi-tenant fixture:**
Add `tenants` table, `orders` table with `tenant_id`, User→tenant FK, `TenantMiddleware` on `/authorize` and `/mcp`, seed 2 tenants + orders + users, wire `order` projection with `tenant_column`/`mcp_ability`.

**D-08 — Dogfood harness:**
Scripted MCP client (Node or Python MCP SDK) + Claude Desktop config. User starts app, performs browser login manually. Result recorded as `200-ACCEPTANCE.md`.

**D-09 — Policy-deny tool-error shape:**
JSON-RPC success envelope, `isError: true` in result content, human-readable message, no rows/columns/filter values leaked.

### Claude's Discretion

- Exact name of the seeded gate ability and the two seed tenants/orders fixtures.
- Internal module placement of the bearer-auth middleware (D-01 ordering).
- Wording of the policy-deny tool-error message (must disclose nothing about data).
- Whether `tenant_column`/`mcp_ability` are separate `Option<String>` fields or a small `McpAccess` sub-struct on `ServiceDef` (keep plain metadata either way).
- Language/runtime of the scripted dogfood client (Python MCP SDK vs Node vs `mcp` CLI).

### Deferred Ideas (OUT OF SCOPE)

- Write intents over MCP (create/submit tools with confirmation).
- Per-tenant tool catalog variation (tenants seeing different tool *sets*).
- Typed `Policy<M>` dispatch for per-row authorization.
- Generalized tenant-FK derivation (auto-detecting tenant column from model metadata).
- `ServiceDef.table` override for irregular plurals — only if `orders` fixture surfaces a mismatch.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AMCP-10 | Tool call executes within token's tenant context via existing multi-tenant middleware; token scoped to one tenant returns only that tenant's rows. | D-01 ordering verified. D-02 predicate injection pattern confirmed. D-03 claim name verified as `tenant_id` integer. D-07 fixture scope defined. |
| AMCP-11 | Tool call gated by same policy layer as web surface; policy-denied call returns MCP tool error with no data disclosure. | D-04 Gate::authorize_for pattern verified. D-09 tool-error shape confirmed against MCP spec. Gate.rs inspected. |
</phase_requirements>

---

## Summary

Phase 200 is a security-critical seam-filling phase. Phases 197–199 built a walking skeleton: projection → tool schema → HTTP transport → OAuth token bound to `(user, tenant)`. Phase 200 makes that skeleton *secure*: a `tools/call` now executes inside the token's tenant context and is gated by the application's existing policy layer. The dogfood gate (a real MCP client, browser login, live app) validates end to end.

All four primary research threads — claim-name reconciliation (D-03), middleware ordering (D-01), dispatch predicate injection (D-02), and policy gating (D-04) — have been resolved by direct code inspection. The critical finding is that **Phase 199 already completed D-03 correctly**: the minted JWT uses `tenant_id: Option<i64>` as the claim name and type, matching `JwtClaimResolver`'s expectation exactly (`claims["tenant_id"].as_i64()`). No reconciliation needed.

The main engineering work in this phase is: (1) promoting Phase 199's inline bearer validation into a `BearerAuthMiddleware` so it runs before `TenantMiddleware`; (2) adding `tenant_column` and `mcp_ability` fields to `ServiceDef`; (3) injecting the tenant predicate in `dispatch`; (4) adding the Gate check in the `/mcp` handler before calling `handle_tools_call`; (5) building the minimal two-tenant fixture in the sample `app`; and (6) executing the dogfood run.

**Primary recommendation:** Treat the middleware ordering (D-01) as the structural spine. Get `[BearerAuthMiddleware → TenantMiddleware]` right first; everything else (dispatch predicate, gate check) reads from the resulting `current_tenant()` and loaded `User` — both of which become available through that chain.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Bearer JWT validation | App middleware (`BearerAuthMiddleware`) | `ferro-mcp-oauth` validator | App wires it; framework crate does the crypto |
| Tenant context scoping | Framework middleware (`TenantMiddleware` + `JwtClaimResolver`) | — | Structural identity requirement (SC-3) |
| Tenant predicate injection | `ferro-mcp-server` `dispatch` | `current_tenant()` in framework | Dispatch owns SQL; reads context set by middleware |
| Policy gate check | App `/mcp` handler glue | `framework` `Gate::authorize_for` | Gate is framework; concrete `User` load is app-specific |
| MCP tool-error shaping | App `/mcp` handler glue | `ferro-mcp-server` `handle_tools_call` error path | Handler owns the JSON-RPC envelope splice |
| Multi-tenant fixture | Sample `app` migrations/seeds | Framework `TenantMiddleware` wiring | Fixture data is app-scoped; middleware is framework |
| Dogfood acceptance | Manual + scripted client | App running locally | Browser login is human-in-the-loop by convention |

---

## Standard Stack

### Core (all already in workspace — no new dependencies)

| Crate | Already Used | Role in Phase 200 |
|-------|-------------|-------------------|
| `ferro-mcp-oauth` | Phase 199 | `validate_bearer` → `BearerCheck::Authenticated(principal)` |
| `framework` `TenantMiddleware` + `JwtClaimResolver` | Phase 95 | Establishes `current_tenant()` on `/mcp` route |
| `framework` `Gate::authorize_for` | Phases 72–74 | Policy gate check with concrete `User` |
| `ferro-projections` `ServiceDef` | Phase 197 | Gains `tenant_column`/`mcp_ability` plain-metadata fields |
| `ferro-mcp-server` `dispatch` | Phase 197 | Gains tenant predicate injection |
| `sea-orm` `Statement::from_sql_and_values` | Phase 197 | Bound-parameter path for tenant predicate |

### New Artifacts This Phase

| Artifact | Location | Purpose |
|----------|----------|---------|
| `BearerAuthMiddleware` | `app/src/middleware/bearer_auth.rs` (or inline in mcp controller) | Runs before `TenantMiddleware`; parses JWT, inserts `serde_json::Value` claims |
| `ServiceDef.tenant_column: Option<String>` | `ferro-projections/src/service.rs` | Declares tenant FK column name |
| `ServiceDef.mcp_ability: Option<String>` | `ferro-projections/src/service.rs` | Declares required Gate ability |
| `tenants` migration | `app/src/migrations/m20260611_create_tenants_table.rs` | Tenant table for dogfood fixture |
| `orders` migration | `app/src/migrations/m20260611_create_orders_table.rs` | Orders table with `tenant_id` FK |
| `200-ACCEPTANCE.md` | `.planning/phases/200-.../` | GO/NO-GO record |

---

## Architecture Patterns

### System Architecture Diagram

```
HTTP POST /mcp
      │
      ▼
BearerAuthMiddleware
  ├── validate_bearer(header, oauth_config, expected_tenant=None)
  ├── BearerCheck::Unauthenticated → 401 challenge (short-circuit)
  ├── BearerCheck::Invalid         → 401 invalid_token (short-circuit)
  ├── BearerCheck::Forbidden       → 403 (short-circuit)
  └── BearerCheck::Authenticated(principal)
        │ req.insert::<serde_json::Value>(principal)
        ▼
TenantMiddleware(JwtClaimResolver("tenant_id", db_lookup))
  ├── JwtClaimResolver::resolve() reads req.get::<serde_json::Value>()
  ├── extracts principal["tenant_id"].as_i64() → tenant_id
  ├── TenantLookup::find_by_id(tenant_id) → TenantContext
  ├── None resolved + on_failure=Forbidden → 403 (short-circuit)
  └── with_tenant_scope(ctx, ...) sets current_tenant()
        │
        ▼
mcp::handle() handler
  1. Parse JSON-RPC body
  2. On "tools/call":
     a. extract sub from principal (stored earlier via req.get or from outer scope)
     b. DB load User::find_by_id(sub)
     c. lookup service.mcp_ability
        ├── None → return policy-deny tool error (fail-closed)
        └── Some(ability):
              Gate::authorize_for(&user, ability, None)
              ├── Denied  → return policy-deny tool error (isError:true)
              └── Allowed → dispatch(service, filters, limit, offset, db)
                              reads current_tenant().id
                              appends AND "tenant_id" = ? to WHERE
                              returns DispatchResult
  3. Return JSON-RPC success envelope
```

### Recommended Project Structure Changes

```
ferro-projections/src/
  service.rs           # +tenant_column: Option<String>, +mcp_ability: Option<String>

ferro-mcp-server/src/
  dispatch.rs          # +tenant predicate injection (reads current_tenant() or param)

app/src/
  middleware/
    bearer_auth.rs     # NEW: BearerAuthMiddleware (promotes Phase 199 inline logic)
  migrations/
    m20260611_create_tenants_table.rs   # NEW
    m20260611_create_orders_table.rs    # NEW (with tenant_id FK, matches order projection fields)
    mod.rs             # +register two new migrations
  models/
    tenants.rs         # NEW: Tenant entity model
    orders.rs          # NEW: Order entity model (id, customer_name, total, status, created_at, tenant_id)
    mod.rs             # +re-export
  projections/
    order.rs           # +tenant_column("tenant_id") +mcp_ability("view-orders")
  routes.rs            # +TenantMiddleware on /authorize and /mcp
  bootstrap/           # +Gate::define("view-orders", ...) +seed two tenants and users

dogfood/
  run_dogfood.{ts|py}  # NEW: scripted MCP client for acceptance gate
```

### Pattern 1: BearerAuthMiddleware inserting claims into request extensions

The middleware must be `mut` over the request to call `req.insert()`. The `Middleware` trait signature is `handle(&self, request: Request, next: Next) -> Response` — `Request` is passed by value, not by reference. This means the middleware can mutate the owned `Request` before passing it to `next`.

```rust
// Source: verified from framework/src/middleware/mod.rs + framework/src/http/request.rs
#[async_trait]
impl Middleware for BearerAuthMiddleware {
    async fn handle(&self, mut request: Request, next: Next) -> Response {
        let auth_header = request.header("Authorization").map(|s| s.to_owned());
        let oauth_config = OAuthConfig::from_env()
            .map_err(|_| challenge_response(&self.mcp_config))?;

        match validate_bearer(auth_header.as_deref(), &oauth_config, None) {
            BearerCheck::Unauthenticated => Err(challenge_response(&self.mcp_config)),
            BearerCheck::Invalid => Err(HttpResponse::new()
                .status(401)
                .header("WWW-Authenticate", "Bearer error=\"invalid_token\"")),
            BearerCheck::Forbidden => Err(HttpResponse::new().status(403)),
            BearerCheck::Authenticated(principal) => {
                // Insert claims so JwtClaimResolver can read them
                request.insert::<serde_json::Value>(principal);
                next(request).await
            }
        }
    }
}
```

**Key: `expected_tenant` in the bearer middleware call is `None`** — tenant validation at bearer-validation time re-checks what `TenantMiddleware` will confirm. Since the middleware runs before `TenantMiddleware`, the bearer middleware cannot yet know which tenant `TenantMiddleware` will resolve (it resolves from the claims it inserts). The tenant check at bearer-validation time should stay `None` or be dropped in the middleware; `TenantMiddleware` with `on_failure(TenantFailureMode::Forbidden)` handles the case where `tenant_id` in the claims doesn't match any real tenant. [VERIFIED: ferro-mcp-oauth/src/validate.rs — `expected_tenant: None` skips tenant check]

### Pattern 2: JwtClaimResolver reading the inserted principal

```rust
// Source: verified from framework/src/tenant/resolver.rs lines 209-211
async fn resolve(&self, req: &Request) -> Option<TenantContext> {
    let claims = req.get::<serde_json::Value>()?;
    let id = claims[&self.claim_field].as_i64()?;   // reads "tenant_id" as i64
    self.tenant_lookup.find_by_id(id).await
}
```

The `BearerAuthMiddleware` inserts `json!({"sub": ..., "tenant_id": ...})` (the `principal` from `BearerCheck::Authenticated`). The resolver reads `claims["tenant_id"].as_i64()` — this works directly because `McpTokenClaims.tenant_id` is `Option<i64>` and the serialized JSON uses the key `tenant_id`. [VERIFIED: ferro-mcp-oauth/src/jwt.rs line 27 — `pub tenant_id: Option<i64>`, key is `tenant_id`]

### Pattern 3: Tenant predicate injection in dispatch

The current `dispatch` signature is:
```rust
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
) -> crate::Result<DispatchResult>
```

`ferro-mcp-server` does NOT depend on `framework` (Cargo.toml verified — deps are only `ferro-projections`, `rmcp`, `serde`, `schemars`, `thiserror`, `tracing`, `sea-orm`). Therefore `dispatch` **cannot call `current_tenant()` directly**. The tenant id must be passed as a parameter.

**Recommended signature extension:**

```rust
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,   // NEW: passed by app handler from current_tenant()
) -> crate::Result<DispatchResult>
```

The app handler passes `ferro::current_tenant().map(|t| t.id)` — the same call already present at line 52 of `app/src/controllers/mcp.rs`. Dispatch reads `service.tenant_column` and appends the predicate when `Some(col)` and `Some(id)` are both present. When `tenant_column = Some(col)` but `tenant_id = None` → fail-closed (return error or zero rows).

Predicate injection at the WHERE-clause build site (immediately after the filter loop):

```rust
// Tenant predicate — injected AFTER user filters, BEFORE count/data queries
if let Some(ref col) = service.tenant_column {
    match tenant_id {
        Some(tid) => {
            where_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
            values.push(sea_orm::Value::BigInt(Some(tid)));
            idx += 1;
        }
        None => {
            // Fail-closed: tenant-scoped projection but no context → deny
            return Err(crate::Error::InvalidFilter(
                "tenant context required but not present".to_string()
            ));
        }
    }
}
```

This appends to BOTH the count and data query because both use the same `where_clauses` / `values` at the same point. [VERIFIED: ferro-mcp-server/src/dispatch.rs lines 118-152 — single where_clauses/values build, used by both count_sql and data_sql]

### Pattern 4: Gate check in the app handler (policy gating)

`Gate::authorize_for<U: Authenticatable>(user: &U, ability: &str, ...)` takes a concrete user and is synchronous. [VERIFIED: framework/src/authorization/gate.rs lines 172-189]

The app handler needs the concrete `User` model. The `sub` claim in the principal is a `String` of the user ID integer. Load order:

```rust
// In mcp::handle, after TenantMiddleware has run (current_tenant() is set):
// Principal was inserted by BearerAuthMiddleware — retrieve it
let principal = req.get::<serde_json::Value>()
    .ok_or_else(|| /* 401 */)?;
let user_id: i64 = principal["sub"].as_str()
    .and_then(|s| s.parse().ok())
    .ok_or_else(|| /* 400 */)?;

// On "tools/call" arm:
let user = crate::models::User::find_by_id(user_id)
    .await
    .map_err(|_| /* 500 */)?
    .ok_or_else(|| /* 401 */)?;

let ability = service.mcp_ability.as_deref()
    .ok_or_else(|| /* policy-deny tool error */)?;   // None = fail-closed

match Gate::authorize_for(&user, ability, None) {
    Ok(()) => { /* proceed to dispatch */ }
    Err(err) => { /* return policy-deny tool error */ }
}
```

**Critical note on `req.get::<serde_json::Value>()`:** After `TenantMiddleware` calls `next(request)`, the handler receives the `request` that had the claims inserted by `BearerAuthMiddleware`. Since `Request` is passed by value through the chain and `insert`/`get` use a `HashMap<TypeId, Box<dyn Any>>`, the inserted value survives through the middleware chain into the handler. [VERIFIED: framework/src/http/request.rs lines 87-93 — `insert` and `get` on owned extensions map]

However: `TenantMiddleware::handle` calls `next(request)` with the request it received — the same request that had `serde_json::Value` inserted. The handler can call `req.get::<serde_json::Value>()` directly. [VERIFIED: framework/src/tenant/middleware.rs line 87 — `next(request)` passes request through unchanged]

### Pattern 5: Policy-deny tool-error shape (D-09)

MCP tool errors use a JSON-RPC **success envelope** with `isError: true` in the result. This distinguishes from Phase 199's transport-level 401/403. Looking at `handle_tools_call` return patterns:

```rust
// MCP tool error — success envelope, isError in result
// No rows, no column data, no filter values disclosed
json!({
    "result": {
        "content": [
            {
                "type": "text",
                "text": "Access denied. You do not have permission to view this resource."
            }
        ],
        "isError": true
    }
})
```

This is then spliced with `"jsonrpc": "2.0"` and `"id"` by the handler's envelope code (mcp.rs lines 94-98). [VERIFIED: app/src/controllers/mcp.rs lines 79-98 — splice logic adds jsonrpc+id to the payload object]

### Anti-Patterns to Avoid

- **Calling `current_tenant()` inside `ferro-mcp-server`:** `ferro-mcp-server` has no `framework` dependency. Any attempt to add one for this purpose re-introduces a coupling the architecture explicitly avoided. Pass `tenant_id: Option<i64>` as a parameter.
- **Using `Gate::allows`/`Gate::authorize` (sync, session-based):** These read `Auth::id()` from a session context that is not set in the MCP path. Use `Gate::authorize_for(&user, ...)` which takes an explicit user. [VERIFIED: gate.rs lines 130-161 — `allows`/`authorize` check `Auth::id()` which is session-based]
- **Forgetting to handle `mcp_ability = None` as a deny:** The `order` projection currently has no `mcp_ability` field. If the check is `if let Some(ability) = service.mcp_ability { ... } else { /* proceed */ }`, all calls would be allowed by default — the wrong default. Always deny when no ability is declared on an `mcp_exposed` projection.
- **Setting `expected_tenant` to a non-None value in the bearer middleware:** At bearer-auth middleware time, `current_tenant()` is not yet set (that happens in `TenantMiddleware`). The bearer middleware does not know the tenant to check against. Pass `None`; let `TenantMiddleware` handle the tenant resolution.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JWT validation + claim extraction | Custom decode loop | `ferro_mcp_oauth::validate_bearer` + `decode_token` | Already covers HS256 pin, audience check, expiry, constant-time |
| Tenant context from claims | Custom task-local | `JwtClaimResolver` + `TenantMiddleware` | Exactly what these are for; SC-3 requires structural identity |
| Tenant scoping SQL | Custom `WHERE tenant_id = $1` builder | Extend existing `Statement::from_sql_and_values` loop | All bound-parameter safety already in dispatch |
| Gate/policy check | MCP-specific permission check | `Gate::authorize_for` from `framework::authorization` | One permission system — the design invariant |
| MCP tool-error serialization | Custom error struct | `json!({"result": {"content": [...], "isError": true}})` | MCP spec is JSON-native; no separate type needed |

---

## Runtime State Inventory

This is a greenfield feature phase (adding new migrations, not renaming existing state). No runtime state inventory required.

None — verified: no rename/refactor/migration of existing identifiers.

---

## Common Pitfalls

### Pitfall 1: `expected_tenant` in bearer middleware breaks the ordering
**What goes wrong:** If `BearerAuthMiddleware` calls `validate_bearer(header, config, ferro::current_tenant().map(|t| t.id))`, `current_tenant()` is `None` at that point (middleware runs before `TenantMiddleware`). All multi-tenant tokens are rejected with 403.
**Why it happens:** Phase 199's inline handler code already has `let expected_tenant = ferro::current_tenant().map(|t| t.id)` — this works inline because it runs after `TenantMiddleware` on a different route. When relocated to a middleware that runs *before* `TenantMiddleware`, it captures `None`.
**How to avoid:** Pass `None` as `expected_tenant` in the bearer middleware. The tenant validation is `TenantMiddleware`'s job. [VERIFIED: mcp.rs line 52 — the existing code has this comment "None for single-tenant /mcp (Phase 200 will supply tenant context)"]
**Warning signs:** Every multi-tenant token gets 403 at bearer validation; single-tenant (`tenant_id = None`) tokens pass.

### Pitfall 2: `req.get::<serde_json::Value>()` retrieves the wrong type
**What goes wrong:** `BearerAuthMiddleware` inserts `principal` (a `serde_json::Value`). If the handler calls `req.get::<McpTokenClaims>()` or `req.get::<serde_json::Map<...>>()`, it gets `None` because the TypeId doesn't match `serde_json::Value`.
**Why it happens:** The extensions HashMap keys on `TypeId` — the exact type must match.
**How to avoid:** Always insert and retrieve as `serde_json::Value`. [VERIFIED: resolver.rs line 210 — `req.get::<serde_json::Value>()`]

### Pitfall 3: `dispatch` signature change breaks `handle_tools_call` call site
**What goes wrong:** Adding `tenant_id: Option<i64>` to `dispatch` breaks the existing call in `jsonrpc.rs` line 82 and the unit tests in `dispatch.rs` (which don't pass a tenant).
**Why it happens:** Mechanical — function signature change.
**How to avoid:** Update all call sites simultaneously. For tests, pass `None` for non-tenant scenarios.

### Pitfall 4: `orders` table field mismatch with the `order` projection
**What goes wrong:** The `order` projection declares fields `[id, customer_name, total, status, created_at]`. If the migration creates columns named differently (e.g., `name` instead of `customer_name`, `amount` instead of `total`), the `SELECT *` returns columns the rows-to-json mapper will serialize with DB column names, not the projection field names — the tool output and schema don't match.
**Why it happens:** The projection field names are the authoritative names; the migration must match exactly.
**How to avoid:** Derive the orders migration columns verbatim from the projection's declared field names: `id INTEGER PK`, `customer_name TEXT NOT NULL`, `total REAL NOT NULL`, `status TEXT NOT NULL`, `created_at TIMESTAMP NOT NULL`, `tenant_id INTEGER NOT NULL FK(tenants.id)`. [VERIFIED: app/src/projections/order.rs lines 13-18]

### Pitfall 5: Table name derivation for `orders`
**What goes wrong:** `dispatch` derives table name as `format!("{}s", service.name.to_lowercase())` where `service.name = "order"` → `"orders"`. If the migration names the table `order` or `Order`, the query fails.
**Why it happens:** The pluralization is naive — it always appends "s". "order" → "orders" works correctly.
**How to avoid:** Name the migration table `orders` explicitly. [VERIFIED: ferro-mcp-server/src/dispatch.rs line 116]

### Pitfall 6: `TenantMiddleware` failure mode on `/mcp`
**What goes wrong:** If `TenantMiddleware` is mounted with the default `TenantFailureMode::NotFound` (404), a token with a valid but unknown `tenant_id` returns 404 instead of 403.
**Why it happens:** Default failure mode is `NotFound`.
**How to avoid:** Mount `TenantMiddleware::new().resolver(...).on_failure(TenantFailureMode::Forbidden)` on the `/mcp` route. A missing tenant on an authenticated request is a 403 scenario, not 404. [VERIFIED: framework/src/tenant/middleware.rs lines 87-100]

### Pitfall 7: Gate::authorize vs Gate::authorize_for
**What goes wrong:** Calling `Gate::authorize("view-orders", None)` in the MCP handler returns `Err` with status 401 ("no authenticated user") because `Auth::id()` checks the session, which is not set in the MCP request path.
**Why it happens:** `Gate::authorize` uses `crate::auth::Auth::id()` which reads session-based auth state.
**How to avoid:** Use `Gate::authorize_for(&user, "view-orders", None)` with the user loaded from `sub`. [VERIFIED: gate.rs lines 151-161 vs 172-189]

### Pitfall 8: Dogfood token has `tenant_id = null` (Phase 199 neutralized tenant)
**What goes wrong:** The dogfood run produces a token with `tenant_id = null` because `/authorize` in Phase 199 had no `TenantMiddleware`, so `current_tenant()` was `None` at authorize time, and `build_claims(user_id, None, ...)` was called.
**Why it happens:** D-07 explicitly notes this caveat — Phase 199 `/authorize` had no `TenantMiddleware`.
**How to avoid:** Wire `TenantMiddleware(JwtClaimResolver("tenant_id", lookup))` onto `/authorize` as part of D-07. The OAuth `authorize_get` and `authorize_post` handlers call `framework::tenant::current_tenant()` to capture the tenant — if middleware is not mounted, they get `None`. [VERIFIED: 200-CONTEXT.md D-07 caveat paragraph]

---

## Code Examples

### D-03 VERIFIED: Exact claim structure minted by Phase 199

```rust
// Source: ferro-mcp-oauth/src/jwt.rs — verified in this session
pub struct McpTokenClaims {
    pub sub: String,               // user ID as string ("42")
    pub tenant_id: Option<i64>,    // EXACT name — matches JwtClaimResolver
    pub aud: Vec<String>,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
}

// BearerCheck::Authenticated carries:
// json!({ "sub": claims.sub, "tenant_id": claims.tenant_id })
// → serde_json::Value with key "tenant_id" as Number|Null
```

### D-03 VERIFIED: JwtClaimResolver reads exactly this

```rust
// Source: framework/src/tenant/resolver.rs lines 209-211
async fn resolve(&self, req: &Request) -> Option<TenantContext> {
    let claims = req.get::<serde_json::Value>()?;
    let id = claims[&self.claim_field].as_i64()?;  // "tenant_id" as i64
    self.tenant_lookup.find_by_id(id).await
}
```

**Reconciliation result: NO CHANGES NEEDED.** Phase 199 already minted `tenant_id: Option<i64>`. The resolver reads `claims["tenant_id"].as_i64()`. They agree. [VERIFIED: both files read in this session]

### D-07 Fixture: DbTenantLookup wiring in bootstrap

```rust
// Source: framework/src/tenant/lookup.rs — DbTenantLookup::new pattern
let tenant_lookup = Arc::new(DbTenantLookup::new(
    |slug| Box::pin(async move {
        // query tenants table by slug
        Tenant::find_by_slug(&slug).await
            .ok()
            .flatten()
            .map(|t| TenantContext { id: t.id as i64, slug: t.slug, name: t.name, plan: None })
    }),
    |id| Box::pin(async move {
        Tenant::find_by_id(id).await
            .ok()
            .flatten()
            .map(|t| TenantContext { id: t.id as i64, slug: t.slug, name: t.name, plan: None })
    }),
));
let jwt_resolver = JwtClaimResolver::new("tenant_id", tenant_lookup.clone());
let tenant_mw = TenantMiddleware::new()
    .resolver(jwt_resolver)
    .on_failure(TenantFailureMode::Forbidden);
```

### D-07 Fixture: orders migration schema (must match projection field names)

```rust
// Source: order projection field names verified from app/src/projections/order.rs
// Table: "orders" (from format!("{}s", "order"))
// Required columns (must match projection fields exactly):
//   id            INTEGER NOT NULL PK AUTOINCREMENT
//   customer_name TEXT NOT NULL
//   total         REAL NOT NULL
//   status        TEXT NOT NULL
//   created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
//   tenant_id     INTEGER NOT NULL REFERENCES tenants(id)
```

### D-07 Fixture: tenants migration schema

```rust
// Mirror the TenantContext struct: id, slug, name
//   id    INTEGER NOT NULL PK AUTOINCREMENT
//   slug  TEXT NOT NULL UNIQUE
//   name  TEXT NOT NULL
//   created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
```

### D-07 Fixture: User → tenant association

Simplest: add `tenant_id INTEGER NULLABLE REFERENCES tenants(id)` to the `users` table via a new migration. A user with `tenant_id = NULL` is a single-tenant/admin user; a user with `tenant_id = 1` belongs to tenant 1. The OAuth `/authorize` handler captures `current_tenant()` — for the dogfood, `TenantMiddleware` must resolve the tenant from the JWT claim (which was set at authorize time). This creates a bootstrapping dependency: at authorize time, the user must already be associated with a tenant, and `TenantMiddleware` on `/authorize` must use a resolver that knows the user's tenant.

**Simpler resolution for the dogfood:** Mount `TenantMiddleware` with a `HeaderResolver` on `/authorize` (tenant selected via `X-Tenant-Slug` header or subdomain), not via JWT claim (which doesn't exist yet at authorize time). The tenant is bound to the token in `authorize_post` → `build_claims(user_id, current_tenant().map(|t| t.id), ...)`.

**Alternative (cleaner for the test):** Pre-seed users with `tenant_id` column. At `/authorize` GET, resolve tenant from a `SubdomainResolver` or `PathResolver` so the user is within a tenant scope before they log in. The minted token picks up `current_tenant().id`.

For the dogfood fixture specifically: a path-based or header-based resolver on `/authorize` is acceptable since the dogfood runs against a known test URL. [ASSUMED — architecture choice is within Claude's Discretion per D-07]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Inline bearer validation in handler (Phase 199) | Bearer middleware before TenantMiddleware (Phase 200) | Phase 200 | Establishes correct ordering for JwtClaimResolver |
| No tenant scoping in dispatch (Phase 197-199 comment: "Phase 200 owns that seam") | Bound-parameter tenant predicate in dispatch | Phase 200 | Cross-tenant isolation |
| No policy gate on tools/call (Phase 197-199 dispatch runs unconditionally) | Gate::authorize_for before dispatch | Phase 200 | Agent reach bounded by user policy |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Mounting `TenantMiddleware` on `/authorize` using a path/header resolver (not JWT) is the right approach for the dogfood fixture to bind `current_tenant()` at authorize time | Code Examples (D-07) | If wrong, the minted token gets `tenant_id = null` and dogfood fails — same symptom as Phase 199 neutralization |
| A2 | The `order` projection's `mcp_ability` will be named `"view-orders"` (within Claude's Discretion) | Standard Stack | The name doesn't matter so long as it's defined in `Gate::define` in bootstrap and referenced in `ServiceDef.mcp_ability` consistently |
| A3 | `users.tenant_id` FK is the simplest User→tenant association for a single-tenant-per-user dogfood | Code Examples (D-07) | A membership table is more flexible but the dogfood only needs one tenant per user |

**Claims tagged `[VERIFIED]`** constitute the majority of this research. The assumptions above are architectural choices within Claude's Discretion scope.

---

## Open Questions (RESOLVED)

1. **`/authorize` tenant resolver for the dogfood**
   - What we know: `authorize_get`/`authorize_post` capture `current_tenant()`. For the dogfood, the token must carry a real `tenant_id`. `TenantMiddleware` must run on `/authorize`.
   - What's unclear: Which resolver? `SubdomainResolver` needs a real domain. `PathResolver` needs URL restructuring. `HeaderResolver` needs client cooperation.
   - Recommendation: Add a `SubdomainResolver` or use a `HeaderResolver("X-Tenant-Slug")` on `/authorize` for the dogfood, pointing at the seeded tenant slug. Alternatively, add `tenant_id` to the existing `users` table and have `/authorize` resolve tenant from the authenticated user's `tenant_id` (a user-scoped resolver).
   - **RESOLVED:** Plan 04 implements `SessionUserTenantResolver` (reads the session user's `tenant_id` via `Auth::id()` at authorize time, since no JWT exists yet); chosen over subdomain/header for the localhost dogfood.

2. **`users` table migration: add `tenant_id` column or not**
   - What we know: The users table has no `tenant_id` column currently. `TenantMiddleware` on `/authorize` needs to know which tenant the logged-in user belongs to.
   - What's unclear: Whether to (a) add FK to users via a new migration, (b) use a separate resolver that doesn't depend on the user table (subdomain/header), or (c) set a cookie/session after login that specifies tenant.
   - Recommendation: Add `tenant_id INTEGER NULLABLE REFERENCES tenants(id)` to users via a new migration and write a custom resolver that reads the authenticated user's `tenant_id`. This is the cleanest single-source-of-truth approach.
   - **RESOLVED:** Plan 03 adds the `users.tenant_id` association (migration `m20260611_add_tenant_id_to_users.rs` in Plan 03's task) so a user maps to a tenant.

3. **`ferro-mcp-oauth/src/authorize.rs` — does it already call `current_tenant()`?**
   - What we know: Phase 199 CONTEXT D-06 says "tenant bound from `current_tenant()` at authorize time." The code was written in Phase 199.
   - What's unclear: Whether `authorize_post` already has `current_tenant()` call, making this a wiring-only task.
   - Recommendation: Verify by reading `ferro-mcp-oauth/src/authorize.rs` before planning.
   - **RESOLVED:** `ferro-mcp-oauth/src/authorize.rs` already calls `current_tenant()` at line 138, so wiring `TenantMiddleware` onto `/authorize` is sufficient (Plan 04, wiring-only).

---

## Environment Availability

Step 2.6: No external dependencies beyond the project's own code. All required crates are already in the workspace. The sample app server is run manually by the user per project convention.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| SQLite | Sample app DB | ✓ | Workspace (sea-orm sqlx-sqlite) | — |
| `jsonwebtoken` v9 | JWT mint/validate | ✓ | Workspace dep via ferro-wallet/ferro-mcp-oauth | — |
| MCP SDK (Node/Python) | Dogfood scripted client | UNVERIFIED | — | `mcp` CLI or manual Claude Desktop |
| `sea-orm-migration` | New migrations | ✓ | Workspace dep | — |

**Missing dependencies with no fallback:** None blocking Rust code compilation.

**Missing dependencies with fallback:** MCP SDK for scripted client — if npm/pip not available, Claude Desktop config is the fallback for the manual dogfood run (though not reproducible as a script).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` / `tokio::test` |
| Config file | None (workspace-level) |
| Quick run command | `cargo test -p ferro-projections -p ferro-mcp-server -- --nocapture` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AMCP-10 | Token for tenant A returns only tenant A rows | Integration (SQLite, two tenant IDs) | `cargo test -p ferro-mcp-server tenant_scoping` | ❌ Wave 0 |
| AMCP-10 | Token for tenant B returns only tenant B rows (cross-tenant isolation) | Integration (SQLite, two tenant IDs) | `cargo test -p ferro-mcp-server tenant_isolation` | ❌ Wave 0 |
| AMCP-10 | `tenant_column = Some`, tenant_id = None → error/zero rows | Unit | `cargo test -p ferro-mcp-server tenant_fail_closed` | ❌ Wave 0 |
| AMCP-11 | `mcp_ability = None` on mcp_exposed projection → policy-deny tool error | Unit | `cargo test -p {app or ferro-mcp-server} policy_deny_no_ability` | ❌ Wave 0 |
| AMCP-11 | Gate deny → `isError: true`, no row data disclosed | Unit | `cargo test -p {app} policy_deny_tool_error_shape` | ❌ Wave 0 |
| SC-3 | `current_tenant()` established via same `TenantMiddleware` path | Unit (middleware chain test) | `cargo test -p framework tenant_middleware_jwt_chain` | ❌ Wave 0 |
| SC-4 | Real MCP client completes browser login, calls tools/call, receives tenant-scoped rows | Manual dogfood | Scripted `dogfood/run_dogfood.ts` + human browser login | ❌ Wave 0 |

**Important:** SC-1 (tenant A ≠ tenant B isolation) REQUIRES a real two-tenant SQLite dataset. A single-tenant happy-path test does not prove isolation. The test must seed two tenants, two users, and orders for each tenant, then verify that querying with tenant A's token does not return tenant B's orders.

### SC-4 is manual-only by design

The dogfood acceptance gate (SC-4) is human-in-the-loop: the user starts the app, performs the browser login, and a scripted MCP client handles the OAuth token exchange and tool call. The result is recorded in `200-ACCEPTANCE.md` as GO or NO-GO. A NO-GO blocks phase completion per the milestone invariant.

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-projections -p ferro-mcp-server -- --nocapture`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work` + `200-ACCEPTANCE.md` recording GO

### Wave 0 Gaps

- [ ] `ferro-mcp-server/src/tests/tenant_scoping.rs` — two-tenant isolation test with SQLite fixture
- [ ] `ferro-mcp-server/src/tests/tenant_fail_closed.rs` — `tenant_column=Some`, no tenant_id → error
- [ ] `app/src/tests/policy_gate.rs` — policy-deny tool error shape test
- [ ] `dogfood/run_dogfood.ts` (or `.py`) — scripted MCP client for acceptance run

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No (handled in Phase 199) | — |
| V3 Session Management | No (JWT, not sessions) | — |
| V4 Access Control | **YES** | `Gate::authorize_for` — same policy layer as web surface |
| V5 Input Validation | **YES** | Dispatch filter-key allowlist already in place (Phase 197); tenant_id is bound parameter never from payload |
| V6 Cryptography | No (JWT crypto handled in Phase 199) | — |

### Tenant Isolation Threat Model

| Threat | STRIDE | Mitigation |
|--------|--------|-----------|
| Agent passes `tenant_id` in tool call arguments to override scope | Tampering | `tenant_id` is NEVER a filter-eligible field (comes from `current_tenant()`, not from call payload); dispatch allowlist rejects it as a filter key |
| Token with crafted `tenant_id` claim for another tenant | Elevation of Privilege | `JwtClaimResolver::find_by_id` validates against DB; `TenantMiddleware` with `on_failure=Forbidden` rejects unknown tenants |
| `mcp_ability = None` projection callable without permission | Elevation of Privilege | Fail-closed: None ability → deny tool error before dispatch |
| Cross-tenant data via unscoped SELECT when `current_tenant()` is None | Information Disclosure | `dispatch`: `tenant_column = Some` + `tenant_id = None` → error (never falls back to unscoped SELECT) |
| Policy-deny response leaking table schema or row count | Information Disclosure | `isError: true` result contains only a human-readable denial message; no columns, no counts, no filter values |

---

## Sources

### Primary (HIGH confidence — code read in this session)

- `ferro-mcp-oauth/src/jwt.rs` — `McpTokenClaims` struct, `build_claims`, `mint_token`, `tenant_id: Option<i64>` field name verified
- `ferro-mcp-oauth/src/validate.rs` — `BearerCheck::Authenticated(serde_json::Value)` carrying `{"sub": ..., "tenant_id": ...}`
- `framework/src/tenant/resolver.rs` — `JwtClaimResolver::resolve`: reads `claims["tenant_id"].as_i64()`
- `framework/src/tenant/middleware.rs` — `TenantMiddleware::handle`: passes `request` by value to `next`, tenant in task-local
- `framework/src/tenant/context.rs` — `current_tenant()` task-local
- `framework/src/tenant/lookup.rs` — `TenantLookup::find_by_id`, `DbTenantLookup::new` pattern
- `framework/src/middleware/mod.rs` — `Middleware` trait, `Next`, `handle(&self, request: Request, ...)` by value
- `framework/src/http/request.rs` — `insert::<T>`, `get::<T>` on owned extensions HashMap
- `framework/src/authorization/gate.rs` — `Gate::authorize_for`, `Gate::inspect` (does NOT check session), `Gate::authorize` (DOES check session via `Auth::id()`)
- `framework/src/authorization/policy.rs` — `Policy<M>` trait, why typed policy needs concrete `M`
- `framework/src/authorization/response.rs` — `AuthResponse`, `.message()`, `.denied()`
- `ferro-mcp-server/src/dispatch.rs` — full dispatch source, `format!("{}s", name)` table derivation, where-clause build
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_call`, result shape, error codes
- `ferro-mcp-server/Cargo.toml` — NO `framework` dependency (confirmed)
- `app/src/controllers/mcp.rs` — Phase 199 state, inline bearer validation, `BearerCheck::Authenticated(_principal)` comment
- `app/src/projections/order.rs` — field names: `id, customer_name, total, status, created_at`
- `app/src/routes.rs` — current route registration (no TenantMiddleware yet)
- `app/src/migrations/mod.rs` — current migrations list
- `app/src/migrations/m20251208_160100_create_users_table.rs` — migration pattern
- `ferro-projections/src/service.rs` — `ServiceDef` current fields (`mcp_exposed: bool` present; `tenant_column`/`mcp_ability` absent)
- `.planning/phases/199-oauth-browser-login/199-CONTEXT.md` — Phase 199 decisions
- `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md` — design spec §Tenant and policy reuse, §Error handling

### Secondary (MEDIUM confidence)

- `.planning/phases/95-multi-tenant-middleware/95-CONTEXT.md` — `TenantContext` shape, resolver patterns

---

## Metadata

**Confidence breakdown:**

- D-03 claim-name reconciliation: HIGH — both sides verified from source
- D-01 middleware ordering feasibility: HIGH — `Middleware` trait, `Request` by value, `insert`/`get` all verified
- D-02 dispatch predicate injection: HIGH — dispatch source read, no `framework` dep confirmed
- D-04 Gate::authorize_for (not Gate::authorize): HIGH — gate.rs session-check vs explicit-user difference verified
- D-07 fixture schema: HIGH for orders fields (from projection), MEDIUM for user→tenant strategy (architectural choice)
- D-09 tool-error shape: HIGH — jsonrpc.rs result pattern + MCP spec `isError` convention

**Research date:** 2026-06-10
**Valid until:** 2026-07-10 (stable framework APIs)
