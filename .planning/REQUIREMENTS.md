# Requirements — v16.3 MCP CRUD Data Surface (Track A)

**Milestone goal:** A projection that opts in derives a complete, safe, tenant-scoped
CRUD interface (create / read+query / update / soft-delete) as MCP tools with zero
hand-written tool code. Foundational track of the broader MCP capability program
(Tracks A–D). Anchor spec:
`docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md`.

**Builds on shipped work:** v16.0 (231/232) StateMachine-derived executor + single
`framework::write` kernel; the `tools/call` `content[]` structured envelope (Phase 205).

## v16.3 Requirements

### CRUD Derivation

- [x] **CRUD-01**: A projection opts into create via `.creatable(true)`, deriving a
  `create_<svc>` MCP tool whose input schema is auto-derived from the existing `field()`
  declarations (excludes Identifier, CreatedAt, and the tenant column; `Status` is set to
  the StateMachine initial state when an SM exists; `Sensitive` excluded).
  *(Declaration surface shipped in `5cb17d60`; tool + schema derivation pending.)*
- [x] **CRUD-02**: A projection opts into update via `.updatable(true)`, deriving
  `update_<svc>` (patch semantics, all fields optional) for **data fields only** — when a
  StateMachine exists, `Status` is never an update input (state changes go through action tools).
- [x] **CRUD-03**: A projection opts into delete via `.deletable(true)`, deriving
  `delete_<svc>` that **soft-deletes** (sets `deleted_at`), is **confirmation-gated**, and is
  filtered out of `list_<svc>` and all read/update/delete paths.

### Query

- [x] **CRUD-04**: `list_<svc>` supports range/comparison filters
  (`<field>__{gt,gte,lt,lte,ne,in}`), sort (`field` / `-field`), and `limit`/`offset`
  pagination — on top of the equality filters that already derive.

### Authorization & Dispatch

- [x] **CRUD-05**: `create`/`update`/`delete` require `read_write` key scope and pass the
  `.mcp_write_ability` policy Gate; `tenant_id` is injected server-side and is never an agent
  input (the tenant column is excluded from every write schema). Cross-tenant / soft-deleted
  targets are indistinguishable from "not found" (non-disclosure).
- [x] **CRUD-06**: CRUD verbs dispatch through the shipped `framework::write` kernel via a
  derived `derive_crud_plan` (the CRUD analog of `derive_transition_plan`), reusing the
  existing override-hook registry, idempotency, channel-parameterized audit, and confirmation —
  single-source across the MCP and visual write surfaces. Does **not** rebuild the dispatcher.
- [x] **CRUD-07**: `ServiceDef::validate()` fails fast at registration when any CRUD verb is
  enabled without `mcp_write_ability`. *(Shipped in `5cb17d60`.)*

## Future Requirements (deferred)

- Dedicated `get_<svc>` single-record tool (currently covered by `list_<svc>` + id equality filter).
- Per-field `immutable()` / `read_only()` overrides (Track A derives field sets from `FieldMeaning`).

## Out of Scope

- **Tracks B/C/D** (richer write semantics, new capability classes, agent-experience meta-tools)
  — each is its own future milestone.
- The `tools/call` `content[]` fix — already resolved (Phase 205); this milestone only extends
  the structured envelope + regression guard to the new CRUD verbs.

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| CRUD-01 | Phase 240 | partial (declaration surface done; tool + schema derivation pending) |
| CRUD-02 | Phase 240 | Complete |
| CRUD-03 | Phase 241 | Complete |
| CRUD-04 | Phase 240 | Complete |
| CRUD-05 | Phase 242 | Complete |
| CRUD-06 | Phase 241 | Complete |
| CRUD-07 | Phase 242 (verified) | done (5cb17d60) |

**Foundation/integration phases (own no requirement uniquely):**
- Phase 239 — soft-delete data model + `deleted_at` migration (substrate for CRUD-03 + CRUD-05).
- Phase 243 — app integration + e2e + envelope guard + catalog/docs (validates CRUD-01..07 end-to-end).

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

# Requirements — v16.5 JSON-UI Design System — COMPLETE

**Status:** Complete — shipped 2026-07-04, published 0.2.86 (awaiting `/gsd-complete-milestone`
archival). Independent of v16.4 (reserved 244–249). Consumer-paired with gestiscilo Phase 232,
unblocked by the 253 publish.

**Milestone goal:** Complete the design system above the token layer — density/motion/
focus-ring tokens with opinionated defaults, a canonical variant vocabulary across all
47 builtin components, and composition patterns codified as machine-readable,
intent-keyed lint rules enforced at the agent-authoring boundary. Anchor spec:
`docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md`.

## v16.5 Requirements

### Token Vocabulary v2

- [x] **DS-01**: The fixed token vocabulary grows from 23 to 30 slots — density
  (`--spacing`), motion (`--motion-duration-fast/base/slow`, `--motion-ease`), focus ring
  (`--color-ring`), display font (`--font-display`) — every new slot with a default in the
  base CSS and `default.css` (light + dark), so **every valid v1 theme remains a valid v2
  theme unchanged**. Regenerated `ferro-base.css` exposes the new utilities as `var()`
  references and collapses motion durations under `prefers-reduced-motion`.
- [x] **DS-02**: `default.css` is refreshed to the documented design language (cool-tinted
  neutrals in both modes, a single accent used sparingly, separation via spacing and
  contrast before borders, small consistent radii, one elevation treatment);
  `docs/src/features/themes.md` documents v2 plus the root-font-size type-scaling recipe.

