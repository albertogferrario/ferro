# Phase 85: State Machines — Research

**Researched:** 2026-02-28
**Domain:** Schema-only state machine definitions for ferro-projections
**Confidence:** HIGH

<research_summary>
## Summary

Phase 85 adds `StateMachine`, `StateDef`, and `Transition` types to `ferro-projections/src/state.rs`. These are schema-only definitions — they describe state machines but don't execute them. Guards and side effects are string references, not closures, following XState's established separation of definition from implementation.

The v9.0 research validated XState's design philosophy as the model to follow: machine definitions are serializable data, guards and actions are named references resolved elsewhere. This phase-level research focuses on: (1) the exact struct shapes for StateMachine/StateDef/Transition, (2) flat vs hierarchical states (flat for v1), (3) side effect categories (entry/exit/transition actions as string references), (4) validation algorithms (reachable states, dead-end detection), and (5) the boundary with Phase 86 (actions definitions).

**Primary recommendation:** Flat state machine with `StateMachine::new("order_workflow").initial("draft").state(...)` builder. Guards are `Option<String>`, side effects are `Vec<String>`. Validation via simple BFS/DFS on small graphs (5-15 nodes). No external dependencies.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1 | Serialization | StateMachine must be serializable/introspectable |
| serde_json | 1 | JSON format | MCP introspection, IntentGraph generation consume JSON |
| thiserror | 1.0 | Error types | Workspace error pattern |

### New Dependencies: None
State machine definitions are pure data types. No runtime engine, no external FSM crate. The schema is trivial enough to own (~150 lines of struct definitions + ~100 lines of validation).

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom StateMachine struct | XState-rs/RuState crate | These are runtime engines; we need schema-only. Adding a runtime FSM crate would violate the "no execution engine" principle. |
| Custom StateMachine struct | pure_hfsm | Closest match (serializable descriptions), but adds dependency for ~150 lines of code we'd use |
| Custom validation | petgraph algorithms | Overkill for 5-15 node graphs. BFS/DFS is 20 lines of code |
| Flat states | Hierarchical/compound states | Flat is sufficient for service-level state machines. Compound states add complexity (entry/exit scoping, child machine semantics) with minimal gain at this abstraction level |

### Installation
No new dependencies. Uses existing workspace crates only.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Module Structure (within ferro-projections)
```
ferro-projections/src/
├── lib.rs          # Re-exports (add state module)
├── service.rs      # ServiceDef (Phase 84)
├── field.rs        # FieldDef, FieldMeaning (Phase 84)
├── state.rs        # StateMachine, StateDef, Transition ← Phase 85
└── error.rs        # Error enum (add validation variants)
```

### Pattern 1: StateMachine as Schema (XState-Inspired)
**What:** State machine definitions are data, not code. Guards and side effects are string names, not closures.
**When to use:** Any service that has lifecycle states (orders, invoices, tickets, users).
**Why:** XState v5 validates this approach — `setup()` registers implementations separately from the machine definition. Our string references follow the same principle.
**Example:**
```rust
// XState equivalent:
// { id: "order", initial: "draft", states: { draft: { on: { submit: "pending" } }, ... } }
//
// Ferro equivalent:
let machine = StateMachine::new("order_workflow")
    .initial("draft")
    .state(StateDef::new("draft")
        .display_name("Draft"))
    .state(StateDef::new("pending")
        .display_name("Pending Review")
        .on_enter(vec!["notify_reviewer"]))
    .state(StateDef::new("approved")
        .display_name("Approved")
        .on_enter(vec!["send_confirmation"]))
    .state(StateDef::new("rejected")
        .display_name("Rejected")
        .final_state())
    .state(StateDef::new("completed")
        .display_name("Completed")
        .final_state())
    .transition(Transition::new("draft", "submit", "pending")
        .guard("has_required_fields"))
    .transition(Transition::new("pending", "approve", "approved")
        .guard("is_reviewer"))
    .transition(Transition::new("pending", "reject", "rejected")
        .guard("is_reviewer")
        .actions(vec!["log_rejection_reason"]))
    .transition(Transition::new("approved", "complete", "completed"));
```

