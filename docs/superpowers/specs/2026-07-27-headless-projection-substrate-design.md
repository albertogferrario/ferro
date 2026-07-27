# Projection-Native Frontend Substrate (Inertia-First) — Design

**Date:** 2026-07-27
**Status:** Draft (design)
**Scope:** Derive a frontend's data, schema, and permitted-actions from the projection so a
custom (Inertia) frontend binds to the same single declaration that drives the visual and
MCP renderers — and reuse the existing write kernel for actions. Inertia is the primary
delivery target; a generic JSON delivery for decoupled SPAs / native clients is an additive
extension of the same derivation. Deriving JSX/React *components* (scaffolds, hooks) from the
projection, and any design-lint on generated frontends, are out of scope here and gated on a
separate coherence-at-scale validation.

## Problem

A Ferro application declares its data and behavior once as a `ServiceDef` (fields, meanings,
validations) plus `ActionDef`s and state-machine guards. That single declaration already
renders to two targets without coupling:

- **Visual UI** via `JsonUiRenderer` (`ferro-json-ui`).
- **Agent-operable MCP surface** via `McpRenderer` (`ferro-mcp-server`), derived purely from
  `ServiceDef`/`ActionDef`.

The transition-execution kernel is channel-agnostic: `framework::write::dispatch_write`
(`framework/src/write/mod.rs`) runs the same guard re-evaluation, persistence, and audit
pipeline whether the caller is the web form surface (`channel = "web"`) or MCP
(`channel = "mcp"`).

`ferro-inertia` is the framework's path to a custom (React/Vue/Svelte) frontend, but it is
**projection-blind**: an Inertia page hand-authors its props, so it does not automatically
receive the projection's data, its field schema/meanings/validations, or — critically — the
set of actions currently permitted for a record (which the visual and MCP surfaces already
compute from the same guards). A developer building a custom frontend re-hand-rolls three
things the built-in renderers get implicitly:

1. **Data** — `ferro-json-ui` assumes a `GET /data/{service}` convention
   (`ferro-json-ui/src/projection/builder.rs`) but nothing derives it from the projection.
2. **Schema** — no derivation exposes a service's field set, meanings, and validations so a
   frontend can bind inputs without reading Rust source.
3. **Permitted actions** — to know which actions to surface, a frontend must re-implement guard
   evaluation or call MCP `tools/list`, duplicating a control surface that already exists
   server-side.

These are missing derivations, not architectural coupling: the operability layer is already
renderer-independent. This spec closes the gaps by deriving them once and delivering them
Inertia-first, keeping a single source of truth.

## Goal

From one `ServiceDef`, derive a custom frontend's **props = { schema, data, permitted-actions }**
and deliver them to an Inertia page in one call; route that frontend's writes through the
existing channel-agnostic kernel; and expose the same derivation as a generic JSON contract
additively for non-Inertia clients.

## Non-Goals

- No projection-native JSX/React *component* derivation (scaffolds, typed hooks). Future work,
  gated on a separate coherence-at-scale validation.
- No design-lint on generated frontends.
- No new authentication model. Inertia reuses the app's existing same-origin session/CSRF; the
  generic JSON mode's token/CORS story is a later addition.
- No change to JSON-UI's role. JSON-UI remains a first-class renderer (the velocity path for
  style-does-not-matter systems); this substrate is the customization path.
- No new write path. Writes reuse `dispatch_write`.

## Design

### 1. The derivation core (single source of truth)

Three derivations, all from the same declarations that already feed MCP and the write kernel:

- **Schema** — a pure `schema_contract(&ServiceDef) -> SchemaContract` (fields, meanings,
  validations, action definitions). Schema-level and dependency-free; a natural sibling of
  `derive_intents()` in `ferro-projections` (it renders nothing — it projects the declaration).
- **Data** — a tenant-scoped query shaped by the `ServiceDef` field set (runtime; `framework`).
- **Permitted actions** — `permitted_actions(service, record, tenant, ctx) -> Vec<ActionName>`:
  the actions whose guards currently pass for a record, evaluated by the **same**
  `GuardEvaluatorFn` the MCP and write paths use.

### 2. The one refactor: share guard-visibility in `framework`

Today the "which actions are permitted" logic lives inside `ferro-mcp-server` (the `tools/list`
filter). To share it with the Inertia substrate **without** making `ferro-inertia` depend on
`ferro-mcp-server`, lift it into `framework` — which already owns the write kernel and the
`GuardEvaluatorFn`. Afterwards `ferro-mcp-server` **and** `ferro-inertia` both call
`framework`'s `permitted_actions(...)`. Guards are evaluated in exactly one place. This is the
only structural change; it removes a latent duplicate control surface rather than adding one.

