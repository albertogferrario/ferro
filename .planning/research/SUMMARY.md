# Project Research Summary

**Project:** ferro v16.6 — POS Component Suite
**Domain:** Touch-first sale-screen components in the ferro-json-ui builtin catalog, consumer-paired with gestiscilo's register/counter mode
**Researched:** 2026-07-04
**Confidence:** HIGH

## Executive Summary

v16.6 adds a suite of POS-specific builtin components to ferro-json-ui that allow agents and spec authors to compose a fully functional touch-first sale screen from a `ServiceDef` declaration, without escaping to `RawHtml`. The milestone is motivated by a concrete friction audit: gestiscilo's cassa pages carry ~1500 lines of RawHtml escape hatches — four hand-built HTML+JS fragments inside `build_product_picker_html()` — that the framework cannot currently replace because it lacks a cart synchronization runtime and category-filter component. The goal is a catalog-grade component suite that closes that gap entirely.

The architecture is an extension of the existing builtin pipeline: new `*Props` structs → render functions → `BUILTIN_TYPES`/`BUILTIN_SPECS` lockstep → drift-guard count bumps → `gen-ferro-base-css.sh` regen → optional runtime JS modules. No new Rust crates, no new JS libraries, no new CSS tooling are needed. Every touch-ergonomics requirement (300ms-delay elimination, press states, scroll chain prevention, hit target floors) is met by CSS platform properties and vanilla JS following the existing `runtime/*.rs` pattern. The Tailwind v4 scanner/safelist discipline from Phase 251 applies unchanged; the semantic-token enforcement from v16.5 applies to every new render function without exception.

The primary risks are behavioral, not architectural: the cart-runtime scope decision (whether the milestone ships an in-memory JS cart runtime or defers it) is the central open question that requirements must resolve before implementation begins. If deferred, gestiscilo cannot migrate — the RawHtml escape hatch stays. All CSS safelist drift, BUILTIN_TYPES lockstep, and design-lint false-positive risks are known failure modes with established prevention patterns from Phases 251-253; they are checklisted, not novel.

---

## Key Findings

### Recommended Stack

No new dependencies. All requirements are covered by the existing stack: CSS platform properties (`touch-action: manipulation`, `:active`, `overscroll-contain`, `select-none`), vanilla JS modules following `runtime/product_tiles.rs` as the pattern, Tailwind v4 utility classes, and the existing `input.css @source inline()` safelist mechanism. The `gen-ferro-base-css.sh` regen step is the only build artifact that changes.

**Core technologies:**
- `touch-action: manipulation` (CSS) — eliminates the 300ms click-event delay and double-tap zoom on all POS tap surfaces; iOS Safari safe since 9.3; `touch-action: none` and per-axis variants are NOT iOS-safe
- `runtime/*.rs` vanilla JS modules — numpad key handling, cart total arithmetic, barcode scanner wedge detection; each a `pub(super) const SOURCE: &str`; no-op when their DOM markers are absent
- `input.css @source inline()` safelist — required for any Tailwind class built via `format!()` or runtime string concatenation; POS additions: `active:scale-95`, `active:bg-border`, `overscroll-contain`, `select-none`, grid-row fraction utilities
- `BUILTIN_TYPES` / `BUILTIN_SPECS` lockstep + drift guard — canonical count in `catalog.rs:1219`, mirror in `ferro-mcp/src/tools/json_ui_catalog.rs:396`; both bumped in the same commit per addition; current count is 47

**Critical version constraints:**
- Input font-size floor: 16px (`text-base`) on all visible inputs inside `fill_viewport` specs — iOS Safari auto-zooms on anything smaller, breaking the fixed-layout register screen
- Hit target floor: `min-h-[44px]` minimum, 48-56px preferred for numpad keys; enforced at Rust render time (not spec/lint level), matching the existing `ProductTile` precedent

---

### Expected Features

The gestiscilo `build_product_picker_html()` function (~1100 lines) is the canonical requirements spec. Its four output fragments map directly to the four new components:

**Must have (table stakes):**
- `ProductGrid` — responsive tile grid with integrated category filter strip and client-side search; each tile emits hidden `<input name="qty_{id}">` for form integration; the headline component, without which the RawHtml escape hatch cannot close
- `CartPanel` — scrollable line-item list with qty +/- per row, remove-on-zero, running total header, empty state; co-dependent with `ProductGrid` via a shared `form_id` scope isolator
- `CartRuntime` JS module (`runtime/cart_runtime.rs`) — the synchronization kernel that propagates qty changes between `ProductGrid` tiles and `CartPanel` rows, recomputes line totals and cart total in integer cents, drives qty-badge overlays on tiles; **this is the load-bearing consolidation piece** (see Open Decision 1)
- `ProductTile` extensions — `categories: Vec<String>` (as `data-product-categories` for filter), `image_url: Option<String>`, `color: Option<String>`, `stock_badge: Option<String>`; backward-compatible additions; runtime-activated qty-badge and picked-ring behaviors
- Touch ergonomics enforced by construction — `touch-action: manipulation`, `select-none`, `:active` press states, `overscroll-contain` on scrollable panes, `min-h-[44px]` on all interactive elements — not expressible at spec level, enforced in every render function

**Should have (differentiators):**
- `Numpad` component — 3x4 touch grid driving a target input; `mode: quantity | price`; minimum 56x56px keys; never a native `<input type="number">` (software keyboard breaks fill_viewport layout)
- Mobile asymmetric row weighting — product pane taller than cart on small screens; `row_weights` prop on `GridProps` analogous to the existing `spans` prop for column distribution (253-FRICTION.md Gap 1); scope decision: v16.6 or deferred
- `DataTable density: compact` — tighter cart row padding for dense mobile card view; additive prop; low risk
- Barcode scanner keyboard-wedge runtime (`runtime/barcode_scanner.rs`) — 40-line timing heuristic detecting USB HID scan bursts vs. human typing; `data-barcode-max-gap` attribute for Bluetooth tuning

**Defer (v17+):**
- Payment tender screen, receipt rendering, shift/session close — explicitly out of scope by milestone boundary; every plan review carries a one-line out-of-scope reminder
- Offline / service worker mode — architectural departure from the server-rendered model
- Per-line discount entry, product modifiers, customer loyalty widget — application-layer concerns
- DataList-backed customer autocomplete — standard `Input` + `datalist` already handles this

---

### Architecture Approach

All new components follow the same 11-file lockstep checklist established for existing builtins (Props struct -> render function -> `BUILTIN_TYPES` -> dispatch arm -> `BUILTIN_SPECS` -> catalog drift guard -> ferro-mcp mirror -> `gen-ferro-base-css.sh` -> runtime module -> docs). The build sequence is parallelizable within waves but sequential across them: Props contracts first, then render functions, then the BUILTIN lockstep in one atomic commit per component, then the projection builder extension.

**Major components:**
1. `ProductGridProps` + `render_product_grid` — composite: owns tile iteration via `$each`, CategoryStrip rendering (integrated, not a separate builtin), search input, hidden qty inputs; new runtime `setupProductGrid()` handling filter, search, qty-badge, picked-ring
2. `CartPanelProps` + `render_cart_panel` (containers.rs) — composite: scrollable line-item list with qty controls per row; `form_id` links to `ProductGrid`; new runtime `setupCartPanel()` inside `CartRuntime`
3. `CartRuntime` JS — the behavioral kernel; `setupCartRuntime()` listens on `input` events from qty fields, recomputes totals in integer cents, syncs tile badges, drives empty state; this is the load-bearing consolidation piece that Option B defers and Option C ships
4. `NumpadProps` + `render_numpad` — standalone; `setupNumpad()` dispatches on `data-numpad-key` attributes; no round-trip
5. `ProductTileProps` extensions — additive fields; `setupProductTiles()` extended for badge/ring behavior when `CartRuntime` is present
6. `builder.rs` Register arm — `emit_register_root()` emitting fill-viewport Grid with cart_pane + products_pane; extends the Collect intent layout template; `Spec::builder().fill_viewport(bool)` required
7. Three new design-lint rules — `pos-fill-viewport`, `pos-cart-present`, `pos-grid-fill`; all scoped to `intents: &["collect"]`; every rule ships with both a static and a `$data.*`-bound fixture

