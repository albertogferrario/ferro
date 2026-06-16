# Transition Planning

A state-transition write needs one fact the write executor cannot invent: the target state. Transition planning derives that fact — and the guard guarding it — directly from the declared `StateMachine`, so the target state is declared once (on the `Transition`) and never re-stated in a hand-written `match action_name => new_status`.

`derive_transition_plan` is pure: it reads a `ServiceDef`'s state machine and one action, and returns a serializable `TransitionPlan`. It touches no database and runs no I/O.

## `derive_transition_plan`

```rust
use ferro::{derive_transition_plan, TransitionPlan};

let plan: TransitionPlan = derive_transition_plan(&service_def, "submit")?;
// plan.action     == "submit"
// plan.event      == "submit"          (= ActionDef.transition_trigger)
// plan.to_state   == "submitted"       (= Transition.to — the single source of truth)
// plan.from_states == ["draft"]        (every Transition.from carrying this event)
// plan.guard      == None              (Transition.guard, if any)
```

The write target comes from one place: the `Transition.to` of the transition whose event matches the action's `transition_trigger`. A consumer write handler derives `to_state` from the plan rather than hard-coding it, which removes the duplicated transition map that previously drifted from the state machine.

## `TransitionPlan`

```rust
pub struct TransitionPlan {
    pub action: String,          // ActionDef.name that produced this plan
    pub event: String,           // = ActionDef.transition_trigger; the StateMachine event name
    pub from_states: Vec<String>, // every Transition.from carrying this event (multi-source aware)
    pub to_state: String,        // Transition.to — the single target; replaces the hand-written match
    pub guard: Option<String>,   // Transition.guard, if any — re-checked live at execution
    pub effects: Vec<String>,    // Transition.actions ∪ ActionDef.effects
}
```

### Multi-source events

An event can be triggered from more than one source state. A `cancel` event reachable from both `draft` and `submitted` produces a plan whose `from_states` carries both:

```rust
// transitions:
//   draft     --cancel--> cancelled
//   submitted --cancel--> cancelled
let plan = derive_transition_plan(&service_def, "cancel")?;
// plan.from_states == ["draft", "submitted"]
// plan.to_state    == "cancelled"
```

This is valid: every source converges on the same target, so the target is unambiguous.

### Ambiguous fan-out is an error

If one event leads to *different* target states depending on the source, the target is not derivable from the event alone. This is reported as an error, never silently resolved:

```rust
// transitions:
//   draft     --close--> archived
//   submitted --close--> cancelled    // same event, different target
let result = derive_transition_plan(&service_def, "close");
// Err(Error::AmbiguousTransition { event: "close" })
```

An action that fans out to multiple targets must be split into distinct events.

### Error modes

| Error | Condition |
|-------|-----------|
| `Error::Validation` | No action named `action_name` on the service |
| `Error::NoTransitionTrigger` | The action declares no `transition_trigger` |
| `Error::NoStateMachine` | The service has no state machine |
| `Error::UndeclaredTransition { action, event }` | The trigger names an event no transition carries (the drift condition) |
| `Error::AmbiguousTransition { event }` | The event fans out to more than one target state |

## Registration-time drift gate

The "declare once" guarantee is enforced at boot, not at runtime. `ServiceDef::validate()` runs the same derivation the runtime uses for every transition-triggering action. If any `ActionDef.transition_trigger` names a transition the state machine does not declare, `validate()` returns `Err` — so an executor that disagrees with the state machine fails at registration, before any request is served.

```rust
let result = service_def.validate();
// Err if any action.transition_trigger references an event the StateMachine
// does not declare, or if any such action derives an ambiguous transition.
```

Because `validate()` builds a `TransitionPlan` for every transition-triggering action, `validate()` accepting an action implies the derivation can build a plan for it. Drift between the two is structurally impossible: a `transition_trigger` MUST reference a declared transition.

## Driving a write

A write handler combines transition planning with the [write kernel](write-kernel.md). The handler derives the transition guard from the plan and hands it to `dispatch_write`; the executor derives `to_state` the same way, so the target state is never read from the request body:

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
    transition_guard,  // derived from the StateMachine, not the request
    "web",
    #[cfg(feature = "confirmation")]
    false,
)
.await;
```

## Reference

| Item | Description |
|------|-------------|
| `derive_transition_plan(svc, action_name)` | Pure derivation of a `TransitionPlan` from the service's state machine |
| `TransitionPlan` | The derived transition facts: `action`, `event`, `from_states`, `to_state`, `guard`, `effects` |
| `ServiceDef::validate()` | Returns `Err` if any `transition_trigger` names an undeclared or ambiguous transition (the drift gate) |

## See also

- [Write Kernel](write-kernel.md) — executing the plan with guard re-eval, idempotency, audit, and override.
- [Service Projections](projections.md) — the `ServiceDef`, state machine, and actions the plan is derived from.
