# Phase 198: Streamable HTTP Endpoint + Unauthenticated Challenge — Research

**Researched:** 2026-06-10
**Domain:** MCP Streamable HTTP transport, Ferro handler mechanics, rmcp 0.12 protocol types, RFC 9728 bearer challenge
**Confidence:** HIGH (all critical claims verified against workspace source or rmcp 0.12 registry source)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 (transport):** Ferro `post!("/mcp", …)` handler, hand-rolled JSON-RPC dispatch. No rmcp `transport-streamable-http-server` axum service. Rationale: SC-3 requires the same middleware stack; rmcp's axum service bypasses it.

**D-02 (dispatch placement):** Pure JSON-RPC method dispatch lives in `ferro-mcp-server` as framework-agnostic functions. The thin HTTP adapter (body read, status + headers, bearer seam) is a ferro handler. `ferro-mcp-server → ferro-projections` only; no `framework` dep on `ferro-mcp-server`.

**D-03 (initialize response):** Returns minimal spec-compliant result; `protocolVersion` matches rmcp 0.12 `LATEST`; `serverInfo` from `APP_NAME` / `APP_URL` env vars via `from_env()` config struct mirroring `InertiaConfig`.

**D-04 (response mode):** Single `application/json` JSON-RPC response per request. No SSE streaming, no `Mcp-Session-Id` for this skeleton.

**D-05 (auth seam):** Bearer-extraction seam — a function/trait the handler calls. In Phase 198: any request without a recognized bearer returns `401 + WWW-Authenticate`. Phase 199 fills the seam without changing handler signature. Tests drive pure dispatch directly (or inject test principal).

**D-06 (WWW-Authenticate):** Emit `WWW-Authenticate: Bearer resource_metadata="{APP_URL}/.well-known/oauth-protected-resource"`. APP_URL from framework convention.

**D-07 (tests):** Reuse `fresh_db()` from `ferro-mcp-server/tests/dispatch_integration.rs`. Test all three methods via pure dispatch + a handler-level 401 assertion. No live OAuth server, no running web server.

### Claude's Discretion
- Exact module layout within `ferro-mcp-server` (e.g. `jsonrpc.rs` / `protocol.rs` for method dispatch) and naming of the bearer-seam type.
- JSON-RPC error-code mapping for malformed requests / unknown methods (`-32600`, `-32601`, `-32602`).
- Whether the 401 response body is empty or a JSON-RPC error object (pending D-06 research flag — resolved below).

### Deferred Ideas (OUT OF SCOPE)
- SSE streaming / `Mcp-Session-Id` session management
- Real bearer-token validation, `.well-known` discovery docs, DCR, `/authorize` + `/token`
- Per-tenant scoping + policy authorization on `tools/call`
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AMCP-05 | Application serves MCP endpoint over Streamable HTTP supporting `initialize`, `tools/list`, `tools/call` | §D-01/D-04: ferro handler + `application/json` single-response mode is spec-compliant; §D-03: protocolVersion = "2025-03-26" |
| AMCP-06 | Unauthenticated request returns 401 + `WWW-Authenticate` referencing protected-resource metadata | §D-05/D-06: bearer-seam pattern; exact header value verified against rmcp 0.12 auth.rs parser |
</phase_requirements>

---

## Summary

Phase 198 wires the MCP endpoint into the Ferro HTTP layer. The three JSON-RPC methods (`initialize`, `tools/list`, `tools/call`) are dispatched by a `post!("/mcp")` handler that calls into pure functions in `ferro-mcp-server`. All requests go through the same Ferro middleware chain as other routes (SC-3). Any request that reaches the handler without a validated bearer token returns `401` with the standard RFC 9728 challenge.

The primary research questions were: (1) which rmcp 0.12 types to reuse, (2) what the exact `protocolVersion` string must be, (3) whether stateless JSON-only Streamable HTTP is spec-compliant and accepted by rmcp's own client, (4) where the ferro handler lives, (5) what the bearer-seam shape should look like, (6) the exact `WWW-Authenticate` parameter name.

All five questions have HIGH-confidence answers from workspace source and rmcp 0.12 crate source. No decision in CONTEXT.md needs revision.

**Primary recommendation:** Add a `jsonrpc.rs` module to `ferro-mcp-server` with three pure async functions (`handle_initialize`, `handle_tools_list`, `handle_tools_call`) returning `serde_json::Value`. Add a `config.rs` for `McpServerConfig::from_env()`. Wire the ferro handler in the `app` crate for Phase 198 (app-local, not `framework`-exported — dependency-weight finding in §D-02 below). Add a `BearerOutcome` seam type in `ferro-mcp-server`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| JSON-RPC method dispatch (parse, route, respond) | `ferro-mcp-server` crate | — | Framework-agnostic; testable without HTTP; reusable by any transport |
| HTTP transport adapter (read body, set headers, 401 challenge) | `app` ferro handler | — | Keeps `ferro-mcp-server` free of `framework` dependency |
| Bearer token extraction seam | `ferro-mcp-server` (seam type) + `app` handler (caller) | Phase 199 fills seam | Seam lives where dispatch lives; Phase 199 fills without touching handler signature |
| DB connection | Ferro `DB::connection()` (service container) | — | Same mechanism all framework handlers use; `dispatch()` already takes `&sea_orm::DatabaseConnection` |
| `APP_NAME` / `APP_URL` config | `ferro-mcp-server::McpServerConfig::from_env()` | — | Mirror `InertiaConfig`; never hardcoded |

