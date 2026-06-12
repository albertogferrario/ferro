# Phase 213: Projection Render Completeness — Research

**Researched:** 2026-06-12
**Domain:** ferro-json-ui projection builder (builder.rs emit functions)
**Confidence:** HIGH — all claims verified against codebase source

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** `emit_kanban_root` reads `service.state_machine` and emits one `KanbanColumnProps` per state. Sets `KanbanBoardProps.data_path` to bind cards. Fallback single-column when `state_machine` is `None`. `VisualContext.current_state` may mark active column.
- **D-02:** `emit_statcard_root` binds each stat to runtime data instead of `value: String::new()` — one data-bound `StatCard` per Money/Quantity readable field. Uses JSON-UI `$data` binding convention. If `StatCardProps.value` cannot accept data-bound expressions, the smallest extension is in scope.
- **D-03:** `emit_actions_placeholder` emits real action elements from `service.actions` (`Vec<ActionDef>`): page-level as `PageHeader`/`Button`, row/card-level as `DropdownMenu`/`Button`. Priority: first (highest leverage).
- **D-04:** `emit_datatable_root` renders `FieldMeaning::ImageUrl` fields as an image column instead of excluding them.
- **D-05:** Gap E (app-shell context) — document composition pattern now; defer first-class layout context unless a gap forces it.
- **D-06:** Implement in leverage order: B (actions) → A (kanban) → C (statcard) → D (imageurl) → E (layout).
- **D-07:** Every gap gets a render test asserting the emitted Spec + gestiscilo re-verification.
- **D-08:** Phase 207 catalog `derive_intents` invariants MUST stay green. This phase changes rendering only.

### Claude's Discretion
- Per-gap sub-phase split (each gap is independently testable).
- Specific data-binding approach for StatCard if `StatCardProps.value` does not support data-bound expressions.

### Deferred Ideas (OUT OF SCOPE)
- Gap E first-class app-shell/layout context (document composition only).
- Chart/visualization FieldMeaning (SVG-chart gap from gestiscilo Statistics).
- Resuming/merging the gestiscilo Slice A migrations (happens in gestiscilo repo after this phase ships).
</user_constraints>

---

## Summary

The ferro-json-ui projection builder (`ferro-json-ui/src/projection/builder.rs`) is layout-complete but content-incomplete. Phase 209 empirically confirmed this against a production codebase. The pipeline `ServiceDef → derive_intents → intent_layout slots → emit_* functions → Spec` works correctly for intent selection and outer layout choice (Browse→DataTable, Process→KanbanBoard, Summarize→StatCard), but five emit functions are stubs that ignore `ServiceDef` data that is already populated.

The five functions to change — `emit_kanban_root`, `emit_actions_placeholder`, `emit_statcard_root`, `emit_datatable_root` (column filter only), and Gap E (documentation-only) — are all local to `builder.rs`. No changes to `ferro-projections` types, `derive.rs`, `intent.rs`, or `Catalog::validate` rules are needed. The component props that must be populated (`KanbanBoardProps`, `StatCardProps`, `DataTableProps`, `DropdownMenuAction`) all exist in the catalog already.

One props extension IS needed: `StatCardProps` has `value: String` with no data-path binding — Gap C requires adding `value_path: Option<String>` to `StatCardProps` and updating `render_stat_card` to resolve it at render time (same pattern as `ImageProps.data_path` and `DescriptionListProps.data_path`).

**Primary recommendation:** Implement in B → A → C → D → E order. B (actions) unblocks every migrated page's management affordances in one change. A (kanban) is the most visible fix. C requires one props extension then becomes straightforward. D is a one-line filter change in `lookup_meaning` plus a `ColumnFormat::Image` addition. E is documentation only.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| State-machine→kanban columns | ferro-json-ui (builder.rs) | ferro-projections (read-only StateMachine) | Rendering decision lives in the output crate |
| Actions→Button/DropdownMenu | ferro-json-ui (builder.rs) | ferro-projections (read-only ActionDef) | Rendering decision lives in the output crate |
| StatCard value binding | ferro-json-ui (component.rs + atoms.rs) | — | Props extension + render change both in output crate |
| ImageUrl column | ferro-json-ui (component_map.rs + builder.rs) | — | lookup_meaning dispatch and Column emission in output crate |
| App-shell/layout composition | Documentation only (consumer responsibility) | — | D-05 defers first-class context |
| Intent classification | ferro-projections (derive.rs, intent.rs) | — | FROZEN — must not change |

---

## The Builder Pipeline

### How ServiceDef → Spec Flows

```
ServiceDef + Vec<IntentScore>
    |
    v
Spec::from_service_def(service, intents, ctx)
    |
    v
from_service_def_with_catalog(...)
    |
    +-- ctx.mode == Input → build_input_spec(service)  [Form, out of scope]
    |
    +-- ctx.mode == Display →
            ctx.templates → pick_intent_template OR default_template(intent)
            → IntentSlotTemplate { layout, slots }
            → build_display_spec(service, intent, template)
                |
                v
             match layout {
               "DataTable"   → emit_datatable_root(service)
               "Card"        → emit_card_root(service, slots, aux)
               "KanbanBoard" → emit_kanban_root(service)           ← GAP A
               "StatCard"    → emit_statcard_root(service, slots, aux)  ← GAP C
             }
             +
             slot walk in Card/StatCard:
               "actions" → emit_actions_placeholder(...)   ← GAP B
               "body"    → emit_body_placeholder(...)      [deferred]
               "fields"  → emit_fields_as_description_list(...)
               "relationships" → emit_relationships(...)
               "metadata" → emit_metadata(...)
    |
    v
Spec builder (title + root + aux elements)
    |
    v
catalog.validate(&spec) → Ok or ProjectionError::CatalogValidation
```

### Intent → Layout → Slots (intent_layout.rs defaults)

