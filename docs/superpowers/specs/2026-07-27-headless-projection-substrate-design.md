# Headless Projection Substrate — Design

**Date:** 2026-07-27
**Status:** Draft (design)
**Scope:** One sub-project of a larger frontend direction. This spec covers ONLY the
frontend-agnostic headless contract. Projection-native JSX derivation, design-lint on
generated frontends, and any change to JSON-UI's role are explicitly out of scope here
and gated behind a separate validation step.

## Problem

A Ferro application declares its data and behavior once as a `ServiceDef` (fields,
meanings, validations) plus `ActionDef`s and state-machine guards. Today that single
declaration already renders to two independent targets without coupling:

- **Visual UI** via `JsonUiRenderer` (`ferro-json-ui`).
- **Agent-operable MCP surface** via `McpRenderer` (`ferro-mcp-server`), derived purely
  from `ServiceDef`/`ActionDef`.

The transition-execution kernel is channel-agnostic: `framework::write::dispatch_write`
(`framework/src/write/mod.rs`) runs the same guard re-evaluation, persistence, and audit
pipeline whether the caller is the web form surface (`channel = "web"`) or MCP
(`channel = "mcp"`).

What is missing is a **third target**: a frontend-agnostic HTTP contract that lets *any*
frontend — a hand-written or externally-generated React/Inertia app, or any other client
— consume the same declared data, schema, and currently-permitted actions. A bespoke
frontend today must hand-roll three things the built-in renderer gets implicitly:

1. **Data.** `ferro-json-ui` assumes a `GET /data/{service}` endpoint exists
   (`ferro-json-ui/src/projection/builder.rs`), but nothing derives that endpoint from
   the projection.
2. **Schema.** No endpoint exposes a service's field set, meanings, and validations so a
   client can bind inputs to fields without reading Rust source.
3. **Permitted actions.** To know which actions to surface, a client must call MCP
   `tools/list` (guard-filtered) or re-implement guard evaluation itself — duplicating a
   control surface that already exists server-side.

These are conveniences absent from the HTTP surface, not architectural coupling: the
operability layer is already renderer-independent. This spec closes the three gaps by
exposing them as a projection-derived contract, keeping a single source of truth.

## Goal

Expose a projection-derived, opt-in, frontend-agnostic HTTP contract — schema, data, and
permitted actions — so any frontend consumes the same declaration that already drives the
visual and MCP renderers, and writes flow through the same channel-agnostic kernel.

## Non-Goals

- No projection-native JSX/React derivation (props/data/action/live hooks, component
  scaffolds). Future work, gated on a separate coherence-at-scale validation.
- No change to JSON-UI's role or the design-system lint target. JSON-UI remains a
  first-class renderer.
- No new authentication model. The contract reuses the existing per-tenant middleware and
  the app's session/key auth.
- No new write path. Writes reuse `dispatch_write` via the existing action route.

## Design

### The contract

For each service that opts in, three read endpoints plus the existing write route:

| Purpose | Route | Derivation |
|---------|-------|-----------|
| Schema | `GET /project/{service}` | Pure function of `ServiceDef` — fields, meanings, validations, action definitions. Static per deploy; cacheable. |
| Data | `GET /data/{service}?filter=…&limit=…&cursor=…` | Tenant-scoped records shaped by the projection's field set. Runtime query. |
| Permitted actions | Per-record, in the data response (and/or `GET /project/{service}/{id}/actions`) | Action names whose guards currently pass for that record + tenant, evaluated by the same `GuardEvaluatorFn` the MCP and web paths use. |
| Write (existing) | `POST /{service}/{action}` | Existing visual-action route → `dispatch_write(.., channel = "web")`. Unchanged. |

### Single source of truth

Every field of the contract derives from declarations that already feed MCP and the write
kernel. No parallel definitions:

