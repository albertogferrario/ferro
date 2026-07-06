# Phase 237: ActionGroup Component + DropdownMenu Replacement — Research

**Researched:** 2026-06-22
**Domain:** ferro-json-ui component addition + removal + version bump
**Confidence:** HIGH — all claims verified against current source at workspace 0.2.72

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Partition / layout rules**
- D-01: `destructive: true` items are **always** forced into the overflow kebab and rendered **last**, regardless of input order. They do not count toward `max_inline`.
- D-02: `max_inline` (default `2`) caps the number of non-destructive inline buttons. Non-destructive items beyond the cap overflow into the kebab in input order.
- D-03: The overflow kebab is **hidden entirely** when nothing overflows. No empty kebab glyph.
- D-04: "Primary first" is expressed by **input order** — no separate `primary: bool` flag.

**Prop shape**
- D-05: `ActionItem` fields: `label`, `action: Action`, `destructive: bool` (default false), `variant: Option<ButtonVariant>`, `icon: Option<String>`, `visible_if: Option<String>`. Same `visible_if` fail-closed semantics as `DropdownMenuAction`.
- D-06: `ActionGroupProps` fields: `items: Vec<ActionItem>` (or `{"$data":"/path"}` binding), `menu_id: String` (required), `max_inline: Option<u8>` (default 2), `overflow_label: Option<String>` (default `"Azioni"`), `row_key: Option<String>`. All optionals `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- D-07: Both structs derive `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`.
- D-08: `items` accepts a literal array **or** `{"$data":"/path"}` binding with `{row_key}` substitution and `visible_if` row gates.

**DropdownMenu retention boundary**
- D-09: Remove `DropdownMenu` from the **public** surface only: `BUILTIN_TYPES`, `BUILTIN_SPECS`, `lib.rs` export, dispatch arm, and catalog. No consumer authors a `DropdownMenu` spec after this phase.
- D-10: **Keep** `render_dropdown_menu` / `render_menu_item` as `pub(crate)` internal render helpers.
- D-11: `DropdownMenuAction` is **retained** as the internal row-action carrier. `DropdownMenuProps` (the public props struct) is removed with the public component.

**Migration scope**
- D-12: `emit_actions_placeholder` emits an `ActionGroup` element instead of `DropdownMenu`. Its unit test updated to decode `ActionGroupProps`.
- D-13: Migrate all ferro-internal / example / test specs authoring `DropdownMenu` to `ActionGroup`. Migrate json-ui docs.

**Builtin-count handling**
- D-14: One-for-one swap: +ActionGroup, −public DropdownMenu → `BUILTIN_TYPES.len()` **stays 47**.

**Form-wrapping non-GET inline actions**
- D-15: A non-GET inline action auto-wraps in `<form>` (method POST, CSRF handled by JS via meta tag). Reuse the existing `render_menu_item` form-rendering code (`render/atoms.rs:1073`). GET action renders as a plain link/button.

### Claude's Discretion
- Exact placement of `render_action_group` (seed suggests `render/containers.rs`; planner confirms).
- Whether `ActionGroup` is registered in the containers section vs atoms section of `BUILTIN_TYPES`.
- Internal helper signatures / how `render_action_group` shares the kebab helper.
- Whether `DropdownMenuAction` gets a type alias to a future `ActionItem` (cosmetic).

### Deferred Ideas (OUT OF SCOPE)
- Renaming `DropdownMenuAction` → `ActionItem` across DataTable/Kanban internals (cascades widely, no behavioral gain; revisit in a future phase).
- New action semantics (async/optimistic, new confirm-dialog variants, new `Action` kinds).
- gestiscilo consumer adoption of `ActionGroup` — separate consumer-repo phase, blocked on published `0.2.73`.
</user_constraints>

---

## Summary

Phase 237 adds `ActionGroup` as the sole public action primitive in ferro-json-ui and removes `DropdownMenu` from the public surface. The swap is one-for-one: the builtin count stays at 47. All existing kebab rendering machinery (`render_menu_item`, `render_dropdown_menu`, `render_inline_dropdown`) is retained as `pub(crate)` helpers that `render_action_group` calls for its overflow path.

All seed line-number claims have been verified against the current source at workspace 0.2.72. The most important correction: the workspace version is already 0.2.72 (it was bumped in the ferro-payments patch cycle), so this phase must target **0.2.72 → 0.2.73**, not 0.2.71 → 0.2.72 as the seed stated.

The `{"$data": "/path"}` binding for `items` requires no special machinery in `ActionGroupProps` — `resolve_expressions` (ferro-json-ui/src/expression.rs:35) resolves `$data` references in all props fields before the render step, replacing the binding with the actual array. The `Vec<ActionItem>` type handles both literal and data-bound cases transparently.

**Primary recommendation:** Follow the 8-touchpoint registration surface exactly as documented, move in the order: props struct → lib.rs export → BUILTIN_TYPES → dispatch → render impl → BUILTIN_SPECS → drift guards → schema test. Then remove public DropdownMenu in reverse. Then migrate projection builder (D-12), tests (DropdownMenu unit tests in atoms.rs), and docs. Version-bump last, publish as an operator-gated step.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ActionGroup prop struct + ActionItem | ferro-json-ui (component.rs) | — | All component props live in component.rs |
| ActionGroup render (inline buttons + overflow kebab) | ferro-json-ui (render/containers.rs) | render/atoms.rs (reused helpers) | Resolves slots like other container components; reuses kebab atom helpers |
| DropdownMenu public surface removal | ferro-json-ui (6 removal sites) | ferro-mcp (name-list update) | Same 8-touchpoint registration surface, in reverse |
| Projection codegen migration | ferro-json-ui (projection/builder.rs) | — | emit_actions_placeholder is the only generator for the action slot |
| ferro-mcp count + name-list mirror | ferro-mcp (json_ui_catalog.rs) | — | Cross-crate mirror; BUILTIN_TYPES is pub(crate), so mcp mirrors it separately |
| Docs migration | docs/src/json-ui/ | docs/src/features/ | User-facing doc files, no code impact |
| Version bump + publish | Cargo.toml (workspace) | publish.yml (Wave 1A) | Operator-gated step; ferro-json-ui is in Wave 1A |

---

## Standard Stack

No new library dependencies introduced. All work uses existing ferro-json-ui internals.

### Core Tools (already in workspace)
| Tool | Purpose | Location |
|------|---------|----------|
| `schemars::JsonSchema` | Schema derive for component props | already in ferro-json-ui Cargo.toml |
| `serde + serde_json` | Serialize/Deserialize for props | already in ferro-json-ui Cargo.toml |
| `strum::AsRefStr` | Optional — NOT needed for ActionGroup (no string enum) | — |

**Installation:** No new dependencies.

---

## Architecture Patterns

### System Architecture Diagram

```
Consumer Spec JSON
       │
       ▼
