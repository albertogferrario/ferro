---
phase: 257-projection-builder-register-layout-template
verified: 2026-07-06T14:00:00Z
status: human_needed
score: 4/4
overrides_applied: 0
deferred:
  - truth: "New public API surface (register_template, ElementBuilder.each, SpecBuilder.fill_viewport) documented in docs/src/"
    addressed_in: "Phase 258"
    evidence: "Phase 258 goal: 'docs/src/json-ui/components.md covers all five new components'; D-19 locked decision; ROADMAP Phase 258 requirements POS-12/POS-13"
human_verification:
  - test: "Open /cassa in Chrome DevTools MCP at a tablet viewport (768px+), tap several product tiles, verify SelectionPanel updates with running total, panes scroll independently without scrolling the page, and the document body does not scroll"
    expected: "SelectionPanel shows tapped tiles with qty stepper + running total; product pane and selection pane scroll independently; body does not scroll (ferro-fill class chain active)"
    why_human: "Visual interaction quality (touch feel, pane independence, running-total accuracy) cannot be asserted by HTML string comparison"
---

# Phase 257: Projection Builder Register Layout Template — Verification Report

**Phase Goal:** A `ServiceDef` with products and cart fields derives a working sale screen within the existing Collect intent; the `/cassa` sample app serves the projection-derived spec without any `RawHtml`.
**Verified:** 2026-07-06T14:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A ServiceDef with products and a confirm action emits a Register-layout spec via `emit_register_root()`; `catalog_validate` accepts the output; seven-intent vocabulary and `KNOWN_INTENTS` drift guard unchanged | VERIFIED | `emit_register_root` at builder.rs:593; `"Register" =>` arm at builder.rs:262; `register_projection_is_catalog_valid` passes (fill_viewport=true, layout=dashboard, root Grid fill=true, exactly one $each Tile, TileGrid+SelectionPanel+Form present); KNOWN_INTENTS = 7 entries, "Register" absent (design/mod.rs:37-45) |
| 2 | /cassa is projection-derived, no RawHtml, `cassa.json` deleted; GET /cassa returns 200 (integration test) | VERIFIED | `register_template` called in cassa.rs index handler; `grep -rn "RawHtml\|render_file\|rimuovi" app/src/controllers/cassa.rs` returns nothing; `app/src/views/cassa.json` deleted; `rimuovi` route absent from routes.rs; `cassa_render_is_projection_derived_fill_viewport` passes with `resp.status_code() == 200` and `html.contains("Caffè")` (CR-01 fix confirmed) |
| 3 | `Spec::builder().fill_viewport(true)` is emitted for Register; fill_viewport propagates through catalog validation; HTML carries `ferro-fill` class chain | VERIFIED | `SpecBuilder::fill_viewport()` setter at spec.rs:401; `fill_viewport: self.fill_viewport_` in build() at spec.rs:470; `builder = builder.fill_viewport(true).layout("dashboard")` at builder.rs:279; `register_projection_is_catalog_valid` asserts `spec.fill_viewport == true`; `cassa_render_is_projection_derived_fill_viewport` asserts `html.contains("ferro-fill")` |
| 4 | `ElementBuilder.each(path, as_)` round-trips through serde; `catalog_validate` accepts the directive on a Tile template; $each-scoped `$data.*` path handling verified by integration test | VERIFIED | `ElementBuilder::each()` setter at spec.rs:525; `each_builder_round_trip` passes; `catalog_each_template_null_data` and `catalog_each_template_populated_data` pass (3/3 catalog_each tests); `register_projection_populated_data_validates` uses `Spec::from_json` to exercise `validate_directives` path-resolves-to-array branch with correct `{"data": {"shop": [...]}}` nesting (WR-01 fix applied) |

