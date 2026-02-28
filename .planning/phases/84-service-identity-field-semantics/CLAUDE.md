# ferro-projections Crate Instructions

Copy this file to `ferro-projections/CLAUDE.md` when creating the crate in Phase 84.

## Crate Purpose

Schema-only service definitions. No runtime engines, no closures. Everything serializable and introspectable.

## Key Conventions

### Naming
- Types: `ServiceDef`, `FieldDef`, `FieldMeaning`, `StateMachine`, `ActionDef`
- Builders: method chaining returning `&mut Self`
- Guards/side effects: always strings, never closures

### Serialization
- All public types derive `Serialize, Deserialize, Debug, Clone`
- Use `#[serde(rename_all = "snake_case")]` on enums
- Custom(String) variants use `#[serde(untagged)]` where appropriate

### Module Structure
```
src/
  lib.rs          — re-exports, ServiceDef builder
  service.rs      — ServiceDef, ServiceDefBuilder
  field.rs        — FieldDef, FieldMeaning enum
  state.rs        — StateMachine, Transition
  action.rs       — ActionDef, Precondition
  relationship.rs — Relationship, Cardinality
  intent.rs       — Intent enum (Orientation/Action/Movement)
  resolved.rs     — ResolvedField
  graph.rs        — IntentGraph, IntentNode, IntentContext
  renderer.rs     — Renderer trait, RenderOutput
  renderers/
    mod.rs
    json_ui.rs    — JsonUiRenderer
    field.rs      — FieldMeaning → component mapping
```

### Builder Pattern
```rust
ServiceDef::new("order")
    .display_name("Order")
    .field("total", DataType::Float, FieldMeaning::Money)
    .field("status", DataType::String, FieldMeaning::Status)
```

### Testing
- Every type: construction + serde round-trip
- Builders: chain validation
- State machines: unreachable state detection
- Intent graphs: edge correctness

### Anti-patterns
- No closures in definitions (breaks serialization)
- No runtime logic in ServiceDef (it's a schema)
- No trait objects for guards (use string references)
- No Default implementations that hide required fields
