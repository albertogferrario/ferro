# Validation

This section specifies the validation rules for `ServiceDef` and its subsystems. Validation ensures structural integrity before derivation or rendering is attempted.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this section are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

## Validation Entry Point

Validation is invoked via `ServiceDef::validate()`:

```
validate(service: ServiceDef) -> Result<Vec<Warning>, Error>
```

- On success, returns a (possibly empty) list of `Warning` values describing structural concerns.
- On failure, returns an `Error` describing a fatal issue that prevents further processing.

Implementations MUST validate all subsystems (fields, state machine, actions, guards, relationships, intent hints) in a single pass. State machine validation is subsumed by the service-level validation entry point.

## Error Variants (MUST Reject)

The following conditions are fatal errors. Implementations MUST reject a `ServiceDef` that exhibits any of these:

### Definition Errors

`Error::Definition(String)` -- Structural problems in the definition itself.

- Duplicate field names within a service.
- Missing required structural elements.

### Validation Errors

`Error::Validation(String)` -- Reference resolution failures and constraint violations.

**State machine reference errors:**

- `initial_state` is empty (not set).
- `initial_state` does not match any defined state name.
- A transition's `from` state does not match any defined state name.
- A transition's `to` state does not match any defined state name.

**Guard reference errors:**

- An action's `preconditions` reference a guard name that is not declared in the `guards` pool.
- A transition's `guard` references a guard name that is not declared in the `guards` pool.

**Transition trigger errors:**

- An action's `transition_trigger` does not match any state machine transition event name.

### Render Errors

`Error::Render(String)` -- Failures during the rendering phase (not validation per se, but part of the error type hierarchy).

### Serialization Errors

`Error::Serialization` -- JSON serialization/deserialization failures.

## Warning Variants (SHOULD Report)

The following conditions are structural concerns. Implementations SHOULD report them as warnings but MUST NOT reject the `ServiceDef`:

### UnreachableState

```
Warning::UnreachableState(state_name: String)
```

A state that is not reachable from `initial_state` via BFS traversal of transitions. See [BFS Reachability](#bfs-reachability) below.

### DeadEndState

```
Warning::DeadEndState(state_name: String)
```

A non-final state (`is_final == false`) with no outgoing transitions. This state, once entered, cannot be exited via the defined state machine.

### NoFinalStates

```
Warning::NoFinalStates
```

No states in the state machine are marked `is_final`. The state machine has no defined completion point.

### UnusedGuard

```
Warning::UnusedGuard(guard_name: String)
```

A guard defined in the `guards` pool that is not referenced by any transition guard or action precondition.

### TransitionTriggerWithoutStateMachine

```
Warning::TransitionTriggerWithoutStateMachine(action_name: String)
```

An action has a `transition_trigger` set, but the `ServiceDef` has no `state_machine` defined. The trigger has no effect.

### DuplicateRelationship

```
Warning::DuplicateRelationship(relationship_name: String)
```

Multiple relationships within the same service share the same `name`.

### ManyToManyWithForeignKey

```
Warning::ManyToManyWithForeignKey { relationship: String }
```

A relationship with `ManyToMany` cardinality has a `foreign_key` set. Many-to-many relationships use join tables, not direct foreign keys.

### ConflictingIntentHints

```
Warning::ConflictingIntentHints { intent: String }
```

The same intent appears in both `IntentHint::Primary` and `IntentHint::Exclude`. These hints contradict each other.

### MultiplePrimaryIntentHints

```
Warning::MultiplePrimaryIntentHints
```

More than one `IntentHint::Primary` is specified. Only one primary intent hint is meaningful.

## BFS Reachability

Implementations MUST check state reachability using breadth-first search (BFS) from the `initial_state`.

**Algorithm:**

1. Initialize a visited set containing `initial_state`.
2. Initialize a queue containing `initial_state`.
3. While the queue is not empty:
   a. Dequeue the current state.
   b. For each transition where `from` equals the current state:
      - If `to` is not in the visited set, add it and enqueue it.
4. Any state not in the visited set after BFS completes is unreachable.

States that are unreachable SHOULD produce `Warning::UnreachableState`. An unreachable state that is also non-final and has no outgoing transitions will produce both `UnreachableState` and `DeadEndState` warnings.

## Reference Resolution

All string references in a `ServiceDef` MUST resolve to defined entities. Undefined references are hard errors, not warnings.

The reference resolution checks are:

| Reference Site | Target Pool | Error on Failure |
|----------------|-------------|------------------|
| `transition.guard` | `guards[].name` | `Error::Validation` |
| `action.preconditions[]` | `guards[].name` | `Error::Validation` |
| `action.transition_trigger` | `state_machine.transitions[].event` | `Error::Validation` |
| `transition.from` | `state_machine.states[].name` | `Error::Validation` |
| `transition.to` | `state_machine.states[].name` | `Error::Validation` |
| `state_machine.initial_state` | `state_machine.states[].name` | `Error::Validation` |

## Cross-Subsystem Validation

Guards form a shared pool referenced from two subsystems:

1. **Transitions** -- `transition.guard` references a guard by name.
2. **Actions** -- `action.preconditions` reference guards by name.

Both reference sites MUST resolve against the same `guards` pool defined at the `ServiceDef` level. A guard not referenced by either site produces `Warning::UnusedGuard`.

Transition triggers bridge actions and the state machine:

- `action.transition_trigger` MUST match a `transition.event` in the state machine.
- If no state machine is defined but an action has a `transition_trigger`, this produces `Warning::TransitionTriggerWithoutStateMachine` (not an error, since the action can still function without the trigger).

## Validation Order

Implementations SHOULD validate in the following order to produce the most useful error messages:

1. State machine structural validation (initial state, state references, transition references).
2. Guard reference validation (action preconditions, transition guards).
3. Transition trigger validation (action triggers match state machine events).
4. Warning collection (unused guards, orphan triggers, duplicate relationships, many-to-many foreign keys, intent hint conflicts).

Fatal errors (steps 1-3) SHOULD be detected before warning collection (step 4). This avoids reporting warnings for a structurally invalid definition.
