# Requirements: v12.6 Consumer App MCP (Browser Login)

**Milestone goal:** A deployed ferro application serves its own OAuth-protected MCP endpoint so a consumer agent can authenticate through the browser and use the application's projections as per-tenant tools.

**Scope:** Walking skeleton — read-only, one opt-in projection, end to end. The MCP endpoint is a rendering target for the projection / intent system (an `McpRenderer` alongside `JsonUiRenderer`), with OAuth/transport implemented to the MCP authorization specification as supporting infrastructure. Design spec: `docs/superpowers/specs/2026-06-10-consumer-app-mcp-browser-login-design.md`.

**Acceptance:** the final phase carries a dogfood GO/NO-GO success criterion — a real MCP client completes a browser login against a live consumer application and lists one projection's data scoped to the authenticated tenant. A run that does not work end to end is cause to revise the design rather than ship it.

## v12.6 Requirements

### Projection → Tool Rendering
- [x] **AMCP-01**: A projection marked MCP-exposed (read-only, opt-in) appears in the MCP server's `tools/list` as exactly one tool; an unmarked projection never appears.
- [x] **AMCP-02**: The exposed tool's input JSON schema is derived from the projection's `ServiceDef` fields (filter / pagination parameters), not declared separately from the validation applied on the call.
- [ ] **AMCP-03**: Calling the tool runs the projection's existing read path and returns its rows as MCP structured content, with the output shape derived from the projection.
- [x] **AMCP-04**: The `McpRenderer` lives in a new output crate `ferro-mcp-server` implementing the `Renderer` trait; `ferro-projections` gains no renderer dependency.

### Endpoint & Transport
- [ ] **AMCP-05**: The application serves an MCP endpoint over Streamable HTTP supporting `initialize`, `tools/list`, and `tools/call`.

### Browser Authentication (OAuth)
- [ ] **AMCP-06**: An unauthenticated request to the MCP endpoint returns `401` with a `WWW-Authenticate` header referencing the protected-resource metadata.
- [ ] **AMCP-07**: The application publishes OAuth discovery metadata (`.well-known/oauth-protected-resource`, `.well-known/oauth-authorization-server`) and a dynamic client registration endpoint, advertising the authorization-code grant with PKCE (S256).
- [ ] **AMCP-08**: A consumer completes a browser authorization-code + PKCE flow that reuses the application's existing login and a consent step, receiving an access token bound to `(user, tenant)` and audience-restricted to the MCP endpoint.
- [ ] **AMCP-09**: The MCP endpoint validates the bearer token; an invalid or expired token returns `401`, and an audience or tenant mismatch returns `403`.

### Per-Tenant Scoping & Authorization
- [ ] **AMCP-10**: A tool call executes within the token's tenant context via the existing multi-tenant middleware; a token scoped to one tenant returns only that tenant's rows.
- [ ] **AMCP-11**: A tool call is gated by the same policy layer as the web surface; a policy-denied call returns an MCP tool error with no data disclosure.

## Future Requirements (deferred)

- Write intents — Collect / Process projections rendered as create/submit tools, with a confirmation step.
- Automatic exposure of multiple projections (catalog-wide), with per-projection configuration.
- MCP App interactive UI derived from intent templates (Browse→grid, Process→kanban) rather than hand-authored.
- Development-time MCP experience improvements in `ferro-mcp` (Track A: start / use agent experience).
- Refresh-token rotation and long-session ergonomics.

## Out of Scope

- An MCP-specific permission model separate from the existing policy layer — reusing existing policies and multi-tenant middleware is a design invariant, not a gap to fill later.
- Rate limiting beyond what the application already applies.
- A local (stdio) consumer transport for the deployed app — the consumer endpoint is HTTP. (`ferro-mcp` already provides stdio for the development-time surface.)

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| AMCP-01 | Phase 197 | Complete |
| AMCP-02 | Phase 197 | Complete |
| AMCP-03 | Phase 197 | Pending |
| AMCP-04 | Phase 197 | Complete |
| AMCP-05 | Phase 198 | Pending |
| AMCP-06 | Phase 198 | Pending |
| AMCP-07 | Phase 199 | Pending |
| AMCP-08 | Phase 199 | Pending |
| AMCP-09 | Phase 199 | Pending |
| AMCP-10 | Phase 200 | Pending |
| AMCP-11 | Phase 200 | Pending |
