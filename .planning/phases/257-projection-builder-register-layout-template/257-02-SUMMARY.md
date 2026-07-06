---
phase: 257-projection-builder-register-layout-template
plan: "02"
subsystem: ferro-json-ui
tags: [projection, register-layout, tdd, lint-rules, pos, catalog-validation]
dependency_graph:
  requires: [ElementBuilder.each, SpecBuilder.fill_viewport, catalog-each-guard]
  provides: [register_template, emit_register_root, RegisterMissingAction]
  affects:
    - ferro-json-ui/src/projection/intent_layout.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-json-ui/src/projection/error.rs
    - ferro-json-ui/src/lib.rs
tech_stack:
  added: []
  patterns: [tdd-red-green, meaning-driven-field-mapping, fill-viewport-composition]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/projection/intent_layout.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-json-ui/src/projection/error.rs
    - ferro-json-ui/src/lib.rs
decisions:
  - "Register is a Collect layout override (template dispatch key), not a new intent — seven-intent vocabulary unchanged"
  - "spec.layout is always an app-shell name (dashboard); Register is never written into spec.layout"
  - "English field names in test fixture (CLAUDE.md project-agnostic rule); Italian names used only in app-land"
  - "GridProps has no Default impl — all fields spelled out explicitly in emit_register_root"
metrics:
  duration_minutes: 35
  completed_date: "2026-07-06"
  tasks_completed: 3
  files_modified: 4
requirements: [POS-10]
---

# Phase 257 Plan 02: Projection Builder — Register Layout Template Summary

Register layout projection: one `ServiceDef` with products and a confirm action derives a working tablet sale-screen spec entirely within the frozen seven-intent vocabulary. The killer feature is that this never required a new intent.

## What Was Built

**`register_template()` — Collect→Register override (D-02/D-03).** Public helper returning a `ThemeTemplates` that overrides the Collect intent's display layout to `"Register"`. All other intent fields are `None` (targeted override). The built-in `default_template(Collect)` (Form layout) is unaffected. Re-exported from `ferro_json_ui::register_template` for short-path access.

**`emit_register_root()` + `"Register"` arm (D-04/D-06/D-08/D-09).** Private helper in `builder.rs` composing the two-grid element tree that satisfies all four published register lint rules:

```
spec.root → register_root: Grid(columns=1, fill=true)
  sale_form: Form(id="sale_form", action=/{svc}/{confirm}, method=POST)
    panes_grid: Grid(columns=1, md_columns=3, spans=[2,1], fill=true)
      tiles_pane: TileGrid(data_path, form_id="sale_form", search=true)
        tile_tmpl: Tile($each, $data-bound item_id/name/price/price_cents/field)
      selection_pane: SelectionPanel(form_id="sale_form")
        confirm_btn: Button(label, Submit, form="sale_form", disable_on_submit=true)
```

All field bindings (item_id, name, price) are meaning-driven via `field_name_by(Identifier/EntityName/Money)` — no hardcoded field names. `price_cents` and `field` are fixed contract keys documented in the rustdoc per the per-row data contract.

**`fill_viewport` + `layout="dashboard"` wiring (D-05).** The Spec-assembly tail in `build_display_spec` sets `fill_viewport(true).layout("dashboard")` when the layout key is `"Register"`. This ensures all four register lint rules are structurally satisfied at projection time.

**`RegisterMissingAction` error (D-08).** New `ProjectionError` variant returned when a Register-layout ServiceDef has no actions. A register with no confirm target is broken by construction; the error is descriptive, never silent.

## Tasks

| Task | Commit | Result |
|------|--------|--------|
| 1: register_template() TDD RED | 068c0cc2 | Test fails (stub returns None) |
| 1: register_template() TDD GREEN + re-export | 90bdc040 | Test passes |
| 2: emit_register_root + Register arm + error | 681216b2 | Builds clean |
| 3: Integration tests (SC-1, D-05, D-08, D-14) | ca1cf722 | 4/4 tests green |

## Tests Added

- `intent_layout::tests::register_template_overrides_collect` — asserts Collect→Register override, all other intents None, default_template(Collect) still Form (regression).
- `builder::tests::register_projection_is_catalog_valid` — asserts fill_viewport=true, layout=dashboard, root Grid fill=true, exactly one Tile with $each, TileGrid+SelectionPanel+Form present.
- `builder::tests::register_projection_is_lint_clean` — asserts ZERO findings for all four register lint rules (register-fill-viewport, register-grid-fill, register-selection-present, fill-viewport-layout-unknown).
- `builder::tests::register_projection_no_actions_errors` — asserts `ProjectionError::RegisterMissingAction` variant.
- `builder::tests::register_projection_populated_data_validates` — asserts `catalog.validate` Ok with a populated data array carrying the per-row contract (price_cents + field keys).

All tests use the injected-catalog pattern (`from_service_def_with_catalog` + `clean_catalog()`).

## Deviations from Plan

**1. [Rule 1 - Bug] English names in test fixture instead of Italian**

- **Found during:** Task 3 authoring
- **Issue:** Plan specified Italian field names ("cassa", "nome", "prezzo", "Conferma"). CLAUDE.md requires `ferro-*` crates to use neutral English defaults only — no Italian strings.
- **Fix:** Test fixture uses English names ("shop", "name", "price", "confirm"). Functional equivalence is identical since meaning-driven dispatch (Identifier/EntityName/Money) does not care about field names.
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **Commit:** ca1cf722

**2. [Rule 1 - Bug] GridProps has no Default impl — plan's `..Default::default()` would not compile**

- **Found during:** Task 2 implementation
- **Issue:** Plan code samples used `GridProps { columns:1, fill:Some(true), ..Default::default() }` but `GridProps` does not derive or implement `Default`.
- **Fix:** Spelled out all fields explicitly in both `register_root` and `panes_grid` constructors.
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **Commit:** 681216b2

## CI Gate

Full CI-exact gate green:
- `cargo fmt --all -- --check` — clean
- `cargo clippy --all --all-targets --all-features -- -D warnings` — clean (49s)
- `cargo test -p ferro-json-ui --all-features` — 793 unit tests + 5+11+8 integration + 6 doc-tests, 0 failures
- `cargo doc --no-deps -p ferro-json-ui` — clean
- Schema export: no churn (docs/protocol/schemas/ unchanged)
- `KNOWN_INTENTS` and `REGISTER_TRIGGER_TYPES` in design/rules.rs: untouched — seven-intent vocabulary unchanged

## Known Stubs

None. The register projection emits a complete, catalog-valid, lint-clean spec. All elements are wired with real $data bindings or static values derived from the ServiceDef.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes at trust boundaries. The projector emits `{"$data":"/p/…"}` pointer objects only — no raw string interpolation of field values into markup (T-257-04). Meaning-driven Tile mapping restricts bindings to Identifier/EntityName/Money; Sensitive/ForeignKey meanings are structurally excluded (T-257-03). The `$each` iterated array is server-supplied by the handler (T-257-05, accepted). No threat flags.

## Self-Check: PASSED