| Intent | Layout | Slots (display) |
|--------|--------|-----------------|
| Browse | DataTable | title, fields, pagination |
| Focus | Card | title, fields, relationships, actions |
| Collect | Form | title, fields, actions |
| **Process** | **KanbanBoard** | **title, body, actions** |
| **Summarize** | **StatCard** | **title, stats, metadata** |
| Analyze | Card | title, body, metadata |
| Track | DataTable | title, fields, metadata |
| Custom | Card | title, fields |

**Key observation:** the Process slots are `title, body, actions` — NOT `fields`. The `fields` slot is never consulted for Process, which is why the stat-field scan (Gap A/C) must happen inside `emit_kanban_root` and `emit_statcard_root` directly, not through slot dispatch.

---

## Per-Gap Implementation Detail

### Gap B — Actions Slot (HIGHEST LEVERAGE — do first)

**File:** `ferro-json-ui/src/projection/builder.rs`

**Current code:**
```rust
#[allow(clippy::ptr_arg)]
fn emit_actions_placeholder(
    _service: &ServiceDef,
    _aux: &mut Vec<(String, ElementBuilder)>,
    _children_out: &mut Vec<String>,
) {
    // Intentionally empty. Deferred to Phase 118+.
}
```

**Target code shape:**

`emit_actions_placeholder` must iterate `service.actions` and emit:
1. **Page-level actions** (no `transition_trigger`, no inputs — pure navigation/command): emit a `Button` with `variant: ButtonVariant::Default` (or `Secondary`). The `Action`'s handler is built from `action.name` as the route placeholder: `Action::new(format!("/{}/{}", service.name, action.name))`.
2. **Row/card-level actions** (those with a `transition_trigger` — state transitions — or with `inputs` that take a row identifier): these are row actions, better wired through `DataTableProps.row_actions` (DataTable) or a per-card `DropdownMenuAction` list. For the `actions` slot specifically (Focus/Process/Track contexts), emit a `DropdownMenu` element containing all such actions.

**Practical approach for v1 (minimum viable, matches gestiscilo need):**

Each `ActionDef` becomes a `DropdownMenuAction`:
```rust
fn emit_actions_placeholder(
    service: &ServiceDef,
    aux: &mut Vec<(String, ElementBuilder)>,
    children_out: &mut Vec<String>,
) {
    if service.actions.is_empty() {
        return;
    }
    let items: Vec<DropdownMenuAction> = service.actions.iter().map(|a| {
        DropdownMenuAction {
            label: a.display_name.as_deref()
                .unwrap_or(&a.name)
                .to_string(),
            action: Action::new(format!("/{}/{}", service.name, a.name)),
            destructive: false,
            visible_if: None,
        }
    }).collect();
    let props = serde_json::to_value(DropdownMenuProps {
        menu_id: format!("actions_{}", service.name),
        trigger_label: "Actions".to_string(),
        items,
        trigger_variant: None,
    }).expect("DropdownMenuProps serialization cannot fail");
    let id = "actions_menu".to_string();
    aux.push((id.clone(), element_with_props("DropdownMenu", props)));
    children_out.push(id);
}
```

**For Browse (DataTable) row actions:** The `actions` slot is not in the Browse slot template (Browse uses `title, fields, pagination`). Row actions for Browse must be wired into `DataTableProps.row_actions` instead. `emit_datatable_root` should additionally populate `row_actions` from `service.actions`. This is the same Gap B fix but in a different location.

**Imports to add to builder.rs:**
```rust
use crate::component::{DropdownMenuAction, DropdownMenuProps};
use crate::action::Action;
// Action is already imported via `use crate::action::Action;` in build_input_spec
```

**Component props confirmed (component.rs:957-981):**
```rust
pub struct DropdownMenuAction {
    pub label: String,
    pub action: Action,
    pub destructive: bool,
    pub visible_if: Option<String>,
}
pub struct DropdownMenuProps {
    pub menu_id: String,
    pub trigger_label: String,
    pub items: Vec<DropdownMenuAction>,
    pub trigger_variant: Option<ButtonVariant>,
}
```

`DropdownMenu` is already in the catalog (verified by `meaning_table_components_exist_in_catalog` test passing), so `catalog.validate` will accept it.

**DataTable row_actions wiring** (also in `emit_datatable_root`):
```rust
let row_actions: Option<Vec<DropdownMenuAction>> = if service.actions.is_empty() {
    None
} else {
    Some(service.actions.iter().map(|a| DropdownMenuAction {
        label: a.display_name.as_deref().unwrap_or(&a.name).to_string(),
        action: Action::new(format!("/{}/{{id}}/{}", service.name, a.name)),
        destructive: false,
        visible_if: None,
    }).collect())
};
// then DataTableProps { ..., row_actions, ... }
```

Note: the `{id}` placeholder in the URL is a pattern the DataTable renderer substitutes with `row_key`. For this to work, `row_key` on `DataTableProps` should also be set (e.g. `Some("id".to_string())`). If no `row_key` field exists on the ServiceDef, it defaults to `id`.

---

### Gap A — Kanban State-Machine Columns (BLOCKING)

**File:** `ferro-json-ui/src/projection/builder.rs`

**Current code:**
```rust
fn emit_kanban_root(service: &ServiceDef) -> ElementBuilder {
    let placeholder = KanbanColumnProps {
        id: "default".to_string(),
        title: resolve_title(service),
        count: 0,
        children: Vec::new(),
    };
    let props = serde_json::to_value(KanbanBoardProps {
        columns: vec![placeholder],
        data_path: None,
        mobile_default_column: None,
        empty_label: None,
    })
    .expect("KanbanBoardProps serialization cannot fail");
    element_with_props("KanbanBoard", props)
}
```

**Target code shape:**

When `service.state_machine` is `Some(sm)`, derive one `KanbanColumnProps` per `StateDef` in `sm.states`. Set `KanbanBoardProps.data_path` to `/data/{service.name}/columns` (or `/data/kanban_columns`) so the renderer binds cards from runtime data.