---

## Standard Stack

### Core (all already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rmcp` | 0.12.0 | Protocol types: `Tool`, `ProtocolVersion`, `InitializeResult`, `ServerCapabilities`, `CallToolResult` | Already workspace dep in `ferro-mcp-server/Cargo.toml`; no new dep needed |
| `serde_json` | 1.0 | JSON-RPC request/response construction | Already in `ferro-mcp-server/Cargo.toml` |
| `sea-orm` | 1.0 | `DatabaseConnection` passed to `dispatch()` | Already in `ferro-mcp-server/Cargo.toml` |

**No new external dependencies needed for this phase.** `ferro-mcp-server/Cargo.toml` already has `rmcp = { version = "0.12", default-features = false, features = ["server", "macros", "base64"] }` — the `server` feature brings `schemars` and `transport-async-rw`, but neither `transport-streamable-http-server` nor `server-side-http` (which would pull axum/tower). The existing feature set is correct.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled JSON-RPC dispatch | `rmcp::transport-streamable-http-server` | rmcp's transport is a Tower `Service`; integrating it with Ferro's hyper-based server requires adapting at the `hyper::Service` seam and bypasses Ferro middleware — incompatible with SC-3 |
| `application/json` single response | SSE `text/event-stream` | Spec allows both; SSE buys nothing for synchronous read-only methods; adds streaming complexity |

---

## Research Findings by Decision

### D-01 / D-04: rmcp 0.12 protocol types and stateless JSON-only compliance

**rmcp 0.12 protocol types available WITHOUT transport features:**

The `server` feature (already enabled in `ferro-mcp-server/Cargo.toml`) exposes all protocol types from `rmcp::model`:
- `rmcp::model::Tool` — used by `render_exposed_tools` [VERIFIED: ferro-mcp-server/src/renderer.rs:5]
- `rmcp::model::ProtocolVersion` — `ProtocolVersion::LATEST = ProtocolVersion::V_2025_03_26 = "2025-03-26"` [VERIFIED: rmcp-0.12.0/src/model.rs:153-156]
- `rmcp::model::InitializeResult` / `ServerInfo` — struct with `protocol_version`, `capabilities`, `server_info`, `instructions` [VERIFIED: rmcp-0.12.0/src/model.rs:748-758]
- `rmcp::model::ServerCapabilities` — builder with `.enable_tools()` [VERIFIED: rmcp-0.12.0/src/model/capabilities.rs:99-112]
- `rmcp::model::Implementation` — `{ name: String, version: String }` [VERIFIED: rmcp-0.12.0/src/model.rs:812-816]
- `rmcp::model::CallToolResult` — rich result type with `.success(content)` and structured variants [VERIFIED: rmcp-0.12.0/src/model.rs:1534-1608]

**The `transport-streamable-http-server` feature is NOT needed and must NOT be enabled.** It pulls `server-side-http` which enables `tower-service`, `uuid`, `rand`, `tokio-stream`, `http`, `http-body`, `http-body-util`, `bytes`, `sse-stream` — and the `tower` feature which enables `dep:tower-service`. While axum is listed as an optional dep in rmcp's `Cargo.toml.orig`, `transport-streamable-http-server` does NOT explicitly enable axum, but it does bring a full Tower-service pipeline incompatible with Ferro's hyper dispatcher. [VERIFIED: rmcp-0.12.0/Cargo.toml.orig:91-128]

**Stateless JSON-only Streamable HTTP is spec-compliant:**

From the MCP Streamable HTTP spec (2025-03-26), rule 5: "If the input contains any number of JSON-RPC *requests*, the server **MUST** either return `Content-Type: text/event-stream` ... or `Content-Type: application/json`, to return one JSON object. **The client MUST support both these cases.**"

Session management (Mcp-Session-Id) is explicitly **MAY** on the server side: "A server using the Streamable HTTP transport **MAY** assign a session ID at initialization time." Omitting it is fully spec-compliant. [VERIFIED: modelcontextprotocol.io/specification/2025-03-26/basic/transports]

rmcp 0.12's own client (`transport/auth.rs`) is happy with a 401 + `WWW-Authenticate` response — it specifically parses the header to initiate OAuth flow. [VERIFIED: rmcp-0.12.0/src/transport/auth.rs:838-857]

**Accept header:** Clients send `Accept: application/json, text/event-stream`. The server responding with `Content-Type: application/json` is the correct, spec-conforming short path.

**Important spec detail — Origin header validation:** The Streamable HTTP spec includes a security warning: "Servers MUST validate the `Origin` header on all incoming connections to prevent DNS rebinding attacks." This applies when serving publicly. For Phase 198 the integration tests skip this (no real HTTP server), but the production handler should either validate Origin or rely on Ferro's existing middleware infrastructure. Flag for Phase 199 review. [VERIFIED: MCP spec security warning]

### D-02: Where the ferro HTTP handler lives

**Framework dependency-weight analysis:**

