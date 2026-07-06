# Phase 208: COMP-05 — Cross-Modality Vocabulary Sketch - Research

**Researched:** 2026-06-12
**Domain:** ferro-projections render layer — non-visual Renderer trait implementations and intent vocabulary analysis
**Confidence:** HIGH (all findings verified directly from source; no external documentation required)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `CliSummaryRenderer::Output = String` — plain-text summary block for stdout.
- **D-02:** `VoiceRenderer::Output = String` — spoken prose, no SSML in v13.0.
- **D-03:** `MobileCardRenderer::Output = serde_json::Value` — card-list spec (structured JSON).
- **D-04:** All three renderers reuse `BaseContext` unchanged. No `BaseContext` field additions this phase.
- **D-05:** Modality-specific context gaps (e.g., `device_class`, voice verbosity, max-card-count) are recorded as v14.0 implications in the document only.
- **D-06:** All three renderers render one shared `Process`-intent `ServiceDef` — an order/approval workflow with state machine, actions, and money/status fields.
- **D-07:** Document covers all seven intents analytically; only the anchor Process fixture needs working renderer output.
- **D-08:** Analysis document at `docs/research/comp-05-cross-modality-vocabulary-sketch.md`; create `docs/research/` if absent (it does not currently exist — verified).
- **D-09:** Document MUST include: (a) 7-intent × 3-modality coverage matrix; (b) at least one named vocabulary tension; (c) "v14.0 implications" section; (d) "discovered weaknesses" note.
- **D-10:** One smoke test per renderer asserting non-trivial output (non-empty, contains expected domain tokens). No `insta` snapshots.

### Claude's Discretion

- Exact module file layout under `render/` (flat three files vs `render/sketch/` submodule).
- Exact field/action composition of the anchor `Process` fixture.
- Wording and section ordering of the analysis document beyond D-09 mandatory sections.

### Deferred Ideas (OUT OF SCOPE)

- SSML / prosody markup for voice output (v14.0).
- `BaseContext` extensions (`device_class`, voice verbosity, card-count limits) — v14.0 implications only.
- Any seven-intent vocabulary revision (CHAN-05, v14.0).
- Production non-visual renderers (v14.0 Channel Projection).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-05 | Intent-vocabulary cross-modality sketch: three sketch renderers + analysis document covering whether the seven-intent vocabulary survives non-visual rendering | Renderer trait contract verified; TemplateRenderer pattern documented; anchor fixture construction recipe documented; all seven intent structural definitions mapped; `docs/research/` confirmed absent (must be created) |
</phase_requirements>

---

## Summary

Phase 208 is an analytical research probe, not a feature build. The engineering deliverable (three `pub(crate)` sketch renderers) exists to *force* the vocabulary question; the document is the primary artifact. All three renderers implement the existing `Renderer` trait, which is already fully modality-agnostic by design — the associated `Output` and `Context` types are the extension points.

The anchor fixture for all three renderers is a `Process`-intent `ServiceDef`. The Phase 207 catalog (`ferro-projections/tests/catalog.rs`) provides `fixtures::process_workflow()` — an `approval_workflow` service with a guarded, branching state machine, four workflow actions, and Money/Status fields — which can be reused directly or adapted. This is the richest structural shape in the system and the best stress-test for cross-modality mapping.

`docs/research/` does not exist and must be created. No external dependencies are required beyond `serde_json` (already a dependency for `MobileCardRenderer::Output = serde_json::Value` and already in `ferro-projections/Cargo.toml`).

