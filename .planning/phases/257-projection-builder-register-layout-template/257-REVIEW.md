---
phase: 257-projection-builder-register-layout-template
reviewed: 2026-07-06T14:04:03Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - app/src/controllers/cassa.rs
  - app/src/routes.rs
  - app/src/tests/cassa_render.rs
  - app/src/tests/mod.rs
  - ferro-json-ui/assets/ferro-base.css
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/projection/builder.rs
  - ferro-json-ui/src/projection/error.rs
  - ferro-json-ui/src/projection/intent_layout.rs
  - ferro-json-ui/src/render/form.rs
  - ferro-json-ui/src/spec.rs
  - framework/src/lib.rs
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 257: Code Review Report (Re-review after 257-04)

**Reviewed:** 2026-07-06T14:04:03Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Re-review of the Register layout phase after the gap-closure plan 257-04 (commits `eef721b9`, `156150e0`) landed the Form fill height-chain: `FormProps.fill: Option<bool>` (component.rs), fill-aware class selection in `render_form` (form.rs), `fill: Some(true)` on the `sale_form` in `emit_register_root` (builder.rs), a regenerated `ferro-base.css`, and new assertions in `app/src/tests/cassa_render.rs`.

**All Critical and Warning findings from the previous review iteration are verified fixed in the current tree:**

- **CR-01 (empty tile grid):** the `/cassa` handler now nests rows as `{ "data": { "cassa": [...] } }` (cassa.rs:85), and the app test asserts a real product renders (`html.contains("Caffè")`, cassa_render.rs:69-71). The `$each` over `/data/cassa` resolves.
- **WR-01 (vacuous tests):** `catalog_each_template_populated_data` and `register_projection_populated_data_validates` now nest fixture data under `"data"` so the path genuinely resolves; the negative counterpart `catalog_each_template_path_not_array_rejected_at_build` asserts `SpecError::EachPathNotArray`; the register test re-validates through `Spec::from_json` (the layer that runs `validate_directives`).
- **WR-02 (fallback leak class):** the `emit_register_root` fallback now filters `f.readable && lookup_meaning(&f.meaning).display.is_some()` (builder.rs), with regression test `register_projection_fallback_excludes_sensitive_fields` confirming Sensitive fields are never bound into tile props.
- **WR-04 (schema gap):** `assemble_full_schema` now exposes `fill_viewport` and `design` at the root, hoists `DesignMeta` into `$defs`, and the drift guard `full_schema_root_exposes_all_spec_fields` pins every Spec root field.
- **WR-03 (docs/src coverage):** verified still absent from docs/src, but explicitly deferred to Phase 258 per locked decision D-19 (rustdoc coverage on the new public surface confirmed complete). Not re-raised.

**The newest 257-04 changes are sound:**

- `FormProps.fill` is additive (`Option<bool>` with `skip_serializing_if`), well-documented, and the input-mode builder path threads `fill: None` so existing Collect forms are unaffected.
- `render_form`'s fill branch (form.rs:98-102) emits full class literals (Tailwind `@source` scanner discipline respected) and the default branch is byte-identical to prior renders — both pinned by tests (`render_form_fill_true_emits_height_chain`, `render_form_default_class_is_byte_identical`).
- The regenerated `ferro-base.css` was verified to contain all fill-chain selectors: `.\[\&\>\*\]\:flex-1>*`, `.\[\&\>\*\]\:min-h-0>*`, `.min-h-0`, `.h-full`, `.flex-col`.
- `emit_register_root` sets `fill: Some(true)` on the `sale_form`, pinned by `register_projection_sale_form_carries_fill` and the app-level class-chain assertion in cassa_render.rs:77-80. The height chain is coherent: root Grid (fill) → sale_form (`flex flex-col h-full min-h-0`, sole child gets `flex-1 min-h-0`) → panes Grid (`h-full` resolves against a real height).
- Catalog Stage-3 template-props removal is schema-safe: element variant schemas only `require: ["type"]`, so removing `props` from the envelope copy of `$each` templates cannot fail the oneOf match; `.prop()` insert semantics correctly let the `$data` pointer objects override the placeholder `TileProps` values.
- The removed `cassa.rimuovi` route has no dangling references (handler deleted, no remaining consumers in app or library source).
- Project-agnostic discipline holds: library fixtures use neutral English names, Italian display copy is confined to app-land, `RegisterMissingAction` fails loud on action-less services.

