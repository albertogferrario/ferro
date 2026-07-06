# Phase 254: Props Contracts + Touch Foundation + Design Rules — Research

**Researched:** 2026-07-05
**Domain:** ferro-json-ui component API extension, render/classes.rs constants, design lint rules
**Confidence:** HIGH — all findings drawn directly from codebase reads; no external lookups required for this codebase-internal phase

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**ProductTile additive props (POS-02)**
- D-01: Field is `categories: Vec<String>` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`. Rendered as space-separated `data-product-categories` attribute, emitted only when non-empty.
- D-02: `image_url: Option<String>`, `color: Option<String>`, `stock_badge: Option<String>` — all `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- D-03: Phase 254 renderer touch is criteria-exact — `render_product_tile` gains `data-product-categories` only; visual rendering of image_url/color/stock_badge is Phase 256.
- D-04: Serde backward-compat test: legacy ProductTile JSON without new fields round-trips unchanged.

**Shared POS touch foundation (POS-07)**
- D-05: Five named constants in `render/classes.rs`: `POS_TOUCH_ACTION`, `POS_HIT_TARGET_MIN`, `POS_PRESS_ACTIVE`, `POS_OVERSCROLL_CONTAIN`, `POS_TAP_HIGHLIGHT`. Every constant is a complete class literal.
- D-06: `render_product_tile` migrates inline `touch-manipulation` and `min-h-[44px] min-w-[44px]` literals to constants this phase.
- D-07: Composition drift-guard test must auto-cover Phase 256 render functions (no manual re-enrollment per component).
- D-08: Run `scripts/gen-ferro-base-css.sh` ONCE at phase end. No `@source inline()` safelist additions expected.

**POS design-lint rules (POS-11)**
- D-09: Four rules join RULE_REGISTRY (11 → 15): `pos-fill-viewport`, `pos-grid-fill`, `pos-cart-present`, `fill-viewport-layout-unknown`. All four are `Severity::Warning`.
- D-10: Rule semantics locked (predicates refined in planning). Research directive: determine exact supported-layout set from CSS chain + layout registry.
- D-11: All four rules use `intents: &[]` (all-intents) with internal presence gates.
- D-12: Three fixtures per rule: violating, conforming, data-bound (no misfire).
- D-13: Matching POS type names against unregistered components is correct — lint operates on raw spec, never consults BUILTIN_TYPES.

**RULE_COMPONENTS + ferro-mcp guard**
- D-14: All four rules mapped to closest existing builtin. Named 256 handoff for extension.
- D-15: Component count stays 47. BUILTIN_TYPES, dispatch, BUILTIN_SPECS untouched.

**Five new Props structs**
- D-16: Declared in `ferro-json-ui/src/component.rs` with full derive set; one schema smoke test each; NOT registered.
- D-17: Behavioral contract anchors locked; field-level naming/types are planning work.
- D-18: No CartRuntime hooks in any Props contract.
- D-19: Grid `row_weights: Vec<u8>`, `#[serde(default, skip_serializing_if = "Vec::is_empty")]`; schema + round-trip test only.

**Gate**
- D-20: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, plus `cargo doc` clean.

### Claude's Discretion

- Exact class strings inside `POS_PRESS_ACTIVE` / `POS_TAP_HIGHLIGHT`.
- Drift-guard test mechanism (source-scan vs composition equality) within D-07 auto-coverage constraint.
- Lint predicate details: exact trigger sets, `element_id` attribution, `suggestion` text.
- Whether `POS_HIT_TARGET_NUMPAD` (56px) constant ships now or in Phase 256.
- Field-level naming and types inside D-17 behavioral contracts.
- Whether `infer.rs` gains a ProductGrid → collect inference branch.

### Deferred Ideas (OUT OF SCOPE)

- ProductTile visual rendering of `image_url`/`color`/`stock_badge` — Phase 256.
- RULE_COMPONENTS association extension to new component names — Phase 256.
- Runtime modules (`setupNumpad`, `setupPosFilter`) — Phase 255.
- CartRuntime, barcode keyboard-wedge, layout-name-independent ferro-fill chain — REQUIREMENTS.md Future Requirements.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| POS-02 | `ProductTile` gains additive props (`categories`, `image_url`, `color`, `stock_badge`) with backward-compat | Verified serde conventions in `component.rs:1345`; render diff in `atoms.rs:1357`; backward-compat test pattern in `schema_smoke_tests` module |
| POS-07 | Shared POS touch constants in `render/classes.rs`; drift-guard test | Classes.rs structure verified (59 lines); drift mechanism resolved; exact class values verified |
| POS-11 | Four POS design-lint rules + RULE_COMPONENTS mapping | `design/rules.rs` pattern fully mapped; RULE_REGISTRY structure verified; D-09 patterns.md hidden dependency found |

</phase_requirements>

---

## Summary

Phase 254 is a pure API-contract phase: no new render functions, no runtime JS, no builtin registration changes. Everything produced here is consumed by Phases 255 and 256 without renegotiation.

