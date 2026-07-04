# Architecture: POS Sale-Screen Components in ferro-json-ui (v16.6)

**Domain:** Touch-first POS builtin component pipeline integration
**Researched:** 2026-07-04
**Overall confidence:** HIGH — all findings drawn from code reads

---

## 1. Lockstep checklist for adding a builtin component

The drift guard enforces a hard count match between `BUILTIN_TYPES` (render dispatch) and `BUILTIN_SPECS` (catalog). Currently 47. Every component addition requires all of the following, in this order:

**File 1: `ferro-json-ui/src/component.rs`**
Add `*Props` struct with `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]`. Use the shared `Variant`, `Tone`, `Size` enums from this same file — the `variant_tone_size_enum_sets_drift_guard` test walks every props field named `variant`/`tone`/`size` and asserts the canonical set. Any new per-component copy of those enums breaks that test.

**File 2: `ferro-json-ui/src/render/{atoms,containers,form,data}.rs`**
Add `pub(crate) fn render_<name>(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String`. Leaf components with no children go in `atoms.rs`. Layout containers with child IDs go in `containers.rs`.

**File 3: `ferro-json-ui/src/render/mod.rs` — two changes**
(a) Add the type name string to `BUILTIN_TYPES` const (line 44). Order must match `BUILTIN_SPECS` in catalog.rs; the `builtin_specs_names_match_dispatch` test asserts set equality.
(b) Add a match arm to the `render_element()` dispatch block (line 177+).

**File 4: `ferro-json-ui/src/catalog.rs` — two changes**
(a) Add `*Props` to the `use crate::component::{...}` import list (line 29+).
(b) Add an entry to `BUILTIN_SPECS` static array (line 124): `("TypeName", "One-sentence imperative description.", || to_value(schema_for!(TypeNameProps)).unwrap(), &[/* slot fields */])`. Order must match `BUILTIN_TYPES`. Leaf components use `&[]` slots.

**File 5: `ferro-json-ui/src/catalog.rs` — drift guard test**
Line 1219: `assert_eq!(crate::render::BUILTIN_TYPES.len(), 47)` → bump to new count. Append to the History comment (`// → 48 (ComponentName)`).

**File 6: `ferro-mcp/src/tools/json_ui_catalog.rs` — two changes**
(a) Test `test_all_components_present` (line 396): `assert_eq!(catalog.components.len(), 47, ...)` → bump count and update the error message string.
(b) Add the component name to the `expected` array (line 403+).
Note at line 391-393: the canonical source-of-truth tripwire is `ferro_json_ui::catalog::tests::builtin_types_count_drift_guard`; this is a documented cross-crate mirror.

**File 7: `scripts/gen-ferro-base-css.sh`**
Run to regenerate `ferro-json-ui/assets/ferro-base.css` after any new Tailwind classes are used.

**Files 8-9: Runtime JS (only if component needs client-side behavior)**
Add `ferro-json-ui/src/runtime/<name>.rs` with `pub(super) const SOURCE: &str` containing a `function setup<Name>() { ... }`. Then in `ferro-json-ui/src/runtime/mod.rs`: (a) `mod <name>;`, (b) `s.push_str(<name>::SOURCE);` in the `FERRO_RUNTIME_JS` LazyLock (line 27), (c) `setup<Name>();` call in the `ferroRuntime()` dispatcher string (line 43), (d) add the function name to both `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` tests. The runtime is a single IIFE bundled into every response — setup functions must be no-ops when their elements are absent.

**File 10: `ferro-mcp/src/tools/json_ui_catalog.rs` — `RULE_COMPONENTS` (line 81, if design rules reference the component)**
Add `("rule-id", &["ComponentName"])`. The `component_rule_mapping_is_exhaustive` test asserts every registry rule id is mapped and every component name exists as a builtin.

**File 11: `docs/src/json-ui/components.md`**
Required per CLAUDE.md: "Always update docs when framework changes."

---

## 2. ProductTile and /cassa demo: current state and gaps

**What exists (confirmed from code):**

- `ProductTileProps` in `component.rs` lines 1340-1352: fields `product_id: String`, `name: String`, `price: String`, `field: String`, `default_quantity: Option<u32>`. No category, no image.
- `render_product_tile` in `atoms.rs` lines 1357-1390: bordered card with `touch-manipulation` CSS, `min-h-[44px] min-w-[44px]` dec/inc buttons using `data-qty-dec`/`data-qty-inc` attributes, a `data-qty-display` span, and a hidden `<input data-qty-input>`.
- `runtime/product_tiles.rs`: JS `setupProductTiles()` wiring the qty buttons; dispatches an `input` event on change for form-guard integration. Always bundled.
- `app/src/views/cassa.json`: working spec using `fill_viewport: true`, root Grid `fill: true, spans: [1, 2]`, `md_columns: 3`, a cart_pane (Card → DataTable + StatCard + Button) and a products_pane (Grid with `$each` over `/prodotti` emitting ProductTile). Three routes: `GET /cassa`, `POST /cassa/conferma`, `POST /cassa/rimuovi/:id`.
- `app/src/controllers/cassa.rs`: stateless handler with hardcoded `prodotti` + `carrello` data; POST handlers are redirect stubs.

