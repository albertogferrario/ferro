---
phase: 147
plan: 03
subsystem: ferro-json-ui
tags: [render, detail-form, structural-coherence, xss-hardening, method-spoofing]
dependency_graph:
  requires:
    - "147-01 (Wave-0 RED tests landed at render.rs:8629+)"
    - "147-02 (Component::DetailForm variant, DetailFormProps, DetailField, EditMode, ComponentNode::detail_form — adjacent Wave-1 plan)"
  provides:
    - "fn render_detail_form(props: &DetailFormProps, data: &Value) -> String"
    - "Component::DetailForm dispatch arm in render_component"
    - "Component::DetailForm container arm in collect_plugin_types_node"
  affects:
    - "render.rs imports (DetailFormProps, EditMode added to non-test use block)"
tech-stack:
  added: []
  patterns:
    - "Structural-coherence contract (§5 of 147-UI-SPEC): identical <dl> opening tag + every <dt>/<dd> wrapper across View and Edit modes"
    - "Option-A label rule (§9): empty inner <label></label>; aria-label wrapper in <dd> supplies screen-reader name without mutating caller props"
    - "Method-spoofing block (T-147-01): verbatim copy of render_form pattern — enum-typed effective_method, fixed literals only"
    - "html_escape discipline (T-147-02): 9 html_escape call sites in render_detail_form body"
key-files:
  created: []
  modified:
    - "ferro-json-ui/src/render.rs (+158 / -7)"
decisions:
  - "Accessibility wrapper is a <span role='group' aria-label='{dt-text}'> inside <dd> (Edit mode only). Avoids mutating caller-supplied input props (§9 Option A) while still meeting §11/§14.4 aria-label requirement. Wrapper is inside <dd>, so §5 structural-coherence invariant (same <dl>, <dt>, <dd> wrapper across modes) is preserved."
  - "DetailField type NOT added to non-test import block — iteration over props.fields type-infers the element; adding DetailField would be an unused import under clippy."
  - "Wave-1 parallel scope fence: render.rs only; no changes to component.rs, lib.rs, resolve.rs, json_ui_catalog.rs, docs. Full compile gate deferred to post-wave merge point."
metrics:
  duration: ~15min
  completed: 2026-04-23
---

# Phase 147 Plan 03: DetailForm Renderer Summary

Implements `render_detail_form(props, data) -> String` in `ferro-json-ui/src/render.rs`, wires the `Component::DetailForm` dispatch arm in `render_component`, and adds the container arm for `Component::DetailForm` in `collect_plugin_types_node` — turning the twelve+ Wave-0 RED render tests GREEN (subject to sibling Wave-1 plans landing their types).

## Work completed

### Insertion 1 — `fn render_detail_form` (render.rs:1039)

Inserted immediately after `render_form`'s closing brace (render.rs:1038). The function:

1. Builds a single `<dl class="grid grid-cols-1 gap-4">…</dl>` body whose `<dl>` opening tag and every `<dt>…</dt>` block are byte-for-byte identical across modes (structural-coherence contract §5 of 147-UI-SPEC). Only each `<dd>`'s inner content differs — View emits `html_escape(field.value)`, Edit emits a `<span role="group" aria-label="{dt-text}">{render_node(field.input, data)}</span>` wrapper around the rendered input.
2. Builds a right-aligned action bar using `<div class="flex gap-2 justify-end mt-6">…</div>`:
   - **View:** one `<a>` "Modifica" link styled with the outline class bundle.
   - **Edit:** one `<a>` "Annulla" link (outline) + one `<button type="submit">` "Salva" (primary).
   - Italian defaults per D-14; overrides via `edit_label` / `save_label` / `cancel_label`.
3. Assembles output:
   - **View:** `<div>{dl}{action_bar}</div>`
   - **Edit:** `<form action="{html_escape(action_url)}" method="{form_method}" class="space-y-4">{optional_method_spoof}{dl}{action_bar}</form>`
4. Method-spoofing block (`effective_method` derivation, `(form_method, needs_spoofing)` match, hidden `_method` input) is a verbatim copy of `render_form` at render.rs:971-1011 — mitigates T-147-01 (the `value="PUT|PATCH|DELETE"` literal is a fixed match-expression output, never touching caller-supplied strings).

### Insertion 2 — dispatch arm (render.rs:312)

Added `Component::DetailForm(props) => render_detail_form(props, data),` under the `// Container components.` banner, immediately after the `Component::Form` arm and before `Component::Modal`.

### Insertion 3 — plugin-walk container arm (render.rs:120)

Added:

```rust
Component::DetailForm(props) => {
    for field in &props.fields {
        collect_plugin_types_node(&field.input, types);
    }
}
```

positioned directly after the `Component::Form` arm at render.rs:115-119. **Not** placed in the leaf catch-all at render.rs:160-189 (Pitfall 2 of 147-RESEARCH.md): DetailForm is a container because `DetailField.input` is a `ComponentNode` that may itself contain plugins.

### Import-block extension (render.rs:13-25)

`DetailFormProps` and `EditMode` added to the non-test `use crate::component::{…}` block. `DetailField` is NOT added — the function iterates `for field in &props.fields`, type-inferring the element.

## Tailwind class list emitted (UI-SPEC §14.2 audit)

Every class below already appears in `render_form`, `render_description_list`, `render_input`, or `render_button`:

