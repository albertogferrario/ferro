# Data Model

The data model defines the schema for service projections. A **service projection** is a structural description of a domain entity: its fields, actions, guards, relationships, state machine, and intent hints.

All types in this section are:

- **JSON-serializable** -- every type round-trips through `serde_json` without loss
- **Transport-agnostic** -- the data model defines structure, not how it reaches consumers
- **Schema-validated** -- JSON Schema files are generated from the canonical Rust type definitions via [schemars](https://graham.cool/schemars/)

## Type Hierarchy

[`ServiceDef`](service-def.md) is the root type. All other types compose into it:

```
ServiceDef
  +-- name: String (required)
  +-- display_name: String (optional)
  +-- description: String (optional)
  +-- fields: FieldDef[]
  |     +-- data_type: DataType
  |     +-- meaning: FieldMeaning
  +-- actions: ActionDef[]
  |     +-- inputs: InputDef[]
  |     +-- preconditions: String[] (guard references)
  +-- guards: GuardDef[]
  +-- relationships: RelationshipDef[]
  |     +-- cardinality: Cardinality
  |     +-- navigation: NavigationHint
  +-- state_machine: StateMachine (optional)
  |     +-- states: StateDef[]
  |     +-- transitions: Transition[]
  +-- intent_hints: IntentHint[]
        +-- intent: Intent
```

## Canonical Definitions

The Rust types in `ferro-projections/src/` are the **canonical** protocol definitions. JSON Schema files in the [JSON Schema Reference](../appendix/json-schema.md) appendix are mechanically generated from these types and MUST NOT be hand-edited.

## Serialization Conventions

All types follow these serialization rules:

- Enum variants use `snake_case` serialization (e.g., `OneToMany` serializes as `"one_to_many"`)
- Optional fields (`Option<T>`) are omitted from output when `None`
- Empty vectors (`Vec<T>`) are omitted from output when empty
- Boolean fields with defaults use `#[serde(default)]` for backward-compatible deserialization
- `Custom(String)` fallback variants use `#[serde(untagged)]` -- they serialize as plain strings, not tagged objects

## Sections

| Page | Types Documented |
|------|-----------------|
| [ServiceDef](service-def.md) | `ServiceDef` |
| [Fields & Types](field-def.md) | `FieldDef`, `DataType`, `FieldMeaning` |
| [State Machines](state-machine.md) | `StateMachine`, `StateDef`, `Transition` |
| [Actions & Guards](actions.md) | `ActionDef`, `InputDef`, `GuardDef` |
| [Relationships](relationships.md) | `RelationshipDef`, `Cardinality`, `NavigationHint` |
| [Intents](intent.md) | `Intent`, `IntentScore`, `IntentHint` |
