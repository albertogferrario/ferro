# Phase 198: Streamable HTTP Endpoint + Unauthenticated Challenge - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen and logged inline)

<domain>
## Phase Boundary

The application server mounts an MCP endpoint over **Streamable HTTP** that handles the
`initialize`, `tools/list`, and `tools/call` JSON-RPC methods. `tools/list` is backed by
Phase 197's `render_exposed_tools`; `tools/call` is backed by Phase 197's `dispatch`. An
**unauthenticated** request returns HTTP `401` with a `WWW-Authenticate` header pointing a
standard MCP client at the protected-resource metadata URL.

**In scope:** transport wiring, JSON-RPC method routing, the three methods, the `401`
challenge, integration tests that exercise all four paths without a live OAuth server, and
the auth-extraction *seam* that Phase 199 fills.

**Out of scope (Phase 199+):** real bearer-token validation, the `.well-known` discovery
documents, dynamic client registration, the `/authorize` + `/token` flow, per-tenant
scoping, and policy authorization. This phase only stubs the seam where those plug in.

**Carrying forward from Phase 197:**
- `ferro-mcp-server` is the output crate; dependency direction is
  `ferro-mcp-server → ferro-projections` only (197 D-01).
- Reuse the workspace's `rmcp` 0.12 for protocol types; emit `serde_json::Value` where
  rmcp's server-runtime coupling is awkward for pure emission (197 D-03).
- `render_exposed_tools(services, &McpContext) -> Vec<Tool>` already produces the
  `tools/list` payload (197 renderer.rs).
- `dispatch(service, filters, limit, offset, db) -> DispatchResult` already executes the
  read path for `tools/call`, with the tenant/ownership filter explicitly deferred to
  Phase 200 (197 D-05, dispatch.rs).

</domain>

<decisions>
## Implementation Decisions