**Gaps to catalog-grade quality (from code + 253-FRICTION.md):**

- No `CategoryNav` component; `ProductTileProps` has no `category_id`.
- No client-side cart runtime — the cart DataTable is server-rendered static; tile taps don't reactively update the cart panel.
- No `Numpad` component; no standalone `QuantityStepper` outside ProductTile.
- Grid `fill` mode uses equal-height rows — a register wants the product pane taller than the cart on small screens (asymmetric mobile weighting).
- DataTable has no `density: "compact"` option for a cart display.
- No search/filter integration in the product grid.
- `cassa.json` declares `design: { intent: "collect", allow: ["page-header", "breadcrumb-on-subpages"] }` — the lint rules don't yet have POS-native exemptions.

---

## 3. Cart state: where does the in-progress order live

Two dimensions: persistence scope (session vs. DB) and interaction model (per-tap round-trip vs. client-accumulate-then-commit).

**Option A: DB draft order per tap**
Each product tap POSTs a line-item create through the existing `framework::write::dispatch_write` kernel and the `line_items` table (`app/src/models/entities/line_items.rs` exists with `order_id`, `amount`, `tenant_id`, `deleted_at`; it lacks `product_id` and `quantity` columns). Page re-renders from fresh DB state on redirect-after-POST. Durable, audit-trailed, CRUD-derivable.
Tradeoff: one HTTP round-trip per product tap — unacceptable latency at register speed (5 taps in 3 seconds).

**Option B: Client-side form state only (current demo pattern)**
ProductTile hidden inputs accumulate quantities; a single "Confirm" form POST submits all `qty_{id}` values. No JS cart panel. Works today.
Tradeoff: cart panel is static (no running total on tap); lost on refresh; payload carries every product's qty field.

**Option C: Client-side cart runtime, single commit POST**
A new `runtime/cart_runtime.rs` (`setupCartRuntime()`) maintains an in-memory cart array driving a dynamically-rendered cart summary + running total. "Confirm" serializes once. This is what gestiscilo's `build_product_picker_html` (~1100 lines) implements today as app-level RawHtml.
Tradeoff: more JS in the runtime bundle; CartPanel needs a `data-cart-target` DOM hook.

**Recommendation: Option B for the initial component suite, with a documented path to Option C.** The milestone goal is catalog-grade components that eliminate the RawHtml escape hatch, not a complete POS client runtime. Note: FEATURES.md research reaches the opposite conclusion (the cart runtime IS the load-bearing consolidation gestiscilo needs) — this is the central scope decision for requirements.

**If Option A:** `line_items` needs `product_id`, `quantity` columns; draft order gets `status = "draft"` in the orders StateMachine; line-items CRUD is derivable from the projection layer.

---

## 4. Projection → register: intent mapping

**The existing declaration is already correct.** `cassa.json` declares `intent: "collect"`. The v16.5 decision is explicit: archetypes ARE the intents — no new intent.

**Option A: Collect with a "Register" layout template variant (recommended)**
`builder.rs::build_display_spec()` (line 251) dispatches on `template.layout` with arms for `"DataTable"`, `"Card"`, `"Form"`, `"KanbanBoard"`, `"StatCard"`. Add a `"Register"` arm calling a new `emit_register_root(service: &ServiceDef) -> ElementBuilder` that emits a fill-viewport Grid `spans: [1, 2]` with a cart_pane (Card → DataTable `/cart` + StatCard total + confirm Button) and a products_pane (Grid → ProductTile with `$each` over `/products`, bound from `EntityName` + `Money` field meanings). The `IntentSlotTemplate` for Collect gains a `layout: "Register"` option in `ferro-json-ui/src/projection/intent_layout.rs`; theme overrides via `ctx.templates` continue to work.

Requires one new builder capability: `ElementBuilder.each(path, as_)` — `Element.each: Option<EachDirective>` already exists in `spec.rs`; this is a localized builder API addition.

**Option B: Browse + Collect split-pane composition via a new archetype — do not pursue.** Directly contradicts the v16.5 constraint.

**What does NOT change:** the seven-intent vocabulary, `Intent` enum, `derive_intents()`, `IntentScore` machinery.

**Open question for the builder phase:** whether `fill_viewport: true` is emitted by the projector (when `layout == "Register"`) or declared by the spec author. Today no projector-emitted spec sets it; making the projector set it requires `Spec::builder().fill_viewport(bool)`.

---

## 5. Design-lint rules for POS components

Rule pattern from `ferro-json-ui/src/design/rules.rs` (RULE_REGISTRY at line 6): `DesignRule { id, title, rationale, intents: &[&str], check: fn(&Spec, Option<&str>) -> Vec<Finding> }`; tests inline; `RULE_COMPONENTS` in ferro-mcp maps rule ids ↔ component names bidirectionally.

