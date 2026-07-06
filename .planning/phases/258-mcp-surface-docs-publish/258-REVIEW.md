---
phase: 258-mcp-surface-docs-publish
reviewed: 2026-07-06T17:34:48Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - ferro-mcp/src/tools/generation_context.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - docs/src/json-ui/components.md
  - docs/src/json-ui/layouts.md
  - docs/src/json-ui/spec-construction.md
findings:
  critical: 0
  warning: 5
  info: 8
  total: 13
status: issues_found
---

# Phase 258: Code Review Report

**Reviewed:** 2026-07-06T17:34:48Z
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Reviewed the Phase 258 MCP surface additions (`register_composition` guidance on `GenerationContext`, `BUILDER_API`/`RULE_COMPONENTS` extensions in `json_ui_catalog`) and the five-component + Register-layout + builder-API documentation, cross-checked against the authoritative sources in `ferro-json-ui` (`component.rs` props structs, `design/rules.rs`, `runtime/*.rs`, `projection/builder.rs`, `projection/intent_layout.rs`, `spec.rs`).

**Verified accurate (no findings):**
- Props tables for TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad, and Tile match `component.rs` exactly — names, types, optionality, and render defaults (`search_placeholder` "Search", `all_label` "All", `total_label` "Total", `NumpadMode` default `quantity`, stepper `min` 0 / `step` 1, grid `columns` 2).
- All 13 `REGISTER_DATA_ATTRIBUTES` entries name attributes that exist in the runtime sources, and the described semantics (empty `data-filter-tab` = All, space→hyphen token normalization, hidden-input `data-qty-input` emission, missing `data-unit-price` = 0 cents) match `runtime/filters.rs`, `runtime/selection.rs`, and `render/atoms.rs`.
- Lint rule ids and firing conditions in both the MCP guidance and `layouts.md` match `design/rules.rs`: `REGISTER_TRIGGER_TYPES` = TileGrid/SelectionPanel/Numpad; supported fill_viewport layouts are exactly `"app"`/`"dashboard"`.
- `register_template()` claims match `projection/intent_layout.rs` (Collect→Register override only; defaults untouched) and `projection/builder.rs` (emits `fill_viewport: true`, layout `"dashboard"`, Form common ancestor, `$each`-templated Tile in TileGrid, SelectionPanel with confirm slot). The `layouts.md` handler example mirrors the compiling `/cassa` reference (`app/src/controllers/cassa.rs`).
- Touch-target claims (≥44 px filter tabs, ≥56 px numpad keys) match `render/classes.rs`/`render/atoms.rs` constants.
- MCP output shape changes are purely additive (new `register_composition` key, extended `BUILDER_API` string, new `component_guidance` entries). No backward-compatibility break for 0.2.89 consumers.
- The `RULE_COMPONENTS` bidirectional drift guard in `json_ui_catalog.rs` is sound (mapped→registry, registry→mapped, component→builtin, all three directions enforced).

**Key concerns:** the new Rust example in `spec-construction.md` does not compile (wrong import path, private constructor); the `register_composition` drift-guard test only partially binds the guidance to the registries — one of its three checks is vacuous by construction and another covers 5 of 13 attributes; and `components.md` has two internal-consistency gaps (Button table missing `disable_on_submit`, overview table missing the five new components). Since 0.2.89 is already published, these feed a docs/tests patch, not a re-publish.

## Warnings

### WR-01: Builder-API doc example does not compile (wrong import path + private constructor)

**File:** `docs/src/json-ui/spec-construction.md:145-147` (also pre-existing instance at `:116-118`)
**Issue:** The Phase 258 "Builder API additions" example opens with `use ferro::json_ui::{Element, Spec, SpecBuilder};` and constructs via `SpecBuilder::new()`. Neither works:
1. `framework::json_ui` (framework/src/json_ui/mod.rs) re-exports none of these types — they are re-exported at the crate root only (`framework/src/lib.rs:85`). `ferro::json_ui::Spec` is an unresolved path.
2. `SpecBuilder::new()` is private (`fn new()` at `ferro-json-ui/src/spec.rs:369`). The public constructor is `Spec::builder()` — which is what `BUILDER_API` in json_ui_catalog correctly documents.

The pre-existing heterogeneous-construction example (line 116) has both defects too; the new example copied the broken pattern. For an agent-authoring-optimized framework, examples are copied verbatim, so this produces compile failures in generated code.
**Fix:**
```rust
use ferro::{Element, Spec, SpecBuilder};

let spec: Spec = Spec::builder()
    .title("Register")
    .layout("dashboard")
    .fill_viewport(true)
    // ... rest unchanged
```
Apply the same fix to the example at line 116. Alternatively, add `pub use ferro_json_ui::{Element, Spec, SpecBuilder, ...};` to `framework/src/json_ui/mod.rs` if the namespaced path is the intended public surface — but then `BUILDER_API` and other docs should agree on one canonical path.