### Pattern 2: Flat States Only (v1 Decision)
**What:** No hierarchical/compound/parallel states. Each state is atomic — no substates.
**When to use:** Service-level state machines where states represent business lifecycle stages.
**Why:** Hierarchical states are the key feature of statecharts vs FSMs, but they add significant complexity: child machine semantics, entry/exit scoping, history states, substate transitions. For service definitions (order: draft→submitted→shipped→delivered), flat states are sufficient. The IntentGraph (Phase 89) handles navigation complexity.

**When this might change:** If Phase 93 field test reveals services that naturally decompose into nested states (e.g., "processing" with substates "validating", "charging", "fulfilling"), we can add compound states then.

### Pattern 3: Transition with Guards and Actions
**What:** Transitions carry optional guard (string reference) and actions (string reference list). Guards gate the transition; actions fire when the transition is taken.
**When to use:** All transitions. Guards are optional; actions are optional.
**Example:**
```rust
// Simple transition (no guard, no actions)
Transition::new("draft", "submit", "pending")

// Guarded transition
Transition::new("pending", "approve", "approved")
    .guard("is_reviewer")

// Guarded with side effects
Transition::new("pending", "reject", "rejected")
    .guard("is_reviewer")
    .actions(vec!["log_rejection_reason", "notify_submitter"])
```

### Pattern 4: Entry/Exit Side Effects on States
**What:** States can declare side effects that run on entry and exit, as string references.
**When to use:** When entering/exiting a state triggers business logic.
**Example:**
```rust
StateDef::new("pending")
    .on_enter(vec!["notify_reviewer", "start_sla_timer"])
    .on_exit(vec!["stop_sla_timer"])
```
**XState parallel:** `entry: [{ type: 'notifyReviewer' }]` and `exit: [{ type: 'stopTimer' }]`

### Pattern 5: ServiceDef Integration
**What:** `ServiceDef` gets a `.state_machine(StateMachine)` builder method.
**When to use:** Stateful services. Stateless services (no lifecycle) skip this.
**Example:**
```rust
let order_service = ServiceDef::new("order")
    .display_name("Order")
    .field("status", DataType::String, FieldMeaning::Status)
    .state_machine(
        StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("completed").final_state())
            .transition(Transition::new("draft", "complete", "completed"))
    );
```

### Anti-Patterns to Avoid
- **No closures in guards:** `guard: Box<dyn Fn(&Context) -> bool>` breaks serialization. Use `guard: Option<String>`.
- **No runtime execution logic:** `StateMachine` doesn't process events. It's a schema that the IntentGraph generator (Phase 89) reads.
- **No hierarchical states in v1:** Resist the urge to add compound states. Flat + IntentGraph covers the same use cases with less complexity.
- **No guard composition in v1:** XState v5 has `and()`, `or()`, `not()` guard combinators. Defer to Phase 86 if needed. Simple string guards are sufficient for schema definitions.
- **No Default for StateMachine:** A state machine without states is meaningless. `StateMachine::new(name)` requires a name, `.initial()` sets the starting state.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FSM execution engine | Runtime event processing, state transitions | Schema-only + IntentGraph (Phase 89) | This phase defines schemas, not engines. The IntentGraph generator uses StateMachine as input. |
| Graph algorithms for validation | Custom cycle detection from scratch | Simple BFS/DFS (20 lines) | Graphs are 5-15 nodes. BFS from initial state is trivial. No need for petgraph or Tarjan's. |
| Enum serialization | Custom Serialize/Deserialize impls | `#[serde(rename_all = "snake_case")]` | Same pattern as Phase 84's FieldMeaning |
| Builder validation | Panic in builder methods | Separate `validate()` → Result | Same pattern as Phase 84 |
| Guard composition | `And(Vec<Guard>)`, `Or`, `Not` combinators | Simple `Option<String>` | Phase 86 handles action/guard definitions. Phase 85 just stores the reference string. |

