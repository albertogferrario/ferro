# Service Projections

Service Projections derive a JSON-UI layout automatically from the structure and semantics of your data. Define what your data IS — fields, types, meanings, and workflow states — and the framework infers the appropriate UI intent and generates a component tree.

## Overview

The pipeline has three stages:

```
ServiceDef  →  derive_intents(&service_def)  →  IntentScore[]
                                                      ↓
                      Renderer.render(&service_def, &intents, &ctx)  →  Output
```

1. **ServiceDef** — describe your service: field names, data types, semantic meanings, state machines, guards, and actions.
2. **derive_intents** — analyzes the service definition and returns a ranked list of `IntentScore` values. The highest-scoring intent is the primary one.
3. **Renderer** — takes the service definition, the ranked intents, and a render context, and produces output. The `Renderer` trait is modality-agnostic: each implementation declares its own `Output` and `Context` types. Two renderers ship today:
   - **`JsonUiRenderer`** (`Output = Spec`, `Context = VisualContext`) — produces a ferro-json-ui component tree for screens.
   - **`TextRenderer`** (`Output = String`, `Context = BaseContext`) — produces conversational text for non-visual channels (see [Conversational-Text Rendering](#conversational-text-rendering)).

The same `ServiceDef` drives every renderer; the modality is chosen by which renderer and context you use.

## Quick Start

Minimal example: a product service deriving its intent and rendering to JSON-UI.

```rust
use ferro::{
    DataType, FieldMeaning, ServiceDef,
    derive_intents, JsonUiRenderer, Renderer, VisualContext,
};

let product = ServiceDef::new("product")
    .display_name("Product")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("name", DataType::String, FieldMeaning::EntityName)
    .field("price", DataType::Float, FieldMeaning::Money);

let intents = derive_intents(&product);
// intents[0] is the highest-confidence intent (Browse for a simple list-like service)

let renderer = JsonUiRenderer;
let result = renderer.render(&product, &intents, &VisualContext::default());
let spec = result.expect("rendering a valid service definition should not fail");
// spec.schema == "ferro-json-ui/v2"
// spec.elements is a flat ID-keyed map; spec.root names the root element
```

## Core Concepts

### ServiceDef Builder

`ServiceDef` is the entry point. Build it with a method chain:

```rust
use ferro::{
    ActionDef, DataType, FieldMeaning, GuardDef,
    ServiceDef, StateDef, StateMachine, Transition,
};

pub fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        // Fields define the data shape
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("total", DataType::Float, FieldMeaning::Money)
        // Workflow states and transitions
        .state_machine(
            StateMachine::new("order_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("completed").final_state())
                .transition(Transition::new("draft", "complete", "completed")),
        )
        // Guards control who can perform actions
        .guard(GuardDef::new("is_manager").display_name("Manager Approval Required"))
        // Actions are operations users can trigger
        .action(ActionDef::new("approve").precondition("is_manager"))
}
```

**Builder methods:**

| Method | Description |
|--------|-------------|
| `ServiceDef::new(name)` | Create a new service definition with a machine-readable name |
| `.display_name(label)` | Human-readable label shown in the UI |
| `.field(name, type, meaning)` | Add a field with its data type and semantic meaning |
| `.state_machine(sm)` | Attach a workflow state machine |
| `.guard(guard_def)` | Define a guard (permission or condition) |
| `.action(action_def)` | Define an action users can trigger |
| `.intent_hint(hint)` | Override or exclude derived intents (see Intent Overrides) |

### Fields and Meanings

Every field needs a `DataType` and a `FieldMeaning`. The data type describes the storage format; the meaning describes what the value represents semantically.

**DataType:**

| Variant | When to use |
|---------|-------------|
| `DataType::Integer` | Whole numbers: IDs, counts, quantities |
| `DataType::Float` | Decimal numbers: prices, measurements, scores |
| `DataType::String` | Text values: names, titles, codes, descriptions |
| `DataType::Boolean` | True/false flags |
| `DataType::Date` | Calendar date (no time) |
| `DataType::DateTime` | Date plus time |
| `DataType::Json` | Structured payloads stored as JSON |
| `DataType::Binary` | Opaque byte sequences |
| `DataType::Uuid` | UUID identifiers |
| `DataType::Enum` | Fixed set of values: status, category |

**FieldMeaning:**

| Variant | When to use |
|---------|-------------|
| `FieldMeaning::Identifier` | Primary key or unique ID |
| `FieldMeaning::ForeignKey` | Reference to another record's identifier |
| `FieldMeaning::EntityName` | Display name of the record |
| `FieldMeaning::Email` | Email address |
| `FieldMeaning::Phone` | Phone number |
| `FieldMeaning::Url` | Web URL |
| `FieldMeaning::ImageUrl` | Image URL or path |
| `FieldMeaning::Money` | Monetary amount |
| `FieldMeaning::Percentage` | Percentage value |
| `FieldMeaning::Quantity` | Aggregate count or numeric quantity |
| `FieldMeaning::Status` | Current state or lifecycle value |
| `FieldMeaning::Category` | Categorical tag or grouping |
| `FieldMeaning::Boolean` | Yes/no flag |
| `FieldMeaning::FreeText` | Long descriptive text |
| `FieldMeaning::CreatedAt` / `FieldMeaning::UpdatedAt` / `FieldMeaning::DateTime` | Timestamp fields |
| `FieldMeaning::Sensitive` | Sensitive value treated as a password input |
| `FieldMeaning::Custom(String)` | Domain-specific meaning not covered above |

**Render hints.** `FieldDef` carries an optional `render_hint: Option<RenderHint>` consumed by non-visual renderers. It controls how a `Url` or `ImageUrl` field — whose value has no useful text form on its own — is presented. The visual renderer ignores it; `None` (the default) preserves existing behavior. Attach a hint with the `field_with_hint` builder:

```rust
use ferro::{DataType, FieldMeaning, RenderHint, ServiceDef};

let profile = ServiceDef::new("profile")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    // Substitute alt text in place of the raw image URL in text output
    .field_with_hint(
        "avatar",
        DataType::String,
        FieldMeaning::ImageUrl,
        RenderHint::AltText("User avatar".into()),
    )
    // Drop a navigational URL from non-visual output entirely
    .field_with_hint(
        "tracking_url",
        DataType::String,
        FieldMeaning::Url,
        RenderHint::Skip,
    );
```

To set a hint on a field built elsewhere, `FieldDef::with_render_hint(hint)` is the consuming-builder equivalent.

| `RenderHint` variant | Effect in non-visual output |
|----------------------|-----------------------------|
| `RenderHint::AltText(String)` | Render the given string in place of the raw URL/image value |
| `RenderHint::Skip` | Omit the field from non-visual output entirely |
| `None` (no hint) | `Url` → a `(link)` label, `ImageUrl` → an `(image)` label; never the raw URL |

### Intent Derivation

`derive_intents` examines five structural signals to score and rank the seven intents:

**Signal analyzers:**

1. **Field count** — many fields suggest a form (Collect); few fields suggest a list (Browse).
2. **Field meanings** — presence of `Money`, `Status`, `Quantity` meanings shifts scores toward specific intents.
3. **State machines** — a state machine with transitions strongly scores Process.
4. **Guards and actions** — approval workflows score Track; rich action sets score Process.
5. **Naming patterns** — service name patterns like "report", "summary", "dashboard" shift scores toward Summarize or Analyze.

**The seven intents:**

| Intent | Structural signal |
|--------|-------------------|
| `Intent::Browse` | List of records with identifier and name fields |
| `Intent::Focus` | Single record detail view |
| `Intent::Collect` | Input form (many fields, writable) |
| `Intent::Process` | Workflow with state machine and transitions |
| `Intent::Summarize` | Aggregated or summary-level data |
| `Intent::Analyze` | Metric-heavy or analytical view |
| `Intent::Track` | Audit trail or progress tracking |

`derive_intents` returns `Vec<IntentScore>`, sorted by confidence descending. Each `IntentScore` has:

```rust
// intents[0] is primary
let primary = &intents[0];
println!("Intent: {:?}", primary.intent);
println!("Confidence: {}", primary.confidence);
println!("Signals: {:?}", primary.matching_signals);
```

### Intent Overrides

If the derived intent is not what you want, use `IntentHint` to force or exclude intents:

```rust
use ferro::{DataType, FieldMeaning, Intent, IntentHint, ServiceDef};

let service = ServiceDef::new("order_summary")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("total", DataType::Float, FieldMeaning::Money)
    // Force Summarize even though signals might score Browse higher
    .intent_hint(IntentHint::Primary(Intent::Summarize))
    // Prevent Browse from appearing even as a fallback
    .intent_hint(IntentHint::Exclude(Intent::Browse));
```

Use `IntentHint::Primary` to promote a specific intent to the top. Use `IntentHint::Exclude` to prevent an intent from being selected at all. Multiple hints can be combined.

### Rendering

`JsonUiRenderer` implements the `Renderer` trait. Use it with a `VisualContext` to control how the output is shaped.

```rust
use ferro::{JsonUiRenderer, VisualContext, RenderMode, Renderer};

// Display mode: read-only view of data
let display_ctx = VisualContext {
    intent_index: 0,              // use primary intent
    current_state: None,          // no workflow state active
    mode: RenderMode::Display,    // read-only layout
    templates: None,              // use default layouts
};

// Input mode: editable form
let input_ctx = VisualContext {
    intent_index: 0,
    current_state: Some("draft".to_string()), // current workflow state
    mode: RenderMode::Input,                  // form layout
    templates: None,
};

let renderer = JsonUiRenderer;
let spec = renderer.render(&service_def, &intents, &input_ctx).expect("rendering a valid service definition should not fail");
```

**VisualContext fields:**

| Field | Type | Description |
|-------|------|-------------|
| `intent_index` | `usize` | Index into the `IntentScore` list; `0` for primary intent |
| `current_state` | `Option<String>` | Active state name from the state machine, if applicable |
| `mode` | `RenderMode` | `RenderMode::Display` for read-only; `RenderMode::Input` for forms |
| `templates` | `Option<ThemeTemplates>` | Custom layout overrides; `None` uses defaults |

**RenderMode:**

| Variant | Output |
|---------|--------|
| `RenderMode::Display` | Read-only component tree for viewing data |
| `RenderMode::Input` | Editable form component tree for creating or updating data |

## Complete Example

An order management service with a workflow, a guard, and an action, rendered in input mode at the "draft" state:

```rust
use ferro::{
    ActionDef, DataType, FieldMeaning, GuardDef,
    JsonUiRenderer, VisualContext, RenderMode, Renderer,
    ServiceDef, StateDef, StateMachine, Transition,
    derive_intents,
};

let order = ServiceDef::new("order")
    .display_name("Order")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("customer", DataType::String, FieldMeaning::EntityName)
    .field("total", DataType::Float, FieldMeaning::Money)
    .field("status", DataType::Enum, FieldMeaning::Status)
    .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
    .state_machine(
        StateMachine::new("lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("approved"))
            .state(StateDef::new("shipped"))
            .state(StateDef::new("completed").final_state())
            .transition(Transition::new("draft", "approve", "approved"))
            .transition(Transition::new("approved", "ship", "shipped"))
            .transition(Transition::new("shipped", "complete", "completed")),
    )
    .guard(GuardDef::new("is_manager").display_name("Manager Approval Required"))
    .action(ActionDef::new("approve").precondition("is_manager"));

let intents = derive_intents(&order);
// Process intent scores highest due to state machine + actions

let ctx = VisualContext {
    intent_index: 0,
    current_state: Some("draft".to_string()),
    mode: RenderMode::Input,
    templates: None,
};

let spec = JsonUiRenderer.render(&order, &intents, &ctx).expect("rendering a valid service definition should not fail");
// Produces a ferro-json-ui component tree with:
// - Fields rendered as form inputs
// - Available transitions ("approve") rendered as action buttons
// - Guard labels displayed on the action
```

## Conversational-Text Rendering

`TextRenderer` is a non-visual `Renderer` that projects the same `ServiceDef` into plain text suitable for a conversational channel — a chat reply, a CLI summary, a notification body. It produces `String` output and takes a `BaseContext` instead of a `VisualContext`.

```rust
use ferro::{
    derive_intents, BaseContext, Renderer, TextRenderer, Verbosity,
    DataType, FieldMeaning, ServiceDef,
};

let product = ServiceDef::new("product")
    .display_name("Product")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("name", DataType::String, FieldMeaning::EntityName)
    .field("price", DataType::Float, FieldMeaning::Money);

let intents = derive_intents(&product);
let text = TextRenderer
    .render(&product, &intents, &BaseContext::default())
    .expect("rendering a valid service definition should not fail");
// Product
// Fields:
//   - Name
//   - Price
```

`BaseContext::default()` renders the primary intent at full verbosity with no guard filtering — the equivalent of the visual renderer's defaults.

### BaseContext fields

| Field | Type | Description |
|-------|------|-------------|
| `intent_index` | `usize` | Index into the `IntentScore` list; `0` for the primary intent |
| `current_state` | `Option<String>` | Active workflow state, surfaced by Process/Track output |
| `evaluated_guards` | `HashMap<String, bool>` | Guard-name → result. Filters action affordances (see below) |
| `verbosity` | `Verbosity` | `Verbosity::Full` (default) or `Verbosity::Brief` |

`BaseContext` is also the modality-agnostic base the visual `VisualContext` embeds, so guard and verbosity context is shared across renderers.

### Per-intent output

The renderer dispatches on the primary intent. Five intents map cleanly to text:

| Intent | Output shape |
|--------|--------------|
| `Browse` | Entity name + its identifying fields as a list |
| `Collect` | "Fields to fill in" — the writable inputs, with `(required)` markers |
| `Process` | Current state + the guard-passing actions available from it |
| `Summarize` | Entity name + a one-line "Key metrics" list |
| `Track` | Current state + the next reachable states |

`Focus` and `Analyze` have no full text form (a media/navigational detail view and a time-series view, respectively). They render a **defined fallback** — the available fields plus a one-line note — rather than failing or fabricating data:

```text
Profile
  - Avatar (image)
  - Website (link)
Note: This is a media/navigational view; full text representation is limited.
```

An empty intent slice returns `ProjectionsError::NoIntents` rather than emitting a placeholder label.

### Guard filtering

In a Process render, an action is shown only when its guards pass. `evaluated_guards` maps each guard name to a boolean; an action is **hidden** only when one of its preconditions is explicitly `false`. An absent key renders the action (the guard is treated as not-yet-evaluated), so `BaseContext::default()` — an empty map — shows every action, matching the visual renderer.

```rust
use std::collections::HashMap;
use ferro::{BaseContext, Renderer, TextRenderer, Verbosity, derive_intents};

// approval_workflow: actions submit, approve, reject, cancel
// approve and reject require the `is_approver` guard.
let intents = derive_intents(&approval_workflow);

// No guard context → every action is listed:
let all = TextRenderer.render(&approval_workflow, &intents, &BaseContext::default()).unwrap();
// ... Available actions: submit, approve, reject, cancel

// Caller is not an approver → approve and reject are filtered out:
let mut guards = HashMap::new();
guards.insert("is_approver".to_string(), false);
let ctx = BaseContext { evaluated_guards: guards, ..Default::default() };
let filtered = TextRenderer.render(&approval_workflow, &intents, &ctx).unwrap();
// ... Available actions: submit, cancel
```

This makes guard evaluation the consumer's responsibility — the renderer never tells a caller they can perform an action their guards would reject.

### Verbosity

`Verbosity::Full` (the default) renders the complete per-intent view. `Verbosity::Brief` collapses it to a single line — useful for a notification or a quick acknowledgement:

```rust
use ferro::{BaseContext, Renderer, TextRenderer, Verbosity};

let ctx = BaseContext { verbosity: Verbosity::Brief, ..Default::default() };
let brief = TextRenderer.render(&approval_workflow, &intents, &ctx).unwrap();
// approval_workflow — Currently: draft. You can: submit, approve, reject, cancel.
```

Brief output still respects guard filtering, so it only ever lists permitted actions.

## Reference

| Type | Description |
|------|-------------|
| `ServiceDef` | Builder for describing a service's data shape, workflow, and capabilities |
| `DataType` | Enum of storage types for a field (Integer, String, Float, etc.) |
| `FieldMeaning` | Semantic meaning of a field (Identifier, Money, Status, etc.) |
| `StateMachine` | Builder for workflow states and transitions |
| `StateDef` | A single workflow state; call `.final_state()` to mark terminal states |
| `Transition` | A directed edge between two states, triggered by an action name |
| `GuardDef` | A named permission or condition checked before an action is allowed |
| `ActionDef` | A user-triggerable operation; optionally requires a guard via `.precondition()` |
| `Intent` | Enum of seven structural intents: Browse, Focus, Collect, Process, Summarize, Analyze, Track |
| `IntentScore` | A ranked intent result with `intent`, `confidence`, and `matching_signals` |
| `IntentHint` | Override directive: `Primary(intent)` promotes, `Exclude(intent)` blocks |
| `derive_intents` | Analyzes a `ServiceDef` and returns a confidence-ranked `Vec<IntentScore>` |
| `JsonUiRenderer` | Implements `Renderer`; converts `ServiceDef` + intents + context to JSON-UI (`Output = Spec`) |
| `TextRenderer` | Implements `Renderer`; converts `ServiceDef` + intents + `BaseContext` to conversational text (`Output = String`) |
| `Renderer` | Modality-agnostic trait; one method `render(def, intents, ctx)` with associated `Output`/`Context` types |
| `VisualContext` | Visual render parameters: intent index, current state, mode, template overrides (embeds `BaseContext`) |
| `BaseContext` | Modality-agnostic render parameters: intent index, current state, evaluated guards, verbosity |
| `RenderMode` | `Display` for read-only output; `Input` for editable form output (visual only) |
| `Verbosity` | `Full` (default, complete render) or `Brief` (single-line) for non-visual output |
| `RenderHint` | Optional `FieldDef` hint for non-visual rendering of `Url`/`ImageUrl`: `AltText(String)` or `Skip` |

## Rendering a Projection Inside an App Shell

A projection `Spec` is a standalone spec rooted at a single content component (`DataTable`, `KanbanBoard`, or `StatCard`). The spec contains no surrounding dashboard chrome — no sidebar, navigation bar, or `PageHeader` wrapper. That surrounding structure is the consumer application's responsibility.

Two supported composition patterns:

### Pattern A: Merge into an existing layout spec

Insert the projection root element into an existing layout spec's main-content children at handler time:

```rust
// Consumer handler pseudocode
let intents = derive_intents(&order_service);
let ctx = VisualContext { mode: RenderMode::Display, ..Default::default() };
let projection_spec = JsonUiRenderer.render(&order_service, &intents, &ctx)?;

// Load the dashboard layout spec and graft the projection root into it
let mut layout_spec = /* load or build dashboard layout spec */;
let root_id = projection_spec.root.clone();
if let Some(root_el) = projection_spec.elements.get(&root_id) {
    layout_spec.elements.insert(root_id.clone(), root_el.clone());
    // also insert any aux elements the projection added
    for (id, el) in &projection_spec.elements {
        if id != &root_id {
            layout_spec.elements.insert(id.clone(), el.clone());
        }
    }
    // wire the projection root into the layout's main-content children list
    if let Some(main_content) = layout_spec.elements.get_mut("main_content") {
        main_content.children.push(root_id);
    }
}
```

### Pattern B: Return the projection spec at a known key

The handler returns both the data payload and the projection spec under a documented key. The dashboard layout template reads and embeds it:

```rust
// Handler response (projection spec returned alongside row data)
serde_json::json!({
    "data": {
        "order": order_rows
    },
    "projection": serde_json::to_value(&projection_spec)?
})
```

The layout template references `projection` at its documented key to render the content area.

### No first-class layout context

A `VisualContext.layout` field for automatic app-shell selection is not provided in this release. The composition patterns above are the supported contract. Authorization of rendered actions, route existence, and tenant scoping remain the consumer application's responsibility — the renderer emits affordances but does not enforce access control.

---

## Projection Content Binding

This section documents the URL and data-path conventions that link a projection `Spec` to the consumer application's route table and handler data. A consumer integrating a projection spec must implement routes and data shapes that match these conventions.

### Action routes

`ActionDef` has no explicit `route` field. The renderer synthesizes action URLs from service and action names:

| Context | URL pattern | Notes |
|---------|-------------|-------|
| Page-level action | `/{service.name}/{action.name}` | Emitted as a `Button` or `ActionGroup` item |
| DataTable row action | `/{service.name}/{row_key}/{action.name}` | `{row_key}` is substituted per row at render time using `DataTableProps.row_key` (defaults to `"id"`) |

The consumer's route table must define handlers at these paths for the action affordances to be functional. Example for a service named `order` with an action named `approve`:

- Page-level: `POST /order/approve`
- Row-level: `POST /order/{id}/approve`

### DataTable rows

`DataTableProps.data_path` points to the flat array of row objects in the handler response:

```
data_path: "/data/{service.name}"
```

Handler provides:

```json
{
  "data": {
    "staff": [
      { "id": 1, "name": "Alice", "active": true },
      { "id": 2, "name": "Bob",   "active": false }
    ]
  }
}
```

### KanbanBoard columns

`KanbanBoardProps.data_path` points to an array of `KanbanColumnProps` objects, one per state:

```
data_path: "/data/{service.name}/columns"
```

The `/columns` suffix distinguishes the column array from the flat item array at `/data/{service.name}`. Handler provides:

```json
{
  "data": {
    "order": {
      "columns": [
        { "id": "draft",     "title": "Draft",     "count": 2, "children": [] },
        { "id": "submitted", "title": "Submitted", "count": 1, "children": [] },
        { "id": "done",      "title": "Done",      "count": 0, "children": [] }
      ]
    }
  }
}
```

The static `columns` in the emitted spec (derived from the service's state machine) serve as the schema reference and render fallback when `data_path` fails to resolve. The handler is responsible for grouping items by state and computing per-column counts. When `data_path` resolves, it takes precedence over the static columns.

### StatCard value

`StatCardProps.value_path` points to the scalar value for the primary stat field:

```
value_path: "/data/{service.name}/{field.name}"
```

The renderer picks the first `Money` or `Quantity` readable field as the primary stat. Handler provides:

```json
{
  "data": {
    "statistics": {
      "total_revenue": "€12,450"
    }
  }
}
```

The `value_path` field is resolved at render time via the same JSON-pointer mechanism as `data_path` on other components (see [Data Binding](../json-ui/data-binding.md)). The static `value` string in the spec is the fallback when `value_path` is absent or fails to resolve.

## Write Contracts

The renderer emits action affordances; executing them is the consumer application's responsibility. Two pages cover the write path:

- [Transition Planning](transition-planning.md) — derive a write's target state and guard from the declared `StateMachine`, so the target is declared once and never hand-written.
- [Write Kernel](write-kernel.md) — the channel-agnostic `dispatch_write` pipeline: server-side guard re-evaluation, idempotency, audit, and post-persist override. One kernel backs both the MCP and the visual write paths.

A page-level action button emits `POST /{service}/{action}`. The handler at that route resolves the `(ServiceDef, ActionDef)`, reads the tenant from auth, derives the transition guard from the plan, and calls `dispatch_write` — it never reads the target state from the request body:

```rust
use ferro::{derive_transition_plan, write::dispatch_write};

let plan = derive_transition_plan(svc, &action.name).ok();
let transition_guard = plan.as_ref().and_then(|p| p.guard.as_deref());

let outcome = dispatch_write(
    action,
    &inputs,
    tenant_id,         // from auth, never the body
    db,
    &dispatcher,
    transition_guard,  // derived from the StateMachine
    "web",             // audit channel
    #[cfg(feature = "confirmation")]
    false,
    None,              // crud_plan: None on the transition path
)
.await;
```

---

## MCP Tools

Five MCP tools support Service Projections development: listing, inspection, rendering, validation, and coverage analysis.

### `list_projections`

- **Returns:** All `ServiceDef` definitions found in the project, with service name, field count, detected intents, and whether a state machine is defined
- **When to use:** Audit what services are defined; find the correct service name before calling `inspect_projection`

### `inspect_projection`

- **Returns:** Full `ServiceDef` breakdown: all fields with their `DataType` and `FieldMeaning`, state machine states and transitions, guards, actions, and the full ranked `IntentScore` list from `derive_intents`
- **When to use:** Understand why a particular intent was derived; verify field meanings before rendering; debug unexpected component tree output

### `render_projection`

- **Returns:** The rendered JSON-UI component tree for a named service, given an intent index and render mode
- **When to use:** Preview what the renderer produces without writing a handler; compare `Display` vs `Input` output; verify that state machine transitions appear correctly at a given current state

### `validate_projection`

- **Returns:** Validation results for a `ServiceDef`: missing required fields, unresolvable guards, unreachable states, and mismatched `IntentHint` directives
- **When to use:** Catch definition errors before runtime; verify a service definition is well-formed after editing

### `projection_coverage`

- **Returns:** A summary of which models have corresponding `ServiceDef` projections and which do not, along with suggestions for field meanings based on column names
- **When to use:** Identify models that could benefit from projection-based UIs; plan which services to define next

---

## MCP CRUD Opt-In

A `ServiceDef` can expose CRUD write tools (`create_<svc>`, `update_<svc>`, `delete_<svc>`) to agents on the MCP surface in addition to the read tool (`list_<svc>`). This is an additive opt-in on the `ServiceDef` — it does not affect the visual/form write path, which uses the same `dispatch_write` kernel independently.

### Enabling CRUD tools

Add four builder calls to your service definition:

```rust
ServiceDef::new("order")
    .mcp_exposed(true)
    .tenant_column("tenant_id")
    .mcp_ability("view-orders")          // read gate (existing)
    .mcp_write_ability("manage-orders")  // write gate: scopes create/update/delete
    .creatable(true)                     // derives create_order tool
    .updatable(true)                     // derives update_order tool
    .deletable(true)                     // derives delete_order tool (soft-delete)
    .soft_delete_column("deleted_at")    // required: list_order excludes soft-deleted rows
    // ... fields, state_machine, actions unchanged ...
```

**Prerequisites:**

| Requirement | Why |
|-------------|-----|
| `tenant_column` set | `tenant_id` is injected into every write; never accepted from the agent's input body |
| `deleted_at` column in the table | `deletable(true)` performs a soft-delete (sets `deleted_at`); hard-delete is not supported |
| `mcp_write_ability` present when any write flag is true | `ServiceDef::validate()` fails at boot otherwise (CRUD-07) |

### Derived tool set

| Tool | Behavior |
|------|----------|
| `create_<svc>` | INSERT a new row; `status` excluded from input when a StateMachine is defined (set server-side to initial state) |
| `update_<svc>` | UPDATE writable fields; `status`, `id`, `created_at`, `tenant_id` excluded from input |
| `delete_<svc>` | Soft-delete (sets `deleted_at`); confirmation-gated when the `confirmation` feature is enabled |
| `list_<svc>` | Read tool (unchanged); excludes soft-deleted rows from results |

### Authorization

A `read_write`-scoped MCP session with `write_authorized: Some(true)` is required for `create_`/`update_`/`delete_` tools. Without it the call returns a `-32603` transport error before reaching the executor. The `list_` tool has no write-authorization requirement.

### Confirmation flow for delete

When the `confirmation` feature is enabled, a bare `delete_<svc>` call returns:

```json
{
  "error_kind": "confirmation_required",
  "request_tool": "request_confirm_delete_<svc>"
}
```

The agent must call `request_confirm_delete_<svc>` to obtain a token, then `confirm_delete_<svc>` with the token to execute the soft-delete. Tokens are single-use.

### Separation from developer-MCP CRUD tools

`ferro-mcp/src/tools/crud_operations.rs` is a separate developer-facing tool (`ferro mcp` CLI) that provides SQL-level model introspection. It is not related to the projection-derived consumer-MCP CRUD tools described here. Do not conflate them.
