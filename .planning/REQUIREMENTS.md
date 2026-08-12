# Requirements (accreted — multi-milestone working file)

> This file accumulates one `# Requirements — vX.Y` section per milestone; it was never
> rotated per-milestone. On each milestone close its section is extracted to
> `milestones/vX.Y-REQUIREMENTS.md` and removed here. Sections currently present, in order:
> **v16.4** Work Distribution (QUEUED — not shipped), **v16.6** POS Component Suite
> (COMPLETE — archive pending), **v18.0** Projection-Native Frontend Substrate (CURRENT).
> Archived and removed so far: v16.3, v16.5 (both 2026-08-12). v17.0 requirements are not in
> this file (tracked inline in ROADMAP).

---

# Requirements — v16.4 Work Distribution (`#[offload]` Service Methods) — QUEUED

**Status:** Queued behind v16.3 Phase 243. v16.3 remains the current milestone until 243
closes; v16.4 phases (244–249) continue the numbering and are planned/executed afterward.

**Milestone goal:** A `#[service]` trait method marked `#[offload]` becomes a distributable
unit of work with zero hand-written queue plumbing — the framework derives the `ferro-queue`
Job, serializable payload, and a typed result handle from the method signature, runs it on a
horizontally scalable worker, and streams the result back via the read-model + broadcast path.
Anchor spec: `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md`.

**Scope decision:** Build the scalable primitive; defer the auto-deciding. Capacity scales by
running more workers (external/operator/k8s-managed N). Autonomous machine lifecycle /
scale-to-zero / KEDA / CRIU is **out of scope** (cost-optimization, not capacity — parked as a
2.0 direction in the spec).

## v16.4 Requirements

### Offload Primitive

- [ ] **OFFLOAD-01**: A developer marks a `#[service]` trait method `#[offload]` and the
  framework derives a `ferro-queue` Job + serializable payload from the method signature — no
  hand-written Job struct, no manual enqueue.
- [ ] **OFFLOAD-02**: Calling an offloaded method returns a typed result handle; a method whose
  parameter or return type is not `Serialize`/`DeserializeOwned` fails at compile time with a
  clear, type-naming diagnostic (this enforcement is the module-isolation boundary).

### Result Delivery

- [ ] **OFFLOAD-03**: An offloaded method's return value is persisted as a `ferro-projection`
  snapshot keyed by the handle, retrievable after completion; a failed run records a terminal
  error state (no silent drop).
- [ ] **OFFLOAD-04**: A client subscribed to a handle receives the result as a `ferro-broadcast`
  delta on completion; the originating request returns immediately and never blocks awaiting it.

### Scalable Execution

- [ ] **OFFLOAD-05**: Offloaded work runs on a deployable `ferro worker` process runnable at N
  replicas against the shared queue; capacity scales by adding replicas; each worker class is an
  independent fault domain. No framework-managed autoscaling (N is external).

### Introspection & Docs

- [ ] **OFFLOAD-06**: Offloadable methods are introspectable through `ferro-mcp` (`list_services`,
  derived payload schema); docs cover the authoring surface, result path, scaling model
  (stateless tier + replicable workers + cache + queue), and the non-goals / deferred elastic
  direction.

## Out of Scope (v16.4)

- **Synchronous, request-path, cross-machine RPC** (the rejected "Approach A" — see spec
  Alternatives).
- **Autonomous machine lifecycle / scale-to-zero** (KEDA, CRIU warm-start, Nomad, WASM isolates)
  — 2.0 direction; the queue-consumer model does not preclude it.

## Traceability (v16.4)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| OFFLOAD-01 | Phase 244 | Not started |
| OFFLOAD-02 | Phase 245 | Not started |
| OFFLOAD-03 | Phase 246 | Not started |
| OFFLOAD-04 | Phase 247 | Not started |
| OFFLOAD-05 | Phase 248 | Not started |
| OFFLOAD-06 | Phase 249 | Not started |

---

# Requirements — v16.6 POS Component Suite — COMPLETE (shipped 0.2.89)

**Status:** Current milestone, started 2026-07-04. Phase numbering continues at 254.
Independent of v16.4 (queued, reserved 244–249). Consumer-paired with gestiscilo's
register/counter ("cassa") mode; seed finding: the ~1500-line RawHtml product picker
audited in `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md`.

