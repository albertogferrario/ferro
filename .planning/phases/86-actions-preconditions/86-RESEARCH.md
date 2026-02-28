# Phase 86: Actions & Preconditions — Research

**Researched:** 2026-02-28
**Domain:** Schema-only action/guard/precondition definitions for ferro-projections
**Confidence:** HIGH

<research_summary>
## Summary

Phase 86 gives meaning to the string references stored by Phase 85. Where Phase 85's `Transition { guard: Some("is_reviewer") }` stores a name, Phase 86 defines `GuardDef { name: "is_reviewer", description: "User has reviewer role" }`. Similarly, Phase 85's action strings become full `ActionDef` structs with inputs, preconditions, effects, and transition triggers.

Three research threads inform the design: (1) XState v5's `setup()` pattern separates action/guard definitions from implementations — actions are `{ type, params }` objects, guards are `{ type, params }` boolean checks, both registered by name. (2) Google A2UI models server actions as `{ event: { name, context } }` with component-level `checks` for preconditions. (3) CQRS command patterns define actions as data structures with identity (name), contract (input schema, preconditions, postconditions), and semantics (description, metadata).

**Primary recommendation:** Three new types — `ActionDef` (business operation schema), `InputDef` (action parameter), `GuardDef` (named boolean condition). Actions reference guards via `preconditions: Vec<String>`. Actions link to state machines via `transition_trigger: Option<String>`. Add `readable: bool` and `writable: bool` to `FieldDef` (Hydra SupportedProperty pattern) for intent derivation. No new dependencies.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1 | Serialization | ActionDef/GuardDef must be serializable/introspectable |
| serde_json | 1 | JSON format | MCP introspection, IntentGraph generation consume JSON |
| thiserror | 1.0 | Error types | Workspace error pattern |

### New Dependencies: None

Action and guard definitions are pure data types — ~200 lines of struct definitions + ~100 lines of validation. Same philosophy as Phase 85: schema-only, no runtime engine.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom InputDef | Reuse FieldDef from Phase 84 | FieldDef lacks `description` for form labels. InputDef is similar but purpose-built for action parameters. |
| Simple GuardDef | GuardDef with GuardType enum | GuardType (Permission, FieldCheck, etc.) adds categorization but couples schema to evaluation strategy. Keep simple — description conveys intent. |
| String preconditions | Structured precondition objects | XState uses simple string refs for guards. Structured objects add complexity without clear benefit at schema level. |
| No ActionKind enum | ActionKind (Create/Update/Delete/Transition/Custom) | IntentGraph can infer action kind from context (has inputs? triggers transition?). Explicit enum is premature categorization. |

### Installation
No new dependencies. Uses existing workspace crates only.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Module Structure (within ferro-projections)
```
ferro-projections/src/
├── lib.rs          # Re-exports (add action module)
├── service.rs      # ServiceDef (Phase 84, extended with actions/guards)
├── field.rs        # FieldDef, FieldMeaning (Phase 84)
├── state.rs        # StateMachine, StateDef, Transition (Phase 85)
├── action.rs       # ActionDef, InputDef, GuardDef ← Phase 86
└── error.rs        # Error enum (add validation variants)
```

### Pattern 1: ActionDef as Business Operation Schema
**What:** An action describes a user-facing operation — what it needs (inputs), when it's available (preconditions), what it does (effects), and whether it changes state (transition_trigger).
**When to use:** Any operation a user/agent can perform on a service entity.
**Why:** XState v5's `{ type, params }` pattern validates named operations with parameters. A2UI's `{ event: { name, context } }` validates actions carrying input data. CQRS validates separating command definition from execution.
**Example:**
```rust
// "Submit Order" action:
// - Requires items to exist (precondition)
// - Takes a "notes" input (optional free text)
// - Triggers "submit" event on the state machine
// - Fires "validate_inventory" side effect
let submit = ActionDef::new("submit")
    .display_name("Submit Order")
    .description("Submit the order for processing")
    .input(InputDef::new("notes", DataType::String, FieldMeaning::FreeText)
        .required(false)
        .description("Optional notes for the reviewer"))
    .precondition("has_items")
    .effect("validate_inventory")
    .transition_trigger("submit");
```

### Pattern 2: GuardDef as Named Boolean Condition
**What:** A guard describes a condition that gates transitions or actions. It's a named reference — the actual evaluation logic is external.
**When to use:** Anywhere Phase 85 stores a guard string or Phase 86 stores a precondition string.
**Why:** XState registers guards by name in `setup({ guards: { name: impl } })`. Our GuardDef is the schema counterpart — it declares the guard exists and what it checks, but not how.
**Example:**
```rust
// Guards used across the order service:
let is_reviewer = GuardDef::new("is_reviewer")
    .display_name("Is Reviewer")
    .description("User has the reviewer role");

let has_items = GuardDef::new("has_items")
    .display_name("Has Items")
    .description("Order contains at least one line item");

let cancellation_allowed = GuardDef::new("cancellation_allowed")
    .display_name("Cancellation Allowed")
    .description("Order has not yet been shipped");
```