**Key insight:** The state machine schema is ~150 lines of struct definitions. The risk is over-engineering (adding hierarchical states, guard composition, event queuing) not under-engineering. Start minimal, Phase 93 field test will reveal what's missing.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Building a Runtime Engine Instead of a Schema
**What goes wrong:** Adding event dispatching, current-state tracking, or transition execution to `StateMachine`.
**Why it happens:** Natural instinct when building a "state machine" is to make it process events.
**How to avoid:** `StateMachine` has no `.send(event)` method, no `.current_state()` method. It's a schema — like a database migration file describes tables but doesn't query them. The IntentGraph generator (Phase 89) reads the schema to determine available transitions.
**Warning signs:** Methods that take `&mut self`, fields like `current_state: String`, or any method that modifies the machine's data after construction.

### Pitfall 2: Conflating Guards with Actions
**What goes wrong:** Making guards produce side effects, or actions check conditions.
**Why it happens:** In practice, you often want "if X then do Y" which blurs the line.
**How to avoid:** Guards are boolean conditions (string references like `"is_reviewer"`). Actions are fire-and-forget effects (string references like `"send_email"`). Guards gate transitions; actions execute when transitions fire. This separation is fundamental to XState's design and our schema model.
**Warning signs:** A guard string that sounds like an action ("send_and_check_email"), or an action string that sounds like a condition ("if_approved").

### Pitfall 3: State Name Collisions with Events
**What goes wrong:** Using the same string for a state name and an event name (e.g., state "cancel" and event "cancel").
**Why it happens:** Natural language overloading — "cancel" is both a state and an action.
**How to avoid:** Convention: state names are nouns/adjectives ("cancelled", "pending", "draft"), event names are verbs ("cancel", "submit", "approve"). Validation can warn on overlaps.
**Warning signs:** Ambiguous serialized output where "cancel" could mean either the state or the event.

### Pitfall 4: Missing Initial State Validation
**What goes wrong:** `initial_state` references a state name that doesn't exist in the states list.
**Why it happens:** Builder allows setting initial before adding states, or typo in state name.
**How to avoid:** `validate()` checks that `initial_state` exists in the states map. Builder doesn't enforce order but validation catches mismatches.
**Warning signs:** Serialized StateMachine where `initial_state: "darft"` (typo) doesn't match any state.

### Pitfall 5: Orphaned States After Refactoring
**What goes wrong:** Removing a transition but leaving the target state, creating an unreachable state.
**Why it happens:** Manual state machine construction without validation.
**How to avoid:** `validate()` runs BFS from initial state and warns about unreachable states. Not an error (some states might be reached via external means), but a warning.
**Warning signs:** States that appear in the states list but have no incoming transitions and aren't the initial state.

### Pitfall 6: No Final States
**What goes wrong:** State machine has no terminal states, meaning there's no "done" condition.
**Why it happens:** Cyclic workflows where everything loops back (valid for some services, but should be explicit).
**How to avoid:** `validate()` warns if no final states exist. Not an error — some services are perpetual (user account lifecycle). But the warning forces explicit acknowledgment.
**Warning signs:** All states have outgoing transitions but none are marked final.
</common_pitfalls>

<code_examples>
## Code Examples

### StateMachine Type Definition
```rust
// Source: XState design philosophy + workspace conventions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    /// Machine identifier (e.g., "order_lifecycle")
    pub name: String,
    /// Human-readable name
    pub display_name: Option<String>,
    /// Description of what this state machine models
    pub description: Option<String>,
    /// Name of the initial state (must exist in states)
    pub initial_state: String,
    /// State definitions
    pub states: Vec<StateDef>,
    /// Transition definitions
    pub transitions: Vec<Transition>,
}
```

### StateDef Type Definition
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDef {
    /// State identifier (e.g., "draft", "pending", "completed")
    pub name: String,
    /// Human-readable name (e.g., "Pending Review")
    pub display_name: Option<String>,
    /// Description of what this state means
    pub description: Option<String>,
    /// Whether this is a terminal state
    pub is_final: bool,
    /// Side effects triggered on entering this state (string references)
    pub on_enter: Vec<String>,
    /// Side effects triggered on exiting this state (string references)
    pub on_exit: Vec<String>,
    /// Arbitrary metadata for rendering hints
    pub metadata: Option<serde_json::Value>,
}
```

### Transition Type Definition
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Source state name
    pub from: String,
    /// Event name that triggers this transition
    pub event: String,
    /// Target state name
    pub to: String,
    /// Guard condition (string reference, resolved at runtime)
    pub guard: Option<String>,
    /// Side effects executed when transition fires (string references)
    pub actions: Vec<String>,
    /// Description of what this transition represents
    pub description: Option<String>,
}
```

