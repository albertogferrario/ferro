# ServiceDef

`ServiceDef` is the protocol's root type. Every service projection is a single `ServiceDef` instance that describes a domain entity and its complete structural shape.

All other data model types compose into `ServiceDef`. A consumer receiving a `ServiceDef` has everything needed to derive intents, validate structure, and render UI.

> **JSON Schema:** [`service-def.json`](../appendix/json-schema.md)

## Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | Machine-readable identifier (e.g., `"order"`) |
| `display_name` | `string` | No | -- | Human-readable name (e.g., `"Order"`) |
| `description` | `string` | No | -- | Description of what this service models |
| `fields` | [`FieldDef[]`](field-def.md) | Yes | `[]` | Field definitions for the entity |
| `actions` | [`ActionDef[]`](actions.md) | No | `[]` | Business operations available on the entity |
| `guards` | [`GuardDef[]`](actions.md#guarddef) | No | `[]` | Named boolean conditions shared across actions and transitions |
| `relationships` | [`RelationshipDef[]`](relationships.md) | No | `[]` | Connections to other services |
| `state_machine` | [`StateMachine`](state-machine.md) | No | -- | Lifecycle state machine for the entity |
| `intent_hints` | [`IntentHint[]`](intent.md#intenthint) | No | `[]` | Manual overrides for intent derivation |

## Normative Rules

1. The `name` field MUST be a non-empty string.
2. The `name` field SHOULD use `snake_case` naming.
3. When `display_name` is absent, consumers SHOULD derive a display name from `name` (e.g., `"order_item"` becomes `"Order Item"`).
4. The `fields` array MAY be empty, but a `ServiceDef` with no fields provides no structural signals for intent derivation.
5. The `guards` array defines a shared pool. Guards MAY be referenced from both [`Transition.guard`](state-machine.md#transition) and [`ActionDef.preconditions`](actions.md#actiondef).
6. The `state_machine` field is optional. When absent, the service has no lifecycle states and Process/Track intent signals from state machine analysis are not produced.
7. The `intent_hints` array is optional. When present, hints override structural intent derivation as specified in the [Intents](intent.md) section.

## Serialization Rules

- `display_name`, `description`, and `state_machine` are omitted from JSON output when not set.
- `actions`, `guards`, `relationships`, and `intent_hints` are omitted from JSON output when empty.
- Field names use `snake_case` in the JSON representation.

## JSON Representation

```json
{
  "name": "order",
  "display_name": "Order",
  "description": "Manages customer orders",
  "fields": [
    {
      "name": "id",
      "data_type": "integer",
      "meaning": "identifier",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": false
    },
    {
      "name": "total",
      "data_type": "float",
      "meaning": "money",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": true
    },
    {
      "name": "status",
      "data_type": "string",
      "meaning": "status",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": false
    }
  ],
  "actions": [
    {
      "name": "submit",
      "display_name": "Submit Order",
      "inputs": [
        {
          "name": "notes",
          "data_type": "string",
          "meaning": "free_text",
          "required": false
        }
      ],
      "preconditions": ["has_items"],
      "transition_trigger": "submit"
    }
  ],
  "guards": [
    {
      "name": "has_items",
      "display_name": "Has Items",
      "description": "Order must contain at least one item"
    }
  ],
  "relationships": [
    {
      "name": "items",
      "target": "order_item",
      "cardinality": "one_to_many",
      "navigation": "nested"
    },
    {
      "name": "customer",
      "target": "customer",
      "cardinality": "many_to_one",
      "navigation": "link",
      "foreign_key": "customer_id"
    }
  ],
  "state_machine": {
    "name": "order_lifecycle",
    "initial_state": "draft",
    "states": [
      { "name": "draft", "display_name": "Draft" },
      { "name": "submitted", "display_name": "Submitted" },
      { "name": "completed", "display_name": "Completed", "is_final": true }
    ],
    "transitions": [
      {
        "from": "draft",
        "event": "submit",
        "to": "submitted",
        "guard": "has_items"
      },
      {
        "from": "submitted",
        "event": "complete",
        "to": "completed"
      }
    ]
  }
}
```

## Builder API (Informative)

> **Note:** The builder API is an implementation detail of the Rust reference implementation. It is informative, not normative. Other implementations MAY use any construction mechanism.

```rust
use ferro_projections::{ServiceDef, DataType, FieldMeaning};

let order = ServiceDef::new("order")
    .display_name("Order")
    .description("Manages customer orders")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("total", DataType::Float, FieldMeaning::Money)
    .optional_field("notes", DataType::String, FieldMeaning::FreeText);
```