**Intent mapping:** Collect. No new intent. `cassa.json` already declares `intent: "collect"`. The POS sale screen is the Register layout template variant of the Collect intent — a new `layout: "Register"` arm in `builder.rs::build_display_spec()`, not an eighth intent variant. This is consistent with the v16.5 decision (archetypes ARE the seven intents) and the `KNOWN_INTENTS` drift guard.

---

### Critical Pitfalls

1. **Sub-44px touch targets baked into visual-first sizing** — dashboard components size by visual element (`py-2` on `text-sm` approx 32px); POS components must enforce `min-h-[44px]` (preferred 48-56px for numpad keys) at the Rust render layer by construction, not as a theme override. The existing `ProductTile` is the correct precedent: `min-h-[44px] min-w-[44px]` is already hard-coded in `atoms.rs:1357`. Every new POS render function reproduces this guarantee. Verification: compute style on a real iPad, not just HTML assertion.

2. **Double-tap zoom and ghost clicks** — iOS Safari synthesizes a delayed `click` ~300ms after `touchend`; rapid taps submit forms twice. Three mechanisms in combination: (a) `touch-action: manipulation` on every interactive POS element (eliminates delay + zoom, zero JS), (b) server-side idempotency key in every cart-mutation form via the existing `framework::write` idempotency hook, (c) `data-disable-on-submit` in the existing runtime. All three must ship before gestiscilo adoption.

3. **Whole-page scroll displacement** — the Phase 253 `fill_viewport` fix is a fragile CSS selector chain (`body.ferro-fill > div.flex > main > ...`) that silently falls back to scrolling when the layout name is unknown. The `fill-viewport-layout-unknown` lint rule must exist before agents author POS specs, or the classic cassa bug (cart panel scrolling off screen) recurs invisibly.

4. **CSS safelist drift** — `format!("grid-cols-{}", n)` and similar runtime-concatenated class names are invisible to the Tailwind v4 scanner and absent from `ferro-base.css`. This is the Phase 253 WR-01 failure mode (`col-span-{2,3,4}` missing). Prevention: every emitted class appears as a complete string literal (exhaustive `match` arms) or in `@source inline()`. Review gate: `grep -rn 'format!(".*-{}' ferro-json-ui/src/` must return zero unaccounted matches.

5. **BUILTIN_TYPES + ferro-mcp mirror lockstep** — each new builtin bumps two independently-tested count assertions (canonical in `catalog.rs:1219`, mirror in `json_ui_catalog.rs:396`). Bumping one but not the other is a known failure mode. Both bump in the same commit per addition; the History comment in the catalog is the audit trail.

6. **Token bypass in render functions** — `bg-orange-500` instead of `bg-primary` is correct with the default theme and silently broken under any consumer theme. Review blocker: no raw palette class (`red-`, `blue-`, `orange-`, `zinc-`, `gray-`, `slate-`) in any POS render function.

7. **Intent vocabulary bloat** — the sale screen doesn't feel like any of the seven intents, creating pressure to add `Intent::Register`. This breaks the `KNOWN_INTENTS` drift guard, `infer_intent`, and the v16.5 decision. Flag at planning time (done); the Collect + Register layout template variant is the correct path.

---

## Open Decisions (Requirements Must Resolve)

These three decisions are flagged explicitly because the research files reach incomplete or conflicting conclusions. Requirements must decide before implementation begins.

### Open Decision 1: CartRuntime scope (CRITICAL — blocks all phase planning)

ARCHITECTURE.md recommends **Option B** (form-state only, defer runtime) on grounds that the milestone goal is catalog-grade components, not a complete POS client runtime.

