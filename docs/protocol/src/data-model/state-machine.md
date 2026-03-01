# State Machines

This page specifies three types: `StateMachine` (lifecycle definition), `StateDef` (individual state), and `Transition` (edge between states).

State machines are optional on [`ServiceDef`](service-def.md). When present, they define the lifecycle of a domain entity and serve as structural signals for Process and Track intent derivation.

**Schema-only constraint:** State machines define structure but do not execute transitions. Guards and side effects are string references resolved externally at runtime. This constraint enables full serialization and introspection.

## StateMachine

A state machine schema describing lifecycle states and transitions.

> **JSON Schema:** [`state-machine.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | Machine identifier (e.g., `"order_lifecycle"`) |
| `display_name` | `string` | No | -- | Human-readable name |
| `description` | `string` | No | -- | Description of what this state machine models |
| `initial_state` | `string` | Yes | -- | Name of the initial state (MUST reference an existing state) |
| `states` | [`StateDef[]`](#statedef) | Yes | `[]` | State definitions |
| `transitions` | [`Transition[]`](#transition) | Yes | `[]` | Transition definitions |

### Normative Rules

1. The `initial_state` MUST reference the `name` of an existing state in the `states` array.
2. The `initial_state` MUST NOT be empty.
3. All state names referenced in `transitions` (both `from` and `to`) MUST exist in the `states` array. Undefined references are validation errors.
4. States unreachable from `initial_state` via BFS traversal of transitions SHOULD produce a warning, not an error. Unreachable states may be entered through external means.
5. Non-final states with no outgoing transitions SHOULD produce a dead-end warning.
6. A state machine with no final states SHOULD produce a warning.
7. `display_name` and `description` are omitted from JSON output when not set.

### JSON Example

An order lifecycle with three states:

```json
{
  "name": "order_lifecycle",
  "initial_state": "draft",
  "states": [
    {
      "name": "draft",
      "display_name": "Draft"
    },
    {
      "name": "submitted",
      "display_name": "Submitted",
      "on_enter": ["validate_inventory", "calculate_totals"]
    },
    {
      "name": "completed",
      "display_name": "Completed",
      "is_final": true
    }
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
```

---

## StateDef

A state within a state machine. States can declare entry/exit side effects as string references and carry optional metadata for rendering hints.

> **JSON Schema:** [`state-def.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | State identifier (e.g., `"draft"`, `"pending"`, `"completed"`) |
| `display_name` | `string` | No | -- | Human-readable name (e.g., `"Pending Review"`) |
| `description` | `string` | No | -- | Description of what this state means |
| `is_final` | `boolean` | No | `false` | Whether this is a terminal state |
| `on_enter` | `string[]` | No | `[]` | Side effects triggered on entering this state (string references) |
| `on_exit` | `string[]` | No | `[]` | Side effects triggered on exiting this state (string references) |
| `metadata` | `JSON value` | No | -- | Arbitrary metadata for rendering hints (e.g., colors, icons) |

### Normative Rules

1. The `name` field MUST be a non-empty string.
2. Final states (`is_final: true`) represent terminal lifecycle points. Renderers SHOULD indicate finality visually.
3. `on_enter` and `on_exit` are string references to side effects resolved externally. They are structural documentation, not executable code.
4. `metadata` is an arbitrary JSON value. The protocol does not constrain its structure. Consumers SHOULD ignore unrecognized metadata keys.
5. The presence of `metadata` prevents the `Eq` trait derivation in the reference implementation because `serde_json::Value` does not implement `Eq`.
6. `display_name`, `description`, `on_enter` (when empty), `on_exit` (when empty), and `metadata` are omitted from JSON output when not set or empty.

### JSON Example

```json
{
  "name": "processing",
  "display_name": "Processing",
  "description": "Order is being processed",
  "is_final": false,
  "on_enter": ["charge_payment", "reserve_inventory"],
  "on_exit": ["release_hold"],
  "metadata": { "color": "blue", "icon": "gear" }
}
```

---

## Transition

A transition between two states, triggered by a named event. Guards gate the transition; actions fire when the transition is taken. Both are string references resolved externally.

> **JSON Schema:** [`transition.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `from` | `string` | Yes | -- | Source state name |
| `event` | `string` | Yes | -- | Event name that triggers this transition |
| `to` | `string` | Yes | -- | Target state name |
| `guard` | `string` | No | -- | Guard condition name (string reference, resolved at runtime) |
| `actions` | `string[]` | No | `[]` | Side effects executed when the transition fires (string references) |
| `description` | `string` | No | -- | Description of what this transition represents |

### Normative Rules

1. `from` and `to` MUST reference existing state names in the parent `StateMachine.states` array.
2. `guard` is a string reference to a [`GuardDef`](actions.md#guarddef) in the parent [`ServiceDef.guards`](service-def.md) array. When present, the guard MUST exist in the guards pool.
3. Multiple transitions MAY share the same `event` name with different `from` states (e.g., a "cancel" event available from multiple states).
4. Multiple transitions from the same `from` state with different events define the available actions in that state.
5. `guard`, `actions` (when empty), and `description` are omitted from JSON output when not set or empty.

### JSON Example

```json
{
  "from": "draft",
  "event": "submit",
  "to": "submitted",
  "guard": "has_items",
  "actions": ["validate_fields", "log_submission"],
  "description": "Customer submits the order for processing"
}
```

---

## State Machine as Intent Signal

The presence and shape of a state machine is a structural signal for intent derivation:

- **Guard density** (ratio of guarded transitions to total transitions) signals the **Process** intent -- complex workflows have more decision points.
- **Branching states** (states with multiple outgoing transitions) signal **Process** -- branching indicates workflow complexity.
- **Transition triggers** linking actions to state machine events signal **Process** -- explicit action-to-state coupling indicates workflow orchestration.
- **Linear progression** (states forming a chain with minimal branching) signals **Track** -- linear workflows are monitored rather than managed.
- **Final states** signal **Track** -- the existence of terminal states implies progress tracking.
