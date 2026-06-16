# Phase 231: StateMachine-Derived Executor — Research

**Researched:** 2026-06-16
**Domain:** Schema-driven write derivation in `ferro-projections` (Rust, no-runtime crate)
**Confidence:** HIGH (every claim grounded in workspace files at 0.2.65; no external library research required)

## Summary

Phase 231 eliminates the "declare twice" duplication on the projection write path. Today, an
`ActionDef` whose `transition_trigger` names a `StateMachine` transition (`app/src/projections/order.rs:38`)
still requires the app to hand-write the transition target in a SeaORM closure
(`app/src/controllers/mcp.rs:97-102`: `match action_name { "submit" => "submitted", "approve" => "approved", ... }`).
The new status `"submitted"` is **exactly** the `to` field of `Transition::new("draft", "submit", "submitted")`
(`order.rs:30`) — the same fact, declared in two places, with nothing keeping them in sync.

The binding constraint discovered in research: **`ferro-projections` is a no-runtime, schema-only crate**.
Its `Cargo.toml` (`ferro-projections/Cargo.toml:18-21`) depends only on `schemars`, `serde`, `serde_json`,
`thiserror` — **no `sea-orm`, no `tokio`, no `async`**. The crate's own `CLAUDE.md` forbids closures and
runtime logic ("No closures in definitions", "No runtime logic in ServiceDef"). Therefore the derived
"executor" **cannot be a closure or a DB-touching function inside `ferro-projections`**. It must be a
**serializable plan** (a description of state-read → guard-check → transition → persist) that the existing
consumer-side runtime (`ferro-mcp-server::dispatch_write`) interprets against a concrete SeaORM entity.

This reframes the work: `ferro-projections` gains a pure function
`derive_transition_plan(&ServiceDef, action_name) -> Result<TransitionPlan, Error>` that reads the
declared `StateMachine` + `ActionDef` and returns the (from-state-set → event → to-state, guard) facts.
The consumer's generic executor consumes the plan and does the I/O. Drift (EXEC-04) is killed by extending
the already-existing `ServiceDef::validate()` (`service.rs:367`) — registration-time, not compile-time,
because services are built with runtime builder chains (`order.rs:10`), not declared in proc-macro
attributes the compiler can inspect.

**Primary recommendation:** Add a `TransitionPlan` value type + `derive_transition_plan()` pure function to
`ferro-projections` (schema-only, serializable, zero new deps). Keep the DB-touching generic executor in the
consumer (`ferro-mcp-server` / Phase 232). Attach the override hook as a registry keyed by action name on the
consumer side. Enforce sync-by-construction by extending `ServiceDef::validate()` — the existing validation
already covers undeclared-trigger detection (`service.rs:404-418`); Phase 231 makes that check a hard boot
gate and adds the executor-derivation guarantee on top.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Transition fact derivation (event→to-state, guard) | `ferro-projections` (schema) | — | It owns `StateMachine`/`ActionDef`; derivation must read FROM them (coherence constraint, REQUIREMENTS.md:7) |
| Serializable executor plan | `ferro-projections` (schema) | — | A plan is data, not behavior — fits the no-closure rule (`ferro-projections/CLAUDE.md`) |
| State read + guard re-eval + persist (I/O) | `ferro-mcp-server` (consumer runtime) | form write path (Phase 232) | DB access needs `sea-orm`/`tokio`, which `ferro-projections` must not pull in (`Cargo.toml:18-21`) |
| Override hook registration | `ferro-mcp-server` (consumer runtime) | — | Closures live with the runtime that calls them, never in the schema crate |
| Sync-by-construction validation | `ferro-projections::validate()` | app boot gate | `validate()` already cross-checks trigger↔event (`service.rs:404-418`); extend + make it a hard gate |

## User Constraints

No `CONTEXT.md` exists for Phase 231 yet (this is a standalone research run preceding `/gsd-plan-phase 231`).
The binding constraints are taken from `REQUIREMENTS.md` and the project's `CLAUDE.md` / `feedback_*` memory:

### Locked Decisions (from REQUIREMENTS.md + CLAUDE.md)
- **Derivation lives in `ferro-projections`** — not a new crate, not a parallel control surface
  (REQUIREMENTS.md:7, :43; `feedback_no_duplicate_control_surface`).
- **Reads FROM existing `StateMachine`/`ActionDef`** — no second way to declare transitions (REQUIREMENTS.md:42).
- **Guard re-eval reuses the live `GuardEvaluatorFn` path**, never `ctx.evaluated_guards`
  (the list-time visibility cache) — `write_dispatch.rs:266-285`, REQUIREMENTS.md:16.
