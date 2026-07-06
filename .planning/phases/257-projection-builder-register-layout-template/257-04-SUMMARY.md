---
phase: 257-projection-builder-register-layout-template
plan: "04"
subsystem: ferro-json-ui
tags: [pos, register, form, height-chain, fill-viewport, css]
dependency_graph:
  requires: [257-03]
  provides: [POS-10-gap-closed, register-fill-form]
  affects: [ferro-json-ui, app/tests/cassa_render]
tech_stack:
  added: []
  patterns:
    - "FormProps.fill additive optional prop with serde default + skip_serializing_if"
    - "fill/non-fill class selection via boolean branch on full string literals (Tailwind scanner discipline)"
    - "fill height-chain: flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0"
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/projection/builder.rs
    - app/src/tests/cassa_render.rs
    - ferro-json-ui/assets/ferro-base.css
decisions:
  - "fill height-chain uses flex-col + [&>*]:flex-1/min-h-0 (not grid) to mirror SelectionPanel shell pattern and reliably stretch the single panes_grid child to full Form height"
  - "fill:absent/false path is byte-identical to prior renders — additive, no migration"
  - "Form-in-fill-chain design lint rule deferred as a future design-system candidate"
metrics:
  duration: "~30m"
  completed: "2026-07-06"
  tasks_completed: 2
  files_modified: 5
requirements: [POS-10]
---

# Phase 257 Plan 04: Form Fill Height-Chain Summary

Closes the single major UAT gap on the projection-derived `/cassa` register: additive `FormProps.fill` + fill-aware `render_form` + `emit_register_root` wired to `fill:true`, with regression coverage at three layers and regenerated CSS.

## What Was Built

**The UAT gap:** Under `fill_viewport`, the SelectionPanel footer (running Total + "Conferma ordine" button) rendered off-viewport at y≈1032–1125 in a 746px viewport. Root cause: `emit_register_root` composes `Grid(fill:true)` → `Form#sale_form` → inner `Grid(h-full)` → panes. The outer cell was correctly height-constrained (673px), but `render_form` emitted `flex flex-wrap` with no height-chain classes, making the Form content-sized (1076px). The inner Grid's `h-full` resolved against 1076px, not 673px.

**The fix (three-layer, additive, backward-compatible):**

1. `FormProps.fill: Option<bool>` — new optional field on `FormProps` in `ferro-json-ui/src/component.rs`, with `#[serde(default, skip_serializing_if = "Option::is_none")]` and rustdoc explaining the 256 D-15 context. All existing `FormProps` literals updated to include `fill: None` (`build_input_spec`).

2. `render_form` fill-aware class selection — in `ferro-json-ui/src/render/form.rs`, a `let form_classes = if props.fill == Some(true)` branch before the `match &props.guard` block selects between the fill class literal (`"flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0"`) and the default class literal (byte-identical to before). Both format! branches use `{form_classes}` interpolation.

3. `emit_register_root` wired — in `ferro-json-ui/src/projection/builder.rs`, the sale_form `FormProps` literal gets `fill: Some(true)`. The adjacent comment explains the 256 D-15 height-chain purpose.

**Regression coverage at three layers:**

- `render_form_fill_true_emits_height_chain` (form.rs tests): fill:true form contains the full height-chain class string, does not contain `flex flex-wrap`.
- `render_form_default_class_is_byte_identical` (form.rs tests): default form contains the byte-identical default class, does not contain `h-full` or `min-h-0`.
- `register_projection_sale_form_carries_fill` (builder.rs tests): the register spec's Form element carries `fill:true` in its props.
- `cassa_render_is_projection_derived_fill_viewport` (app/tests): rendered HTML contains `[&>*]:flex-1 [&>*]:min-h-0` as a proxy for the fill height-chain (geometry not assertable in Rust; live UAT deferred).

**CSS:** `ferro-base.css` regenerated — `[&>*]:flex-1` and `[&>*]:min-h-0` utilities added (3 total `[&>*]` rules: existing `[&>*]:w-full` + two new). Size: 41,912 bytes.

**CI-exact gate:** All four steps green: `cargo fmt --all -- --check` → `cargo clippy --all --all-targets --all-features -- -D warnings` → `cargo test --all-features` → `cargo doc --no-deps`.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The fill height-chain is fully wired: `FormProps.fill` → `render_form` class selection → `emit_register_root fill:Some(true)` → rendered `<form class="flex flex-col h-full min-h-0 [&>*]:flex-1 [&>*]:min-h-0">`.

Live geometry re-verify (deferred to UAT): re-open `/cassa` in Chrome DevTools at 1024×768 and confirm the SelectionPanel Total + "Conferma ordine" sit inside the viewport while the tiles pane and cart lines scroll independently.

## Threat Flags

None — this change emits static CSS class literals on a `<form>` element. No new trust boundary, network endpoint, auth path, or schema change introduced.

## Self-Check: PASSED

All 6 key files found on disk. Both task commits (eef721b9, 156150e0) confirmed in git log.
