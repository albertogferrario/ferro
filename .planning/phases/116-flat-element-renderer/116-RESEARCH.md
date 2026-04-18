# Phase 116: Flat Element Renderer - Research

**Researched:** 2026-04-18
**Domain:** HTML string emission over a flat ID-keyed element graph (ferro-json-ui v2 runtime)
**Confidence:** HIGH

## Summary

Phase 116 replaces the ~95-LOC Phase 115 placeholder in `ferro-json-ui/src/render.rs` with
a real flat-element walker. The walker is a single recursive function `render_element(id,
spec, data, depth) -> String` that looks up an element by ID, evaluates visibility,
dispatches on `type_name` to a per-component renderer, and lets each container recurse
back into `render_element` for its child IDs.

The v1 renderer (retrieved via `git show 40385f32^:ferro-json-ui/src/render.rs`, 8057
LOC) is the authoritative HTML contract. Phase 116 preserves its byte-level output for
every built-in component so the gestiscilo field test (Phase 121) sees no regressions.
The architectural rewrite is confined to three surfaces: (1) the dispatch layer (v1
matched `Component` enum variants, v2 matches `type_name: &str`), (2) child lookup (v1
read `props.children: Vec<ComponentNode>`, v2 reads ID strings from `Element.children`
or typed slot fields and calls `spec.elements.get(id)`), (3) plugin asset collection (v1
recursed through typed slots; v2 is a flat `spec.elements.values()` pass).

**Primary recommendation:** Port the v1 per-component function bodies verbatim, rewrite
only the signature and child-lookup sites, add a `Visibility::evaluate(&Value) -> bool`
method (which does NOT currently exist — see Risks HIGH-3), split `render.rs` into
`render/{mod,containers,form,data,atoms}.rs`, and land Phase 116 as 5–6 small plans
rather than one monolithic plan.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Element graph lookup (id → Element) | ferro-json-ui render module | — | `Spec.elements` HashMap is the only source of truth |
| Per-component HTML emission | ferro-json-ui render/{atoms,form,data,containers}.rs | — | Per-type functions own their HTML contract |
| Visibility evaluation | ferro-json-ui visibility.rs (new `evaluate()`) | ferro-json-ui render/mod.rs (call site) | Pure data-predicate logic belongs next to the `Visibility` enum |
| Action URL wiring | ferro-json-ui resolve.rs (pre-render, existing) | ferro-json-ui render (consumer) | Renderer consumes `Element.action.url` as pre-resolved string |
| Data-path resolution for form pre-fill | ferro-json-ui data.rs (existing, unused) | ferro-json-ui render/{form,data}.rs | Drop `#[allow(dead_code)]`; callers are Input/Select/Checkbox/Switch/Table/DataTable renderers |
| Plugin dispatch | ferro-json-ui plugin.rs (existing `with_plugin`) | ferro-json-ui render/mod.rs default match arm | Type-erased dispatch at the walker default arm |
| Plugin asset collection | ferro-json-ui render/mod.rs (new `collect_plugin_types`) | ferro-json-ui plugin.rs (existing `collect_plugin_assets`) | Flat pass over `spec.elements.values()`, subtract `BUILTIN_TYPES` |
| Framework-level HTTP wrapping | framework/src/json_ui/mod.rs (unchanged) | — | Already calls `render_spec_to_html_with_plugins`; signature frozen by Phase 115-03 |

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Dispatch architecture:**
- **D-01:** Single `match el.type_name.as_str()` dispatch. ~30 arms for built-ins plus a default arm that consults the plugin registry.
- **D-02:** Reject the trait-object dispatch alternative.
- **D-03:** Dispatch default arm is `with_plugin(type_name, |p| p.render(&el.props, data))`. If neither built-in nor plugin matches, emit an HTML comment diagnostic and render nothing.
- **D-04:** Per-element pipeline, in order:
  1. Evaluate `el.visible` — if false, return `""` without walking children.
  2. Match `el.type_name` → dispatch to typed renderer.
  3. Typed renderer: deserialize `el.props` into the component's `*Props` struct, then emit HTML. Children are resolved by ID lookup inside the renderer.

**Element graph and slot binding:**
- **D-05:** `Element.children` is the graph-canonical primary slot. Single-slot container components render `Element.children` in order.
- **D-06:** Multi-slot components carry slot-specific `Vec<String>` ID lists in their Props. Phase 116 re-adds these fields:
  - `CardProps.footer: Vec<String>`
  - `ModalProps.footer: Vec<String>`
  - `Tab { value, label, children: Vec<String> }`
  - `KanbanColumnProps.children: Vec<String>`
  - `PageHeaderProps.actions: Vec<String>`
- **D-07:** Slot IDs are NOT covered by Phase 115's structural validator. Known Phase 116 limitation.
- **D-08:** No Element struct changes.

**Graceful failure surface:**
- **D-09:** Renderer is infallible — returns `String`, never `Result`.
- **D-10:** Diagnostic surface is HTML comments. Zero new dependencies (no `tracing` / `log`). Emitted cases:
  - Missing slot/children ID: `<!-- ferro-json-ui: element 'parent' references missing child 'ghost' -->`
  - Unknown `type_name`: `<!-- ferro-json-ui: unknown component type 'Foo' -->`
  - Props deserialization failure: `<!-- ferro-json-ui: failed to decode Card props on element 'hero': missing field 'title' -->`
  - Cycle tripwire: `<!-- ferro-json-ui: cycle guard tripped at depth N — spec should have been rejected at parse time -->`
- **D-11:** Defense-in-depth cycle guard. Depth counter passed through the walker; fires at `MAX_NESTING_DEPTH + 1` (= 4).
- **D-12:** `serde_json::from_value::<TProps>(el.props.clone())` is the standard decode step. On `Err`, emit D-10 comment and return `""`.

**Visibility:**
- **D-13:** Inline per-element evaluation using `Visibility::evaluate(&Value) -> bool`.
- **D-14:** Invisible elements are skipped entirely — their children are not rendered. If invisible root, emit `<!-- ferro-json-ui: root hidden -->`.

**Action resolution:**
- **D-15:** Renderer assumes actions are pre-resolved (`Element.action.url = Some(...)`).
- **D-16:** Actions with `url = None` at render time degrade to `href="#"` + diagnostic comment.

**Plugin rendering and asset collection:**
- **D-17:** Plugin fallback in the dispatch default arm. Reads `with_plugin(type_name, |p| p.render(&el.props, data))`.
- **D-18:** Plugin asset collection is a separate pass over `spec.elements` via `collect_plugin_types(spec) -> HashSet<String>`.
- **D-19:** Plugins are identified by "type_name not in BUILTINS AND present in plugin registry". Built-in list is `const BUILTIN_TYPES: &[&str]` in `render/mod.rs`. Plugins cannot shadow built-ins.

**Module layout:**
- **D-20:** Split `render.rs` into `render/{mod,containers,form,data,atoms}.rs`. Each file <2000 LOC. Per-component renderer signature: `(props: &TProps, el: &Element, spec: &Spec, data: &Value, depth: usize) -> String`.
- **D-21:** Per-component renderer bodies port verbatim from v1 where possible (git ref `40385f32^:ferro-json-ui/src/render.rs`).
- **D-22:** `html_escape` stays in `render/mod.rs`, exported `pub(crate)`.

**Testing:** D-23–D-26 — port every v1 renderer test, add walker-specific tests, update framework/src/json_ui/mod.rs integration tests, snapshot fixtures optional.

**Performance:** D-27 O(n) walk only; D-28 no memoization.

**Out-of-scope:** D-29 no `$data`/`$template` evaluation; D-30 no catalog validation; D-31 `JsonUi::render` signature unchanged.

### Claude's Discretion

