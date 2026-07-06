# Phase 208: COMP-05 — Cross-Modality Vocabulary Sketch - Pattern Map

**Mapped:** 2026-06-12
**Files analyzed:** 6 (4 new src files, 1 new doc file, 1 modified src file)
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-projections/src/render/sketch/mod.rs` | module-entry | — | `ferro-projections/src/render/mod.rs` (module registration pattern) | role-match |
| `ferro-projections/src/render/sketch/cli.rs` | renderer (pub(crate)) | transform | `ferro-projections/src/render/template.rs` | exact |
| `ferro-projections/src/render/sketch/voice.rs` | renderer (pub(crate)) | transform | `ferro-projections/src/render/template.rs` | exact |
| `ferro-projections/src/render/sketch/mobile.rs` | renderer (pub(crate)) | transform | `ferro-projections/src/render/template.rs` | exact |
| `ferro-projections/src/render/mod.rs` | module-registration | — | itself (add one line) | exact |
| `docs/research/comp-05-cross-modality-vocabulary-sketch.md` | analysis document | — | `ferro-projections/tests/catalog.rs` file-level docblock (discovered-weaknesses pattern) | partial |

---

## Pattern Assignments

### `ferro-projections/src/render/sketch/mod.rs` (module entry)

**Analog:** `ferro-projections/src/render/mod.rs` (module declaration style, lines 8)

**Module declaration pattern** (`render/mod.rs` line 8):
```rust
pub mod template;
```

The sketch `mod.rs` follows the same structure but uses `pub(crate)` re-exports and carries the module-level doc pointing to the analysis document:

```rust
//! Cross-modality sketch renderers for intent vocabulary validation.
//!
//! # Research sketch — not stable API
//!
//! See `docs/research/comp-05-cross-modality-vocabulary-sketch.md` for the
//! full 7-intent × 3-modality analysis and v14.0 implications.

pub(crate) mod cli;
pub(crate) mod mobile;
pub(crate) mod voice;

pub(crate) use cli::CliSummaryRenderer;
pub(crate) use mobile::MobileCardRenderer;
pub(crate) use voice::VoiceRenderer;
```

---

### `ferro-projections/src/render/sketch/cli.rs` (CliSummaryRenderer, Output = String)

**Analog:** `ferro-projections/src/render/template.rs` (full file — exact structural match)

**Imports pattern** (`template.rs` lines 7–13):
```rust
use serde_json::{json, Map, Value};

