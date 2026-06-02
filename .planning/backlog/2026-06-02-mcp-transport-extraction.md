# Feedback: extract `ferro-mcp-transport` from `ferro-mcp` introspection catalog

**Source:** Downstream Inertia + WebSocket streaming consumer app (private, AI-native chat product), field assessment 2026-06-02
**Severity:** Architectural split — moderate scope, high leverage for any Ferro app that wants to ship its own MCP surface
**Ferro version inspected:** ferro-mcp HEAD as of 2026-06-02 (paired with the broadcast-backlog feedback also filed 2026-06-02)

## Planning Note

This document is a sketch from a downstream-app perspective, not an inside-Ferro design. When promoted from backlog to a phase, the Ferro planning agent should reconcile against `.planning/VISION.md` and existing conventions before drafting `PLAN.md`.

---

## Problem statement

`ferro-mcp` today does two things in one crate:

1. **Transport + machinery** — the streamable-HTTP route, JSON-RPC request/response handling, schema generation from `#[mcp_tool]` macros (or whatever the equivalent is), error envelope plumbing.
2. **Developer-introspection tool catalog** — `application_info`, `list_routes`, `list_models`, `db_schema`, `last_error`, `read_logs`, `list_services`, `list_jobs`, etc. These are the tools an *agent helping the developer build the app* should be able to call.

The downstream app needs to expose **its own** MCP tool catalog (per-tenant vault operations: `search`, `get_note`, `create_note`, `move_note`, `get_backlinks`, `vault_status`, etc.) to authenticated external agents (the user's own Claude Code, Cursor, etc.) over an authenticated streamable-HTTP endpoint.

It very much does NOT want to expose `application_info`, `db_schema`, `last_error`, `read_logs` etc. to those external agents. Even with an auth layer in front, the *existence* of those tools in the published catalog leaks framework structure. Worse: every time Ferro adds a new introspection tool (which it should — that's `ferro-mcp`'s value to the developer), it would auto-appear on the downstream app's public surface without a planning review.

That's the shape of accidental zero-day exposure surface.

## Why this is a framework concern, not a downstream concern

The downstream app could solve this by writing its own MCP transport from scratch (axum route, JSON-RPC dispatch, schema generation, error envelope, streamable-HTTP). But that's exactly the plumbing that should not be hand-rolled per app. It's modality-agnostic infrastructure. It belongs in the framework.

What's per-app is the **tool catalog** — *which tools exist, what shapes they have, how they map to the app's domain primitives*. That's the projection/intent shape Ferro applies elsewhere; the framework owns the transport, the app owns the catalog.

## Proposed split

```
ferro-mcp                          (current crate — kept for backwards compat)
├── re-exports ferro-mcp-transport (transport primitives)
└── re-exports ferro-mcp-introspection (developer catalog)

ferro-mcp-transport                (NEW — extracted)
├── streamable-HTTP route handler  (#[mcp_endpoint] or App::mcp_route())
├── JSON-RPC request/response machinery
├── #[mcp_tool] macro              (schema generation, dispatch wiring)
├── auth middleware plug-points    (bring-your-own auth/scope)
├── error envelope contracts
└── (no tools — pure plumbing)

ferro-mcp-introspection            (NEW — extracted from current ferro-mcp)
├── application_info
├── list_routes / list_models / list_services / list_jobs
├── db_schema / last_error / read_logs
└── ... (the existing developer-facing introspection catalog)
```

A downstream app exposing its own MCP surface then does:

```rust
use ferro_mcp_transport::{App, mcp_tool, McpRouter};

#[mcp_tool(name = "search", scope = "read")]
async fn search(tenant: TenantContext, query: String) -> Result<Vec<SearchHit>> { ... }

#[mcp_tool(name = "create_note", scope = "write")]
async fn create_note(tenant: TenantContext, note: NoteSpec) -> Result<NoteRef> { ... }

App::new()
    .with_mcp(McpRouter::new()
        .auth(my_auth_middleware)         // resolves bearer → TenantContext + scope
        .tools(my_app_tool_catalog())     // the kb_tools module
        .mount_at("/mcp"))
    .run().await
```

No introspection tools. The catalog is fully app-owned. The framework provides transport + machinery, the app provides intent.

## Acceptance criteria

- New crate `ferro-mcp-transport` with the transport/machinery primitives extracted out of current `ferro-mcp`
- New crate `ferro-mcp-introspection` with the developer-catalog tools extracted out of current `ferro-mcp`
- Existing `ferro-mcp` crate retains its current public API surface via re-exports (zero-breakage for existing users)
- `#[mcp_tool]` macro (or equivalent) lives in `ferro-mcp-transport` and works for any user-defined tool function
- Auth middleware plug-points are documented: tool dispatch receives a `TenantContext`/scope inferred from middleware, not hard-coded to introspection
- Documentation: an "Embedding your own MCP tool catalog" guide in ferro-mcp-transport's README with a complete end-to-end example
- A reference embedding test: a tiny app under `app/` (or examples) that exposes a `hello_world` tool over the transport, NOT inheriting introspection
- Existing `ferro mcp` CLI subcommand continues to work for developers (it now activates `ferro-mcp-introspection` explicitly)

## Why the downstream app cares

The downstream app is Phase-6-bound on this split. Without it, the app has two equally bad options:

- **Option A:** Embed `ferro-mcp` as-is and filter the tool list via a middleware whitelist before exposing. The framework-introspection tools are still defined, just hidden. Maintenance risk: every Ferro upgrade adds new introspection tools the app has to remember to whitelist out.
- **Option B:** Hand-roll the entire MCP transport in the app. Defeats the whole point of using Ferro for production MCP work; loses every future ferro-mcp-transport improvement.

The split gives a clean third option: embed the transport, define a catalog, ship.

## Why this is a Ferro win, not just a downstream win

The Ferro vision (CLAUDE.md) explicitly calls for compressive abstractions: "compressive (projection / intent) → operational → conceptual → aesthetic." This split IS that compression — transport is modality-agnostic plumbing; catalog is per-app intent. Every Ferro app that ever wants to expose its own MCP surface (which, in an MCP-native ecosystem, will be most production apps) benefits from this split landing.

Concretely:
- The introspection-only `ferro mcp` CLI keeps doing its current job for developers
- Any production Ferro app can now expose a focused, audited tool catalog without inheriting framework introspection
- The MCP transport machinery (auth plug-points, error envelopes, streamable HTTP wire shape, schema generation) gets to evolve in one place and benefit every consumer

## Source / provenance

Sibling to the broadcast-backlog feedback also filed 2026-06-02. Both surface from the same Phase 0 / Phase 6 design work in the downstream app.

Filed by the downstream app per its dogfooding discipline rule.