The research resolves six open verification items from the CONTEXT.md. Most significantly:
1. The `ferro-fill` CSS chain supports exactly two layout names — `"app"` (builtin) and `"dashboard"` (user-registered convention). This is the set the `fill-viewport-layout-unknown` lint rule must encode.
2. A hidden CI dependency exists: adding four rules to `RULE_REGISTRY` triggers `patterns_md_matches_rule_registry` test failure unless four new sections are added to `docs/src/design-system/patterns.md`. This is not in the success criteria but will fail `cargo test`.
3. The D-07 auto-covering drift-guard mechanism is the `CARGO_MANIFEST_DIR` + `std::fs::read_dir` source-scan pattern already used in `design/mod.rs:326`.
4. `POS_TAP_HIGHLIGHT` has two viable implementation paths — Tailwind v4 arbitrary property (needs live verification) or `@utility` definition in `input.css` (guaranteed).
5. `ProductTileProps` field `categories` is plural `Vec<String>`, matching the CONTEXT decision — confirmed the existing struct has only 5 fields, all required or Option.

**Primary recommendation:** Execute in three parallelizable work streams within a single wave: (A) component.rs additions (ProductTileProps extension + 5 new Props structs + GridProps row_weights), (B) classes.rs constants + render_product_tile migration, (C) design/rules.rs additions + patterns.md + RULE_COMPONENTS. Complete with ferro-base.css regen.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ProductTile additive props (serde) | ferro-json-ui component.rs | — | Props structs live in component.rs; serde is compile-time |
| ProductTile data-attr emission | ferro-json-ui render/atoms.rs | render/classes.rs (constants) | Render function owns HTML output; constants own class strings |
| POS touch constants | ferro-json-ui render/classes.rs | — | Single source of truth module for all shared class fragments |
| Drift-guard test (POS constants) | ferro-json-ui render/classes.rs (tests) | — | Tests live adjacent to the constants they guard |
| POS design-lint rules | ferro-json-ui design/rules.rs | design/mod.rs (dispatch) | RULE_REGISTRY is static in rules.rs; lint() in mod.rs routes to it |
| RULE_COMPONENTS mapping | ferro-mcp json_ui_catalog.rs | — | Cross-crate mapping lives in mcp tool that derives agent guidance |
| patterns.md documentation | docs/src/design-system/patterns.md | — | D-09 drift guard enforces bidirectional sync with RULE_REGISTRY |

---

## Standard Stack

### Core (all already in workspace — no new deps)
| Crate/Module | Purpose | Notes |
|---|---|---|
| `ferro-json-ui/src/component.rs` | Props struct declarations | All new `*Props` go here alongside existing Props |
| `ferro-json-ui/src/render/classes.rs` | Shared class fragment constants | Add 5 POS constants following existing INTERACTIVE_BASE pattern |
| `ferro-json-ui/src/render/atoms.rs` | Leaf component renderers | `render_product_tile` migrates literals to constants, gains data-attr |
| `ferro-json-ui/src/design/rules.rs` | RULE_REGISTRY + check functions | 11 rules → 15; existing `check_*` pattern reused |
| `ferro-json-ui/src/design/mod.rs` | Lint engine dispatch | No change to dispatch logic; RULE_REGISTRY auto-loaded |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | RULE_COMPONENTS static + 3-direction guard | 11 entries → 15; Direction 2 forces new entries |
| `docs/src/design-system/patterns.md` | Design rule documentation | **MANDATORY**: 4 new sections required by D-09 drift guard |
| `scripts/gen-ferro-base-css.sh` | Tailwind CSS regeneration | Run once at phase end; new constants are scanner-visible |

---

## Architecture Patterns

### System Architecture Diagram

```
Phase 254 data flow:

ProductTileProps (component.rs)
  ├── [D-01] categories: Vec<String> → render_product_tile
  │     └── data-product-categories attr (non-empty only)
  ├── [D-02] image_url/color/stock_badge → serde only, no render in 254
  └── [serde backward-compat] legacy JSON with absent fields → round-trip unchanged

GridProps (component.rs)
  └── [D-19] row_weights: Vec<u8> → serde + schema_smoke_test only; no render in 254

Five new Props structs (component.rs) → declared, schema_smoke_test, NOT registered
  ProductGridProps / CartPanelProps / CategoryNavProps / QuantityStepperProps / NumpadProps

render/classes.rs (new constants)
  POS_TOUCH_ACTION / POS_HIT_TARGET_MIN / POS_PRESS_ACTIVE
  POS_OVERSCROLL_CONTAIN / POS_TAP_HIGHLIGHT
    └── render_product_tile migrates inline literals → constants (same output)
    └── drift-guard test: source-scan all render/*.rs for raw literals

RULE_REGISTRY (design/rules.rs) 11 → 15
  pos-fill-viewport / pos-grid-fill / pos-cart-present / fill-viewport-layout-unknown
    └── check functions: internal presence gates (not intent-keyed dispatch)
    └── 3 fixtures each: violating / conforming / data-bound
    ↓
  design/mod.rs lint() dispatch (intents: &[] → all specs matched)
    ↓
  RULE_COMPONENTS (ferro-mcp json_ui_catalog.rs) 11 → 15
    └── Direction 2 guard: every registry id must be mapped
    └── Direction 3 guard: every mapped component name must be a builtin
    └── Resolution: &["Grid"] for POS rules; &[] for fill-viewport-layout-unknown

  patterns.md (docs/src/design-system/patterns.md)
    └── D-09 drift guard: 4 new ## `rule-id` sections required
```

### Recommended Project Structure (unchanged — additions only)

