# Consumer App MCP Endpoint with Browser Login — Design

**Date:** 2026-06-10
**Status:** Draft (pre-1.0, experimental)
**Scope:** Walking skeleton — first end-to-end slice of a larger capability.

## Context

Ferro applications are built around the projection / intent abstraction. A
projection maps a data model to an intent (Browse, Collect, Process, …), and a
`Renderer` turns that projection into an output for a given modality. The
`Renderer` trait is modality-agnostic by design: `JsonUiRenderer` (in
`ferro-json-ui`) renders projections to a visual JSON-UI spec.

Today there is no way for an autonomous agent to consume a *running* ferro
application directly. The available paths are:

- `ferro-mcp` — a development-time introspection server (in-process via
  `ferro mcp`), for an agent authoring a project.
- `ferro-api-mcp` — a standalone bridge that exposes an OpenAPI spec as MCP
  tools, authenticated with a static API key.

Neither lets a deployed application present its own projections to an agent as
MCP tools, authenticated as a specific user within a specific tenant. This
design adds that capability as a second, non-visual rendering target for the
projection / intent system: an MCP endpoint served by the application itself.

## Goal (walking skeleton)

A deployed ferro application serves an OAuth-protected MCP endpoint. A consumer
MCP client discovers the endpoint, completes a browser-based login that reuses
the application's existing authentication, receives an access token bound to a
specific `(user, tenant)`, and calls a single read tool rendered from a single
opt-in projection. The tool returns that tenant's data, gated by the
application's existing authorization policies.

The slice exercises, once and end to end:

1. MCP transport served by the application.
2. Browser-based OAuth login and token issuance.
3. A projection rendered as an MCP tool.
4. Per-tenant scoping and policy enforcement on the tool call.

## Non-goals (deferred to follow-on specs)

- Write intents (Collect / Process rendered as create/submit tools).
- Automatic exposure of all projections.
- An MCP-specific permission model separate from existing policies.
- Rate limiting beyond what the application already applies.
- Refresh-token rotation and long-session ergonomics.

## Architecture

### New crate: `ferro-mcp-server` (output crate)

Holds the MCP rendering target for projections, mirroring how `ferro-json-ui`
holds the visual rendering target. `ferro-projections` remains renderer-free.

- `McpRenderer` implements `Renderer`. It maps an opt-in projection's
  `ServiceDef` to one MCP tool definition:
  - tool name derived from the projection,
  - description derived from the projection's intent,
  - `inputSchema` (JSON Schema) derived from the projection's filter and
    pagination parameters.
  - For the skeleton, a Browse-intent projection maps to a `list_<entity>` read
    tool.
- Tool dispatch executes the projection's existing read path within the caller's
  tenant and policy context, then serializes the result rows into MCP tool
  content. No query or authorization logic is reimplemented in the renderer.

### Framework HTTP additions

The application server mounts the transport and OAuth endpoints. This is
application-server infrastructure, so it lives in `framework`, not in the
renderer crate.

- **Transport:** `POST /mcp`, Streamable HTTP. JSON-RPC methods: `initialize`,
  `tools/list`, `tools/call`.
- **OAuth authorization + resource server:**
  - `GET /.well-known/oauth-protected-resource` — resource metadata pointing to
    the authorization server.
  - `GET /.well-known/oauth-authorization-server` — authorization-server
    metadata.
  - `POST /register` — dynamic client registration (RFC 7591).
  - `GET /authorize` — authorization-code flow with PKCE. If no active session,
    redirect to the application's existing login; then present a consent screen;
    then issue an authorization code.
  - `POST /token` — exchange code for an access token. The token is bound to
    `(user, tenant)` and audience-restricted to this endpoint.
  - Bearer-token validation middleware on `/mcp`, including audience
    verification (RFC 8707).

### Opt-in exposure marker

A projection declares MCP exposure explicitly (a read-only `mcp_exposed`
marker). Only marked projections appear in `tools/list`. One projection is
marked for the skeleton.

### Tenant and policy reuse

The access token carries `(user, tenant)`. The `/mcp` middleware establishes the
same tenant context the existing multi-tenant middleware uses and runs the same
authorization policies. An agent can do exactly what the authenticated user
could do through the UI — no second permission system, no parallel control
surface.

## Data flow

1. The client requests `/mcp` without a token and receives `401` with a
   `WWW-Authenticate` header referencing the protected-resource metadata.
2. The client fetches the `.well-known` metadata, discovers the authorization
   server, dynamically registers, and builds a PKCE authorization URL.
3. The client opens the browser to `/authorize`. The user authenticates through
   the existing login, then approves a consent screen scoped to the requesting
   client and tenant. The flow redirects back with an authorization code.
4. The client exchanges the code at `/token` for an access token bound to
   `(user, tenant)` with this endpoint as audience.
5. The client calls `/mcp`: `initialize`, then `tools/list` (the one rendered
   read tool), then `tools/call` for `list_<entity>`. The middleware validates
   the token, establishes tenant context, applies policies, and the renderer
   executes the projection read. The tenant's rows are returned.

## Error handling

- Missing, invalid, or expired token: `401` with `WWW-Authenticate` (prompts
  re-authentication).
- Token audience or tenant mismatch: `403`.
- Policy denial: an MCP tool error with a clear message, no data disclosure.
- A projection that is not marked exposed never appears in `tools/list` and is
  therefore not callable.
- Consent declined: standard OAuth `access_denied`.

## Security

- Access tokens are audience-bound (RFC 8707); a token for one application
  cannot be replayed against another.
- PKCE is mandatory; the implicit flow is not supported.
- Tenant isolation reuses the existing multi-tenant middleware; no scoping logic
  is duplicated.
- Authorization reuses existing policies; the agent's reach is bounded by the
  authenticated user's reach.
- The skeleton is read-only; there is no mutation surface.
- Access tokens are short-lived.

## Testing and acceptance

- **Unit:** `McpRenderer` maps a fixture projection to the expected tool schema;
  dispatch returns tenant-scoped rows.
- **Integration:** the full sequence — discovery, registration, authorize (with
  a mocked session), token exchange, `tools/call` — asserting that a token for
  one tenant returns only that tenant's rows, and that a policy denial returns a
  clean error.
- **Dogfood acceptance gate:** a real MCP client completes a browser login
  against a live consumer application and lists one projection's data, scoped to
  the authenticated tenant. Acceptance requires the flow to work end to end; a
  flow that does not is cause to revise the design rather than ship it.

## Future work

- Write intents: Collect and Process projections rendered as create and submit
  tools, with a confirmation step.
- Configurable exposure across multiple projections.
- The projection checkpoint as a guard on what may be exposed.
- Development-time agent experience improvements in `ferro-mcp` (separate
  track).

## References

- MCP authorization specification (OAuth 2.1 resource server, protected-resource
  metadata, dynamic client registration, PKCE, resource indicators).
- `ferro-projections` — `Renderer` trait, `ServiceDef`, `derive_intents`.
- `ferro-json-ui` — `JsonUiRenderer`, the existing visual rendering target.
- Multi-tenant middleware (Phase 95) and the policy layer.
- `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` — prior
  projection-layer design; dogfood acceptance discipline.