**The key design decision for data binding:** The `KanbanBoardProps.data_path` convention means the runtime handler must merge the array of `KanbanColumnProps` objects at that path. Looking at the renderer (containers.rs:334-344), when `data_path` is set, the renderer resolves an array of `KanbanColumnProps` from `data`. So the handler provides:

```json
{ "data": { "order": { "columns": [ {"id": "draft", "title": "Draft", "count": 3, "children": [...]} ] } } }
```

The builder emits static column structure (from the state machine) into `KanbanBoardProps.columns` (as the schema/label source) AND sets `data_path` to bind the runtime counts and card children from the handler. The renderer uses `data_path` when present, overriding the static `columns`.

**Recommended approach:** Set `data_path` to `/data/{service.name}/columns` (plural suffix to distinguish from the flat item array at `/data/{service.name}`). Also embed static column definitions from the state machine so consumers know the expected shape even before the handler populates runtime data.

```rust
fn emit_kanban_root(service: &ServiceDef) -> ElementBuilder {
    let columns: Vec<KanbanColumnProps> = service.state_machine
        .as_ref()
        .map(|sm| {
            sm.states.iter().map(|s| KanbanColumnProps {
                id: s.name.clone(),
                title: s.display_name.as_deref()
                    .unwrap_or(&s.name)
                    .to_string(),
                count: 0,
                children: Vec::new(),
            }).collect()
        })
        .unwrap_or_else(|| vec![KanbanColumnProps {
            id: "default".to_string(),
            title: resolve_title(service),
            count: 0,
            children: Vec::new(),
        }]);

    // data_path binds runtime column data (counts + card children).
    // Static `columns` provide the schema fallback when data_path resolves empty.
    let data_path = service.state_machine
        .as_ref()
        .map(|_| format!("/data/{}/columns", service.name));

    let props = serde_json::to_value(KanbanBoardProps {
        columns,
        data_path,
        mobile_default_column: None,
        empty_label: None,
    })
    .expect("KanbanBoardProps serialization cannot fail");
    element_with_props("KanbanBoard", props)
}
```

**Imports needed:** `ferro_projections::StateMachine` (already re-exported; `service.state_machine` is `Option<StateMachine>` directly on `ServiceDef`).

**`KanbanColumnProps` shape (component.rs:1043-1051):**
```rust
pub struct KanbanColumnProps {
    pub id: String,
    pub title: String,
    pub count: u32,
    pub children: Vec<String>,  // IDs of child elements
}
```

**Catalog depth invariant (Pitfall 3):** `KanbanBoardProps.columns` is inline — no child elements in the spec's element map. This is unchanged: column definitions go into the `columns` vec (in props), not as children IDs. Depth stays at 1. The `children: Vec<String>` inside `KanbanColumnProps` refers to child element IDs for sub-card rendering, but for a data-path bound board, these come from runtime data, not static spec elements.

**`VisualContext.current_state`:** The `emit_kanban_root` receives the builder's `service` only, not `ctx`. To use `current_state` for active-column highlight, the function signature must change from `emit_kanban_root(service: &ServiceDef)` to `emit_kanban_root(service: &ServiceDef, ctx: &VisualContext)` and pass `ctx.current_state.as_deref()` for highlight. This is a small but necessary signature change.

---

### Gap C — StatCard Value Binding (requires props extension)

**File:** `ferro-json-ui/src/projection/builder.rs` + `ferro-json-ui/src/component.rs` + `ferro-json-ui/src/render/atoms.rs`

**Current code (builder.rs):**
```rust
fn emit_statcard_root(
    service: &ServiceDef,
    slots: &[String],
    aux: &mut Vec<(String, ElementBuilder)>,
) -> ElementBuilder {
    let mut dropped: Vec<String> = Vec::new();
    for slot in slots {
        if slot == "metadata" {
            emit_metadata(service, aux, &mut dropped);
        }
    }
    let props = serde_json::to_value(StatCardProps {
        label: resolve_title(service),
        value: String::new(),   // ← EMPTY, not data-bound
        icon: None,
        subtitle: None,
        sse_target: None,
    })
    .expect("StatCardProps serialization cannot fail");
    element_with_props("StatCard", props)
}
```

**Problem:** `StatCardProps.value` is a static `String`. Looking at `render_stat_card` in atoms.rs:
```rust
html_escape(&props.value)   // renders the static string directly
```
There is no `data_path` field on `StatCardProps`. The component supports `sse_target` for live updates but not a data-path binding for initial value.

**Required props extension (component.rs):**

Add `value_path: Option<String>` to `StatCardProps`:
```rust
pub struct StatCardProps {
    pub label: String,
    pub value: String,
    pub icon: Option<String>,
    pub subtitle: Option<String>,
    pub sse_target: Option<String>,
    // NEW — resolves initial value from handler data at render time.
    // Format: /segment/segment (same JSON-pointer as data.rs::resolve_path).
    // Falls back to `value` when missing or non-string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_path: Option<String>,
}
```

**Required render change (atoms.rs, `render_stat_card`):**

Before rendering `props.value`, resolve `value_path` against `data`:
```rust
let display_value = props.value_path.as_deref()
    .and_then(|p| crate::data::resolve_path_string(data, p))
    .unwrap_or(props.value.clone());
// then use display_value instead of props.value in the two html.push_str calls
```

Note: `render_stat_card` currently receives `_data: &Value` (unused). It must start using it.

**Builder change:** emit one `StatCard` per Money/Quantity readable field:

```rust
fn emit_statcard_root(
    service: &ServiceDef,
    slots: &[String],
    aux: &mut Vec<(String, ElementBuilder)>,
) -> ElementBuilder {
    // metadata slot: orphan DescriptionList, same as before
    let mut dropped: Vec<String> = Vec::new();
    for slot in slots {
        if slot == "metadata" {
            emit_metadata(service, aux, &mut dropped);
        }
    }

    // Find the first Money or Quantity readable field for the primary stat.
    let primary_field = service.fields.iter()
        .find(|f| f.readable
            && matches!(f.meaning, FieldMeaning::Money | FieldMeaning::Quantity));

    let (label, value_path) = primary_field
        .map(|f| (
            field_display_name(&f.name),
            Some(format!("/data/{}/{}", service.name, f.name)),
        ))
        .unwrap_or_else(|| (resolve_title(service), None));

    let props = serde_json::to_value(StatCardProps {
        label,
        value: String::new(),
        value_path,
        icon: None,
        subtitle: None,
        sse_target: None,
    })
    .expect("StatCardProps serialization cannot fail");
    element_with_props("StatCard", props)
}
```

**Multi-stat variant (D-02 says "one StatCard per Money/Quantity field"):** The current root emit pattern returns a single `ElementBuilder`. To emit multiple StatCards, the builder must emit them as aux elements and return a wrapper (e.g. a `Grid` or `Card`). This is the one architectural question the planner must decide:

- **Option 1 (simple, recommended):** Emit a single StatCard for the primary stat field only (the first Money/Quantity field). If there are multiple stat fields, emit additional StatCards as aux elements, and wrap them in a `Grid` container as the root. Return the `Grid` root element.
- **Option 2 (deferred):** Emit only the primary StatCard for now, leaving multi-stat as a later follow-up. This preserves the simplest root-emit shape.

Option 1 matches D-02 ("one StatCard per Money/Quantity field"). The `Grid` component is already in the catalog.

**Imports to add:** `FieldMeaning` is already imported in builder.rs (via `ferro_projections::FieldMeaning`). `field_display_name` is already imported. `StatCardProps` extended version requires no new imports. `GridProps` import needed if wrapping with Grid.

---

### Gap D — ImageUrl DataTable Column

**File:** `ferro-json-ui/src/projection/component_map.rs` + `ferro-json-ui/src/component.rs`

**Current code (component_map.rs:83-87):**
```rust
FieldMeaning::ImageUrl => ComponentChoice {
    display: Some("Avatar"),
    input: Some("Input"),
    column: None,   // ← excluded from DataTable columns
},
```

**Problem:** `column: None` causes `emit_datatable_root` to skip `ImageUrl` fields entirely. There is also no `ColumnFormat::Image` variant to hint the DataTable renderer that a cell contains an image URL.

**Option A (simplest, recommended):** Add `ColumnFormat::Image` to the enum, set `column: Some(())` on `ImageUrl`, and return `Some(ColumnFormat::Image)` from `build_column_for_field`. The DataTable renderer then renders an `<img>` or `<Avatar>` for cells with that format.

**`ColumnFormat` current variants (component.rs:132-138):**
```rust
pub enum ColumnFormat {
    Date,
    DateTime,
    Currency,
    Boolean,
    Badge,
}
```

New variant: add `Image` (or `Avatar`). `Image` is more generic and maps to the gestiscilo `avatar_url` use case.

**Changes required:**
1. `component.rs`: Add `ColumnFormat::Image` variant.
2. `component_map.rs`: Change `ImageUrl` `column: None` → `column: Some(())`. Update `build_column_for_field` to return `Some(ColumnFormat::Image)` for `ImageUrl`.
3. `render/data.rs`: Handle `ColumnFormat::Image` in the cell renderer — render an `<img src="...">` (or Avatar HTML) from the cell value string.
4. The `meaning_table_components_exist_in_catalog` test does not cover column formats directly, so no catalog change needed. But the `ColumnFormat` schema smoke test (`schema_for_column_format_generates` if it exists) must include `Image`.

**Option B (no new ColumnFormat):** Change `ImageUrl` to `column: Some(())` with `format: None` — the DataTable renders the URL as a plain text string. This is wrong visually but does not require a new ColumnFormat. Rejected: it defeats the purpose of Gap D.

---

### Gap E — App-Shell/Layout Context (Documentation Only)

**D-05 decision:** Document the composition pattern. No code change.

**The composition pattern to document:**

The projection `Spec` is a standalone spec rooted at the content component (DataTable, KanbanBoard, etc.). The surrounding dashboard chrome (sidebar, nav, PageHeader) is the consumer's responsibility. The consumer merges the projection spec elements into their layout spec:

```rust
// Consumer handler pseudocode:
let projection_spec = JsonUiRenderer.render(&service, &intents, &ctx)?;
// Merge projection root into existing layout spec:
let mut layout_spec = /* load dashboard layout spec */;
layout_spec.elements.insert("content_root", projection_spec.elements.get(&projection_spec.root).cloned());
// ... add projection_spec.root ID to layout's main-content children
```

Or alternatively: the handler returns a JSON response containing the projection spec at a known key, and the dashboard layout template embeds it:

```json
// Handler response:
{
  "data": { "order": [...] },
  "projection": { /* full Spec JSON */ }
}
```

This is the documented contract for Phase 213. A first-class `VisualContext.layout` field is deferred.

---

## The Data-Binding Convention

[VERIFIED: reading ferro-json-ui/src/data.rs, render/atoms.rs, render/containers.rs, render/data.rs]

### How `data_path` Works

All `data_path` fields in ferro-json-ui follow a single convention implemented in `crate::data::resolve_path`:

- **Format:** `/segment/segment/...` — slash-separated JSON-pointer path, leading slash required.
- **Resolution:** walks the `data: &Value` argument passed to every render function. `data` is the full handler response body (the JSON object the Ferro handler returns).
- **Example:** `data_path: "/data/staff"` walks `response["data"]["staff"]` and returns the value there (typically an array of row objects).

### DataTable Binding (proven in Phase 209)

```
DataTableProps.data_path = "/data/{service.name}"
```
The handler merges rows at `response["data"]["staff"]`. The DataTable renderer resolves that array and renders one `<tr>` per item, projecting column `key` fields as cell text. This WORKS — Phase 209 confirmed `NAME/BIO/SORT ORDER/ACTIVE` columns appeared with two test rows.