```
ferro-json-ui/src/
├── component.rs          # Add: 5 new *Props, GridProps.row_weights, ProductTileProps.categories+
├── render/
│   ├── classes.rs        # Add: 5 POS constants + drift-guard test
│   └── atoms.rs          # Modify: render_product_tile (constant migration + data-attr)
└── design/
    └── rules.rs          # Add: 4 check functions + RULE_REGISTRY entries
ferro-mcp/src/tools/
└── json_ui_catalog.rs    # Add: 4 RULE_COMPONENTS entries
docs/src/design-system/
└── patterns.md           # Add: 4 new rule sections (D-09 required)
```

---

## Verified Findings (Phase-Specific)

### Finding 1: `fill-viewport-layout-unknown` Supported-Layout Set (resolves D-10)

**Verified from:** `ferro-json-ui/src/layout.rs` (DOM structure) + `ferro-json-ui/assets/input.css` (comment at line 124)

The `ferro-fill` CSS chain targets this DOM structure:
```
body.ferro-fill              → height:100dvh; overflow:hidden
  └── > div.flex             → height:100%; min-height:0
        └── main             → min-height:0; overflow:hidden; flex-column
              └── > div      → flex:1 1 0%; min-height:0
                    └── #ferro-json-ui → flex:1 1 0%; min-height:0
```

DOM shape produced by each builtin layout:
- **`"app"` layout** (`AppLayout`): `<nav> + <div class="flex"> → <main class="flex-1 ..."> → <div class="mx-auto w-full max-w-7xl"> → #ferro-json-ui` → CSS chain FULLY MATCHES
- **`"auth"` layout** (`AuthLayout`): `<div class="min-h-screen flex items-center justify-center"> → <div class="w-full max-w-md"> → #ferro-json-ui` → has `div.flex` as direct body child but NO `main` element → `body.ferro-fill main` selector finds nothing → NOT SUPPORTED
- **`"default"` layout** (`DefaultLayout`): `#ferro-json-ui` directly in body → no `div.flex` child → `body.ferro-fill > div.flex` selector finds nothing → NOT SUPPORTED
- **`None` (no layout)**: same as `"default"` → NOT SUPPORTED
- **`"dashboard"` layout** (user-registered): The `input.css` comment at line 124 EXPLICITLY states "dashboard/app layouts"; `is_app_shell_layout()` in `rules.rs:90` encodes `matches!(spec.layout.as_deref(), Some("dashboard") | Some("app"))` → SUPPORTED by framework convention

**Conclusion:** The `fill-viewport-layout-unknown` rule must warn when `fill_viewport: true` AND `spec.layout` is NOT `Some("app")` or `Some("dashboard")`. Use the `is_app_shell_layout` helper pattern exactly.

**Supported set: `{"app", "dashboard"}`** [VERIFIED: ferro-json-ui/src/layout.rs + assets/input.css:124]

### Finding 2: `POS_TOUCH_ACTION` and `POS_HIT_TARGET_MIN` (resolves D-06)

**Verified from:** `ferro-json-ui/src/render/atoms.rs:1357-1389`

Existing `render_product_tile` contains these inline literals:
- Line 1373: `"... touch-manipulation"` (in outer div class string)
- Lines 1380+1384: `"min-h-[44px] min-w-[44px] ..."` (in both button class strings)

Constants:
- `POS_TOUCH_ACTION = "touch-manipulation"` — substituting into the format! macro produces byte-identical output
- `POS_HIT_TARGET_MIN = "min-h-[44px] min-w-[44px]"` — same reasoning

Both are already scanner-visible in the codebase (inline literals); defining them as constants in `classes.rs` does not change the generated CSS. [VERIFIED: atoms.rs:1373,1380,1384]

### Finding 3: `POS_PRESS_ACTIVE` Recommended Value

**From STACK.md candidates:** `active:scale-95`, `active:bg-border`, `active:scale-[0.97]`, `active:brightness-95`

Recommended: `"active:scale-95 active:bg-border"`
- `active:scale-95` — built-in Tailwind scale step, full literal, scanner picks up from Rust source via `@source "../../ferro-json-ui/src"`
- `active:bg-border` — uses `--color-border` semantic token, not a raw palette class; token-compliant per CLAUDE.md; full literal
- Neither class exists in current codebase (verified grep) — they are genuinely new utilities requiring CSS generation [ASSUMED based on STACK.md analysis; VERIFY: run `gen-ferro-base-css.sh` and grep output]

### Finding 4: `POS_TAP_HIGHLIGHT` — Two Viable Implementation Paths (resolves D-05 open question)

**From STACK.md + Tailwind v4 arbitrary-property analysis:**

**Path A (arbitrary property class):** `"[-webkit-tap-highlight-color:transparent]"`
- Tailwind v4 supports `[property:value]` arbitrary property syntax
- Appears as a full string literal in `classes.rs` → scanned by `@source "../../ferro-json-ui/src"`
- RISK: STACK.md says "Tailwind does not generate this as a utility class" — this assessment is Tailwind v3 behavior; Tailwind v4 does support arbitrary properties. However, scanner behavior for arbitrary CSS properties with vendor prefixes is uncertain.
- **VERIFICATION REQUIRED:** After adding the constant, run `scripts/gen-ferro-base-css.sh` and confirm the property appears in `ferro-base.css`.

**Path B (custom @utility — recommended for certainty):**
Add to `input.css`:
```css
@utility pos-tap-highlight {
  -webkit-tap-highlight-color: transparent;
}
```
Then `POS_TAP_HIGHLIGHT = "pos-tap-highlight"` as a plain class name constant.
- Guaranteed to generate the CSS (follows the existing `@utility duration-fast` pattern at `input.css:94`)
- Full literal class name → scanner-visible
- Does NOT require `@source inline()` safelist entry
- D-05 says "full class literal" — `"pos-tap-highlight"` satisfies this