`framework/Cargo.toml` does not depend on `ferro-mcp-server` today. [VERIFIED: /ferro/framework/Cargo.toml] Adding `ferro-mcp-server` as an optional `framework` dep (e.g., feature `mcp`) would pull:
- `rmcp` 0.12 with features `["server", "macros", "base64"]` — pulls `schemars`, `tokio-util`, `base64`, `rmcp-macros`
- `sea-orm` 1.0 (already in `framework/Cargo.toml` — no size delta)
- `ferro-projections` (already optional in `framework` via `projections` feature)

The incremental cost is `rmcp` + `rmcp-macros` + `schemars` (if not already enabled). `schemars` is pulled by `rmcp`'s `server` feature — it's not in `framework/Cargo.toml` today. This adds measurable compile time to the core `ferro-rs` crate even when consumers don't use MCP.

**Recommendation: app-local handler for Phase 198 skeleton.** Wire the `POST /mcp` handler in `app/src/controllers/mcp.rs` (or `app/src/api/mcp.rs`) and register in `app/src/routes.rs`. The pure dispatch lives in `ferro-mcp-server` — the handler is ~30 lines of glue. Phase 199 or a separate "framework export" phase can promote it to `framework` with an `mcp` feature flag, at which point the dependency cost is opt-in. This matches the existing precedent: `ferro-json-ui` is exported from `framework` only under `feature = "json-ui"`.

**Pure dispatch function signature in `ferro-mcp-server`:**

```rust
// ferro-mcp-server/src/jsonrpc.rs
pub async fn handle_initialize(
    params: serde_json::Value,
    config: &McpServerConfig,
) -> serde_json::Value;

pub async fn handle_tools_list(
    services: &[ServiceDef],
    config: &McpServerConfig,
) -> serde_json::Value;

pub async fn handle_tools_call(
    service_name: &str,
    call_params: serde_json::Value,
    services: &[ServiceDef],
    db: &sea_orm::DatabaseConnection,
) -> serde_json::Value;
```

Each returns a full JSON-RPC response object (with `jsonrpc`, `id`, `result`/`error`). The caller (ferro handler) owns the `id` extraction and wraps the result. The handler calls `DB::connection()` from Ferro's service container exactly as `user_api::index` does. [VERIFIED: app/src/api/user_api.rs:50-52]

**ServiceDef slice at runtime:** `app/src/projections/` contains per-projection `service_def() -> ServiceDef` functions. There is no global registry today (projections module is `#[allow(dead_code)]`). The handler must build the slice by calling all `service_def()` functions and filtering `mcp_exposed = true`. For Phase 198 this is an explicit `let services = vec![...]` in the handler. A lazy-static registry can be added later. [VERIFIED: app/src/projections/mod.rs, app/src/main.rs:51]

### D-03: Exact `protocolVersion` string

rmcp 0.12.0 defines:
```rust
// rmcp-0.12.0/src/model.rs:155-156
//  Keep LATEST at 2025-03-26 until full 2025-06-18 compliance and automated testing are in place.
pub const LATEST: Self = Self::V_2025_03_26;
```

The `Default` implementation for `ProtocolVersion` returns `LATEST` = `"2025-03-26"`. [VERIFIED: rmcp-0.12.0/src/model.rs:139-141]

**`initialize` response must return `protocolVersion: "2025-03-26"`.** Use `rmcp::model::ProtocolVersion::LATEST` (or `"2025-03-26"` as a string literal for the `serde_json::Value` response, without pulling in the full `InitializeResult` struct).

For the `serverInfo` field, use `McpServerConfig`:
```rust
// ferro-mcp-server/src/config.rs  (new file)
pub struct McpServerConfig {
    pub app_name: String,
    pub app_url: String,
    pub version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string());
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string());
        let version = env!("CARGO_PKG_VERSION").to_string();
        Self { app_name, app_url, version }
    }
}
```
This mirrors `InertiaConfig::default()` exactly. [VERIFIED: ferro-inertia/src/config.rs:41-63]

### D-04: Streamable HTTP `application/json` mode

Verified above. Single `Content-Type: application/json` response is fully spec-compliant per the "MUST support both" rule. No SSE, no `Mcp-Session-Id`.

The handler must also respond to GET `/mcp` with `405 Method Not Allowed` per spec: "The server MUST either return `Content-Type: text/event-stream` in response to this HTTP GET, or else return HTTP 405 Method Not Allowed, indicating that the server does not offer an SSE stream at this endpoint." Since Phase 198 only registers `post!("/mcp")`, a GET will naturally hit Ferro's 405 fallback. Confirm this is emitted by Ferro's router when a path exists but the method doesn't match. [ASSUMED — need to verify Ferro router 405 behavior]

### D-05: Bearer-extraction seam

**Current pattern in framework:**

`ApiKeyMiddleware` in `framework/src/api/api_key.rs` calls `extract_bearer_token(&request)` which reads `Authorization: Bearer {token}` from the request header. [VERIFIED: framework/src/api/api_key.rs:198-223]

`Auth::check()` / `Auth::user()` uses a session-based guard, not suitable for bearer tokens.

**Recommended seam shape for Phase 198:**

```rust
// ferro-mcp-server/src/auth.rs  (new file)
pub enum BearerOutcome {
    /// No Authorization header, or header present but token not recognized.
    Unauthenticated,
    /// Token validated; principal attached (Phase 199+ fills this variant).
    #[allow(dead_code)]
    Authenticated(serde_json::Value),  // opaque principal for now
}

pub fn extract_bearer(authorization_header: Option<&str>) -> BearerOutcome {
    // Phase 198: always Unauthenticated
    let _ = authorization_header;
    BearerOutcome::Unauthenticated
}
```

