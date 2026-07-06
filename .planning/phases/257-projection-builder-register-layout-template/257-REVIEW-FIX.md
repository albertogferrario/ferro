---
phase: 257-projection-builder-register-layout-template
fixed_at: 2026-07-06T13:00:00Z
review_path: .planning/phases/257-projection-builder-register-layout-template/257-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 4
skipped: 1
status: partial
---

# Phase 257: Code Review Fix Report

**Fixed at:** 2026-07-06T13:00:00Z
**Source review:** .planning/phases/257-projection-builder-register-layout-template/257-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (fix_scope: critical_warning — CR-01, WR-01..WR-04; IN-01/IN-02 out of scope)
- Fixed: 4
- Skipped: 1

## Fixed Issues

### CR-01: /cassa register renders zero product tiles

**Files modified:** `app/src/controllers/cassa.rs`, `app/src/tests/cassa_render.rs`
**Commit:** cd2eba1b
**Applied fix:** Handler payload now nests rows per the projection convention:
`{ "data": { "cassa": cassa_products() } }`, so `$each` over `/data/cassa`
resolves and the tile grid renders. The app test uses the same nesting and
adds a content assertion (`html.contains("Caffè")`) so the suite can never
pass again with an empty grid.

### WR-01: vacuous "populated data" each-path tests

**Files modified:** `ferro-json-ui/src/catalog.rs`, `ferro-json-ui/src/projection/builder.rs`
**Commit:** 9d81c6bc
**Applied fix:** `catalog_each_template_populated_data` nests fixture rows under
a top-level `"data"` key so `/data/items` resolves; the resolves-to-array branch
now genuinely runs in `validate_directives` at `.build()` time. Added the
negative counterpart `catalog_each_template_path_not_array_rejected_at_build`
(non-array path -> `SpecError::EachPathNotArray`).
`register_projection_populated_data_validates` now nests data correctly and
re-validates the populated spec through `Spec::from_json` (the layer that runs
`validate_directives`), while keeping the `catalog.validate` end-to-end check
required by SC-4.

### WR-02: register fallback field ignored readable/meaning filtering

**Files modified:** `ferro-json-ui/src/projection/builder.rs`
**Commit:** 3309a043
**Applied fix:** The fallback in `emit_register_root` now selects the first
field with `f.readable && lookup_meaning(&f.meaning).display.is_some()`
(mirroring the sibling emitters), so Sensitive/ForeignKey fields can never be
bound into tile props (T-257-03). Added regression test
`register_projection_fallback_excludes_sensitive_fields`.

### WR-04: fill_viewport (and design) missing from the assembled full Spec JSON Schema

**Files modified:** `ferro-json-ui/src/catalog.rs`
**Commit:** 7f894237
**Applied fix:** `assemble_full_schema` root properties now include
`"fill_viewport": { "type": "boolean", "default": false }` and
`"design": { "$ref": "#/$defs/DesignMeta" }`; the `DesignMeta` schema is
hoisted into shared `$defs` alongside Action/Visibility. Added drift-guard
test `full_schema_root_exposes_all_spec_fields` asserting every `Spec` root
field is discoverable from the schema (MCP `json_ui_schema` surface).

## Skipped Issues

### WR-03: New public API surface undocumented in docs/src/

**File:** `ferro-json-ui/src/spec.rs:401-404,525-531`, `ferro-json-ui/src/projection/intent_layout.rs:50-66`, `framework/src/lib.rs:270`
**Reason:** locked decision D-19 / ROADMAP Phase 258 scope — docs/src register
and composition documentation is explicitly Phase 258 work; this phase's
documentation obligation is rustdoc only. Rustdoc coverage verified complete
on the new public surface: `register_template()` (intent_layout.rs, full
contract incl. Collect override and slot semantics), `ElementBuilder::each`
(spec.rs, path/as_ semantics + catalog skip note), `SpecBuilder::fill_viewport`
(spec.rs, fill:true + layout preconditions), framework re-export present
(framework/src/lib.rs:270).
**Original issue:** `register_template()`, `ElementBuilder::each(path, as_)`,
and `SpecBuilder::fill_viewport(bool)` have no coverage under docs/src/.

## Verification Gate

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --all --all-targets --all-features -- -D warnings`: PASS
- `cargo test --all-features`: PASS. One environmental failure during the
  fixer's run (`ferro-macros --test action_macro`, trybuild) was caused by
  disk-full (ENOSPC) scratch-build errors — the documented recurring
  environmental failure for the full gate on this machine, not a code defect.
  After freeing `target/tests` (~1.1 GB) and removing one ENOSPC-corrupted
  rlib, the orchestrator re-ran
  `cargo test -p ferro-macros --test action_macro --all-features`:
  `test action_macro_ui ... ok` (1 passed, 0 failed). No schema-export churn
  in the working tree.

---

_Fixed: 2026-07-06T13:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