### WR-02: Attribute drift guard covers only 5 of 13 REGISTER_DATA_ATTRIBUTES

**File:** `ferro-mcp/src/tools/generation_context.rs:594-606`
**Issue:** Check 3 of `register_composition_drift_guard` asserts a hardcoded list of 5 attribute names (`data-qty-input`, `data-filter-tokens`, `data-filter-text`, `data-numpad-target`, `data-disable-on-submit`) against `FERRO_RUNTIME_JS`. The published `REGISTER_DATA_ATTRIBUTES` array (lines 260-274) contains 13 entries; the other 8 (`data-filter-scope`, `data-filter-tab`, `data-filter-search`, `data-qty-inc`, `data-qty-dec`, `data-qty-display`, `data-unit-price`, `data-numpad-mode`) are unguarded. A runtime rename of any of those leaves the MCP guidance stale with green CI. All 13 currently do exist in the runtime (verified), but the guard does not enforce it. Each array entry is machine-parseable (attribute name is the token before the first `=` or space), so the guard can derive its inputs instead of duplicating them.
**Fix:**
```rust
// 3. EVERY published attribute appears in the assembled runtime bundle.
for entry in ctx.register_composition.data_attributes {
    let name = entry
        .split([' ', '='])
        .next()
        .expect("attribute entry is non-empty");
    assert!(
        ferro_json_ui::FERRO_RUNTIME_JS.contains(name),
        "runtime bundle missing `{name}` — register guidance is stale"
    );
}
```

### WR-03: Rule-id drift guard check is vacuous by construction

**File:** `ferro-mcp/src/tools/generation_context.rs:581-592`
**Issue:** Check 2 of `register_composition_drift_guard` iterates `ctx.register_composition.lint_rules` and asserts each `id` exists in `design::rules()`. But `lint_rules` is built in `execute()` (lines 342-350) by *filtering* `design_rules()` on `register_rule_ids` — every element trivially originates from the registry, so this assertion can never fail regardless of drift. The only real protection is the `lint_rules.len() == 4` assert in `test_generation_context_has_all_sections` (line 550), which lives in a different test and does not identify *which* id went missing. If a register rule is renamed in `rules.rs`, the vacuous loop passes and only the remote count assert trips.
**Fix:** Assert against the hardcoded source array instead, inside the drift guard:
```rust
// 2. Every id the guidance hardcodes exists in the rule registry, and is derived.
let expected = [
    "register-fill-viewport",
    "register-grid-fill",
    "register-selection-present",
    "fill-viewport-layout-unknown",
];
let derived: HashSet<&str> = ctx.register_composition.lint_rules.iter().map(|r| r.id).collect();
for id in expected {
    assert!(rule_ids.contains(id), "registry lost rule `{id}`");
    assert!(derived.contains(id), "guidance failed to derive rule `{id}`");
}
```
Consider extracting `register_rule_ids` to a module-level `const` shared by `execute()` and the test so there is one source.

### WR-04: Button props table omits `disable_on_submit`, which the register docs require

**File:** `docs/src/json-ui/components.md:1010-1018` (referenced from `:1499` and generation_context `form_state_contract`)
**Issue:** The SelectionPanel section instructs "Put the confirm `Button` (with `disable_on_submit: true`) in the `children` slot", and the MCP `form_state_contract` guidance repeats it — but the Button component's own props table (the canonical props reference) does not list `disable_on_submit`. The prop exists on `ButtonProps` (`ferro-json-ui/src/component.rs:334`), along with `form: Option<String>` (HTML5 `form` attribute, line 330) which is also undocumented. An agent reading the Button table to validate its confirm button will conclude the prop does not exist.
**Fix:** Add rows to the Button props table:
```markdown
| `form` | `string \| null` | HTML5 `form` attribute — lets a button outside its target `<form>` submit it by `id` |
| `disable_on_submit` | `boolean \| null` | Emits `data-disable-on-submit`; the runtime disables the button after first submit (double-submit guard). Pair with the confirm button in register compositions |
```

### WR-05: Component Overview table missing the five components documented in this phase

**File:** `docs/src/json-ui/components.md:25-36`
**Issue:** The overview table's Commerce row lists only `Tile`. TileGrid, SelectionPanel, FilterTabs, QuantityStepper, and Numpad each received full sections in this phase but were not added to the discovery table at the top of the same file. (Pre-existing gaps also exist — StreamText, DetailPage, MediaCardGrid, SegmentedControl, SidebarLayout — but the five register components are this phase's responsibility.)
**Fix:** Extend the Commerce row (or add a "Register / POS" row):
```markdown
| **Commerce / Register** | Tile, TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad |
```

## Info

### IN-01: "Four register-* lint rule ids" wording is inaccurate

