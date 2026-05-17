# Service Projections

Service Projections derive a JSON-UI layout automatically from the structure and semantics of your data. Define what your data IS — fields, types, meanings, and workflow states — and the framework infers the appropriate UI intent and generates a component tree.

## Overview

The pipeline has three stages:

```
ServiceDef  →  derive_intents(&service_def)  →  IntentScore[]
                                                      ↓
JsonUiRenderer.render(&service_def, &intents, &ctx)  →  serde_json::Value
```

1. **ServiceDef** — describe your service: field names, data types, semantic meanings, state machines, guards, and actions.
2. **derive_intents** — analyzes the service definition and returns a ranked list of `IntentScore` values. The highest-scoring intent is the primary one.
3. **JsonUiRenderer** — takes the service definition, the ranked intents, and a render context, and produces a ferro-json-ui component tree.

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
| `DataType::String` | Short text: names, titles, codes |
| `DataType::Boolean` | True/false flags |
| `DataType::Date` | Calendar date (no time) |
| `DataType::DateTime` | Date plus time |
| `DataType::Text` | Long-form prose: descriptions, body content |
| `DataType::Enum` | Fixed set of values: status, category |

**FieldMeaning:**

| Variant | When to use |
|---------|-------------|
| `FieldMeaning::Identifier` | Primary key or unique ID |
| `FieldMeaning::EntityName` | Display name of the record |
| `FieldMeaning::Money` | Monetary amount |
| `FieldMeaning::Description` | Long descriptive text |
| `FieldMeaning::Status` | Current state or lifecycle value |
| `FieldMeaning::Email` | Email address |
| `FieldMeaning::Phone` | Phone number |
| `FieldMeaning::Url` | Web URL |
| `FieldMeaning::Image` | Image URL or path |
| `FieldMeaning::Timestamp` | Created/updated timestamps |
| `FieldMeaning::Count` | Aggregate count |
| `FieldMeaning::Location` | Geographic location |
| `FieldMeaning::Generic` | No specific semantic meaning |

### Intent Derivation

`derive_intents` examines five structural signals to score and rank the seven intents:

**Signal analyzers:**

1. **Field count** — many fields suggest a form (Collect); few fields suggest a list (Browse).
2. **Field meanings** — presence of `Money`, `Status`, `Count` meanings shifts scores toward specific intents.
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

`JsonUiRenderer` implements the `Renderer` trait. Use it with a `RenderContext` to control how the output is shaped.

```rust
use ferro::{JsonUiRenderer, RenderContext, RenderMode, Renderer};

// Display mode: read-only view of data
let display_ctx = RenderContext {
    intent_index: 0,              // use primary intent
    current_state: None,          // no workflow state active
    mode: RenderMode::Display,    // read-only layout
    templates: None,              // use default layouts
};

// Input mode: editable form
let input_ctx = RenderContext {
    intent_index: 0,
    current_state: Some("draft".to_string()), // current workflow state
    mode: RenderMode::Input,                  // form layout
    templates: None,
};

let renderer = JsonUiRenderer;
let json = renderer.render(&service_def, &intents, &input_ctx).expect("rendering a valid service definition should not fail");
```

**RenderContext fields:**

| Field | Type | Description |
|-------|------|-------------|
| `intent_index` | `usize` | Index into the `IntentScore` list; `0` for primary intent |
| `current_state` | `Option<String>` | Active state name from the state machine, if applicable |
| `mode` | `RenderMode` | `RenderMode::Display` for read-only; `RenderMode::Input` for forms |
| `templates` | `Option<...>` | Custom layout overrides (from `theme.json`); `None` uses defaults |

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
    JsonUiRenderer, RenderContext, RenderMode, Renderer,
    ServiceDef, StateDef, StateMachine, Transition,
    derive_intents,
};

let order = ServiceDef::new("order")
    .display_name("Order")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("customer", DataType::String, FieldMeaning::EntityName)
    .field("total", DataType::Float, FieldMeaning::Money)
    .field("status", DataType::Enum, FieldMeaning::Status)
    .field("created_at", DataType::DateTime, FieldMeaning::Timestamp)
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

let ctx = RenderContext {
    intent_index: 0,
    current_state: Some("draft".to_string()),
    mode: RenderMode::Input,
    templates: None,
};

let json = JsonUiRenderer.render(&order, &intents, &ctx).expect("rendering a valid service definition should not fail");
// Produces a ferro-json-ui component tree with:
// - Fields rendered as form inputs
// - Available transitions ("approve") rendered as action buttons
// - Guard labels displayed on the action
```

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
| `JsonUiRenderer` | Implements `Renderer`; converts `ServiceDef` + intents + context to JSON-UI |
| `Renderer` | Trait implemented by renderers; one method: `render(def, intents, ctx)` |
| `RenderContext` | Render parameters: intent index, current state, mode, template overrides |
| `RenderMode` | `Display` for read-only output; `Input` for editable form output |

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