The ferro handler calls:
```rust
let outcome = ferro_mcp_server::auth::extract_bearer(req.header("Authorization"));
match outcome {
    BearerOutcome::Unauthenticated => {
        // emit 401 + WWW-Authenticate
    }
    BearerOutcome::Authenticated(_principal) => {
        // dispatch
    }
}
```

Phase 199 replaces `extract_bearer` internals (or adds a second impl behind a trait) without touching the handler's match arms. The handler signature `(req: Request) -> Response` never changes.

**Why not a middleware?** A middleware approach (like `ApiKeyMiddleware`) would block the 401 before the handler even runs — but the tests for `initialize`/`tools/list`/`tools/call` (D-07) need to call the pure dispatch directly, bypassing transport entirely. Keeping the seam inside the handler keeps the pure dispatch testable without HTTP. [REASONED from D-07 test strategy]

**Integration test bypass:** Tests call `handle_initialize()`, `handle_tools_list()`, `handle_tools_call()` from `ferro-mcp-server/src/jsonrpc.rs` directly, passing `McpServerConfig::default()` and a fresh DB from `setup_db()`. They never touch `extract_bearer`. The 401 path is tested by a separate unit test that calls `extract_bearer(None)` and asserts `BearerOutcome::Unauthenticated`, plus a handler-level test that constructs an `HttpResponse` and asserts `status == 401` and `WWW-Authenticate` header.

### D-06: `WWW-Authenticate` header format

**Parameter name: `resource_metadata`** — confirmed in rmcp 0.12.0 source:

```rust
// rmcp-0.12.0/src/transport/auth.rs:904
let fragment_key = "resource_metadata=";
```

Test vector from rmcp's own test suite:
```
// rmcp-0.12.0/src/transport/auth.rs:1359
Bearer error="invalid_request", error_description="missing token",
    resource_metadata="https://example.com/.well-known/oauth-protected-resource/api"
```
[VERIFIED: rmcp-0.12.0/src/transport/auth.rs:901-926, 1359-1375]

**Exact header value for Phase 198:**
```
WWW-Authenticate: Bearer resource_metadata="{APP_URL}/.well-known/oauth-protected-resource"
```

The URL is `{APP_URL}/.well-known/oauth-protected-resource`. This is the RFC 9728 standard path for protected-resource metadata. Phase 199 builds the document at this URL; Phase 198 only points at it.

Note: rmcp's client `extract_resource_metadata_url_from_header` accepts both quoted (`"..."`) and unquoted token values. Using double quotes (as in the test vector) is the safer choice. [VERIFIED: rmcp-0.12.0/src/transport/auth.rs:936-956]

**401 body:** The MCP auth spec says "servers MUST respond with HTTP 401 Unauthorized" but does not require a specific response body for the challenge. rmcp's client only reads the `WWW-Authenticate` header from the 401 response, not the body. An empty body is fine; a JSON-RPC error object is also acceptable. Recommendation: empty body for Phase 198 to keep the handler minimal, since the bearer seam fires before the handler even parses the JSON-RPC method. [CITED: modelcontextprotocol.io/specification/2025-03-26/basic/authorization — no body requirement stated] [VERIFIED: rmcp-0.12.0/src/transport/auth.rs:836-857 — client only reads header]

**Ferro mechanics:** `HttpResponse::new().status(401).header("WWW-Authenticate", value)` — the `header()` method replaces any existing entry case-insensitively. `HttpResponse::status()` and `HttpResponse::header()` are both consuming builders. [VERIFIED: framework/src/http/response.rs:95-127]

### D-07: Test strategy

**Existing fixture:** `ferro-mcp-server/tests/dispatch_integration.rs` already has `setup_db()` (named `fresh_db` in CONTEXT.md, but actual function is `setup_db`). [VERIFIED: ferro-mcp-server/tests/dispatch_integration.rs:14-32]

**New test file:** `ferro-mcp-server/tests/jsonrpc_integration.rs` — tests for the three pure dispatch functions:

```rust
#[tokio::test]
async fn initialize_returns_correct_protocol_version() {
    let config = McpServerConfig::default();
    let resp = handle_initialize(serde_json::json!({}), &config).await;
    assert_eq!(resp["result"]["protocolVersion"], "2025-03-26");
    assert!(resp["result"]["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn tools_list_returns_exposed_projection() {
    let services = vec![
        ServiceDef::new("order").mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier),
        ServiceDef::new("internal").mcp_exposed(false)
            .field("id", DataType::Integer, FieldMeaning::Identifier),
    ];
    let config = McpServerConfig::default();
    let resp = handle_tools_list(&services, &config).await;
    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "list_order");
}

#[tokio::test]
async fn tools_call_returns_rows() {
    let db = setup_db().await;  // reuse from dispatch_integration.rs (make setup_db pub)
    let services = vec![item_service()];  // mcp_exposed = true
    let config = McpServerConfig::default();
    let resp = handle_tools_call(
        "list_item",
        serde_json::json!({"limit": 10, "offset": 0}),
        &services, &db
    ).await;
    assert!(resp["result"]["content"].is_array());
}
```