- **No StateMachine/guard redesign** — derivation consumes both as-is (REQUIREMENTS.md:45).
- **EXEC-05 (cross-surface wiring) is Phase 232, not 231** (REQUIREMENTS.md:57). Phase 231 stops at
  derivation + plan + override-hook surface + validation.

### Claude's Discretion
- Exact shape of `TransitionPlan` (struct fields, serde form).
- Whether the override hook is a registry on the consumer or a typed builder seam.
- Whether `derive_transition_plan` returns one plan or a set (multi-source events like `cancel` have 3 sources — `state.rs:707`).

### Deferred Ideas (OUT OF SCOPE)
- Derived executor for **non-transition** plain CRUD writes (REQUIREMENTS.md:34) — keep current path.
- Operating-AX / NL description quality (REQUIREMENTS.md:35).
- Projection `body` slot (REQUIREMENTS.md:36).
- gestiscilo / consumer migration (REQUIREMENTS.md:44; `feedback_cross_repo_phase_split`).

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EXEC-01 | Derive default executor (state read → guard re-eval → transition → persist); no `WriteDispatcher` match for common path | `derive_transition_plan()` reads `Transition.to` (`state.rs:79-90`) so the app no longer writes `match action_name => new_status` (`mcp.rs:97-102`). The consumer's generic executor consumes the plan. |
| EXEC-02 | Server-side guard re-eval at execution; reject guard-failing transition | The plan carries `transition.guard` (`state.rs:82-83`); the executor runs it through the existing live `GuardEvaluatorFn` loop (`write_dispatch.rs:276-285`) before persisting. `BaseContext.evaluated_guards` (`render/mod.rs:46`) is NOT consulted at execution (it is the list-time cache). |
| EXEC-03 | Override hook for app-specific side effects without replacing base dispatch | A consumer-side hook (closure keyed by action name) runs **after** the derived persist, inside `dispatch_write` (`write_dispatch.rs:326-350`), reusing the existing audit/idempotency envelope. Common path stays declaration-only. |
| EXEC-04 | Build/registration-time rejection of undeclared-transition reference | Extend `ServiceDef::validate()` — it already errors on `transition_trigger` with no matching event (`service.rs:404-418`). Make it a hard boot gate + add a `derive_transition_plan` round-trip check. Registration-time (not compile-time) because services are runtime builder chains (`order.rs:10`), not macro-declared. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (none new) | — | Derivation is pure Rust over existing types | `ferro-projections` must stay dependency-minimal (`Cargo.toml:18-21`); adding the executor needs zero new crates [VERIFIED: ferro-projections/Cargo.toml] |
| `serde` / `serde_json` | 1 | `TransitionPlan` derives `Serialize/Deserialize/JsonSchema` like every other projection type | All projection types are serializable (`ferro-projections/CLAUDE.md`) [VERIFIED] |
| `schemars` | 1 | `JsonSchema` derive for introspection (`ferro-mcp` exposes plans) | Every public projection type derives `JsonSchema` (`action.rs:24`, `state.rs:23`) [VERIFIED] |
| `thiserror` | 1 | New `Error` variants for derivation failures | Existing `Error` enum uses thiserror (`error.rs:1`) [VERIFIED] |

### Supporting (consumer-side, Phase 232 — listed for plan continuity)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `sea-orm` | (workspace) | Generic executor reads/persists the entity | Already a `ferro-mcp-server` dep (`write_dispatch.rs:13`) [VERIFIED] |
| `tokio` (via boxed futures) | — | `dispatch_write` is async (`write_dispatch.rs:258`) | The derived executor body is async on the consumer side [VERIFIED] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Serializable `TransitionPlan` interpreted by consumer | A `derive_default_executor() -> impl Fn(...)` closure in `ferro-projections` | **Rejected** — violates the no-closure rule (`ferro-projections/CLAUDE.md`) and would force `sea-orm`/`tokio` into the schema crate, breaking modality-agnosticism (CLAUDE.md "No runtime logic in ServiceDef"). |
| Registration-time validation (extend `validate()`) | Compile-time proc-macro / trybuild rejection | **Rejected for the trigger check** — services are built with runtime builder chains (`order.rs:10`), so the compiler can't see transition names. trybuild stays useful only if a future `#[service]` macro declares transitions in attributes (not the case at 0.2.65). |
| Consumer-side override registry keyed by action name | New typed builder field on `ActionDef` holding a closure | **Rejected** — `ActionDef` is serializable schema (`action.rs:24`); a closure field breaks serde + the no-closure rule. |

**Installation:** No new crates. Phase 231 adds modules to `ferro-projections/src/` and re-exports from `lib.rs`.