### 3. Inertia delivery (the primary target)

In `ferro-inertia`: `Inertia::from_projection(req, service, query) -> InertiaResponse` that
loads the tenant-scoped data, attaches the `SchemaContract`, attaches `permitted_actions`
per record, and serializes them as Inertia props. The React/Vue/Svelte component receives
typed props of a known shape and renders freely. Per the renderer-location rule, this
delivery helper lives in its output crate (`ferro-inertia`), not in `ferro-projections`.

### 4. Writes: reuse, do not rebuild

Inertia forms `POST /{service}/{action}` → the existing `dispatch_write(.., channel = "web")`
(the visual-action route from Phase 232). Guard re-evaluation, persistence, and audit are
already there. The substrate adds read derivation only.

### 5. Operability comes free

An Inertia-first app still declares `ServiceDef`s, so `McpRenderer` derives the per-tenant MCP
tools from the same declaration. The custom frontend and the agent surface are two projections
of one source; neither is hand-maintained against the other.

### 6. Generic JSON mode (additive, later)

The same three derivations, serialized as REST instead of Inertia props:
`GET /project/{service}` (schema) and `GET /data/{service}?filter&limit` (records +
per-record permitted actions). This serves a decoupled SPA (Next.js/SvelteKit) or a
native/mobile client. It is a second delivery format over the identical derivation core — not
a second implementation. Deferred until a consumer needs it.

### 7. Auth

- **Inertia** — same-origin: reuse the app's existing session/cookie auth + CSRF. Nothing new.
- **Generic JSON mode** — cross-origin/native: needs token auth (the per-tenant key or a JWT)
  + CORS. Added with that mode, not before.

### Data flow

```
Inertia page ──Inertia::from_projection──▶ props = { schema, data, permitted_actions }
custom frontend ──POST /{service}/{action}──▶ dispatch_write(channel="web")  ── guard re-eval → persist → audit
agent ──/mcp──────────────────────────────────▶ ServiceDef-derived tools     ── same kernel, channel="mcp"
                              ▲
                    permitted_actions(...) in framework — shared by Inertia props AND MCP tools/list
```

## Error handling

- Reuse the framework JSON error envelope and `WriteError` mapping so a custom frontend sees
  the same errors across channels.
- Tenant scoping enforced on data and permitted-action reads; cross-tenant ids are not found.
- Guard failures on write flow through the existing `dispatch_write` error path — no new
  failure semantics.

## Testing

- **Schema**: snapshot of `schema_contract(&ServiceDef)` against the field set, meanings,
  validations, and action definitions.
- **Data**: tenant scoping (other tenants excluded), filter/limit shaping matches the projection.
- **Permitted-actions parity** (the key single-source test): for a record in a given state,
  the permitted-action set from `permitted_actions(...)` equals the guard-filtered set the MCP
  `tools/list` returns for the same record and tenant; changing state changes both identically.
  Mirrors the existing `single_source_both_channels` pattern across the Inertia surface.
- **Write parity**: an Inertia `POST /{service}/{action}` reaches the same `dispatch_write`
  kernel (guard re-eval + audit) that MCP reaches, differing only by channel tag.

## Open Questions (for planning)

- **`SchemaContract` location.** `ferro-projections` (pure schema, sibling of `derive_intents`)
  vs. `ferro-inertia`. Leaning `ferro-projections` since it renders nothing.
- **Opt-in surface.** A builder flag on the projection vs. a registration list mirroring the
  MCP `exposed_services()`.
- **Permitted-actions placement in props.** Per-record inline vs. a separate lookup.
- **Pagination contract** for the data query (cursor vs. offset; default/max limit).
- **The `permitted_actions` extraction** from `ferro-mcp-server` into `framework`: confirm the
  current `tools/list` filter can be lifted cleanly without regressing the MCP surface.

## Sequencing

1. **This spec — Inertia-first substrate:** `schema_contract` + the `permitted_actions`
   extraction into `framework` + `Inertia::from_projection` + write reuse. Independently
   valuable; makes the Inertia path projection-native.
2. **Generic JSON delivery mode** — additive, when a decoupled/native client needs it.
3. **Projection-native JSX component derivation** (scaffolds/hooks) + design discipline on the
   output — a *separate later spec*, gated on validating that projection-derived,
   design-constrained frontends stay coherent at scale.