### Transport mechanism (D-01) — AMCP-05, SC-1, SC-3
- **D-01:** Implement the endpoint as a **ferro `post!("/mcp", …)` handler** that hand-rolls
  JSON-RPC method dispatch over the framework's own HTTP layer — **not** rmcp's
  `transport-streamable-http-server` axum service. The handler parses the JSON-RPC request
  body, routes on `method`, and returns a JSON-RPC response built with `serde_json::Value`
  (reusing rmcp's `Tool` and protocol *types* for the payload shapes, per 197 D-03).
  - **[auto] recommended default** — chosen over (b) mounting rmcp's `StreamableHttpService`.
    Rationale: SC-3 requires the endpoint to "integrate via the **same middleware stack** as
    other framework routes." rmcp's axum service runs its own request pipeline and would
    bypass ferro's middleware, the session/tenant context (Phase 200), and the bearer seam
    (Phase 199). A ferro handler keeps the endpoint inside the framework's auth + tenant
    seams that the next two phases must hook. `HttpResponse` already supports
    `.status(401)` + `.header(…)`, so the `401` challenge is native.

### Method-dispatch placement (D-02) — SC-1, dependency-direction continuity
- **D-02:** The **pure JSON-RPC method dispatch** (`initialize` / `tools/list` / `tools/call`
  routing, request parsing, response shaping) lives in `ferro-mcp-server` as
  framework-agnostic functions that take parsed input + `&[ServiceDef]` + a DB connection and
  return a JSON-RPC `serde_json::Value`. The **thin HTTP adapter** (read body, emit status +
  headers, run the bearer seam) is a ferro handler. This preserves
  `ferro-mcp-server → ferro-projections` only and keeps `ferro-mcp-server` free of a
  `framework` dependency.
  - **[auto] recommended default.**
  - **RESEARCH FLAG (load-bearing):** decide where the ferro HTTP handler lives so consumer
    apps don't each reimplement it. Preferred: a **reusable mountable route exported from
    `framework`** (e.g. `ferro::mcp_endpoint()`), delegating to `ferro-mcp-server`'s pure
    dispatch — so every consumer app gets `POST /mcp` for free, matching "app-served." Fallback
    if a `framework → ferro-mcp-server`/`rmcp` dependency is judged too heavy for the core
    crate: wire the handler in the sample `app` for the 198 skeleton and promote it to
    `framework` in a later phase. Record the dependency-weight finding either way.

### `initialize` response (D-03) — AMCP-05, SC-1
- **D-03:** `initialize` returns a minimal spec-compliant result: `protocolVersion` matching
  the MCP version rmcp 0.12 negotiates, `capabilities: { tools: {} }` (no `listChanged`),
  and `serverInfo { name, version }`. `name`/`url` are sourced from the **`APP_NAME` /
  `APP_URL` framework conventions** (per CLAUDE.md project-agnostic rule), never hardcoded;
  `ferro-mcp-server` reads them via its own `from_env()` config struct mirroring
  `InertiaConfig::app_name`.
  - **[auto] recommended default.**
  - **RESEARCH FLAG:** confirm the exact `protocolVersion` string rmcp 0.12 expects/advertises
    so a standard client's `initialize` succeeds.

### Streamable HTTP response mode (D-04) — AMCP-05, SC-1
- **D-04:** Respond with a **single `application/json` JSON-RPC response** for all three
  methods (no SSE stream, stateless — no `Mcp-Session-Id` requirement for the skeleton). The
  Streamable HTTP spec permits a non-streaming JSON reply; the projection read path is
  synchronous and read-only, so streaming buys nothing here.
  - **[auto] recommended default** — chosen over full SSE streaming (defer until a method
    needs server-initiated messages).
  - **RESEARCH FLAG:** confirm a standard MCP client (and rmcp's own client) accepts a
    stateless JSON-only Streamable HTTP server — specifically the `Accept` header negotiation
    (`application/json, text/event-stream`) and whether omitting `Mcp-Session-Id` is tolerated.

### Auth seam + when `401` fires (D-05) — AMCP-06, SC-2, SC-4
- **D-05:** Introduce a **bearer-extraction seam** — a single function/trait the handler calls
  to resolve a request to an authenticated principal. In Phase 198 the seam has **no
  valid-token path**: any request that does not carry a recognized bearer returns `401` +
  `WWW-Authenticate`. Phase 199 fills this seam with real PKCE/bearer validation **without
  changing the handler signature**. Integration tests for the three JSON-RPC methods drive
  the **pure dispatch directly** (D-02) — or inject a test principal — so SC-1/SC-4 hold
  with no live OAuth server, while a separate handler-level test asserts the `401` path.
  - **[auto] recommended default** — resolves the apparent tension between SC-1 ("handles the
    three methods") and SC-2 ("unauthenticated returns 401"): the *method logic* is real and
    tested; the *live HTTP surface* challenges until Phase 199 supplies tokens.

### `WWW-Authenticate` header format (D-06) — AMCP-06, SC-2
- **D-06:** Emit `WWW-Authenticate: Bearer resource_metadata="{APP_URL}/.well-known/oauth-protected-resource"`
  (RFC 9728 protected-resource-metadata discovery, RFC 6750 Bearer scheme). The referenced
  URL is the document Phase 199 builds; Phase 198 only points at it. `{APP_URL}` comes from
  the framework convention (D-03), not a literal.
  - **[auto] recommended default.**
  - **RESEARCH FLAG:** confirm the exact `WWW-Authenticate` parameter MCP clients follow
    (`resource_metadata` per RFC 9728) and whether a `401` body (JSON-RPC error vs empty) is
    expected alongside the header.

### Integration test strategy (D-07) — SC-4
- **D-07:** Tests live alongside Phase 197's pattern. Drive the pure dispatch (D-02) with a
  fixture `ServiceDef` + in-memory SQLite, **reusing the `fresh_db()` helper** from
  `ferro-mcp-server/tests/dispatch_integration.rs`, to assert `initialize` (capabilities +
  protocolVersion), `tools/list` (one exposed projection → one tool, schema present), and
  `tools/call` (rows returned). A handler-level test asserts `401` + the `WWW-Authenticate`
  value on a request with no bearer. No live OAuth server, no running web server.
  - **[auto] recommended default.**

### Claude's Discretion
- Exact module layout within `ferro-mcp-server` (e.g. a `jsonrpc.rs` / `protocol.rs` module
  for method dispatch) and naming of the bearer-seam type.
- JSON-RPC error-code mapping for malformed requests / unknown methods (use standard JSON-RPC
  codes: `-32600` invalid request, `-32601` method not found, `-32602` invalid params).
- Whether the `401` response body is an empty body or a JSON-RPC error object (pending the
  D-06 research flag).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & forward-looking seams
- `.planning/ROADMAP.md` §"Phase 198" — goal, success criteria (SC-1…SC-4).
- `.planning/ROADMAP.md` §"Phase 199" / §"Phase 200" — the bearer-validation, `.well-known`,
  tenant-scoping, and policy seams this phase must leave room for (read so the 198 seam shapes
  do not block 199/200).
- `.planning/REQUIREMENTS.md` — AMCP-05, AMCP-06 (this phase); AMCP-07…AMCP-09 (Phase 199,
  for seam shape).

### Phase 197 carry-forward (the read path this phase serves)
- `.planning/phases/197-mcprenderer-ferro-mcp-server/197-CONTEXT.md` — D-03 (rmcp types),
  D-05 (dispatch seam, tenant deferral).
- `ferro-mcp-server/src/renderer.rs` — `render_exposed_tools` (→ `tools/list`), `McpContext`.
- `ferro-mcp-server/src/dispatch.rs` — `dispatch(service, filters, limit, offset, db)`
  (→ `tools/call`), `MAX_LIMIT` clamp, filter allowlist (`is_filter_field`).
- `ferro-mcp-server/tests/dispatch_integration.rs` — `fresh_db()` test fixture pattern (D-07).
- `ferro-mcp-server/Cargo.toml` — current `rmcp` features `["server", "macros", "base64"]`
  (a new HTTP/transport feature may be needed; see D-04 research flag).

### Framework integration points
- `app/src/routes.rs` — `routes!` / `post!` / `group!(...).middleware(...)` registration
  pattern; `ApiKeyMiddleware` as the existing auth-challenge analog.
- `framework/src/http/response.rs` — `HttpResponse::status(u16)` + `header(name, value)`
  (the `401` + `WWW-Authenticate` mechanism).
- `framework/src/routing/group.rs`, `framework/src/routing/mod.rs` — `Router` / `GroupRouter`
  for a reusable mountable route (D-02 research flag).
- `framework/src/auth/middleware.rs`, `framework/src/auth/extract.rs` — existing
  auth-extraction patterns to mirror for the bearer seam (D-05).
- `app/src/main.rs` (`run_server`, `Server::from_config(router).run()`) — server bootstrap.

### External specs (no repo file — read upstream)
- MCP **Streamable HTTP** transport spec (modelcontextprotocol.io) — request/response,
  `Accept` negotiation, optional `Mcp-Session-Id` (D-04).
- **RFC 9728** OAuth 2.0 Protected Resource Metadata — `WWW-Authenticate: ... resource_metadata`
  parameter (D-06).
- **RFC 6750** OAuth 2.0 Bearer Token Usage — `Bearer` challenge scheme (D-06).
- `rmcp` 0.12 docs — `Tool` type, negotiated `protocolVersion`, and whether
  `transport-streamable-http-server` is needed or protocol types alone suffice (D-01, D-03, D-04).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-mcp-server::render_exposed_tools` — produces the `tools/list` tool array directly.
- `ferro-mcp-server::dispatch` — executes `tools/call` read path with limit clamp + filter
  allowlist already enforced (security seam from 197 WR-01/WR-02).
- `fresh_db()` in `dispatch_integration.rs` — in-memory SQLite fixture for tests (D-07).
- `HttpResponse::status` + `HttpResponse::header` — native `401` + `WWW-Authenticate` emission.
- `ApiKeyMiddleware` (`app/src/api/routes.rs`) — closest analog for a request-gating middleware
  that challenges on missing credentials.

### Established Patterns
- Routes register via the `routes!` macro with `post!(...)` and `.middleware(...)`; groups
  carry shared middleware — the `/mcp` route slots into this same surface (SC-3).
- `ferro-*` crates read app identity from `APP_NAME` / `APP_URL` via a `from_env()` config
  struct (mirror `InertiaConfig::app_name`) — `serverInfo` and the metadata URL use this.
- rmcp 0.12 already in the workspace; ferro-mcp serves over **stdio** today — this phase adds
  the **HTTP** transport surface, the first non-stdio MCP server in the workspace.

### Integration Points
- New `POST /mcp` route mounted in the application router (reusable from `framework`
  preferred; app-local fallback — D-02 research flag).
- Bearer-extraction seam consumed by the handler, filled by Phase 199 middleware.
- DB connection + `&[ServiceDef]` (the exposed projections) handed to the pure dispatch at
  request time — same connection/context the web surface uses.

</code_context>

<specifics>
## Specific Ideas

- The endpoint is the **transport half** of "MCP as a projection/intent renderer" — the
  killer feature is the per-tenant projection-derived toolset reachable over HTTP. Phase 198
  delivers the pipe and the unauthenticated challenge; the value lands when 199/200 attach
  identity and tenancy. Keep the seam shapes (bearer extraction, DB/tenant context) clean so
  those phases plug in without reshaping the handler.
- Stateless JSON-only Streamable HTTP is the deliberately minimal skeleton; SSE and session
  IDs are future work only if a method needs server-initiated messages.

</specifics>

<deferred>
## Deferred Ideas

- **SSE streaming / `Mcp-Session-Id` session management** — only if a future method needs
  server-initiated messages (not required by read-only projection tools).
- **Real bearer-token validation, `.well-known` discovery docs, DCR, `/authorize` + `/token`**
  — Phase 199 (AMCP-07…AMCP-09).
- **Per-tenant scoping + policy authorization on `tools/call`** — Phase 200 (AMCP-10/11);
  `dispatch` already leaves the tenant filter seam open.

None of these belong in Phase 198 — discussion stayed within scope.

</deferred>

---

*Phase: 198-streamable-http-endpoint-unauthenticated-challenge*
*Context gathered: 2026-06-10*