- **Schema** is a new implementation of the modality-agnostic `Renderer` trait
  (`ferro-projections::Renderer`) with `Output` = the schema contract. Per the framework's
  renderer-location rule, this renderer lives in its output crate, not in
  `ferro-projections`. It sits alongside `JsonUiRenderer`, `McpRenderer`, and
  `TextRenderer` as another projection target.
- **Permitted actions** call the same `GuardEvaluatorFn` registered on the app's
  `WriteDispatcher`. A record's permitted set is exactly the actions MCP `tools/list`
  would expose for that record — verified by a cross-surface test.
- **Writes** are untouched: the contract adds no write path; clients POST to the existing
  action route.

This honors the "no duplicate control surface" convention: the contract is a *projection*
of existing declarations, never a second place to define fields, actions, or guards.

### Opt-in

A service joins the headless contract through the same kind of explicit registration the
MCP surface already uses (the app's `exposed_services()` list), rather than exposing every
model implicitly. Exact surface (a builder flag on the projection vs. a registration list)
is a planning decision; the principle is explicit opt-in, tenant-scoped by default.

### Data flow

```
frontend ──GET /project/{service}──▶ schema contract         (fields, meanings, validations, actions)
frontend ──GET /data/{service}?…──▶ tenant-scoped records + per-record permitted actions
frontend ──POST /{service}/{action}▶ dispatch_write(channel="web")  ── guard re-eval → persist → audit
agent    ──/mcp──────────────────────▶ ServiceDef-derived tools     ── same kernel, channel="mcp"
```

The frontend renders pixels however it likes; correctness (validation, permissions,
transitions, audit) and the agent surface come from the one declaration underneath.

### Error handling

- Reuse the framework's JSON error envelope and `WriteError` mapping (identical to the MCP
  and web surfaces) so a client sees consistent errors across channels.
- Tenant scoping is enforced on data and permitted-action reads: cross-tenant ids are not
  found.
- A service that has not opted in, or an unknown service, returns 404.
- Guard failures on write are surfaced through the existing `dispatch_write` error path —
  no new failure semantics.

### Testing

- **Schema**: snapshot of `GET /project/{service}` against the `ServiceDef` field set,
  meanings, validations, and action definitions.
- **Data**: tenant scoping (records from other tenants are excluded), filter/limit/cursor
  behavior, field shaping matches the projection.
- **Permitted actions parity** (the key test): for a record in a given state, the
  permitted-action set from the data endpoint equals the guard-filtered set the MCP
  `tools/list` returns for the same record and tenant. Changing the record's state changes
  both identically. This mirrors the existing `single_source_both_channels` pattern across
  a third surface.
- **Write parity**: a POST through the contract's action route reaches the same
  `dispatch_write` kernel, with the same guard re-evaluation and audit record (differing
  only by channel tag), that MCP and the visual surface reach.

## Open Questions (for planning)

- **Host crate.** Does the schema renderer + the data/permitted-action handlers live in
  `framework` (HTTP-native) or a dedicated small crate (e.g. `ferro-headless`)? The
  runtime handlers need DB + tenant + guard evaluator; the schema renderer is pure.
- **Permitted actions placement.** Embedded per-record in the data response, a separate
  `GET /project/{service}/{id}/actions`, or both. Trade-off: fewer round-trips vs. a
  cacheable data endpoint.
- **Opt-in surface.** Builder flag on the projection vs. a registration list mirroring
  `exposed_services()`.
- **Auth.** A same-origin bespoke SPA likely uses the app's session/cookie auth; the MCP
  surface uses the per-tenant bearer key. Confirm both map to the same tenant middleware.
- **Pagination contract.** Cursor vs. offset; default and maximum `limit`.

## Sequencing note

This substrate is independently valuable and frontend-agnostic — it works under any
frontend, including the existing renderers. It is the foundation any later
projection-native-frontend work would build on, but it does not commit to that work. The
downstream layers (deriving a frontend from the projection, and enforcing design
consistency on a generated frontend) depend on a separate question — whether a
projection-derived, design-constrained generated frontend stays coherent at scale — which
should be validated on its own before those layers are specced.