### Pattern 3: InputDef for Action Parameters
**What:** Describes a single input parameter for an action, using the same DataType/FieldMeaning vocabulary as FieldDef.
**When to use:** Any action that requires user-provided data.
**Why:** Reusing DataType and FieldMeaning from Phase 84 gives the IntentGraph (Phase 89) and Renderer (Phase 90) the semantic information they need to generate appropriate form controls. A money input renders as a currency field; an email input renders with email validation.
**Example:**
```rust
// Inputs for a "reject" action:
InputDef::new("reason", DataType::String, FieldMeaning::FreeText)
    .required(true)
    .description("Reason for rejection")

InputDef::new("notify_customer", DataType::Boolean, FieldMeaning::Boolean)
    .required(false)
    .description("Send rejection notice to customer")
```

### Pattern 4: Guard References Across Phases
**What:** Guards are defined once in `ServiceDef.guards` and referenced by name from two places: `Transition.guard` (Phase 85) and `ActionDef.preconditions` (Phase 86).
**When to use:** Always — this is the core linking mechanism.
**Example:**
```
ServiceDef.guards: [
    GuardDef { name: "is_reviewer", ... },
    GuardDef { name: "has_items", ... },
]

Phase 85 refs:  Transition { guard: Some("is_reviewer"), ... }
Phase 86 refs:  ActionDef { preconditions: ["has_items"], ... }
                            ↓                    ↓
Both resolve to: ServiceDef.guards entries by name
```

### Pattern 5: ServiceDef Integration
**What:** ServiceDef gets `actions: Vec<ActionDef>` and `guards: Vec<GuardDef>` fields with builder methods.
**When to use:** All services. Actions are the "what can you do" layer. Guards are shared conditions.
**Example:**
```rust
let order_service = ServiceDef::new("order")
    .display_name("Order")
    .field("status", DataType::String, FieldMeaning::Status)
    // Guards (shared conditions)
    .guard(GuardDef::new("has_items")
        .description("Order has at least one line item"))
    .guard(GuardDef::new("is_reviewer")
        .description("User has reviewer role"))
    // Actions (business operations)
    .action(ActionDef::new("submit")
        .display_name("Submit Order")
        .precondition("has_items")
        .transition_trigger("submit"))
    .action(ActionDef::new("approve")
        .display_name("Approve")
        .precondition("is_reviewer")
        .transition_trigger("approve"));
```

### Pattern 6: Siren Action Format as Design Reference