- Whether per-component render functions live as `pub(crate) fn render_card(...)` or private with a module-level re-export — pick the Rust-idiomatic choice.
- Whether to introduce a small `Walker` struct vs. passing three args explicitly — pick whichever reads cleaner.
- Whether `render_spec_to_html` returns a diagnostic comment when `spec.root` is missing from `spec.elements` (shouldn't happen post-from_json) or just returns empty — defensive skip is fine.
- Diagnostic comment exact wording — consistency matters more than the specific phrase.
- Exact split point between `render/data.rs` and `render/containers.rs` for borderline components — keep files balanced.

### Deferred Ideas (OUT OF SCOPE)

- Render-cache / memoization (post-v1.0 perf pass).
- Streaming renderer (`Write`-based variant) — post-v1.0.
- `tracing` / `log` integration — HTML-comment diagnostics are the Phase 116 observability surface.
- Catalog-driven validation before render — Phase 117.
- `$data` / `$template` expression resolution — Phase 118.
- Spec hot-reload / file-watcher — Phase 119.
- CLI + MCP updates for v2 — Phase 120.
- Docs rewrite — Phase 121.
- Full slot-ID graph validation — Phase 117 catalog concern.
- `render_to_writer<W: Write>` API — not a v12.0 goal.
- Client-side React/Vue/Svelte runtime consuming the same Spec JSON.
- Per-element render instrumentation / profiling hooks — post-v1.0.
- Sandboxing untrusted Specs.

</user_constraints>

<phase_requirements>
## Phase Requirements

Requirement IDs `RENDER-01/02/03` appear in `ROADMAP.md` §"Phase 116" but are NOT
expanded in `REQUIREMENTS.md` (which tracks v13.0 only). Per the research brief, the 6
ROADMAP success criteria are the enforceable requirements for Phase 116.

| ID | Description | Research Support |
|----|-------------|------------------|
| SC-1 | `render_spec_to_html(spec, data)` renders all component types from flat element map | v1 renderer inventory (Section "v1 Renderer Inventory") lists all 43 `render_*` functions; all get ported verbatim with signature changes per D-20/D-21. Dispatch table in Section "Built-in type_name Canonical List" |
| SC-2 | Element ID lookup handles missing children gracefully (skip + warn, don't panic) | Diagnostic HTML-comment spec (D-10); walker always calls `spec.elements.get(id)` via `Option` and emits comment on `None` |
| SC-3 | Action resolution works on flat elements (handler → URL via callback) | Existing `resolve_actions(&mut spec, resolver)` in `resolve.rs` runs pre-render (already wired in `framework/src/json_ui/mod.rs:39-42`); renderer reads `el.action.url` as `Some(String)`; None → `href="#"` + comment (D-16) |
| SC-4 | Visibility evaluation works on flat elements (conditional rendering) | Requires new `Visibility::evaluate(data: &Value) -> bool` method — NOT currently in the codebase (see Risks HIGH-3). Calling site is `render_element` pre-dispatch (D-13) |
| SC-5 | Plugin components render correctly in v2 specs | Existing `with_plugin(type_name, \|p\| p.render(&el.props, data))` is the default match arm; `collect_plugin_types(spec) -> HashSet<String>` is a new flat pass; `collect_plugin_assets` unchanged |
| SC-6 | Old `render_to_html(view, data)` function is deleted | Already deleted in Phase 115-02 (commit `c88745a4`); Phase 116 verifies no regressions via grep for `render_to_html` (must find zero live occurrences) |

</phase_requirements>

## Problem Framing

**What Phase 116 must do:** Replace the Phase 115 `<pre>`-JSON placeholder with a real
HTML walker that takes a validated `Spec` and emits the same Tailwind-classed HTML the
v1 renderer produced, but reading children by ID lookup into a flat `HashMap<String,
Element>` instead of recursing into typed `Vec<ComponentNode>` slots.

**Why this is non-trivial:**
1. **Scale.** The v1 renderer was 8057 LOC with 43 per-component functions; every one
   must be ported.
2. **New slot model.** Five Props structs lost their slot fields in Phase 115-02
   (v1-type strip). Phase 116 re-adds them as `Vec<String>` ID lists, requiring
   coordinated edits to both `component.rs` and the per-component renderers that read
   them.
3. **New surface area.** `Visibility::evaluate()` is called out in D-13 as existing, but
   it does not exist in the codebase — neither v1 nor current visibility.rs implements
   it. Phase 116 must add it (see Risks HIGH-3).
4. **Infallible-renderer contract.** Every failure path must degrade to an HTML comment,
   no unwraps, no panics, no `Result` return. This is a design discipline applied
   consistently across 43 renderer functions.
5. **Byte-level HTML preservation.** gestiscilo production depends on specific class
   combinations, data-attribute names, and ARIA wiring from v1. The walker can change;
   the emitted HTML cannot.

**What keeps it tractable:**
- The v1 renderer is the spec. Port verbatim; redesign nothing.
- Phase 115 already shipped the parse-time graph validator (no cycles, depth ≤ 3). The
  walker depth guard is pure defense-in-depth.
- `data.rs` and `plugin.rs` are ready-to-use; no new infrastructure needed.

## v1 Renderer Inventory

All 43 `render_*` functions from `git show 40385f32^:ferro-json-ui/src/render.rs`. LOC
are approximate from the start line of one function to the start of the next. "Slot
re-add" column lists which `Vec<String>` slot field (if any) must be re-added to the
component's Props struct per D-06. Assignments to `render/*.rs` files are a proposal
per D-20; planner may shift borderline items.

> `render_node`, `render_component`, `render_css_tags`, `render_js_tags`, and
> `collect_plugin_types_node` are v1 infrastructure functions; they collapse into the
> new walker (`render_element`) and into `render/mod.rs` helpers. They are NOT
> per-component renderers.

### Atoms (leaf components, `render/atoms.rs`)

| v1 fn | type_name | LOC | v1 child access | Slot re-add | Target file |
|-------|-----------|-----|-----------------|-------------|-------------|
| `render_text` | `Text` | 19 | — (leaf) | none | atoms.rs |
| `render_button` | `Button` | 65 | — (leaf, but wraps in `<a>` via render_node for GET actions) | none | atoms.rs |
| `render_badge` | `Badge` | 46 | — | none | atoms.rs |
| `render_alert` | `Alert` | 30 | — | none | atoms.rs |
| `render_separator` | `Separator` | 8 | — | none | atoms.rs |
| `render_progress` | `Progress` | 22 | — | none | atoms.rs |
| `render_avatar` | `Avatar` | 31 | — | none | atoms.rs |
| `render_image` | `Image` | 42 | — | none | atoms.rs |
| `render_skeleton` | `Skeleton` | 20 | — | none | atoms.rs |
| `render_breadcrumb` | `Breadcrumb` | 28 | — (BreadcrumbItem, not an element) | none | atoms.rs |
| `render_empty_state` | `EmptyState` | 29 | — | none | atoms.rs |
| `render_stat_card` | `StatCard` | 35 | — | none | atoms.rs |
| `render_checklist` | `Checklist` | 63 | — (ChecklistItem, not an element) | none | atoms.rs |
| `render_toast` | `Toast` | 44 | — | none | atoms.rs |
| `render_notification_dropdown` | `NotificationDropdown` | 67 | — (NotificationItem, not an element) | none | atoms.rs |
| `render_sidebar` | `Sidebar` | 44 | — (SidebarNavItem / SidebarGroup, not elements) | none | atoms.rs |
| `render_sidebar_nav_item` | (private helper, no type_name) | 20 | — | none | atoms.rs (private) |
| `render_header` | `Header` | ~40 (last fn, EOF-bounded) | — | none | atoms.rs |
| `render_dropdown_menu` | `DropdownMenu` | 103 | — (DropdownMenuAction, not an element) | none | atoms.rs |
| `render_calendar_cell` | `CalendarCell` | 64 | — | none | atoms.rs |
| `render_action_card` | `ActionCard` | 52 | — | none | atoms.rs |
| `render_product_tile` | `ProductTile` | 28 | — | none | atoms.rs |

**atoms.rs estimated LOC:** ~930 + helpers/imports. Largest file; splitting if needed
along dashboard-chrome (Sidebar/Header/NotificationDropdown) vs. primitives would be
cosmetic.

### Containers (`render/containers.rs`)

| v1 fn | type_name | LOC | v1 child access | Slot re-add | Target file |
|-------|-----------|-----|-----------------|-------------|-------------|
| `render_card` | `Card` | 46 | `props.children: Vec<ComponentNode>`, `props.footer: Vec<ComponentNode>` | `CardProps.footer: Vec<String>` (children already in `Element.children`) | containers.rs |
| `render_modal` | `Modal` | 50 | `props.children`, `props.footer` | `ModalProps.footer: Vec<String>` (children in `Element.children`) | containers.rs |
| `render_tabs` | `Tabs` | 96 | `props.tabs[i].children: Vec<ComponentNode>` | `Tab.children: Vec<String>` (field on the `Tab` struct inside `TabsProps.tabs`) | containers.rs |
| `render_kanban_board` | `KanbanBoard` | 92 | `props.columns[i].children: Vec<ComponentNode>` | `KanbanColumnProps.children: Vec<String>` | containers.rs |
| `render_page_header` | `PageHeader` | 50 | `props.actions: Vec<ComponentNode>` | `PageHeaderProps.actions: Vec<String>` | containers.rs |
| `render_grid` | `Grid` | 42 | `props.children` | none — uses `Element.children` (single-slot per D-05) | containers.rs |
| `render_collapsible` | `Collapsible` | 20 | `props.children` | none — uses `Element.children` | containers.rs |
| `render_form_section` | `FormSection` | 49 | `props.children` | none — uses `Element.children` | containers.rs |
| `render_button_group` | `ButtonGroup` | 11 | v1 used `props.buttons`; v2 uses `Element.children` | none — uses `Element.children` | containers.rs |

**containers.rs estimated LOC:** ~460 + imports. Well under 2000-LOC target.

### Form controls (`render/form.rs`)

| v1 fn | type_name | LOC | v1 child access | Slot re-add | Target file |
|-------|-----------|-----|-----------------|-------------|-------------|
| `render_form` | `Form` | 56 | `props.fields: Vec<ComponentNode>` | none — v2 uses `Element.children` as the fields slot per D-05 (FormProps loses `fields` field entirely; already done in Phase 115-02) | form.rs |
| `render_input` | `Input` | 149 | — (leaf); reads `props.data_path` + `data` via `resolve_path_string` | none | form.rs |
| `render_select` | `Select` | 98 | — (leaf); reads `data_path` | none | form.rs |
| `render_checkbox` | `Checkbox` | 67 | — (leaf); reads `data_path` | none | form.rs |
| `render_switch` | `Switch` | 112 | — (leaf); reads `data_path`; wraps itself in a `<form>` when `action` prop present | none | form.rs |

**form.rs estimated LOC:** ~482. Well under target.

### Data displays (`render/data.rs`)

| v1 fn | type_name | LOC | v1 child access | Slot re-add | Target file |
|-------|-----------|-----|-----------------|-------------|-------------|
| `render_table` | `Table` | 87 | — (leaf, reads rows from `data` via `data_path`) | none | data.rs |
| `render_data_table` | `DataTable` | 185 | — (leaf, URL-template `{id}` replacement per row) | none | data.rs |
| `render_description_list` | `DescriptionList` | 16 | — | none | data.rs |
| `render_pagination` | `Pagination` | 81 | — | none | data.rs |

**data.rs estimated LOC:** ~369. Comfortable size.

### Infrastructure (`render/mod.rs`)

| v1 fn | Purpose | LOC | New signature in v2 |
|-------|---------|-----|---------------------|
| `render_to_html` | Top-level wrapper | ~12 | Replaced by `render_spec_to_html(spec, data) -> String` |
| `render_to_html_with_plugins` | Plugin-aware top-level | ~20 | Replaced by `render_spec_to_html_with_plugins(spec, data) -> RenderResult` |
| `render_node` | Per-node dispatch + `<a>` wrapping for GET actions | 37 | Merged into `render_element(id, spec, data, depth) -> String` |
| `render_component` | Match on `Component` enum variant | 67 | Becomes `match el.type_name.as_str()` inside `render_element` |
| `collect_plugin_types` / `_node` | Recursive plugin-type walk | ~95 total | Replaced by flat `collect_plugin_types(spec: &Spec) -> HashSet<String>` (~20 LOC) |
| `render_css_tags` / `render_js_tags` | Asset tag emitters | 22 + 29 | Port verbatim |
| `html_escape` | Reused escape helper | 7 | Already in place (Phase 115 placeholder) |
| `render_plugin` | Plugin dispatch (v1 had a `Component::Plugin` variant) | 14 | Replaced by default arm inside `render_element` match |

**mod.rs estimated LOC:** ~300–400 (dispatch match + walker + plugin collection + asset
tag helpers + escape helper + public API + `BUILTIN_TYPES` const).

### v1 test count

201 tests at `#[cfg(test)]` in v1 `render.rs` (per `grep -c "^    fn [a-z]"`). Phase
115-02 deleted them with the v1 types. D-23 says port every one; realistic target given
test redundancy (e.g., `button_size_xs/sm/default/lg` are 4 tests of the same concern)
is ~60 meaningful test cases covering all 29 built-in type_names plus variants and edge
cases.

## Built-in type_name Canonical List

The complete list going into the dispatch match AND `const BUILTIN_TYPES: &[&str]` in
`render/mod.rs`. 29 entries (v1 had 30 Component variants; v2 drops `Component::Plugin`
since plugins are now type-erased per D-03). Ordering chosen for readability in the
match; final ordering at plan time is Claude's discretion.

| type_name | Props struct | Slot-bearing Props fields (added by Phase 116) | v1 render fn |
|-----------|--------------|-----------------------------------------------|--------------|
| `Text` | `TextProps` | — | render_text |
| `Button` | `ButtonProps` | — | render_button |
| `Badge` | `BadgeProps` | — | render_badge |
| `Alert` | `AlertProps` | — | render_alert |
| `Separator` | `SeparatorProps` | — | render_separator |
| `Progress` | `ProgressProps` | — | render_progress |
| `Avatar` | `AvatarProps` | — | render_avatar |
| `Image` | `ImageProps` | — | render_image |
| `Skeleton` | `SkeletonProps` | — | render_skeleton |
| `Breadcrumb` | `BreadcrumbProps` | — | render_breadcrumb |
| `Pagination` | `PaginationProps` | — | render_pagination |
| `DescriptionList` | `DescriptionListProps` | — | render_description_list |
| `EmptyState` | `EmptyStateProps` | — | render_empty_state |
| `StatCard` | `StatCardProps` | — | render_stat_card |
| `Checklist` | `ChecklistProps` | — | render_checklist |
| `Toast` | `ToastProps` | — | render_toast |
| `NotificationDropdown` | `NotificationDropdownProps` | — | render_notification_dropdown |
| `Sidebar` | `SidebarProps` | — | render_sidebar |
| `Header` | `HeaderProps` | — | render_header |
| `DropdownMenu` | `DropdownMenuProps` | — | render_dropdown_menu |
| `CalendarCell` | `CalendarCellProps` | — | render_calendar_cell |
| `ActionCard` | `ActionCardProps` | — | render_action_card |
| `ProductTile` | `ProductTileProps` | — | render_product_tile |
| `Card` | `CardProps` | **footer: Vec\<String\>** (children via `Element.children`) | render_card |
| `Modal` | `ModalProps` | **footer: Vec\<String\>** (children via `Element.children`) | render_modal |
| `Tabs` | `TabsProps` (contains `Vec<Tab>`) | **Tab.children: Vec\<String\>** (on the nested `Tab` struct) | render_tabs |
| `KanbanBoard` | `KanbanBoardProps` (contains `Vec<KanbanColumnProps>`) | **KanbanColumnProps.children: Vec\<String\>** | render_kanban_board |
| `PageHeader` | `PageHeaderProps` | **actions: Vec\<String\>** | render_page_header |
| `Grid` | `GridProps` | — (uses `Element.children`) | render_grid |
| `Collapsible` | `CollapsibleProps` | — (uses `Element.children`) | render_collapsible |
| `FormSection` | `FormSectionProps` | — (uses `Element.children`) | render_form_section |
| `ButtonGroup` | `ButtonGroupProps` | — (uses `Element.children`) | render_button_group |
| `Form` | `FormProps` | — (fields via `Element.children`; v1 `FormProps.fields` field was dropped in Phase 115-02 and stays dropped) | render_form |
| `Table` | `TableProps` | — | render_table |
| `DataTable` | `DataTableProps` | — | render_data_table |
| `Input` | `InputProps` | — | render_input |
| `Select` | `SelectProps` | — | render_select |
| `Checkbox` | `CheckboxProps` | — | render_checkbox |
| `Switch` | `SwitchProps` | — | render_switch |

**Total:** 34 distinct type_names (some atoms cross the 29 I said above — recounting
against current `component.rs` gives 34 `*Props` structs with render functions, matching
the v1 Component-enum variant count minus `Plugin`).

**`const BUILTIN_TYPES` declaration:**

```rust
pub(crate) const BUILTIN_TYPES: &[&str] = &[
    "Text", "Button", "Badge", "Alert", "Separator", "Progress", "Avatar", "Image",
    "Skeleton", "Breadcrumb", "Pagination", "DescriptionList", "EmptyState", "StatCard",
    "Checklist", "Toast", "NotificationDropdown", "Sidebar", "Header", "DropdownMenu",
    "CalendarCell", "ActionCard", "ProductTile",
    "Card", "Modal", "Tabs", "KanbanBoard", "PageHeader",
    "Grid", "Collapsible", "FormSection", "ButtonGroup",
    "Form", "Table", "DataTable", "Input", "Select", "Checkbox", "Switch",
];
```

(39 entries — the 34 above plus the 5 multi-slot containers already counted.
Double-check against the dispatch match at plan time; they must be one-to-one.)

## Slot Re-addition Plan

Concrete diff for `ferro-json-ui/src/component.rs`. Each re-addition is a single field
addition with standard serde defaults. No derive changes needed — all 5 Props structs
already derive `JsonSchema`, and `Vec<String>` is `JsonSchema`-friendly.

### `CardProps` (add `footer`)

```rust
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FormMaxWidth>,
    /// IDs of footer elements (resolved against `Spec.elements`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<String>,
}
```

### `ModalProps` (add `footer`)

```rust
pub struct ModalProps {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_label: Option<String>,
    /// IDs of footer elements (resolved against `Spec.elements`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<String>,
}
```

### `Tab` (add `children`)

```rust
pub struct Tab {
    pub value: String,
    pub label: String,
    /// IDs of elements rendered inside this tab's panel.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
}
```

### `KanbanColumnProps` (add `children`)

```rust
pub struct KanbanColumnProps {
    pub id: String,
    pub title: String,
    pub count: u32,
    /// IDs of elements rendered inside this column.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
}
```

### `PageHeaderProps` (add `actions`)

```rust
pub struct PageHeaderProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breadcrumb: Vec<BreadcrumbItem>,
    /// IDs of action button elements rendered to the right of the title.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
}
```

**Phase 115 test impact analysis:**
- `tests/round_trip.rs` fixtures don't exercise these slot fields, so all 8
  round-trip tests keep passing.
- `tests/reject.rs` doesn't touch Props internals; no impact.
- Inline `schema_for_*_generates` tests assert non-empty `properties` field — adding a
  field only adds a key, so all 42 schema tests keep passing.
- Inline `component::tests` (if any remain) should be audited at Plan 116-01 but the
  Phase 115 VERIFICATION report shows 42/42 schema tests plus round-trip/reject all
  green after the current shape; adding a serde-default field is backward-compatible
  with existing fixtures (absent → empty Vec).

## Dispatch Function Signature

### Walker entrypoint

```rust
/// Render a single element by ID. The one recursive function in the whole
/// walker — all dispatch, visibility, depth-guard, and diagnostic logic
/// lives here.
pub(crate) fn render_element(
    id: &str,
    spec: &Spec,
    data: &Value,
    depth: usize,
) -> String {
    // Depth tripwire (defense-in-depth per D-11).
    if depth > MAX_NESTING_DEPTH + 1 {
        return format!(
            "<!-- ferro-json-ui: cycle guard tripped at depth {depth} — spec should have been rejected at parse time -->"
        );
    }

    // ID lookup (missing child diagnostic per D-10).
    let Some(el) = spec.elements.get(id) else {
        return format!(
            "<!-- ferro-json-ui: element references missing id '{}' -->",
            html_escape(id)
        );
    };

    // Visibility check (D-13/D-14).
    if let Some(vis) = &el.visible {
        if !vis.evaluate(data) {
            return String::new();
        }
    }

    // Dispatch by type_name.
    match el.type_name.as_str() {
        "Text" => atoms::render_text(el, spec, data, depth),
        "Button" => atoms::render_button(el, spec, data, depth),
        // ... ~37 more built-in arms ...
        "Card" => containers::render_card(el, spec, data, depth),
        "Modal" => containers::render_modal(el, spec, data, depth),
        // ... rest ...
        other => render_plugin_or_unknown(other, el, data),
    }
}
```

### Per-component function signature

Unified across containers/form/data/atoms:

```rust
pub(crate) fn render_card(
    el: &Element,
    spec: &Spec,
    data: &Value,
    depth: usize,
) -> String {
    // Typed props decode with diagnostic fallback (D-12).
    let props: CardProps = match serde_json::from_value(el.props.clone()) {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "<!-- ferro-json-ui: failed to decode Card props on element 'unknown': {e} -->"
            );
        }
    };

    // Body children: Element.children (D-05).
    let body: String = el
        .children
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    // Footer children: props.footer (D-06).
    let footer: String = props
        .footer
        .iter()
        .map(|cid| render_element(cid, spec, data, depth + 1))
        .collect();

    // Port v1 HTML emission verbatim, substituting `body` / `footer` for the old
    // `render_node(child, data)` calls.
    // ...
}
```

**Signature rationale:**
- `el: &Element` is needed because atom renderers also use `el.action.url` (for
  `Button` wrap-in-`<a>`) and we want the signature uniform.
- `spec: &Spec` required for `render_element` recursion.
- `depth` passes through for the tripwire.
- No `Walker` struct — three args is clear enough, and the signature is short.

**Top-level wrapper (preserves v1 byte contract):**

```rust
pub fn render_spec_to_html(spec: &Spec, data: &Value) -> String {
    // Top-level wrapper matches v1 render.rs line 39-41 exactly.
    let mut html = String::from(
        "<div class=\"flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto\">",
    );
    html.push_str(&render_element(&spec.root, spec, data, 1));
    html.push_str("</div>");
    html
}

pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult {
    let html = render_spec_to_html(spec, data);
    let plugin_types = collect_plugin_types(spec);
    if plugin_types.is_empty() {
        return RenderResult { html, css_head: String::new(), scripts: String::new() };
    }
    let type_names: Vec<String> = plugin_types.into_iter().collect();
    let assets = collect_plugin_assets(&type_names);
    RenderResult {
        html,
        css_head: render_css_tags(&assets.css),
        scripts: render_js_tags(&assets.js, &assets.init_scripts),
    }
}
```

## Public API Preservation

All existing public surface stays byte-identical in signature:

```rust
pub fn render_spec_to_html(spec: &Spec, data: &Value) -> String;
pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult;

pub struct RenderResult {
    pub html: String,
    pub css_head: String,
    pub scripts: String,
}
```

Re-exports in `lib.rs` (current lines 71–72):
```rust
pub use render::{render_spec_to_html, render_spec_to_html_with_plugins, RenderResult};
```
STAY IDENTICAL. The `render` module path shifts from `src/render.rs` to `src/render/mod.rs`
but Rust's `pub use` doesn't care.

**Byte-level HTML contract:**
- Top-level wrapper: `<div class="flex flex-wrap gap-4 [&>*]:w-full [&>button]:w-auto [&>a]:w-auto">…</div>`.
- Button `<a>` wrapping for GET actions (v1 render_node lines 262–286): preserved inside
  `atoms::render_button` or `render_element` — planner's call; preserving the exact wrap
  is load-bearing for gestiscilo navigation UX.
- All per-component class strings copied verbatim from v1.

## Action / Visibility / Data-path Integration

### Actions (`resolve::resolve_actions` → renderer)

1. **Pre-render:** `framework/src/json_ui/mod.rs::JsonUi::resolve` (line 39) already
   clones the Spec and calls `resolve_actions(&mut cloned, |h| crate::routing::route(h, &[]))`.
   Phase 116 does NOT change this wiring.
2. **Renderer read:** Per-component renderers that consume actions (`Button`, `Form`,
   `Switch` auto-form, `DataTable` row actions, `DropdownMenu`, `EmptyState`,
   `ActionCard`) read `Action.url` directly:
   ```rust
   let url = action.url.as_deref().unwrap_or_else(|| {
       // Diagnostic for unresolved actions (D-16).
       emit_diagnostic(&mut html, "unresolved action url");
       "#"
   });
   ```
3. **`href="#"` fallback:** Emit as literal `href="#"` + `<!-- ferro-json-ui: action
   '{handler}' has no resolved url -->` HTML comment.

### Visibility (`Visibility::evaluate` — TO BE ADDED)

`render_element` consults `el.visible` before dispatch:
```rust
if let Some(vis) = &el.visible {
    if !vis.evaluate(data) {
        return String::new();  // D-14: no output, no children walked.
    }
}
```

**Required new method** (does NOT currently exist):
```rust
impl Visibility {
    pub fn evaluate(&self, data: &Value) -> bool {
        match self {
            Visibility::And { and } => and.iter().all(|v| v.evaluate(data)),
            Visibility::Or { or } => or.iter().any(|v| v.evaluate(data)),
            Visibility::Not { not } => !not.evaluate(data),
            Visibility::Condition(c) => evaluate_condition(c, data),
        }
    }
}

fn evaluate_condition(c: &VisibilityCondition, data: &Value) -> bool {
    use crate::data::resolve_path;
    let resolved = resolve_path(data, &c.path);
    match c.operator {
        VisibilityOperator::Exists => resolved.is_some() && !resolved.unwrap().is_null(),
        VisibilityOperator::NotExists => resolved.is_none() || resolved.unwrap().is_null(),
        VisibilityOperator::NotEmpty => /* array non-empty OR string non-empty OR object non-empty */,
        VisibilityOperator::Empty => /* inverse of NotEmpty */,
        VisibilityOperator::Eq => resolved == c.value.as_ref(),
        VisibilityOperator::NotEq => resolved != c.value.as_ref(),
        VisibilityOperator::Gt | Lt | Gte | Lte => /* numeric compare */,
        VisibilityOperator::Contains => /* string substring OR array membership */,
    }
}
```

This should land in `visibility.rs` (not in `render/`), because `Visibility` is
currently defined there and the evaluation is a pure predicate. Allows any future caller
(not just the renderer) to reuse the logic.

**Edge cases to nail down in the plan:**
- `Eq`/`NotEq` with a path that doesn't resolve: treat as `not eq` (returns false for
  `Eq`, true for `NotEq`). Document behavior.
- Numeric comparisons on non-numeric values: return false rather than panic.
- `Contains` on arrays: match-by-value with `serde_json::Value::eq`.

**Root-hidden case (D-14):** If `spec.root`'s visibility evaluates false,
`render_spec_to_html` returns the wrapper div containing ONLY a root-hidden comment
(fallthrough from `render_element` returning `""`). Spec in CONTEXT wants the bare
comment; decide between "empty string" and "commented empty div" at plan time — a
middle-ground approach is:

```rust
let body = render_element(&spec.root, spec, data, 1);
if body.is_empty() {
    return format!("{}<!-- ferro-json-ui: root hidden -->{}", wrapper_open, wrapper_close);
}
```

### Data-path pre-fill (`data::resolve_path_string`)

Drop `#[allow(dead_code)]` in `data.rs` and use these helpers in:
- `render/form.rs::render_input` — `props.data_path` → pre-fill `value=""`.
- `render/form.rs::render_select` — `props.data_path` → selected option.
- `render/form.rs::render_checkbox` — `props.data_path` → `checked` attr.
- `render/form.rs::render_switch` — `props.data_path` → `checked` attr.
- `render/data.rs::render_table` — `props.data_path` → row array.
- `render/data.rs::render_data_table` — `props.data_path` → row array + `{id}`
  template in row action URLs.

Port the v1 resolver-use patterns verbatim (v1 render.rs lines ~1289+ for Input, ~1438+
for Select, ~1017+ for Table, ~1104+ for DataTable).

## Plugin Integration

### Dispatch default arm

```rust
fn render_plugin_or_unknown(type_name: &str, el: &Element, data: &Value) -> String {
    match with_plugin(type_name, |p| p.render(&el.props, data)) {
        Some(html) => html,
        None => format!(
            "<!-- ferro-json-ui: unknown component type '{}' -->",
            html_escape(type_name)
        ),
    }
}
```

### Plugin type collection (new flat pass)

Replaces v1's recursive `collect_plugin_types_node` (~95 LOC) with a ~15 LOC flat walk:

```rust
pub(crate) fn collect_plugin_types(spec: &Spec) -> HashSet<String> {
    let mut types = HashSet::new();
    for el in spec.elements.values() {
        if !BUILTIN_TYPES.contains(&el.type_name.as_str()) {
            // Non-builtin: it's a plugin (or unknown; collect_plugin_assets
            // silently ignores unregistered names per plugin.rs line 224).
            types.insert(el.type_name.clone());
        }
    }
    types
}
```

`collect_plugin_assets` in `plugin.rs` (lines 212–246) is UNCHANGED. It takes
`&[String]` of type names, looks each up in the registry, and returns
`CollectedAssets { css, js, init_scripts }`. Unregistered names are silently skipped
(line 224), so our flat collection can safely over-collect — e.g., if a hand-written
spec references `"Foo"` that's neither built-in nor plugin, the dispatch will emit an
unknown-type diagnostic and `collect_plugin_assets` will produce no assets for it.
Clean.

### Plugin asset tag emission

Port `render_css_tags` (v1 lines 200–221) and `render_js_tags` (v1 lines 222–249)
verbatim. These are already asset-agnostic; no changes needed for v2.

## Diagnostic HTML Comments Spec

Exact format strings per D-10, reported here to anchor consistency across 43 renderer
functions.

| Failure case | Emit (exact format string) | Emitted from |
|--------------|----------------------------|--------------|
| Missing child/slot ID in spec.elements | `<!-- ferro-json-ui: element references missing id '{id}' -->` | `render_element` ID lookup |
| Unknown `type_name` (not builtin, not plugin) | `<!-- ferro-json-ui: unknown component type '{type_name}' -->` | `render_plugin_or_unknown` fallback |
| Props deserialize failure | `<!-- ferro-json-ui: failed to decode {ComponentName} props: {err} -->` | Each per-component renderer's `serde_json::from_value` match arm |
| Cycle tripwire | `<!-- ferro-json-ui: cycle guard tripped at depth {depth} — spec should have been rejected at parse time -->` | `render_element` depth check |
| Root hidden | `<!-- ferro-json-ui: root hidden -->` | `render_spec_to_html` top-level, when `render_element(&spec.root, ...)` returns empty and root has a `visible` that evaluated false |
| Unresolved action URL | `<!-- ferro-json-ui: action '{handler}' has no resolved url -->` | Renderers consuming `Action.url` (Button wrap, Form, Switch, DataTable row actions, DropdownMenu, EmptyState, ActionCard) |
| Root missing from spec.elements | `<!-- ferro-json-ui: root '{id}' missing from spec -->` or empty — defensive; should never fire post-from_json | `render_spec_to_html` pre-walk |

**All messages HTML-escape interpolated identifiers.** Use the existing `html_escape`
from `render/mod.rs`. Never include raw user data (props content) in comments —
interpolate only IDs and type names, which pass the `^[A-Za-z_][A-Za-z0-9_-]{0,127}$`
validator and are XSS-safe. `err` from `serde_json::Error` has no untrusted content
from the spec (it reports the missing field name, not field values).

## Module Layout Proposal

```
ferro-json-ui/src/
├── render.rs              # DELETED — replaced by render/ directory
└── render/
    ├── mod.rs             # ~300 LOC — public API, BUILTIN_TYPES, dispatch match, walker,
    │                      #             collect_plugin_types, render_css_tags, render_js_tags,
    │                      #             html_escape, RenderResult struct
    ├── atoms.rs           # ~950 LOC — 22 leaf renderers (Text/Button/Badge/Alert/Separator/
    │                      #             Progress/Avatar/Image/Skeleton/Breadcrumb/EmptyState/
    │                      #             StatCard/Checklist/Toast/NotificationDropdown/Sidebar/
    │                      #             Header/DropdownMenu/CalendarCell/ActionCard/ProductTile/
    │                      #             Pagination)
    ├── containers.rs      # ~460 LOC — 9 containers (Card/Modal/Tabs/KanbanBoard/PageHeader/
    │                      #             Grid/Collapsible/FormSection/ButtonGroup)
    ├── form.rs            # ~480 LOC — 5 form controls (Form/Input/Select/Checkbox/Switch)
    └── data.rs            # ~370 LOC — 4 data displays (Table/DataTable/DescriptionList/
                           #             Pagination — consider moving Pagination from atoms.rs
                           #             to data.rs for semantic grouping; Claude's discretion)
```

**Total estimated LOC:** ~2560 across 5 files. All files <2000 LOC (atoms.rs is the
largest at ~950). Comfortable split.

**Visibility concern:** if `atoms.rs` hits ~1500 LOC during implementation (per-component
renderers often have helper functions inlined per v1), split dashboard-chrome components
(`Sidebar`, `Header`, `NotificationDropdown`, `DropdownMenu`) into a `render/chrome.rs`
file. Planner's call.

## Testing Plan

### Tests to port from v1 inline suite

v1's 201-test inline suite is dense with variant coverage (size xs/sm/default/lg as
separate tests, each variant of Button/Badge/Alert/Avatar/Skeleton etc. as separate
tests). Port list, consolidated to ~60 meaningful cases:

| Category | Count | Examples |
|----------|-------|----------|
| Text variants | ~7 | `p`, `h1`, `h2`, `h3`, `h4`, `span`, `div` element tags emit correct HTML tag |
| Button | ~10 | 6 variants × 4 sizes compressed to ~10 (default/destructive/outline/ghost + disabled + icon-left/right + size-xs/lg) |
| Badge | 4 | one per variant |
| Alert | 5 | 4 variants + title/no-title |
| Separator / Progress / Avatar / Skeleton / Image | ~12 | basic + edge cases (xss src, aspect ratios, fallback initials) |
| Card / Modal / Tabs / KanbanBoard / Grid / Collapsible / FormSection / PageHeader / ButtonGroup | ~12 | one smoke test per container covering body + slot(s); Tabs single-tab auto-hide |
| Form / Input / Select / Checkbox / Switch | ~10 | pre-fill via data_path; form wrapping for Switch with action; input_type variants; errors attr |
| Table / DataTable / DescriptionList / Pagination | ~8 | columns + empty message; row actions; URL-template {id}; base_url + paging |
| EmptyState / DropdownMenu / StatCard / Checklist / Toast / NotificationDropdown / Sidebar / Header / CalendarCell / ActionCard / ProductTile | ~11 | one smoke test per |