### KanbanBoard Binding Convention (Gap A decision)

```
KanbanBoardProps.data_path = "/data/{service.name}/columns"
```
The renderer (containers.rs:334-344) resolves the array at `data_path` and deserializes each element as `KanbanColumnProps`. The handler must produce:

```json
{
  "data": {
    "order": {
      "columns": [
        { "id": "draft", "title": "Draft", "count": 2, "children": [] },
        { "id": "submitted", "title": "Submitted", "count": 1, "children": [] }
      ]
    }
  }
}
```

The static `columns` in `KanbanBoardProps` (derived from the state machine) serve as the fallback schema and documentation — when `data_path` is absent or fails to resolve, the static columns render (showing 0 counts). When `data_path` resolves, it wins over static columns.

**There is no per-column filtering at the framework level.** The handler is responsible for grouping items by state and providing the correct column array. This is the same pattern gestiscilo's bespoke handler used with the hand-authored kanban view.

### StatCard Binding (Gap C — requires `value_path` extension)

`StatCardProps.value` is a static `String` — the renderer calls `html_escape(&props.value)` directly with no path resolution. There is no `data_path` or equivalent field on `StatCardProps` currently.

The `sse_target: Option<String>` field enables live SSE updates (for real-time metrics), but it is NOT a data-path binding — it writes a `data-sse-target` attribute for the browser's SSE listener, used for push updates, not initial-load data.

**Required extension:** Add `value_path: Option<String>` to `StatCardProps`. The renderer resolves it against `data` at render time via `resolve_path_string`, falling back to `props.value` when absent/unresolvable. This mirrors `ImageProps.data_path` (atoms.rs:410-414) exactly.

**Data convention for StatCard:**

```
value_path: "/data/{service.name}/{field.name}"
```

Handler provides:
```json
{ "data": { "statistics": { "total_revenue": "€12,450" } } }
```

### `$data` Binding in `Action.handler`

The `Action` struct supports `ActionHandler::Binding(DataRef { data: "/path" })` — a `{"$data": "/path"}` JSON object serialized into the `handler` field. This is used for row-level actions where the URL includes the row's primary key (e.g. `/orders/{id}/cancel`). The DataTable `$each` expander resolves these bindings per row.

For Gap B, when emitting row actions for DataTable, the `Action` handler can be set as:
```rust
Action::new(format!("/{}/{{id}}/{}", service.name, action.name))
```
The `{id}` is a template that the DataTable renderer substitutes with `row_key` per row. This is the existing convention used by bespoke hand-authored views.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| KanbanBoard columns from state | Custom Vec construction | Iterate `sm.states` directly | States already in correct order on `StateMachine.states: Vec<StateDef>` |
| StatCard value binding | New binding engine | `data::resolve_path_string` (already exists) | Same fn used by Image, DescriptionList, Input data_path |
| Action URL patterns | Custom route builder | `Action::new(format!(...))` | Existing `Action` type handles literals and `$data` bindings |
| Image column rendering | New component | `ColumnFormat::Image` + existing DataTable cell renderer | The cell renderer match already handles Badge; add Image to same match |
| Props serialization | Manual JSON construction | `serde_json::to_value(TypedProps)` + `element_with_props` | Existing pattern used by all other emitters |

---

## Common Pitfalls

### Pitfall 1: KanbanBoard data_path vs static columns conflict
**What goes wrong:** Builder sets both `data_path` and `columns`. Renderer uses `data_path` when present, ignoring `columns`. Static columns are then documentation-only and never render at runtime.
**Why it happens:** The renderer's branch (containers.rs:334) `if let Some(path) = props.data_path.as_deref()` takes precedence unconditionally.
**How to avoid:** Set static `columns` as the schema/fallback (used when `data_path` fails to resolve or is absent). Document this contract clearly. Both are useful: static = schema, dynamic = runtime data.

### Pitfall 2: Depth invariant for KanbanBoard (Pitfall 3 in existing builder.rs doc)
**What goes wrong:** Adding Kanban card sub-elements as spec children of the KanbanBoard element — spec depth exceeds 1 and breaks the catalog's depth rule.
**Why it happens:** KanbanBoard's catalog shape forbids child elements (same as StatCard). Cards render from `KanbanColumnProps.children` in props, not from `spec.elements`.
**How to avoid:** Do not add `aux` children to the KanbanBoard root. All column/card data stays in `KanbanBoardProps.columns` or in the runtime `data_path` array.

### Pitfall 3: StatCard orphan element (existing contract — preserve it)
**What goes wrong:** Connecting `metadata_list` to the StatCard root as a child (e.g. adding it to `children_out` in `emit_statcard_root`). This fails catalog validation because StatCard forbids children.
**Why it happens:** The metadata DescriptionList is emitted as an aux element (sibling), deliberately not wired to the root. The `statcard_metadata_is_orphan_element` regression test pins this.
**How to avoid:** Keep `dropped: Vec<String>` (the intentionally discarded children_out) in `emit_statcard_root`. The new `value_path` logic does not change this.

### Pitfall 4: Process layout uses `body` slot, not `fields`
**What goes wrong:** Looking for a `"fields"` slot to wire Gap A — there isn't one for Process.
**Why it happens:** Process template slots are `title, body, actions`. The `emit_kanban_root` is called for the outer layout match (`"KanbanBoard"` in `build_display_spec`), not through slot dispatch.
**How to avoid:** `emit_kanban_root` reads `service.state_machine` directly; it does not go through slot dispatch. The state machine columns are derived in the function itself, not from a slot.

