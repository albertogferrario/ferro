# Phase 254: Props Contracts + Touch Foundation + Design Rules - Context

**Gathered:** 2026-07-05 (auto mode — recommended defaults selected, logged in 254-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

Lock the POS component API contracts, shared touch primitives, and design-lint
rules before any render code (Phase 256) or runtime JS (Phase 255) is written,
preventing contract thrash when the renderers are built. Concretely:

- `ProductTileProps` additive extension + `data-product-categories` attribute
  emission (POS-02)
- Shared POS touch constants in `render/classes.rs` with a composition
  drift-guard (POS-07)
- Four POS design-lint rules with violating/conforming/data-bound fixtures +
  `RULE_COMPONENTS` entries (POS-11)
- All five new `*Props` struct declarations (ProductGridProps, CartPanelProps,
  CategoryNavProps, QuantityStepperProps, NumpadProps) — declared, NOT registered
- `row_weights` prop on `GridProps` (schema substrate only; render path is 256)

NOT this phase: any new render function, any BUILTIN_TYPES/dispatch/BUILTIN_SPECS
change (count stays 47, both drift guards untouched), any runtime JS module,
any projection-builder work. Requirements: POS-02, POS-07, POS-11.

**Milestone constraints carried into every decision:** form-state cart only
(CartRuntime deferred to Future Requirements); CategoryNav IS a standalone
builtin (operator decision 2026-07-04 — do not relitigate the research
recommendation); seven-intent vocabulary frozen; all POS components are
builtins; no new crates; single publish at Phase 258.

</domain>

<decisions>
## Implementation Decisions

### ProductTile additive props (POS-02)
- **D-01:** The category field is **`categories: Vec<String>`** (plural), with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`. Rendered as a
  space-separated `data-product-categories` attribute, emitted only when
  non-empty. Deliberate deviation from the singular `category` shorthand in
  REQUIREMENTS/ROADMAP prose: the attribute contract is plural in both the 254
  and 255 success criteria, multi-category products (gestiscilo evidence) must
  filter under each membership, and a one-element vec covers the singular case.
- **D-02:** `image_url: Option<String>`, `color: Option<String>`,
  `stock_badge: Option<String>` — all
  `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- **D-03:** Phase 254's renderer touch is **criteria-exact**:
  `render_product_tile` gains the `data-product-categories` emission (the Phase
  255 filter runtime contract) but the VISUAL rendering of
  `image_url`/`color`/`stock_badge` is a named handoff to Phase 256, where tile
  visuals are designed together with ProductGrid. Existing specs (new fields
  absent) must produce byte-identical HTML — assert with a render-output
  equality test alongside the serde test.
- **D-04:** Serde backward-compat test per SC-1: legacy ProductTile JSON without
  the new fields deserializes cleanly and re-serializes without the new keys
  (round-trip unchanged).

### Shared POS touch foundation (POS-07)
- **D-05:** Five named constants in `render/classes.rs`, names exactly as SC-2:
  `POS_TOUCH_ACTION` (touch-manipulation), `POS_HIT_TARGET_MIN`
  (min-h-[44px] min-w-[44px]), `POS_PRESS_ACTIVE` (`:active` press state on the
  motion tokens — duration-fast/ease-base family), `POS_OVERSCROLL_CONTAIN`
  (overscroll-contain), `POS_TAP_HIGHLIGHT` (tap-highlight reset). Every
  constant is a complete class literal (Tailwind `@source` scanner contract,
  documented at the top of classes.rs); token-sourced, no raw palette classes.
  Exact class picks inside POS_PRESS_ACTIVE / POS_TAP_HIGHLIGHT are planner's
  call against `.planning/research/STACK.md` (candidates: `active:scale-95`,
  `active:bg-border`, arbitrary-property `[-webkit-tap-highlight-color:transparent]`
  — verify Tailwind v4 emits the arbitrary-property utility before committing
  to it).
- **D-06:** `render_product_tile` migrates its inline `touch-manipulation` and
  `min-h-[44px] min-w-[44px]` literals to the new constants in this phase —
  zero visual change (identical class strings), and the drift guard gets its
  first real consumer.
- **D-07:** The constant-composition drift-guard test (SC-2) follows the
  existing source-scan/composition test patterns
  (`interactive_base_is_motion_fast_plus_focus_ring` in classes.rs,
  `variant_classes_use_semantic_tokens`): assert POS render functions reference
  the constants rather than inlining the raw literals. Mechanism is planner's
  discretion, but it must automatically cover the Phase 256 render functions
  when they land (e.g. scan render sources for the raw literal strings outside
  classes.rs) — a guard that needs manual re-enrollment per component is drift
  waiting to happen.
- **D-08:** Run `scripts/gen-ferro-base-css.sh` ONCE at phase end: the new
  constant literals are scanner-visible the moment they exist in crate source,
  so the generated CSS changes this phase even before any renderer uses them.
  Phase 256 re-runs the regen after the renderers land. No `@source inline()`
  safelist additions expected — everything ships as full literals (if any class
  turns out dynamic, that's a design error, not a safelist entry).

### POS design-lint rules (POS-11)
- **D-09:** Four rules join `design/rules.rs::RULE_REGISTRY` (11 → 15):
  `pos-fill-viewport`, `pos-grid-fill`, `pos-cart-present`,
  `fill-viewport-layout-unknown`. All four are `Severity::Warning` — they guard
  broken-register hazards (silent page scroll, dead panes, cart-less registers)
  and must trip `--deny` in consumer CI. Diagnostics-only, pure, pre-expansion
  (252 D-12 unchanged).
- **D-10:** Rule semantics locked at intent level (predicates refined in
  planning):
  - `pos-fill-viewport`: element map contains POS component type names
    (trigger set — ProductGrid/CartPanel/Numpad at minimum — planner's call)
    but `Spec.fill_viewport != true` → warning.
  - `pos-grid-fill`: a `fill_viewport` spec whose register-root `Grid` lacks
    `fill: true` → warning (panes silently lose internal scroll).
  - `pos-cart-present`: `ProductGrid` present with no `CartPanel` anywhere in
    the element map → warning (incomplete register composition).
  - `fill-viewport-layout-unknown`: `fill_viewport: true` with `Spec.layout`
    outside the set the `ferro-fill` CSS chain actually supports → warning
    (fill silently degrades to whole-page scroll — the classic cassa bug).
    **Research directive:** determine the exact supported-layout set from the
    `assets/input.css` chain (`body.ferro-fill > div.flex > main …`) plus the
    layout registry. The registry ships `default`/`app`/`auth`
    (`layout.rs:669-671`) while 252 D-14 claimed `dashboard`/`app`/`auth` —
    re-verify the world-state claim before encoding the set
    (feedback_validate_scope_premises).
- **D-11:** All four rules use `intents: &[]` (all-intents) with **internal
  presence gates** (POS type names / `fill_viewport` flag), NOT
  `intents: &["collect"]`. Rationale: intent-keyed rules only run when the
  declared-or-inferred intent matches, and the inference heuristics predate POS
  components — an agent-authored ProductGrid spec with no declared intent would
  silently skip collect-keyed rules. This mirrors `page-header`'s
  internal-layout-gate pattern. Optionally extend `design/infer.rs` with a
  ProductGrid → collect inference branch (discretionary).
- **D-12:** Three fixtures per rule per SC-3: violating (expected severity
  asserted), conforming (zero findings from that rule), and data-bound
  (`$data.*`-scoped props, asserting no misfire) — the 252-PATTERNS.md
  false-positive fixture class is mandatory here, not optional.
- **D-13:** Matching POS type names as strings against components that are not
  yet registered builtins is correct and intentional — lint operates on the raw
  spec (252 D-12) and never consults BUILTIN_TYPES.

### RULE_COMPONENTS + ferro-mcp guard sequencing
- **D-14:** Adding the four rules to the registry FORCES `RULE_COMPONENTS`
  entries in this phase (guard Direction 2: every registry id must be mapped —
  `json_ui_catalog.rs:~750`) and FORCES existing-builtin-only associations
  (Direction 3: every mapped component name must be in catalog output;
  ProductGrid/CartPanel/Numpad are not builtins until Phase 256). Resolution:
  map all four rules to the closest existing builtin — recommended `&["Grid"]`
  (the register-root component all four rules structurally concern) — and
  extend the associations to the new component names in Phase 256, in the same
  commit that registers them in BUILTIN_TYPES. This is a named 256 handoff. An
  empty-slice mapping is acceptable for `fill-viewport-layout-unknown` if the
  planner verifies no per-component-guidance test breaks; never weaken the
  Direction 3 assertion itself.
- **D-15:** Component count stays 47 — both drift-guard assertions
  (`catalog.rs:1219` canonical, `json_ui_catalog.rs:396` mirror) are untouched
  this phase. No BUILTIN_TYPES, dispatch, or BUILTIN_SPECS changes.

### Five new Props structs (substrate for 256)
- **D-16:** `ProductGridProps`, `CartPanelProps`, `CategoryNavProps`,
  `QuantityStepperProps`, `NumpadProps` are declared in
  `ferro-json-ui/src/component.rs` with the crate's full derive set
  (Debug/Clone/PartialEq/Serialize/Deserialize/JsonSchema), serde conventions
  matching existing Props, rustdoc on every struct and field, and one D-32
  schema smoke test each (`schema_smoke_tests` module pattern). They are NOT
  registered anywhere. Pub items in a lib crate produce no dead-code warnings;
  if clippy flags anything under `--all-features`, fix structurally — no
  `#[allow]`.
- **D-17:** Behavioral contract anchors locked now (field-level naming/types
  are planning work against `.planning/research/INVENTORY-PRIMITIVES.md`,
  `FEATURES.md`, and the gestiscilo picker evidence):
  - `ProductGridProps`: products iterate via the `$each` children contract
    (the Phase 257 `ElementBuilder.each()` target); integrated category strip
    (`categories_path`) + client-side search toggle; tiles keep the existing
    hidden-input form contract (`data-qty-input` / `name={field}`).
  - `CartPanelProps`: server-rendered line items (`$data`-bound); per-line
    quantity stepper + remove affordance; running total; EmptyState when
    empty; confirm-action slot; the contract must support pin + internal
    scroll under `fill_viewport`.
  - `CategoryNavProps`: standalone builtin (operator-locked); category list
    data-bindable; filter contract is client-side `data-product-categories`
    matching (Phase 255 runtime).
  - `QuantityStepperProps`: reusable +/− stepper on the ProductTile
    hidden-input contract (targets a field name); bounds/step fields are
    planner's call.
  - `NumpadProps`: tap-surface keypad writing to a declared target field;
    NEVER a native input (software keyboard must not trigger); `mode`
    (quantity | price) per research — final shape planner's call.
- **D-18:** NO CartRuntime extension hooks in any Props contract (no
  `data-cart-target` prop, no cart-state fields). CartRuntime is deferred
  (REQUIREMENTS.md Future Requirements); pre-adding its hooks would create a
  control surface for a feature that may change shape.
- **D-19:** Grid `row_weights`: additive `row_weights: Vec<u8>` on `GridProps`
  mirroring the `spans` conventions —
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, positionally
  aligned with children, meaningful only with `fill: true`, ignored in
  `scrollable` mode (state all of this in rustdoc now). Phase 254 delivers the
  schema + round-trip test only; the render path (fractional
  `grid-template-rows` via inline style, per 256 SC-4) is Phase 256.

### Gate
- **D-20:** CI-exact gate before commit: `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus the docs build (`cargo doc` clean) — the
  new pub structs ship rustdoc. Re-run fmt after any hand-edit.

### Claude's Discretion
- Exact class strings inside `POS_PRESS_ACTIVE` / `POS_TAP_HIGHLIGHT`
  (token-compliant, full literals, verified emittable by Tailwind v4).
- Drift-guard test mechanism (source-scan vs composition equality) within the
  D-07 auto-coverage constraint.
- Lint predicate details: exact trigger sets, register-root-Grid
  identification, `element_id` attribution and `suggestion` text per finding.
- Whether a `POS_HIT_TARGET_NUMPAD` (56px) constant ships now or in 256.
- Field-level naming and types inside the D-17 behavioral contracts.
- Whether `infer.rs` gains a ProductGrid → collect inference branch.

### Folded Todos
None — `todo match-phase 254` returned 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone research (2026-07-04 — anchor for all v16.6 phases)
- `.planning/research/SUMMARY.md` — Phase-1 mapping (§Implications), pitfalls
  digest, open-decision synthesis (NOTE: operator resolved the open decisions
  AGAINST parts of the synthesis — CartRuntime deferred, CategoryNav
  standalone; ROADMAP/REQUIREMENTS are authoritative, not the synthesis).
- `.planning/research/PITFALLS.md` — sub-44px targets, safelist drift,
  lockstep, token bypass, data-bound lint misfires.
- `.planning/research/STACK.md` — touch-action/press-state/overscroll/
  tap-highlight class candidates and iOS Safari constraints (D-05 source).
- `.planning/research/FEATURES.md` + `.planning/research/INVENTORY-PRIMITIVES.md`
  — field-level evidence for the D-17 Props contracts (gestiscilo picker).
- `.planning/research/ARCHITECTURE.md` — build-sequence rationale, exact
  file/line anchors for every integration point.

### Seed friction (why this milestone exists)
- `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — the
  ~1500-line RawHtml cassa picker audit; Gap 1 (scroll displacement → the
  fill-viewport-layout-unknown rule), Gap 2 (mobile row weighting → D-19).

### Planning
- `.planning/ROADMAP.md` — v16.6 section: milestone scope constraints
  (builtins-only, intent vocabulary frozen, form-state cart) + Phase 254
  details (goal, SC 1–4).
- `.planning/REQUIREMENTS.md` — POS-02, POS-07, POS-11; Scope decisions
  (2026-07-04); Future Requirements (CartRuntime/barcode/fill-chain deferrals).

### Prior phase decisions (vocabulary and posture this phase extends)
- `.planning/phases/252-design-module-lint-cli/252-CONTEXT.md` — D-04
  (Severity vocabulary), D-10 (static registry), D-12 (pure pre-expansion
  lint), D-13 (`allow` semantics).
- `.planning/phases/252-design-module-lint-cli/252-PATTERNS.md` — the
  data-binding false-positive fixture class (mandatory per D-12 here).
- `.planning/phases/251-component-variant-discipline-interactive-state-pass/251-PATTERNS.md`
  — full-literal match-arm rule, semantic-token enforcement.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/render/classes.rs` — the exact home for the POS constants;
  existing FOCUS_RING/MOTION_FAST/DISABLED_BASE/INTERACTIVE_BASE + composition
  drift tests are the pattern to extend (59 lines, read it whole).
- `ferro-json-ui/src/component.rs:1345` — current `ProductTileProps`
  (product_id, name, price, field, default_quantity); `GridProps` at :882 with
  the `spans`/`fill` conventions `row_weights` mirrors; `schema_smoke_tests`
  module (D-32 pattern) for the five new smoke tests.
- `ferro-json-ui/src/render/atoms.rs:1357` — `render_product_tile`: already
  emits `touch-manipulation`, `min-h-[44px] min-w-[44px]`, INTERACTIVE_BASE,
  and the `data-qty-dec/inc/display/input` contract; D-03/D-06 land here.
- `ferro-json-ui/src/design/rules.rs` — `RULE_REGISTRY` (11 rules), the
  `DesignRule { id, title, rationale, intents, check }` shape, and the
  internal-gate pattern (`page-header`) D-11 mirrors.
- `ferro-json-ui/src/design/types.rs:13` — `Severity { Info, Warning }`;
  `Finding` shape (rule, element_id, severity, message, suggestion).
- `ferro-mcp/src/tools/json_ui_catalog.rs:81` — `RULE_COMPONENTS` static +
  the three-direction drift guard (~:740) that forces D-14.
- `ferro-json-ui/src/runtime/product_tiles.rs` — the existing hidden-input
  runtime contract the Props contracts must stay compatible with.

### Established Patterns
- Additive serde fields: `#[serde(default, skip_serializing_if = ...)]` —
  Option::is_none / Vec::is_empty; round-trip tests for present + absent.
- Every emitted class is a full string literal in crate source (Tailwind v4
  scanner contract); dynamic construction requires exhaustive match arms or
  `@source inline()` — neither expected this phase.
- Drift guards over conventions: composition tests in classes.rs, count
  assertions in catalog.rs:1219 + json_ui_catalog.rs:396 (untouched at 47),
  bidirectional rule-mapping guard in ferro-mcp.
- CI-exact gate incl. `--all-features` clippy/test (local convenience gates
  miss `--all-features`-only failures).

### Integration Points
- Phase 255 consumes: `data-product-categories` (filter runtime), the Numpad
  target-field contract, `data-disable-on-submit` (255's own).
- Phase 256 consumes: all five Props structs, the POS constants (every new
  render function imports them — drift-guarded), the RULE_COMPONENTS
  association extension (D-14 handoff), ProductTile visual rendering (D-03
  handoff), Grid row_weights render path (D-19 handoff).
- Phase 257 consumes: the ProductGridProps `$each` iteration contract.

</code_context>

<specifics>
## Specific Ideas

- The phase's function is thrash prevention: every contract locked here is one
  the 256 renderers, 255 runtime, and 257 projector implement against without
  renegotiation. When a contract question arises during planning, the tiebreak
  is "what does the gestiscilo picker actually need" (253-FRICTION.md is the
  battle-tested spec).
- "Structural guarantees over one-off fixes": the composition drift-guard
  (D-07) and the forced RULE_COMPONENTS sequencing (D-14) follow the same
  philosophy as 251's enum-set guard and 252's registry drift test — make
  drift compile-visible, don't document it.
- Lint findings should read like a good reviewer (252 posture): message states
  what breaks on a real register (page scrolls, cart off-screen), suggestion
  states the concrete fix (`set fill_viewport: true`, `add fill: true to the
  root Grid`).

</specifics>

<deferred>
## Deferred Ideas

- **ProductTile visual rendering** of `image_url`/`color`/`stock_badge` —
  Phase 256 (D-03 named handoff; tile visuals designed with ProductGrid).
- **RULE_COMPONENTS association extension** to the new component names —
  Phase 256, same commit as their BUILTIN_TYPES registration (D-14 handoff).
- **Runtime modules** (`setupNumpad`, `setupPosFilter`) and
  `data-disable-on-submit` double-submit guard — Phase 255 (POS-08).
- **CartRuntime** (live cart JS), **barcode keyboard-wedge**,
  **layout-name-independent ferro-fill chain** — already recorded in
  REQUIREMENTS.md Future Requirements; not re-opened here.
- **infer.rs ProductGrid → collect branch** — discretionary this phase; if
  skipped, revisit when gestiscilo authors register specs (FRICTION loop).

### Reviewed Todos (not folded)
None — no pending todos matched this phase.

</deferred>

---

*Phase: 254-props-contracts-touch-foundation-design-rules*
*Context gathered: 2026-07-05*