**Version verification:** No external packages to verify — Phase 231 is internal to the workspace
(version 0.2.65, `Cargo.toml:18`) [VERIFIED: workspace Cargo.toml]. Existing deps (`serde`, `schemars 1`,
`thiserror 1`) are already pinned in `ferro-projections/Cargo.toml` [VERIFIED].

## Architecture Patterns

### System Architecture Diagram

```
  Agent / form caller
        │  tools/call {name: "submit", arguments: {id, ...}}
        ▼
  ferro-mcp-server::handle_write_call          (write_dispatch.rs:363)
        │  resolve ActionDef by name           (find_action, :80)
        │  validate required inputs            (validate_action_inputs, :99)
        ▼
  ferro-mcp-server::dispatch_write             (write_dispatch.rs:258)  ── security envelope
        │ 1. guard re-eval (LIVE) ─────────────┐  EXEC-02
        │ 2. idempotency check                 │
        │ 3. confirmation seam (feature)       │
        │ 4. ┌─────────────────────────────────┴───────────────┐
        │    │  DERIVED EXECUTOR (new, generic)                 │
        │    │     plan = derive_transition_plan(svc, name) ◄───┼── ferro-projections
        │    │            (pure, schema-only)                   │     (state.rs/action.rs)
        │    │     read entity by id (+tenant)  ── SeaORM       │  EXEC-01
        │    │     assert current_state ∈ plan.from_states      │
        │    │     re-check plan.guard (LIVE) ── EXEC-02        │
        │    │     set status = plan.to_state ── SeaORM persist │
        │    │     run override hook (if registered) ── EXEC-03 │
        │    └──────────────────────────────────────────────────┘
        │ 5. store idempotency result
        │ 6. audit (ferro-audit)
        ▼
  CallToolResult (structured)                  (write_dispatch.rs:462)

  ──────────────────────────────────────────────────────────────────
  BOOT (app startup):
    for svc in services: svc.validate()?       ── EXEC-04 hard gate
       └─ rejects ActionDef.transition_trigger with no matching event
          (already implemented at service.rs:404-418; make fatal at boot)
```

### Recommended Project Structure (ferro-projections additions)
```
ferro-projections/src/
├── executor.rs       # NEW: TransitionPlan, derive_transition_plan(), Error variants
├── action.rs         # unchanged (transition_trigger already present, :38)
├── state.rs          # unchanged (Transition.to is the derivation source, :79)
├── service.rs        # extend validate(): make undeclared-trigger fatal + plan round-trip (:404)
└── lib.rs            # re-export TransitionPlan, derive_transition_plan (:13-23)
```

### Pattern 1: Pure plan derivation (the EXEC-01 core)
**What:** A pure function maps `(ServiceDef, action_name)` to the transition facts already declared.
**When to use:** Called once per write, by the consumer's generic executor, before any DB I/O.
**Example:**
```rust
// Source: derived from state.rs:73-90 (Transition fields) + action.rs:38 (transition_trigger)
//         + service.rs:404-418 (existing trigger→event matching logic)
/// A serializable description of a state transition write, derived from the
/// declared StateMachine + ActionDef. Carries NO behavior — the consumer
/// interprets it against a concrete entity. (schema-only, no closures)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPlan {
    pub action: String,            // ActionDef.name
    pub event: String,             // = ActionDef.transition_trigger
    pub from_states: Vec<String>,  // every Transition.from with this event (cancel→3 sources)
    pub to_state: String,          // Transition.to — replaces the hand-written match
    pub guard: Option<String>,     // Transition.guard — re-checked live at execution (EXEC-02)
    pub effects: Vec<String>,      // Transition.actions ∪ ActionDef.effects (override-hook inputs)
}

pub fn derive_transition_plan(
    svc: &ServiceDef,
    action_name: &str,
) -> Result<TransitionPlan, Error> {
    let action = svc.actions.iter().find(|a| a.name == action_name)
        .ok_or_else(|| Error::Validation(format!("no action '{action_name}'")))?;
    let event = action.transition_trigger.as_deref()
        .ok_or_else(|| Error::Validation(format!("action '{action_name}' has no transition_trigger")))?;
    let sm = svc.state_machine.as_ref()
        .ok_or_else(|| Error::Validation("service has no state machine".into()))?;
    let matches: Vec<&Transition> = sm.states_for_event(event); // state.rs:238
    if matches.is_empty() {
        return Err(Error::Validation(format!(
            "transition_trigger '{event}' matches no transition" // EXEC-04 by construction
        )));
    }
    // All transitions for one event must converge to a single target (or this is ambiguous).
    let to_state = matches[0].to.clone();
    // ... collect from_states, guard, effects ...
    Ok(TransitionPlan { /* ... */ })
}
```