**Recommendation for planner:** Use Path B for zero uncertainty. If Path A is preferred for semantic transparency, include a Wave 0 verification step: run the CSS generator and grep for `tap-highlight` before proceeding. [ASSUMED: Path A scanner behavior; VERIFIED: Path B pattern via input.css:94]

### Finding 5: D-07 Drift-Guard Mechanism (resolves D-07)

**Verified pattern from:** `ferro-json-ui/src/design/mod.rs:325-358` (`patterns_md_matches_rule_registry` test)

The existing D-09 test uses `std::env::var("CARGO_MANIFEST_DIR")` + `std::fs::read_to_string` to read external files at test time. Apply the same pattern for the POS drift guard:

```rust
// In ferro-json-ui/src/render/classes.rs, cfg(test) block
#[test]
fn pos_render_functions_use_constants_not_literals() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let render_dir = std::path::Path::new(&manifest_dir).join("src/render");
    // Raw literals that must NOT appear outside classes.rs
    let guarded_literals = [
        "touch-manipulation",
        "min-h-[44px] min-w-[44px]",
    ];
    for entry in std::fs::read_dir(&render_dir).expect("src/render readable") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        if filename == "classes.rs" { continue; } // source, not consumer
        let source = std::fs::read_to_string(&path).unwrap();
        for literal in &guarded_literals {
            assert!(
                !source.contains(literal),
                "{filename}: raw POS literal {literal:?} — import from render::classes instead"
            );
        }
    }
}
```

**Auto-coverage:** The `read_dir` loop scans ALL `*.rs` files in `src/render/`. When Phase 256 adds new render files (e.g., `pos.rs`), the test automatically covers them without manual enrollment — satisfying D-07's auto-coverage constraint. [VERIFIED: pattern from design/mod.rs:326]

### Finding 6: CRITICAL HIDDEN DEPENDENCY — D-09 patterns.md Drift Guard

**Verified from:** `ferro-json-ui/src/design/mod.rs:319-358`

The `patterns_md_matches_rule_registry` test does a **bidirectional sync check** between `RULE_REGISTRY` and `docs/src/design-system/patterns.md`:
- Forward: every rule id in `RULE_REGISTRY` must appear as text in `patterns.md`
- Reverse: every `## \`rule-id\`` section header in `patterns.md` must exist in `RULE_REGISTRY`

**When 4 new rules are added to RULE_REGISTRY, `cargo test` FAILS unless 4 new sections are added to `docs/src/design-system/patterns.md`.** This is not in the Phase 254 success criteria but is enforced by an existing CI test under `--all-features`.

Required additions to `patterns.md` (each must have `## \`rule-id\`` format per the reverse check):
- `## \`pos-fill-viewport\``
- `## \`pos-grid-fill\``
- `## \`pos-cart-present\``
- `## \`fill-viewport-layout-unknown\``

Each section should follow the existing pattern (seen at `patterns.md:9-55`): Title, Rationale, Intents, conforming example, violating example, "How to allow" block.

**The plan MUST include a task to update patterns.md in the same wave as RULE_REGISTRY changes.** [VERIFIED: design/mod.rs:325-358]

### Finding 7: RULE_COMPONENTS Direction 3 — Empty-Slice Compatibility

**Verified from:** `ferro-mcp/src/tools/json_ui_catalog.rs:757-763`

Direction 3 guard code:
```rust
for (_, comps) in RULE_COMPONENTS {
    for &c in *comps {  // inner loop does not execute if comps is &[]
        assert!(builtins.contains(c), ...);
    }
}
```

An empty slice `&[]` passes Direction 3 unconditionally. The `fill-viewport-layout-unknown` rule has no structural component association (it concerns layout choice, not component composition), so `&[]` is semantically correct.

**RULE_COMPONENTS mapping for Phase 254:**
- `("pos-fill-viewport", &["Grid"])` — register-root concern
- `("pos-grid-fill", &["Grid"])` — directly about Grid.fill property
- `("pos-cart-present", &["Grid"])` — register composition concern; Grid is the register root
- `("fill-viewport-layout-unknown", &[])` — layout concern; no component association; passes Direction 3

Phase 256 extends these associations to `ProductGrid`, `CartPanel`, `Numpad` in the same commit that registers them in BUILTIN_TYPES (D-14 named handoff). [VERIFIED: json_ui_catalog.rs:757-763]

### Finding 8: render_product_tile Exact Diff (resolves D-03/D-06)

**Verified from:** `ferro-json-ui/src/render/atoms.rs:1357-1389`

Current function (5 fields: product_id, name, price, field, default_quantity):

| Line | Current inline literal | Phase 254 replacement |
|------|----------------------|----------------------|
| 1373 | `"...touch-manipulation"` | `format!("...{POS_TOUCH_ACTION}")` |
| 1380 | `"min-h-[44px] min-w-[44px] ...button classes..."` | `{POS_HIT_TARGET_MIN} ...` |
| 1384 | `"min-h-[44px] min-w-[44px] ...button classes..."` | `{POS_HIT_TARGET_MIN} ...` |
| (new) | — | conditional `data-product-categories="{joined}"` on the outer div when `props.categories` non-empty |