FEATURES.md identifies **Option C** (client-side cart runtime, single commit POST) as the load-bearing consolidation piece, stating: "the `CartRuntime` JS is the load-bearing dependency — all visual components are thin wrappers over data attributes; the runtime is what makes them behave as a coherent POS unit."

**Synthesis: Option C is correct.** The milestone exists to eliminate gestiscilo's ~1500 lines of RawHtml escape hatches. That RawHtml is primarily a JS cart runtime (qty synchronization, running total, badge/ring updates). Shipping `ProductGrid` + `CartPanel` without `CartRuntime` delivers two components with no synchronization behavior — gestiscilo cannot migrate from RawHtml to these components because the components don't replicate the behavior they need to drop. Option B defers the one thing that makes the component suite worth shipping. Option A (DB write per tap) is ruled out by latency (300-800ms per product tap, unacceptable at register speed).

**Recommendation: ship `runtime/cart_runtime.rs` (Option C) as a table-stakes requirement.** It is not scope expansion — it is what the FEATURES.md MVP ranks as item 3 of 5, and what PITFALLS.md identifies as the "server round-trip perceived latency" trap if deferred.

---

### Open Decision 2: CategoryNav as standalone builtin vs. integrated prop on ProductGrid

FEATURES.md: "Implement as part of `ProductGrid` (via `categories_path` + `search` props), but expose the strip's rendering as an internal sub-renderer that can be tested independently."

ARCHITECTURE.md: "`CategoryNavProps` — New (evaluate SegmentedControl reuse first)"; suggests standalone component status is the default assumption.

**Synthesis: Integrate into ProductGrid, expose as a testable internal sub-renderer. Do not create a standalone `CategoryNav` builtin. Do not reuse `SegmentedControl`.**

`SegmentedControl` is URL-based navigation semantics (triggers page loads); `CategoryStrip` inside a register is client-side card-visibility filtering. The contracts are different. Bending `SegmentedControl` into filter semantics introduces two incompatible behaviors under one component name. A standalone `CategoryNav` builtin adds catalog overhead for a component that is never used outside `ProductGrid`. The integration approach (rendered as part of `ProductGrid` via `categories_path` prop) is correct: simpler spec authoring, no BUILTIN lockstep cost for a component that is not independently useful.

---

### Open Decision 3: Grid asymmetric mobile row weighting — v16.6 or deferred

ARCHITECTURE.md flags this as an open question: "Grid `fill` gives equal-height rows; asymmetric panes need `grid-template-rows` fractions, i.e. a `row_spans`/`row_weights` prop on `GridProps`."

FEATURES.md lists "Mobile row weighting" as a should-have differentiator with MEDIUM complexity.

253-FRICTION.md names equal-height rows as a concrete gap: on phones, the product pane should be taller than the cart (current equal-height split is wrong for register use).

**Synthesis: include in v16.6 as a `row_weights` prop on `GridProps`.** It is the same additive pattern as the `spans` prop shipped in Phase 253 (`col-span-{N}` -> now `row-span-{N}`). The gestiscilo register cannot be a first-class mobile experience without it. It is low-risk (pure CSS, additive prop, no new builtin) and scoped to a GridProps extension.

---

## Implications for Roadmap

The 6-wave ARCHITECTURE.md build sequence maps to 5 ferro phases (projection builder + MCP/docs/publish merge as one phase). Each phase has a concrete acceptance gate from PITFALLS.md.

### Phase 1: Props Contracts + Design Rules + Scope Lock

**Rationale:** Props definitions are the contract everything else implements against. Getting them right — especially with Open Decisions 1-3 resolved — before any render code prevents thrash. Design rules can be authored before their target components exist (they operate on type names in the spec element map).
**Delivers:** All new `*Props` structs (`ProductGridProps`, `CartPanelProps`, `NumpadProps`, `ProductTileProps` extensions); three new design-lint rules (`pos-fill-viewport`, `pos-cart-present`, `pos-grid-fill`) with dual fixtures per rule; `RULE_COMPONENTS` entries for new rules; `row_weights: Option<Vec<u8>>` prop on `GridProps` (the Open Decision 3 resolution); REQUIREMENTS.md scope boundary (all POS components are builtins, no `register_component`, no payment/receipt/shift scope).
**Addresses:** Intent vocabulary bloat pitfall (no new intent declared); plugin registry confusion pitfall (scope boundary stated); lint data-bound misfire pitfall (dual fixtures from the start).
**Research flag:** One targeted read of `SegmentedControl` render + runtime to confirm URL-navigation semantics before Props are finalized (30-minute verification, not a full research sprint).