use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use super::{is_system_field, BaseContext, Renderer};
```

For `cli.rs`, drop `serde_json` and adjust the `super` path (one level deeper into `sketch/`):
```rust
use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use super::super::{field_display_name, is_system_field, BaseContext, Renderer};
```

**Struct declaration pattern** (`template.rs` line 64):
```rust
pub struct TemplateRenderer;
```

For `cli.rs`:
```rust
// Research sketch — not stable API
pub(crate) struct CliSummaryRenderer;
```

**`impl Renderer` signature** (`template.rs` lines 66–76):
```rust
impl Renderer for TemplateRenderer {
    type Output = serde_json::Value;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        _intents: &[IntentScore],
        _ctx: &BaseContext,
    ) -> Result<Value, Error> {
```

For `cli.rs`, change `Output` type and make `intents`/`ctx` live (used to read primary intent):
```rust
impl Renderer for CliSummaryRenderer {
    type Output = String;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<String, Error> {
```

**Intent consumption pattern** (derived from `render/mod.rs` `BaseContext` doc + RESEARCH.md):
```rust
let primary = intents.get(ctx.intent_index);
let intent_label = primary
    .map(|s| format!("{:?}", s.intent).to_lowercase())
    .unwrap_or_else(|| "unknown".to_string());
```

**Field iteration pattern** (`template.rs` lines 78–90 — the `is_system_field` filter):
```rust
for f in &service.fields {
    if !is_system_field(&f.meaning) {
        // include f in output
    }
}
```

**State machine access pattern** (`template.rs` lines 117–145):
```rust
let state_machine: Option<Value> = service.state_machine.as_ref().map(|sm| {
    // sm.initial_state, sm.states, sm.transitions
});
```

For CLI, the same access pattern produces text lines:
```rust
if let Some(sm) = &service.state_machine {
    // sm.initial_state: &str
    // sm.states: Vec<StateDef> — s.name, s.display_name, s.is_final
    // sm.transitions: Vec<Transition> — t.from, t.event, t.to
}
```

**Test module pattern** (`template.rs` lines 156–291):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionDef, InputDef};
    use crate::derive::derive_intents;
    use crate::field::{DataType, FieldMeaning};
    use crate::service::ServiceDef;
    use crate::state::{StateDef, StateMachine, Transition};

    fn render(svc: &ServiceDef) -> Value {
        let intents = derive_intents(svc);
        let renderer = TemplateRenderer;
        renderer
            .render(svc, &intents, &BaseContext::default())
            .expect("render must succeed")
    }
    // ... test functions ...
}
```

For `cli.rs`, the test module uses the inlined anchor fixture (see Shared Patterns below) and the smoke-test shape from RESEARCH.md:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionDef, GuardDef};
    use crate::derive::derive_intents;
    use crate::field::{DataType, FieldMeaning};
    use crate::service::ServiceDef;
    use crate::state::{StateDef, StateMachine, Transition};

    // inline approval_workflow_fixture() here (see Shared Patterns)

    #[test]
    fn cli_summary_non_trivial_output() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let renderer = CliSummaryRenderer;
        let result = renderer
            .render(&svc, &intents, &BaseContext::default())
            .expect("render must succeed");
        assert!(!result.is_empty(), "output must not be empty");
        assert!(
            result.contains("draft") || result.contains("submitted") || result.contains("approved"),
            "output must mention at least one state name; got: {result}"
        );
    }
}
```

---

### `ferro-projections/src/render/sketch/voice.rs` (VoiceRenderer, Output = String)

**Analog:** `ferro-projections/src/render/template.rs` (same as cli.rs — exact structural mirror)

All import, struct, impl, and test module patterns are identical to `cli.rs` above. The only difference is:

- Struct name: `VoiceRenderer`
- Output content: spoken prose (natural language sentence structure, mentioning action verbs)
- `render()` body: iterate `service.actions` and `service.state_machine` to produce narration

**Action iteration pattern** (`template.rs` lines 93–114):
```rust
let actions: Vec<Value> = service
    .actions
    .iter()
    .map(|a| {
        // a.name, a.display_name (Option<String>), a.inputs
        let display = a.display_name.as_deref().unwrap_or(&a.name);
        // ...
    })
    .collect();
```

For voice, produce strings rather than JSON:
```rust
let action_verbs: Vec<&str> = service
    .actions
    .iter()
    .map(|a| a.display_name.as_deref().unwrap_or(a.name.as_str()))
    .collect();
```

**Smoke test assertion** (from RESEARCH.md code examples):
```rust
#[test]
fn voice_non_trivial_output() {
    let svc = approval_workflow_fixture();
    let intents = derive_intents(&svc);
    let renderer = VoiceRenderer;
    let result = renderer
        .render(&svc, &intents, &BaseContext::default())
        .expect("render must succeed");
    assert!(!result.is_empty(), "output must not be empty");
    assert!(
        result.contains("submit") || result.contains("approve")
            || result.contains("reject") || result.contains("cancel"),
        "voice output must mention at least one action verb; got: {result}"
    );
}
```

---

### `ferro-projections/src/render/sketch/mobile.rs` (MobileCardRenderer, Output = serde_json::Value)

**Analog:** `ferro-projections/src/render/template.rs` (exact structural match — also outputs `serde_json::Value`)

**Imports pattern** (same as `template.rs` lines 7–13, unchanged since `Value` is the same output type):
```rust
use serde_json::{json, Value};