### Component Variant Discipline

- [x] **DS-03**: All 47 builtin components use the canonical `variant`
  (primary/secondary/outline/ghost/destructive), `tone` (neutral/success/warning/
  destructive), and `size` (sm/md/lg) enums; catalog prop schemas enforce them; drift
  guards extend to the enum sets; a migration table lists every rename for consumers.
- [x] **DS-04**: Every interactive component has hover, `focus-visible` (ring from
  `--color-ring`), and disabled states; transitions use the motion tokens at
  frequency-appropriate tiers; `ferro-base.css` regenerated after class changes.

### Pattern Layer

- [x] **DS-05**: `Spec` gains an optional `design` field (`intent` + `allow`); a pure
  `design::lint(&Spec)` engine implements the ~10 intent-keyed rules (intent inferred
  with info-level finding when undeclared); lint never affects rendering or validation;
  each rule ships a violating/conforming unit-test pair.
- [x] **DS-06**: `ferro design:lint [path] [--json] [--deny]` CLI — recursive over spec
  JSON files, human-readable + `--json` output, exit 0 always unless `--deny`.

### Agent Surface & Docs

- [x] **DS-07**: ferro-mcp gains a `design_lint` tool (inline spec or path);
  `json_ui_catalog` extends with the canonical variant vocabulary and per-component
  design guidance; `generation_context` gains a design-system summary.
- [x] **DS-08**: New `docs/src/design-system/` chapter (principles, token v2 reference,
  variant vocabulary, pattern catalog, linting guide); single crates.io publish at the
  end of the milestone.

## Out of Scope (v16.5)

- New crates — pattern rules live in ferro-json-ui; tokens stay in ferro-theme.
- Hard validation — rendering never rejects a spec on design grounds.
- Per-size type tokens or font-weight tokens (root `font-size` is the type-scale mechanism).
- Retroactive redesign of intent templates (`ThemeTemplates`).
- Consumer adoption (gestiscilo Phase 232) — separate repo, gated on the Phase 253 publish.

## Traceability (v16.5)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| DS-01 | Phase 250 | Complete (verified 2026-07-03) |
| DS-02 | Phase 250 | Complete (verified 2026-07-03) |
| DS-03 | Phase 251 | Complete (verified 2026-07-03) |
| DS-04 | Phase 251 | Complete (verified 2026-07-03) |
| DS-05 | Phase 252 | Complete (verified 2026-07-04) |
| DS-06 | Phase 252 | Complete (verified 2026-07-04) |
| DS-07 | Phase 253 | Complete (verified 2026-07-04) |
| DS-08 | Phase 253 | Complete (published 0.2.85/0.2.86, 2026-07-04) |

---

# Requirements — v16.6 POS Component Suite — CURRENT

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
- [ ] **POS-03**: A `FilterTabs` standalone builtin filters visible tiles by
  filter token client-side (show/hide), touch targets ≥44px.
- [ ] **POS-04**: A `SelectionPanel` builtin renders the running selection as a **live
  client-side view of the form state** (operator decision 2026-07-05, un-deferring the
  CartRuntime slice): lines appear as tiles are tapped, each line has a per-line
  QuantityStepper + remove, the running total is client-computed in integer cents,
  EmptyState shows when nothing is selected, and a confirm-action slot hosts the
  single confirm POST. The panel pins and internally scrolls within a `fill_viewport`
  layout.
- [ ] **POS-05**: A `QuantityStepper` standalone builtin provides a reusable +/− numeric
  stepper on the Tile hidden-input contract, usable in selection lines and forms.
- [ ] **POS-06**: A `Numpad` builtin provides a custom tap-surface numeric keypad
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

- [ ] **POS-10**: A `ServiceDef` renders a working sale screen via a `Register` layout
  template under the **Collect** intent (builder `emit_register_root` +
  `ElementBuilder.each()` + `fill_viewport` emission) — the seven-intent vocabulary is
  unchanged.

### Agent-Authoring Boundary

- [x] **POS-11**: POS design-lint rules ship — `pos-fill-viewport`, `pos-grid-fill`,
  `pos-cart-present`, `fill-viewport-layout-unknown` — each with violating/conforming AND
  data-bound fixtures; `RULE_COMPONENTS` mapping updated.
  *(Delivered 254 under the old ids; renamed `register-fill-viewport`,
  `register-grid-fill`, `register-selection-present` in Phase 255.)*
- [ ] **POS-12**: The MCP + docs surface is extended — `json_ui_catalog` entries/count for
  the new components, `generation_context` register composition guidance, `docs/src`
  updates.

### Release

- [ ] **POS-13**: The `/cassa` sample app flips to the new components (projection-derived
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
| POS-01 | Phase 256 | Not started |
| POS-02 | Phase 254 | Not started |
| POS-03 | Phase 256 | Not started |
| POS-04 | Phase 256 | Not started |
| POS-05 | Phase 256 | Not started |
| POS-06 | Phase 256 | Not started |
| POS-07 | Phase 254 | Not started |
| POS-08 | Phase 255 | Not started |
| POS-09 | Phase 256 | Not started |
| POS-10 | Phase 257 | Not started |
| POS-11 | Phase 254 | Not started |
| POS-12 | Phase 258 | Not started |
| POS-13 | Phase 258 | Not started |