**Rule 1: `pos-fill-viewport` (Warning, intents: `&["collect"]`)** — spec has `ProductTile` but `fill_viewport == false` with a dashboard-family layout. Suggestion: set `fill_viewport: true` + Grid `fill: true`. Components: ProductTile, Grid.

**Rule 2: `pos-cart-present` (Info, intents: `&["collect"]`)** — `ProductTile` present with no cart display (DataTable + StatCard). Info because a products-only page is a valid intermediate state.

**Rule 3: `pos-grid-fill` (Warning, intents: `&["collect"]`)** — `fill_viewport` true but the product Grid has no `fill` mode — scroll behavior incoherent.

**NOT a lint rule:** touch-target enforcement. `min-h-[44px]` is baked into render functions by construction — the spec layer cannot express button size; it is a render-time guarantee.

**`allow` implication:** POS specs legitimately suppress `page-header`/`breadcrumb-on-subpages`; the new POS rules should NOT be allow-listed by default.

---

## 6. Suggested build order

Dependencies: Props structs → render functions → BUILTIN_TYPES/BUILTIN_SPECS → drift guard → ferro-base.css regen. Runtime JS is a parallel track. Design rules are partially independent (operate on the spec element map, can be written against type names early). Projection builder depends on all component work (catalog must validate before the projector emits specs for it).

**Wave 1 (parallelizable):** new `*Props` structs (`CategoryNavProps`, `NumpadProps`, `ProductTileProps` extensions); design-lint rules + RULE_COMPONENTS.
**Wave 2:** render functions (`render_category_nav`, `render_numpad` in atoms.rs or a new `render/pos.rs`; CartPanel in containers.rs if distinct); runtime JS (`runtime/numpad.rs`, optional `runtime/cart_runtime.rs`).
**Wave 3 (sequential):** BUILTIN_TYPES + dispatch arms; BUILTIN_SPECS + imports; drift-guard count bump; runtime/mod.rs updates; `gen-ferro-base-css.sh` regen.
**Wave 4:** ferro-mcp count + expected names; full CI-exact gate.
**Wave 5 (overlaps 4):** projection builder extension (`emit_register_root` + `.each()` + intent_layout Register template); `/cassa` sample app demonstrating the projection-derived spec.
**Wave 6:** ferro-mcp `generation_context` POS composition patterns; docs; single crates.io publish.

---

## Component boundaries (new vs. modified)

| Component | Status | File(s) |
|-----------|--------|---------|
| `ProductTileProps` | Modified (add `category_id`, cart-runtime data attrs) | `component.rs`, `render/atoms.rs` |
| `CategoryNavProps` | New (evaluate SegmentedControl reuse first) | `component.rs`, `atoms.rs`, `catalog.rs`, `render/mod.rs` |
| `NumpadProps` | New | same lockstep |
| `CartPanelProps` | Decision: possibly skip as distinct component — Card+DataTable+StatCard composition may suffice; FEATURES.md disagrees (CartPanel + cart runtime is the consolidation target). Requirements must decide. | — |
| `runtime/numpad.rs` | New | `runtime/` |
| `runtime/cart_runtime.rs` | Scope decision (Option C) | `runtime/` |
| Design rules (3 new) | New | `design/rules.rs` |
| `builder.rs` | Modified (Register arm + `.each()`) | `projection/builder.rs` |
| `intent_layout.rs` | Modified (Collect → Register template) | `projection/intent_layout.rs` |

---

## Key integration points (exact paths)

- Drift guard: `ferro-json-ui/src/catalog.rs:1219` (canonical) + `ferro-mcp/src/tools/json_ui_catalog.rs:396` (mirror).
- Dispatch: `ferro-json-ui/src/render/mod.rs:177`; `BUILTIN_TYPES` at line 44.
- Catalog specs: `ferro-json-ui/src/catalog.rs:124`.
- Runtime bundle: `ferro-json-ui/src/runtime/mod.rs:27` (LazyLock) + line 43 (dispatcher).
- ProductTile render: `ferro-json-ui/src/render/atoms.rs:1357`.
- Design rules: `ferro-json-ui/src/design/rules.rs:6`; RULE_COMPONENTS: `ferro-mcp/src/tools/json_ui_catalog.rs:81`.
- Projection builder: `ferro-json-ui/src/projection/builder.rs:251`.
- Sample cassa: `app/src/views/cassa.json`, `app/src/controllers/cassa.rs`.

---

## Open questions flagged for phase-specific research

1. **`$each` in projector output** — `ElementBuilder` lacks `.each()`; verify catalog-validate-after-emit handles `$each`-scoped `$data` paths (`strip_expr_objects`).
2. **CategoryNav data source** — static props vs `data_path` vs reusing `SegmentedControl` (which already supports `items`/`data_path` + active segment). Evaluate before creating `CategoryNavProps`.
3. **Mobile row weighting** — Grid `fill` gives equal-height rows; asymmetric panes need `grid-template-rows` fractions, i.e. a `row_spans`/`row_weights` prop on `GridProps` (analogous to Phase 253 `spans`). Scope decision: v16.6 or deferred.

---

*Architecture research for: v16.6 POS Component Suite*
*Researched: 2026-07-04*