One new Warning on the fill/max_width interaction, plus the two acknowledged Info carry-overs and one naming nit.

## Warnings

### WR-01: `fill: true` + `max_width` silently breaks the height chain — the same bug class 257-04 just closed

**File:** `ferro-json-ui/src/render/form.rs:98-102` and `140-144`
**Issue:** When a spec author sets both `fill: true` and `max_width: narrow|wide` on a Form, the max-width wrapper is applied unconditionally:

```rust
let html = match props.max_width.as_ref().unwrap_or(&FormMaxWidth::Default) {
    FormMaxWidth::Default => html,
    FormMaxWidth::Narrow => format!("<div class=\"max-w-2xl\">{html}</div>"),
    FormMaxWidth::Wide => format!("<div class=\"max-w-4xl\">{html}</div>"),
};
```

The wrapper `<div>` carries no `h-full`/`min-h-0`, so the form's `h-full` resolves against an auto-height parent and computes to auto — the entire fill chain dies silently, reproducing exactly the "footer off-viewport" failure mode that UAT caught and 257-04 fixed. The register builder never emits this combination (`max_width: None` in `emit_register_root`), but the component surface allows it, no lint rule covers it, and the failure produces no diagnostic. This is a latent authoring trap on a prop pair whose interaction is invisible until live geometry testing.
**Fix:** Make the wrapper fill-aware so the chain survives:

```rust
let wrapper_classes = if props.fill == Some(true) {
    ("max-w-2xl h-full min-h-0", "max-w-4xl h-full min-h-0")
} else {
    ("max-w-2xl", "max-w-4xl")
};
```

(or, if a width-constrained fill workspace is considered nonsensical, ignore `max_width` when `fill == Some(true)` and emit a `<!-- ferro-json-ui: max_width ignored in fill mode -->` diagnostic comment, matching the existing decode-failure diagnostic discipline). Either way, add a unit test pinning the chosen behavior.

## Info

### IN-01: `$each` template skip disables props validation even for literal (non-expression) props — carried over, acknowledged

**File:** `ferro-json-ui/src/catalog.rs:759-770`
**Issue:** Stage 2 skips per-element Props validation entirely when `el.each.is_some()`. A template element with an invalid *literal* prop (e.g. a mistyped enum value alongside the `$data` bindings) passes static validation and only surfaces at render time. The trade-off is documented in the code comment and was accepted in the previous iteration (out of fix scope).
**Fix (optional):** Validate a copy with expression-valued keys removed and `required` relaxed for the removed keys — catches literal-prop typos at build time while leaving data-bound props to the runtime resolver.

### IN-02: `register_template` slot list uses `"items"`, outside the declared 8-slot vocabulary — carried over, acknowledged

**File:** `ferro-json-ui/src/projection/intent_layout.rs:54`
**Issue:** `slots: vec!["items".into(), "actions".into()]` — `"items"` is not in the module-header vocabulary (`title, body, fields, actions, relationships, pagination, metadata, stats`). Documented as informational for the Register arm, but a non-vocabulary name invites drift if a future refactor starts dispatching Register slots.
**Fix:** Use existing vocabulary names (`"fields"`, `"actions"`) or add `"items"` to the ferro-theme slot vocabulary documentation.

### IN-03: `SpecBuilder.fill_viewport_` trailing-underscore field is inconsistent with sibling fields

**File:** `ferro-json-ui/src/spec.rs:362,373,401-404`
**Issue:** The builder field is named `fill_viewport_` to avoid clashing with the `fill_viewport(mut self, v: bool)` method — but Rust fields and methods live in separate namespaces, and the same struct already proves it: `title`, `layout`, and `data` fields coexist with identically-named methods. The underscore is unnecessary and breaks the struct's own naming pattern.
**Fix:** Rename the field to `fill_viewport` (private field; zero API impact).

---

_Reviewed: 2026-07-06T14:04:03Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