**Byte-identical assertion for legacy specs:** Since `POS_TOUCH_ACTION = "touch-manipulation"` and `POS_HIT_TARGET_MIN = "min-h-[44px] min-w-[44px]"`, and `categories` defaults to `Vec::is_empty()` → no attribute emitted, a legacy spec without the new fields produces identical HTML. The D-04 backward-compat test verifies this by round-tripping the old JSON and asserting the rendered output equals the pre-change baseline.

**`data-product-categories` emission pattern:**
```rust
let categories_attr = if props.categories.is_empty() {
    String::new()
} else {
    format!(" data-product-categories=\"{}\"", html_escape(&props.categories.join(" ")))
};
// Add {categories_attr} in the format! string on the outer div
```

[VERIFIED: atoms.rs:1357-1389]

### Finding 9: Five New Props Struct Field Evidence (resolves D-17)

From `FEATURES.md` (gestiscilo `build_product_picker_html` evidence + POS product survey):

**`ProductGridProps`** (renders the products pane; $each iteration target for Phase 257):
- `data_path: String` — JSON pointer to product array
- `form_id: String` — scope isolator linking to CartPanel
- `categories_path: Option<String>` — JSON pointer to category string array; absent → no category strip
- `columns: Option<u8>` — override base grid columns (default 2 in render)
- `search: Option<bool>` — enable search input (default true when categories present)

**`CartPanelProps`** (server-rendered cart; pins + internally scrolls under fill_viewport):
- `form_id: String` — scope isolator matching ProductGrid.form_id
- `empty_message: Option<String>` — placeholder text when no items
- `show_staff: Option<bool>` — whether Staff column visible (gestiscilo booking mode)
- `show_people: Option<bool>` — whether People stepper column visible