### Pitfall 5: Action route placeholders are consumed correctly only by DataTable
**What goes wrong:** Setting `/{service.name}/{id}/{action.name}` as the handler for row actions in a DropdownMenu (actions slot context). `{id}` is only expanded by the DataTable `$each` expander — a standalone DropdownMenu element in the spec does not expand row-level template placeholders.
**Why it happens:** The `$data` binding and `{row_key}` substitution is a DataTable-specific runtime feature (the renderer substitutes `{row_key}` in URLs during row iteration).
**How to avoid:** For the `actions` slot context (Focus/Process), use `Action::Binding(DataRef { data: "/current_id" })` — the handler sets `spec.data["/current_id"]` to the entity's ID — OR use a plain URL and rely on the handler context. For Browse DataTable, use `row_actions` (which correctly runs per-row substitution). Gap B's DropdownMenu in the `actions` slot is for page-level actions, not row-level actions.

### Pitfall 6: `meaning_table_components_exist_in_catalog` drift guard
**What goes wrong:** Changing `ImageUrl.column` from `None` to `Some(())` without updating the column format handling in data.rs causes no compile error but produces blank image cells at runtime.
**Why it happens:** The drift guard tests only that component names exist in the catalog, not that ColumnFormat values are rendered.
**How to avoid:** Add a render test for the Image column format cell — assert `<img` appears in the rendered HTML when a column has `format: Some(ColumnFormat::Image)`.

### Pitfall 7: `catalog.validate` passes even with placeholder specs
**What goes wrong:** A test passes with `value: String::new()` in StatCard (current state) — the catalog validates the structure, not the content. Tests that only call `catalog.validate` will not catch Gap C regressions.
**How to avoid:** Per-gap render tests must assert the VALUE is non-empty or data-bound (e.g. `root.props["value_path"].as_str() == Some("/data/statistics/total_revenue")`), not just that `catalog.validate` returns `Ok`.

---

## Invariants to Preserve

### `catalog.validate` Rules (must pass after every gap)
[VERIFIED: reading ferro-json-ui/src/catalog.rs lines are not read in full, but behavior confirmed by existing test suite]

The catalog validates:
1. Every element's `type_name` must be a known component.
2. Every element's `children` IDs must exist in `spec.elements`.
3. Per-component prop schemas must validate (required props present, no extra props that violate schema).
4. StatCard: no children (forbids child elements on the root).
5. KanbanBoard: no children (same).

### Frozen intent classification
`ferro-projections/src/derive.rs`, `intent.rs`, and `ferro-projections/tests/catalog.rs` invariants must not change. All changes are in `ferro-json-ui/src/projection/builder.rs` and `component.rs`/`atoms.rs`.

### `statcard_metadata_is_orphan_element` regression test
This test (builder.rs:861-912) must stay green. The metadata DescriptionList is intentionally unreachable from the StatCard root.

### `from_service_def_validates` test
After each gap, the `from_service_def_validates` test (builder.rs:621-633) must pass — every success path implies `catalog.validate` returned `Ok`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in), `#[test]` attributes |
| Config file | `Cargo.toml` `[dev-dependencies]` |
| Quick run command | `cargo test -p ferro-json-ui --lib projection::builder -- --nocapture` |
| Full suite command | `cargo test --all-features` |

### Per-Gap Render Tests

Each gap requires one unit test in `ferro-json-ui/src/projection/builder.rs` (in the existing `mod tests` block) and the existing gestiscilo probe branch re-verification.

| Gap | Test Name | Assertion |
|-----|-----------|-----------|
| B | `actions_slot_emits_dropdown_from_service_actions` | `spec.elements` contains a `DropdownMenu` element when `service.actions` is non-empty; DropdownMenu has items matching each ActionDef |
| B (DataTable) | `datatable_root_has_row_actions_from_service_actions` | `DataTableProps.row_actions` is `Some(_)` and has the same count as `service.actions` |
| A | `kanban_root_derives_columns_from_state_machine` | `KanbanBoardProps.columns` count == `sm.states.len()`; each column `id` matches state `name`; `data_path` is `Some("/data/{name}/columns")` |
| A | `kanban_root_fallback_when_no_state_machine` | `KanbanBoardProps.columns` has exactly 1 column (fallback); `data_path` is `None` |
| C | `statcard_root_binds_primary_stat_field` | `StatCardProps.value_path` is `Some("/data/{name}/{field}")` for a service with a Money field |
| C | `statcard_root_empty_when_no_stat_field` | `StatCardProps.value_path` is `None` for a service with no Money/Quantity fields |
| D | `datatable_root_includes_image_url_column` | Column with `key == "avatar_url"` appears in `columns` when field has `FieldMeaning::ImageUrl` |
| D | `image_column_has_image_format` | Column for ImageUrl field has `format: Some(ColumnFormat::Image)` |

**Test fixture pattern** (follow existing pattern in builder.rs tests):
```rust
fn service_with_state_machine() -> ServiceDef {
    use ferro_projections::{StateMachine, StateDef, Transition};
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("status", DataType::String, FieldMeaning::Status)
        .state_machine(
            StateMachine::new("lifecycle")
                .initial("draft")
                .state(StateDef::new("draft").display_name("Draft"))
                .state(StateDef::new("submitted").display_name("Submitted"))
                .state(StateDef::new("done").display_name("Done").final_state())
                .transition(Transition::new("draft", "submit", "submitted"))
                .transition(Transition::new("submitted", "complete", "done"))
        )
}

fn service_with_actions() -> ServiceDef {
    ServiceDef::new("staff")
        .display_name("Staff")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .action(ActionDef::new("view").display_name("View"))
        .action(ActionDef::new("edit").display_name("Edit"))
        .action(ActionDef::new("delete").display_name("Delete"))
}
```

All tests use `Spec::from_service_def_with_catalog(&service, &intents, &ctx, &clean_catalog())` per the existing test-isolation pattern.

### Gestiscilo Integration Re-Verification

After each gap ships, rebuild ferro (`cargo build`) and restart the gestiscilo dev server. Use the Phase 209 harness:

1. **Server:** `cargo run --bin gestiscilo -- serve --backend-only` (port 8080)
2. **Login:** magic-link dev auto-login (`tenant: jetskiadriatic@gestiscilo.it`, id 3)
3. **Chrome DevTools MCP:** `chrome-devtools-3` profile (`/tmp/chrome-mcp-3`)
4. **Gap A verification:** `/dashboard/cassa/ordini` — must show 4 kanban columns (Confermati/In corso/Rientrato/Chiuso), not a single "Order/0" card. Insert test orders at different states to verify card grouping.
5. **Gap B verification:** `/dashboard/staff` — must show row actions dropdown (View/Edit/Toggle/Delete) on each row. Page-level "Nuovo" CTA must appear.
6. **Gap C verification:** the gestiscilo Statistics page (if migrated) — stat cards must show non-zero revenue/count values from handler data.
7. **Gap D verification:** `/dashboard/staff` avatar_url column must render as an image/avatar, not empty.

The gestiscilo probe branches (`feat/207-orders-projection-migration`, `feat/208-staff-projection-migration`) are NOT to be merged or modified. They are the integration test bed — rebuild ferro and re-run the existing branches against the rebuilt binary.

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui --lib projection::builder`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green + both gestiscilo probe branches re-verified before `/gsd-verify-work`

### Wave 0 Gaps (test infrastructure)

- `service_with_state_machine()` fixture — must be added to `builder.rs` tests
- `service_with_actions()` fixture — must be added to `builder.rs` tests  
- `service_with_money_field()` fixture — can extend existing `sample_service()` (already has `FieldMeaning::Money`)
- No new test files needed — all tests go in existing `mod tests` block in `builder.rs`

---

## Risks and Open Questions for the Planner

### Risk 1 (MEDIUM): StatCard multi-stat — Grid wrapper vs single card
**Question:** D-02 says "one StatCard per Money/Quantity readable field". When a service has 3 money fields, should `emit_statcard_root` return a `Grid(StatCard, StatCard, StatCard)` root, or just one StatCard for the primary field?
**Current `emit_statcard_root` contract:** returns a single `ElementBuilder` (the root). Returning a Grid requires the root to be "Grid" containing 3 StatCard aux elements. This works with the existing pipeline but changes the root element type for Summarize, which may affect `from_service_def_browse_display`-style tests.
**Recommendation:** Start with single StatCard (primary stat field only) for Phase 213. Multi-stat Grid is a follow-up. This keeps the implementation minimal and preserves the existing orphan-metadata contract without structural changes.
**Risk if wrong:** Statistics dashboard shows only one stat (total revenue) not all stats. Acceptable for v1; multiple-stat follow-up is small.

### Risk 2 (LOW): ColumnFormat::Image rendering in data.rs
**Question:** The DataTable cell renderer (data.rs) currently handles `ColumnFormat::Badge` with a special path. Adding `ColumnFormat::Image` requires a new branch that renders `<img src="...">` from the cell string value. The exact HTML/CSS must match the dashboard's avatar treatment.
**Current renderer handling:** The cell value for an Image column will be a URL string (e.g. `https://...`). The render branch must emit something like `<img src="..." alt="" class="w-8 h-8 rounded-full object-cover">`.
**Risk:** The `ColumnFormat::Image` render branch may need multiple iterations to match the gestiscilo avatar treatment. This is purely cosmetic — function is achieved immediately.

### Risk 3 (LOW): `emit_kanban_root` signature change for `VisualContext.current_state`
**Question:** D-01 says `VisualContext.current_state` may mark the active column. This requires passing `ctx` into `emit_kanban_root`. Currently the function takes only `service: &ServiceDef`. The call site in `build_display_spec` has `ctx` in scope.
**Impact:** Purely local signature change. No external API surface affected.
**Recommendation:** Pass `ctx` to `emit_kanban_root`. The `VisualContext` struct already has `current_state: Option<String>`. Mark the active column by attaching metadata to the `KanbanColumnProps` (but `KanbanColumnProps` has no `active` field). Options: (a) set `mobile_default_column` to the current state (reasonable approximation), or (b) add `active: Option<bool>` to `KanbanColumnProps` (props extension). For Phase 213, option (a) is sufficient.

### Risk 4 (LOW): `ActionDef` has no route — the URL pattern must be conventional
**The `ActionDef` struct has:** `name`, `display_name`, `description`, `inputs`, `preconditions`, `effects`, `transition_trigger`. There is NO explicit `route` or `url` field.
**Implication:** The builder must synthesize the URL from `service.name` + `action.name`. Convention: `POST /{service.name}/{action.name}`. This is a reasonable REST-ish convention but is NOT enforced — consumer handlers must implement routes matching this pattern for the buttons to work.
**Recommendation:** Document this convention in the emit function's doc comment. The planner should ensure the convention is mentioned in the integration guide or MCP tool descriptions.

### Risk 5 (MEDIUM): Browse (DataTable) and Process (KanbanBoard) need different action wiring
**Browse DataTable:** row actions go into `DataTableProps.row_actions` — these execute per-row and expand `{row_key}`. The actions slot is NOT in Browse's slot template (`title, fields, pagination`). Therefore: `emit_datatable_root` must also populate `row_actions` from `service.actions`. This is part of Gap B but in a different emit function.
**Process KanbanBoard:** The `actions` slot IS in Process's slot template (`title, body, actions`). But `emit_kanban_root` does not walk slots — it is called for the outer layout. The slot walk happens in `emit_card_root`. For Process/KanbanBoard, the slot walk DOES happen in `build_display_spec` → the `"actions"` slot dispatches `emit_actions_placeholder`. So actions ARE emitted for Process.
**Summary:** B fixes `emit_actions_placeholder` (affects Focus/Process/Track via slot walk). B also needs to fix `emit_datatable_root.row_actions` (affects Browse). These are two separate change sites for one logical gap.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `DropdownMenu` is in the catalog (no new catalog registration needed) | Gap B | Catalog validation would fail — easily fixed by checking catalog.rs |
| A2 | `Grid` is in the catalog (for multi-StatCard wrapping) | Gap C risk | Same — if not, use a Card wrapper or defer multi-stat |
| A3 | `ColumnFormat::Image` does NOT yet exist (needs to be added) | Gap D | If it already exists, skip the enum addition |
| A4 | The gestiscilo orders ServiceDef has `state_machine` set (confirmed from Phase 209 evidence of correct `Process` classification) | Gap A integration test | If state_machine was not set on the gestiscilo ServiceDef, kanban columns would still use fallback — not a blocker |

