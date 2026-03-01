# Actions & Guards

This page specifies three types: `ActionDef` (business operations), `InputDef` (action parameters), and `GuardDef` (named boolean conditions).

Actions are the verbs of a service: "submit order", "approve review", "send invoice". Guards are shared boolean conditions referenced from both action preconditions and state machine transitions.

## ActionDef

A business operation schema describing what an action does and what it needs.

> **JSON Schema:** [`action-def.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | Machine-readable action name (e.g., `"submit_order"`) |
| `display_name` | `string` | No | -- | Human-readable name (e.g., `"Submit Order"`) |
| `description` | `string` | No | -- | Description of what this action does |
| `inputs` | [`InputDef[]`](#inputdef) | No | `[]` | Input parameters required by this action |
| `preconditions` | `string[]` | No | `[]` | Guard names that must hold before this action can execute |
| `effects` | `string[]` | No | `[]` | Side effect descriptions that occur when this action executes |
| `transition_trigger` | `string` | No | -- | State machine event name this action triggers |

### Normative Rules

1. The `name` field MUST be a non-empty string.
2. Each string in `preconditions` MUST reference the `name` of a [`GuardDef`](#guarddef) in the parent [`ServiceDef.guards`](service-def.md) array. Undefined guard references are validation errors.
3. `transition_trigger`, when present, SHOULD reference an event name used in the parent [`ServiceDef.state_machine`](state-machine.md) transitions. An action with `transition_trigger` set but no state machine on the `ServiceDef` SHOULD produce a warning.
4. `effects` are descriptive string references for documentation and introspection. They are not executable.
5. `display_name`, `description`, `inputs` (when empty), `preconditions` (when empty), `effects` (when empty), and `transition_trigger` are omitted from JSON output when not set or empty.

### JSON Example

```json
{
  "name": "submit_order",
  "display_name": "Submit Order",
  "description": "Validates and submits a customer order for processing",
  "inputs": [
    {
      "name": "order_id",
      "data_type": "integer",
      "meaning": "identifier",
      "required": true
    },
    {
      "name": "notes",
      "data_type": "string",
      "meaning": "free_text",
      "required": false,
      "description": "Optional order notes"
    }
  ],
  "preconditions": ["has_items", "payment_valid"],
  "effects": ["notify_customer", "send_confirmation"],
  "transition_trigger": "submit"
}
```

---

## InputDef

An input parameter definition for an action. Reuses [`DataType`](field-def.md#datatype) and [`FieldMeaning`](field-def.md#fieldmeaning) to maintain a single type vocabulary across the entire projection schema. There are no parallel type systems for fields and inputs.

> **JSON Schema:** [`input-def.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | Parameter name (e.g., `"order_id"`) |
| `data_type` | [`DataType`](field-def.md#datatype) | Yes | -- | Structural data category (same enum as `FieldDef.data_type`) |
| `meaning` | [`FieldMeaning`](field-def.md#fieldmeaning) | Yes | -- | Semantic meaning (same enum as `FieldDef.meaning`) |
| `required` | `boolean` | No | `true` | Whether this input is required |
| `description` | `string` | No | -- | Description of this input parameter |

### Normative Rules

1. The `name` field MUST be a non-empty string.
2. `data_type` and `meaning` MUST use the same type vocabulary as [`FieldDef`](field-def.md). Implementations MUST NOT define separate type enums for inputs.
3. When `required` is omitted from input, consumers MUST default it to `true`.
4. `description` is omitted from JSON output when not set.

### JSON Example

```json
{
  "name": "email",
  "data_type": "string",
  "meaning": "email",
  "required": true,
  "description": "Customer email for confirmation"
}
```

---

## GuardDef

A named boolean condition that guards action execution or state transitions. Guards are declarative: they name a check that exists, but the evaluation logic lives outside the projection schema.

> **JSON Schema:** [`guard-def.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | Guard identifier (e.g., `"has_items"`) |
| `display_name` | `string` | No | -- | Human-readable name (e.g., `"Has Items"`) |
| `description` | `string` | No | -- | Description of what this guard checks |

### Normative Rules

1. The `name` field MUST be a non-empty string.
2. Guards are a **shared pool** within a `ServiceDef`. The same guard MAY be referenced from both [`Transition.guard`](state-machine.md#transition) and [`ActionDef.preconditions`](#actiondef).
3. A guard that is declared but never referenced from any transition or action precondition SHOULD produce an unused-guard warning.
4. `display_name` and `description` are omitted from JSON output when not set.

### JSON Example

```json
{
  "name": "has_items",
  "display_name": "Has Items",
  "description": "Order must contain at least one line item"
}
```

---

## Actions as Intent Signals

Actions contribute to intent derivation through their structural properties:

- **Transition triggers** linking actions to state machine events signal the **Process** intent -- explicit action-to-state coupling indicates workflow orchestration.
- **Multiple inputs** (more than 2) signal the **Collect** intent -- data-gathering actions suggest a form-oriented service.
- **Preconditions** signal the **Process** intent -- guarded actions indicate workflow complexity.
- **Simple CRUD actions** (few inputs, no preconditions, no transition triggers) provide weak **Browse** signals.