---

### Phase 2: CartRuntime JS + ProductTile Extensions

**Rationale:** The runtime is the behavioral kernel; it is the hardest piece to get right and is the dependency that makes any visual component coherent. Build and test it against the existing `ProductTile` before new renderers exist, so the runtime contract is stable when renderers target it.
**Delivers:** `runtime/cart_runtime.rs` (`setupCartRuntime()`) — qty synchronization, integer-cents arithmetic, running total, qty-badge, picked-ring, remove-on-zero, empty state; `ProductTileProps` extended with `categories`, `image_url`, `color`, `stock_badge`; `runtime/numpad.rs` (`setupNumpad()`); `runtime/mod.rs` updated with new module entries and dispatcher calls; `input.css @source inline()` additions for new POS classes.
**Uses:** Existing `data-qty-input`, `data-qty-display`, `data-qty-inc`, `data-qty-dec` attribute pattern from `product_tiles.rs`; integer-cents `data-unit-price` attribute on cart rows.
**Avoids:** Round-trip latency pitfall (no server round-trip per cart tap); double-submit pitfall (`:active` classes in runtime; idempotency-key groundwork); CSS safelist drift pitfall (full-literal class strings; no `format!("util-{}", n)`).
**Acceptance gate:** `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` tests pass for all new module names.

---

### Phase 3: New Component Renderers + BUILTIN Lockstep

**Rationale:** Render functions come after runtime because the runtime's data-attribute contract drives the HTML the renderers must emit. Doing renderers before runtime risks mismatched attribute names. The BUILTIN lockstep (count bumps, dispatch arms, catalog specs) is a single atomic operation per component.
**Delivers:** `render_product_grid` (containers.rs) with integrated CategoryStrip sub-renderer and search input; `render_cart_panel` (containers.rs) with scrollable line items and qty controls; `render_numpad` (atoms.rs or `render/pos.rs`); Grid `row_weights` render path (emitting `grid-template-rows` fractional CSS); BUILTIN_TYPES + dispatch arms + BUILTIN_SPECS + imports for all new builtins; drift-guard count bumps (both sites, same commit per component); `gen-ferro-base-css.sh` regen.
**Addresses:** CSS safelist drift pitfall (exhaustive match arms verified by grep gate); sub-44px target pitfall (`min-h-[44px]` baked into every render function); token-bypass pitfall (semantic classes only, no raw palette, review gate); BUILTIN lockstep pitfall (both guards pass per addition, checked incrementally).
**Acceptance gate:** `builtin_specs_names_match_dispatch` passes; `variant_classes_use_semantic_tokens` passes; grep for unaccounted `format!(".*-{}` returns zero; HTML assertions for `min-h-[44px]` on every interactive element; both drift-guard count asserts pass.

---

### Phase 4: Projection Builder — Register Layout Template

**Rationale:** The projection builder extension depends on all catalog work being complete and validated: `emit_register_root()` emits specs referencing `ProductGrid` and `CartPanel`, which must be registered builtins with valid catalog entries before the projector can emit specs the catalog will accept.
**Delivers:** `layout: "Register"` arm in `builder.rs::build_display_spec()`; `emit_register_root(service) -> ElementBuilder` emitting fill-viewport Grid `spans: [1, 2]` with cart_pane (CartPanel + StatCard + confirm Button) and products_pane (ProductGrid with `$each` over the Browse-intent entity); `Spec::builder().fill_viewport(bool)` method; `IntentSlotTemplate` Collect -> Register in `intent_layout.rs`; updated `/cassa` sample app demonstrating the projection-derived spec (no RawHtml); `ElementBuilder.each(path, as_)` builder API.
**Addresses:** Intent bloat pitfall (Collect + Register template, seven-intent vocabulary unchanged); whole-page scroll displacement pitfall (`fill_viewport: true` emitted by projector for Register layout).
**Research flag:** Verify `$each`-scoped `$data` path handling (`strip_expr_objects`) in `catalog_validate` before implementing the builder. One test-run resolves this.