use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use super::super::{is_system_field, BaseContext, Renderer};
```

**`impl Renderer` signature** (mirrors `template.rs` lines 66–76, same `Output = serde_json::Value`):
```rust
impl Renderer for MobileCardRenderer {
    type Output = serde_json::Value;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<Value, Error> {
```

**Output construction pattern** (`template.rs` lines 147–153 — top-level `json!({})` macro):
```rust
Ok(json!({
    "service": service.display_name.as_deref().unwrap_or(&service.name),
    "fields": Value::Object(fields),
    "actions": actions,
    "state_machine": state_machine,
}))
```

For mobile, target a card-list spec with a `"cards"` array key:
```rust
Ok(json!({
    "intent": intent_label,
    "service": service.display_name.as_deref().unwrap_or(&service.name),
    "cards": cards,   // Vec<Value>, must be non-empty per D-10 smoke test
}))
```

**Smoke test assertion** (from RESEARCH.md code examples):
```rust
#[test]
fn mobile_card_non_trivial_output() {
    let svc = approval_workflow_fixture();
    let intents = derive_intents(&svc);
    let renderer = MobileCardRenderer;
    let result = renderer
        .render(&svc, &intents, &BaseContext::default())
        .expect("render must succeed");
    let cards = result
        .get("cards")
        .and_then(|c| c.as_array())
        .expect("output must have a 'cards' array");
    assert!(!cards.is_empty(), "card array must not be empty");
}
```

---

### `ferro-projections/src/render/mod.rs` (modification — add one line)

**Analog:** itself (line 8)

**Existing line** (`render/mod.rs` line 8):
```rust
pub mod template;
```

**Add immediately after:**
```rust
pub(crate) mod sketch; // Research sketch — not stable API
```

No other changes to this file. `lib.rs` must NOT be touched — sketch types must not appear in the public API (`grep -n "sketch\|CliSummary\|VoiceRenderer\|MobileCard" ferro-projections/src/lib.rs` must return empty).

---

### `docs/research/comp-05-cross-modality-vocabulary-sketch.md` (analysis document)

**Analog:** `ferro-projections/tests/catalog.rs` (lines 1–20 — file-level docblock with "Discovered weaknesses" section heading and analytical prose)

The catalog docblock establishes the pattern for naming weaknesses analytically and citing the structural signals involved. The analysis document follows the same neutral, precise voice.

**Mandatory sections per D-09:**

(a) 7-intent × 3-modality coverage matrix:
```markdown
## Intent × Modality Coverage Matrix

| Intent | CLI Summary | Voice | Mobile Card |
|--------|-------------|-------|-------------|
| Browse | ... | ... | ... |
| Focus | ... | ... | ... |
| Collect | ... | ... | ... |
| Process | ... | ... | ... |
| Summarize | ... | ... | ... |
| Analyze | ... | ... | ... |
| Track | ... | ... | ... |
```

(b) Named vocabulary tension (non-empty):
```markdown
## Vocabulary Tensions

### [Tension name]
...
```

(c) v14.0 implications section:
```markdown
## v14.0 Implications

| Open Question | Sketch Evidence | Proposed CHAN-* Scope |
|---------------|-----------------|----------------------|
| Does `BaseContext` need `device_class`? | ... | ... |
| Does `Track` map cleanly to voice? | ... | ... |
```

(d) Discovered weaknesses (non-empty):
```markdown
## Discovered Weaknesses

...
```

---

## Shared Patterns

### Anchor Process Fixture (inline in each sketch test module)

**Source:** `ferro-projections/tests/catalog.rs` lines 119–173 (fixtures::process_workflow())

**Critical constraint:** `tests/catalog.rs` is an integration test module — it CANNOT be imported from `src/`. The fixture MUST be declared inline inside each `#[cfg(test)] mod tests` block (or in a shared `sketch/test_fixtures.rs` module visible only under `#[cfg(test)]`).

**Exact fixture to inline** (adapted from catalog.rs lines 119–173 — the actual source uses no `.display_name()` calls on states, unlike the RESEARCH.md proposed version):
```rust
fn approval_workflow_fixture() -> ServiceDef {
    ServiceDef::new("approval_workflow")
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
                .state(StateDef::new("draft"))
                .state(StateDef::new("submitted"))
                .state(StateDef::new("approved").final_state())
                .state(StateDef::new("rejected").final_state())
                .state(StateDef::new("cancelled").final_state())
                .transition(
                    Transition::new("draft", "submit", "submitted")
                        .guard("has_required_fields"),
                )
                .transition(
                    Transition::new("submitted", "approve", "approved").guard("is_approver"),
                )
                .transition(
                    Transition::new("submitted", "reject", "rejected").guard("is_approver"),
                )
                .transition(
                    Transition::new("draft", "cancel", "cancelled").guard("is_cancellable"),
                )
                .transition(
                    Transition::new("submitted", "cancel", "cancelled").guard("is_cancellable"),
                ),
        )
        .action(
            ActionDef::new("submit")
                .precondition("has_required_fields")
                .transition_trigger("submit"),
        )
        .action(
            ActionDef::new("approve")
                .precondition("is_approver")
                .transition_trigger("approve"),
        )
        .action(
            ActionDef::new("reject")
                .precondition("is_approver")
                .transition_trigger("reject"),
        )
        .action(
            ActionDef::new("cancel")
                .precondition("is_cancellable")
                .transition_trigger("cancel"),
        )
}
```

Note: the actual catalog.rs does NOT call `.display_name()` on `StateDef`. RESEARCH.md's proposed version adds those — the executor should match the real catalog source (no display_name on states) to stay consistent with Phase 207 validated output.

### `// Research sketch — not stable API` Marker

**Required on:** every `pub(crate) struct` in `sketch/cli.rs`, `sketch/voice.rs`, `sketch/mobile.rs`, and the `sketch/mod.rs` module-level doc block.

**Pattern** (place as a line comment immediately above the struct, and in the module doc):
```rust
// Research sketch — not stable API
pub(crate) struct CliSummaryRenderer;
```

SC#1 check: `grep -r "Research sketch" ferro-projections/src/render/sketch/` must return four hits (once per file).

### `super::super::` Import Path for Helpers

**Source:** `ferro-projections/src/render/mod.rs` lines 66–88 (field_display_name, is_system_field)

Since the sketch files live at `src/render/sketch/*.rs`, the path to `render/mod.rs`'s public items is `super::super::`:
```rust
use super::super::{field_display_name, is_system_field, BaseContext, Renderer};
```

`TemplateRenderer` uses `super::` (one level up, since it sits at `render/template.rs`). The sketch files are one level deeper, requiring `super::super::`.

### Test Helper Imports (consistent across all three sketch tests)

**Source:** `template.rs` lines 158–163

```rust
use crate::action::{ActionDef, GuardDef};
use crate::derive::derive_intents;
use crate::field::{DataType, FieldMeaning};
use crate::service::ServiceDef;
use crate::state::{StateDef, StateMachine, Transition};
```

Note: `GuardDef` is needed for the anchor fixture but not used by `TemplateRenderer` tests — it is an addition specific to the Process fixture.

---

## No Analog Found

No files in this phase lack a close analog. All three renderer structs map exactly to `TemplateRenderer`.

---

## Metadata

**Analog search scope:** `ferro-projections/src/render/`, `ferro-projections/tests/`
**Files scanned:** `render/mod.rs`, `render/template.rs`, `tests/catalog.rs` (lines 100–233), `src/intent.rs` (lines 1–60), `src/lib.rs`
**Pattern extraction date:** 2026-06-12