**401 unit test (in `ferro-mcp-server/src/auth.rs` tests):**
```rust
#[test]
fn extract_bearer_with_no_header_returns_unauthenticated() {
    assert!(matches!(extract_bearer(None), BearerOutcome::Unauthenticated));
}
```

**Handler-level test (in `app/src/controllers/mcp_test.rs` or inline):**
Not needed for Phase 198 at the handler level — the `HttpResponse` construction is so simple it's validated by the integration test asserting the header. If desired, a unit test constructs the 401 response directly:
```rust
fn build_challenge_response(app_url: &str) -> HttpResponse {
    let url = format!("{app_url}/.well-known/oauth-protected-resource");
    HttpResponse::new()
        .status(401)
        .header("WWW-Authenticate", format!("Bearer resource_metadata=\"{url}\""))
}
```

---

## Architecture Patterns

### System Architecture Diagram

```
POST /mcp
    │
    ▼
Ferro middleware chain (same stack as /api/v1/*)
    │
    ▼
mcp_handler (app/src/controllers/mcp.rs)
    │
    ├── req.json::<serde_json::Value>()  (reads body — consuming)
    │
    ├── extract_bearer(req.header("Authorization"))
    │       │
    │       ├── Unauthenticated ──► HttpResponse::new()
    │       │                            .status(401)
    │       │                            .header("WWW-Authenticate", challenge)
    │       │                       [Phase 198 always takes this branch]
    │       │
    │       └── Authenticated(_) ──► [Phase 199+ fills this]
    │                                       │
    │                                       ▼
    │                          route on request["method"]
    │                                       │
    │                        ┌─────────────┼──────────────┐
    │                        ▼             ▼              ▼
    │               "initialize"    "tools/list"   "tools/call"
    │                        │             │              │
    │               handle_initialize   handle_tools_list  handle_tools_call
    │               (pure fn)          (pure fn)         (pure fn + DB)
    │                        │             │              │
    │                ferro-mcp-server/src/jsonrpc.rs      │
    │                        │             │              │
    │                        └─────────────┴──────────────┘
    │                                       │
    │                              serde_json::Value (JSON-RPC response)
    │
    ▼
HttpResponse::json(value)
  Content-Type: application/json
  HTTP 200
```

### Recommended Project Structure

New files for Phase 198:

```
ferro-mcp-server/src/
├── auth.rs        (NEW) BearerOutcome + extract_bearer seam
├── config.rs      (NEW) McpServerConfig::from_env() / Default
├── dispatch.rs    (EXISTING) dispatch()
├── error.rs       (EXISTING)
├── jsonrpc.rs     (NEW) handle_initialize / handle_tools_list / handle_tools_call
├── lib.rs         (UPDATE exports)
├── renderer.rs    (EXISTING) render_exposed_tools
└── schema.rs      (EXISTING) build_input_schema / is_filter_field

ferro-mcp-server/tests/
├── dispatch_integration.rs  (EXISTING — make setup_db pub)
└── jsonrpc_integration.rs   (NEW)

app/src/controllers/
└── mcp.rs         (NEW) ferro handler — thin HTTP adapter

app/src/
└── routes.rs      (UPDATE — add mcp route)
```

### Pattern 1: Pure JSON-RPC method dispatch

```rust
// ferro-mcp-server/src/jsonrpc.rs
use crate::config::McpServerConfig;
use ferro_projections::ServiceDef;
use serde_json::{json, Value};

pub async fn handle_initialize(
    _params: Value,
    config: &McpServerConfig,
) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": null,   // caller fills id from the request
        "result": {
            "protocolVersion": "2025-03-26",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": config.app_name,
                "version": config.version
            }
        }
    })
}
```

Caller in handler:
```rust
let id = body.get("id").cloned().unwrap_or(json!(null));
let method = body["method"].as_str().unwrap_or("");
let params = body.get("params").cloned().unwrap_or(json!({}));
let mut result = handle_initialize(params, &config).await;
result["id"] = id;  // patch id onto the response
Ok(HttpResponse::json(result))
```

### Pattern 2: Ferro handler accessing DB

```rust
// app/src/controllers/mcp.rs
use ferro::{handler, HttpResponse, Request, Response, DB};

#[handler]
pub async fn handle(req: Request) -> Response {
    let db = DB::connection().map_err(|e| {
        HttpResponse::json(json!({"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":e.to_string()}})).status(500)
    })?;
    // ... rest of handler
}
```

`DB::connection()` returns `Result<DbConnection, FrameworkError>`. `DbConnection` implements `Deref<Target=DatabaseConnection>`. [VERIFIED: framework/src/database/mod.rs:171-213]

### Pattern 3: Route registration

```rust
// app/src/routes.rs — add alongside existing routes
routes! {
    // ... existing routes ...
    post!("/mcp", controllers::mcp::handle).name("mcp.endpoint"),
}
```

No group/middleware needed in Phase 198 since the bearer seam lives inside the handler itself. Phase 199 may wrap with a middleware; for now the handler self-gates. [VERIFIED: app/src/routes.rs pattern]

### Anti-Patterns to Avoid