**Source:** [Siren Hypermedia Specification](https://github.com/kevinswiber/siren)

Siren defines actions as `{ name, title, method, href, type, fields }`. Ferro adapts this pattern but drops HTTP-specific concerns:

| Siren Property | Ferro Equivalent | Rationale |
|---------------|-----------------|-----------|
| `name` (required) | `ActionDef.name` | Keep — action identifier |
| `title` (optional) | `ActionDef.display_name` | Keep — matches existing convention |
| `method` (HTTP verb) | **Not adopted** | ActionDef is protocol-agnostic; IntentGraph infers semantics from inputs + transition_trigger |
| `href` (required) | **Dropped** | ServiceDef is schema-only, not protocol-bound |
| `type` (encoding) | **Dropped** | Protocol concern |
| `fields` (array) | `ActionDef.inputs: Vec<InputDef>` | Keep — reuses DataType/FieldMeaning |

Siren's field objects have `{ name, type, value, title, class }` where `type` maps to HTML5 input types (text, number, email, etc.). Ferro replaces this with DataType + FieldMeaning which provides richer semantic information for intent-driven rendering.

### Pattern 7: Readable/Writable on FieldDef (Hydra SupportedProperty)

**Source:** [Hydra Core Vocabulary](https://www.hydra-cg.com/spec/latest/core/) — SupportedProperty

Hydra defines two boolean properties on SupportedProperty:
- `readable: xsd:boolean` — "True if the client can retrieve the property's value, false otherwise."
- `writable: xsd:boolean` — "True if the client can change the property's value, false otherwise."

OpenAPI 3.0 has similar `readOnly`/`writeOnly` but as **mutually exclusive** — a property cannot be both. Hydra's model is more flexible: both booleans are independent, yielding 4 access modes.

**Ferro adaptation — add to FieldDef:**

```rust
pub struct FieldDef {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    pub required: bool,
    pub is_list: bool,
    pub readable: bool,   // NEW: default true
    pub writable: bool,   // NEW: default true
}
```

| readable | writable | Meaning | Example Fields |
|----------|----------|---------|---------------|
| true | true | Read-write (default) | name, email, notes |
| true | false | Read-only (computed/system) | id, created_at, total |
| false | true | Write-only (sensitive input) | password, api_key |
| false | false | Internal (hidden from UI) | hashed_password, internal_flags |

**Intent derivation signal (Phase 89):** The count of writable fields directly drives intent selection:
- `writable_count > 3` → Collect intent (form/wizard)
- All fields readable, few writable → Focus intent (detail view)
- Sensitive fields write-only → security-aware rendering

**Defaults:** Both default to `true`. This matches the common case where fields are read-write. Developers opt specific fields into restricted modes (id → read-only, password → write-only).

**Serde compatibility:** Both fields MUST use `#[serde(default = "default_true")]` so existing JSON without these fields deserializes correctly, maintaining backward compatibility with Phase 84/85 ServiceDef JSON.

### Anti-Patterns to Avoid
- **No closures in guards/actions:** Guards check conditions but the check logic is external. `GuardDef` describes what, not how.
- **No ActionKind enum:** Don't categorize actions as Create/Update/Delete/Transition. The IntentGraph can infer this from inputs + transition_trigger. Explicit categorization is premature and rigid.
- **No guard composition in schema:** If you need "is_reviewer AND not_author", define a single guard "is_reviewer_not_author". Composition is a runtime concern, not schema.
- **No input validation rules in InputDef:** InputDef describes what data is needed (type + meaning), not validation constraints. Validation rules live in the handler, not the projection.
- **Don't conflate effects with transition triggers:** Effects are side effects (fire-and-forget). Transition triggers fire state machine events. An action can have effects without changing state, or change state without side effects.
- **Don't use OpenAPI's mutually exclusive readOnly/writeOnly:** Hydra's independent booleans are more expressive. A field that is neither readable nor writable (internal) is a valid and useful combination that OpenAPI cannot express.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Input type system | Custom type enum for inputs | Reuse `DataType` + `FieldMeaning` from Phase 84 | Already handles 10 data types + 18 semantic meanings. Creating a parallel type system fragments the vocabulary. |
| Guard evaluation engine | Runtime guard checking | Schema-only `GuardDef` | This phase defines schemas, not engines. Guard evaluation is the handler's job. |
| Form schema language | JSON Schema subset for inputs | `Vec<InputDef>` with DataType/FieldMeaning | JSON Schema is powerful but overkill for action inputs. Our existing type vocabulary provides the same semantic information in a simpler structure. |
| Action workflow engine | Multi-step action sequences | Single `ActionDef` per operation | Multi-step workflows are Phase 89 (IntentGraph) territory. Phase 86 defines atomic operations. |
| Precondition logic | Boolean expression trees for guards | Simple `Vec<String>` referencing `GuardDef` names | XState uses the same approach — guards are named references, composition happens at runtime. |

**Key insight:** Phase 86 defines the vocabulary for "what can you do with this service" — not the execution engine. ActionDef is to an HTTP handler what a database migration is to a query: it describes structure, not behavior. The IntentGraph (Phase 89) reads these definitions to build navigation. The Renderer (Phase 90) reads inputs to generate forms.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Building Runtime Logic Into ActionDef
**What goes wrong:** Adding methods like `.execute()`, `.can_perform()`, or `.validate_inputs()` to ActionDef.
**Why it happens:** Natural instinct — an "action" should do something.
**How to avoid:** ActionDef has no `&mut self` methods. It's constructed once (builder) and read (getters + serialization). The handler resolves guard names and executes logic. ActionDef just describes what's possible.
**Warning signs:** Methods that take `&Context` or return `Result<(), ActionError>`.

### Pitfall 2: Duplicating FieldDef in InputDef
**What goes wrong:** Creating a completely separate type system for action inputs when FieldDef's vocabulary already exists.
**Why it happens:** Action inputs "feel different" from entity fields, so the instinct is to create new types.
**How to avoid:** InputDef reuses `DataType` and `FieldMeaning` from Phase 84. This ensures the IntentGraph and Renderer can apply the same semantic rendering (money = currency format, email = email validation) to both entity fields and action inputs.
**Warning signs:** A new `InputType` enum that mirrors `DataType` with slightly different names.

### Pitfall 3: Conflating Guards with Preconditions
**What goes wrong:** Treating transition guards and action preconditions as the same concept or storing them in different incompatible formats.
**Why it happens:** Both gate availability of something. The difference is scope.
**How to avoid:** Both reference `GuardDef` by name string. The difference is WHERE they're attached: `Transition.guard` gates a specific transition; `ActionDef.preconditions` gates the entire action. Same pool of `GuardDef` names, different attachment points.
**Warning signs:** Two parallel guard definition systems, or preconditions that duplicate transition guards.

### Pitfall 4: Over-Specifying Guard Semantics
**What goes wrong:** Adding `GuardType::Permission`, `GuardType::FieldCheck`, `GuardType::StateCheck` etc. to categorize guards.
**Why it happens:** Desire to help the IntentGraph "understand" guards programmatically.
**How to avoid:** GuardDef is intentionally minimal: name + description. The IntentGraph doesn't evaluate guards — it presents them. The handler evaluates. If semantic categorization is needed, add it in Phase 89 as IntentGraph metadata, not in the guard definition itself.
**Warning signs:** A `GuardType` enum with more than 3 variants that maps to evaluation strategies.

### Pitfall 5: Actions Without Clear Boundaries
**What goes wrong:** An ActionDef that "does everything" — submits the order, sends email, charges payment, updates inventory.
**Why it happens:** Modeling a user flow as a single action instead of a state machine transition + side effects.
**How to avoid:** Actions should be atomic operations. Complex flows are sequences of state transitions (Phase 85) with side effects. The action "submit" triggers the "submit" event; the state machine's on_enter effects handle downstream logic.
**Warning signs:** ActionDef with >5 effects, or effects that are sequential dependencies (must run in order).

### Pitfall 6: Breaking FieldDef Serde Compatibility with Readable/Writable
**What goes wrong:** Adding `readable`/`writable` fields to FieldDef breaks deserialization of existing JSON.
**Why it happens:** New fields without serde defaults cause missing-field errors on old JSON.
**How to avoid:** Both fields MUST have `#[serde(default = "default_true")]` so existing JSON without these fields deserializes to read-write (the common case). The `default_true` helper already exists in field.rs.
**Warning signs:** Existing Phase 84/85 tests fail after adding the fields.

### Pitfall 7: Missing Connection Between Actions and Transitions
**What goes wrong:** Actions exist as standalone operations disconnected from the state machine, making the IntentGraph unable to determine which actions are available in which states.
**Why it happens:** ActionDef and StateMachine are built independently without linking.
**How to avoid:** `transition_trigger: Option<String>` on ActionDef explicitly connects to Transition.event. Validation checks that transition_trigger values match existing transition event names. Actions without transition_trigger are state-independent (available in any state).
**Warning signs:** Actions that "should" only be available in certain states but have no transition_trigger, leading to UI showing unavailable operations.
</common_pitfalls>

<code_examples>
## Code Examples

### ActionDef Type Definition
```rust
// Source: XState { type, params } pattern + CQRS command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    /// Action identifier (e.g., "submit", "approve", "add_note")
    pub name: String,
    /// Human-readable name (e.g., "Submit Order")
    pub display_name: Option<String>,
    /// Description of what this action does
    pub description: Option<String>,
    /// Input parameters required by this action
    pub inputs: Vec<InputDef>,
    /// Guard names that must all pass for this action to be available
    pub preconditions: Vec<String>,
    /// Side effect names fired when this action executes
    pub effects: Vec<String>,
    /// State machine event this action triggers (connects to Transition.event)
    pub transition_trigger: Option<String>,
}
```

### InputDef Type Definition
```rust
// Source: Reuses Phase 84 DataType + FieldMeaning vocabulary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDef {
    /// Parameter name (e.g., "reason", "amount")
    pub name: String,
    /// Data type for this input
    pub data_type: DataType,
    /// Semantic meaning (drives UI rendering: Money → currency field, Email → email input)
    pub meaning: FieldMeaning,
    /// Whether this input is required
    pub required: bool,
    /// Human-readable description (form label / help text)
    pub description: Option<String>,
}
```

### GuardDef Type Definition
```rust
// Source: XState guard registration pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardDef {
    /// Guard identifier (e.g., "is_reviewer", "has_items")
    pub name: String,
    /// Human-readable name (e.g., "Is Reviewer")
    pub display_name: Option<String>,
    /// Description of what this guard checks
    pub description: Option<String>,
}
```

### Builder APIs
```rust
impl ActionDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            inputs: Vec::new(),
            preconditions: Vec::new(),
            effects: Vec::new(),
            transition_trigger: None,
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

    pub fn input(mut self, input: InputDef) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn precondition(mut self, guard_name: impl Into<String>) -> Self {
        self.preconditions.push(guard_name.into());
        self
    }

    pub fn effect(mut self, effect_name: impl Into<String>) -> Self {
        self.effects.push(effect_name.into());
        self
    }

    pub fn transition_trigger(mut self, event: impl Into<String>) -> Self {
        self.transition_trigger = Some(event.into());
        self
    }
}

impl InputDef {
    pub fn new(
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        Self {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            description: None,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

impl GuardDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
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
}
```

### ServiceDef Integration
```rust
// ServiceDef extensions (added to existing service.rs)
impl ServiceDef {
    pub fn action(mut self, action: ActionDef) -> Self {
        self.actions.push(action);
        self
    }

    pub fn guard(mut self, guard: GuardDef) -> Self {
        self.guards.push(guard);
        self
    }
}

// ServiceDef struct gets new fields:
pub struct ServiceDef {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub fields: Vec<FieldDef>,
    // Phase 85:
    pub state_machine: Option<StateMachine>,
    // Phase 86:
    pub actions: Vec<ActionDef>,
    pub guards: Vec<GuardDef>,
}
```

### Cross-Phase Validation
```rust
impl ServiceDef {
    /// Validates that all guard/action references resolve to defined names.
    /// Call after building the complete ServiceDef with state machine + actions + guards.
    pub fn validate(&self) -> Result<Vec<Warning>, Error> {
        let mut warnings = Vec::new();
        let guard_names: HashSet<&str> = self.guards.iter().map(|g| g.name.as_str()).collect();

        // 1. Action preconditions must reference declared guards
        for action in &self.actions {
            for precondition in &action.preconditions {
                if !guard_names.contains(precondition.as_str()) {
                    return Err(Error::Validation(format!(
                        "action '{}' precondition '{}' not found in guards",
                        action.name, precondition
                    )));
                }
            }
        }

        // 2. Transition guards must reference declared guards (if state machine exists)
        if let Some(ref machine) = self.state_machine {
            for transition in &machine.transitions {
                if let Some(ref guard) = transition.guard {
                    if !guard_names.contains(guard.as_str()) {
                        return Err(Error::Validation(format!(
                            "transition '{} -{}-> {}' guard '{}' not found in guards",
                            transition.from, transition.event, transition.to, guard
                        )));
                    }
                }
            }
        }

        // 3. Action transition_triggers must match state machine event names
        if let Some(ref machine) = self.state_machine {
            let event_names: HashSet<&str> =
                machine.transitions.iter().map(|t| t.event.as_str()).collect();
            for action in &self.actions {
                if let Some(ref trigger) = action.transition_trigger {
                    if !event_names.contains(trigger.as_str()) {
                        return Err(Error::Validation(format!(
                            "action '{}' transition_trigger '{}' not found in state machine events",
                            action.name, trigger
                        )));
                    }
                }
            }
        }

        // 4. Warn about declared guards that are never referenced
        let mut referenced_guards: HashSet<&str> = HashSet::new();
        for action in &self.actions {
            for p in &action.preconditions {
                referenced_guards.insert(p.as_str());
            }
        }
        if let Some(ref machine) = self.state_machine {
            for t in &machine.transitions {
                if let Some(ref g) = t.guard {
                    referenced_guards.insert(g.as_str());
                }
            }
        }
        for guard in &self.guards {
            if !referenced_guards.contains(guard.name.as_str()) {
                warnings.push(Warning::UnusedGuard(guard.name.clone()));
            }
        }

        // 5. Warn about actions with transition_trigger but no matching state machine
        if self.state_machine.is_none() {
            for action in &self.actions {
                if action.transition_trigger.is_some() {
                    warnings.push(Warning::TransitionTriggerWithoutStateMachine(
                        action.name.clone(),
                    ));
                }
            }
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    // Phase 85 warnings...
    UnreachableState(String),
    DeadEndState(String),
    NoFinalStates,
    // Phase 86 warnings:
    UnusedGuard(String),
    TransitionTriggerWithoutStateMachine(String),
}
```

### FieldDef with Readable/Writable
```rust
// Source: Hydra SupportedProperty semantics
// New builder convenience methods on ServiceDef:

impl ServiceDef {
    /// Adds a read-only field (readable=true, writable=false).
    /// For system-assigned or computed fields: id, created_at, total.
    pub fn read_only_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: false,
            readable: true,
            writable: false,
        });
        self
    }

    /// Adds a write-only field (readable=false, writable=true).
    /// For sensitive inputs: password, api_key.
    pub fn write_only_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: false,
            readable: false,
            writable: true,
        });
        self
    }
}

// Usage:
let user = ServiceDef::new("user")
    .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("name", DataType::String, FieldMeaning::EntityName)          // default: read-write
    .field("email", DataType::String, FieldMeaning::Email)              // default: read-write
    .write_only_field("password", DataType::String, FieldMeaning::Sensitive)
    .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);
// writable_count = 2 (name, email) → NOT enough for Collect intent
// readable_count = 4 (id, name, email, created_at) → Focus intent likely
```

### Full Order Service Example
```rust
let order_service = ServiceDef::new("order")
    .display_name("Order")
    .description("Manages customer orders and fulfillment")
    // Fields (Phase 84)
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
    .field("total", DataType::Float, FieldMeaning::Money)
    .field("status", DataType::String, FieldMeaning::Status)
    .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
    // Guards (Phase 86)
    .guard(GuardDef::new("has_items")
        .description("Order contains at least one line item"))
    .guard(GuardDef::new("is_reviewer")
        .description("User has the reviewer role"))
    .guard(GuardDef::new("cancellation_allowed")
        .description("Order has not yet been shipped"))
    .guard(GuardDef::new("payment_valid")
        .description("Payment method is verified"))
    // State machine (Phase 85)
    .state_machine(
        StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("submitted").on_enter(vec!["validate_inventory"]))
            .state(StateDef::new("approved"))
            .state(StateDef::new("cancelled").final_state())
            .state(StateDef::new("completed").final_state())
            .transition(Transition::new("draft", "submit", "submitted")
                .guard("has_items"))
            .transition(Transition::new("submitted", "approve", "approved")
                .guard("is_reviewer"))
            .transition(Transition::new("submitted", "reject", "cancelled")
                .guard("is_reviewer"))
            .transition(Transition::new("approved", "complete", "completed"))
            .transition(Transition::new("draft", "cancel", "cancelled"))
            .transition(Transition::new("submitted", "cancel", "cancelled")
                .guard("cancellation_allowed"))
    )
    // Actions (Phase 86)
    .action(ActionDef::new("submit")
        .display_name("Submit Order")
        .description("Submit the order for review")
        .precondition("has_items")
        .transition_trigger("submit"))
    .action(ActionDef::new("approve")
        .display_name("Approve")
        .description("Approve the submitted order")
        .precondition("is_reviewer")
        .effect("send_approval_notification")
        .transition_trigger("approve"))
    .action(ActionDef::new("reject")
        .display_name("Reject")
        .description("Reject the submitted order")
        .precondition("is_reviewer")
        .input(InputDef::new("reason", DataType::String, FieldMeaning::FreeText)
            .required(true)
            .description("Reason for rejection"))
        .effect("send_rejection_notification")
        .transition_trigger("reject"))
    .action(ActionDef::new("cancel")
        .display_name("Cancel Order")
        .precondition("cancellation_allowed")
        .effect("refund_payment")
        .transition_trigger("cancel"))
    .action(ActionDef::new("add_note")
        .display_name("Add Note")
        .description("Add a note to the order (no state change)")
        .input(InputDef::new("note", DataType::String, FieldMeaning::FreeText)
            .required(true)
            .description("Note content")));
```

### Serde Round-Trip Test
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_action() -> ActionDef {
        ActionDef::new("submit")
            .display_name("Submit Order")
            .description("Submit for review")
            .input(InputDef::new("notes", DataType::String, FieldMeaning::FreeText)
                .required(false)
                .description("Optional notes"))
            .precondition("has_items")
            .effect("validate_inventory")
            .transition_trigger("submit")
    }

    #[test]
    fn action_def_serde_round_trip() {
        let action = sample_action();
        let json = serde_json::to_string_pretty(&action).unwrap();
        let parsed: ActionDef = serde_json::from_str(&json).unwrap();

        assert_eq!(action.name, parsed.name);
        assert_eq!(action.inputs.len(), parsed.inputs.len());
        assert_eq!(action.preconditions, parsed.preconditions);
        assert_eq!(action.transition_trigger, parsed.transition_trigger);
    }

    #[test]
    fn guard_def_serde_round_trip() {
        let guard = GuardDef::new("is_reviewer")
            .display_name("Is Reviewer")
            .description("User has reviewer role");

        let json = serde_json::to_string(&guard).unwrap();
        let parsed: GuardDef = serde_json::from_str(&json).unwrap();
        assert_eq!(guard.name, parsed.name);
    }

    #[test]
    fn action_without_transition() {
        let action = ActionDef::new("add_note")
            .display_name("Add Note")
            .input(InputDef::new("note", DataType::String, FieldMeaning::FreeText));

        assert!(action.transition_trigger.is_none());
        assert!(action.preconditions.is_empty());
        assert_eq!(action.inputs.len(), 1);
    }
}
```
</code_examples>

<design_decisions>
## Design Decisions

### Decision 1: Separate InputDef (Not Reuse FieldDef)
**Chosen:** New `InputDef` struct similar to FieldDef but with `description` and without `is_list`.
**Rationale:** Action inputs need form labels/help text (`description`) which entity fields don't carry. Entity fields carry `is_list` which action inputs don't need. While structurally similar, their purposes differ enough to warrant separate types. Both share `DataType` and `FieldMeaning` for consistent rendering semantics.
**Revisit in:** Phase 93 field test. If actions need list inputs (e.g., "select items to remove"), add `is_list` to InputDef.

### Decision 2: Guards as Minimal Schema (Name + Description Only)
**Chosen:** `GuardDef { name, display_name, description }` — no categorization, no condition type.
**Rationale:** XState's guards are named references resolved at runtime. Adding `GuardType::Permission`, `GuardType::FieldCheck`, etc. couples the schema to evaluation strategy. The IntentGraph (Phase 89) doesn't evaluate guards — it presents available transitions/actions. If semantic categorization helps Phase 89, add it there as IntentGraph metadata.
**Revisit in:** Phase 89 if the IntentGraph needs guard semantics to generate correct navigation.

### Decision 3: Preconditions Are Guard References, Not Separate Types
**Chosen:** `ActionDef.preconditions: Vec<String>` referencing `GuardDef` names — same pool as `Transition.guard`.
**Rationale:** Both action preconditions and transition guards answer the same question: "is this available?" The difference is attachment point (action-level vs transition-level), not type. Using the same `GuardDef` pool avoids parallel guard systems and enables cross-validation.

### Decision 4: transition_trigger Connects Actions to State Machines
**Chosen:** `ActionDef.transition_trigger: Option<String>` matching `Transition.event`.
**Rationale:** This is the bridge between "what can you do" (actions) and "what changes" (state machine). An action that triggers "submit" maps to `Transition::new("draft", "submit", "submitted")`. Actions without transition_trigger are state-independent (available regardless of current state). Validation ensures trigger values match actual transition events.

### Decision 5: No ActionKind Enum
**Chosen:** No categorization of actions as Create/Update/Delete/Transition/Custom.
**Rationale:** The IntentGraph (Phase 89) can infer action kind from context:
- Has inputs + no transition_trigger → data modification (form)
- No inputs + transition_trigger → state change (button)
- Has inputs + transition_trigger → state change with data (form + submit)
- No inputs + no transition_trigger → simple operation (button)
Explicit categorization would be premature and could conflict with these inferences.
**Revisit in:** Phase 89 if the IntentGraph needs explicit action categories.

### Decision 6: Cross-Phase Validation on ServiceDef
**Chosen:** `ServiceDef::validate()` checks all guard references, transition triggers, and reports unused guards as warnings.
**Rationale:** Guard names are forward references across Phase 85 (transitions) and Phase 86 (actions). Without centralized validation, typos in guard names silently break. Validation catches: undefined guard refs, unmatched transition triggers, and unused guard declarations.

### Decision 7: Readable/Writable as Independent Booleans (Hydra, Not OpenAPI)
**Chosen:** Two independent `bool` fields on FieldDef: `readable` (default true) and `writable` (default true).
**Rationale:** Hydra SupportedProperty uses independent booleans, yielding 4 access modes (read-write, read-only, write-only, internal). OpenAPI's readOnly/writeOnly are mutually exclusive, limiting to 3 modes. The "internal" mode (readable=false, writable=false) is useful for fields like hashed_password that exist in the schema but should never appear in UI.
**Serde:** Both fields use `#[serde(default = "default_true")]` for backward compatibility with existing Phase 84/85 JSON.
**Intent signal:** Writable field count directly feeds Phase 89 intent derivation (>3 writable → Collect intent).

### Decision 8: Actions and Guards on ServiceDef (Not Global)
**Chosen:** Actions and guards are per-service, stored on `ServiceDef`.
**Rationale:** Each service defines its own operations and conditions. An "is_reviewer" guard on the Order service is different from "is_reviewer" on the Invoice service (different reviewers). Global guards would require namespacing. Service-scoped guards are naturally namespaced.
</design_decisions>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| XState v4 inline actions | XState v5 `setup({ actions })` registration | Dec 2023 | Validates named action references over inline definitions |
| XState v4 `cond` for guards | XState v5 `guard` property with `{ type, params }` | Dec 2023 | Validates our `{ name, params-equivalent }` structure |
| A2UI v0.8 basic actions | A2UI v0.9 server actions with context binding | 2025 | Validates action + context (inputs) pattern |
| Monolithic command objects | CQRS with explicit pre/postconditions | Established | Validates separating conditions from execution |

**Industry validation:**
- XState v5's `setup()` pattern validates declaring actions/guards by name, implementing elsewhere — exactly our ActionDef/GuardDef → handler pattern
- A2UI's `{ event: { name, context } }` validates our ActionDef with inputs (context = input data)
- CQRS command pattern validates separating command definition (schema) from command handler (execution)
- Design-by-contract (preconditions/postconditions) validates our `preconditions: Vec<String>` approach

**No new tools/patterns needed:** This remains an internal pattern design phase building on Phase 84/85 vocabulary.
</sota_updates>

<open_questions>
## Open Questions

1. **Should InputDef support default values?**
   - What we know: XState actions accept `params` that can be static or dynamic. A2UI action context can include literal values. Default values help with form pre-population.
   - What's unclear: How to represent defaults in a type-agnostic way (`serde_json::Value`? String? Per-DataType enum?).
   - Recommendation: NO for v1. Defaults add serialization complexity (`serde_json::Value` for any-type defaults). If Phase 93 needs defaults, add `default: Option<serde_json::Value>` then.

2. **Should ActionDef declare postconditions (expected state after execution)?**
   - What we know: Design-by-contract includes postconditions. CQRS events capture what happened. Postconditions could help the IntentGraph predict outcomes.
   - What's unclear: Whether postconditions add value at the schema level or are better modeled as state machine transitions (which already describe the outcome).
   - Recommendation: NO for v1. The state machine's target state IS the postcondition for transition-triggering actions. Non-transition actions don't change observable state from the schema's perspective.

3. **Builder API for readable/writable on FieldDef**
   - What we know: Need to set readable/writable per field. Current builder has `.field()` and `.optional_field()`.
   - What's unclear: Whether to use modifier methods (`.field(...).read_only()`), separate builder methods (`.read_only_field(...)`), or pass as parameters.
   - Recommendation: Add `read_only_field()` and `write_only_field()` convenience methods on ServiceDef. Keep existing `field()` unchanged (defaults to read-write). This avoids changing method signatures and follows the existing `optional_field()` pattern.

4. **Should there be a way to declare CRUD actions implicitly from fields?**
   - What we know: Most services have create/read/update/delete operations derivable from their field definitions. Explicit ActionDefs for CRUD feels boilerplate-heavy.
   - What's unclear: Whether implicit CRUD generation belongs in Phase 86 (schema) or Phase 89 (IntentGraph generation).
   - Recommendation: DEFER to Phase 89. ActionDef handles explicit, domain-specific actions. The IntentGraph generator can infer standard CRUD operations from ServiceDef fields. This keeps Phase 86 focused on explicit action definitions.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- XState v5 docs (Context7) — action/guard structure, `{ type, params }` pattern, `setup()` registration, action categorization (entry/exit/transition)
- [Phase 85 RESEARCH.md](85-state-machines/85-RESEARCH.md) — phase boundary definition, string reference pattern, builder conventions
- [Phase 84 crate code](../../ferro-projections/src/) — DataType, FieldMeaning, FieldDef, ServiceDef builder pattern, serde conventions
- [v9.0-RESEARCH.md](v9.0-RESEARCH.md) — schema-only philosophy, XState design validation, A2UI action model

### Secondary (MEDIUM confidence)
- [Siren Hypermedia Specification](https://github.com/kevinswiber/siren) — Action and field format (name, method, fields, href)
- [Siren JSON Schema](https://github.com/kevinswiber/siren/blob/master/siren.schema.json) — Exact action/field property definitions with types and enums
- [Hydra Core Vocabulary](https://www.hydra-cg.com/spec/latest/core/) — SupportedProperty, readable/writable semantics
- [Hydra JSON-LD Context](https://github.com/HydraCG/Specifications/blob/master/spec/latest/core/core.jsonld) — Exact readable (`xsd:boolean`) and writable (`xsd:boolean`) definitions
- [A2UI v0.9 specification](https://a2ui.org/specification/v0.9-a2ui/) — server action structure `{ event: { name, context } }`, checks/preconditions on components
- [CQRS Pattern (Microsoft)](https://learn.microsoft.com/en-us/azure/architecture/patterns/cqrs) — command definition as data, pre/postcondition separation
- [JSON Schema for forms](https://json-schema.org/understanding-json-schema/reference/object) — input schema patterns (decided against full JSON Schema, too heavy)
- [Design by Contract (Microsoft)](https://learn.microsoft.com/en-us/dotnet/framework/debug-trace-profile/code-contracts) — precondition/postcondition/invariant formalization
- [OpenAPI 3.0.3 readOnly/writeOnly](https://spec.openapis.org/oas/v3.0.3.html) — mutually exclusive alternative to Hydra's independent booleans

### Tertiary (LOW confidence — needs validation)
- None. All findings verified against Context7 docs, official specifications, or established patterns.
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust struct design for action/guard/input schemas in ferro-projections
- Ecosystem: XState v5 (action/guard model), A2UI (server actions), CQRS (command patterns), Siren (action format), Hydra (readable/writable)
- Patterns: ActionDef builder, InputDef reusing DataType/FieldMeaning, GuardDef as named condition, cross-phase validation, FieldDef readable/writable
- Pitfalls: Runtime logic creep, type system duplication, guard/precondition conflation, over-categorization, serde backward compatibility

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, uses existing workspace crates
- Architecture: HIGH — XState/A2UI/CQRS patterns validated, builds directly on Phase 84/85 vocabulary
- Pitfalls: HIGH — derived from XState docs, CQRS patterns, and Phase 85 experience
- Code examples: HIGH — follows Phase 84/85 conventions, validated against builder patterns

**Research date:** 2026-02-28
**Valid until:** 2026-03-28 (30 days — internal patterns, stable)
</metadata>

---

*Phase: 86-actions-preconditions*
*Research completed: 2026-02-28*
*Ready for planning: yes*
