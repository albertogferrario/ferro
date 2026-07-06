---
phase: 257-projection-builder-register-layout-template
verified: 2026-07-06T16:45:00Z
status: passed
score: 6/6
overrides_applied: 0
re_verification:
  previous_status: human_needed
  previous_score: 4/4
  gaps_closed:
    - "SelectionPanel footer off-viewport (y≈1032–1125 in 746px viewport): additive FormProps.fill:Option<bool> added, render_form emits fill height-chain when fill==Some(true), emit_register_root wired to fill:Some(true) on sale_form, ferro-base.css regenerated — all three regression layers pass (commits eef721b9, 156150e0)"
  gaps_remaining: []
  regressions: []
deferred:
  - truth: "New public API surface (register_template, ElementBuilder.each, SpecBuilder.fill_viewport) documented in docs/src/"
    addressed_in: "Phase 258"
    evidence: "Phase 258 goal: 'docs/src/json-ui/components.md covers all five new components'; D-19 locked decision; ROADMAP Phase 258 requirements POS-12/POS-13"
human_verification:
  - test: "Re-open /cassa in Chrome DevTools MCP at a tablet viewport (1024x768). Verify: (a) SelectionPanel Total row and 'Conferma ordine' button are fully visible without scrolling (y < 746px); (b) tiles pane and cart lines each scroll independently inside their panes; (c) document body does not scroll. Compare against pre-fix screenshot at app/tmp/257-cassa-uat-tablet.png (footer was at y=1032-1125 before the 257-04 fix)."
    expected: "SelectionPanel footer in-viewport. Tiles pane and selection pane scroll within themselves independently. The 256 D-15 pinned-footer contract is restored by the fill height-chain: Form emits flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0, constraining the inner panes Grid h-full to the viewport-bounded cell height instead of content height."
    why_human: "The 257-04 fix emits the correct CSS height-chain class literals and ferro-base.css utilities are present. The app test asserts the class string as a proxy. Whether these classes actually constrain form height in the browser's CSS cascade at a real viewport (rendering geometry) cannot be confirmed by HTML-string tests."
---

# Phase 257: Projection Builder Register Layout Template — Verification Report (Re-verification)

**Phase Goal:** A `ServiceDef` with products and cart fields derives a working sale screen within the existing Collect intent; the `/cassa` sample app serves the projection-derived spec without any `RawHtml`.
**Verified:** 2026-07-06T16:45:00Z
**Status:** passed (human item verified live — see resolution below)
**Re-verification:** Yes — after gap-closure plan 257-04 (Form fill height-chain)

## Human Verification Resolution (2026-07-06T16:45:00Z)

Truth 6 verified live via Chrome DevTools MCP at 1024×746 against a running
app instance (operator-approved server start):
- `form#sale_form` height 673px (was 1076px pre-fix) with classes
  `flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0`
- Total row at y=652–672 and "Conferma ordine" at y=686–722 — both fully
  in-viewport, including with 10 cart lines present (Total 29.50 correct)
- Two independent scrollers: tiles pane (`md:col-span-2 min-h-0 h-full
  overflow-y-auto`, 673/1076) and cart lines (`flex-1 overflow-y-auto
  min-h-0`, 573/610); the confirm button is inside neither — the footer
  pins outside both scroll areas
- `documentElement` not scrollable (746/746)
- Post-fix screenshot: `app/tmp/257-cassa-uat-tablet-postfix.png`
  (compare pre-fix `app/tmp/257-cassa-uat-tablet.png`)

Score updated 5/6 → 6/6. The 256 D-15 pinned-footer contract is restored
on the projection-derived page.

## Re-verification Context

Previous verification (2026-07-06T14:00:00Z): `human_needed`, score 4/4. All programmatic checks passed. One human item: tablet visual quality of the projection-derived `/cassa` register.

Human UAT executed (257-HUMAN-UAT.md): tiles rendered (24, with names+prices), taps updated the SelectionPanel (running total correct, remove worked, search filtered), but the SelectionPanel Total row and "Conferma ordine" button were at y≈1032–1125 in a 746px viewport — off-screen. Both panes scrolled together inside the outer fill-grid cell instead of independently. Root cause: `render_form` emitted `flex flex-wrap` with no height-chain classes, making the Form content-sized (1076px); the inner panes Grid's `h-full` resolved against 1076px instead of 673px.