**`CategoryNavProps`** (standalone builtin per operator decision):
- `items: Vec<String>` — static category list (OR `data_path: Option<String>` for data-binding; planner's choice)
- `all_label: Option<String>` — label for the "show all" tab (default "Tutte")
- Filter contract: emits `data-nav-tab` attributes; Phase 255 runtime wires the client-side filter

**`QuantityStepperProps`** (reusable +/- stepper; ProductTile hidden-input contract):
- `field: String` — name of the hidden input this stepper drives
- `min: Option<u32>` — lower bound (default 0)
- `max: Option<u32>` — upper bound (optional)
- `step: Option<u32>` — increment size (default 1)

**`NumpadProps`** (tap-surface keypad; never a native input):
- `target_field: String` — name of the hidden/visible input this numpad drives
- `mode: Option<NumpadMode>` — `Quantity` (integer only) or `Price` (two decimal places); enum needs derive set

[MEDIUM confidence: FEATURES.md is first-party evidence from gestiscilo code; exact field names are planner's discretion per D-17]

### Finding 10: `GridProps.row_weights` Conventions (D-19)

**Verified from:** `ferro-json-ui/src/component.rs:882-914` (`GridProps` struct)

Existing `spans` field (the pattern to mirror):
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub spans: Vec<u8>,
```

`row_weights` mirrors exactly:
```rust
/// Per-row height weights for fill-mode grids. Positional alignment with `children` (missing
/// entries default to equal weight). A row with weight N receives N fractional units of the
/// available height — e.g. `row_weights: [2, 1]` gives the first row 2/3 and second row 1/3.
/// Meaningful only when `fill: true`; ignored in `scrollable` mode.
/// The render path (fractional `grid-template-rows` via inline style) lands in Phase 256.
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub row_weights: Vec<u8>,
```

Schema smoke test follows the existing `schema_for_grid_props_generates` test at `component.rs:1565`.

[VERIFIED: component.rs:882-914]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Source-scan drift guard | Custom build.rs or proc-macro scanning | `std::env::var("CARGO_MANIFEST_DIR")` + `std::fs::read_dir` in test | D-09 already uses this pattern; consistent, zero setup |
| CSS class generation for `-webkit-tap-highlight-color` | Custom CSS generation | `@utility pos-tap-highlight { ... }` in `input.css` | Follows existing `@utility duration-fast` pattern; guaranteed output |
| Pattern doc sync | Manual tracking | D-09 `patterns_md_matches_rule_registry` test | Already enforced bidirectionally; just write the docs |
| Lint intent dispatch | Per-rule intent filtering | `intents: &[]` with internal gate | Existing `page-header` pattern; more robust than intent-keyed dispatch for POS components |

---

## Common Pitfalls

### Pitfall 1: Missing patterns.md Update Fails CI
**What goes wrong:** Four new rules added to RULE_REGISTRY; `cargo test --all-features` fails on `patterns_md_matches_rule_registry` test in `design/mod.rs:325`.
**Why it happens:** The D-09 drift guard is bidirectional — any RULE_REGISTRY addition requires a matching `## \`rule-id\`` section in patterns.md.
**How to avoid:** Add the 4 patterns.md sections in the SAME task as the RULE_REGISTRY entries. Never split these across tasks.
**Warning signs:** `patterns.md is missing rule id 'pos-fill-viewport'` — panics at test time with file path.

### Pitfall 2: POS_TAP_HIGHLIGHT Not Generated in CSS
**What goes wrong:** `POS_TAP_HIGHLIGHT = "[-webkit-tap-highlight-color:transparent]"` added as constant but never appears in `ferro-base.css`. Phase 256 components use it but the CSS has no corresponding rule. Visual effect absent silently.
**Why it happens:** Tailwind v4 scanner may not detect arbitrary properties with vendor prefixes in Rust source files.
**How to avoid:** Use Path B (`@utility pos-tap-highlight { ... }` in `input.css`) OR include a Wave 0 verification step: run `scripts/gen-ferro-base-css.sh` and `grep "tap-highlight" ferro-json-ui/assets/ferro-base.css`.
**Warning signs:** CSS output is 0 bytes larger after adding the constant; grep finds nothing.

### Pitfall 3: Direction 3 Fails with Non-Builtin Component Name
**What goes wrong:** Any of the four new RULE_COMPONENTS entries maps to `"ProductGrid"`, `"CartPanel"`, `"Numpad"`, or `"QuantityStepper"` — components that are NOT yet in BUILTIN_TYPES at Phase 254.
**Why it happens:** Direction 3 checks `builtins.contains(c)` against the live catalog output (which only knows registered builtins).
**How to avoid:** All four new rules must map to existing builtins (`"Grid"` or `&[]`). Never name unregistered components in RULE_COMPONENTS until Phase 256.
**Warning signs:** `RULE_COMPONENTS references non-builtin component 'ProductGrid'` — panic in `design_system_component_guidance_drift_guarded`.

### Pitfall 4: Raw Literal Left in atoms.rs After Constant Migration
**What goes wrong:** `render_product_tile` migrates inline literals to constants but leaves one copy (e.g., in a comment or a second format! branch). The drift-guard test doesn't catch it because the guard checks only for the string in non-classes-rs render files.
**Why it happens:** Format strings are long; easy to miss one occurrence.
**How to avoid:** After migration, run `grep -n '"touch-manipulation"' ferro-json-ui/src/render/atoms.rs` — must return 0 results (the constant definition in classes.rs is the only source of the string).
**Warning signs:** Drift-guard test passes (because it finds no raw literal), but VSCode search shows the literal in atoms.rs — the guard would have caught it only if the file was scanned, which it is. Actually the guard WILL catch it. But code review is the first line of defense.

### Pitfall 5: Serde backward-compat Broken by Field Order
**What goes wrong:** Adding new fields to `ProductTileProps` between existing fields (not at the end) changes the serialization order in some formats, breaking round-trip tests that assert JSON equality.
**Why it happens:** serde_json serializes struct fields in declaration order.
**How to avoid:** Add new optional fields (`categories`, `image_url`, `color`, `stock_badge`) AFTER the existing 5 fields (`product_id`, `name`, `price`, `field`, `default_quantity`). The backward-compat test in D-04 uses `skip_serializing_if` which omits absent fields — field order only matters when fields are present.
**Warning signs:** `round-trip unchanged` test fails with different JSON key ordering.

### Pitfall 6: `allow` validation rejects new rule ids until registry update
**What goes wrong:** If a spec author tries to suppress a POS rule via `design.allow` before the rule is in RULE_REGISTRY, the `allow` validator emits a "Unknown allow id" warning. This is correct behavior — the issue would be if someone tests against the old rule registry.
**Why it happens:** `allow` validation in `design/mod.rs:108-120` checks against `RULE_REGISTRY` at runtime.
**How to avoid:** This is correct behavior. The `cassa.json` sample app should NOT pre-emptively add POS rule ids to its allow list before Phase 254 ships.

---

## Code Examples

### Adding a constant to classes.rs
```rust
// Source: ferro-json-ui/src/render/classes.rs — existing pattern
pub(crate) const POS_TOUCH_ACTION: &str = "touch-manipulation";
pub(crate) const POS_HIT_TARGET_MIN: &str = "min-h-[44px] min-w-[44px]";
pub(crate) const POS_OVERSCROLL_CONTAIN: &str = "overscroll-contain";
pub(crate) const POS_PRESS_ACTIVE: &str = "active:scale-95 active:bg-border";
// Path B (recommended):
pub(crate) const POS_TAP_HIGHLIGHT: &str = "pos-tap-highlight"; // defined via @utility in input.css
```

### Adding a lint rule to RULE_REGISTRY (D-11 internal-gate pattern)
```rust
// Source: ferro-json-ui/src/design/rules.rs — mirroring page-header pattern
DesignRule {
    id: "pos-fill-viewport",
    title: "POS register pages must fill the viewport",
    rationale: "A ProductGrid or CartPanel outside a fill_viewport spec causes silent \
                whole-page scroll, breaking the kiosk feel.",
    intents: &[], // all-intents; internal gate in check fn
    check: check_pos_fill_viewport,
},

fn check_pos_fill_viewport(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    const POS_TRIGGER_TYPES: &[&str] = &["ProductGrid", "CartPanel", "Numpad"];
    let has_pos = spec.elements.values()
        .any(|el| POS_TRIGGER_TYPES.contains(&el.type_name.as_str()));
    if !has_pos || spec.fill_viewport {
        return vec![];
    }
    vec![Finding {
        rule: "pos-fill-viewport",
        element_id: None,
        severity: Severity::Warning,
        message: "Spec contains POS components but fill_viewport is not set.".into(),
        suggestion: "Set fill_viewport: true at the spec level and add fill: true to the root Grid.".into(),
    }]
}
```

### Adding a RULE_COMPONENTS entry (D-14)
```rust
// Source: ferro-mcp/src/tools/json_ui_catalog.rs:81 — RULE_COMPONENTS static
// Add after the existing 11 entries:
("pos-fill-viewport", &["Grid"]),
("pos-grid-fill", &["Grid"]),
("pos-cart-present", &["Grid"]),
("fill-viewport-layout-unknown", &[]), // layout concern; no component; passes Direction 3
```

### serde backward-compat test pattern (D-04)
```rust
// Source: pattern from existing schema_smoke_tests module
#[test]
fn product_tile_legacy_json_round_trips_unchanged() {
    let legacy = r#"{"product_id":"p1","name":"Widget","price":"€10,00","field":"qty_p1"}"#;
    let props: ProductTileProps = serde_json::from_str(legacy).unwrap();
    assert!(props.categories.is_empty());
    assert!(props.image_url.is_none());
    assert!(props.color.is_none());
    assert!(props.stock_badge.is_none());
    let re_serialized = serde_json::to_string(&props).unwrap();
    // New optional fields must not appear when absent (skip_serializing_if)
    assert!(!re_serialized.contains("categories"));
    assert!(!re_serialized.contains("image_url"));
}
```

### patterns.md section format (D-09 requirement)
```markdown
## `pos-fill-viewport`

**Title:** POS register pages must fill the viewport

**Rationale:** A ProductGrid or CartPanel outside a fill_viewport spec causes silent
whole-page scroll, breaking the kiosk feel.

**Intents:** all (applies to any spec containing POS component types)

### Conforming example
...

### Violating example
...

### How to allow
...
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none — workspace Cargo.toml |
| Quick run command | `cargo test -p ferro-json-ui render::classes` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| POS-02 | ProductTileProps compiles with new fields | unit | `cargo test -p ferro-json-ui component::schema_smoke_tests` | ✅ module exists, new test added |
| POS-02 | Legacy JSON round-trips unchanged | unit | `cargo test -p ferro-json-ui` (new test in component.rs) | ❌ Wave 0 gap |
| POS-02 | render_product_tile output byte-identical for legacy spec | unit | `cargo test -p ferro-json-ui render` (new test in atoms.rs) | ❌ Wave 0 gap |
| POS-07 | 5 POS constants defined and composed correctly | unit | `cargo test -p ferro-json-ui render::classes` | ❌ Wave 0 gap |
| POS-07 | Drift guard: render files use constants not literals | unit | `cargo test -p ferro-json-ui render::classes::tests::pos_render_functions_use_constants_not_literals` | ❌ Wave 0 gap |
| POS-11 | 4 new rules: violating fixture | unit | `cargo test -p ferro-json-ui design::rules::tests` (new tests) | ❌ Wave 0 gap |
| POS-11 | 4 new rules: conforming fixture | unit | same | ❌ Wave 0 gap |
| POS-11 | 4 new rules: data-bound no-misfire | unit | same | ❌ Wave 0 gap |
| POS-11 | RULE_COMPONENTS exhaustive (SC-4) | unit | `cargo test -p ferro-mcp` | ✅ guard test exists; new entries needed |
| POS-11 | patterns.md in sync with RULE_REGISTRY (D-09) | unit | `cargo test -p ferro-json-ui design::docs_drift_tests` | ✅ test exists; new sections needed |

### Wave 0 Gaps
- [ ] Backward-compat round-trip test for `ProductTileProps` (new test in `component.rs`)
- [ ] Byte-identical render output test for legacy ProductTile spec (new test in `atoms.rs` or inline in `render_product_tile` cfg(test) block)
- [ ] `POS_*` constants composition tests in `classes.rs` tests module (3-5 new tests)
- [ ] `pos_render_functions_use_constants_not_literals` drift-guard test in `classes.rs` tests module
- [ ] 12 new fixtures in `design/rules.rs` tests module (4 rules × 3 fixtures each)
- [ ] 4 new `## \`rule-id\`` sections in `docs/src/design-system/patterns.md`

---

## Security Domain

Security enforcement is not the focus of this phase (no auth, no inputs, no data write paths). The lint rules and Props structs are pure data definitions + diagnostic outputs. No ASVS categories apply.

---

## Environment Availability

No external tool dependencies beyond the workspace toolchain. `scripts/gen-ferro-base-css.sh` uses `scripts/install-tailwind.sh` to auto-install the pinned Tailwind v4 CLI binary into `.tooling/bin/`. The script exists and is verified functional (used in prior phases). Run once at phase end.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Tailwind v4 CLI (auto-installed) | ferro-base.css regen (D-08) | auto-installed by gen-ferro-base-css.sh | pinned in scripts/install-tailwind.sh | none needed |
| cargo | all compilation | ✓ | workspace Rust toolchain | — |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `POS_PRESS_ACTIVE = "active:scale-95 active:bg-border"` — both classes not currently in ferro-base.css | Finding 3 | Low: if already in CSS, no harm; if not picked up by scanner, add to @source inline() |
| A2 | Tailwind v4 scanner picks up `[-webkit-tap-highlight-color:transparent]` as a class literal from Rust source | Finding 4 (Path A) | Medium: CSS may not be generated; use Path B to eliminate risk |
| A3 | `CategoryNavProps` should have `items: Vec<String>` rather than mandatory `data_path: String` | Finding 9 | Low: planner has discretion per D-17; either is valid |
| A4 | Phase 254 does NOT cause new lint findings on existing spec files (cassa.json etc.) that require immediate allow-list updates | Pitfall 6 | Low: existing specs don't yet have ProductGrid/CartPanel type names; rules check for those type names |

**All critical claims verified.** The assumptions above are discretionary design choices, not factual uncertainties about codebase behavior.

---

## Open Questions (RESOLVED)

1. **`POS_TAP_HIGHLIGHT` Path A vs Path B**
   - RESOLVED (Path B): `@utility pos-tap-highlight` adopted in Plan 254-02 Task 1 (input.css `@utility` analog, zero scanner risk).
   - What we know: Tailwind v4 arbitrary property syntax is supported; scanner behavior for vendor-prefixed arbitrary properties in Rust source files is uncertain
   - What's unclear: Whether `[-webkit-tap-highlight-color:transparent]` appearing as a Rust string literal is detected by the Tailwind v4 content scanner
   - Recommendation: Use Path B (`@utility pos-tap-highlight`) for zero risk; include a verification step if Path A is chosen

2. **`POS_HIT_TARGET_NUMPAD` (56px) — Phase 254 or 256?**
   - RESOLVED (Phase 254): `POS_HIT_TARGET_NUMPAD = "min-h-[56px] min-w-[56px]"` ships in Plan 254-02 Task 1 (pure declaration; classes.rs is its home; Phase 256 consumes it).
   - What we know: Numpad keys should be ≥56px per STACK.md; this phase declares `NumpadProps` but not its renderer
   - What's unclear: Whether declaring a 56px constant now is premature (the renderer that uses it lands in Phase 256)
   - Recommendation: Include `POS_HIT_TARGET_NUMPAD = "min-h-[56px] min-w-[56px]"` in Phase 254 since the constant is in `classes.rs` which is the right home and Phase 256 will need it immediately

3. **`infer.rs` ProductGrid → collect inference branch (D-17 discretionary)**
   - RESOLVED (skipped): infer.rs is not touched in Phase 254 — the four rules use `intents: &[]` with internal presence gates and fire regardless of inferred intent; revisit on gestiscilo adoption friction (CONTEXT deferred section).
   - What we know: Current `infer_intent` doesn't recognize ProductGrid; a spec with ProductGrid only gets `None` intent → `declare-intent` Info finding; POS lint rules use `intents: &[]` so they run regardless
   - What's unclear: Whether adding inference now helps (rules run regardless) or just adds complexity
   - Recommendation: Skip in Phase 254; the rules fire correctly via the internal presence gate; revisit after gestiscilo adoption (friction loop per CONTEXT deferred section)

---

## Sources

### Primary (HIGH confidence — codebase reads)
- `ferro-json-ui/src/render/classes.rs` — existing constants structure, 59 lines, test patterns
- `ferro-json-ui/src/render/atoms.rs:1357-1389` — render_product_tile exact inline literals
- `ferro-json-ui/src/layout.rs:330-405` — DefaultLayout, AppLayout, AuthLayout DOM structures
- `ferro-json-ui/assets/input.css:1-156` — @source scanning setup, @utility pattern, ferro-fill chain + comment
- `ferro-json-ui/src/design/rules.rs` — all 11 rules, DesignRule structure, internal-gate pattern
- `ferro-json-ui/src/design/types.rs` — Severity/Finding/DesignRule type definitions
- `ferro-json-ui/src/design/infer.rs` — infer_intent heuristics; no ProductGrid branch
- `ferro-json-ui/src/design/mod.rs:1-358` — lint() engine, KNOWN_INTENTS, D-09 drift guard test
- `ferro-json-ui/src/component.rs:882-914` — GridProps (spans convention for row_weights)
- `ferro-json-ui/src/component.rs:1340-1352` — ProductTileProps (5 existing fields)
- `ferro-json-ui/src/component.rs:1382-1411` — schema_smoke_tests module pattern
- `ferro-mcp/src/tools/json_ui_catalog.rs:81-96` — RULE_COMPONENTS static (11 entries)
- `ferro-mcp/src/tools/json_ui_catalog.rs:720-765` — 3-direction drift guard test
- `ferro-json-ui/src/runtime/mod.rs:1-130` — variant_classes_use_semantic_tokens test
- `docs/src/design-system/patterns.md` — section format requirement for D-09
- `scripts/gen-ferro-base-css.sh` — Tailwind CSS regen script

### Secondary (MEDIUM confidence — milestone research files)
- `.planning/research/STACK.md` — touch-action/press-state/tap-highlight class candidates, iOS Safari constraints
- `.planning/research/ARCHITECTURE.md` — build-sequence, integration point anchors
- `.planning/research/FEATURES.md` — Props struct field evidence from gestiscilo picker

---

## Metadata

**Confidence breakdown:**
- Supported-layout set: HIGH — directly read from layout.rs DOM structures + input.css comment
- POS constants (TOUCH_ACTION, HIT_TARGET_MIN, OVERSCROLL_CONTAIN): HIGH — existing inline literals verified
- POS_PRESS_ACTIVE class choice: MEDIUM — STACK.md evidence; planner discretion; needs CSS generation verification
- POS_TAP_HIGHLIGHT (Path B): HIGH — follows existing @utility pattern; Path A: MEDIUM
- D-07 drift-guard mechanism: HIGH — mirrors existing D-09 test exactly
- D-09 patterns.md hidden dependency: HIGH — test code verified
- RULE_COMPONENTS Direction 3 empty-slice: HIGH — guard code verified
- Props struct fields: MEDIUM — FEATURES.md is first-party but field names are planning discretion

**Research date:** 2026-07-05
**Valid until:** 2026-08-05 (stable codebase; no external deps)