---

### Phase 5: MCP Surface, Docs, Publish

**Rationale:** Documentation, MCP generation-context, and the crates.io publish are downstream of all implementation. Single publish at the end follows the friction-loop release cadence convention.
**Delivers:** ferro-mcp `generation_context` POS composition patterns (when to use Register vs. form-only Collect; CartRuntime data-attribute contract; fill_viewport dependency); ferro-mcp count + expected-names array updated; `docs/src/json-ui/components.md` updated for all new components; crates.io publish.
**Avoids:** RawHtml blindspot pitfall (after publish, gestiscilo adoption phase runs `design:lint --deny` over cassa specs; remaining RawHtml must be explicitly `allow`-justified).
**Note:** gestiscilo cassa adoption is a separate phase in the gestiscilo repo, not a ferro phase. The consumer-repo phase follows the cross-repo phase split convention: ferro phase = framework ships; gestiscilo phase = migration code.

---

### Phase Ordering Rationale

- Phase 1 before everything: Open Decisions 1-3 must be resolved before Props are written; Props are the contract all later phases implement against. Writing renderers before resolving the CartRuntime scope decision risks rebuilding them if the scope changes.
- Phase 2 before Phase 3: The runtime data-attribute contract (`data-product-categories`, `data-cart-line`, `data-unit-price`, etc.) drives the exact HTML the renderers emit. Mismatched attribute names between runtime and renderer are a runtime-only bug invisible to render-output tests.
- Phase 3 before Phase 4: `emit_register_root()` emits specs referencing `ProductGrid` and `CartPanel` by type name; those names must be in `BUILTIN_TYPES` before `catalog_validate` will accept projector output.
- Phase 4 before Phase 5: MCP `generation_context` POS patterns are authored after the projection derivation path exists and is demonstrated in the sample app.
- Single publish at Phase 5: mid-loop publishes freeze the API before Phase 4 can revise the projection builder surface.

---

### Research Flags

Phases needing targeted verification before implementation begins:
- **Phase 1:** Read `SegmentedControl` render function and JS runtime to confirm URL-navigation semantics (confirms Open Decision 2; one file read).
- **Phase 4:** Run `catalog_validate` against a `$each`-scoped spec to confirm `strip_expr_objects` handles `$data.*` bindings under `$each` correctly before building the projector (one test run).

Phases with well-established patterns (no per-phase research sprint needed):
- **Phase 2:** `product_tiles.rs` is the exact pattern for runtime JS modules; barcode scanner heuristic is fully specified in STACK.md; no novel patterns.
- **Phase 3:** BUILTIN lockstep is a documented 11-file checklist; the Phase 251-253 review findings are encoded as acceptance gates; no novel patterns.
- **Phase 5:** ferro-mcp update pattern is mechanical (count + names array); docs pattern is established.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | CSS properties verified via MDN/caniuse; JS patterns verified against existing runtime source; WCAG numbers from W3C primary docs; zero new external dependencies |
| Features | HIGH (table stakes), MEDIUM (differentiators) | gestiscilo `build_product_picker_html()` (~1100 lines) is first-hand battle-tested evidence for table stakes; Numpad/barcode patterns synthesized from library source + research, not from production ferro code |
| Architecture | HIGH | All findings from direct code reads: exact line numbers in component.rs, atoms.rs, catalog.rs, render/mod.rs, runtime/mod.rs; no inference |
| Pitfalls | HIGH | Grounded in live incidents (Phase 253 WR-01 safelist gap, Phase 252 lint misfire, 253-FRICTION.md gap audit) and in-codebase evidence; not generic web research |

