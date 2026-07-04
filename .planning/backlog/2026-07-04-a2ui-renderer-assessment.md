# A2UI renderer assessment (ferro-a2ui)

**Captured:** 2026-07-04
**Trigger conditions:** A2UI reaches 1.0 (v0.9.1 today; 1.0 candidate adds
client→server RPC) AND at least one mainstream A2UI client surface is broadly
available to end users.

## What

A2UI (a2ui.org, Apache 2.0, Google + CopilotKit) is an open protocol for
agent-generated declarative UI: flat streaming JSON component model with
adjacency lists, client-declared component catalogs, data binding, and actions
streamed back to the agent. It runs over A2A and MCP.

## Why it maps onto ferro cleanly

- Its component model (flat map + ID adjacency + catalog + declarative-only
  security) is structurally equivalent to ferro-json-ui's spec shape —
  independent convergence on the same design.
- The `Renderer` trait is modality-agnostic; A2UI is a natural additional
  output crate (`ferro-a2ui`), alongside ferro-json-ui (HTML), ferro-text, and
  the MCP renderer, projecting the same `ServiceDef`/specs.
- Ferro apps already expose a per-tenant consumer MCP endpoint; A2UI-over-MCP
  would let those endpoints return interactive surfaces to A2UI clients
  instead of JSON text.

## Scope sketch (when triggered)

1. Research spike: map json-ui elements → A2UI components; identify the lossy
   edges (layouts, design meta, guards are server concerns and stay home).
2. `ferro-a2ui` output crate implementing `Renderer` for A2UI messages.
3. Wire through the consumer MCP endpoint (content negotiation or dedicated
   tool).
4. Design-system story: ferro's lint/tokens govern what gets emitted even when
   the client renders — the enforcement layer stays server-side.

## Near-term inspiration (independent of the trigger)

- Progressive/streaming spec rendering — the flat element map already permits
  incremental emit.
- Catalog capability negotiation — client declares supported components;
  relevant to embedded rendering contexts.

## Explicit non-goals

- Replacing the json-ui wire format or server-side HTML rendering.
- Tracking a pre-1.0 external spec with production code.