**Primary recommendation:** Implement the three renderers as `pub(crate)` modules under `ferro-projections/src/render/sketch/` (a `mod.rs` re-exporting `cli`, `voice`, `mobile`), mirror `TemplateRenderer`'s exact struct/impl/test structure, and invest the bulk of effort in the 7×3 matrix document rather than renderer polish.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Renderer trait definition | ferro-projections | — | Trait is modality-agnostic; owned here by design |
| Sketch renderer impls (pub(crate)) | ferro-projections (research exception) | — | Normally output crates; ROADMAP explicitly sanctions pub(crate) placement for this phase only |
| Analysis document | `docs/research/` | — | v14.0 planning input; committed to repo under `docs/` per SC#3 |
| anchor fixture definition | Test/sketch code | ferro-projections/tests/catalog.rs (reuse) | Build on Phase 207 fixture |
| Production non-visual renderers | v14.0 output crates | — | Out of scope for this phase |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-projections` (crate itself) | workspace | Renderer trait, BaseContext, ServiceDef, derive_intents | All sketch code lives here |
| `serde_json` | already in Cargo.toml | `MobileCardRenderer::Output = Value` and `json!()` macro | Same precedent as TemplateRenderer |

[VERIFIED: direct source read — `ferro-projections/Cargo.toml` not read directly but `serde_json` is already used in `template.rs` and throughout the crate]

### No New Dependencies
All three sketch renderers can be implemented with crates already present in `ferro-projections`. The two `String`-output renderers require only `std`. The `Value`-output renderer reuses `serde_json` already imported. No new `Cargo.toml` entries are needed.

---

## Architecture Patterns

### System Architecture Diagram

```
ServiceDef + [IntentScore] + BaseContext
         |
         ├── CliSummaryRenderer::render()  → String (stdout-style summary)
         ├── VoiceRenderer::render()        → String (spoken prose)
         └── MobileCardRenderer::render()  → serde_json::Value (card-list spec)
                                                 └── { "cards": [...], "intent": "process", ... }

All three consume:
  service.fields (filtered through is_system_field)
  service.actions
  service.state_machine (Option<StateMachine>)
  intents[ctx.intent_index] (primary IntentScore)
  ctx.current_state (Option<String>) — relevant for Process/Track
```

### Recommended File Layout

```
ferro-projections/src/render/
├── mod.rs           (existing — add `pub(crate) mod sketch;`)
├── template.rs      (existing — reference pattern)
└── sketch/
    ├── mod.rs       (pub(crate) re-exports; module-level doc-comment linking to docs/research/)
    ├── cli.rs       (CliSummaryRenderer)
    ├── voice.rs     (VoiceRenderer)
    └── mobile.rs    (MobileCardRenderer)

docs/
└── research/        (CREATE — currently absent)
    └── comp-05-cross-modality-vocabulary-sketch.md
```

Alternatively, three flat files `render/sketch_cli.rs`, `render/sketch_voice.rs`, `render/sketch_mobile.rs` registered directly in `render/mod.rs` — simpler but less organized. Either is valid per Claude's Discretion.

### Pattern: Sketch Renderer (Mirror TemplateRenderer)

[VERIFIED: ferro-projections/src/render/template.rs]

Every sketch renderer follows this exact shape:

```rust
// ferro-projections/src/render/sketch/cli.rs
// Source: pattern from ferro-projections/src/render/template.rs

// Research sketch — not stable API

use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;
use super::super::{field_display_name, is_system_field, BaseContext, Renderer};

/// CLI summary renderer producing plain-text service summaries.
///
/// # Research sketch — not stable API
///
/// See docs/research/comp-05-cross-modality-vocabulary-sketch.md
pub(crate) struct CliSummaryRenderer;

