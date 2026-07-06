---
phase: 257-projection-builder-register-layout-template
reviewed: 2026-07-06T11:09:26Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - app/src/controllers/cassa.rs
  - app/src/routes.rs
  - app/src/tests/cassa_render.rs
  - app/src/tests/mod.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/projection/builder.rs
  - ferro-json-ui/src/projection/error.rs
  - ferro-json-ui/src/projection/intent_layout.rs
  - ferro-json-ui/src/spec.rs
  - framework/src/lib.rs
findings:
  critical: 1
  warning: 4
  info: 2
  total: 7
status: issues_found
---

# Phase 257: Code Review Report

**Reviewed:** 2026-07-06T11:09:26Z
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Reviewed the Register layout arm of the projection builder (`emit_register_root`), the `register_template()` Collect→Register theme override, the `ElementBuilder::each` / `SpecBuilder::fill_viewport` setters, the `Catalog::validate` `$each`-template guard, and the `/cassa` sample-app flip to a projection-derived spec.

The core library work is solid: `emit_register_root` is meaning-driven (no hardcoded field names in the mapping path), errors correctly on action-less services (`RegisterMissingAction`), binds tile props exclusively via `$data` pointer objects, keeps integer-cents on the `price_cents` contract, and stays project-agnostic (neutral English defaults, English fixture names in library tests, Italian copy confined to app-land). The catalog's Stage 2 continue + Stage 3 props-key removal for template elements is correctly scoped to the envelope copy and does not mutate the spec. Framework re-exports are correctly feature-gated.

However, the phase's flagship deliverable is silently broken: the `/cassa` handler passes its product rows at the wrong nesting level relative to the projection's `/data/{service}` path convention, so the `$each` template expands to **zero tiles** and the register renders an empty product grid. The phase's tests pass anyway because none of them asserts that a product actually renders — and two library tests that claim to cover the "each-path resolves to array" branch are vacuous for the same nesting reason.

## Critical Issues

### CR-01: /cassa register renders zero product tiles — handler data not nested under `data`, so `$each` over `/data/cassa` never resolves

**File:** `app/src/controllers/cassa.rs:83` (also `app/src/tests/cassa_render.rs:45`)
**Issue:** The projection builder emits `each.path = "/data/cassa"` and `TileGrid.data_path = "/data/cassa"` (`ferro-json-ui/src/projection/builder.rs:636,712`), following the documented projection convention where the handler provides `{ "data": { "<service>": [ ... ] } }` (docs/src/json-ui/data-binding.md §data_path Reference: "Handler provides `{ "data": { "staff": [ ... ] } }`"). The handler instead passes:

```rust
let data = serde_json::json!({ "cassa": cassa_products() });
```

The failure chain is deterministic:
1. `JsonUi::render` → `merge_data` does a shallow top-level merge → `spec.data = {"cassa": [...]}` (framework/src/json_ui/mod.rs:87-90).
2. `expand_directives` → `expand_each` resolves `each.path` with `resolve_path(spec.data, "/data/cassa")` (ferro-json-ui/src/resolve.rs:305-308); `resolve_path` walks literal segments `["data", "cassa"]` (ferro-json-ui/src/data.rs:19-45) — there is no `"data"` key → `None` → `rows = []`.
3. The `tile_tmpl` template is removed and replaced by zero clones (resolve.rs:335-339); `rewrite_parent_children` prunes the TileGrid's child list to empty.
4. `render_tile_grid` renders only `el.children` (ferro-json-ui/src/render/containers.rs:962-966) — the product grid is empty. The SelectionPanel, search input, and confirm button still render, which is exactly what the app test asserts, so the suite stays green while the page shows no products.

The old hand-written `cassa.json` used un-prefixed paths (`/prodotti`) matched to un-nested handler data, so this regressed when the page was flipped to projection-derived.

**Fix:**
```rust
// app/src/controllers/cassa.rs (handler)
let data = serde_json::json!({ "data": { "cassa": cassa_products() } });

// app/src/tests/cassa_render.rs (same nesting), plus a content assertion that
// would have caught this:
let data = json!({ "data": { "cassa": cassa_products() } });
let resp = JsonUi::render(&spec, &data).expect("render ok");
let html = resp.body();
assert!(html.contains("Caffè"), "product tiles must render from $each expansion");
```

## Warnings

### WR-01: "populated data" tests claim to cover the `$each` path-resolves-to-array branch but are vacuous (same nesting bug as CR-01)

**File:** `ferro-json-ui/src/catalog.rs:2439-2474` and `ferro-json-ui/src/projection/builder.rs:1705-1737`
**Issue:** Both tests state they cover "the validate_directives path-resolves-to-array branch (D-14)":
- `catalog_each_template_populated_data` builds a spec with `.data(json!({"items": [...]}))` while `each.path = "/data/items"`. In `validate_directives` the check is `if let Some(value) = resolve_path(&spec.data, &each.path)` (spec.rs:862-870) — the path does not resolve (`{"items": ...}` has no `"data"` key), so the `EachPathNotArray` branch silently skips and the test passes vacuously.
- `register_projection_populated_data_validates` assigns `spec.data = json!({"shop": [...]})` **after** `build()` and then calls `cat.validate(&spec)` — but `Catalog::validate` never runs `validate_directives` at all (that runs only inside `Spec::build`/`Spec::from_json` via `validate_structure`, spec.rs:741-752). The stated branch cannot be exercised on this path, resolved or not.

These tests institutionalize the same `/data`-nesting confusion that produced CR-01.