**Score:** 4/4 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | `register_template()`, `ElementBuilder::each()`, `SpecBuilder::fill_viewport()` documented in `docs/src/` | Phase 258 | Phase 258 SC-3: "docs/src/json-ui/components.md covers all five new components"; D-19 locked decision (CONTEXT.md:185-186): "docs/src register/composition documentation is Phase 258 scope"; ROADMAP POS-12 → Phase 258 |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/spec.rs` | `ElementBuilder::each` setter + `SpecBuilder::fill_viewport` setter | VERIFIED | `pub fn each` at :525; `pub fn fill_viewport` at :401; `fill_viewport: self.fill_viewport_` in `build()` at :470; tests `each_builder_round_trip` and `fill_viewport_builder` green |
| `ferro-json-ui/src/catalog.rs` | `$each` template-element guard Stage 2 + Stage 3 | VERIFIED | `el.each.is_some()` continue at :770 (Stage 2); `obj.remove("props")` at :863 (Stage 3 — removes key rather than nulling, correct fix per deviation note); `full_schema_root_exposes_all_spec_fields` drift guard at :2513; `"fill_viewport"` in assemble_full_schema root properties at :592 (WR-04 fix) |
| `ferro-json-ui/src/projection/intent_layout.rs` | `register_template()` Collect→Register override helper | VERIFIED | `pub fn register_template` at :50; returns `ThemeTemplates` with Collect→Register, all other intents None; `register_template_overrides_collect` test asserts regression (default_template(Collect) still Form) |
| `ferro-json-ui/src/projection/builder.rs` | `emit_register_root` + `"Register"` arm + fill_viewport/layout wiring | VERIFIED | `fn emit_register_root` at :593; `"Register" => emit_register_root` at :262; `fill_viewport(true).layout("dashboard")` at :279; meaning-driven `field_name_by` closure with readable+display-eligible fallback (WR-02 fix at :625-630); `RegisterMissingAction` variant used correctly |
| `ferro-json-ui/src/projection/error.rs` | `RegisterMissingAction` error variant | VERIFIED | Variant at :40 with descriptive error message |
| `app/src/controllers/cassa.rs` | Projection-derived handler + `cassa_service_def` + `cassa_products` pub helpers | VERIFIED | `pub fn cassa_service_def` at :10; `pub fn cassa_products` at :23; index handler uses `JsonUiRenderer.render` with `register_template()`; data nested as `{ "data": { "cassa": ... } }` (CR-01 fix) |
| `app/src/tests/cassa_render.rs` | Integration test: 200 + ferro-fill + register markers + lint-clean + product tiles | VERIFIED | `cassa_render_is_projection_derived_fill_viewport` asserts: `spec.fill_viewport==true`, `spec.layout=="dashboard"`, zero register lint findings, 200 status, `ferro-fill` class, `data-selection-panel`, `data-filter-search`, "Conferma ordine", "Caffè" (CR-01 content guard) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `build_display_spec` Register arm | `Spec::builder().fill_viewport(true).layout("dashboard")` | layout-conditional at spec assembly | VERIFIED | builder.rs:276-281: `if layout == "Register" { builder = builder.fill_viewport(true).layout("dashboard"); }` |
| `emit_register_root` Tile template | `field_name_by(Identifier/EntityName/Money)` $data bindings | meaning-driven prop mapping | VERIFIED | builder.rs:618-636: `field_name_by` closure + readable+display-eligible fallback; no hardcoded "id"/"nome"/"prezzo" string literals as tile prop pointers |
| `app/src/controllers/cassa.rs` index | `JsonUiRenderer.render(&service, &intents, &ctx)` with `register_template()` | projection call | VERIFIED | cassa.rs:77-82 wires the renderer; cassa.rs:85: `json!({ "data": { "cassa": cassa_products() } })` (CR-01 fix) |
| `app/src/routes.rs` | No `cassa::rimuovi` reference | route deletion | VERIFIED | routes.rs has only `cassa.index` (GET /cassa) and `cassa.conferma` (POST /cassa/conferma) |
| `app/src/tests/mod.rs` | `pub mod cassa_render` | test module registration | VERIFIED | mod.rs:1: `pub mod cassa_render;` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `cassa.rs` index → `JsonUi::render` | `spec` (from `JsonUiRenderer.render`) + `data` (from `cassa_products()`) | ServiceDef→projection→24 hardcoded product rows with price_cents | Yes — `cassa_products()` synthesizes 24 real rows with all per-row contract keys | FLOWING |
| `cassa_render.rs` test | `html` | `JsonUi::render(&spec, &data)` where data = `{ "data": { "cassa": ... } }` | Yes — `html.contains("Caffè")` asserts a specific product name renders | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `each_builder_round_trip` test | `cargo test -p ferro-json-ui --features projections -- each_builder` | 1 passed | PASS |
| `catalog_each_template` tests (3) | `cargo test -p ferro-json-ui --features projections -- catalog_each` | 3 passed | PASS |
| `register_projection` tests (5) | `cargo test -p ferro-json-ui --features projections -- register_projection` | 5 passed | PASS |
| `cassa_render_is_projection_derived_fill_viewport` | `cargo test -p app -- cassa_render` | 1 passed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| POS-10 | 257-01, 257-02, 257-03 | ServiceDef → Register layout template under Collect; seven-intent vocabulary unchanged | SATISFIED | SC-1 through SC-4 all verified; `emit_register_root` + `register_template()` + `fill_viewport` + `$each` all shipping and tested; KNOWN_INTENTS unchanged |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-json-ui/src/projection/builder.rs` | 171, 360-362, 903, 968 | `placeholder` references in non-Register emit helpers | Info | Pre-existing; not in Register/cassa code paths; no impact on SC-1/SC-2/SC-3/SC-4 |

No blocking anti-patterns found in this phase's new code. The `placeholder` references are pre-existing in other layout arms (`emit_actions_placeholder`, `emit_body_placeholder`) untouched by this phase.

### Human Verification Required

#### 1. Tablet Visual Quality — /cassa Register Feel

**Test:** Run the app (`cargo run -p app`), open `/cassa` in Chrome DevTools MCP at a tablet viewport (768×1024 or similar). Tap several product tiles. Verify: SelectionPanel updates with a running total line for each tapped product; per-line QuantityStepper shows; panes scroll independently (product pane and selection pane each scroll within themselves); the browser body/window does NOT scroll; the "Conferma ordine" confirm button is visible and tappable.

**Expected:** Tile taps add lines to SelectionPanel with correct item name + running total in cents; pane scroll is contained (not whole-page); ferro-fill class on body is visually effective; register feel is cohesive at tablet size.

**Why human:** Visual interaction quality, pane scroll independence, running-total accuracy, and the overall register feel are not assertable by HTML string comparison tests. The integration test confirms structural correctness (ferro-fill class present, SelectionPanel renders, Caffè renders from $each) but cannot verify that the fill-viewport layout actually works at the browser rendering level.

### Gaps Summary

No gaps. All four success criteria are verified. The single outstanding item is human-only visual validation of tablet interaction quality.

The WR-03 docs/src/ gap from the code review is properly deferred to Phase 258 (D-19, POS-12), which explicitly includes `docs/src` updates as a success criterion.

---

_Verified: 2026-07-06T14:00:00Z_
_Verifier: Claude (gsd-verifier)_