**File:** `ferro-mcp/src/tools/generation_context.rs:116` (also test message at `:553`)
**Issue:** The `lint_rules` field doc says "(e) The four register-* lint rule ids" and the test asserts "must derive all four register-* rules", but `fill-viewport-layout-unknown` is not register-prefixed.
**Fix:** Reword to "the four register-composition lint rule ids (three `register-*` rules plus `fill-viewport-layout-unknown`)".

### IN-02: Component-name drift guard is a parallel list, not bound to the prose

**File:** `ferro-mcp/src/tools/generation_context.rs:563-579`
**Issue:** Check 1 validates a hardcoded name list against the builtin catalog but never verifies those names actually appear in `when_to_use`/`form_state_contract`. Renaming a component in the prose alone (e.g. SelectionPanel → CartPanel) passes the guard. The struct doc's claim "prose is drift-guarded" overstates the coverage.
**Fix:** For each name, additionally assert it appears in the guidance strings, e.g. `assert!(ctx.register_composition.when_to_use.contains(name) || ctx.register_composition.form_state_contract.contains(name), ...)` for the names each string mentions.

### IN-03: BUILDER_API shape listings not updated for the new surface

**File:** `ferro-mcp/src/tools/json_ui_catalog.rs:361, 370-379`
**Issue:** Three internal inconsistencies in the extended `BUILDER_API` string: (a) the `Spec { $schema, root, elements, title?, layout?, data? }` wire-shape listing omits `fill_viewport?` even though the same string now documents the setter and it is a spec-level JSON field agents hand-write; (b) the `Element { type, props, children, action?, visible? }` listing omits the `$each?` directive key (`spec.rs:148` — serialized as `$each`); (c) `.fill_viewport(bool)` is listed *after* `.build() -> Result<Spec, SpecError>` in the method chain, reading as if callable on the built result.
**Fix:** Move `.fill_viewport(bool)` above `.build()`, add `fill_viewport?` to the Spec listing and `$each?` to the Element listing.

### IN-04: Numpad mapped to `register-selection-present`, a rule it can never trigger

**File:** `ferro-mcp/src/tools/json_ui_catalog.rs:104-107`
**Issue:** `RULE_COMPONENTS` maps `register-selection-present` to `["Grid", "TileGrid", "Numpad", "SelectionPanel"]`, but the rule's check (`check_pos_cart_present`, `rules.rs:495`) fires only on TileGrid-present-without-SelectionPanel — Numpad plays no part. Numpad's `component_guidance` therefore carries a rule its presence cannot fire. The adjacent comment justifies only the `register-fill-viewport` additions.
**Fix:** Either drop `Numpad` from that mapping or add a comment stating it is intentional related-guidance (Numpad implies a register composition where the rule is relevant).

### IN-05: `register-selection-present` absent from layouts.md

**File:** `docs/src/json-ui/layouts.md:242-246`
**Issue:** The fill_viewport requirements table lists three of the four register lint rules; `register-selection-present` appears nowhere in layouts.md even though the Register Layout Template section describes the exact TileGrid+SelectionPanel pairing it guards. The MCP guidance names all four.
**Fix:** Add a row: `| TileGrid needs a paired SelectionPanel | register-selection-present | Spec contains a TileGrid but no SelectionPanel |` (noting it applies regardless of fill_viewport), or mention the rule in the Register Layout Template section.

### IN-06: layouts.md handler example import list omits `handler`

**File:** `docs/src/json-ui/layouts.md:259-263`
**Issue:** The example uses the `#[handler]` attribute but the `use ferro::{...}` list does not import `handler` (the reference `cassa.rs:2` does). Copied verbatim, the snippet fails to resolve the attribute macro.
**Fix:** Add `handler` to the import list.

### IN-07: Builder example spec trips `register-selection-present` when linted

**File:** `docs/src/json-ui/spec-construction.md:147-167`
**Issue:** The register-flavored builder example contains a TileGrid with no SelectionPanel and no Form ancestor. An agent copying it and following the docs' own advice to run `design_lint` gets a `register-selection-present` warning; the TileGrid also references `form_id: "sale_form"` with no such Form in the spec.
**Fix:** Either note that the snippet is an API illustration, not a complete register composition (with a pointer to `register_template()` / layouts.md for the full shape), or extend it with the Form + SelectionPanel.

### IN-08: Tile `categories` optionality not marked in props table

**File:** `docs/src/json-ui/components.md:1422`
**Issue:** `categories` is typed `string[]` with no optionality marker, but it is `#[serde(default)]` (`component.rs:1387`) — omitting it is valid and yields an untagged tile. Other optional rows in the same table carry `\| null` or a stated default.
**Fix:** Note the default: `string[]` → `string[]` with "(default: `[]` — untagged tile, always visible under filters)".

---

_Reviewed: 2026-07-06T17:34:48Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