**Fix:** Nest the fixture data under `"data"` (`json!({"data": {"items": [...]}})`) so the path actually resolves, and exercise the branch through a build (or `Spec::from_json`) with the data attached. Add the negative case:
```rust
let err = Spec::builder()
    .data(json!({"data": {"items": {"not": "an array"}}}))
    .element("tile_tmpl", Element::new("Tile").each("/data/items", "p"))
    .build()
    .unwrap_err();
assert!(matches!(err, SpecError::EachPathNotArray { .. }));
```

### WR-02: `emit_register_root` fallback field selection contradicts the documented T-257-03 exclusion invariant

**File:** `ferro-json-ui/src/projection/builder.rs:624-634`
**Issue:** The comment claims "Sensitive/ForeignKey/system meanings are structurally excluded because field_name_by only matches Identifier / EntityName / Money (T-257-03)" — but the fallback path is not filtered at all:

```rust
let fallback = service
    .fields
    .first()
    .map(|f| f.name.clone())
    .unwrap_or_else(|| "id".to_string());
```

`fallback` ignores both `readable` and meaning. For a service with no Identifier/EntityName/Money field whose first declared field is e.g. `Sensitive` or `readable = false`, `id_field`/`money_field` bind that field name into tile props as `{"$data": "/p/<name>"}`, and `$each` expansion inlines the row value into rendered tile attributes — the exact leak class the sibling emitters guard against with `lookup_meaning(...).column/display.is_some()`.

**Fix:** Filter the fallback by readability and display eligibility, mirroring the other emitters:
```rust
let fallback = service
    .fields
    .iter()
    .find(|f| f.readable && lookup_meaning(&f.meaning).display.is_some())
    .map(|f| f.name.clone())
    .unwrap_or_else(|| "id".to_string());
```
(Alternatively: return a `ProjectionError` when none of the three meanings is present — a register without a name/price binding is arguably broken by construction, like the no-actions case.)

### WR-03: New public API surface undocumented in docs/src/

**File:** `ferro-json-ui/src/spec.rs:401-404,525-531`, `ferro-json-ui/src/projection/intent_layout.rs:50-66`, `framework/src/lib.rs:270`
**Issue:** `register_template()` (now a framework re-export), `ElementBuilder::each(path, as_)`, and `SpecBuilder::fill_viewport(bool)` have no coverage anywhere under `docs/src/` (verified by grep: `register_template` appears nowhere in docs/; `.each(`/`fill_viewport` appear in neither spec-construction.md, expressions.md, nor layouts.md — only the design-system patterns.md lint-rule examples mention `fill_viewport` as a JSON key). Project CLAUDE.md requires "Update documentation in docs/src/ (required)" for every user-facing feature.
**Fix:** Add to `docs/src/json-ui/spec-construction.md`: builder-side `$each` (`.each(...)` + `$data` row-scoped props) and `.fill_viewport(true)` with the dashboard/app layout + root `fill:true` preconditions; document `register_template()` (Collect→Register override, `RegisterMissingAction` contract, the `price_cents`/`field` per-row data contract, and the `{"data": {...}}` handler payload shape).

### WR-04: `fill_viewport` (and `design`) missing from the assembled full Spec JSON Schema

**File:** `ferro-json-ui/src/catalog.rs:571-588`
**Issue:** `assemble_full_schema` declares root properties `$schema`, `root`, `elements`, `title`, `layout`, `data` — but not `fill_viewport` (this phase's root flag) or `design`. Validation only passes because the root object does not set `additionalProperties: false`. The consequence is a contract gap, not a validation failure: this schema is advertised as "the full Spec shape (D-13)" and is consumed by agents via the MCP `json_ui_schema` surface, so a spec-authoring agent cannot discover `fill_viewport` from the schema, and any schema-strict external consumer would reject valid register specs. MCP surface accuracy is held to the same quality bar as the Rust API per project instructions. (The `design` omission is pre-existing; `fill_viewport` is new in this phase family.)
**Fix:** Add to the root `properties` map in `assemble_full_schema`:
```rust
"fill_viewport": { "type": "boolean", "default": false },
"design": { "$ref": "#/$defs/DesignMeta" }  // hoist schema_for!(DesignMeta) into shared_defs
```

## Info

### IN-01: `$each` template skip disables props validation even for literal (non-expression) props

**File:** `ferro-json-ui/src/catalog.rs:753-763`
**Issue:** Stage 2 skips per-element Props validation entirely when `el.each.is_some()`. This is broader than needed: a template element with an invalid *literal* prop (e.g. a mistyped enum value alongside the `$data` bindings) passes static validation and only surfaces at render time — as a decode-failure HTML comment, or via the render-time re-validation in `JsonUi::resolve` which logs at error level but does not fail. The trade-off is documented in the comment; acceptable for now.
**Fix (optional):** Instead of skipping, validate a copy with expression-valued keys removed and `required` checks relaxed for the removed keys — catches literal-prop typos at build time while leaving data-bound props to the runtime resolver.

### IN-02: `register_template` slot list uses a name outside the declared 8-slot vocabulary

**File:** `ferro-json-ui/src/projection/intent_layout.rs:54`
**Issue:** `slots: vec!["items".into(), "actions".into()]` — `"items"` is not part of the 8-slot vocabulary this module's header declares (`title, body, fields, actions, relationships, pagination, metadata, stats`). The doc comment correctly notes the list is informational for the Register arm, but a non-vocabulary slot name invites drift if a future refactor starts dispatching Register slots.
**Fix:** Use existing vocabulary names (`"fields"`, `"actions"`) or add `"items"` to the ferro-theme slot vocabulary documentation.

---

_Reviewed: 2026-07-06T11:09:26Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