### Pattern 2: Override hook = consumer-side registry keyed by action name (EXEC-03)
**What:** A `HashMap<String, OverrideFn>` on the consumer (parallel to `WriteDispatcher`), run after persist.
**When to use:** Only the app-specific 20% (related-record writes, notifications) registers; common path empty.
**Example:**
```rust
// Source: mirrors the ExecutorFn boxed-future pattern at write_dispatch.rs:38-47
/// Runs AFTER the derived persist, inside dispatch_write, reusing the audit/
/// idempotency envelope. Receives the persisted result so it can chain writes.
pub type OverrideFn = Box<
    dyn Fn(&str, &Value, i64, &DatabaseConnection) // action, inputs, tenant, db
        -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send>> + Send + Sync,
>;
// Registered alongside the dispatcher; absent key = no override (common path).
```

### Anti-Patterns to Avoid
- **Closure or async fn returned from `ferro-projections`:** breaks the no-closure rule and pulls
  `sea-orm`/`tokio` into a schema crate (`ferro-projections/CLAUDE.md`, `Cargo.toml:18-21`). The crate
  returns a *plan* (data), the consumer supplies the *behavior*.
- **Reading `ctx.evaluated_guards` at execution time:** that is the Phase 218 list-time visibility cache
  (`render/mod.rs:42-50`); using it for authorization is the privilege-escalation class the existing
  comment warns against (`write_dispatch.rs:270-275`). Re-evaluate live (EXEC-02).
- **A new `match action_name` anywhere:** that re-introduces the exact duplication this phase deletes
  (`mcp.rs:97-102`). The to-state must come only from `Transition.to`.
- **A second transition-declaration surface** (e.g. an imperative executor DSL): violates the binding
  coherence constraint (REQUIREMENTS.md:42; `feedback_no_duplicate_control_surface`).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| event → transition lookup | A new index/match | `StateMachine::states_for_event()` (`state.rs:238`) | Already exists, returns `Vec<&Transition>`; handles multi-source events (`cancel`→3, `state.rs:707`) |
| current-state → outgoing transitions | A new map | `StateMachine::events_from_state()` (`state.rs:246`) | Already exists; useful to assert the entity is in a legal `from` state before transitioning |
| trigger↔event validation | A new validator | extend `ServiceDef::validate()` step 5 (`service.rs:404-418`) | The undeclared-trigger error already exists; EXEC-04 just makes it a fatal boot gate |
| guard re-evaluation loop | A new guard runner | the existing `GuardEvaluatorFn` loop in `dispatch_write` (`write_dispatch.rs:276-285`) | Live-state, fail-closed, already audited on deny (`:465-480`) |
| audit / idempotency / confirmation envelope | New plumbing | the existing `dispatch_write` pipeline (`write_dispatch.rs:258-351`) | The derived executor slots into step 4; steps 1-3, 5-6 are unchanged |

**Key insight:** Almost everything the derived executor needs already exists as schema queries
(`states_for_event`, `events_from_state`, `Transition.to`) and as consumer runtime
(`dispatch_write` envelope, `GuardEvaluatorFn`). Phase 231 is mostly **connecting declared facts to
existing machinery**, not building new subsystems. The only genuinely new code is the `TransitionPlan`
value type and the `derive_transition_plan` pure function.

## Common Pitfalls

### Pitfall 1: Trying to put the executor body in `ferro-projections`
**What goes wrong:** Adding `sea-orm`/`tokio` to `ferro-projections` to let it persist, or returning a closure.
**Why it happens:** "Derive the executor" sounds like "produce a callable", but the crate is schema-only.
**How to avoid:** `ferro-projections` returns a `TransitionPlan` (data). The consumer interprets it.
Verify `ferro-projections/Cargo.toml` still has only `schemars/serde/serde_json/thiserror` after the phase.
**Warning signs:** A new `[dependencies]` line in `ferro-projections/Cargo.toml`; an `async fn` or `Box<dyn Fn>` in `executor.rs`.

### Pitfall 2: Multi-source events collapse the wrong target
**What goes wrong:** `cancel` fires from `draft`, `submitted`, `processing` (`state.rs:693-700`). A naive
"first match" picks one `to` but ignores that the legal `from` set has 3 members, or worse assumes one source.
**Why it happens:** `states_for_event` returns a `Vec`, not a single transition (`state.rs:238`).
**How to avoid:** `from_states` is a `Vec`; assert the entity's `current_state ∈ from_states` before persisting.
If different sources have different `to` targets for the same event, that is a fan-in the plan must reject or
represent explicitly — decide in planning.
**Warning signs:** A test with a `cancel`-style multi-source event passes only for one source state.