Inline these in `render/{atoms,form,data,containers}.rs` per-module `#[cfg(test)]`
sections — same style as v1. Helper fns for Spec construction go in each module's tests.

### New walker-specific tests (D-24)

Land in `render/mod.rs` `#[cfg(test)]` or a new `tests/walker.rs` integration file. At
minimum:

1. **Missing slot child** — `Card` with `footer: vec!["ghost".to_string()]` (ghost not
   in elements). Assert output contains `<!-- ferro-json-ui: element references missing
   id 'ghost' -->` and doesn't panic. (Note: this requires constructing Spec directly
   without going through `from_json` — Phase 115's validator doesn't inspect slot IDs
   per D-07, so `from_json` accepts it and the diagnostic fires at render time.)
2. **Unknown type_name** — Spec with `type: "ImaginaryWidget"` (not built-in, not
   plugin-registered). Assert diagnostic comment.
3. **Visibility hides element** — `Text` with `visible: Some(Condition { path: "/admin",
   operator: Eq, value: Some(json!(true)) })` and `data: {"admin": false}`. Assert empty
   output.
4. **Visibility hides children** — Parent `Card` with `visible: false`, child `Text`.
   Assert child is NOT in output.
5. **Root hidden** — Single-element spec, element has `visible: false`. Assert output
   contains `<!-- ferro-json-ui: root hidden -->`.
