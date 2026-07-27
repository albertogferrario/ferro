# Phase 263: Projection-native Inertia substrate - Context

**Gathered:** 2026-07-27
**Status:** Ready for planning
**Source:** PRD Express Path (`docs/superpowers/specs/2026-07-27-headless-projection-substrate-design.md`)

<domain>
## Phase Boundary

Derive a custom (Inertia) frontend's **props = `{ schema, data, permitted_actions }`** from a
single `ServiceDef`/`ActionDef` declaration — the same declaration that already feeds the
visual (`JsonUiRenderer`) and MCP (`McpRenderer`) surfaces — and deliver them to an Inertia
page in one call. Route that frontend's writes through the existing channel-agnostic
`dispatch_write` kernel (no new write path). Lift the "which actions are permitted for a
record" logic out of `ferro-mcp-server` into `framework` so guards are evaluated in exactly
one place, shared by MCP `tools/list` and the Inertia substrate.

**In scope (this phase):**
1. `schema_contract(&ServiceDef) -> SchemaContract` — pure, schema-level derivation in
   `ferro-projections` (fields, meanings, validations, action definitions). Renders nothing.
2. The one refactor — lift `permitted_actions(service, record, tenant, ctx)` guard-visibility
   logic from `ferro-mcp-server` (`tools/list` filter) into `framework`; both MCP and Inertia
   then call it. One guard-evaluation site.
3. Tenant-scoped **data** query shaped by the `ServiceDef` field set (runtime; `framework`).
4. `Inertia::from_projection(req, service, query)` in `ferro-inertia` — attaches schema + data +
   per-record permitted actions as Inertia props.
5. **Write reuse** — Inertia `POST /{service}/{action}` → existing `dispatch_write(channel="web")`.
6. Single-source parity tests (permitted-actions Inertia↔MCP, write parity, schema snapshot,
   data tenant-scoping).

**Explicitly NOT in scope** (see Deferred Ideas): JSX/React component derivation, design-lint on
generated frontends, the generic JSON delivery mode, any new auth model, any change to JSON-UI's
role, any new write path.
</domain>

<decisions>
## Implementation Decisions

Everything in the anchor spec is a locked decision.

### Derivation core (single source of truth)
- Three derivations, all from the declarations that already feed MCP and the write kernel:
  **schema** (pure, dependency-free), **data** (tenant-scoped, field-set-shaped, runtime),
  **permitted actions** (guards that currently pass for a record).
- `schema_contract` is a natural sibling of `derive_intents()` in `ferro-projections` — it
  projects the declaration and renders nothing.
- `permitted_actions(...)` is evaluated by the **same** `GuardEvaluatorFn` the MCP and write
  paths use — not a re-implementation.

### The one refactor (share guard-visibility in `framework`)
- The `tools/list` "which actions are permitted" filter currently lives inside
  `ferro-mcp-server`. Lift it into `framework` (which already owns the write kernel and the
  `GuardEvaluatorFn`).
- After the lift, `ferro-mcp-server` **and** `ferro-inertia` both call `framework`'s
  `permitted_actions(...)`. This removes a latent duplicate control surface rather than adding
  one. `ferro-inertia` must NOT depend on `ferro-mcp-server`.
- This is the only structural change in the phase.

### Inertia delivery (primary target)
- `Inertia::from_projection(req, service, query) -> InertiaResponse` loads tenant-scoped data,
  attaches the `SchemaContract`, attaches `permitted_actions` per record, serializes as Inertia
  props. Component receives typed props of a known shape and renders freely.
- **Placement (corrected 2026-07-27, operator-approved):** the delivery helper lives on the
  **framework-side Inertia facade — `framework/src/inertia/projection.rs`** — the `Request`-aware
  `Inertia` delivery module that already wraps `ferro_inertia::Inertia::render`. The spec's literal
  "`in ferro-inertia`" is a hard Cargo cycle: `framework` already depends on `ferro-inertia`
  (optional `inertia` feature), so `ferro-inertia → framework` is forbidden, and `from_projection`
  is inherently framework-coupled (needs the `Request`, DB handle, `permitted_actions`, and
  `projection_read`). The renderer-location rule's intent is still honored — the helper is NOT in the
  pure `ferro-projections` crate; it is in the framework's Inertia delivery layer. Cycle class matches
  Phase 261's `ferro-bundle`.

### Writes: reuse, do not rebuild
- Inertia forms `POST /{service}/{action}` → the existing `dispatch_write(.., channel = "web")`
  (the visual-action route from Phase 232). Guard re-evaluation, persistence, and audit already
  exist. The substrate adds read derivation only.

### Operability comes free
- An Inertia-first app still declares `ServiceDef`s, so `McpRenderer` derives the per-tenant MCP
  tools from the same declaration. Custom frontend and agent surface are two projections of one
  source; neither is hand-maintained against the other.

### Auth
- **Inertia** — same-origin: reuse the app's existing session/cookie auth + CSRF. Nothing new.
- **Generic JSON mode** — token + CORS, added *with that mode*, not before (deferred).

### Error handling
- Reuse the framework JSON error envelope and `WriteError` mapping so a custom frontend sees the
  same errors across channels.
