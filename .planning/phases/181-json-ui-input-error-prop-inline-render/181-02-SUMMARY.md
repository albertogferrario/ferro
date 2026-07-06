---
phase: 181
plan: 02
status: complete
wave: 2
commits:
  - 6c25369e
  - bdc74f86
files_modified:
  - framework/src/json_ui/mod.rs
  - ferro-json-ui/src/resolve.rs
---

## What was built

Wave 1 — Pipeline fixes. Two co-located changes landed in one plan as a joint fix:

- **Fix A** (`framework/src/json_ui/mod.rs`): `render_with_config` and `render_with_errors_config` now clone the spec and `merge_data(data.clone())` before calling `resolve` / `resolve_with_errors`. This mirrors the already-shipped `render_file_with_config` pattern at line 202. Runtime data is now visible to `resolve_expressions`, unlocking the manual `$data` binding authoring path per CONTEXT D-04.
- **Fix B** (`ferro-json-ui/src/resolve.rs`): `attach_errors` per-field branch now writes `error: String` (singular, first message wins) instead of `errors: Array<String>`. The shape now matches `InputProps.error: Option<String>` exactly. The `else if all` (full-bag) branch is intentionally unchanged — it serves a different contract.
- **Test fixture fix** (incidental, in Fix B commit): `form_spec_with_inputs` fixture placed `action` in `el.action` instead of `el.props`, which prevented the Form component from decoding correctly. Corrected to `el.props.action`.

## GREEN evidence

All 7 acceptance-criteria tests pass:

```
=== ferro-json-ui resolve tests ===
test resolve::tests::resolve_errors_matches_by_field_prop ... ok
test resolve::tests::resolve_errors_matches_by_name_prop ... ok
test resolve::tests::resolve_errors_all_writes_full_bag_when_no_match ... ok

=== framework (ferro-rs) tests ===
test json_ui::tests::pipeline_data_binding_error_prop_renders_p_tag ... ok   (D-07a RED → GREEN)
test json_ui::tests::pipeline_render_validation_error_renders_p_tag ... ok   (D-07b RED → GREEN)
test json_ui::tests::render_with_errors_populates_form_fields ... ok          (upgraded test RED → GREEN)
test json_ui::tests::render_validation_error_accepts_framework_type ... ok    (upgraded test RED → GREEN)
```

All four Wave 0 tests from Plan 01 transitioned RED → GREEN as expected. The two updated resolve.rs assertions now read singular `error`; the third (full-bag) test still asserts on `errors` plural unchanged.

## Decisions deferred

`JsonUi::render_json` and `JsonUi::render_json_with_errors` were NOT modified in this plan (per plan §Task 1 closing paragraph and RESEARCH open question 1). They return spec+data JSON, not rendered HTML, and their merge-data semantics are a separate design call. Documented as known follow-up.

## Acceptance criteria — verification

- `grep 'spec.clone().merge_data(data.clone())' framework/src/json_ui/mod.rs` → 2 matches ✓
- `grep 'props_obj.insert("error"' ferro-json-ui/src/resolve.rs` → 1 match (new per-field branch) ✓
- `grep 'props_obj.insert("errors"' ferro-json-ui/src/resolve.rs` → 1 match (unchanged `else if all` branch) ✓
- `grep 'msgs.first()' ferro-json-ui/src/resolve.rs` → 1 match (new first-wins logic) ✓
- All 7 named tests pass ✓
- No backward-compat shim added (D-08 clean break) ✓

## Per-CPU-feedback note

Per `feedback_one_cpu_op_at_a_time.md`: full-workspace `cargo test --all-features` and the canonical pre-commit gate are NOT run here. They run once at the phase boundary in Wave 7 / Plan 181-07. The targeted 7 tests above provide sufficient GREEN evidence for this plan.

## Key files modified

- `framework/src/json_ui/mod.rs` — Fix A merge_data at 2 call sites
- `ferro-json-ui/src/resolve.rs` — Fix B singular error shape + 2 test assertion updates + 1 fixture fix