resolve_expressions()     ← replaces {"$data":"/path"} with actual array in-place
       │
       ▼
render_element("ActionGroup")
       │
       ├─→ partition items: inline (non-destructive, ≤ max_inline) | overflow + destructive
       │
       ├─→ for each inline GET item  → <a href="..."><button>...</button></a>
       │
       ├─→ for each inline non-GET   → render_menu_item()-style <form method="post">
       │                                (reuse atoms::render_menu_item's form-wrap logic)
       │
       └─→ if overflow exists:
               render_dropdown_menu-derived overflow popover
               (reuse atoms::render_dropdown_menu's kebab trigger + popover panel)
               └─→ for each overflow item → render_menu_item(item, ...)
```

### Recommended Project Structure

No new files required. All additions go inside existing files:

```
ferro-json-ui/src/
├── component.rs          # ADD ActionItem + ActionGroupProps (after ButtonGroupProps ~:936)
├── lib.rs                # SWAP pub use: remove DropdownMenuProps, add ActionGroupProps + ActionItem
├── render/
│   ├── mod.rs            # SWAP BUILTIN_TYPES entry; SWAP dispatch arm
│   ├── containers.rs     # ADD render_action_group(); BUILTIN_SPECS entry moves here conceptually
│   └── atoms.rs          # keep render_menu_item + render_dropdown_menu as pub(crate)
│                         # DELETE: dropdown_menu_emits_actions + dropdown_menu_get_action_renders_anchor tests
│                         #         (replaced by render_action_group tests)
├── catalog.rs            # SWAP BUILTIN_SPECS entry; update count comment; keep count = 47
└── projection/
    └── builder.rs        # SWAP emit_actions_placeholder to emit ActionGroup + test update
docs/src/
├── json-ui/components.md # REPLACE DropdownMenu section with ActionGroup section
└── features/projections.md # UPDATE action route table reference
ferro-mcp/src/tools/
└── json_ui_catalog.rs    # SWAP "DropdownMenu" → "ActionGroup" in expected[] array
                          # FIX: also add "SegmentedControl" + "SidebarLayout" (pre-existing gap: 45 entries vs 47 count)
```

### Pattern 1: ActionItem Partition Logic

The render function must partition items before emitting HTML. The ordering is enforced structurally:

```rust
// Source: derived from existing atoms.rs render_dropdown_menu pattern
fn partition_items(items: &[ActionItem], max_inline: u8) -> (Vec<&ActionItem>, Vec<&ActionItem>) {
    let (normal, destructive): (Vec<_>, Vec<_>) = items.iter().partition(|i| !i.destructive);
    let max = max_inline as usize;
    let inline_items: Vec<_> = normal.iter().take(max).copied().collect();
    let mut overflow: Vec<_> = normal.iter().skip(max).copied().collect();
    overflow.extend(destructive.iter().copied()); // destructive items rendered LAST
    (inline_items, overflow)
}
```

### Pattern 2: Non-GET Inline Button Form-Wrapping

D-15 requires inline non-GET buttons to wrap in `<form>`. The existing `render_menu_item` at `atoms.rs:1073` already implements this pattern (non-GET → `<form method="post"><input _method><button type="submit">…</form>`). CSRF is handled by the Inertia JS layer via `<meta name="csrf-token">` — no hidden `_token` input required in the HTML.

**Important clarification on D-15 wording:** The CONTEXT says "reuse the existing Button-in-form path (`atoms.rs:203` area)." The `render_button` at line 203 does NOT wrap non-GET in `<form>` — it returns a bare `<button>` for non-GET and relies on "client runtime." The actual form-wrapping code to reuse is `render_menu_item` at line 1073. The planner should treat `render_menu_item` as the form-wrapping template for inline non-GET ActionGroup items, not `render_button_inner`.

```rust
// Source: ferro-json-ui/src/render/atoms.rs:1127-1151 (the non-GET branch of render_menu_item)
// Pattern to reuse in render_action_group for inline non-GET items:
let mut html = format!("<form action=\"{}\" method=\"post\">", html_escape(url));
if let Some(m) = method_spoof {
    html.push_str(&format!("<input type=\"hidden\" name=\"_method\" value=\"{m}\">"));
}
html.push_str(&format!("<button type=\"submit\" class=\"{btn_classes}\">{label}</button>"));
html.push_str("</form>");
```

### Pattern 3: $data Binding for items

`ActionGroupProps.items: Vec<ActionItem>` handles the `{"$data": "/path"}` binding transparently. `resolve_expressions()` in `expression.rs:35` walks all element props and replaces any `{"$data": "/pointer"}` object with the resolved JSON value **before** render dispatch. The `render_action_group` function receives an already-resolved `Vec<ActionItem>` — no binding logic needed in the renderer.

```rust
// Source: ferro-json-ui/src/expression.rs:42-66
// This runs before render_element — by the time ActionGroup's props are decoded,
// items is already a JSON array (resolved from spec.data), not a {"$data": ...} object.
fn resolve_value(val: &mut Value, data: &Value) {
    match val {
        Value::Object(map) => {
            if let Some(path) = is_data_expr(map) {
                *val = resolve_path(data, &path).cloned().unwrap_or(Value::Null);
                // Single-pass: do NOT recurse into the resolved value.
            } else { /* recurse into non-$data objects */ }
        }
        // ...
    }
}
```

### Pattern 4: Overflow Kebab Reuse

`render_dropdown_menu` at `atoms.rs:1154` and `render_menu_item` at `atoms.rs:1073` are the existing kebab building blocks. `render_action_group` should NOT re-implement the popover trigger SVG, popover anchoring, or destructive styling — it calls these helpers directly. Both are already `pub(crate)`, so no visibility change is needed.

`render_dropdown_menu`'s current signature: `fn render_dropdown_menu(el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String` — this takes an `Element`. The new overflow helper in `render_action_group` will need to call the lower-level building blocks directly (popover button + panel loop calling `render_menu_item`), rather than delegating to `render_dropdown_menu` which expects an `Element`. The planner should draft a thin private `render_overflow_kebab(menu_id, overflow_label, items)` that embeds the pattern from lines 1164–1193 of atoms.rs.

### Anti-Patterns to Avoid

- **Re-implementing the kebab popover:** The SVG, popover trigger, and popover panel HTML in `atoms.rs:1164–1193` is the single source of truth. Do not duplicate it in `render_action_group`.
- **Adding CSRF hidden input to forms:** The framework's CSRF relies on the JS meta-tag approach, not hidden fields in server-rendered forms. Adding one would be redundant and inconsistent with existing form rendering (`render_menu_item` does not add one either).
- **Auto-generating `menu_id`:** D-06 requires `menu_id` to be a required caller-supplied field, mirroring DropdownMenu. This prevents duplicate IDs when multiple ActionGroups appear on one page.
- **Counting destructive items toward `max_inline`:** D-01 is absolute: destructive items are always in the kebab, regardless of count.
- **Using `render_button` for non-GET inline items:** `render_button` at line 203 does NOT emit a `<form>` for non-GET actions — it emits a bare `<button>` relying on a JS runtime. Use the `render_menu_item` form-wrap pattern instead.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Kebab popover HTML | Custom popover | `render_menu_item` + `render_dropdown_menu` atoms (`atoms.rs:1073,1154`) | Single source of truth for popover trigger SVG, anchoring, destructive CSS, confirm dialog attrs |
| `visible_if` row gate | Custom row-visibility logic | `action_visible_for_row` pattern in `data.rs:445` | Fail-closed semantics are already specified and tested; copying the pattern prevents drift |
| `{row_key}` substitution | Custom URL interpolation | `template_actions` in `data.rs:467` | DataTable/Kanban already use this; ActionGroup's row context uses the same path |
| `$data` binding for items | Custom binding resolver | `resolve_expressions()` at `expression.rs:35` | Already runs before render dispatch; ActionGroupProps.items receives a resolved array |
| Non-GET form wrapping | Custom `<form>` emitter | `render_menu_item`'s non-GET branch (`atoms.rs:1127`) | Exact same HTML pattern (method spoof, confirm attrs, CSS class) — extract or inline-copy |

---

## Complete Registration Surface (Verified)

This is the full touch-point inventory for the swap, with verified file:line references:

### ADD ActionGroup (8 sites):

| # | Site | File | Current Line | Action |
|---|------|------|-------------|--------|
| 1 | Props structs `ActionItem` + `ActionGroupProps` | `ferro-json-ui/src/component.rs` | ~after :932 (ButtonGroupProps) | INSERT two new structs |
| 2 | Public export | `ferro-json-ui/src/lib.rs` | :49 (`pub use component::{…}`) | ADD `ActionGroupProps, ActionItem` |
| 3 | `BUILTIN_TYPES` entry | `ferro-json-ui/src/render/mod.rs` | :43 (array), containers section ~:70 | ADD `"ActionGroup"` in containers section |
| 4 | Dispatch arm | `ferro-json-ui/src/render/mod.rs` | :176–230 (match block) | ADD `"ActionGroup" => containers::render_action_group(…)` |
| 5 | Render impl | `ferro-json-ui/src/render/containers.rs` | ~:946 (after ButtonGroup) | ADD `pub(crate) fn render_action_group(…)` |
| 6 | `BUILTIN_SPECS` entry | `ferro-json-ui/src/catalog.rs` | :124 (array) | ADD tuple at same ordinal as BUILTIN_TYPES position |
| 7a | Runtime drift guard | `ferro-json-ui/src/catalog.rs` | :576 | Passes automatically (relational check) |
| 7b | Count drift guard comment + assertion | `ferro-json-ui/src/catalog.rs` | :1093 (test `builtin_types_count_drift_guard`) | UPDATE history comment to record swap; assertion stays `47` |
| 8 | Schema-nonempty test | `ferro-json-ui/src/component.rs` | :1509–1515 area | ADD `assert_schema_nonempty_object::<ActionGroupProps>(…)` + `assert_schema_nonempty_object::<ActionItem>(…)` |

### REMOVE public DropdownMenu (5 sites):

| # | Site | File | Current Line | Action |
|---|------|------|-------------|--------|
| R1 | Public export | `ferro-json-ui/src/lib.rs` | :54 | REMOVE `DropdownMenuProps` from the export list |
| R2 | `BUILTIN_TYPES` entry | `ferro-json-ui/src/render/mod.rs` | :64 | REMOVE `"DropdownMenu"` |
| R3 | Dispatch arm | `ferro-json-ui/src/render/mod.rs` | :197 | REMOVE `"DropdownMenu" => atoms::render_dropdown_menu(…)` |
| R4 | `BUILTIN_SPECS` entry | `ferro-json-ui/src/catalog.rs` | :240–244 | REMOVE the `("DropdownMenu", …)` tuple |
| R5 | `DropdownMenuProps` struct | `ferro-json-ui/src/component.rs` | :1074–1082 | DELETE struct (keep `DropdownMenuAction` at :1059) |

### KEEP (internal, no change required):

| Item | File | Lines | Reason |
|------|------|-------|--------|
| `render_menu_item` | `render/atoms.rs` | :1073 | Internal helper reused by ActionGroup overflow + DataTable/Kanban |
| `render_dropdown_menu` | `render/atoms.rs` | :1154 | Internal — kept for reference pattern; may become dead code after this phase (DataTable uses `render_inline_dropdown`, not `render_dropdown_menu`). Planner should note this. |
| `render_inline_dropdown` | `render/data.rs` | :520 | DataTable/Kanban per-row kebab — unchanged |
| `DropdownMenuAction` | `component.rs` | :1059 | Still used by `DataTableProps.row_actions` (:1091), `KanbanBoardProps.row_actions` (:1194), `MediaCardGridProps.row_actions` (:1135) |
| `action_visible_for_row` | `render/data.rs` | :445 | Internal — unchanged |
| `template_actions` | `render/data.rs` | :467 | Internal — unchanged |

### MIGRATE (projection + tests + docs):

| Item | File | Lines | Change |
|------|------|-------|--------|
| `emit_actions_placeholder` | `projection/builder.rs` | :672–699 | Emit `"ActionGroup"` + `ActionGroupProps` instead of `"DropdownMenu"` + `DropdownMenuProps` |
| Projection builder test | `projection/builder.rs` | :1220–1237 | Update to decode `ActionGroupProps`; change `"DropdownMenu must be emitted"` comment |
| Builder import | `projection/builder.rs` | :29 | SWAP `DropdownMenuProps` → `ActionGroupProps` in the import list |
| Unit tests | `render/atoms.rs` | :2018–2064 | REMOVE `dropdown_menu_emits_actions` + `dropdown_menu_get_action_renders_anchor` tests |
| Catalog import | `catalog.rs` | :32 | SWAP `DropdownMenuProps` → `ActionGroupProps` |
| Catalog DataTable description | `catalog.rs` | :404 | UPDATE "with per-row DropdownMenu" → "with per-row dropdown" or similar |
| Docs: component table | `docs/src/json-ui/components.md` | :29 | REPLACE `DropdownMenu` → `ActionGroup` in Forms category |
| Docs: DropdownMenu section | `docs/src/json-ui/components.md` | :985–1014 | REPLACE with `ActionGroup` section documenting new props |
| Docs: projections table | `docs/src/features/projections.md` | :504 | UPDATE "DropdownMenu item" → "ActionGroup item" |
| Docs: expressions.md | `docs/src/json-ui/expressions.md` | :156 | UPDATE incidental mention of DropdownMenu as a sibling example |
| ferro-mcp name list | `ferro-mcp/src/tools/json_ui_catalog.rs` | :300–346 | SWAP `"DropdownMenu"` → `"ActionGroup"` in `expected[]` |
| schema-nonempty tests | `component.rs` | :1509–1515 | REPLACE DropdownMenuProps test with ActionGroupProps; keep DropdownMenuAction test (struct retained) |

### CRITICAL PRE-EXISTING BUG (not introduced by this phase, but must be fixed as part of this change):

The `ferro-mcp` `test_all_components_present` at `json_ui_catalog.rs:300` asserts count = 47 but its `expected[]` array contains only **45 entries** — `"SegmentedControl"` and `"SidebarLayout"` are missing (added in 0.2.69 but never added to this array). The count assertion would pass (it checks `catalog.components.len()`) but the `names.contains(name)` loop would miss these. This array **must** gain `"SegmentedControl"` and `"SidebarLayout"` in addition to the `DropdownMenu` → `ActionGroup` swap. This is a 45→47 correction of the name-check list (count stays 47).

---

## Version Bump Correction

**The seed says 0.2.71 → 0.2.72. This is WRONG for the current state.**

The workspace `Cargo.toml` is already at `version = "0.2.72"` (bumped during the ferro-payments patch cycle: 0.2.72 shipped ferro-payments 0.1.2 + ferro-stripe 0.9.2 on 2026-06-21). The CONTEXT.md was drafted before those patches were published.

**This phase must bump: 0.2.72 → 0.2.73.**

- `Cargo.toml` workspace version line: `:46`
- ferro-json-ui is in **Wave 1A** of `.github/workflows/publish.yml` (line :211)
- Publish is operator-gated per project convention — do NOT plan an automated publish step. The plan should end with a Wave N "Operator-gated publish" task with the CLI commands, not an automated execution.

---

## Common Pitfalls

### Pitfall 1: Touching BUILTIN_TYPES / BUILTIN_SPECS Out of Lockstep
**What goes wrong:** The runtime drift guard at `catalog.rs:576` panics if `BUILTIN_SPECS.len() != BUILTIN_TYPES.len()`, breaking `Catalog::build()` which is called at server startup and in tests.
**Why it happens:** Adding the BUILTIN_TYPES entry without the BUILTIN_SPECS entry, or vice versa.
**How to avoid:** Always update both in the same commit. The `builtin_specs_len_matches_dispatch` test at `catalog.rs:1104` will fail immediately if they diverge.
**Warning signs:** `CatalogError::BuildFailed` in test output.

### Pitfall 2: Forgetting the ferro-mcp Name List (and its Pre-Existing 45-vs-47 Bug)
**What goes wrong:** The `test_all_components_present` test in `ferro-mcp/src/tools/json_ui_catalog.rs:286` checks a hardcoded `expected[]` name array. It currently has 45 entries (missing SegmentedControl and SidebarLayout); the count assertion at :294 still passes because it checks catalog length, not the expected array length.
**Why it happens:** `BUILTIN_TYPES` is `pub(crate)` in ferro-json-ui, so the mcp test can't reference it directly. The manual mirror drifted after 0.2.69.
**How to avoid:** Fix the expected array to 47 entries (add SegmentedControl, SidebarLayout, ActionGroup; remove DropdownMenu). The count stays 47.
**Warning signs:** The `names.contains(name)` loop is currently not testing all 47 names — the test passes silently but is incomplete.

### Pitfall 3: Removing `DropdownMenuAction` (the struct, not just the props)
**What goes wrong:** `DataTableProps.row_actions`, `KanbanBoardProps.row_actions`, and `MediaCardGridProps.row_actions` all use `Option<Vec<DropdownMenuAction>>`. Removing `DropdownMenuAction` breaks compilation for all three.
**Why it happens:** Conflating "remove DropdownMenu public component" with "remove all DropdownMenu types."
**How to avoid:** D-11 is explicit: only `DropdownMenuProps` is removed. `DropdownMenuAction` stays. The `lib.rs` export must keep `DropdownMenuAction`.
**Warning signs:** Compile errors in `component.rs`, `render/data.rs`, `projection/builder.rs` referencing `DropdownMenuAction`.

### Pitfall 4: Treating `render_button` as the Form-Wrapping Source
**What goes wrong:** D-15 says "reuse the existing Button-in-form path (`atoms.rs:203` area)" but `render_button` at line 203 does NOT wrap non-GET in `<form>` — it returns a bare `<button>` and says "client runtime reads data-* attributes added elsewhere."
**Why it happens:** Misreading the CONTEXT D-15 reference.
**How to avoid:** The form-wrapping template is `render_menu_item` at `atoms.rs:1073` (lines 1127–1151 for the non-GET branch). Extract the form-emit logic from there, or inline it in `render_action_group`.
**Warning signs:** Inline non-GET buttons render without `<form>` wrapper → POST submissions fail (no form to submit).

### Pitfall 5: Forgetting the catalog.rs `DropdownMenuProps` Import
**What goes wrong:** `catalog.rs` line :32 imports `DropdownMenuProps` for the `BUILTIN_SPECS` schema function. Removing the BUILTIN_SPECS entry without removing the import causes an "unused import" warning, which is a compile error under `-D warnings`.
**How to avoid:** When removing the `DropdownMenuProps` BUILTIN_SPECS entry, remove it from the `catalog.rs` import block at line :32 simultaneously.

### Pitfall 6: Forgetting the builder.rs Import Block
**What goes wrong:** `projection/builder.rs:29` imports `DropdownMenuProps` and uses it in `emit_actions_placeholder`. After the migration, `DropdownMenuProps` no longer exists; the import must be swapped to `ActionGroupProps`.
**How to avoid:** The migration of `emit_actions_placeholder` (D-12) must include updating the import at line :29.

### Pitfall 7: Version Bump Target
**What goes wrong:** Bumping to 0.2.72 (seed target) when the workspace is already at 0.2.72.
**How to avoid:** The target is `0.2.73`. Update only `version = "..."` at `Cargo.toml:46` (workspace version, inherited by all crates via `version.workspace = true`).

---

## Code Examples

### Example 1: ActionItem + ActionGroupProps Structs (Pattern from D-05, D-06, D-07)

```rust
// Target location: ferro-json-ui/src/component.rs, after ButtonGroupProps (~:936)

/// A single action in an ActionGroup's ordered item list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionItem {
    pub label: String,
    pub action: Action,
    /// When true, this item is forced into the overflow kebab and rendered last,
    /// regardless of position in `items`. Does not count toward `max_inline`.
    #[serde(default)]
    pub destructive: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<ButtonVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Fail-closed row gate (same semantics as `DropdownMenuAction.visible_if`).
    /// When set, the item is only shown when `row[visible_if]` is truthy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_if: Option<String>,
}

/// Props for ActionGroup — ordered action list rendering inline buttons (up to
/// `max_inline`) plus a trailing overflow kebab for the remainder. Destructive
/// items are always in the kebab, rendered last.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActionGroupProps {
    pub items: Vec<ActionItem>,
    /// ID pairing the overflow popover to its trigger button. Required; callers
    /// must supply a unique value per page to prevent DOM id collisions.
    pub menu_id: String,
    /// Maximum non-destructive items rendered inline (default 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_inline: Option<u8>,
    /// Aria-label for the overflow trigger button (default "Azioni").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_label: Option<String>,
    /// Key used for `{row_key}` substitution in action URLs (DataTable / Kanban context).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_key: Option<String>,
}
```

### Example 2: BUILTIN_TYPES Swap (render/mod.rs)

```rust
// REMOVE (line 64):
"DropdownMenu",

// ADD in containers section (~after "ButtonGroup" at position in containers group):
"ActionGroup",
```

### Example 3: Dispatch Arm Swap (render/mod.rs)

```rust
// REMOVE (line 197):
"DropdownMenu" => atoms::render_dropdown_menu(el, spec, data, depth),

// ADD in Containers section:
"ActionGroup" => containers::render_action_group(el, spec, data, depth),
```

### Example 4: BUILTIN_SPECS Swap (catalog.rs)

```rust
// REMOVE (~lines 240-244):
(
    "DropdownMenu",
    "Trigger button with an absolutely-positioned kebab-style action panel.",
    || to_value(schema_for!(DropdownMenuProps)).unwrap(),
    &[],
),

// ADD (at same ordinal position as ActionGroup in BUILTIN_TYPES):
(
    "ActionGroup",
    "Ordered action list: inline buttons up to max_inline, trailing overflow kebab for the rest; destructive items forced into the kebab last.",
    || to_value(schema_for!(ActionGroupProps)).unwrap(),
    &[],
),
```

### Example 5: emit_actions_placeholder Migration (projection/builder.rs)

```rust
// BEFORE (lines 680-698):
fn emit_actions_placeholder(…) {
    let items: Vec<DropdownMenuAction> = service.actions.iter().map(|a| DropdownMenuAction {
        label: …, action: …, destructive: false, visible_if: None,
    }).collect();
    let props = serde_json::to_value(DropdownMenuProps {
        menu_id: format!("actions_{}", service.name),
        trigger_label: "Actions".to_string(),
        items,
        trigger_variant: None,
    }).expect(…);
    aux.push((id.clone(), element_with_props("DropdownMenu", props)));
}

// AFTER:
fn emit_actions_placeholder(…) {
    let items: Vec<ActionItem> = service.actions.iter().map(|a| ActionItem {
        label: a.display_name.as_deref().unwrap_or(&a.name).to_string(),
        action: Action::new(format!("/{}/{}", service.name, a.name)),
        destructive: false,
        variant: None,
        icon: None,
        visible_if: None,
    }).collect();
    let props = serde_json::to_value(ActionGroupProps {
        items,
        menu_id: format!("actions_{}", service.name),
        max_inline: None,
        overflow_label: None,
        row_key: None,
    }).expect("ActionGroupProps serialization cannot fail");
    let id = "actions_menu".to_string();
    aux.push((id.clone(), element_with_props("ActionGroup", props)));
}
```

### Example 6: ferro-mcp expected[] Array Fix

```rust
// BEFORE (json_ui_catalog.rs:300-346): 45 entries, missing SegmentedControl and SidebarLayout, has DropdownMenu
let expected = [
    // ... 45 entries including "DropdownMenu" but missing "SegmentedControl", "SidebarLayout"
];

// AFTER: 47 entries — swap DropdownMenu→ActionGroup, add SegmentedControl, SidebarLayout
let expected = [
    // ... same 44 entries minus "DropdownMenu" plus "ActionGroup", "SegmentedControl", "SidebarLayout"
];
// Keep count assertion at 47.
```

---

## Full DropdownMenu Reference Classification

All `DropdownMenu` references in the codebase, classified by required action:

**Public surface removal (must change):**
- `ferro-json-ui/src/lib.rs:54` — export block: remove `DropdownMenuProps` (keep `DropdownMenuAction`)
- `ferro-json-ui/src/render/mod.rs:64` — BUILTIN_TYPES: remove `"DropdownMenu"`
- `ferro-json-ui/src/render/mod.rs:197` — dispatch arm: remove `"DropdownMenu" =>` arm
- `ferro-json-ui/src/catalog.rs:32` — import: remove `DropdownMenuProps` from catalog import
- `ferro-json-ui/src/catalog.rs:241–244` — BUILTIN_SPECS entry: remove the tuple
- `ferro-json-ui/src/component.rs:1074–1082` — `DropdownMenuProps` struct: DELETE
- `ferro-json-ui/src/component.rs:1515` — `schema_for_dropdown_menu_props_generates` test: REPLACE with ActionGroupProps test (keep ActionItem schema test at :1510)

**Internal helpers (keep as-is):**
- `ferro-json-ui/src/render/atoms.rs:1060–1195` — `render_menu_item` + `render_dropdown_menu`: keep `pub(crate)`
- `ferro-json-ui/src/render/atoms.rs:15` — import: keep `DropdownMenuAction, DropdownMenuProps` (render_dropdown_menu still uses DropdownMenuProps; if render_dropdown_menu becomes dead code, this can be cleaned up in a follow-up)
- `ferro-json-ui/src/render/data.rs` — all references are to `DropdownMenuAction`: unchanged
- `ferro-json-ui/src/component.rs:1059–1072` — `DropdownMenuAction` struct: keep
- `ferro-json-ui/src/component.rs:1068` — DropdownMenu reference in comment: can update to say "outside ActionGroup kebab contexts" or similar

**Spec/test migration (must migrate to ActionGroup):**
- `ferro-json-ui/src/render/atoms.rs:2018–2064` — two DropdownMenu render tests: DELETE (replaced by ActionGroup tests)
- `ferro-json-ui/src/projection/builder.rs:29,667–698,1221–1237` — `emit_actions_placeholder` + test + imports: MIGRATE (D-12)

**Docs migration (must migrate):**
- `docs/src/json-ui/components.md:29` — Forms category table: REPLACE `DropdownMenu` → `ActionGroup`
- `docs/src/json-ui/components.md:985–1014` — DropdownMenu section: REPLACE with ActionGroup section
- `docs/src/features/projections.md:504` — action route table: UPDATE reference
- `docs/src/json-ui/expressions.md:156` — incidental mention: UPDATE to reference ActionGroup or make generic

**ferro-mcp mirror (must update):**
- `ferro-mcp/src/tools/json_ui_catalog.rs:335` — expected[] name: SWAP `"DropdownMenu"` → `"ActionGroup"`, ADD `"SegmentedControl"`, `"SidebarLayout"` (fix the pre-existing 45-vs-47 gap)

---

## CSS Regen Step

`scripts/gen-ferro-base-css.sh` must be run **after** the component code lands, per the project checklist (MEMORY.md). The script runs Tailwind v4 CLI against `ferro-json-ui/assets/input.css` and outputs `ferro-json-ui/assets/ferro-base.css`.

ActionGroup is expected to reuse existing CSS classes:
- Inline button row: likely `flex items-center gap-2` (same as ButtonGroup container)
- Inline buttons: existing `ButtonVariant` classes (already in the CSS)
- Overflow kebab trigger: same classes as `render_dropdown_menu` trigger button (`inline-flex items-center justify-center rounded-md p-1.5 text-text-muted hover:text-text hover:bg-surface …`)
- Overflow popover panel: same `w-48 rounded-md border border-border bg-card shadow-md` as existing DropdownMenu

If `render_action_group` introduces no new CSS class names (highly likely since it reuses the same building blocks), the regen may produce identical output. The step is still required by the checklist — run it and commit the result even if the file is unchanged.

---

## Validation Architecture

`nyquist_validation` is not explicitly set to `false` in `.planning/config.json` (the key is absent) — treat as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | `ferro-json-ui/Cargo.toml` (no separate test config) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req (SC) | Behavior | Test Type | Automated Command | File Exists? |
|----------|----------|-----------|-------------------|-------------|
| SC-1 | Inline ≤ max_inline, overflow ≥ max_inline, destructive always in kebab last | unit | `cargo test -p ferro-json-ui render_action_group` | ❌ Wave 0 |
| SC-1 | Kebab hidden when no overflow | unit | `cargo test -p ferro-json-ui action_group_no_overflow_hides_kebab` | ❌ Wave 0 |
| SC-2 | `$data` binding renders identically to literal list | unit | `cargo test -p ferro-json-ui action_group_data_binding_parity` | ❌ Wave 0 |
| SC-2 | `visible_if` row gate (fail-closed) | unit | `cargo test -p ferro-json-ui action_group_visible_if` | ❌ Wave 0 (pattern exists in data.rs visible_if tests) |
| SC-3 | Non-GET inline action renders inside `<form>` | unit | `cargo test -p ferro-json-ui action_group_non_get_wraps_form` | ❌ Wave 0 |
| SC-3 | GET action renders as anchor/link | unit | `cargo test -p ferro-json-ui action_group_get_renders_link` | ❌ Wave 0 |
| SC-4 | `DropdownMenu` absent from BUILTIN_TYPES | unit | `builtin_types_count_drift_guard` (existing, stays at 47) | ✅ |
| SC-4 | `ActionGroup` present in catalog | unit | `build_populates_all_builtins` (existing, now includes ActionGroup) | ✅ existing tests cover this |
| SC-4 | Drift guards pass (count = 47) | unit | `cargo test -p ferro-json-ui builtin_types_count_drift_guard` | ✅ (update count comment) |
| SC-5 | `emit_actions_placeholder` emits `ActionGroup` not `DropdownMenu` | unit | `actions_slot_emits_dropdown_from_service_actions` (existing test, update name + decode type) | ✅ (update existing) |
| SC-5 | ferro-mcp name list contains `ActionGroup`, not `DropdownMenu` | unit | `cargo test -p ferro-mcp test_all_components_present` | ✅ (update expected[]) |

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/render/containers.rs` — add `render_action_group` tests covering: inline-vs-overflow partition, destructive-last ordering, form-wrapping for non-GET, kebab-hidden-when-no-overflow, `visible_if` gate
- [ ] `ferro-json-ui/src/component.rs` — add `schema_for_action_group_props_generates()` and `schema_for_action_item_generates()` tests

*(Existing test infrastructure covers all other requirements — the bulk of the validation is in the existing drift-guard tests and projection builder tests that get updated.)*

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before moving to the publish step

---

## Environment Availability

Step 2.6: SKIPPED — this phase is purely code changes within the existing workspace. No external services, databases, or CLI tools beyond the standard Rust toolchain are required. The CSS regen script (`scripts/gen-ferro-base-css.sh`) downloads Tailwind CLI on first run via `scripts/install-tailwind.sh`, but the script handles its own dependency.

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is not a rename/refactor/migration phase. `ActionGroup` is a new name; `DropdownMenu` removal is from the public surface only; no stored data, live service configs, or OS-registered state reference either name as an identifier.

---

## Open Questions (RESOLVED)

1. **Is `render_dropdown_menu` dead code after this phase? — RESOLVED: YES, delete it.**
   - What we know: `render_dropdown_menu` at `atoms.rs:1154` is currently called only from the dispatch arm (BUILTIN_TYPES "DropdownMenu" → atoms dispatch). After removing that dispatch arm, no call site in the main render path calls `render_dropdown_menu` directly — DataTable uses `render_inline_dropdown` (data.rs:520), and `render_action_group` reuses the building blocks (`render_menu_item`'s non-GET `<form>` branch + the kebab trigger/panel HTML at atoms.rs:1166-1193) directly rather than going through `render_dropdown_menu`'s `Element`-based API.
   - **RESOLUTION:** `render_dropdown_menu` becomes dead code and is **deleted** (along with its atoms.rs tests ~:2018-2064) in Plan 03, after Plan 02 removes the dispatch arm. CI `-D warnings` enforces this. `render_menu_item` and the kebab building blocks are **kept** (`pub(crate)`). `DropdownMenuAction` is **kept** (D-11). This resolution refines CONTEXT D-10, which has been updated to match — the "keep the kebab rendering in one place" intent holds; only the specific retained helper changes from `render_dropdown_menu` to `render_menu_item` + building blocks.

2. **Should ActionGroup slots appear in `BUILTIN_SPECS` slot_fields? — RESOLVED: NO, use `&[]`.**
   - What we know: `BUILTIN_SPECS` tuples have a `&[&str]` for slot fields. Current containers with slots: Card (`footer`), Modal (`footer`), PageHeader (`actions`), DetailPage (`actions`). ActionGroup has no child slots (all content is driven by the `items` prop array).
   - **RESOLUTION:** Use `&[]` for ActionGroup's slot_fields (no children slots). This matches DropdownMenu's current `&[]`. Applied in Plan 02 (BUILTIN_SPECS entry).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | CSS regen produces identical output (no new class names introduced) | CSS Regen Step | Low — if new classes are introduced, regen still works; just produces a larger CSS file |
| A2 | `render_dropdown_menu` becomes dead code after the dispatch arm is removed and unit tests migrate | Open Questions | Low — compiler `-D warnings` will surface it as an unused function if so |

All other claims in this research are verified by direct source inspection.

---

## Sources

### Primary (HIGH confidence — all verified by direct code inspection)

- `ferro-json-ui/src/render/mod.rs` — BUILTIN_TYPES (`:43`), dispatch (`:176`), DropdownMenu at `:64` / `:197`
- `ferro-json-ui/src/render/atoms.rs` — `render_menu_item` (`:1073`), `render_dropdown_menu` (`:1154`), `render_button` (`:203`)
- `ferro-json-ui/src/render/containers.rs` — `render_page_header` (`:597`), `render_detail_page` (`:685`), `render_button_group` (`:946`)
- `ferro-json-ui/src/render/data.rs` — `action_visible_for_row` (`:445`), `template_actions` (`:467`), `render_inline_dropdown` (`:520`)
- `ferro-json-ui/src/component.rs` — `DropdownMenuAction` (`:1059`), `DropdownMenuProps` (`:1076`), `DataTableProps.row_actions` (`:1091`), `ButtonProps` (`:262`), `ButtonVariant` (`:55`), schema tests (`:1509`)
- `ferro-json-ui/src/lib.rs` — export block (`:49–63`), `DropdownMenuProps` at `:54`
- `ferro-json-ui/src/catalog.rs` — `BUILTIN_SPECS` (`:124`), DropdownMenu spec (`:241`), runtime guard (`:576`), count guard (`:1093`)
- `ferro-json-ui/src/projection/builder.rs` — `emit_actions_placeholder` (`:672`), its test (`:1220`)
- `ferro-json-ui/src/expression.rs` — `resolve_expressions` (`:35`), `resolve_value` (`:42`)
- `ferro-json-ui/src/action.rs` — `Action` struct (`:154`), `HttpMethod` (`:28`)
- `ferro-mcp/src/tools/json_ui_catalog.rs` — count mirror (`:292`), expected[] (`:300`), current 45-vs-47 gap verified
- `Cargo.toml` — workspace version at `:46` = `"0.2.72"` [VERIFIED]
- `.github/workflows/publish.yml` — ferro-json-ui in Wave 1A at `:211` [VERIFIED]
- `docs/src/json-ui/components.md` — DropdownMenu section at `:985` [VERIFIED]
- `docs/src/features/projections.md` — DropdownMenu reference at `:504` [VERIFIED]
- `docs/src/json-ui/expressions.md` — incidental mention at `:156` [VERIFIED]
- `framework/src/csrf/mod.rs` — CSRF via meta tag (`:9`), confirms no hidden field needed [VERIFIED]
- `scripts/gen-ferro-base-css.sh` — exists, runs Tailwind v4 CLI [VERIFIED]

---

## Metadata

**Confidence breakdown:**
- Registration surface (8 touchpoints): HIGH — all file:line verified against current source
- Version bump target (0.2.73): HIGH — workspace Cargo.toml directly read
- Form-wrapping pattern (render_menu_item, not render_button): HIGH — both functions read
- $data binding mechanics: HIGH — expression.rs resolve_expressions verified
- ferro-mcp 45-vs-47 gap: HIGH — counted array entries directly
- CSS regen effect: MEDIUM — likely unchanged (reuses same CSS classes), but not run in this session

**Research date:** 2026-06-22
**Valid until:** 2026-07-22 (stable codebase; only invalidated by changes to the 8 registration files)