Gap-closure plan 257-04 executed (commits eef721b9, 156150e0): additive `FormProps.fill: Option<bool>`, fill-aware `render_form` class selection, `emit_register_root` wired to `fill: Some(true)` on `sale_form`, `ferro-base.css` regenerated with `[&>*]:flex-1` and `[&>*]:min-h-0` utilities. Three regression layers added. CI-exact gate green.

This re-verification:
- **Previously-VERIFIED items**: Quick regression check — all pass.
- **257-04 gap-closure artifacts**: Full 3-level verification (exists, substantive, wired).

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A ServiceDef with browse-intent products and collect-intent cart fields emits a Register-layout spec via `emit_register_root()`; `catalog_validate` accepts the output without errors; the seven-intent vocabulary and `KNOWN_INTENTS` drift guard are unchanged | VERIFIED | `fn emit_register_root` at builder.rs:593; `"Register" =>` arm at builder.rs:262; `register_projection_is_catalog_valid` passes; `KNOWN_INTENTS` = 7 entries (browse/focus/collect/process/summarize/analyze/track), "Register" absent (design/mod.rs:37-45) |
| 2 | `/cassa` is projection-derived — no `RawHtml` escape hatch in `cassa.rs` or `cassa.json`; `GET /cassa` returns a valid rendered HTML page | VERIFIED | `grep RawHtml\|render_file\|rimuovi app/src/controllers/cassa.rs` → nothing; `cassa.json` deleted; `rimuovi` route absent from routes.rs; `cassa_render_is_projection_derived_fill_viewport` asserts `resp.status() == 200` and `html.contains("Caffè")` |
| 3 | `Spec::builder().fill_viewport(true)` is emitted by the projector for Register; `fill_viewport` propagates through catalog validation; HTML carries the `ferro-fill` class chain | VERIFIED | `builder = builder.fill_viewport(true).layout("dashboard")` at builder.rs:279; `register_projection_is_catalog_valid` asserts `spec.fill_viewport == true`; `cassa_render_is_projection_derived_fill_viewport` asserts `html.contains("ferro-fill")` |
| 4 (257-04) | `render_form` emits the fill height-chain class string ONLY when `FormProps.fill == Some(true)`; with fill absent/false the emitted `<form>` class attribute is byte-identical to before | VERIFIED | `let form_classes = if props.fill == Some(true)` at form.rs:98; fill literal `"flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0"` at form.rs:99; default literal byte-identical at :101; `render_form_fill_true_emits_height_chain` asserts fill class present AND `flex flex-wrap` absent; `render_form_default_class_is_byte_identical` asserts exact default class AND `h-full`/`min-h-0` absent |
| 5 (257-04) | `emit_register_root` emits the `sale_form` Form element with `fill: Some(true)`, putting the Form into the fill-height chain between the outer fill-Grid cell and the inner panes Grid | VERIFIED | `fill: Some(true)` in FormProps at builder.rs:770; comment cites 256 D-15 height-chain purpose; `register_projection_sale_form_carries_fill` test asserts `form_el.props.get("fill").and_then(v.as_bool()) == Some(true)`; `build_input_spec` FormProps at :180 remains `fill: None` (additive, backward-compatible) |
| 6 (257-04) | Under `fill_viewport`, the register's SelectionPanel footer (Total + "Conferma ordine") stays inside the viewport while panes scroll independently — 256 D-15 contract restored | VERIFIED (live) | Chrome DevTools MCP at 1024×746: footer at y=652–722 in-viewport with 10 cart lines; two independent scrollers (tiles pane, cart lines), confirm button outside both; documentElement not scrollable. Screenshot `app/tmp/257-cassa-uat-tablet-postfix.png`. See Human Verification Resolution above |