- Tenant scoping enforced on data and permitted-action reads; cross-tenant ids are not found.
- Guard failures on write flow through the existing `dispatch_write` error path — no new failure
  semantics.

### Claude's Discretion (open questions to resolve in planning)
- **`SchemaContract` location** — `ferro-projections` (pure schema, sibling of `derive_intents`)
  vs. `ferro-inertia`. Spec leans `ferro-projections` since it renders nothing. Confirm.
- **Opt-in surface** — a builder flag on the projection vs. a registration list mirroring the MCP
  `exposed_services()`.
- **Permitted-actions placement in props** — per-record inline vs. a separate lookup.
- **Pagination contract** for the data query — cursor vs. offset; default/max limit.
- **`permitted_actions` extraction cleanliness** — confirm the current `ferro-mcp-server`
  `tools/list` filter can be lifted into `framework` without regressing the MCP surface.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Anchor spec
- `docs/superpowers/specs/2026-07-27-headless-projection-substrate-design.md` — the design:
  problem, goal, non-goals, the three derivations, the one refactor, data flow, testing,
  open questions, sequencing.

### Write kernel (reuse — do not rebuild)
- `framework/src/write/mod.rs` — `dispatch_write`, `GuardEvaluatorFn`; channel-agnostic
  transition kernel (Phase 231/232). Inertia writes reuse it at `channel = "web"`.

### Guard-visibility extraction source
- `ferro-mcp-server` `tools/list` filter — the current home of "which actions are permitted";
  the logic lifted into `framework::permitted_actions(...)`.

### Projection declaration + read shape
- `ferro-projections/src/lib.rs` (`ServiceDef`, `ActionDef`, `derive_intents()`) — the
  declaration `schema_contract` projects; `schema_contract` is the pure sibling.
- `ferro-json-ui/src/projection/builder.rs` — the `GET /data/{service}` read convention the
  data derivation formalizes.

### Delivery target
- `framework/src/inertia/projection.rs` — where `Inertia::from_projection` lives, on the
  framework-side Inertia facade that wraps `ferro_inertia::Inertia::render` (see the corrected
  placement decision above; `ferro-inertia`-crate placement is a Cargo cycle). Read
  `framework/src/inertia/` (esp. `context.rs`, the `Request`-aware `Inertia` struct) as the analog.

### Existing single-source test to mirror
- The `single_source_both_channels` test pattern (v16.0 / Phase 232) — the permitted-actions
  parity test mirrors it across the Inertia surface.
</canonical_refs>

<specifics>
## Specific Ideas

### Data flow (from the spec)
```
Inertia page ──Inertia::from_projection──▶ props = { schema, data, permitted_actions }
custom frontend ──POST /{service}/{action}──▶ dispatch_write(channel="web") ── guard re-eval → persist → audit
agent ──/mcp──────────────────────────────────▶ ServiceDef-derived tools ── same kernel, channel="mcp"
                              ▲
                    permitted_actions(...) in framework — shared by Inertia props AND MCP tools/list
```

### Testing contracts (the acceptance shape)
- **Schema**: snapshot of `schema_contract(&ServiceDef)` against the field set, meanings,
  validations, and action definitions.
- **Data**: tenant scoping (other tenants excluded); filter/limit shaping matches the projection.
- **Permitted-actions parity** (key single-source test): for a record in a given state, the set
  from `permitted_actions(...)` equals the guard-filtered set MCP `tools/list` returns for the
  same record and tenant; changing state changes both identically.
- **Write parity**: an Inertia `POST /{service}/{action}` reaches the same `dispatch_write`
  kernel (guard re-eval + audit) MCP reaches, differing only by channel tag.

### Architectural constraints
- `ferro-inertia` must not depend on `ferro-mcp-server` (that is the whole reason for lifting
  `permitted_actions` into `framework`).
- One guard-evaluation site after the refactor (grep-verifiable).
- Renderer-location rule: derivation cores in `ferro-projections`/`framework`; delivery helper
  in the framework Inertia module (`framework/src/inertia/projection.rs`) — NOT the pure
  `ferro-projections` crate. (Framework-side rather than the `ferro-inertia` crate because
  `framework` already depends on `ferro-inertia`; see the corrected placement decision above.)
</specifics>

<deferred>
## Deferred Ideas

- **Projection-native JSX/React component derivation** (scaffolds, typed hooks) — future work,
  gated on a separate coherence-at-scale validation (spec sequencing step 3, separate later spec).
- **Design-lint on generated frontends** — deferred with the JSX work.
- **Generic JSON delivery mode** — `GET /project/{service}` (schema) + `GET /data/{service}?filter&limit`
  (records + per-record permitted actions) for a decoupled SPA / native client. A second delivery
  format over the identical derivation core, not a second implementation. Deferred until a consumer
  needs it (spec sequencing step 2); its token/CORS auth story lands with it.
- **New authentication model** — Inertia reuses same-origin session/CSRF; nothing new here.
- **Any change to JSON-UI's role** — JSON-UI stays the first-class velocity renderer; this
  substrate is the customization path.
- **Any new write path** — writes reuse `dispatch_write`.
</deferred>

---

*Phase: 263-projection-native-inertia-substrate*
*Context gathered: 2026-07-27 via PRD Express Path*