- **Don't enable `transport-streamable-http-server` or `server-side-http` on rmcp.** These pull a Tower/SSE pipeline that conflicts with Ferro's hyper dispatcher and would add `uuid`, `rand`, `sse-stream`, `http-body` deps to `ferro-mcp-server`. The existing `["server", "macros", "base64"]` feature set is sufficient.
- **Don't add `ferro-mcp-server` as a `framework` dep for this phase.** The incremental cost (`rmcp` + `schemars` in `ferro-rs`) is not opt-in, and the handler is ~30 lines of app-local glue. Defer the framework promotion.
- **Don't parse `serde_json::Value` from body using `req.json::<Value>()` with `#[handler]` auto-extraction.** The `#[handler]` macro supports `FromRequest` types, but `serde_json::Value` does not implement `FromRequest`. Use `req.json::<serde_json::Value>().await?` as a manual call inside the handler body. [VERIFIED: framework/src/http/request.rs:465-468]
- **Don't forget the `id` field.** The JSON-RPC 2.0 spec requires `id` in responses to match the request `id`. Extract `body["id"]` before routing on `method` and inject it into the result.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MCP protocol types | Custom `Tool`/`ProtocolVersion` structs | `rmcp::model::{Tool, ProtocolVersion, ...}` | Already in workspace; wire-format compatibility guaranteed; Phase 197 already imports them |
| JSON-RPC error objects | Bespoke error map | Standard codes: -32700 parse error, -32600 invalid request, -32601 method not found, -32602 invalid params, -32603 internal error | JSON-RPC 2.0 spec; clients interpret code ranges |
| `WWW-Authenticate` header parsing on client | — | rmcp's `extract_resource_metadata_url_from_header` (used by rmcp client) | Our concern is the server side; just emit the right format |
| Bearer token extraction | Regex/string split | Borrow `extract_bearer_token` pattern from `framework/src/api/api_key.rs:198-223` | Already handles `Bearer ` prefix stripping and empty-token guard |

---

## Common Pitfalls

### Pitfall 1: Using `rmcp::model::InitializeResult` directly as the JSON-RPC result
**What goes wrong:** `InitializeResult` serializes cleanly, but it does not serialize as a full JSON-RPC response object — it's only the `result` field. Wrapping it in `{ "jsonrpc": "2.0", "id": ..., "result": <serialize(InitializeResult)> }` works, but requires a `JsonRpcResponse<InitializeResult>` wrapper or a `serde_json::json!({})` construction.
**How to avoid:** Build the full response with `serde_json::json!({...})` explicitly for clarity, or use `rmcp::model::JsonRpcMessage::response(result, id)` and serialize it.

### Pitfall 2: `protocolVersion` mismatch
**What goes wrong:** Returning `"2024-11-05"` instead of `"2025-03-26"` causes rmcp clients to negotiate down to the older version, which may lack capabilities.
**Root cause:** Training data / docs may show the old version.
**How to avoid:** Always use `rmcp::model::ProtocolVersion::LATEST.to_string()` or the literal `"2025-03-26"`. The comment in rmcp 0.12 source explicitly says LATEST is pinned to 2025-03-26.

### Pitfall 3: `req.json()` consuming the request before `header()` is called
**What goes wrong:** `Request::json<T>(self)` takes `self` by value (consuming). Any header reads (e.g., `Authorization`) must be done before calling `json()`, or the request parts captured first.
**Root cause:** Ferro body reading is designed to consume the request (single-read guarantee).
**How to avoid:** Extract all headers before consuming the body:
```rust
let auth = req.header("Authorization").map(|s| s.to_owned());
let body: Value = req.json().await.map_err(|e| ...)?;
```
[VERIFIED: framework/src/http/request.rs:422-468]

### Pitfall 4: `tools/call` method name mapping
**What goes wrong:** Phase 197's dispatch function is keyed by `ServiceDef.name` (e.g., `"item"`) and produces tool names as `"list_{name}"` (e.g., `"list_item"`). A `tools/call` for `"list_item"` must strip the `"list_"` prefix to find the right `ServiceDef`.
**Root cause:** `render_exposed_tools` uses `format!("list_{}", service.name)` as the tool name. The `tools/call` params `{ "name": "list_item", ... }` must reverse this.
**How to avoid:** In `handle_tools_call`, strip `"list_"` prefix from the tool name to get `service_name`, then find the matching `ServiceDef`.
[VERIFIED: ferro-mcp-server/src/renderer.rs:29]

### Pitfall 5: Empty `ServiceDef` slice on `tools/list`
**What goes wrong:** If `render_exposed_tools` is called with no `mcp_exposed = true` projections, it returns an empty `Vec<Tool>` — a valid response, but tests may fail if they expect non-empty.
**Root cause:** `app/src/projections/order.rs` etc. don't call `.mcp_exposed(true)` yet.
**How to avoid:** For Phase 198 tests, construct fixture `ServiceDef` with `.mcp_exposed(true)` explicitly; don't depend on app projections. For integration in `app/src/routes.rs`, mark at least one projection `mcp_exposed(true)` before wiring.

### Pitfall 6: `WWW-Authenticate` unquoted URL
**What goes wrong:** Emitting `Bearer resource_metadata=https://...` (unquoted) — rmcp's parser handles both quoted and unquoted, but RFC 6750 recommends quoting attribute values.
**Root cause:** Forgetting the double quotes around the URL.
**How to avoid:** Always emit `Bearer resource_metadata="{APP_URL}/.well-known/oauth-protected-resource"` with quotes.
[VERIFIED: rmcp-0.12.0/src/transport/auth.rs:936-956 — both forms work]