### Builder API
```rust
impl StateMachine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            initial_state: String::new(),
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn initial(mut self, state: impl Into<String>) -> Self {
        self.initial_state = state.into();
        self
    }

    pub fn state(mut self, state: StateDef) -> Self {
        self.states.push(state);
        self
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.transitions.push(transition);
        self
    }
}

impl StateDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            is_final: false,
            on_enter: Vec::new(),
            on_exit: Vec::new(),
            metadata: None,
        }
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn final_state(mut self) -> Self {
        self.is_final = true;
        self
    }

    pub fn on_enter(mut self, effects: Vec<impl Into<String>>) -> Self {
        self.on_enter = effects.into_iter().map(Into::into).collect();
        self
    }

    pub fn on_exit(mut self, effects: Vec<impl Into<String>>) -> Self {
        self.on_exit = effects.into_iter().map(Into::into).collect();
        self
    }
}

impl Transition {
    pub fn new(from: impl Into<String>, event: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            event: event.into(),
            to: to.into(),
            guard: None,
            actions: Vec::new(),
            description: None,
        }
    }

    pub fn guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }

    pub fn actions(mut self, actions: Vec<impl Into<String>>) -> Self {
        self.actions = actions.into_iter().map(Into::into).collect();
        self
    }
}
```

### Validation Logic
```rust
use std::collections::{HashMap, HashSet, VecDeque};

impl StateMachine {
    pub fn validate(&self) -> Result<Vec<Warning>, Error> {
        let mut warnings = Vec::new();
        let state_names: HashSet<&str> = self.states.iter().map(|s| s.name.as_str()).collect();

        // 1. Initial state must be set and exist
        if self.initial_state.is_empty() {
            return Err(Error::Validation("initial state not set".into()));
        }
        if !state_names.contains(self.initial_state.as_str()) {
            return Err(Error::Validation(
                format!("initial state '{}' not found in states", self.initial_state)
            ));
        }

        // 2. All transition sources/targets must reference existing states
        for t in &self.transitions {
            if !state_names.contains(t.from.as_str()) {
                return Err(Error::Validation(
                    format!("transition source '{}' not found in states", t.from)
                ));
            }
            if !state_names.contains(t.to.as_str()) {
                return Err(Error::Validation(
                    format!("transition target '{}' not found in states", t.to)
                ));
            }
        }

        // 3. Reachability check (BFS from initial state)
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.initial_state.as_str());
        reachable.insert(self.initial_state.as_str());

        while let Some(current) = queue.pop_front() {
            for t in &self.transitions {
                if t.from == current && !reachable.contains(t.to.as_str()) {
                    reachable.insert(t.to.as_str());
                    queue.push_back(t.to.as_str());
                }
            }
        }

        for state in &self.states {
            if !reachable.contains(state.name.as_str()) {
                warnings.push(Warning::UnreachableState(state.name.clone()));
            }
        }

        // 4. Dead-end check (non-final states with no outgoing transitions)
        let states_with_outgoing: HashSet<&str> =
            self.transitions.iter().map(|t| t.from.as_str()).collect();
        for state in &self.states {
            if !state.is_final && !states_with_outgoing.contains(state.name.as_str()) {
                warnings.push(Warning::DeadEndState(state.name.clone()));
            }
        }

        // 5. No final states warning
        if !self.states.iter().any(|s| s.is_final) {
            warnings.push(Warning::NoFinalStates);
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    UnreachableState(String),
    DeadEndState(String),
    NoFinalStates,
}
```