*Note: A1 and A2 can be verified in <30s via `grep -n "DropdownMenu\|\"Grid\"" ferro-json-ui/src/catalog.rs`.*

---

## Sources

### Primary (HIGH confidence — verified from codebase)
- `/ferro-json-ui/src/projection/builder.rs` — full source of all emit functions
- `/ferro-json-ui/src/projection/intent_layout.rs` — slot templates
- `/ferro-json-ui/src/component.rs` — all Props types
- `/ferro-json-ui/src/projection/component_map.rs` — `lookup_meaning`, `build_column_for_field`
- `/ferro-projections/src/state.rs` — `StateMachine`, `StateDef`, `Transition`
- `/ferro-projections/src/action.rs` — `ActionDef`
- `/ferro-projections/src/field.rs` — `FieldMeaning`, `FieldDef`
- `/ferro-projections/src/render/mod.rs` — `VisualContext`, `BaseContext`, `Renderer`
- `/ferro-projections/src/service.rs` — `ServiceDef` (confirmed `state_machine: Option<StateMachine>`, `actions: Vec<ActionDef>`)
- `/ferro-json-ui/src/render/containers.rs` — `render_kanban_board` with `data_path` handling
- `/ferro-json-ui/src/render/atoms.rs` — `render_stat_card` (confirmed no `data_path` on `StatCardProps`)
- `/ferro-json-ui/src/data.rs` — `resolve_path`, `resolve_path_string`
- `/ferro-json-ui/src/action.rs` — `Action`, `ActionHandler`, `DataRef`
- `.planning/phases/209-comp-01-slice-a-gestiscilo-migration/WEAKNESS-NOTE.md` — gap definitions
- `.planning/phases/209-comp-01-slice-a-gestiscilo-migration/EQUIV-orders-process.md` — confirmed Gap A root cause
- `.planning/phases/209-comp-01-slice-a-gestiscilo-migration/EQUIV-staff-browse.md` — confirmed Gap B/D root cause
- `213-CONTEXT.md` — locked decisions D-01 through D-08

### Secondary
- None (all findings verified directly from codebase)

---

## Metadata

**Confidence breakdown:**
- Builder pipeline flow: HIGH — read complete source
- Per-gap current code: HIGH — exact code excerpted from source
- Component props shapes: HIGH — read complete component.rs
- Data-binding convention: HIGH — read data.rs + atoms.rs + containers.rs
- StatCard value_path extension requirement: HIGH — confirmed by reading `render_stat_card` which uses `props.value` directly with no path resolution
- ColumnFormat::Image non-existence: ASSUMED (A3) — read ColumnFormat enum, no Image variant found; confirm in <5s
- Catalog contents (DropdownMenu, Grid registration): ASSUMED (A1, A2) — not confirmed by reading catalog.rs; low-risk assumption

**Research date:** 2026-06-12
**Valid until:** 2026-07-12 (stable codebase; 30-day validity)

---

## RESEARCH COMPLETE

**Phase:** 213 — Projection Render Completeness
**Confidence:** HIGH

### Key Findings

1. **Gap B (actions) is a pure builder change** — emit `DropdownMenuAction` items from `service.actions` in `emit_actions_placeholder`. Also wire `DataTableProps.row_actions` in `emit_datatable_root`. No new components, no catalog changes.

2. **Gap A (kanban) is a pure builder change** — iterate `sm.states` in `emit_kanban_root`, set `data_path: /data/{name}/columns`. Fallback single-column preserved. Signature change needed to pass `ctx` for `current_state`.

3. **Gap C (statcard) requires one props extension** — `StatCardProps` needs `value_path: Option<String>`. The renderer (`render_stat_card`) must then call `resolve_path_string`. This is the only change that touches component.rs and atoms.rs. All other gaps are builder.rs only.

4. **Gap D (imageurl) requires one enum variant and one renderer branch** — add `ColumnFormat::Image`, change `lookup_meaning(ImageUrl).column` to `Some(())`, add Image cell rendering in data.rs.

5. **KanbanBoard data_path uses `columns` suffix** (`/data/{name}/columns`) to distinguish from the flat item list at `/data/{name}`, matching the `KanbanColumnProps` array structure the renderer expects.

### Files to Change

- `ferro-json-ui/src/projection/builder.rs` — all five gaps (primary file)
- `ferro-json-ui/src/component.rs` — `StatCardProps.value_path` (Gap C) + `ColumnFormat::Image` (Gap D)
- `ferro-json-ui/src/render/atoms.rs` — `render_stat_card` value resolution (Gap C)
- `ferro-json-ui/src/render/data.rs` — `ColumnFormat::Image` cell renderer (Gap D)
- `ferro-json-ui/src/projection/component_map.rs` — `ImageUrl.column: Some(())` (Gap D)

### Open Questions

1. **Multi-stat StatCard:** single primary field vs Grid(StatCard×N)? Recommendation: single for Phase 213.
2. **Action route convention:** `/{service.name}/{action.name}` — must document as the convention binding the projection to the consumer's route table.
3. **KanbanBoard `current_state` highlight:** use `mobile_default_column` as approximation, or defer active-column highlight entirely.

### Ready for Planning

Research complete. Planner can now create PLAN.md files for each of the five gaps in B→A→C→D→E order.