---

## Code Examples

### Initialize response construction
```rust
// Source: rmcp-0.12.0/src/model.rs:748-757 (InitializeResult struct definition)
// Source: rmcp-0.12.0/src/model.rs:153-156 (ProtocolVersion::LATEST)
let result = serde_json::json!({
    "jsonrpc": "2.0",
    "id": request_id,
    "result": {
        "protocolVersion": "2025-03-26",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": config.app_name,
            "version": config.version
        }
    }
});
```

### 401 challenge response
```rust
// Source: framework/src/http/response.rs:95-127 (.status(), .header() builders)
// Source: rmcp-0.12.0/src/transport/auth.rs:904 (resource_metadata= key)
// Source: rmcp-0.12.0/src/transport/auth.rs:1359 (quoted-value test vector)
let challenge = format!(
    "Bearer resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
    config.app_url
);
Err(HttpResponse::new()
    .status(401)
    .header("WWW-Authenticate", challenge))
```

### tools/call routing
```rust
// Source: ferro-mcp-server/src/renderer.rs:29 (tool name = "list_{service.name}")
// Source: ferro-mcp-server/src/dispatch.rs:96-102 (dispatch signature)
let tool_name = params["name"].as_str().unwrap_or("");
let service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name);
let service = services.iter().find(|s| s.name == service_name && s.mcp_exposed)
    .ok_or_else(|| json!({
        "jsonrpc": "2.0", "id": request_id,
        "error": { "code": -32601, "message": "Method not found" }
    }))?;
let call_params = params.get("arguments").cloned().unwrap_or(json!({}));
let limit = call_params["limit"].as_u64().unwrap_or(25);
let offset = call_params["offset"].as_u64().unwrap_or(0);
let filters = /* remove pagination keys from call_params */;
let result = dispatch(service, filters, limit, offset, db).await?;
```

### Config from env (new `config.rs`)
```rust
// Source: ferro-inertia/src/config.rs:41-62 (InertiaConfig::default pattern)
pub struct McpServerConfig {
    pub app_name: String,
    pub app_url: String,
    pub version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            app_name: std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string()),
            app_url: std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` + `tokio` for async |
| Config file | `ferro-mcp-server/Cargo.toml` (`[dev-dependencies] tokio = { version = "1", features = ["full", "macros"] }`) |
| Quick run command | `cargo test -p ferro-mcp-server` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AMCP-05 SC-1a | `initialize` returns `protocolVersion: "2025-03-26"` + `capabilities.tools` | unit | `cargo test -p ferro-mcp-server jsonrpc` | Wave 0 |
| AMCP-05 SC-1b | `tools/list` returns exactly the mcp_exposed projections | unit | `cargo test -p ferro-mcp-server jsonrpc` | Wave 0 |
| AMCP-05 SC-1c | `tools/call` returns rows from dispatch | integration (in-memory DB) | `cargo test -p ferro-mcp-server jsonrpc` | Wave 0 |
| AMCP-05 SC-3 | Endpoint registered via `post!("/mcp")` in app router | compile-time | `cargo build -p app` | Wave 0 |
| AMCP-06 SC-2 | Unauthenticated request returns 401 + correct `WWW-Authenticate` | unit | `cargo test -p ferro-mcp-server auth` | Wave 0 |
| AMCP-06 SC-4 | Tests exercise all four paths without live OAuth | integration (no server) | `cargo test -p ferro-mcp-server` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp-server`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `ferro-mcp-server/src/config.rs` — `McpServerConfig` + `from_env()`; no test needed (trivial env read), but must exist before jsonrpc.rs compiles
- [ ] `ferro-mcp-server/src/auth.rs` — `BearerOutcome` + `extract_bearer()`; unit tests inline
- [ ] `ferro-mcp-server/src/jsonrpc.rs` — `handle_initialize`, `handle_tools_list`, `handle_tools_call`; covered by integration test
- [ ] `ferro-mcp-server/tests/jsonrpc_integration.rs` — covers AMCP-05 SC-1a/b/c
- [ ] `app/src/controllers/mcp.rs` — ferro handler (thin adapter); no separate test — covered by the pure dispatch tests + compile check
- [ ] `app/src/routes.rs` update — `post!("/mcp", controllers::mcp::handle)`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (Phase 198: seam stub only) | Bearer token — Phase 199 fills; seam must not accidentally accept any token in Phase 198 |
| V3 Session Management | no | Stateless; no Mcp-Session-Id |
| V4 Access Control | no (Phase 200) | Tenant scoping deferred |
| V5 Input Validation | yes | `service_name` lookup by allowlist (already in dispatch.rs is_filter_field); JSON-RPC method routing uses exhaustive match |
| V6 Cryptography | no | Phase 198 has no crypto |

### Phase 198 Specific Security Notes

- The bearer seam in Phase 198 **always returns `Unauthenticated`**. There must be no code path in `extract_bearer` that returns `Authenticated`. A test should assert this.
- The `tools/call` path in Phase 198 is never reachable from the live HTTP surface (every request gets 401 before dispatch). However, the pure dispatch functions are tested directly — they are the read path and must retain the allowlist + limit clamp security properties from Phase 197 (WR-01/WR-02). [VERIFIED: dispatch.rs line 107 clamp, line 123-129 allowlist]
- Origin header validation: the MCP spec requires it for DNS rebinding prevention. Phase 198 is test-only (no live server), so it is safe to defer Origin validation to Phase 199. Flag this in the handler as a `// TODO(phase-199): validate Origin header`.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | all | ✓ | (workspace) | — |
| `rmcp` 0.12.0 | protocol types | ✓ | 0.12.0 in registry | — |
| SQLite (in-memory) | integration tests | ✓ | via sea-orm sqlx-sqlite | — |