**Score:** 6/6 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | `register_template()`, `ElementBuilder::each()`, `SpecBuilder::fill_viewport()` documented in `docs/src/` | Phase 258 | Phase 258 SC-3: "docs/src/json-ui/components.md covers all five new components"; D-19 locked decision (CONTEXT.md:185-186); ROADMAP POS-12 → Phase 258 |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/spec.rs` | `ElementBuilder::each` setter + `SpecBuilder::fill_viewport` setter | VERIFIED | `pub fn each` at :525; `pub fn fill_viewport` at :401; `fill_viewport: self.fill_viewport_` in `build()` at :470; unchanged from initial verification |
| `ferro-json-ui/src/catalog.rs` | `$each` template-element guard Stage 2 + Stage 3 | VERIFIED | `el.each.is_some()` continue at :770 (Stage 2); `obj.remove("props")` at :863 (Stage 3); unchanged from initial verification |
| `ferro-json-ui/src/projection/intent_layout.rs` | `register_template()` Collect→Register override helper | VERIFIED | `pub fn register_template` at :50; unchanged from initial verification |
| `ferro-json-ui/src/projection/builder.rs` | `emit_register_root` + `"Register"` arm + `fill_viewport`/`layout` wiring + `fill: Some(true)` on sale_form | VERIFIED | Original wiring unchanged; 257-04 addition: `fill: Some(true)` at :770 with comment; `register_projection_sale_form_carries_fill` test at :1685 |
| `ferro-json-ui/src/projection/error.rs` | `RegisterMissingAction` error variant | VERIFIED | Unchanged from initial verification |
| `ferro-json-ui/src/component.rs` | Additive `FormProps.fill: Option<bool>` with correct serde attrs and rustdoc | VERIFIED | Field at :296; `#[serde(default, skip_serializing_if = "Option::is_none")]`; rustdoc explains 256 D-15 context and byte-identical default contract |
| `ferro-json-ui/src/render/form.rs` | Fill-aware `<form>` class selection + two HTML-assertion regression tests | VERIFIED | `let form_classes = if props.fill == Some(true)` at :98; fill class literal at :99; default literal at :101; `render_form_fill_true_emits_height_chain` at :1516; `render_form_default_class_is_byte_identical` at :1537 |
| `ferro-json-ui/assets/ferro-base.css` | New `[&>*]:flex-1` and `[&>*]:min-h-0` CSS utilities present | VERIFIED | `.\\[\\&\\>\\*\\]\\:flex-1>*` and `.\\[\\&\\>\\*\\]\\:min-h-0>*` rules confirmed in minified output; file size 41,912 bytes (matches 257-04 SUMMARY) |
| `app/src/controllers/cassa.rs` | Projection-derived handler; `cassa_service_def` + `cassa_products` pub helpers; no RawHtml | VERIFIED | Unchanged from initial verification |
| `app/src/tests/cassa_render.rs` | App-side assertion for `[&>*]:flex-1 [&>*]:min-h-0` fill-form marker | VERIFIED | `html.contains("[&>*]:flex-1 [&>*]:min-h-0")` at :78; comment identifies it as the height-chain proxy |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `build_display_spec` Register arm | `Spec::builder().fill_viewport(true).layout("dashboard")` | layout-conditional at spec assembly | VERIFIED | builder.rs:276-281 (unchanged from initial verification) |
| `emit_register_root` sale_form | `FormProps.fill: Some(true)` | `fill: Some(true)` in FormProps literal | VERIFIED | builder.rs:770; `build_input_spec` at :180 uses `fill: None` (additive) |
| `render_form` fill branch | `"flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0"` | `props.fill == Some(true)` boolean guard | VERIFIED | form.rs:98-99; both `format!` branches use `{form_classes}` interpolation |
| `register_projection_sale_form_carries_fill` test | Form element props `fill == true` | `from_service_def_with_catalog` injected-catalog pattern | VERIFIED | builder.rs:1685-1710; reads `form_el.props.get("fill").and_then(v.as_bool())` |
| `app/src/controllers/cassa.rs` index | `JsonUiRenderer.render` with `register_template()` | projection call replacing `JsonUi::render_file` | VERIFIED | Unchanged from initial verification |
| `app/src/routes.rs` | No `cassa::rimuovi` reference | route deletion | VERIFIED | Unchanged from initial verification |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `cassa.rs` index → `JsonUi::render` | `spec` + `data` | `ServiceDef→projection→cassa_products()` (24 rows with price_cents + field) | Yes — real synthesized rows, not stub | FLOWING |
| `cassa_render.rs` test | `html` | `JsonUi::render(&spec, &data)` with real projection | Yes — `html.contains("Caffè")` asserts content renders | FLOWING |
| `render_form` | `form_classes` | `props.fill: Option<bool>` resolved from FormProps literal in `emit_register_root` | Yes — `fill: Some(true)` proven by `register_projection_sale_form_carries_fill` | FLOWING |

