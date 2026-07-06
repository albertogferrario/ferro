# Phase 237: ActionGroup + DropdownMenu Replacement - Pattern Map

**Mapped:** 2026-06-22
**Files analyzed:** 10 new/modified files
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/component.rs` | model (props structs) | request-response | `DropdownMenuAction` :1059, `DropdownMenuProps` :1076 | exact |
| `ferro-json-ui/src/render/containers.rs` | renderer | request-response | `render_button_group` :946, `render_dropdown_menu` atoms.rs:1154 | exact |
| `ferro-json-ui/src/render/mod.rs` | registry/dispatch | request-response | `"DropdownMenu"` entry :64, dispatch arm :197 | exact |
| `ferro-json-ui/src/lib.rs` | export block | — | existing `pub use component::{…}` block :49 | exact |
| `ferro-json-ui/src/catalog.rs` | catalog spec + drift guards | — | `DropdownMenu` BUILTIN_SPECS tuple :241, drift guard test :1093 | exact |
| `ferro-json-ui/src/projection/builder.rs` | codegen | CRUD | `emit_actions_placeholder` :672 | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | mirror test | — | `expected[]` array :300, count assertion :292 | exact |
| `docs/src/json-ui/components.md` | docs | — | DropdownMenu section :985 | exact |
| `docs/src/features/projections.md` | docs | — | action routes table :504 | exact |
| `docs/src/json-ui/expressions.md` | docs | — | incidental mention :156 | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/component.rs` — ADD `ActionItem` + `ActionGroupProps`, DELETE `DropdownMenuProps`

**Analog:** `ferro-json-ui/src/component.rs:1059–1082`

**Analog: `DropdownMenuAction` struct** (lines 1057–1072 — template for `ActionItem`):
```rust
/// A single action item in a dropdown menu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DropdownMenuAction {
    pub label: String,
    pub action: Action,
    #[serde(default)]
    pub destructive: bool,
    /// When set, this item is only emitted in a DataTable row when the row's
    /// `visible_if` field is truthy (true / non-zero number / non-empty string /
    /// non-empty array or object). An absent or falsy field hides the item —
    /// fail-closed so a typo in the view spec cannot leak an action onto every
    /// row. Outside DataTable contexts (e.g. standalone `DropdownMenu` element)
    /// the field is ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_if: Option<String>,
}
```

**Analog: `DropdownMenuProps` struct** (lines 1074–1082 — template for `ActionGroupProps`):
```rust
/// Props for DropdownMenu component — trigger button with absolutely-positioned action panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DropdownMenuProps {
    pub menu_id: String,
    pub trigger_label: String,
    pub items: Vec<DropdownMenuAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_variant: Option<ButtonVariant>,
}
```

**Analog: schema-nonempty test pattern** (lines 1508–1516 — add analogous tests for ActionItem + ActionGroupProps):
```rust
#[test]
fn schema_for_dropdown_menu_action_generates() {
    assert_schema_nonempty_object::<DropdownMenuAction>("DropdownMenuAction");
}

#[test]
fn schema_for_dropdown_menu_props_generates() {
    assert_schema_nonempty_object::<DropdownMenuProps>("DropdownMenuProps");
}
```

**What to adapt:**
- `ActionItem` adds `variant: Option<ButtonVariant>` and `icon: Option<String>` vs `DropdownMenuAction` (which has neither). Keep `destructive`, `visible_if`, `label`, `action` identical.
- `ActionGroupProps` replaces `trigger_label: String` with `max_inline: Option<u8>` and `overflow_label: Option<String>`; keeps `menu_id: String`; adds `row_key: Option<String>`. `items` changes type to `Vec<ActionItem>`.
- Insert both structs after `ButtonGroupProps` (~line 936). The `DropdownMenuProps` struct (lines 1074–1082) is deleted; `DropdownMenuAction` (lines 1059–1072) is kept untouched.
- Replace the `schema_for_dropdown_menu_props_generates` test with two tests: `schema_for_action_item_generates` + `schema_for_action_group_props_generates`. Keep `schema_for_dropdown_menu_action_generates`.

---

### `ferro-json-ui/src/render/containers.rs` — ADD `render_action_group`