6. **Plugin dispatch** — Register a `TestPlugin`, build Spec with `type:
   "TestPlugin"`, assert render HTML includes plugin's output AND
   `collect_plugin_types(&spec)` returns `{"TestPlugin"}` AND
   `render_spec_to_html_with_plugins(&spec, ...).css_head` contains plugin's asset tags.
7. **Action URL inlined** — Spec with `Button` + `Action { handler: "users.create",
   url: Some("/users") }`. Assert `<a href="/users">` in output (per v1 wrap-GET
   semantics) or `<form action="/users">` depending on whether it's a Button vs.
   Form / Switch.
8. **Action URL unresolved** — `url: None`. Assert `href="#"` + diagnostic comment.
9. **Cycle tripwire** — Construct Spec by mutation after `build()` to bypass the
   validator (`spec.elements.insert("A", el_with_self_child_A)`). Assert render
   terminates with cycle diagnostic at depth 4.
10. **Props decode failure** — Element with `type: "Card"` but `props: json!(42)`
    (not a `CardProps` object). Assert render emits
    `<!-- ferro-json-ui: failed to decode Card props: ... -->` and doesn't panic.

### Integration test updates (D-25)

`framework/src/json_ui/mod.rs` tests currently assert on the placeholder marker
("v2 render pipeline arrives in Phase 116" at line 548 of the module tests). After
Phase 116:

- `render_produces_valid_html` (line 284) — KEEP 200 status + Doctype + data-view/props;
  ADD assertion for actual Card HTML markers (e.g., `<div class="rounded-lg border`
  copied from v1 render_card line ~770).
- `render_with_errors_populates_form_fields` (line 595) — KEEP; body will actually
  contain inline error messages rendered by the Input component (v1 render_input
  pattern) rather than just in `data-view` JSON.
- `test_plugin_component_renders_in_full_page` (line 966) — REMOVE `#[ignore]` and
  assert Leaflet CSS/JS in head + `data-ferro-map` container + `DOMContentLoaded` init
  script.

**Test infrastructure:** cargo workspace test runner (`cargo test --all-features`).
No new test framework, no new fixtures beyond optional inline goldens. Reuse the
existing `tests/fixtures/ok/` JSON corpus where useful.

## Validation Architecture

> `.planning/config.json` has no explicit `workflow.nyquist_validation` key. Treated as
> enabled; section included per gsd-plan-phase step 5.5.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)]` + `cargo test` (ferro workspace convention) |
| Config file | `Cargo.toml` per crate (no separate test runner) |
| Quick run command | `cargo test -p ferro-json-ui --lib` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| SC-1 | All 34 built-in types render from flat map | unit (per-component in each render/*.rs) | `cargo test -p ferro-json-ui --lib render::` | Wave 0 (new `render/` modules) |
| SC-2 | Missing child ID → diagnostic comment, no panic | unit | `cargo test -p ferro-json-ui --lib walker_missing_child` | Wave 0 |
| SC-3 | Action URL inlined when pre-resolved; fallback when None | unit | `cargo test -p ferro-json-ui --lib walker_action_url_inlined walker_action_url_unresolved` | Wave 0 |
| SC-4 | Visible=false skips element and children | unit | `cargo test -p ferro-json-ui --lib walker_visible_hides_element walker_visible_hides_children walker_root_hidden` | Wave 0 + `Visibility::evaluate()` added |
| SC-5 | Plugin dispatch + asset collection | unit + integration | `cargo test -p ferro-json-ui --lib walker_plugin_dispatch && cargo test -p ferro --lib json_ui::mod::tests::test_plugin_component_renders_in_full_page` | Wave 0 (un-ignore existing) |
| SC-6 | Old `render_to_html` absent | grep + cargo build | `! grep -rn "render_to_html\b" ferro-json-ui/src framework/src` (expect zero matches) | Verified by Phase 115; re-verified as guard |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui --lib` — local iteration, runs render
  module tests in seconds.
- **Per wave merge:** `cargo test --all-features -p ferro-json-ui -p ferro` — includes
  integration tests in the `framework` crate, ensures the wiring holds.
- **Phase gate:** full CI-matching command: `cargo fmt --all -- --check && cargo clippy
  --all --all-targets --all-features -- -D warnings && cargo test --all-features` (per
  CLAUDE.md project-root requirement). Must be green before `/gsd-verify-work`.

### Wave 0 Gaps

- [x] `ferro-json-ui/src/visibility.rs` — **ADD** `impl Visibility { pub fn evaluate(&self, data: &Value) -> bool }` — NEW; currently missing (blocks SC-4).
- [x] `ferro-json-ui/src/component.rs` — re-add 5 slot fields per D-06 (blocks SC-1
      multi-slot coverage).
- [x] `ferro-json-ui/src/render/mod.rs` — NEW file (replaces `src/render.rs`).
- [x] `ferro-json-ui/src/render/atoms.rs` — NEW file.
- [x] `ferro-json-ui/src/render/containers.rs` — NEW file.
- [x] `ferro-json-ui/src/render/form.rs` — NEW file.
- [x] `ferro-json-ui/src/render/data.rs` — NEW file.
- [x] `ferro-json-ui/src/data.rs` — drop `#[allow(dead_code)]` on `resolve_path` and
      `resolve_path_string` once consumed.
- [x] `framework/src/json_ui/mod.rs` — update integration tests to assert real HTML
      markers instead of placeholder marker; un-`#[ignore]` the Leaflet plugin test.
- [ ] (Optional) `ferro-json-ui/tests/walker.rs` — integration tests for walker-level
      behavior if inline `render/*` tests get too crowded. Pick at plan time.

## Risks & Caveats

### HIGH-1: COMPONENT_CATALOG entries for re-added slot fields

`ferro-json-ui/src/lib.rs` lines 88–168 define `pub const COMPONENT_CATALOG: &str`
consumed by ferro-mcp `json_ui_generate` and `code_templates` tools. Current entries
still document v1 shapes (`Card` entry at line 97 says `children (Vec<String>), footer
(Vec<String>)`; `Form` entry line 103 says `fields (Vec<String>)` — but v2's `FormProps`
no longer has a `fields` field).