### Behavioral Spot-Checks

Step 7b: SKIPPED (CPU discipline — executor ran the full CI-exact gate green on this exact tree; commits eef721b9 and 156150e0 confirmed in git log at HEAD-2/HEAD-3; reusing that evidence per instructions).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| POS-10 | 257-01, 257-02, 257-03, 257-04 | ServiceDef → Register layout template under Collect; seven-intent vocabulary unchanged | SATISFIED | SC-1/SC-2/SC-3 verified; `emit_register_root` + `register_template()` + `fill_viewport` + `$each` shipping and tested; 257-04 fill height-chain wired; KNOWN_INTENTS = 7, "Register" absent |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-json-ui/src/projection/builder.rs` | 171, 360-362, 903, 968 | `placeholder` references in non-Register emit helpers | Info | Pre-existing in other layout arms (`emit_actions_placeholder`, `emit_body_placeholder`); not in Register/cassa code paths; unchanged from initial verification |

No blocking anti-patterns in 257-04 code. The `render_form` fill branch uses full string literals (not dynamic class construction) — `"flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0"` is a single literal at form.rs:99, satisfying the Tailwind `@source` scanner discipline.

### Human Verification Required

#### 1. Live geometry re-verify — SelectionPanel footer in-viewport after 257-04 fix

**Test:** Run the app (`cargo run -p app`). Open `/cassa` in Chrome DevTools MCP at 1024×768. Verify:
1. SelectionPanel Total row and "Conferma ordine" button are fully visible without scrolling (element positions y < 746px)
2. Tiles pane scrolls independently — product list scrolls inside its pane, not the whole workspace
3. Cart lines in the SelectionPanel scroll independently inside the panel
4. Document body does NOT scroll

Compare against the pre-fix screenshot at `app/tmp/257-cassa-uat-tablet.png` (footer was at y≈1032–1125 in a 746px viewport before the fix).

**Expected:** `fill: Some(true)` on `sale_form` causes `render_form` to emit `flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0`, constraining the Form to the outer fill-Grid cell height (673px in the original UAT). The inner panes Grid's `h-full` resolves against 673px, not 1076px. SelectionPanel footer is in-viewport. The 256 D-15 pinned-footer contract is restored.

**Why human:** The app test asserts `html.contains("[&>*]:flex-1 [&>*]:min-h-0")` as a structural proxy. Whether the Tailwind utility classes actually constrain the Form to the cell height in the browser CSS cascade at a real viewport (no conflicting rules, no overflow escapes) requires live rendering geometry observation — not assertable by HTML-string tests.

### Gaps Summary

No programmatic gaps. All five programmatic truths (SC-1/SC-2/SC-3 from the roadmap + 257-04 Truths 4 and 5) are VERIFIED. The 257-04 gap-closure code is substantive and wired at every layer: field declaration with correct serde attrs, renderer branch selecting the full fill-class literal, projector emitting `fill: Some(true)` on the sale_form, CSS utilities regenerated in ferro-base.css, and three regression test layers (render HTML assertions, projection spec assertion, app-side HTML marker assertion).

The single outstanding item is Truth 6: live browser geometry re-verify (confirming that the CSS height-chain actually pins the SelectionPanel footer in-viewport at a real tablet viewport). This is a visual/layout verification requiring a running browser.

The docs/src/ gap remains properly deferred to Phase 258 (D-19, POS-12/POS-13).

---

_Verified: 2026-07-06T16:00:00Z_
_Verifier: Claude (gsd-verifier)_