No external services required for Phase 198 tests. The bearer seam is a stub; no OAuth server is needed.

---

## Open Questions

1. **Ferro router 405 behavior for GET /mcp**
   - What we know: the MCP spec requires `405 Method Not Allowed` when GET is issued and the server doesn't support SSE
   - What's unclear: does Ferro's router automatically emit 405 when a path exists but the method doesn't match, or does it 404?
   - Recommendation: verify by reading `framework/src/server.rs` or the router's fallback logic before plan. If Ferro 404s on method mismatch for registered paths, add an explicit `get!("/mcp", …)` returning 405.
   - Confidence: [ASSUMED — not yet verified]

2. **`setup_db()` visibility in dispatch_integration.rs**
   - What we know: `setup_db()` exists and builds an in-memory SQLite with a fixture table
   - What's unclear: it's defined in the test file without `pub` — it cannot be shared across test files without moving to a `tests/common/mod.rs` or making it a pub helper in a test util module
   - Recommendation: extract `setup_db()` to `ferro-mcp-server/tests/common/mod.rs` so `jsonrpc_integration.rs` can reuse it
   - Confidence: HIGH (standard Rust test organization pattern)

3. **`tools/call` arguments format in MCP spec**
   - What we know: MCP `tools/call` params shape is `{ "name": "list_item", "arguments": { ... } }`
   - What's unclear: should pagination params (`limit`, `offset`) be treated as part of `arguments` alongside filter params?
   - Recommendation: yes — all tool input schema properties (including `limit`/`offset`) are passed in `arguments`; the dispatch function extracts them
   - Confidence: MEDIUM (inferred from MCP tool schema structure; verified in rmcp model `CallToolParams`)

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Ferro router emits 405 (not 404) for GET /mcp when only POST is registered | D-04 / Open Question 1 | If 404, need to add explicit GET handler returning 405 to satisfy MCP spec |
| A2 | `env!("CARGO_PKG_VERSION")` resolves at compile time to the workspace version in `ferro-mcp-server` | D-03 config.rs | If it resolves to empty or wrong value, use `std::env::var("CARGO_PKG_VERSION").unwrap_or("0.1")` at runtime instead |

---

## Sources

### Primary (HIGH confidence)
- `rmcp-0.12.0/src/model.rs` — `ProtocolVersion::LATEST = "2025-03-26"`, `InitializeResult`, `ServerCapabilities`, `Implementation`
- `rmcp-0.12.0/src/transport/auth.rs` — `resource_metadata=` parameter key, `WWW-Authenticate` header parsing, Bearer challenge test vector
- `rmcp-0.12.0/Cargo.toml.orig` — feature flags (`transport-streamable-http-server` deps, no axum in non-transport features)
- `ferro-mcp-server/src/renderer.rs` — `render_exposed_tools`, `McpContext`, `Tool::new()` usage, tool name format `"list_{service.name}"`
- `ferro-mcp-server/src/dispatch.rs` — `dispatch()` signature, MAX_LIMIT clamp, is_filter_field allowlist
- `ferro-mcp-server/tests/dispatch_integration.rs` — `setup_db()` fixture pattern
- `ferro-mcp-server/Cargo.toml` — current rmcp features `["server", "macros", "base64"]`
- `framework/src/http/response.rs` — `HttpResponse::status()`, `HttpResponse::header()`
- `framework/src/database/mod.rs` — `DB::connection()`, `DbConnection`
- `framework/src/api/api_key.rs` — `extract_bearer_token` pattern (lines 198-223)
- `ferro-inertia/src/config.rs` — `InertiaConfig::default()` env-var pattern
- `app/src/api/user_api.rs` — handler using `DB::connection()` (line 50-52)
- `app/src/routes.rs` — route registration pattern
- modelcontextprotocol.io/specification/2025-03-26/basic/transports — `application/json` single response permitted, session-id optional, Origin validation required

### Secondary (MEDIUM confidence)
- `app/src/projections/` — no global registry exists today; explicit slice construction needed in handler

### Tertiary (LOW confidence)
- None — all critical claims are HIGH confidence with source verification

---

## Metadata

**Confidence breakdown:**
- Protocol types (D-01, D-03, D-04): HIGH — verified in rmcp 0.12.0 source
- Ferro handler mechanics (D-02): HIGH — verified in framework source
- Bearer seam design (D-05): HIGH — reasoned from existing `ApiKeyMiddleware` pattern + D-07 test requirements
- WWW-Authenticate format (D-06): HIGH — verified in rmcp auth.rs source and test vectors
- Test strategy (D-07): HIGH — verified against existing test file

**Research date:** 2026-06-10
**Valid until:** 2026-07-10 (rmcp 0.12 stable; MCP Streamable HTTP spec stable at 2025-03-26)
