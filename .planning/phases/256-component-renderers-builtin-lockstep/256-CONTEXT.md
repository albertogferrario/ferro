# Phase 256: Component Renderers + BUILTIN Lockstep - Context

**Gathered:** 2026-07-06 (auto mode — recommended defaults selected, logged in 256-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

All five POS builtins become first-class catalog members, and the register
interaction model (operator, 2026-07-05) is implemented end-to-end on the
Phase 255 attribute contract:

- **Render functions + registration** for `TileGrid`, `SelectionPanel`,
  `FilterTabs`, `QuantityStepper`, `Numpad` — BUILTIN_TYPES + dispatch +
  BUILTIN_SPECS + both count guards 47 → 52, bumped per component addition in
  the same commit (POS-01/03/04/05/06).
- **Tile tap-to-add redesign**: the 255 `render_tile` stepper markup is
  replaced — one tap adds one unit, NO on-tile +/- steppers or qty display;
  the 254 visual handoff (`image_url`/`color`/`stock_badge`) is rendered here.
- **SelectionPanel live view — the un-deferred CartRuntime slice** (operator,
  2026-07-05): lines appear/update as tiles are tapped, per-line stepper +
  remove, running total client-computed in integer cents, EmptyState toggle.
  The form-state contract is UNCHANGED — hidden inputs accumulate, one confirm
  POST; the panel is a client-side VIEW of that state, never a second source
  of truth. New runtime module required (`runtime/selection.rs`).
- **`Grid.row_weights` render path**: fractional `grid-template-rows` inline
  style (POS-09, 254 D-19 handoff).
- **Named 254/255 handoffs closed here**: RULE_COMPONENTS association
  extension (254 D-14), tile visuals (254 D-03), `ferro-base.css` regen after
  renderers land (254 D-08), SelectionPanel confirm slot consuming
  `ButtonProps.disable_on_submit` (255 D-16).

NOT this phase: projection-builder work / `emit_register_root` /
`ElementBuilder.each()` (Phase 257); `/cassa` flip off RawHtml (257); MCP
`generation_context` guidance + `docs/src/json-ui/components.md` full props
tables for the five components (Phase 258); publish (258); payment flow,
receipts, barcode wedge (out of scope / deferred).

**Milestone constraints carried into every decision:** builtins only (never
plugins); seven-intent vocabulary frozen; structural vocabulary only — no
domain nouns or consumer-specific props in `ferro-*` crates; every emitted
class is a full string literal; no raw palette classes; no new crates; single
publish at Phase 258.

**Design reference (operator):** Shopify POS interaction patterns — dense
image-led tile grid with instant search, side-pinned selection pane with
per-line quantity edits, horizontal filter-tab strip, large tap targets.
Inspiration for layout/density/interaction only; all visuals compose from
semantic tokens.

</domain>

<decisions>
## Implementation Decisions

### Tile tap-to-add redesign (POS-01 interaction model + 254 D-03 visuals)
- **D-01:** The whole tile becomes ONE tap surface: the tile root is rendered
  as a `<button type="button">` carrying `data-qty-inc="{field}"` — the
  shipped `initQtyButton` in `runtime/tiles.rs` binds any `[data-qty-inc]`
  element document-wide, so a tile-tap increments the tile's hidden input
  with ZERO new runtime code for the add path. The hidden input
  (`name="{field}" data-qty-input="{field}"`) stays inside the tile markup
  (existing contract; note: inputs inside a `<button>` are invalid HTML — if
  the root is a button, the hidden input moves adjacent within a wrapper;
  exact structure planner's call, contract is "tile emits exactly one hidden
  input per field, tap increments it").
- **D-02:** NO on-tile qty display, NO `data-qty-display` on the tile, NO +/-
  buttons, NO qty badge, NO picked-state ring (FEATURES.md table-stakes rows
  for badge/ring are superseded by the operator interaction model — selection
  feedback lives in the panel; press feedback comes from `PRESS_ACTIVE`).
  The 254 `product_tile_legacy_render_is_byte_identical` test (extended in
  255) is SUPERSEDED and deleted — it guarded against accidental drift before
  the designed redesign; this IS the designed redesign. Replace with new HTML
  assertions for the tap-to-add markup. `Tile`-as-renamed was never published;
  the `ProductTile`→`Tile` migration-table entry in
  `docs/src/json-ui/components.md` gains one line noting the tap-to-add
  markup redesign (full new-component docs are Phase 258).
- **D-03:** Visual composition (semantic tokens only): `image_url` renders an
  image area at the top of the tile (aspect + object-fit planner's call;
  lazy-loading consistent with existing image handling); absent → text-only
  tile (both layouts tested). `stock_badge` renders as a Badge-style overlay
  chip (existing badge class vocabulary). `color` maps through an
  **exhaustive match** on the canonical `Tone` value set to full-literal
  accent classes (unknown/absent → default border) — NEVER
  `format!("bg-{color}")`-style dynamic class construction (SC-3). If
  planning determines `Tone` is the wrong vocabulary for tile accents, the
  fallback is dropping `color` rendering this phase rather than introducing
  dynamic classes.
- **D-04:** New additive prop `TileProps.price_cents: Option<u64>`
  (`#[serde(default, skip_serializing_if = "Option::is_none")]`), emitted as
  `data-unit-price="{cents}"` on the tile root. Rationale: the client-computed
  running total needs machine-readable money; `price` is a display string and
  cannot be parsed. Runtime treats a missing attribute as 0 cents. Rustdoc
  states both facts and that `price`/`price_cents` are expected to agree (the
  257 projector emits both from one source). Integer cents ONLY — never float
  (PITFALLS.md).
- **D-05:** Touch/interaction classes on the tile root: `TOUCH_ACTION`,
  `HIT_TARGET_MIN`, `PRESS_ACTIVE`, `INTERACTIVE_BASE` from
  `render/classes.rs` (the 254 composition drift-guard must pass for every
  new render function — constants, not inline literals).

### SelectionPanel live view — CartRuntime slice (POS-04)
- **D-06:** New runtime module `runtime/selection.rs` exposing
  `setupSelection()`, wired into `FERRO_RUNTIME_JS` concatenation + the
  `ferroRuntime()` dispatcher + BOTH drift-list tests
  (`bundle_contains_all_setup_functions`, `dispatcher_invokes_every_setup`)
  in the same commit. House ES5 style (var/function, no arrows/template
  literals). No-op when no `[data-selection-panel]` exists.
- **D-07:** Reconciliation is **input-event-driven**: the runtime listens
  (delegated) for bubbling `input` events from `[data-qty-input]` fields
  within the panel's paired form scope and reconciles the panel — qty > 0 →
  ensure line exists, update qty + line total; qty = 0 → remove line; always
  recompute the running total and toggle EmptyState visibility. Tile taps,
  panel steppers, and Numpad writes ALL dispatch `input` (255 contract), so
  every mutation funnels through one code path and the panel stays a pure
  view. `setupSelection()` performs one initial reconciliation pass at load
  (covers `default_quantity > 0` server-rendered state).
- **D-08:** Line creation: `render_selection_panel` emits a
  `<template data-selection-line-template>` inside the panel; the runtime
  clones it per line and fills name/qty/line-total. Rationale: all markup and
  class literals stay in Rust render source (Tailwind scanner contract +
  single markup source); no imperative `createElement` chains in JS.
- **D-09:** Line metadata comes from the tile DOM: the runtime resolves the
  event's input → tile root (`closest`), reads the display name from
  `data-filter-text` (always present per 255 D-08) and unit price from
  `data-unit-price` (D-04). No duplicated catalog data in the panel.
- **D-10:** Per-line controls use panel-scoped **delegated** click handling
  with `data-selection-*` attributes (e.g. `data-selection-inc/dec/remove`,
  exact names planner's call) — NOT `data-qty-inc/dec` (those are bound
  per-element at load by `setupTiles`; cloned lines appear post-load and
  double-binding vs no-binding races are exactly what delegation avoids).
  Handlers mutate the SAME hidden input by field name and dispatch `input` —
  the reconciler does the rest. Remove sets the input to 0 (stepper-decrement
  to 0 removes the line too — remove-on-zero, FEATURES.md).
- **D-11:** Scoping: `form_id` = the HTML `id` of the `<form>` that owns the
  hidden inputs (`FormProps.id` already exists; `ButtonProps.form` already
  pairs). `TileGrid` and `SelectionPanel` render `<div>`s, never `<form>`s;
  the spec composes one `Form` as common ancestor (the 257 register template
  emits it). The runtime scopes queries to
  `document.getElementById(form_id)`, falling back to document when absent
  (single-register pages). Panel root emits `data-selection-panel` +
  `data-selection-form="{form_id}"`.
- **D-12:** Running total: integer-cents arithmetic (sum of qty ×
  `data-unit-price`), written to a `data-selection-total` element. Display
  formatting is attribute-driven with **neutral defaults**: additive
  SelectionPanelProps display props (exact fields planner's call — candidate
  `currency: Option<String>` symbol emitted as a data attribute; two-decimal
  formatting, "." separator default). No locale tables, no float math ever.
  Line totals use the same JS formatting helper.
- **D-13:** EmptyState: server-rendered inside the panel (reuse the existing
  EmptyState markup vocabulary; `empty_message` prop, neutral English
  default, planner's wording); runtime toggles it vs the lines container via
  inline `style.display` (255 D-11 mechanism).
- **D-14:** Confirm slot: the panel renders its element children into the
  pinned footer/confirm area — the author supplies the confirm `Button` (with
  `disable_on_submit: true` + `form: {form_id}`, closing the 255 D-16
  handoff). No dedicated button-config props on SelectionPanelProps.
- **D-15:** Panel layout contract: pins and internally scrolls under
  `fill_viewport` (`OVERSCROLL_CONTAIN`; lines container is the scroll
  region; header/total/confirm stay pinned). Verified by class-presence HTML
  assertions.

### TileGrid + FilterTabs composition (POS-01, POS-03)
- **D-16:** `render_tile_grid` root emits `data-filter-scope` (always) and
  renders, inside it: the integrated filter strip when `categories_path` is
  set, the search input when `search: true`, and the tile grid (children
  rendered via the standard child pipeline; `$each` expansion is upstream
  data-binding, not this renderer's concern). Grid columns: `columns`
  override, render default 2 (per props rustdoc), full-literal responsive
  column classes via exhaustive match on the accepted `columns` range.
- **D-17:** ONE shared Rust helper renders the tab-strip markup for BOTH the
  standalone `FilterTabs` component and TileGrid's integrated strip — single
  markup source, tabs emit `data-filter-tab="{token}"` (+ the all tab with
  empty value), ≥44px targets (`HIT_TARGET_MIN`), `TOUCH_ACTION`, semantic
  inactive-state classes exactly matching what `updateFilterTabClasses`
  toggles (`border-transparent text-text-muted hover:text-text`).
- **D-18:** Standalone `FilterTabs` participates via the **nearest ancestor
  `data-filter-scope`** (the shipped filters.rs semantics — no runtime change
  this phase). Composition constraint documented in rustdoc: place FilterTabs
  inside a filter scope; TileGrid emits one automatically and its
  `categories_path` strip is the standard register composition (SC-5 tests
  this path). A sibling-pairing mechanism (e.g. `data-filter-for` value
  matching) is DEFERRED until the 257 template or gestiscilo shows the need.
- **D-19:** Search input: rendered by TileGrid with `data-filter-search`,
  `type="search"` styling on house form-control classes, **`text-base`
  minimum font** (16px — iOS Safari zoom pitfall, STACK.md) and ≥44px hit
  height.
- **D-20:** "Uncategorized" sentinel tab: NOT rendered this phase (255 D-10
  forward-compatibility note stands — a future sentinel needs a distinct
  reserved non-empty token). Untokened tiles are visible under All only.

### QuantityStepper + Numpad renderers (POS-05, POS-06)
- **D-21:** `render_quantity_stepper` (standalone) emits: dec button
  (`data-qty-dec="{field}"`), display span (`data-qty-display="{field}"`),
  inc button (`data-qty-inc="{field}"`), and its OWN hidden input
  (`name`/`data-qty-input`) — self-contained, bound by `setupTiles` at load.
  Buttons ≥44px (`HIT_TARGET_MIN`), `TOUCH_ACTION` + `PRESS_ACTIVE`.
- **D-22:** Bounds: when `min`/`max`/`step` are set, emit
  `data-qty-min/-max/-step` attributes and extend `initQtyButton` in
  `runtime/tiles.rs` to read them (defaults 0 / unbounded / 1) — a small
  additive runtime change so the declared props are honored, not decorative.
  The SelectionPanel line stepper (D-10) shares visual classes with this
  renderer via a shared helper but emits `data-selection-*` attributes and no
  hidden input (the tile owns the input; one input per field, always).
- **D-23:** `render_numpad` emits the EXACT 255 contract: container
  `data-numpad data-numpad-target="{field}"` (+ `data-numpad-mode="price"`
  when `mode: price`), display `data-numpad-display`, 3×4 key grid `1-9` /
  `clear` / `0` / `backspace` with `data-numpad-key`, and the hidden input
  `name="{field}" data-numpad-input="{field}"` adjacent — NEVER a visible
  native input. Keys ≥56px (`HIT_TARGET_NUMPAD`), `TOUCH_ACTION`,
  `PRESS_ACTIVE`, `TAP_HIGHLIGHT`. Key labels are digits plus neutral
  glyphs/aria for backspace/clear (planner's call, no locale strings).

### Grid row_weights render path (POS-09)
- **D-24:** In `fill: true` mode with non-empty `row_weights`, the grid emits
  an inline `style="grid-template-rows: 2fr 1fr"` (weights joined as `{n}fr`)
  — exactly as SC-4 specifies. Ignored (not emitted) in scrollable mode and
  when empty (254 D-19 rustdoc already states this). Emit weights as given —
  no validation/clamping this phase (a degenerate `0fr` is author error; lint
  candidate later). Verify how fill-mode currently sizes rows in
  `containers.rs` and that the inline style takes precedence; existing Grid
  specs without `row_weights` must render byte-identically (regression
  assertion).

### BUILTIN lockstep + RULE_COMPONENTS (SC-1, 254 D-14 handoff)
- **D-25:** One commit per component registration, each containing: the
  `BUILTIN_TYPES` entry (render/mod.rs) + dispatch arm + `BUILTIN_SPECS`
  entry (catalog.rs) + BOTH count bumps (`catalog.rs:~1219` canonical,
  `ferro-mcp/src/tools/json_ui_catalog.rs:~396` mirror) + the History comment
  line — counts advance 48, 49, 50, 51, 52. Registration order planner's
  call (suggested: TileGrid, FilterTabs, QuantityStepper, Numpad,
  SelectionPanel — panel last since its runtime depends on tile emission).
- **D-26:** RULE_COMPONENTS extension (254 D-14): append the new component
  names to `register-fill-viewport` / `register-grid-fill` /
  `register-selection-present` mappings in the SAME commits that register
  TileGrid / SelectionPanel / Numpad in BUILTIN_TYPES (Direction 3 of the
  ferro-mcp guard requires mapped names to exist in catalog output — never
  weaken the guard). `REGISTER_TRIGGER_TYPES` in design/rules.rs already
  names them as raw strings — no change needed there.
- **D-27:** BUILTIN_SPECS catalog entries: each new component gets a valid
  example spec (existing pattern); the catalog validation test
  (`BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` + per-spec render smoke)
  covers them automatically.

### Locale-neutral defaults audit (project-agnostic crates principle)
- **D-28:** All user-visible defaults emitted by `ferro-json-ui` renderers
  are neutral English: `FilterTabsProps.all_label` render default is
  **"All"** (the 254 rustdoc claim "Tutte" is corrected — it predates the
  vocabulary-neutralization posture; consumers pass `all_label: "Tutte"`),
  and the redesigned tile/panel aria-labels are neutral English (the current
  `render_tile` Italian aria-labels — "Diminuisci/Aumenta quantità" — are
  removed with the stepper markup; new labels like "Add {name}",
  "Remove {name}" planner's wording). `empty_message` default likewise
  neutral English. This is an audit-fix
  (feedback_audit_report_fix_discrepancies), not scope creep.

### Gate + regen
- **D-29:** `scripts/gen-ferro-base-css.sh` runs ONCE after all five
  renderers land (254 D-08 second regen); commit the generated
  `ferro-base.css` if changed. Any class that turns out dynamic is a design
  error, not a safelist entry.
- **D-30:** Schema export artifacts (`docs/protocol/schemas/*.json`)
  regenerate with REAL changes (new props: `price_cents`, panel display
  props) — commit them with the phase (255 V-07 precedent, not the usual
  discard-churn rule).
- **D-31:** CI-exact gate before every commit: `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus `cargo doc` clean. Re-run fmt after any
  hand-edit. Serialize CPU-heavy runs (one at a time).

### Claude's Discretion
- Exact tile markup structure (button root vs button-role wrapper for valid
  HTML around the hidden input), image aspect/object-fit, text-only fallback
  layout.
- Exact `data-selection-*` attribute names; panel template internals; JS
  money-format helper naming; exact SelectionPanel display-format prop names.
- Registration order of the five components; BUILTIN_SPECS example content.
- Responsive column-class ladder for TileGrid `columns`.
- Aria-label wording (neutral English); backspace/clear key glyphs.
- Whether `design/infer.rs` gains a TileGrid → collect inference branch
  (carried discretionary from 254).

### Folded Todos
None — `todo match-phase 256` returned 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Operator-locked interaction model + requirements
- `.planning/ROADMAP.md` — v16.6 Phase 256 section: goal, SC 1–5, the
  **Design reference** and **Interaction model** paragraphs (operator,
  2026-07-05) — these two paragraphs are the phase's constitution.
- `.planning/REQUIREMENTS.md` — POS-01/03/04/05/06/09 texts; the
  **UN-DEFERRED CartRuntime** paragraph under Future Requirements (what the
  slice includes and what it must NOT — no `data-cart-target` props hook);
  the Vocabulary decision (2026-07-05).

### Milestone research (2026-07-04 — read through the 255 rename map)
- `.planning/research/STACK.md` — **§Cart Runtime Module is the operator-named
  design anchor** for the selection runtime (integer cents, input-event
  delegation, data-unit-price); §Hit Target Standards (44/56px rationale);
  §Vanilla-JS Patterns (numpad contract, no-optimistic-fetch posture);
  16px-input iOS zoom pitfall.
- `.planning/research/PITFALLS.md` — integer-cents money rule; sub-44px
  targets; token bypass / safelist drift; lockstep pitfalls.
- `.planning/research/FEATURES.md` — remove-on-zero, running total,
  empty-cart-state evidence; §Composition diagram; NOTE: the on-tile qty
  badge / picked-ring table-stakes rows are SUPERSEDED by the operator
  tap-to-add model (D-02).
- `.planning/research/ARCHITECTURE.md` — file/line anchors (pre-rename
  naming; read through V-01..V-05).

### Prior phase contracts this phase implements against
- `.planning/phases/255-pos-runtime-modules-double-submit-protection/255-CONTEXT.md`
  — V-01..V-08 (final vocabulary), D-01..D-06 (numpad contract render must
  emit), D-07..D-12 (filter contract render must emit), D-16 (ButtonProps
  `disable_on_submit` → panel confirm slot).
- `.planning/phases/254-props-contracts-touch-foundation-design-rules/254-CONTEXT.md`
  — D-03 (tile visual handoff), D-07 (composition drift-guard covers new
  render functions automatically), D-08 (css regen cadence), D-14
  (RULE_COMPONENTS same-commit rule), D-17 (behavioral contract anchors),
  D-19 (row_weights semantics).
- `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — the
  gestiscilo picker audit (battle-tested spec for contract tiebreaks).

### Source anchors (post-rename, current on this branch)
- `ferro-json-ui/src/component.rs` :1359–1475 — the five Props structs +
  `TileProps` (D-04 adds `price_cents`); `FormProps.id` :275; `ButtonProps`.
- `ferro-json-ui/src/render/atoms.rs` :1365 — current `render_tile` (the
  markup D-01/D-02 replaces, incl. the Italian aria-labels D-28 removes).
- `ferro-json-ui/src/render/classes.rs` — TOUCH_ACTION / HIT_TARGET_MIN /
  HIT_TARGET_NUMPAD / PRESS_ACTIVE / OVERSCROLL_CONTAIN / TAP_HIGHLIGHT +
  composition drift-guard.
- `ferro-json-ui/src/render/mod.rs` :67, :200 — BUILTIN_TYPES + dispatch.
- `ferro-json-ui/src/render/containers.rs` — Grid render (row_weights D-24
  lands here; child-render pipeline pattern for TileGrid).
- `ferro-json-ui/src/runtime/tiles.rs` — `initQtyButton` (document-wide field
  lookup D-01 exploits; D-22 extends with min/max/step).
- `ferro-json-ui/src/runtime/filters.rs` — the shipped scope semantics D-16/
  D-18 target (header comment documents the full attribute contract).
- `ferro-json-ui/src/runtime/numpad.rs` — the shipped numpad runtime D-23
  emits against (incl. `data-numpad-mode`).
- `ferro-json-ui/src/runtime/form_guards.rs` — number-guard input collection;
  `data-disable-on-submit` guard the confirm slot rides on.
- `ferro-json-ui/src/runtime/mod.rs` — bundle concat, dispatcher, both
  drift-list tests (D-06 extends).
- `ferro-json-ui/src/catalog.rs` :1213–1219 — canonical count guard +
  History comment; BUILTIN_SPECS sync test :574.
- `ferro-mcp/src/tools/json_ui_catalog.rs` :99–103 — RULE_COMPONENTS (D-26
  extends); :~396 mirror count (D-25 bumps).
- `docs/src/json-ui/components.md` — migration table (D-02 adds the redesign
  note).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `initQtyButton` (runtime/tiles.rs) does document-wide
  `[data-qty-display="f"]` / `[data-qty-input="f"]` lookups — a tile root
  carrying `data-qty-inc` and a panel stepper for the same field both work
  against one hidden input with no new add-path runtime (the roadmap
  interaction-model paragraph calls this out explicitly).
- `runtime/filters.rs` header comment = the full filter attribute contract;
  `updateFilterTabClasses` defines the exact active/inactive class sets the
  D-17 tab-strip helper must emit as initial state.
- `runtime/numpad.rs` = the full numpad attribute contract incl.
  `data-numpad-mode`; render_numpad just emits it.
- `render/classes.rs` POS touch constants + composition drift-guard (254
  D-07) auto-covers new render functions — import constants, never inline.
- `render_empty_state` / badge / image class vocabularies in atoms.rs for the
  panel EmptyState, stock badge chip, and tile image.
- `FormProps.id` + `ButtonProps.form` — the form_id pairing mechanism already
  exists end-to-end (form_guards resolves `form="<id>"` buttons too).
- `catalog.rs` BUILTIN_SPECS render-smoke machinery auto-validates new
  example specs.

### Established Patterns
- ES5-only runtime JS; one `setup*` per concern; no-op when targets absent;
  delegated events for post-load DOM (template-cloned lines).
- Every emitted class = full string literal in crate source (JS literals in
  runtime/*.rs count — the scanner and the
  `variant_classes_use_semantic_tokens` scan cover them).
- Additive props: `#[serde(default, skip_serializing_if = …)]` + rustdoc +
  schema smoke test + round-trip test.
- Lockstep: count bumps in the same commit as the component addition; History
  comment as audit trail; both guards (canonical + ferro-mcp mirror).
- CI-exact gate incl. `--all-features`; fmt after any hand-edit.

### Integration Points
- Phase 257 consumes: registered `TileGrid`/`SelectionPanel` in
  BUILTIN_TYPES (`catalog_validate` gate), the TileGrid `$each` children
  contract, `fill_viewport` + `row_weights` render, the Form-ancestor
  composition (D-11).
- Phase 258 consumes: the 52-count MCP catalog, the composition patterns for
  `generation_context`, components.md props tables for all five.
- gestiscilo register phase consumes: everything, via the 258 publish.

</code_context>

<specifics>
## Specific Ideas

- The phase's killer feature is the **live SelectionPanel** — the moment a
  tile tap makes a line appear with a running total, the register feels like
  a product instead of a form. The reconciler design (D-07: one input-event
  code path, panel as pure view) is what makes it safe: no cart state to
  desync, ever. Spend the polish budget there.
- Tap-to-add works by REUSING the existing tiles runtime (tile root carries
  `data-qty-inc`) — resist any temptation to write a new "add" runtime.
- Shopify POS is the density/interaction reference: image-led tiles, pinned
  panel, instant search. Structural vocabulary + semantic tokens still rule —
  no commerce nouns, no raw palette.
- Contract tiebreaks during planning: "what does the gestiscilo picker
  actually need" (253-FRICTION.md is the battle-tested spec).

</specifics>

<deferred>
## Deferred Ideas

- **Per-line extra columns generic mechanism** (255 V-01 named handoff from
  removing `show_staff`/`show_people`): DEFERRED past 256 — the operator's
  tap-to-add redesign postdates that handoff and the live-view line
  (name/qty/line-total/remove) has no evidence-backed need for extra columns;
  designing the mechanism now would create a speculative control surface.
  Revisit on gestiscilo register-adoption friction.
- **Sibling FilterTabs↔TileGrid pairing** (`data-filter-for` value matching)
  — deferred until the 257 register template or a consumer needs
  out-of-scope tab placement (D-18).
- **"Uncategorized" virtual sentinel tab** — still deferred (D-20); needs a
  reserved non-empty token when it comes.
- **Qty badge / picked-state ring on tiles** — superseded by the tap-to-add
  model this phase; revisit only on operator redirection.
- **`row_weights` validation lint** (degenerate `0fr`, weight/child count
  mismatch) — lint candidate, not a render concern.
- **Barcode keyboard-wedge**, **payment flow / receipts / shift close** —
  standing milestone deferrals.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.

</deferred>

---

*Phase: 256-component-renderers-builtin-lockstep*
*Context gathered: 2026-07-06*
