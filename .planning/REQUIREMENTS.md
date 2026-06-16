# Requirements: v16.0 Write-Boundary AX — StateMachine-Derived Executor

**Milestone goal:** Eliminate the "declare twice" duplication on the projection write path. Today an `ActionDef` whose `transition_trigger` names a StateMachine transition still requires a hand-written `WriteDispatcher` `match` arm that re-encodes the same transition facts — the workflow is declared twice, with nothing keeping the two in sync. v16.0 derives a default write executor from the `ServiceDef` StateMachine the framework already knows (state read → guard re-eval → transition → persist), with an override hook for the app-specific 20% (side effects, related-record writes, custom post-transition work).

**Grounding:** Verified against code 2026-06-16 at 0.2.65 — `ferro-projections` has zero executor-derivation machinery (no `Executor` type, no `derive_default_executor`); `StateMachine` (`ferro-projections/src/state.rs`) and `ActionDef.transition_trigger` (`action.rs`) are purely declarative; the executor is hand-written in the consumer/`WriteDispatcher`. The READ path is complete (visual `JsonUiRenderer`, `ferro-text::TextRenderer`, `ferro-mcp-server::McpRenderer`) and Phase 213 closed the render-content gaps. This is the last load-bearing gap in the projection/intent killer feature's WRITE path. The v15.0 MCP write dispatch (`AMCP-04`) is the immediate downstream consumer of the derived executor.

**Coherence constraint (binding):** the derived executor must read FROM the existing StateMachine / `ActionDef` declarations. It must NOT introduce a parallel imperative control surface or a second source of truth for transitions. Projection/intent stays the single source of truth (see `feedback_no_duplicate_control_surface`).

---

## v16 Requirements

### Executor Derivation

- [x] **EXEC-01**: A developer declares a state-transition write solely by naming a `StateMachine` transition on the `ActionDef` (`transition_trigger`), and the framework derives the default executor — state read → guard re-evaluation → transition → persist — from the `StateMachine` declaration alone. No hand-written `WriteDispatcher` `match` arm is required for the common path.
- [x] **EXEC-02**: The derived executor re-evaluates the transition's guard server-side at execution time (reusing the `evaluated_guards` surface), and rejects a transition whose guard does not hold — an agent or caller cannot drive an illegal transition through the derived path.

### Override Hook

- [x] **EXEC-03**: A developer attaches app-specific side effects (related-record writes, notifications, custom post-transition logic) to a derived executor through an override hook, without replacing the base transition dispatch or re-declaring the transition. The common path stays declaration-only; only the 20% writes code.

### Sync-by-Construction

- [x] **EXEC-04**: Executor/StateMachine drift is structurally prevented — an `ActionDef` or override that references a transition the `StateMachine` does not declare is rejected at build or registration time, not at runtime. The "declared twice, fell out of sync" bug class is eliminated by construction.

### Single Source Across Write Surfaces

- [x] **EXEC-05**: The derived executor drives writes from the single `ServiceDef` across the existing write surfaces — the v15.0 MCP write dispatch (`AMCP-04`) and the visual/form write path — so one declaration backs writes in every modality with no per-channel executor.

---

## Future Requirements (deferred)

- **Derived executor for non-transition writes** — pure create/update `ActionDef`s with no `StateMachine` transition keep their current path for now; deriving a default executor for plain CRUD writes (without a state machine) is a follow-up once the transition-backed path is proven.
- **Operating-AX: NL description quality ≡ classification accuracy** — the inbound-NL side of write AX is gated on a funded COMP-03 live run; out of scope here (this milestone is authoring-AX, not operating-AX).
- **Projection `body` slot** — the optional free-form rich-content slot (`emit_body_placeholder`) remains deferred; unrelated to the write boundary.

## Out of Scope

| Item | Reason |
|------|--------|
| A new imperative executor DSL / parallel control surface | Violates the coherence constraint — the executor must be derived FROM the StateMachine, not a second way to declare transitions |
| A new crate for executor derivation | Derivation belongs in `ferro-projections` (it owns `StateMachine`/`ActionDef`); consumers (`ferro-mcp-server`, the form write path) call it. A new crate would duplicate the control surface |
| gestiscilo adoption / consumer migration | Consumer-repo follow-up; ferro phases deliver the framework capability + synthetic validation only (`feedback_cross_repo_phase_split`) |
| Replacing the StateMachine or guard model | The derivation reads the existing `StateMachine` + guard surfaces as-is; no redesign of either |

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| EXEC-01 | Phase 231 | Complete |
| EXEC-02 | Phase 231 | Complete |
| EXEC-03 | Phase 231 | Complete |
| EXEC-04 | Phase 231 | Complete |
| EXEC-05 | Phase 232 | Complete |

*Phase assignments filled by the roadmapper. EXEC-01..04 (derivation + guard re-eval + override hook + sync-by-construction in `ferro-projections`) map to Phase 231; EXEC-05 (wiring the derived executor across the MCP + visual/form write surfaces, retiring the hand-written `WriteDispatcher`) maps to Phase 232. Every v16.0 requirement maps to exactly one phase — 5/5 covered, no orphans.*