**Analog A:** `render_button_group` (containers.rs:946–963) — function signature and container shape:
```rust
pub(crate) fn render_button_group(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String {
    // Decode-check: malformed props surface via HTML comment rather than crash.
    if !el.props.is_null() {
        if let Err(e) = serde_json::from_value::<ButtonGroupProps>(el.props.clone()) {
            return format!(
                "<!-- ferro-json-ui: failed to decode ButtonGroup props: {} -->",
                html_escape(&e.to_string())
            );
        }
    }
    // ...
    format!("<div class=\"flex items-center gap-2 flex-wrap\">{body}</div>")
}
```

**Analog B:** `render_dropdown_menu` (atoms.rs:1154–1195) — kebab trigger + popover panel pattern (call this from `render_action_group`'s overflow path; do not re-implement):
```rust
pub(crate) fn render_dropdown_menu(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: DropdownMenuProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("DropdownMenu", e),
    };
    let mut html = String::new();

    let trigger_icon = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><circle cx=\"12\" cy=\"5\" r=\"1\"/><circle cx=\"12\" cy=\"12\" r=\"1\"/><circle cx=\"12\" cy=\"19\" r=\"1\"/></svg>";
    html.push_str(&format!(
        "<button type=\"button\" popovertarget=\"{}\" aria-label=\"{}\" \
         class=\"inline-flex items-center justify-center rounded-md p-1.5 \
         text-text-muted hover:text-text hover:bg-surface transition-colors duration-150 \
         focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary \
         focus-visible:ring-offset-2\">{}</button>",
        html_escape(&props.menu_id),
        html_escape(&props.trigger_label),
        trigger_icon,
    ));

    html.push_str(&format!(
        "<div popover id=\"{}\" data-popover-menu \
         class=\"w-48 rounded-md border border-border bg-card shadow-md text-left p-0\">",
        html_escape(&props.menu_id),
    ));

    for item in &props.items {
        html.push_str(&render_menu_item(
            item,
            "block px-4 py-2 text-sm text-text hover:bg-surface transition-colors duration-150",
            "block px-4 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors duration-150",
            "",
        ));
    }

    html.push_str("</div>"); // close popover panel
    html
}
```

**Analog C:** `render_menu_item` (atoms.rs:1073–1152) — the full non-GET form-wrapping branch (D-15; use this pattern for inline non-GET ActionItems):
```rust
pub(crate) fn render_menu_item(
    item: &DropdownMenuAction,
    normal_class: &str,
    destructive_class: &str,
    role_attr: &str,
) -> String {
    // url resolution ...
    match item.action.method {
        HttpMethod::Get => format!(
            "<a href=\"{}\"{} class=\"{}\"{}{}>{}</a>",
            html_escape(url), role_attr, class_attr, confirm_attrs, onclick,
            html_escape(&item.label),
        ),
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete => {
            let method_spoof = match item.action.method {
                HttpMethod::Put => Some("PUT"),
                HttpMethod::Patch => Some("PATCH"),
                HttpMethod::Delete => Some("DELETE"),
                _ => None,
            };
            let mut html = format!("<form action=\"{}\" method=\"post\">", html_escape(url));
            if let Some(m) = method_spoof {
                html.push_str(&format!(
                    "<input type=\"hidden\" name=\"_method\" value=\"{m}\">"
                ));
            }
            html.push_str(&format!(
                "<button type=\"submit\"{} class=\"w-full text-left {}\"{}{}>{}</button>",
                role_attr, class_attr, confirm_attrs, onclick,
                html_escape(&item.label),
            ));
            html.push_str("</form>");
            html
        }
    }
}
```

**Import additions needed in containers.rs** (extend line 14 import block):
- Add `ActionGroupProps, ActionItem` to the `crate::component` import
- Add `atoms::render_menu_item` (or call via `super::atoms::render_menu_item`) for the overflow path

**Implementation notes:**
- `render_action_group` decodes `ActionGroupProps` from `el.props` (same error-comment pattern as `render_button_group`).
- Partition items: non-destructive first (take `max_inline.unwrap_or(2)`) → inline; skip(max_inline) non-destructive + all destructive → overflow vec (destructive items appended last).
- Inline GET items: `<a href="..."><button>...</button></a>` or just `<a href="...">`.
- Inline non-GET items: extract the `<form action="..." method="post">` branch from `render_menu_item` above (adapt to `ActionItem` type, same method-spoof + confirm-attrs logic).
- Overflow (only when non-empty): emit the kebab trigger button + popover panel verbatim from the `render_dropdown_menu` pattern above, calling `atoms::render_menu_item` for each overflow item. Extract this into a private `render_overflow_kebab(menu_id, overflow_label, items: &[&ActionItem])` helper inside containers.rs — `render_menu_item` takes `&DropdownMenuAction`; the planner must decide whether to convert `ActionItem` to `DropdownMenuAction` or duplicate the menu-item render inline.
- Wrap everything in `<div class="flex items-center gap-2 flex-wrap">`.
- Place the function after `render_button_group` at ~line 965 (before `render_segmented_control`).

---

### `ferro-json-ui/src/render/mod.rs` — SWAP `BUILTIN_TYPES` entry + dispatch arm

**Analog: `BUILTIN_TYPES` atom entry at line 64 (to remove) + containers section (to add)**:
```rust
// REMOVE (line 64 — atoms section):
"DropdownMenu",

// ADD (after "ButtonGroup" + "SegmentedControl" + "SidebarLayout" in containers section, ~line 82):
"ActionGroup",
```

**Analog: dispatch arm at line 197 (to remove) + containers block (to add)**:
```rust
// REMOVE (line 197):
"DropdownMenu" => atoms::render_dropdown_menu(el, spec, data, depth),

// ADD (after "SidebarLayout" dispatch arm, ~line 215):
"ActionGroup" => containers::render_action_group(el, spec, data, depth),
```

**Pattern for containers dispatch block** (lines 203–215):
```rust
// Containers
"Card" => containers::render_card(el, spec, data, depth),
"Modal" => containers::render_modal(el, spec, data, depth),
"Tabs" => containers::render_tabs(el, spec, data, depth),
"KanbanBoard" => containers::render_kanban_board(el, spec, data, depth),
"PageHeader" => containers::render_page_header(el, spec, data, depth),
"DetailPage" => containers::render_detail_page(el, spec, data, depth),
"Grid" => containers::render_grid(el, spec, data, depth),
"Collapsible" => containers::render_collapsible(el, spec, data, depth),
"FormSection" => containers::render_form_section(el, spec, data, depth),
"ButtonGroup" => containers::render_button_group(el, spec, data, depth),
"SegmentedControl" => containers::render_segmented_control(el, spec, data, depth),
"SidebarLayout" => containers::render_sidebar_layout(el, spec, data, depth),
```

**Critical:** Order in `BUILTIN_TYPES` array must exactly match order in `BUILTIN_SPECS` (the runtime drift guard at catalog.rs:576 enforces this). Add `"ActionGroup"` in the containers section of both arrays at the same relative position.

---

### `ferro-json-ui/src/lib.rs` — SWAP `pub use component::{…}` export block

**Analog: current export block** (lines 49–63):
```rust
pub use component::{
    ActionCardProps, ActionCardVariant, AlertProps, AlertVariant, AvatarProps, BadgeProps,
    BadgeVariant, BreadcrumbItem, BreadcrumbProps, ButtonGroupProps, ButtonProps, ButtonType,
    ButtonVariant, CardProps, CardVariant, CheckboxListProps, CheckboxProps, ChecklistItem,
    ChecklistProps, CollapsibleProps, Column, ColumnFormat, DataTableProps, DescriptionItem,
    DescriptionListProps, DropdownMenuAction, DropdownMenuProps, EmptyStateProps, FormMaxWidth,
    FormProps, FormSectionProps, GapSize, GridProps, HeaderProps, IconPosition, ImageProps,
    InputProps, InputType, KanbanBoardProps, KanbanColumnProps, ModalProps,
    NotificationDropdownProps, NotificationItem, Orientation, PageHeaderProps, PaginationProps,
    ProductTileProps, ProgressProps, RawHtmlProps, RichTextEditorProps, SegmentedControlProps,
    SegmentedItem, SelectOption, SelectProps, SeparatorProps, SidebarGroup, SidebarLayoutItem,
    SidebarLayoutProps, SidebarNavItem, SidebarProps, Size, SkeletonProps, SortDirection,
    StatCardProps, SwitchProps, Tab, TableProps, TabsProps, TextElement, TextProps, ToastProps,
    ToastVariant,
};
```

**Changes:** Remove `DropdownMenuProps` from this list. Add `ActionGroupProps, ActionItem` (alphabetically: before `ActionCardProps` or after `ActionCardVariant` depending on sorted position — `ActionGroup` sorts before `ActionCard` alphabetically, so `ActionGroupProps, ActionItem` come before `ActionCardProps`). Keep `DropdownMenuAction` (D-11: the struct is retained for DataTable/Kanban).

---

### `ferro-json-ui/src/catalog.rs` — SWAP `BUILTIN_SPECS` entry + import, update drift guard comment

**Analog: `DropdownMenu` BUILTIN_SPECS tuple** (lines 240–245 — template for `ActionGroup` entry):
```rust
(
    "DropdownMenu",
    "Trigger button with an absolutely-positioned kebab-style action panel.",
    || to_value(schema_for!(DropdownMenuProps)).unwrap(),
    &[],
),
```

**Analog: import block** (lines 29–38 — remove `DropdownMenuProps`, add `ActionGroupProps, ActionItem`):
```rust
use crate::component::{
    ActionCardProps, AlertProps, AvatarProps, BadgeProps, BreadcrumbProps, ButtonGroupProps,
    ButtonProps, CalendarCellProps, CardProps, CheckboxListProps, CheckboxProps, ChecklistProps,
    CollapsibleProps, DataTableProps, DescriptionListProps, DetailPageProps, DropdownMenuProps,
    EmptyStateProps, FormProps, FormSectionProps, GridProps, HeaderProps, ImageProps, InputProps,
    KanbanBoardProps, MediaCardGridProps, ModalProps, NotificationDropdownProps, PageHeaderProps,
    PaginationProps, ProductTileProps, ProgressProps, RawHtmlProps, SegmentedControlProps,
    SelectProps, SeparatorProps, SidebarLayoutProps, SidebarProps, SkeletonProps, StatCardProps,
    StreamTextProps, SwitchProps, TableProps, TabsProps, TextProps, ToastProps,
};
```

**Analog: runtime drift guard** (line 576 — relational, passes automatically; no change needed):
```rust
if BUILTIN_SPECS.len() != crate::render::BUILTIN_TYPES.len() {
    return Err(CatalogError::BuildFailed(format!(
        "BUILTIN_SPECS has {} entries but BUILTIN_TYPES has {} — ...",
        BUILTIN_SPECS.len(),
        crate::render::BUILTIN_TYPES.len(),
    )));
}
```

**Analog: `builtin_types_count_drift_guard` test** (lines 1093–1101 — update history comment only, assertion stays 47):
```rust
#[test]
fn builtin_types_count_drift_guard() {
    // SINGLE source of truth for the absolute builtin-component count. When
    // BUILTIN_TYPES changes, update the number HERE and nowhere else — every
    // other test asserts its invariant relationally (against
    // BUILTIN_TYPES.len()), so a component addition breaks only this test.
    // History: 39 → 40 (CheckboxList) → 42 (DetailPage) → 43 (CheckboxGroup)
    // → 44 (MediaCardGrid) → 45 (StreamText) → 47 (SegmentedControl, SidebarLayout).
    assert_eq!(crate::render::BUILTIN_TYPES.len(), 47);
}
```

**Changes:** Append `→ 47 (DropdownMenu replaced by ActionGroup)` to the history comment. The `assert_eq!(..., 47)` stays unchanged (one-for-one swap). Remove `DropdownMenuProps` from the import block, add `ActionGroupProps, ActionItem`. The catalog.rs also has a DataTable description at ~line 404 that says "per-row DropdownMenu" — update to "per-row dropdown" or "per-row action menu".

---

### `ferro-json-ui/src/projection/builder.rs` — MIGRATE `emit_actions_placeholder` + test + import

**Analog: import block** (lines 25–31 — swap `DropdownMenuProps` → `ActionGroupProps`):
```rust
use crate::component::{
    CardProps, CardVariant, Column, DataTableProps, DescriptionItem, DescriptionListProps,
    DropdownMenuAction, DropdownMenuProps, FormProps, KanbanBoardProps, KanbanColumnProps,
    StatCardProps, Tab, TableProps, TabsProps,
};
```

**Analog: `emit_actions_placeholder` function** (lines 667–699 — migrate from DropdownMenu to ActionGroup):
```rust
/// `actions` slot. Emits a single `DropdownMenu` element carrying one item
/// per `ServiceDef.action`. ...
fn emit_actions_placeholder(
    service: &ServiceDef,
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) {
    if service.actions.is_empty() {
        return;
    }
    let items: Vec<DropdownMenuAction> = service
        .actions
        .iter()
        .map(|a| DropdownMenuAction {
            label: a.display_name.as_deref().unwrap_or(&a.name).to_string(),
            action: Action::new(format!("/{}/{}", service.name, a.name)),
            destructive: false,
            visible_if: None,
        })
        .collect();
    let props = serde_json::to_value(DropdownMenuProps {
        menu_id: format!("actions_{}", service.name),
        trigger_label: "Actions".to_string(),
        items,
        trigger_variant: None,
    })
    .expect("DropdownMenuProps serialization cannot fail");
    let id = "actions_menu".to_string();
    aux.push((id.clone(), element_with_props("DropdownMenu", props)));
    children_out.push(id);
}
```

**Analog: corresponding test** (lines 1220–1237):
```rust
#[test]
fn actions_slot_emits_dropdown_from_service_actions() {
    use crate::component::DropdownMenuProps;
    let service = service_with_actions();
    let mut aux: Vec<(String, ElementBuilder)> = Vec::new();
    let mut children: Vec<String> = Vec::new();
    emit_actions_placeholder(&service, &mut aux, &mut children);
    assert_eq!(children, vec!["actions_menu".to_string()]);
    let pos = aux
        .iter()
        .position(|(id, _)| id == "actions_menu")
        .expect("DropdownMenu must be emitted");
    let (_, el) = aux.remove(pos);
    let built = el.build();
    let props: DropdownMenuProps =
        serde_json::from_value(built.props).expect("props decode as DropdownMenuProps");
    assert_eq!(props.items.len(), service.actions.len());
    assert_eq!(props.items[0].label, "View");
}
```

**Changes:**
- Import: replace `DropdownMenuProps` with `ActionGroupProps` (keep `DropdownMenuAction` — still used by DataTable emitters at lines 300 and 463).
- Function body: change `Vec<DropdownMenuAction>` → `Vec<ActionItem>` for the `items` local; change `ActionItem` struct fields to add `variant: None, icon: None`; change `DropdownMenuProps { ... }` → `ActionGroupProps { items, menu_id, max_inline: None, overflow_label: None, row_key: None }`; change `element_with_props("DropdownMenu", ...)` → `element_with_props("ActionGroup", ...)`.
- Test: rename function to `actions_slot_emits_action_group_from_service_actions`; swap the `use` import to `ActionGroupProps`; replace the `.expect("DropdownMenu must be emitted")` comment; decode as `ActionGroupProps`. The `props.items.len()` and label assertions remain identical.
- Update the doc comment on `emit_actions_placeholder` to reference `ActionGroup`.

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` — SWAP name list + fix pre-existing 45-vs-47 gap

**Analog: `test_all_components_present` test** (lines 285–349):

Count assertion at lines 292–297 (stays 47 — no change to the number):
```rust
assert_eq!(
    catalog.components.len(),
    47,
    "Catalog should contain all 47 built-in components (incl. SegmentedControl, SidebarLayout), got {}",
    catalog.components.len()
);
```

`expected[]` array at lines 300–346 (currently 45 entries — missing `"SegmentedControl"` and `"SidebarLayout"`, has `"DropdownMenu"`):
```rust
let expected = [
    "Text", "Button", "Card", "Table", "Form", "Input", "Select", "Alert", "Badge",
    "Modal", "Checkbox", "CheckboxList", "CheckboxGroup", "Switch", "Separator",
    "DescriptionList", "Tabs", "Breadcrumb", "Pagination", "Progress", "Avatar",
    "Skeleton", "StatCard", "Checklist", "Toast", "NotificationDropdown", "Sidebar",
    "Header", "Grid", "Collapsible", "EmptyState", "FormSection", "PageHeader",
    "ButtonGroup", "DropdownMenu", "DataTable", "KanbanBoard", "CalendarCell",
    "ActionCard", "ProductTile", "RawHtml", "StreamText", "Image", "DetailPage",
    "MediaCardGrid",
    // Missing: "SegmentedControl", "SidebarLayout"  ← pre-existing bug
];
```

**Changes (both in one edit):**
1. Remove `"DropdownMenu"` from the array.
2. Add `"ActionGroup"`, `"SegmentedControl"`, `"SidebarLayout"` to the array.
3. Array grows from 45 → 47 entries (net: -1 +3 = +2 new, matching actual catalog count).
4. The count assertion at line 292 stays `47` — unchanged.
5. Update the inline comment string from `"incl. SegmentedControl, SidebarLayout"` to `"incl. SegmentedControl, SidebarLayout, ActionGroup"` if desired.

---

### `docs/src/json-ui/components.md` — REPLACE DropdownMenu with ActionGroup

**Analog: category table row** (line 29):
```markdown
| **Forms** | Form, Input, Select, Checkbox, CheckboxList, CheckboxGroup, Switch, Button, ButtonGroup, DropdownMenu |
```
Change `DropdownMenu` → `ActionGroup`.

**Analog: DropdownMenu section** (lines 985–1014):
```markdown
### DropdownMenu

A button that opens a dropdown with action items. Useful for per-row table actions.

| Prop | Type | Description |
|------|------|-------------|
| `label` | `string` | Trigger button label |
| `actions` | `array` | Action items (see below) |

Each action object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Menu item text |
| `handler` | `string` | Route handler name |
| `method` | `string` | HTTP method |
| `variant` | `string \| null` | `"destructive"` for danger actions |

```json
"row_actions": {
  "type": "DropdownMenu",
  "props": {
    "label": "Actions",
    "actions": [
      { "label": "View Details", "handler": "orders.show", "method": "GET" },
      { "label": "Delete", "handler": "orders.destroy", "method": "DELETE", "variant": "destructive" }
    ]
  }
}
```
```

Replace this entire section with an `### ActionGroup` section documenting the new props (D-05/D-06): `items`, `menu_id` (required), `max_inline` (default 2), `overflow_label` (default "Azioni"), `row_key`; each item: `label`, `action`, `destructive` (default false), `variant`, `icon`, `visible_if`. Include a JSON example showing inline buttons + overflow kebab behavior.

---

### `docs/src/features/projections.md` — UPDATE action routes table

**Analog: line 504**:
```markdown
| Page-level action | `/{service.name}/{action.name}` | Emitted as a `Button` or `DropdownMenu` item |
```

Change `DropdownMenu item` → `ActionGroup item`.

---

### `docs/src/json-ui/expressions.md` — UPDATE incidental mention

**Analog: line 156**:
```markdown
The pattern shows up when a Card and its Badge / DropdownMenu children all iterate over the same source array — each card's badge and dropdown belong to the same row.
```

Change `DropdownMenu` → `ActionGroup` (or make it generic: "… its Badge / action-group children …").

---

### `Cargo.toml` — workspace version bump

**Analog: line 46**:
```toml
[workspace.package]
version = "0.2.72"
```

Change to `version = "0.2.73"`. This is the workspace-level version; all crates inherit via `version.workspace = true` — no per-crate Cargo.toml edits needed.

---

## Shared Patterns

### Props struct conventions
**Source:** `ferro-json-ui/src/component.rs:1058–1082`
**Apply to:** `ActionItem`, `ActionGroupProps`
- Derives: `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema` (D-07).
- `bool` fields with `false` default: `#[serde(default)]`, no `skip_serializing_if`.
- `Option<T>` fields: `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- Required fields (`label`, `action`, `menu_id`): no serde annotations beyond the struct-level derive.

### Render function signature
**Source:** `ferro-json-ui/src/render/containers.rs:946`, `atoms.rs:1154`
**Apply to:** `render_action_group`
```rust
pub(crate) fn render_action_group(el: &Element, spec: &Spec, data: &Value, depth: usize) -> String
```
All render functions share this exact four-argument signature. `depth` is passed but ActionGroup does not recurse into children — pass `depth + 1` to any child render calls.

### Decode-failure diagnostic comment
**Source:** `ferro-json-ui/src/render/containers.rs:948–955`
**Apply to:** `render_action_group` props decode
```rust
if let Err(e) = serde_json::from_value::<ActionGroupProps>(el.props.clone()) {
    return format!(
        "<!-- ferro-json-ui: failed to decode ActionGroup props: {} -->",
        html_escape(&e.to_string())
    );
}
```

### Kebab trigger + popover panel HTML
**Source:** `ferro-json-ui/src/render/atoms.rs:1166–1193`
**Apply to:** `render_action_group` overflow path
- Trigger button: `popovertarget="{menu_id}"`, `aria-label="{overflow_label}"`, classes `inline-flex items-center justify-center rounded-md p-1.5 text-text-muted hover:text-text hover:bg-surface transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2`, three-dot SVG icon.
- Popover panel: `popover id="{menu_id}" data-popover-menu`, classes `w-48 rounded-md border border-border bg-card shadow-md text-left p-0`.
- Menu item classes: normal `block px-4 py-2 text-sm text-text hover:bg-surface transition-colors duration-150`; destructive `block px-4 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors duration-150`.

### BUILTIN_TYPES / BUILTIN_SPECS lockstep invariant
**Source:** `ferro-json-ui/src/catalog.rs:576`, `render/mod.rs:43`
**Apply to:** every edit that touches either array
Add `"ActionGroup"` to `BUILTIN_TYPES` and the corresponding tuple to `BUILTIN_SPECS` in the same commit. Remove `"DropdownMenu"` from both in the same commit. The runtime guard panics on mismatch; the `builtin_specs_len_matches_dispatch` test (catalog.rs:1104) catches divergence at test time.

---

## KEEP (no change required — internal helpers)

| Item | File | Lines | Reason |
|------|------|-------|--------|
| `render_menu_item` | `render/atoms.rs` | :1073–1152 | Internal helper reused by ActionGroup overflow and DataTable/Kanban |
| `render_inline_dropdown` | `render/data.rs` | :520 | DataTable/Kanban per-row kebab — unchanged |
| `DropdownMenuAction` struct | `component.rs` | :1059–1072 | Still used by `DataTableProps.row_actions` (:1091), `KanbanBoardProps.row_actions` (:1194), `MediaCardGridProps.row_actions` (:1135) |
| `action_visible_for_row` | `render/data.rs` | :445 | Internal — unchanged |
| `template_actions` | `render/data.rs` | :467 | Internal — unchanged |
| atoms.rs import of `DropdownMenuAction, DropdownMenuProps` | `render/atoms.rs` | :15 | `render_dropdown_menu` still references `DropdownMenuProps`; if `render_dropdown_menu` becomes dead code at phase end, clean up then |

### Dead-code question for `render_dropdown_menu`
After the dispatch arm removal, `render_dropdown_menu` (atoms.rs:1154) has no public call site. DataTable uses `render_inline_dropdown` (data.rs:520), not `render_dropdown_menu`. ActionGroup's overflow path will call the lower-level building blocks directly. Compiler `-D warnings` will catch the unused function. **Recommendation:** delete `render_dropdown_menu` at the end of this phase after the unit tests that test it (`dropdown_menu_emits_actions` / `dropdown_menu_get_action_renders_anchor` at atoms.rs:2018–2064) are removed. The "no deprecated code" principle applies.

---

## Atoms.rs Test Deletions

**Source:** `ferro-json-ui/src/render/atoms.rs:2018–2064`
**Action:** Delete both tests (`dropdown_menu_emits_actions` and `dropdown_menu_get_action_renders_anchor`). Replace with new `render_action_group` tests in `containers.rs` covering the behaviors specified in RESEARCH.md validation table (SC-1 through SC-3).

Test anatomy to copy from the deleted tests:
```rust
#[test]
fn dropdown_menu_emits_actions() {
    let spec = spec_with_root(
        Element::new("DropdownMenu")
            .prop("menu_id", "m1")
            .prop("trigger_label", "Actions")
            .prop("items", json!([{
                "label": "Edit",
                "action": {"handler": "edit", "method": "POST", "url": "/edit"}
            }])),
    );
    let el = spec.elements.get("root").unwrap();
    let html = render_dropdown_menu(el, &spec, &json!({}), 1);
    assert!(html.contains("Edit"), "got: {html}");
    assert!(html.contains("popovertarget=\"m1\""), "got: {html}");
    assert!(html.contains("popover id=\"m1\" data-popover-menu"), "got: {html}");
}
```

New `render_action_group` tests should follow the same `spec_with_root` + `el.props.get("root")` + `render_action_group(el, &spec, &json!({}), 1)` pattern.

---

## No Analog Found

None — every file in scope has a verified analog in the existing codebase.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/`, `ferro-mcp/src/`, `docs/src/`, `Cargo.toml`
**Files read:** 14 source files (all line numbers verified against workspace 0.2.72)
**Pattern extraction date:** 2026-06-22