### Pitfall 3: Guard from the transition vs. precondition from the action diverge
**What goes wrong:** `Transition.guard` (`state.rs:82`) and `ActionDef.preconditions` (`action.rs:35`) are two
guard surfaces. The order projection puts `is_manager` on **both** the transition (`order.rs:31`) and the action
(`order.rs:42`). The executor must not double-deny or skip one.
**Why it happens:** Two declaration sites for guards on the same logical transition.
**How to avoid:** Decide the union semantics in planning: the live guard loop in `dispatch_write` already runs
`action.preconditions` (`write_dispatch.rs:276`). EXEC-02 adds `plan.guard` — ensure they de-duplicate
(`is_manager` appearing in both should run once, not twice).
**Warning signs:** A guard evaluator invoked twice for the same name in one write.

### Pitfall 4: EXEC-04 attempted at compile time
**What goes wrong:** Writing a trybuild fixture expecting `Transition::new(...).transition_trigger("typo")` to
fail compilation. It won't — those are runtime builder calls (`order.rs:30-44`), invisible to the compiler.
**Why it happens:** The phase brief mentions trybuild infra (Phase 212), tempting a compile-time check.
**How to avoid:** EXEC-04 is **registration-time**: `ServiceDef::validate()` returns `Err` (`service.rs:411`);
the app must call it at boot and fail fast. trybuild is only appropriate if/when a `#[service]` attribute macro
declares transitions statically (not the case at 0.2.65 — `order.rs` is a plain `fn service_def()`).
**Warning signs:** A `tests/ui/.../fail/*.rs` fixture referencing a bad transition trigger.

## Runtime State Inventory

Not applicable — Phase 231 is a greenfield code addition (new `executor.rs` module + `validate()` extension).
No rename/refactor/migration; no stored data, live-service config, OS-registered state, secrets, or build
artifacts carry transition names. **None — verified by reading the phase scope (REQUIREMENTS.md:13-24) and
confirming no string-rename component.**

## Code Examples

### Deriving the to-state the app currently hand-writes
```rust
// Source: app/src/projections/order.rs:30 (declaration) vs app/src/controllers/mcp.rs:97-102 (duplication)
// TODAY — the app re-encodes the transition target:
let new_status = match action_name.as_str() {
    "submit" => "submitted",   // == Transition::new("draft", "submit", "submitted").to
    "approve" => "approved",   // == Transition::new("submitted", "approve", "approved").to
    "ship" => "shipped",       // == Transition::new("approved", "ship", "shipped").to
    _ => return Err(/* ... */),
};
// AFTER Phase 231 — the to-state is derived, the match is deleted:
let plan = derive_transition_plan(svc, action_name)?; // ferro-projections, pure
let new_status = &plan.to_state;                       // single source of truth
```

### Existing validation this phase promotes to a hard gate (EXEC-04)
```rust
// Source: ferro-projections/src/service.rs:404-418 (already implemented)
if let Some(ref sm) = self.state_machine {
    let event_names: HashSet<&str> = sm.transitions.iter().map(|t| t.event.as_str()).collect();
    for action in &self.actions {
        if let Some(ref trigger) = action.transition_trigger {
            if !event_names.contains(trigger.as_str()) {
                return Err(crate::Error::Validation(format!(
                    "action '{}' has transition_trigger '{}' that does not match any state machine event",
                    action.name, trigger
                )));
            }
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| App hand-writes `ExecutorFn` with `match action_name => new_status` | Derive `TransitionPlan` from declared `Transition.to`; generic executor consumes it | Phase 231 (this) | Deletes `mcp.rs:97-102`; one declaration backs the write |
| Guard re-eval only over `action.preconditions` | Re-eval over `preconditions ∪ transition.guard` (deduped) | Phase 231 (EXEC-02) | Transition-level guards (`order.rs:31`) enforced at execution, not just list-time |
| `validate()` errors are advisory (only `auth_controller.rs:53` calls it) | `validate()` is a hard boot gate over all registered services | Phase 231 (EXEC-04) | Undeclared-trigger drift caught at startup, not first call |

**Deprecated/outdated:** The app-supplied transition `match` in `make_write_dispatcher`
(`app/src/controllers/mcp.rs:68-114`) is the thing this milestone retires. Phase 231 builds the derivation;
Phase 232 wires it in and deletes the hand-written executor (REQUIREMENTS.md:57-59).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | All transitions sharing one `event` converge to a single `to` state (so a plan has one `to_state`) | Pattern 1, Pitfall 2 | If an event fans out to different targets per source, `to_state` must become per-source; plan shape changes. Verify against the synthetic catalog in planning. |
| A2 | The override hook running *after* persist (not before/instead) satisfies "without replacing base dispatch" (EXEC-03) | Pattern 2 | If apps need pre-persist side effects, a second hook point may be needed. Confirm with a sample app use case during discuss/plan. |
| A3 | EXEC-04 is registration-time (not compile-time) is acceptable to the milestone owner | Pitfall 4, Q3 | If a compile-time guarantee is required, a `#[service]` macro must first declare transitions statically — larger scope. The phrase "build OR registration time" (REQUIREMENTS.md:24) reads as either-acceptable. |
| A4 | `transition.guard ∪ action.preconditions` should be de-duplicated by name (run once) | Pitfall 3 | If they are semantically distinct guards that happen to share a name, dedup would skip a real check. The `order.rs` overlap (`is_manager` on both) suggests they are the same. |