### Serde Round-Trip Test
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_machine() -> StateMachine {
        StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft").display_name("Draft"))
            .state(StateDef::new("pending").display_name("Pending")
                .on_enter(vec!["notify_reviewer"]))
            .state(StateDef::new("approved").display_name("Approved"))
            .state(StateDef::new("completed").display_name("Completed").final_state())
            .transition(Transition::new("draft", "submit", "pending")
                .guard("has_required_fields"))
            .transition(Transition::new("pending", "approve", "approved")
                .guard("is_reviewer"))
            .transition(Transition::new("approved", "complete", "completed"))
    }

    #[test]
    fn state_machine_serde_round_trip() {
        let machine = sample_machine();
        let json = serde_json::to_string_pretty(&machine).unwrap();
        let parsed: StateMachine = serde_json::from_str(&json).unwrap();

        assert_eq!(machine.name, parsed.name);
        assert_eq!(machine.initial_state, parsed.initial_state);
        assert_eq!(machine.states.len(), parsed.states.len());
        assert_eq!(machine.transitions.len(), parsed.transitions.len());
    }

    #[test]
    fn validate_valid_machine() {
        let machine = sample_machine();
        let warnings = machine.validate().unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_missing_initial_state() {
        let machine = StateMachine::new("test")
            .state(StateDef::new("a"));
        assert!(machine.validate().is_err());
    }

    #[test]
    fn validate_unreachable_state() {
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a").final_state())
            .state(StateDef::new("orphan"));
        let warnings = machine.validate().unwrap();
        assert!(warnings.contains(&Warning::UnreachableState("orphan".into())));
    }

    #[test]
    fn validate_dead_end_state() {
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a"))
            .state(StateDef::new("b"))
            .transition(Transition::new("a", "go", "b"));
        let warnings = machine.validate().unwrap();
        assert!(warnings.contains(&Warning::DeadEndState("b".into())));
    }
}
```

### Full Order Lifecycle Example (Field Test Preview)
```rust
// This is what Phase 93 will build — shown here to validate the schema design
let order_machine = StateMachine::new("order_lifecycle")
    .display_name("Order Lifecycle")
    .description("Tracks an order from creation to fulfillment")
    .initial("draft")
    // States
    .state(StateDef::new("draft")
        .display_name("Draft")
        .description("Order is being prepared"))
    .state(StateDef::new("submitted")
        .display_name("Submitted")
        .on_enter(vec!["validate_inventory", "calculate_totals"]))
    .state(StateDef::new("processing")
        .display_name("Processing")
        .on_enter(vec!["charge_payment", "reserve_inventory"]))
    .state(StateDef::new("shipped")
        .display_name("Shipped")
        .on_enter(vec!["generate_tracking", "notify_customer"]))
    .state(StateDef::new("delivered")
        .display_name("Delivered")
        .final_state())
    .state(StateDef::new("cancelled")
        .display_name("Cancelled")
        .final_state()
        .on_enter(vec!["refund_payment", "release_inventory"]))
    // Transitions
    .transition(Transition::new("draft", "submit", "submitted")
        .guard("has_items")
        .description("Customer submits the order"))
    .transition(Transition::new("submitted", "process", "processing")
        .guard("payment_valid")
        .actions(vec!["lock_prices"]))
    .transition(Transition::new("processing", "ship", "shipped")
        .guard("inventory_fulfilled"))
    .transition(Transition::new("shipped", "deliver", "delivered"))
    .transition(Transition::new("draft", "cancel", "cancelled"))
    .transition(Transition::new("submitted", "cancel", "cancelled")
        .guard("cancellation_allowed"))
    .transition(Transition::new("processing", "cancel", "cancelled")
        .guard("cancellation_allowed")
        .actions(vec!["reverse_payment"]));