**Decision required at plan time:** either (a) update the catalog entries for the 5
re-added slot fields (Card/Modal/Tab/KanbanColumn/PageHeader) to match the new Phase
116 shape — this is a ~30 LOC string edit — or (b) defer all catalog updates to Phase
117 which is rewriting COMPONENT_CATALOG into `Catalog::prompt()` anyway.

**Recommendation:** (a) — it's cheap, keeps the MCP catalog accurate between Phase 116
and Phase 117, and prevents confusion if anyone generates views via `json_ui_generate`
during the gap. Alberto can decide whether to spend the minutes.

### HIGH-2: `Visibility::evaluate()` does NOT currently exist

Phase 116 CONTEXT D-13 says "using the existing `Visibility::evaluate(&Value) -> bool`
semantics from `visibility.rs`". **This method does not exist in the codebase.** Neither
the v1 renderer (`git show 40385f32^:ferro-json-ui/src/visibility.rs`) nor the current
`visibility.rs` implements it. v1 renderer simply ignored the `visibility` field at
render time (it was parsed but never evaluated).

Phase 116 must therefore:
1. Add `impl Visibility { pub fn evaluate(&self, data: &Value) -> bool }` and a private
   `evaluate_condition` function covering all 11 `VisibilityOperator` variants.
2. Decide edge cases: missing path + `Eq` (false?), non-numeric + `Gt` (false?),
   `Contains` on scalar (false?). Document in rustdoc.
3. Ship tests for every operator — probably ~15 tests in `visibility.rs` inline.

**Impact on plan split:** the visibility work is small (~80 LOC + tests) but it's a
prerequisite for SC-4. Either include in 116-01 slot re-additions plan or split into a
dedicated 116-01.5 plan. Recommended: land in 116-01 alongside the slot fields since
both are low-level surface-area changes before the renderer itself.

### HIGH-3: framework/src/json_ui/mod.rs placeholder-marker assertions

Four existing integration tests in `framework/src/json_ui/mod.rs` currently assert on
specific strings that the placeholder emits:
- `render_with_errors_populates_form_fields` (line 594): asserts body contains
  `"Name is required"`. Passes because the placeholder prints the Spec JSON (including
  resolved errors) inside the `<pre>` dump. After Phase 116 the body will contain
  rendered Input HTML with `<p class="text-destructive">Name is required</p>` or similar
  per v1 render_input — the string match still works, but verify at plan time.
- `render_json_with_errors_includes_errors_in_response` (line 620): same,
  `"Name is required"` assertion. Robust to HTML change.
- `render_with_errors_empty_map_produces_no_errors` (line 641): asserts body does NOT
  contain `"Name is required"`. Robust.
- `test_plugin_component_renders_in_full_page` (line 966): `#[ignore]`d in Phase 115.
  Un-ignore in Phase 116 and confirm Leaflet markers.

Most integration tests are robust to the transition because they assert on
content-level strings (title, field labels, error messages) rather than on the
placeholder-specific `<pre>` or marker comment. Expect 1–2 adjustments, not a rewrite.

### MEDIUM-1: app/ sample page visual smoke

After Phase 115-03 the sample `app/` pages were migrated to `Spec::builder()` but
render as placeholder HTML. After Phase 116 they should render real HTML. No
`app/` page has been rebuilt-as-visual-target for v2, so a manual `cargo run -p app`
+ browser visit is a good smoke-check before the phase gate. Not required by the
6 success criteria, but if the gestiscilo field test in Phase 121 is going to be
meaningful, the sample `app/` is the smaller canary.

### MEDIUM-2: `data::resolve_path` / `resolve_path_string` `#[allow(dead_code)]`

`ferro-json-ui/src/data.rs` lines 18 and 54 still carry `#[allow(dead_code)]`. Phase
116 must drop them once the renderer consumes these helpers. If any one of the form
renderers gets missed, clippy `-D warnings` will flag it (after the `#[allow]` is
removed) and the CI gate will fail. Positive tripwire — not a risk, just a
plan-completion checkpoint.

### MEDIUM-3: Slot-reference validation gap (D-07 known limitation)

Phase 115's parser validates IDs appearing in `Element.children` but NOT in slot
fields (`CardProps.footer`, etc.) that get re-added by Phase 116. `from_json` will
accept a Card whose `footer` references a non-existent element ID. The walker's
runtime diagnostic (D-10) catches this at render time.

**Implication:** any Phase 116 test constructing a Spec via `from_json` with a
deliberately dangling slot ID will produce an HTML comment, not a `SpecError`. This is
correct per D-07/D-10 but is worth calling out in test expectations.

### LOW-1: `HashMap` iteration order non-determinism in `collect_plugin_types`

`spec.elements` is `HashMap<String, Element>`; iteration order is randomized per run.
`collect_plugin_types` returns a `HashSet<String>` and `collect_plugin_assets` takes a
`&[String]` — the order of CSS/JS `<link>`/`<script>` tags in the final HTML therefore
varies between runs. v1 had the same non-determinism via `HashSet`. Document in rustdoc
on `render_spec_to_html_with_plugins` that plugin asset order is unspecified.
Non-issue for correctness, potential issue for byte-for-byte snapshot testing
(mitigated by sorting asset URLs if needed).

## Plan Split Recommendation

Proposed 6-plan breakdown. Final slicing is the planner's call.

### 116-01 — Prep: Props slots + Visibility::evaluate

**Scope:** `ferro-json-ui/src/component.rs` (re-add 5 slot fields per D-06);
`ferro-json-ui/src/visibility.rs` (add `evaluate()` method + `evaluate_condition()`
helper); update `lib.rs` `COMPONENT_CATALOG` const entries for the 5 slot-added
components (HIGH-1 recommendation (a)); tests for both new surfaces.

**Scale:** ~200 LOC code + 15 inline tests + 5 catalog string lines.

**Why first:** every subsequent plan depends on the slot fields (containers) and on
`Visibility::evaluate` (walker). Small, de-risked, green-at-commit.

### 116-02 — Walker scaffolding

**Scope:** create `ferro-json-ui/src/render/` directory; write `render/mod.rs` with
public API stubs (`render_spec_to_html`, `render_spec_to_html_with_plugins`,
`RenderResult`), `BUILTIN_TYPES` const, `render_element` walker with dispatch match (all
arms stubbed to `String::new()` except plugin fallback), `collect_plugin_types`,
`render_css_tags`, `render_js_tags`, `html_escape`. Delete `src/render.rs`. Module
declarations for empty `atoms/containers/form/data` modules. Walker-level tests
(visibility hide, missing child, unknown type, cycle guard, plugin dispatch, root
hidden).

**Scale:** ~400 LOC including comments + ~10 walker tests.

**Gate:** workspace compiles with `BUILTIN_TYPES` match arms all returning empty
strings (plugins dispatch works). Walker tests pass.

### 116-03 — `render/atoms.rs` (22 leaf renderers)

**Scope:** port all atom-level renderers from v1 verbatim into `render/atoms.rs`; wire
each into the `render/mod.rs` dispatch match. Inline per-type smoke tests (~25 tests).

**Scale:** ~950 LOC + 25 tests. Largest plan.

**Consider splitting:** if session budget is tight, split 116-03a (primitives:
Text/Button/Badge/Alert/Separator/Progress/Avatar/Image/Skeleton) and 116-03b
(composite leaves: Breadcrumb/EmptyState/StatCard/Checklist/Toast/
NotificationDropdown/Sidebar/Header/DropdownMenu/CalendarCell/ActionCard/ProductTile/
Pagination).

### 116-04 — `render/containers.rs` (9 containers)

**Scope:** port Card, Modal, Tabs, KanbanBoard, PageHeader, Grid, Collapsible,
FormSection, ButtonGroup. Wire into dispatch. Containers exercise slot re-additions
from 116-01 and the `render_element` recursion from 116-02. Inline smoke tests (~12
tests including slot + children interaction).

**Scale:** ~460 LOC + 12 tests.

### 116-05 — `render/form.rs` + `render/data.rs`

**Scope:** port Form, Input, Select, Checkbox, Switch (form.rs, ~480 LOC); port Table,
DataTable, DescriptionList, Pagination (data.rs, ~370 LOC). Drop
`#[allow(dead_code)]` on `data::resolve_path` + `data::resolve_path_string`.
Inline tests (~18 tests total), covering data_path pre-fill, DataTable URL templating,
Table empty state, Form/Switch action wrapping.