## Open Questions

1. **Multi-target events** (A1)
   - What we know: `states_for_event` returns a `Vec<&Transition>` (`state.rs:238`); `cancel` has 3 sources (`state.rs:707`).
   - What's unclear: whether any real/synthetic service has one event mapping to *different* `to` states per source.
   - Recommendation: model `from_states: Vec<String>` + single `to_state`; add a `validate()` warning if an event's transitions disagree on `to`. Revisit if the catalog needs per-source targets.

2. **Guard union semantics** (A4)
   - What we know: both `Transition.guard` and `ActionDef.preconditions` exist and overlap in `order.rs`.
   - What's unclear: the exact de-dup/precedence rule.
   - Recommendation: union by name, run each guard once; `validate()` warns if a transition guard is not also a declared `GuardDef` (the existing check at `service.rs:391-401` already enforces transition guards reference declared guards).

3. **Override hook timing** (A2)
   - What we know: `dispatch_write` runs executor at step 4, audit at step 6 (`write_dispatch.rs:329-348`).
   - What's unclear: whether one post-persist hook covers all 20% cases.
   - Recommendation: ship the post-persist hook in Phase 231; defer a pre-persist seam unless a sample app demands it.

## Environment Availability

Not applicable — Phase 231 is a pure-Rust addition to an existing workspace crate. No external tools, services,
or runtimes beyond the standard `cargo` toolchain already required by the project. SeaORM/tokio are needed only
for the **consumer-side** executor (Phase 232) and are already present in `ferro-mcp-server`
(`write_dispatch.rs:13`) [VERIFIED].

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` + `insta` (snapshots) + `trybuild` (UI) |
| Config file | none — `cargo test` (workspace); `insta` configured per-crate; `trybuild` in `ferro-macros/tests` |
| Quick run command | `cargo test -p ferro-projections` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EXEC-01 | `derive_transition_plan(order_svc, "submit").to_state == "submitted"` (matches `Transition.to`, no match needed) | unit | `cargo test -p ferro-projections derive_transition_plan` | ❌ Wave 0 (`executor.rs`) |
| EXEC-01 | Deriving a plan for an action with no `transition_trigger` returns `Err` | unit | `cargo test -p ferro-projections derive_no_trigger` | ❌ Wave 0 |
| EXEC-01 | End-to-end: generic executor uses `plan.to_state` to persist, no `match action_name` in app | integration | `cargo test -p ferro-mcp-server` / `app` write-dispatch test | ✅ extend `app/src/tests/mcp_write_dispatch.rs` |
| EXEC-02 | Plan carries `guard`; executor denies when live guard returns false (reuses `dispatch_write` loop) | unit | `cargo test -p ferro-mcp-server guard_denied_at_call_time` | ✅ exists (`write_dispatch.rs:856`) — extend for transition guard |
| EXEC-02 | `ctx.evaluated_guards` is NOT consulted at execution | unit | `cargo test -p ferro-mcp-server` (assert evaluator called, not cache) | ✅ pattern exists (`:851-881`) |
| EXEC-03 | Registered override runs after persist; absent override = common path unchanged | integration | `cargo test -p ferro-mcp-server override_hook_runs` | ❌ Wave 0 |
| EXEC-03 | Override failure surfaces as an error without corrupting the audit envelope | integration | `cargo test -p ferro-mcp-server override_error` | ❌ Wave 0 |
| EXEC-04 | `ServiceDef::validate()` returns `Err` for `transition_trigger` with no matching event | unit | `cargo test -p ferro-projections validate_catches_unmatched_transition_trigger` | ✅ exists (`service.rs:1002`) |
| EXEC-04 | App boot fails fast when a registered service fails `validate()` | integration | `cargo test -p app boot_rejects_invalid_service` | ❌ Wave 0 |
| EXEC-04 | `derive_transition_plan` errors (not panics) on undeclared trigger — same fact as validate, by construction | unit | `cargo test -p ferro-projections derive_undeclared_trigger_errors` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-projections` (fast, no DB)
- **Per wave merge:** `cargo test -p ferro-projections -p ferro-mcp-server`
- **Phase gate:** Full suite green (`fmt + clippy --all --all-targets -D warnings + test --all-features`)
  before `/gsd-verify-work`. NOTE: `cargo test --all-features` recurrently disk-full-fails on this host
  (`project_ferro_disk_full_test_gate`) — check `df` and clean `target/` first.