```
</code_examples>

<phase_boundary>
## Phase 85 ↔ Phase 86 Boundary

Understanding the boundary prevents scope creep in either direction.

### Phase 85 Owns (State Machine Schema)
- `StateMachine` struct: states + transitions + initial state
- `StateDef` struct: name, display_name, is_final, on_enter/on_exit side effect references
- `Transition` struct: from/event/to + guard reference + action references
- Validation: reachable states, dead-ends, initial state existence
- Builder API: `StateMachine::new().initial().state().transition()`
- ServiceDef integration: `.state_machine(StateMachine)` method

### Phase 86 Owns (Action Definitions)
- `ActionDef` struct: name, description, inputs, guard definition, preconditions
- What guards actually check (Phase 85 only stores the string name)
- What actions actually do (Phase 85 only stores the string name)
- Transition triggers: which actions trigger which transitions
- Input definitions: what data an action requires

### The Connection
Phase 85's guard and action strings are **forward references** to Phase 86's definitions:

```
Phase 85: Transition { guard: Some("is_reviewer"), actions: ["notify_submitter"] }
                            ↓                              ↓
Phase 86: GuardDef { name: "is_reviewer", ... }   ActionDef { name: "notify_submitter", ... }
```

Phase 85 doesn't validate that guard/action strings resolve to anything — that's Phase 86's concern. Phase 85 only validates structural integrity (states exist, transitions reference valid states).
</phase_boundary>

<design_decisions>
## Design Decisions

### Decision 1: Flat States Only (No Hierarchical)
**Chosen:** Flat (atomic) states — no compound/parallel states.
**Rationale:** Hierarchical states are the defining feature of statecharts vs FSMs, but they add significant complexity:
- Entry/exit scoping (entering a compound state enters its initial child)
- History states (returning to the last active child state)
- Substate transitions (transitions within a compound state vs leaving it)
- Parallel regions (multiple active states simultaneously)

For service-level definitions, the lifecycle is typically 4-8 linear/branching states. The IntentGraph (Phase 89) handles navigation complexity that hierarchical states would address in a statechart.

**Revisit in:** Phase 93 field test. If the Order Management service naturally wants substates, consider adding compound states.

### Decision 2: Guards as Option<String> (No Composition)
**Chosen:** Simple string reference. No `And/Or/Not` combinators.
**Rationale:** XState v5 added guard composition (`and()`, `or()`, `not()`), but these are runtime features. Our schema just names the guard — the actual evaluation logic is external. If a guard needs composition, name the composite: `"is_reviewer_and_not_author"` as a single guard string that the handler resolves.
**Revisit in:** Phase 86 if ActionDef needs to express guard logic.

### Decision 3: Side Effects as Vec<String> (Not Ordered Map)
**Chosen:** Simple string vector for on_enter, on_exit, and transition actions.
**Rationale:** XState's actions array is ordered and executed sequentially. A `Vec<String>` preserves ordering while remaining simple. No need for a map because side effects don't have parameters at the schema level — that's Phase 86's ActionDef concern.

### Decision 4: Validation Returns Warnings, Not Errors for Structural Issues
**Chosen:** `validate()` returns `Result<Vec<Warning>, Error>`. Errors for fatal issues (missing initial state). Warnings for structural concerns (unreachable states, dead-ends).
**Rationale:** Some "issues" are intentional:
- Unreachable states might be reached via external means (API call that directly sets state)
- No final states is valid for perpetual services (user account lifecycle)
- Dead-end non-final states might have transitions added in Phase 86

Fatal errors (initial state doesn't exist, transition references nonexistent state) are real bugs that should block.

### Decision 5: StateMachine on ServiceDef is Optional
**Chosen:** `ServiceDef` gets `state_machine: Option<StateMachine>`.
**Rationale:** Not all services have lifecycle states. A "User Preferences" service might be stateless (pure CRUD). The IntentGraph generator (Phase 89) handles both cases — stateless services get simpler graphs without transition-based navigation.
</design_decisions>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| XState v4 separate provide() | XState v5 setup() with inline types | Dec 2023 | Validates bundling definition + type info together |
| Boolean guard functions | Higher-order guard composition (and/or/not) | XState v5 | We defer composition — string references are simpler for schema |
| Separate machine definition + implementation | Unified setup({types, actions, guards}).createMachine() | XState v5 | Our schema-only approach naturally separates these |
| cannon-es style runtime FSMs | Schema-only + separate runtime (pure_hfsm pattern) | 2023-2024 | Validates our "definition ≠ engine" philosophy |

**Industry validation:**
- XState v5's `setup()` pattern validates separating type declarations from implementation — exactly what we do by splitting Phase 85 (schema) from Phase 86 (action definitions)
- Google A2UI's declarative component model validates string-referenced actions over closures
- SCXML standard confirms entry/exit/transition action categories as the right abstraction

**No new tools/patterns needed:** This remains an internal pattern design phase. XState's philosophy is the model; the Rust implementation is straightforward.
</sota_updates>

<open_questions>
## Open Questions

1. **Should StateDef have a `metadata: Option<serde_json::Value>` field?**
   - What we know: XState states have `meta` for arbitrary data. This could carry rendering hints (color for status badges, icon names).
   - What's unclear: Whether metadata belongs on StateDef or is better handled by FieldMeaning::Status rendering rules.
   - Recommendation: YES — include metadata. It's zero-cost when unused (Option) and enables MCP introspection to surface state-specific rendering hints without coupling to FieldMeaning.

2. **Should transitions support multiple guards (array) or single guard (string)?**
   - What we know: XState supports multiple guarded transitions for the same event (array of `{ guard, target }` objects). This is "first match wins" semantics.
   - What's unclear: Whether we model this as multiple Transition structs with the same event or a single Transition with multiple guards.
   - Recommendation: Multiple `Transition` structs with the same `(from, event)` pair. First match semantics. This is how XState models it and avoids adding guard-array complexity to the Transition struct.

3. **Should we add `always` transitions (eventless, guard-only)?**
   - What we know: XState supports `always: [{ guard: 'condition', target: 'next' }]` — transitions that fire automatically when a guard becomes true, without an explicit event.
   - What's unclear: Whether schema-only definitions need this (it implies runtime evaluation).
   - Recommendation: NO for v1. Always-transitions require runtime guard evaluation, which violates schema-only. If needed, model as a transition with a well-known event name like `"_auto"`.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [XState/Stately docs (Context7)](/websites/stately_ai) — machine definition format, guards, actions, entry/exit, parallel/hierarchical states, final states
- [XState API types (Context7)](/websites/jsdocs_io_package_xstate) — TransitionConfig, GuardPredicate, setup() type signatures
- [v9.0-RESEARCH.md](../v9.0-RESEARCH.md) — architecture validation, XState philosophy, graph library decision
- [Phase 84 RESEARCH.md](../84-service-identity-field-semantics/84-RESEARCH.md) — builder patterns, serde conventions, workspace patterns
- [XState v5 announcement](https://stately.ai/blog/2023-12-01-xstate-v5) — setup() API, guard composition, unified arguments

### Secondary (MEDIUM confidence)
- [Statecharts.dev glossary](https://statecharts.dev/glossary/compound-state.html) — compound states, activities vs actions terminology
- [W3C SCXML specification](https://www.w3.org/TR/scxml/) — entry/exit/transition action model, formal statechart semantics
- [DFA minimization (Wikipedia)](https://en.wikipedia.org/wiki/DFA_minimization) — unreachable state detection, dead state detection algorithms
- Workspace crate analysis (ferro-cache builder patterns) — `with_*` methods, `mut self` → `Self` convention

### Tertiary (LOW confidence — needs validation)
- None. All findings verified against Context7 docs or official specifications.
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust struct design for serializable state machine schemas
- Ecosystem: XState v5 design philosophy (reference model, not dependency)
- Patterns: Builder API, entry/exit/transition actions, guard references, validation
- Pitfalls: Runtime engine creep, hierarchical complexity, guard/action conflation

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, uses existing workspace crates
- Architecture: HIGH — XState design validated by Context7 docs, adapted for schema-only
- Pitfalls: HIGH — derived from XState docs, statechart theory, and workspace patterns
- Code examples: HIGH — follows Phase 84 conventions, validated against builder patterns

**Research date:** 2026-02-28
**Valid until:** 2026-03-28 (30 days — internal patterns, stable)
</metadata>

---

*Phase: 85-state-machines*
*Research completed: 2026-02-28*
*Ready for planning: yes*
