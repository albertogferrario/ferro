# ferro-projections Crate Instructions

## Crate Purpose

Schema-only service definitions. No runtime engines, no closures. Everything serializable and introspectable.

## Key Conventions

### Naming
- Types: `ServiceDef`, `FieldDef`, `FieldMeaning`, `StateMachine`, `ActionDef`
- Builders: method chaining with `mut self -> Self` (consuming builder)
- Guards/side effects: always strings, never closures

### Serialization
- All public types derive `Serialize, Deserialize, Debug, Clone`
- Use `#[serde(rename_all = "snake_case")]` on enums
- Custom(String) variants use `#[serde(untagged)]` where appropriate

### Module Structure
```
src/
  lib.rs          — re-exports, crate docs
  service.rs      — ServiceDef
  field.rs        — FieldDef, FieldMeaning, DataType, infer_meaning
  error.rs        — Error enum
  state.rs        — StateMachine, Transition (future)
  action.rs       — ActionDef, Precondition (future)
  relationship.rs — Relationship, Cardinality (future)
  intent.rs       — Intent enum (future)
  resolved.rs     — ResolvedField (future)
  graph.rs        — IntentGraph, IntentNode, IntentContext (future)
  renderer.rs     — Renderer trait, RenderOutput (future)
  renderers/      — Renderer implementations (future)
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