| Class                                           | Source                                |
|-------------------------------------------------|---------------------------------------|
| `grid grid-cols-1 gap-4`                        | `render_description_list` (L2427)     |
| `text-sm font-medium text-text-muted`           | `render_description_list` (L2431)     |
| `mt-1 text-sm text-text`                        | `render_description_list` (L2432)     |
| `flex gap-2 justify-end mt-6`                   | `render_form` submit-row pattern      |
| `inline-flex items-center justify-center rounded-md font-medium` | `render_button` base           |
| `transition-colors duration-150 motion-reduce:transition-none` | `render_button` base              |
| `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2` | `render_input` / `render_button` focus ring |
| `px-4 py-2 text-sm`                             | `render_button` default size          |
| `bg-primary text-primary-foreground hover:bg-primary/90` | `render_button` Primary variant |
| `border border-border bg-background text-text hover:bg-surface` | `render_button` Outline variant |
| `space-y-4`                                     | form wrapper spacing (already used)   |

**Zero new classes.** §14.2 audit: PASS.

## Option-A label rule compliance (UI-SPEC §9, §11, §14.4)

- Inner `<label></label>` remains empty (caller-supplied per §9 Option A — renderer does not mutate).
- Renderer wraps the rendered input inside `<span role="group" aria-label="{html_escape(field.label)}">…</span>` in Edit mode, so screen readers announce the field name even with an empty inner label.
- Wrapper lives inside `<dd>`; the `<dd>` opening tag and classes remain byte-identical to View mode — §5 structural-coherence invariant is preserved.

## Test-gating status

| Test                                                      | Expected | Gate          |
|-----------------------------------------------------------|----------|---------------|
| `render_detail_form_view_mode`                            | GREEN    | post 147-02   |
| `render_detail_form_edit_mode`                            | GREEN    | post 147-02   |
| `render_detail_form_scaffold_invariance`                  | GREEN    | post 147-02   |
| `render_detail_form_edit_method_spoofing_put`             | GREEN    | post 147-02   |
| `render_detail_form_edit_method_spoofing_patch`           | GREEN    | post 147-02   |
| `render_detail_form_edit_method_spoofing_delete`          | GREEN    | post 147-02   |
| `render_detail_form_edit_get_no_spoofing`                 | GREEN    | post 147-02   |
| `render_detail_form_view_shows_modifica_link`             | GREEN    | post 147-02   |
| `render_detail_form_edit_shows_salva_and_annulla`         | GREEN    | post 147-02   |
| `render_detail_form_view_xss_escapes_strings`             | GREEN    | post 147-02   |
| `render_detail_form_edit_xss_escapes_cancel_url`          | GREEN    | post 147-02   |
| `render_detail_form_custom_labels`                        | GREEN    | post 147-02   |
| `render_detail_form_view_action_bar_below_dl`             | GREEN    | post 147-02   |

**13 tests expected to turn GREEN once Plan 147-02's types converge.** Pre-merge compilation currently fails with exactly 3 expected errors: two unresolved imports (`DetailFormProps`, `EditMode`) and one missing enum variant (`Component::DetailForm`) — all owned by Plan 147-02.

## Acceptance-criteria grep checks (all PASS)

| Check                                                                 | Result    |
|-----------------------------------------------------------------------|-----------|
| `grep 'fn render_detail_form'`                                        | 1 match   |
| `grep 'Component::DetailForm(props) => render_detail_form(props, data)'` | 1 match |
| Container arm recurses `collect_plugin_types_node(&field.input, ...)` | 1 match   |
| DetailForm NOT in leaf catch-all                                      | 0 matches |
| Method-spoofing `("post", true)` block count                          | 4 (render_form + render_detail_form + 2 prior) |
| `html_escape(` calls inside `render_detail_form` body                 | 9         |
| `cargo fmt -p ferro-json-ui -- --check`                               | clean     |

## Deviations from Plan

### Auto-added (Rule 2 — accessibility requirement)

**1. [Rule 2 — Accessibility] aria-label wrapper inside Edit-mode `<dd>`**

- **Source:** UI-SPEC §11 + §14.4 ("Each Edit-mode `<input>` has both `<label></label>` (empty) and a non-empty `aria-label` matching the `<dt>` text") and the `<wave_1_green_expectation>` execution directive ("DetailForm emits aria-label derived from dt text on the rendered input (or wraps the rendered input with a labeling element) — follow the UI-SPEC §9 contract exactly").
- **Plan 03 text itself** (Task 1 Behavior §4) explicitly punted this to rustdoc-only: "THIS PLAN's minimum: the rustdoc on render_detail_form re-states the Option-A rule and points callers at `aria-label`. No mutation of caller props."
- **Resolution:** A `<span role="group" aria-label="{html_escape(field.label)}">` wrapper is inserted *inside* `<dd>` around the rendered input in Edit mode only. This satisfies §11/§14.4 without mutating caller props (the inner `<label></label>` stays empty; the caller's `InputProps` is untouched). The wrapper lives inside `<dd>`, so the §5 structural-coherence invariant (same `<dl>`, `<dt>`, `<dd>` opening tags across modes) is preserved.
- **Files modified:** ferro-json-ui/src/render.rs (Edit-mode `<dd>` branch of `render_detail_form`).
- **Commit:** 1daa6087

## Self-Check: PASSED

- [x] `fn render_detail_form` exists at render.rs:1039+
- [x] Dispatch arm exists at render.rs:312
- [x] Container arm in `collect_plugin_types_node` exists at render.rs:120
- [x] DetailForm is NOT in leaf catch-all
- [x] Method-spoofing block copied verbatim from `render_form`
- [x] ≥6 `html_escape` calls in function body (actual: 9)
- [x] Zero new Tailwind classes (§14.2)
- [x] `cargo fmt -p ferro-json-ui -- --check` clean
- [x] Remaining cargo-check errors limited to the 3 expected missing types from Plan 147-02
- [x] Commit `1daa6087` created with --no-verify