**Milestone goal:** Touch-first sale-screen components in the ferro-json-ui builtin
catalog — tile grid, selection panel, numpad, quantity/filter primitives — at a tablet
interaction quality bar, derivable from a `ServiceDef` through the projection layer and
agent-authorable through the v16.5 MCP + design-lint boundary.

**Research:** `.planning/research/` (STACK, FEATURES, ARCHITECTURE, PITFALLS, SUMMARY —
2026-07-04). Zero new dependencies; all components are builtins (never plugins); the
seven-intent vocabulary is unchanged.

**Scope decisions (2026-07-04):** form-state cart only (client CartRuntime deferred —
consequence accepted: gestiscilo's RawHtml elimination is partial until it ships);
FilterTabs (née CategoryNav) is a standalone builtin; Grid `row_weights` included;
Numpad included; barcode keyboard-wedge deferred.

**Vocabulary decision (2026-07-05):** the builtin catalog uses domain-neutral,
structural nouns only — mirroring the seven structural intents. Commerce naming is
confined to sample apps and docs examples. Renames (applied in Phase 255, all
pre-publish): `ProductTile`→`Tile` (incl. the published component — breaking,
migration table entry required), `ProductGrid`→`TileGrid`, `CartPanel`→`SelectionPanel`
(consumer-specific props removed), `CategoryNav`→`FilterTabs`; runtime attributes
`data-product-*`→`data-filter-*`; lint rule ids `pos-*`→`register-*`; no
consumer-specific props in any `ferro-*` crate. The `Register` layout template name
(POS-10) is retained as a structural layout term.

## v16.6 Requirements

### Components

- [x] **POS-01**: A spec author binds an items data path to a `TileGrid` builtin and
  gets a responsive, touch-first tile grid (Tile children via `$each`) with
  built-in client-side text search filtering. Tiles are **tap-to-add-only**
  surfaces (operator decision 2026-07-05): one tap adds one unit; no on-tile
  +/- steppers or quantity display — quantity editing lives in the
  SelectionPanel (POS-04).
- [x] **POS-02**: `ProductTile` gains additive props — `category`, `image_url`, `color`,
  `stock_badge` — with existing specs rendering unchanged (serde-backward-compatible).
  *(Delivered 254 under the old name; component renamed `Tile` in Phase 255.)*
- [x] **POS-03**: A `FilterTabs` standalone builtin filters visible tiles by
  filter token client-side (show/hide), touch targets ≥44px.
- [x] **POS-04**: A `SelectionPanel` builtin renders the running selection as a **live
  client-side view of the form state** (operator decision 2026-07-05, un-deferring the
  CartRuntime slice): lines appear as tiles are tapped, each line has a per-line
  QuantityStepper + remove, the running total is client-computed in integer cents,
  EmptyState shows when nothing is selected, and a confirm-action slot hosts the
  single confirm POST. The panel pins and internally scrolls within a `fill_viewport`
  layout.
- [x] **POS-05**: A `QuantityStepper` standalone builtin provides a reusable +/− numeric
  stepper on the Tile hidden-input contract, usable in selection lines and forms.
- [x] **POS-06**: A `Numpad` builtin provides a custom tap-surface numeric keypad
  (≥56px keys) writing to a target field — never a native input, so the software keyboard
  is never triggered.

### Touch Interaction Quality

- [x] **POS-07**: A shared POS touch foundation — `touch-action: manipulation`, `:active`
  press states on the motion tokens, tap-highlight reset, overscroll containment, minimum
  hit-target constants — is centralized in `render/classes.rs` and applied across all POS
  components; every emitted class is a full literal (Tailwind-scanner/safelist-safe).
  *(Delivered 254 as `POS_*` constants; prefix neutralized in Phase 255.)*
- [x] **POS-08**: POS forms are double-submit protected — a `data-disable-on-submit`
  runtime guard plus the documented idempotency-key pattern on the existing
  `framework::write` idempotency hook.

### Layout

- [x] **POS-09**: `Grid` gains `row_weights` — asymmetric fill-row weighting (product pane
  taller than cart on phones), additive alongside the Phase 253 `spans` prop.

### Projection

- [x] **POS-10**: A `ServiceDef` renders a working sale screen via a `Register` layout
  template under the **Collect** intent (builder `emit_register_root` +
  `ElementBuilder.each()` + `fill_viewport` emission) — the seven-intent vocabulary is
  unchanged.

### Agent-Authoring Boundary

- [x] **POS-11**: POS design-lint rules ship — `pos-fill-viewport`, `pos-grid-fill`,
  `pos-cart-present`, `fill-viewport-layout-unknown` — each with violating/conforming AND
  data-bound fixtures; `RULE_COMPONENTS` mapping updated.
  *(Delivered 254 under the old ids; renamed `register-fill-viewport`,
  `register-grid-fill`, `register-selection-present` in Phase 255.)*
- [x] **POS-12**: The MCP + docs surface is extended — `json_ui_catalog` entries/count for
  the new components, `generation_context` register composition guidance, `docs/src`
  updates.

### Release

- [x] **POS-13**: The `/cassa` sample app flips to the new components (projection-derived
  Register), the full CI-exact gate is green, and a single crates.io publish closes the
  milestone (gestiscilo's register phase gates on it).

## Future Requirements (deferred)

- ~~**CartRuntime** — client-side live cart (tap → cart panel update + running total, single
  commit POST) via a `runtime/cart_runtime.rs` module with a documented `data-cart-target`
  extension hook. Deferred by scope decision; revisit on gestiscilo adoption friction.~~
  **UN-DEFERRED into Phase 256 (operator decision 2026-07-05):** the register interaction
  model is tap-tile-adds-one / quantities-edited-in-the-SelectionPanel (Shopify POS model),
  which requires the panel to be a live client-side view of the form state — lines
  appear/update as tiles are tapped, per-line QuantityStepper edit + remove, running total
  in integer cents. The form-state contract is unchanged (hidden inputs, single confirm
  POST); no `data-cart-target` props hook (254 D-18 still holds — the runtime binds by
  attribute contract, not props extension).
- **Barcode keyboard-wedge input** — keystroke-timing scanner detection (~40 lines runtime
  JS, `data-barcode-max-gap` tuning attribute).
- **Layout-name-independent `ferro-fill` chain** — removes the dashboard-family selector
  dependency of `fill_viewport` (deeper refactor; the `fill-viewport-layout-unknown` lint
  rule mitigates meanwhile).

## Out of Scope (v16.6)

- **Payment flow** (cash/card/split tender), **receipt rendering**, **shift/session close**
  — the sale screen itself only; a later milestone.
- **New intents** (`Register`/`Kiosk`) — the seven-intent vocabulary is frozen (v16.5
  decision: archetypes ARE the intents).
- **Plugin-registry placement** — all POS components are builtins; any `register_component`
  use is a review blocker.
- **Hardware integration** (cash drawers, receipt printers, scanner SDKs) — keyboard-wedge
  input is the only scanner path ever considered, and it is deferred.

## Traceability (v16.6)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| POS-01 | Phase 256 | Complete |
| POS-02 | Phase 254 | Complete |
| POS-03 | Phase 256 | Complete |
| POS-04 | Phase 256 | Complete |
| POS-05 | Phase 256 | Complete |
| POS-06 | Phase 256 | Complete |
| POS-07 | Phase 254 | Complete |
| POS-08 | Phase 255 | Complete |
| POS-09 | Phase 256 | Complete |
| POS-10 | Phase 257 | Complete |
| POS-11 | Phase 254 | Complete |
| POS-12 | Phase 258 | Complete |
| POS-13 | Phase 258 | Complete |

---

# Requirements — v18.0 Projection-Native Frontend Substrate — CURRENT

**Status:** Current milestone, created 2026-07-27 from committed anchor spec. Phase
numbering continues at 263, following v17.0 Live Projection Surface (259–262).

**Milestone goal:** Derive a custom (Inertia) frontend's data, field schema, and
permitted-actions from the same `ServiceDef`/`ActionDef` declaration that already drives
the visual (`JsonUiRenderer`) and MCP (`McpRenderer`) surfaces, and deliver them
Inertia-first in one `Inertia::from_projection` call. Route that frontend's writes through
the existing channel-agnostic `dispatch_write` kernel. The single structural change lifts
guard-visibility (`permitted_actions`) out of `ferro-mcp-server` into `framework` so guards
are evaluated in exactly one place. These are missing derivations, not new coupling — the
operability layer is already renderer-independent.

**Anchor spec:** `docs/superpowers/specs/2026-07-27-headless-projection-substrate-design.md`.

**Builds on shipped work:** `framework::write::dispatch_write` (Phase 231/232) channel-agnostic
transition kernel; the `ferro-mcp-server` `tools/list` guard filter (the extraction source);
`ferro-projections` (`ServiceDef`/`ActionDef`, `derive_intents()`); `ferro-inertia` (delivery
target); the `ferro-json-ui` `GET /data/{service}` read convention.

## v18.0 Requirements

### Derivation core

- [x] **SUBST-01**: A pure `schema_contract(&ServiceDef) -> SchemaContract` derivation in
  `ferro-projections` returns the service's field set, meanings, validations, and action
  definitions. It is schema-level and dependency-free (renders nothing — a sibling of
  `derive_intents()`) and snapshot-tested against the declaration.
- [x] **SUBST-02**: The "which actions are permitted for a record" logic is lifted out of
  `ferro-mcp-server` (`tools/list` filter) into `framework` as
  `permitted_actions(service, record, tenant, ctx) -> Vec<ActionName>`, evaluated by the
  same `GuardEvaluatorFn` the write path uses. After the refactor `ferro-mcp-server` **and**
  `ferro-inertia` both call `framework`'s `permitted_actions(...)`; guard-visibility is
  evaluated in exactly one place (no duplicate control surface).

### Inertia delivery

- [x] **SUBST-03**: `Inertia::from_projection(req, service, query) -> InertiaResponse` in
  `ferro-inertia` loads the tenant-scoped data shaped by the `ServiceDef` field set, attaches
  the `SchemaContract` and per-record `permitted_actions`, and serializes them as Inertia
  props `{ schema, data, permitted_actions }`. The helper lives on the framework-side Inertia
  facade (`framework/src/inertia/projection.rs`), not in the pure `ferro-projections` crate.
  (Corrected 2026-07-27, operator-approved: `framework` already depends on `ferro-inertia`, so the
  spec's literal `ferro-inertia` placement is a hard Cargo cycle — same class as Phase 261's
  `ferro-bundle`; the renderer-location intent is preserved since the helper is in the framework's
  Inertia delivery layer, not the pure projection crate.) Data reads are tenant-scoped
  (cross-tenant ids are not found) with filter/limit shaping matching the field set.

### Writes

- [x] **SUBST-04**: An Inertia form `POST /{service}/{action}` routes through the existing
  `dispatch_write(.., channel = "web")` kernel (guard re-eval + persist + audit). No new write
  path, no new failure semantics — the substrate adds read derivation only. Errors reuse the
  framework JSON error envelope and `WriteError` mapping across channels.

### Single source (tests)

- [x] **SUBST-05**: Parity is proven by test: (a) permitted-actions parity — for a record in
  a given state the set from `permitted_actions(...)` equals the guard-filtered set MCP
  `tools/list` returns for the same record and tenant, and changing state changes both
  identically (mirrors `single_source_both_channels`); (b) write parity — an Inertia
  `POST /{service}/{action}` reaches the same `dispatch_write` kernel MCP reaches, differing
  only by channel tag; (c) schema snapshot (SUBST-01); (d) data tenant-scoping (SUBST-03).

## Open Questions (resolve in planning)

- `SchemaContract` location — `ferro-projections` (pure schema, leaning) vs. `ferro-inertia`.
- Opt-in surface — a projection builder flag vs. a registration list mirroring MCP
  `exposed_services()`.
- Permitted-actions placement in props — per-record inline vs. a separate lookup.
- Pagination contract for the data query — cursor vs. offset; default/max limit.
- Confirm the `tools/list` filter can be lifted cleanly into `framework` without regressing
  the MCP surface.

## Out of Scope (v18.0)

- **Projection-native JSX/React component derivation** (scaffolds, typed hooks) and any
  **design-lint on generated frontends** — a separate later spec, gated on a coherence-at-scale
  validation.
- **Generic JSON delivery mode** (`GET /project/{service}`, `GET /data/{service}` for decoupled
  SPA / native clients) — additive over the identical derivation core, deferred until a consumer
  needs it; its token/CORS auth story lands with it.
- **New authentication model** — Inertia reuses the app's same-origin session/cookie + CSRF.
- **Any change to JSON-UI's role** — it remains the first-class velocity renderer; this
  substrate is the customization path.

## Traceability (v18.0)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| SUBST-01 | Phase 263 | Complete |
| SUBST-02 | Phase 263 | Complete |
| SUBST-03 | Phase 263 | Complete |
| SUBST-04 | Phase 263 | Complete |
| SUBST-05 | Phase 263 | Complete |