### Wave 0 Gaps
- [ ] `ferro-projections/src/executor.rs` — `TransitionPlan` + `derive_transition_plan()`; covers EXEC-01, EXEC-04
- [ ] `ferro-projections` unit tests for derivation (to-state, no-trigger, undeclared, multi-source) — covers EXEC-01/04
- [ ] `ferro-mcp-server` override-hook tests (`override_hook_runs`, `override_error`) — covers EXEC-03
- [ ] `ferro-mcp-server` transition-guard re-eval test (extend `guard_denied_at_call_time`) — covers EXEC-02
- [ ] `app` boot-validation integration test (`boot_rejects_invalid_service`) — covers EXEC-04 as a real gate
- Framework install: none — all test tooling already present in the workspace.

## Security Domain

`security_enforcement` is not explicitly `false` in config → included. Phase 231 sits directly on the
write-authorization path, so the security surface is load-bearing.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V1 Architecture | yes | Single declared source of truth for transitions (coherence constraint); no parallel control surface (REQUIREMENTS.md:42) |
| V4 Access Control | yes | Server-side guard re-evaluation at execution against LIVE state (EXEC-02); never trust the list-time `evaluated_guards` cache (`write_dispatch.rs:270-275`) |
| V5 Input Validation | yes | `validate_action_inputs` enforces declared inputs (`write_dispatch.rs:99`); plan derivation rejects undeclared triggers (EXEC-04) |
| V7 Error Handling | yes | Executor errors must not leak SQL/table/column names — existing redaction at `write_dispatch.rs:497-504` is the model |
| V11 Business Logic | yes | The state machine IS the business-logic guard; deriving from it prevents an illegal transition being driven through a hand-written gap |

### Known Threat Patterns for the derived write path

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Guard bypass via direct `tools/call` (skip `tools/list`) | Elevation of Privilege | Live guard re-eval in `dispatch_write` (`:276-285`); plan carries `guard`, executor re-checks — never reads `evaluated_guards` (EXEC-02) |
| Driving an illegal transition (entity not in a legal `from` state) | Tampering | Plan's `from_states` asserted against the live entity's `current_state` before persist (Pitfall 2) |
| Executor/StateMachine drift introducing a silent unguarded path | Tampering / EoP | EXEC-04 fatal `validate()` at boot — drift cannot exist by construction |
| Override hook used to escalate (bypass base guard) | Elevation of Privilege | Override runs AFTER the guarded persist (EXEC-03), inside the same audited envelope; it cannot suppress the base guard or transition |
| Cross-tenant write through the derived path | Information Disclosure / Tampering | Tenant scoping stays in the executor's `find_for_tenant`/`Column::TenantId.eq` predicate (`mcp.rs:86-95`); the plan is tenant-agnostic data |

## Sources

### Primary (HIGH confidence)
- `ferro-projections/src/state.rs` — `StateMachine`, `Transition { from, event, to, guard, actions }`, `states_for_event` (:238), `events_from_state` (:246), multi-source `cancel` (:707)
- `ferro-projections/src/action.rs` — `ActionDef.transition_trigger` (:38), `preconditions` (:35), builder (:86)
- `ferro-projections/src/service.rs` — `ServiceDef.state_machine` (:79), `validate()` trigger↔event check (:404-418), transition-guard-declared check (:391-401)
- `ferro-projections/src/lib.rs` — exports; confirms no `Executor`/`derive_default_executor` exists (:13-23)
- `ferro-projections/src/render/mod.rs` — `BaseContext.evaluated_guards` semantics (:42-50)
- `ferro-projections/src/error.rs` — `Error` enum (thiserror)
- `ferro-projections/Cargo.toml` — deps = schemars/serde/serde_json/thiserror only (:18-21); **no sea-orm/tokio**
- `ferro-projections/CLAUDE.md` — no-closure / no-runtime-logic / boundary rules
- `ferro-mcp-server/src/write_dispatch.rs` — `WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn` (:38-73), `dispatch_write` pipeline (:258-351), live guard loop (:276-285), error redaction (:497-504), tests (:856, :889, :953)
- `app/src/controllers/mcp.rs` — `make_write_dispatcher` with the hand-written `match action_name => new_status` duplication (:68-114, esp. :97-102)
- `app/src/projections/order.rs` — declaration site: `Transition.to` vs `ActionDef.transition_trigger` (:30-44)
- `ferro-macros/tests/action_macro.rs` + `tests/ui/` — trybuild harness (compile-fail/pass fixtures)
- `.planning/REQUIREMENTS.md`, `.planning/STATE.md` — milestone scope, EXEC-01..05, phase split