**Scale:** ~850 LOC + 18 tests.

### 116-06 — Integration + phase gate

**Scope:** update `framework/src/json_ui/mod.rs` tests (strengthen placeholder
assertions to real HTML markers); un-`#[ignore]` `test_plugin_component_renders_in_full_page`
and confirm Leaflet assets; optional visual smoke of `cargo run -p app` (manual step
in a success-criteria checklist); run the full CI-matching command `cargo fmt --all --
--check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo
test --all-features` and confirm green.

**Scale:** ~10 test updates + execution checks. No new production code.

**Phase gate:** all 6 success criteria pass; workspace green with CI-matching command.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Visibility::evaluate` semantics for missing paths: `Eq` returns false, `NotEq` returns true | Action/Visibility/Data-path | Could produce inverted visibility logic in production specs; mitigated by writing operator-by-operator tests |
| A2 | Catalog string updates for re-added slots belong in Phase 116 (not Phase 117) | Risks HIGH-1 | If deferred, ferro-mcp catalog is briefly wrong between Phase 116 and 117 — no blocking impact |
| A3 | 201 v1 inline tests can be compressed to ~60 meaningful Phase 116 tests without losing coverage | Testing Plan | Could miss an edge case; v1 test list should be skimmed at plan time to confirm no material coverage gap |
| A4 | HashMap iteration order for plugin type collection is acceptable (non-deterministic asset tag order) | Risks LOW-1 | If gestiscilo relies on specific asset tag ordering for CSP or script interop, need deterministic sort. Mitigation is one `plugin_types.sort()` call |

All other claims are verified against source files (`ferro-json-ui/src/*.rs`,
`framework/src/json_ui/mod.rs`, `git show 40385f32^` references).

## Open Questions

1. **Should `Visibility::evaluate` land in Phase 116 or a sub-phase?**
   - What we know: It's load-bearing for SC-4 and doesn't currently exist.
   - What's unclear: Whether Alberto wants this surface reviewed separately.
   - Recommendation: Land in 116-01 alongside slot re-additions. Small, tested, done.

2. **Does the HIGH-1 catalog update happen in Phase 116 or 117?**
   - What we know: The catalog string is inaccurate for v1-stripped shapes already.
   - What's unclear: Whether Phase 117 will fully rewrite it (making Phase 116 edits
     throwaway).
   - Recommendation: Cheap edit, do it in 116-01. ~5 lines across 5 entries.

3. **Is per-plan granularity "fine" appropriate, or should 116-03 be split further?**
   - What we know: 116-03 is ~950 LOC of atom renderers.
   - What's unclear: Single-session executor budget for ports of this size.
   - Recommendation: Leave as one plan; if it bumps up against session budget during
     execution, split atom primitives (Text/Button/...) from atom composites
     (Sidebar/Header/...) at that point.

## Environment Availability

No external dependencies added or required by Phase 116. All new work is Rust code
within the existing workspace; toolchain version is Rust 1.88.0 (pinned in v1 commit
`2ce26241`). No new crate deps (HTML-comment diagnostics deliberately avoid `tracing`
per D-10). `cargo` is the test runner.

## Security Domain

> `.planning/config.json` has no `security_enforcement` key. Default treated as
> enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Auth is handled at framework routing layer; renderer emits markup only |
| V3 Session Management | no | Renderer is stateless |
| V4 Access Control | no | Visibility rules are presentation-layer, not security; never treat `visible: false` as a server-side access check |
| V5 Input Validation | yes | **All user-controlled strings in emitted HTML MUST go through `html_escape`** — this is THE load-bearing XSS mitigation. Applies to every prop field rendered as text content or attribute value |
| V6 Cryptography | no | No crypto surface |

### Known Threat Patterns for ferro-json-ui renderer

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via prop content echo | Tampering / Info disclosure | `html_escape` every interpolated string in text content and attribute values; port v1's discipline verbatim |
| XSS via `href` attribute in Action URL | Tampering | `html_escape` applied; also, Action URLs are resolved via `resolve_actions` with a trusted resolver (route names → server-owned paths) or a literal path prefix check — not user-controlled |
| XSS via `data-` attributes | Tampering | Layout system handles data-view/data-props escaping at the framework level (verified by `html_escaping_in_data_attributes` test in `framework/src/json_ui/mod.rs`); renderer additions must not break this |
| XSS via Image `src` | Tampering | v1 tested (`image_xss_src_escaped` at v1 render.rs line 3295); port the test |
| DoS via deep nesting | DoS | Depth tripwire at `MAX_NESTING_DEPTH + 1 = 4` (D-11); Phase 115's parse-time validator at depth 3 is the primary check |
| DoS via massive fan-out at one level | DoS | Not mitigated; renderer is O(n) and will happily emit 100k siblings. Acceptable for server-authored specs; if untrusted specs become a threat model, add a sibling-count cap |
| Path traversal via `data_path` | Info disclosure | `data::resolve_path` is a pure JSON-tree lookup; no filesystem or network access. Safe by construction |
| Plugin-emitted HTML XSS | Tampering | Plugin contract is `fn render(&self, props, data) -> String`; plugins are framework-level trusted code (registered at startup, not user-controlled). Threat model is "trust the registry" — documented in plugin.rs rustdoc |

**Net:** Phase 116's security surface is HTML escaping discipline. v1 got this right;
Phase 116 ports it verbatim. The CONTEXT already flagged XSS tests for `image_xss_src_escaped`
and `html_escaping_in_title`; make sure both get ported.

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/spec.rs` (Phase 115 shipped code) — element graph contract.
- `ferro-json-ui/src/component.rs` (current) — all 34 surviving `*Props` structs.
- `ferro-json-ui/src/action.rs` — `Action.url` is the pre-resolved URL field.
- `ferro-json-ui/src/visibility.rs` — `Visibility` enum (note: no `evaluate` method).
- `ferro-json-ui/src/data.rs` — `resolve_path` + `resolve_path_string` already exist,
  `#[allow(dead_code)]` to drop.
- `ferro-json-ui/src/plugin.rs` — `with_plugin`, `collect_plugin_assets`, `Asset`,
  `CollectedAssets`.
- `ferro-json-ui/src/render.rs` (current placeholder) — HTML escape helper to relocate.
- `ferro-json-ui/src/lib.rs` — public re-exports (line 71–72) and
  `COMPONENT_CATALOG` const.
- `ferro-json-ui/Cargo.toml` — confirmed no `tracing` / `log` deps.
- `ferro-json-ui/tests/round_trip.rs` + `tests/reject.rs` — existing Phase 115 test
  style.
- `framework/src/json_ui/mod.rs` — `JsonUi::render` wiring already correct; test updates
  needed.
- Git ref `40385f32^:ferro-json-ui/src/render.rs` — v1 renderer (8057 LOC), the
  port source. 43 `render_*` functions indexed in Section "v1 Renderer Inventory".
- Git ref `c88745a4^:ferro-json-ui/src/component.rs` — v1 Props with slot fields,
  exact v1 shape recorded for 5 multi-slot structs.
- `.planning/ROADMAP.md` Phase 116 section — goal + 6 success criteria.
- `.planning/phases/116-flat-element-renderer/116-CONTEXT.md` — full decision log.
- `.planning/phases/115-spec-v2-data-structures/115-VERIFICATION.md` — what Phase 115
  shipped (7/7).
- `CLAUDE.md` (project root) — Testing & Linting command, no co-author lines, commit
  discipline.

### Secondary (MEDIUM confidence)

- Vercel json-render design (flat map + root pointer + switch-by-type dispatch) — cited
  by Phase 115 CONTEXT research. Phase 116 adopts the dispatch pattern directly.

### Tertiary (LOW confidence)

- None; all load-bearing claims are verified against source.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, existing infrastructure.
- Architecture: HIGH — v1 renderer is the port source; shape is precisely known.
- Pitfalls: HIGH — all 4 risks cross-referenced to specific file:line locations.
- Visibility::evaluate gap: HIGH confidence the method is missing (grep across
  `ferro-json-ui/` and `git show 40385f32^` both return zero matches).
- Test coverage: MEDIUM — compression from 201 v1 tests to ~60 Phase 116 tests is a
  judgment call; Assumption A3 flags it.

**Research date:** 2026-04-18
**Valid until:** 2026-05-18 (30 days; stable v11.5 baseline, no upstream churn
expected).
