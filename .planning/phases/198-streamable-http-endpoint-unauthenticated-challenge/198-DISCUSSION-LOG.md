# Phase 198: Streamable HTTP Endpoint + Unauthenticated Challenge - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-10
**Phase:** 198-streamable-http-endpoint-unauthenticated-challenge
**Mode:** `--auto` (all gray areas selected; recommended defaults chosen)
**Areas discussed:** Transport mechanism, Method-dispatch placement, `initialize` response, Streamable HTTP response mode, Auth seam & 401 timing, `WWW-Authenticate` format, Integration test strategy

---

## Transport mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Ferro `post!("/mcp")` handler, hand-rolled JSON-RPC dispatch | Endpoint inside framework HTTP layer + middleware stack | ✓ |
| rmcp `transport-streamable-http-server` (axum service) | Mount rmcp's own server; bypasses ferro middleware/tenant/auth seams | |

**User's choice:** Ferro handler (auto default).
**Notes:** SC-3 mandates the same middleware stack as other framework routes; rmcp's axum
service would bypass the Phase 199 bearer seam and Phase 200 tenant context. `HttpResponse`
natively supports status + headers.

---

## Method-dispatch placement

| Option | Description | Selected |
|--------|-------------|----------|
| Pure dispatch in `ferro-mcp-server`; thin HTTP adapter in ferro handler | Keeps `ferro-mcp-server → ferro-projections` only; no `framework` dep | ✓ |
| All logic in the ferro handler (app/framework) | Couples protocol logic to HTTP types | |

**User's choice:** Pure dispatch in crate, thin adapter (auto default).
**Notes:** RESEARCH FLAG — reusable `framework`-exported route vs app-local skeleton, gated on
the `framework → ferro-mcp-server`/`rmcp` dependency weight.

---

## `initialize` response

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: protocolVersion + `capabilities.tools={}` + serverInfo from APP_NAME/APP_URL | Spec-compliant, project-agnostic | ✓ |
| Hardcoded serverInfo | Violates project-agnostic crate rule | |

**User's choice:** Minimal, env-sourced serverInfo (auto default).
**Notes:** RESEARCH FLAG — confirm exact rmcp 0.12 protocolVersion string.

---

## Streamable HTTP response mode

| Option | Description | Selected |
|--------|-------------|----------|
| Single `application/json` JSON-RPC response (stateless, no session id) | Minimal spec-compliant skeleton | ✓ |
| Full SSE streaming + `Mcp-Session-Id` | Needed only for server-initiated messages | |

**User's choice:** Single JSON response (auto default).
**Notes:** RESEARCH FLAG — confirm standard client + rmcp client accept stateless JSON-only
server (Accept negotiation, omitted session id).

---

## Auth seam & 401 timing

| Option | Description | Selected |
|--------|-------------|----------|
| Bearer-extraction seam; no valid path in 198 → all live requests 401; tests drive pure dispatch | Resolves SC-1 vs SC-2 tension; Phase 199 fills seam | ✓ |
| Temporary "allow all" mode | Leaves no challenge surface; contradicts SC-2 | |

**User's choice:** Bearer seam + 401, tests bypass via pure dispatch (auto default).
**Notes:** Method logic is real and tested; live HTTP surface challenges until Phase 199
supplies tokens. Handler signature stable across 199.

---

## `WWW-Authenticate` format

| Option | Description | Selected |
|--------|-------------|----------|
| `Bearer resource_metadata="{APP_URL}/.well-known/oauth-protected-resource"` | RFC 9728 + RFC 6750; URL built in Phase 199 | ✓ |
| Bare `Bearer` | No discovery pointer; client can't follow | |

**User's choice:** RFC 9728 resource_metadata form (auto default).
**Notes:** RESEARCH FLAG — confirm the exact parameter MCP clients follow and whether a 401
body is expected.

---

## Integration test strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Pure-dispatch tests w/ `fresh_db()` SQLite + handler-level 401 test; no live OAuth | Reuses Phase 197 fixture pattern | ✓ |
| Live web-server / OAuth-backed e2e | Out of scope; SC-4 forbids requiring OAuth server | |

**User's choice:** Fixture-driven + handler 401 test (auto default).

## Claude's Discretion

- Module layout within `ferro-mcp-server` and bearer-seam type naming.
- JSON-RPC error-code mapping (standard `-32600`/`-32601`/`-32602`).
- `401` body shape (empty vs JSON-RPC error), pending D-06 research.

## Deferred Ideas

- SSE streaming / session management (future, if server-initiated messages needed).
- Bearer validation, `.well-known` docs, DCR, `/authorize`+`/token` (Phase 199).
- Per-tenant scoping + policy on `tools/call` (Phase 200).