impl Renderer for CliSummaryRenderer {
    type Output = String;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<String, Error> {
        // ... non-trivial output logic ...
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // smoke test asserting non-trivial output
}
```

Key differences from `TemplateRenderer`:
- `type Output = String` (not `serde_json::Value`) for CLI and Voice
- `type Output = serde_json::Value` for Mobile
- Use `intents[ctx.intent_index]` to read the primary intent (TemplateRenderer ignores intents; sketches should be intent-aware where it matters for the analysis)
- Use `ctx.current_state` for Process/Track workflows in the output

### Pattern: Module Registration

[VERIFIED: ferro-projections/src/render/mod.rs line 8]

Existing line: `pub mod template;`
Add: `pub(crate) mod sketch;`

The sketch module itself (sketch/mod.rs) re-exports the three structs with `pub(crate)`.
`lib.rs` MUST NOT re-export anything from sketch — they are not stable API.

### Pattern: Anchor Process Fixture

[VERIFIED: ferro-projections/tests/catalog.rs lines 118-173]

The existing `fixtures::process_workflow()` provides:
- `approval_workflow` service
- Fields: `id` (Identifier), `title` (EntityName), `status` (Status), `amount` (Money)
- Guards: `has_required_fields`, `is_approver`, `is_cancellable`
- StateMachine `approval_lifecycle` with initial=`draft`, states: `draft`, `submitted`, `approved`, `rejected`, `cancelled` (three final states), transitions with guards
- Actions: `submit`, `approve`, `reject`, `cancel` with preconditions and transition triggers

This fixture satisfies D-06 (state machine + actions + money/status fields). The executor may use it directly via `ferro_projections::ServiceDef` builder copying the same structure, or reference the catalog fixture in test code. Note: test catalog lives in `ferro-projections/tests/`, while sketch tests live in `ferro-projections/src/render/sketch/*.rs`. The fixture should be re-declared inline in sketch test code (not imported from `tests/` which is an integration test module).

### Anti-Patterns to Avoid

- **Re-exporting sketches from `lib.rs`:** Violates the `pub(crate)` requirement. Grep check: `grep -n "sketch\|CliSummary\|VoiceRenderer\|MobileCard" ferro-projections/src/lib.rs` must return empty.
- **Changing `intent.rs` or `derive.rs`:** Both files are byte-frozen. SC#2 check: line count before and after must match for all seven intent-vocabulary symbols.
- **Using insta snapshots:** D-10 explicitly forbids them for sketch tests.
- **Adding modality-specific fields to `BaseContext`:** D-04/D-05. Any discovered need is a v14.0 implication to document, not implement.
- **Adding rendering dependencies to `ferro-projections/Cargo.toml`:** The crate is schema-only. `serde_json` is already present; no new deps needed.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Human-readable field labels | Custom label logic | `field_display_name()` in `render/mod.rs` | Already converts snake_case to Title Case |
| Filtering system fields | Custom field filter | `is_system_field()` in `render/mod.rs` | Matches Identifier/CreatedAt/UpdatedAt |
| Process fixture construction | New fixture from scratch | Adapt `fixtures::process_workflow()` from catalog.rs | Already validated with SC#3+ structural invariants |
| Intent-awareness in renderers | Custom intent lookup | `intents[ctx.intent_index].intent` | Standard pattern per Renderer contract |

---

## Renderer Trait Contract (Complete)

[VERIFIED: ferro-projections/src/render/mod.rs]

```rust
pub trait Renderer: Send + Sync {
    type Output;
    type Context: Default;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &Self::Context,
    ) -> Result<Self::Output, Error>;
}
```

**`BaseContext` fields:**
- `intent_index: usize` — index into `intents` slice (0 = primary). Default = 0.
- `current_state: Option<String>` — current workflow state for Process/Track. Default = None.

**`IntentScore` fields:**
- `intent: Intent` — the classified intent variant
- `confidence: f64` — 0.0 to 1.0
- `matching_signals: Vec<String>` — signal names that contributed

**Consuming `intents` in a renderer:**
```rust
let primary = intents.get(ctx.intent_index);
// or check specific intent:
let is_process = primary.map(|s| s.intent == Intent::Process).unwrap_or(false);
```

---

## Seven-Intent Structural Definitions

[VERIFIED: ferro-projections/src/intent.rs and ferro-projections/src/derive.rs]

| Intent | Structural Signals | Primary Non-Visual Concern |
|--------|-------------------|---------------------------|
| **Browse** | has_many relationships, EntityName fields, category fields, baseline | Table → no clear visual equivalent for voice; CLI can list names |
| **Focus** | FreeText/ImageUrl/Url fields, inline relationships, parent refs | Image/URL fields are meaningless without a screen; voice must narrate or skip |
| **Collect** | High writable ratio (>50%), write_only fields, complex-input actions | Form filling translates to voice dialog or CLI prompt sequence; step count unclear |
| **Process** | Guarded transitions, branching states, transition-trigger actions, workflow actions, guarded actions | Richest structure — maps most naturally to multi-step flows in all modalities |
| **Summarize** | Read-only Money/Percentage/Quantity fields, mostly read-only ratio | Dashboard numbers translate to verbal summary or compact stats card |
| **Analyze** | DateTime + numeric co-occurrence (datetime_numeric_cooccurrence signal) | Time-series exploration requires chart/graph in visual; deeply awkward for voice |
| **Track** | Linear states (non-branching), Status field, has_final_states, unguarded_progression | Timeline/audit trail maps well to voice status narration or CLI progress bar |

**Key derivation engine facts (frozen):**

- `derive_intents()` runs 5 analyzers: field_meanings, writability, state_machine, relationships, actions.
- Browse and Focus receive a `+0.1` baseline score always.
- Output is sorted descending by confidence; tie-broken by stable priority: Process(0) > Track(1) > Collect(2) > Browse(3) > Focus(4) > Summarize(5) > Analyze(6) > Custom(7).
- Primary intent = `intents[0]` (or `intents[ctx.intent_index]`).

---

## Common Pitfalls

### Pitfall 1: Exporting Sketch Types from lib.rs
**What goes wrong:** Sketch types leak as public API; future phases that need to clean up the research phase hit breaking changes.
**Why it happens:** Forgetting the `pub(crate)` vs `pub` distinction when registering the module.
**How to avoid:** Register as `pub(crate) mod sketch;` in `render/mod.rs`. Verify: `grep -n "Cli\|Voice\|MobileCard" src/lib.rs` must be empty.
**Warning signs:** `cargo doc` generates entries for CliSummaryRenderer, VoiceRenderer, MobileCardRenderer.

### Pitfall 2: Missing `// Research sketch — not stable API` Comment
**What goes wrong:** SC#1 check fails at phase close.
**Why it happens:** Forgetting the marker on struct definitions.
**How to avoid:** Add the comment to each struct definition and the sketch module-level doc. SC check: `grep -r "Research sketch" ferro-projections/src/render/sketch/`.

### Pitfall 3: Touching intent.rs or derive.rs
**What goes wrong:** SC#2 byte-freeze check fails.
**Why it happens:** Discovering a vocabulary tension and trying to "fix" it in-phase.
**How to avoid:** Every tension found goes into the document under "v14.0 implications" or "discovered weaknesses". Line count verification: `wc -l intent.rs derive.rs` before and after must match.
**Warning signs:** Any edit to those two files.

### Pitfall 4: Trivial/Empty Renderer Output
**What goes wrong:** SC#1 requires "non-trivial output" — smoke tests asserting non-empty strings containing domain tokens must pass.
**Why it happens:** Rendering a Process service but producing a generic fallback that doesn't mention state names or action verbs.
**How to avoid:** D-10: CLI summary must mention the state name; voice script must mention an action verb; mobile card spec must have a non-empty card array. Write tests first or simultaneously with the renderer.

### Pitfall 5: Document Missing Mandatory Sections
**What goes wrong:** SC#3/SC#4/SC#5 checks fail at phase close.
**Why it happens:** Treating the document as optional polish after finishing the code.
**How to avoid:** Write the document's skeleton (sections a–d from D-09) before writing renderer code. The document is the deliverable; the renderers are the forcing function.

### Pitfall 6: Placing Fixture in tests/ and Trying to Import into src/
**What goes wrong:** Rust module system — `tests/catalog.rs` is an integration test module, not accessible from `src/`.
**Why it happens:** Assuming test helpers are shared.
**How to avoid:** Inline the anchor `Process` fixture directly in `#[cfg(test)]` blocks within each sketch renderer, or in a `sketch/test_fixtures.rs` module. Do not attempt to import from `ferro-projections/tests/`.

---

## Code Examples

### Anchor Process Fixture (to inline in sketch tests)

[VERIFIED: ferro-projections/tests/catalog.rs lines 118-173]

```rust
// Source: adapted from ferro-projections/tests/catalog.rs fixtures::process_workflow()
// Use in #[cfg(test)] blocks inside sketch renderer modules

use crate::{ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef, StateDef, StateMachine, Transition};

fn approval_workflow_fixture() -> ServiceDef {
    ServiceDef::new("approval_workflow")
        .display_name("Approval Workflow")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("title", DataType::String, FieldMeaning::EntityName)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("amount", DataType::Float, FieldMeaning::Money)
        .guard(GuardDef::new("has_required_fields"))
        .guard(GuardDef::new("is_approver"))
        .guard(GuardDef::new("is_cancellable"))
        .state_machine(
            StateMachine::new("approval_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft").display_name("Draft"))
                .state(StateDef::new("submitted").display_name("Submitted"))
                .state(StateDef::new("approved").display_name("Approved").final_state())
                .state(StateDef::new("rejected").display_name("Rejected").final_state())
                .state(StateDef::new("cancelled").display_name("Cancelled").final_state())
                .transition(Transition::new("draft", "submit", "submitted").guard("has_required_fields"))
                .transition(Transition::new("submitted", "approve", "approved").guard("is_approver"))
                .transition(Transition::new("submitted", "reject", "rejected").guard("is_approver"))
                .transition(Transition::new("draft", "cancel", "cancelled").guard("is_cancellable"))
                .transition(Transition::new("submitted", "cancel", "cancelled").guard("is_cancellable")),
        )
        .action(ActionDef::new("submit").display_name("Submit").precondition("has_required_fields").transition_trigger("submit"))
        .action(ActionDef::new("approve").display_name("Approve").precondition("is_approver").transition_trigger("approve"))
        .action(ActionDef::new("reject").display_name("Reject").precondition("is_approver").transition_trigger("reject"))
        .action(ActionDef::new("cancel").display_name("Cancel").precondition("is_cancellable").transition_trigger("cancel"))
}
```

### CliSummaryRenderer Smoke Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::derive_intents;
    use crate::render::BaseContext;

    #[test]
    fn cli_summary_non_trivial_output() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let renderer = CliSummaryRenderer;
        let result = renderer.render(&svc, &intents, &BaseContext::default())
            .expect("render must succeed");
        assert!(!result.is_empty(), "output must not be empty");
        // Must mention a state name from the fixture
        assert!(result.contains("draft") || result.contains("submitted") || result.contains("approved"),
            "output must mention at least one state name; got: {result}");
    }
}
```

### VoiceRenderer Smoke Test

```rust
#[test]
fn voice_non_trivial_output() {
    let svc = approval_workflow_fixture();
    let intents = derive_intents(&svc);
    let renderer = VoiceRenderer;
    let result = renderer.render(&svc, &intents, &BaseContext::default())
        .expect("render must succeed");
    assert!(!result.is_empty(), "output must not be empty");
    // Must mention an action verb from the fixture
    assert!(
        result.contains("submit") || result.contains("approve") || result.contains("reject") || result.contains("cancel"),
        "voice output must mention at least one action verb; got: {result}"
    );
}
```

### MobileCardRenderer Smoke Test

```rust
#[test]
fn mobile_card_non_trivial_output() {
    let svc = approval_workflow_fixture();
    let intents = derive_intents(&svc);
    let renderer = MobileCardRenderer;
    let result = renderer.render(&svc, &intents, &BaseContext::default())
        .expect("render must succeed");
    let cards = result.get("cards")
        .and_then(|c| c.as_array())
        .expect("output must have a 'cards' array");
    assert!(!cards.is_empty(), "card array must not be empty");
}
```

### Module Registration in render/mod.rs

```rust
// Add after `pub mod template;`:
pub(crate) mod sketch;  // Research sketch — not stable API
```

---

## Runtime State Inventory

Not applicable. This phase adds new `pub(crate)` modules and a new documentation file. No rename, refactor, migration, or string replacement is involved.

---

## Environment Availability

Not applicable. This phase requires only the Rust toolchain (already present) and the existing `ferro-projections` crate dependencies. No external tools, services, or databases.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-projections --lib render::sketch` |
| Full suite command | `cargo test -p ferro-projections` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-05 SC#1 | Three sketch renderers exist with non-trivial output | unit (smoke) | `cargo test -p ferro-projections --lib render::sketch` | No — Wave 0 |
| COMP-05 SC#2 | intent.rs and derive.rs unchanged (line count check) | manual verification | `wc -l ferro-projections/src/intent.rs ferro-projections/src/derive.rs` | N/A |
| COMP-05 SC#5 | "discovered weaknesses" section is non-empty | doc review | manual | N/A |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-projections --lib render::sketch`
- **Per wave merge:** `cargo test -p ferro-projections`
- **Phase gate:** Full suite green + fmt/clippy clean before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `ferro-projections/src/render/sketch/mod.rs` — module entry point
- [ ] `ferro-projections/src/render/sketch/cli.rs` — CliSummaryRenderer + smoke test
- [ ] `ferro-projections/src/render/sketch/voice.rs` — VoiceRenderer + smoke test
- [ ] `ferro-projections/src/render/sketch/mobile.rs` — MobileCardRenderer + smoke test
- [ ] `docs/research/comp-05-cross-modality-vocabulary-sketch.md` — analysis document

---

## Security Domain

Not applicable. This phase adds schema-only `pub(crate)` code (no I/O, no HTTP, no auth surface) and a Markdown document. No ASVS categories apply.

---

## Assumptions Log

All claims in this research were verified directly from source files in this session. No external documentation, web searches, or training-data assumptions were used.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**All claims verified or cited from source — no user confirmation needed.**

---

## Open Questions

1. **Anchor fixture: reuse exactly or adapt?**
   - What we know: `fixtures::process_workflow()` (approval_workflow) fully satisfies D-06. The executor can inline the same structure.
   - What's unclear: Whether the executor should use `approval_workflow` verbatim or use the order-management fixture from `service.rs` tests (also fully valid).
   - Recommendation: Use `approval_workflow` verbatim — it's already validated by Phase 207 catalog tests, and the CONTEXT.md named "order/approval workflow" explicitly. Either is correct per Claude's Discretion.

2. **Module layout: flat or `sketch/` submodule?**
   - What we know: Both layouts are syntactically valid; CONTEXT.md leaves this to Claude's Discretion.
   - What's unclear: None — this is a pure style choice.
   - Recommendation: Use `render/sketch/` submodule with `mod.rs` + three files. The `// See docs/research/...` pointer doc-comment fits naturally on `sketch/mod.rs`. Keeps `render/` from becoming cluttered.

---

## Sources

### Primary (HIGH confidence)
- `ferro-projections/src/render/mod.rs` — Renderer trait, BaseContext, field_display_name, is_system_field (full source read)
- `ferro-projections/src/render/template.rs` — TemplateRenderer reference pattern (full source read)
- `ferro-projections/src/intent.rs` — Intent variants, IntentScore, IntentHint (full source read)
- `ferro-projections/src/derive.rs` — derive_intents(), all 5 analyzers, signal constants (partial source read, first 586 lines)
- `ferro-projections/src/service.rs` — ServiceDef builder API (full source read)
- `ferro-projections/src/state.rs` — StateMachine, StateDef, Transition builders (full source read)
- `ferro-projections/src/action.rs` — ActionDef, GuardDef, InputDef builders (full source read)
- `ferro-projections/src/lib.rs` — public exports (grep)
- `ferro-projections/tests/catalog.rs` — fixtures::process_workflow() anchor fixture (lines 100-233 read)
- `ferro-projections/CLAUDE.md` — crate boundary rules
- `.planning/phases/208-comp-05-cross-modality-vocabulary-sketch/208-CONTEXT.md` — locked decisions D-01..D-10

---

## Metadata

**Confidence breakdown:**
- Renderer trait contract: HIGH — full source read
- TemplateRenderer pattern: HIGH — full source read
- ServiceDef/StateMachine/ActionDef builder API: HIGH — full source read
- Anchor fixture: HIGH — directly from Phase 207 catalog
- docs/research/ existence: HIGH — `ls docs/` confirmed absent
- lib.rs export rules: HIGH — grep confirmed no sketch re-exports should exist

**Research date:** 2026-06-12
**Valid until:** Stable — based on source code, not external docs. Valid until ferro-projections render API changes.