**Overall confidence:** HIGH

### Gaps to Address

- **CartRuntime scope (Open Decision 1):** ARCHITECTURE.md and FEATURES.md reach opposite conclusions. Requirements must decide before Phase 1. This summary recommends Option C (ship CartRuntime). If requirements disagrees, Phase 2 scope changes significantly.
- **CategoryNav standalone vs. integrated (Open Decision 2):** One `SegmentedControl` source read resolves this before Phase 1 Props are finalized. This summary recommends integrated (no standalone builtin).
- **Grid row weighting scope (Open Decision 3):** MEDIUM complexity. If deferred, mobile register experience remains broken at the reference consumer. This summary recommends including in v16.6.
- **`fill_viewport` projector emission:** No existing projector-emitted spec sets `fill_viewport: true`. `Spec::builder().fill_viewport(bool)` must be added in Phase 4. Verify it propagates through catalog validation before the Phase 4 plan is written.
- **Idempotency key for cart-mutation forms:** `framework::write` kernel has an idempotency hook (Phases 231/232); verify the hook is accessible from a server-rendered (non-CRUD-derived) form before the adoption phase depends on it.

---

## Sources

### Primary (HIGH confidence)
- `gestiscilo/app/src/controllers/helpers.rs:423` — `build_product_picker_html()` ~1100 lines; canonical requirements spec
- `ferro-json-ui/src/component.rs:1340` — `ProductTileProps` current definition
- `ferro-json-ui/src/render/atoms.rs:1357` — `render_product_tile` with touch-action, min-h, data-attribute pattern
- `ferro-json-ui/src/runtime/product_tiles.rs` — qty inc/dec/display/input runtime pattern
- `ferro-json-ui/src/runtime/mod.rs` — IIFE assembly, LazyLock, dispatcher
- `ferro-json-ui/src/catalog.rs:1219` — drift-guard canonical count (currently 47)
- `ferro-json-ui/assets/input.css` — `@source inline()` safelist, `fill_viewport` CSS chain
- `ferro-mcp/src/tools/json_ui_catalog.rs:396` — mirror count assertion
- `ferro-json-ui/src/design/rules.rs:6` — lint rule pattern, `RULE_REGISTRY`
- `ferro-json-ui/src/projection/builder.rs:251` — `build_display_spec()` dispatch
- `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — gestiscilo cassa audit, Gap 1 (scroll displacement), Gap 2 (mobile row weighting)
- `.planning/phases/253-mcp-surface-docs-publish/253-REVIEW.md` — WR-01 col-span safelist gap
- `.planning/phases/251-component-variant-discipline-interactive-state-pass/251-PATTERNS.md` — full-literal match-arm rule, semantic-token enforcement
- `.planning/phases/252-design-module-lint-cli/252-PATTERNS.md` — lint rule data-binding false-positive class, dual fixture requirement
- MDN `touch-action`, caniuse `manipulation` — iOS Safari support table (HIGH)
- W3C WCAG 2.5.5 / 2.5.8 Understanding docs — 44px AAA, 24px AA, five exceptions (HIGH)

### Secondary (MEDIUM confidence)
- Square POS item grid docs, Shopify POS design principles, Loyverse home sale screen layouts, Odoo grid/list switcher — category-tab convention, tap-to-add grid, image-emphasis on tablets
- GitHub axenox/onscan.js — barcode scanner timing heuristics (used as reference, not as a dependency)
- MDN `-webkit-tap-highlight-color` — non-standard, universally supported in WebKit/Chrome Android
- defensivecss.dev — iOS input font-size 16px zoom prevention (matches Apple behavior)

### Tertiary (LOW confidence)
- hashmato.com POS design principles — three-tap rule, touch ergonomics; consistent with primary sources but not authoritative
- Shopify POS UI Extensions Tile component — smart grid tile patterns; informational

---
*Research completed: 2026-07-04*
*Ready for roadmap: yes*