### Secondary (MEDIUM confidence)
- None — all claims grounded in primary source files.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — confirmed zero new deps by reading `ferro-projections/Cargo.toml` directly.
- Architecture: HIGH — the derivation source (`Transition.to`) and the duplication site (`mcp.rs:97-102`) were both read and quoted.
- Pitfalls: HIGH — each is grounded in a specific file:line (no-closure rule, multi-source events, guard overlap, runtime builder chains).
- Open questions (A1–A4): MEDIUM — design choices for the planner, not unknowns about the codebase.

**Research date:** 2026-06-16
**Valid until:** 2026-07-16 (stable — internal workspace code, no fast-moving external deps)

## Recommended Approach (the 4 design questions)

**Q1 — Shape of the derived executor / how it reaches a concrete SeaORM entity.**
Recommendation: a **serializable `TransitionPlan` value type + pure `derive_transition_plan(&ServiceDef, &str) -> Result<TransitionPlan, Error>`** in `ferro-projections`. The plan is data (`to_state`, `from_states`, `guard`, `effects`), not behavior. The **consumer's** generic executor (Phase 232, living where `dispatch_write` already runs) interprets the plan against the concrete entity. ferro-projections reaches **no** SeaORM entity — that is precisely why it returns a plan. *Why ferro-idiomatic:* the crate is schema-only with no `sea-orm`/`tokio` (`Cargo.toml:18-21`) and forbids closures (`CLAUDE.md`); every existing projection type is a serializable value (`ServiceDef`, `StateMachine`, `ActionDef`). A plan fits this mold exactly; a closure-returning function would break it.

**Q2 — Override hook (EXEC-03) without replacing base dispatch.**
Recommendation: a **consumer-side registry of boxed-future closures keyed by action name** (mirroring `ExecutorFn` at `write_dispatch.rs:38-47`), invoked **after** the derived persist inside `dispatch_write` (between steps 4 and 6), reusing the existing audit/idempotency envelope. Absent key = common path, declaration-only. *Why ferro-idiomatic:* it follows the established `WriteDispatcher` callback pattern (boxed futures, no `async-trait` dep), keeps closures in the runtime crate (not the schema crate), and never touches `ActionDef` (which must stay serializable). Do **not** add a closure field to `ActionDef` — that breaks serde and the no-closure rule.

**Q3 — EXEC-04 build- vs registration-time.**
Recommendation: **registration-time**, by extending `ServiceDef::validate()` (the undeclared-trigger error already exists at `service.rs:404-418`) and making the app call `validate()` for every registered service at boot, failing fast. *Why ferro-idiomatic:* services are runtime builder chains (`order.rs:10`, a plain `fn`), so the compiler/trybuild cannot see transition names — a compile-time check is structurally impossible without first introducing a `#[service]` attribute macro that declares transitions statically (out of scope, larger than this phase). REQUIREMENTS.md:24 explicitly accepts "build OR registration time". Reserve trybuild for a future static-declaration milestone. Pair the validate() gate with a `derive_transition_plan` error on the same condition so the two checks cannot diverge (the plan derivation *is* the same fact, enforced by construction).

**Q4 — Where guard re-evaluation lives and its reusability.**
Recommendation: **reuse the existing live `GuardEvaluatorFn` loop in `dispatch_write`** (`write_dispatch.rs:276-285`), extending it to also cover `plan.guard` (the transition-level guard, `state.rs:82`) in addition to `action.preconditions`, de-duplicated by name. It is async, runs against live DB state, is fail-closed, and already audits denials (`:465-480`). *Why ferro-idiomatic:* it is the one place the project already designates as the authorization gate, with an explicit comment forbidding use of `ctx.evaluated_guards` for authorization (`:270-275`). EXEC-02 is therefore an extension of an existing, tested loop — not a new guard runner. The transition guard (`order.rs:31`) and action precondition (`order.rs:42`) overlap on `is_manager`, so dedup-by-name is required to avoid double-evaluation.
