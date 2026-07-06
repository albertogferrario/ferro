# Phase 216: Conversational-text Renderer (output crate) - Research

**Researched:** 2026-06-13
**Domain:** Internal Rust architecture — new output crate, schema extension, renderer implementation
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**A. Output crate identity**
- D-01: New output crate; NOT in `ferro-projections`. Mirrors `JsonUiRenderer` / `McpRenderer` pattern.
  Recommended name: `ferro-text`, renderer type `TextRenderer`.
- D-02: `impl Renderer for TextRenderer { type Output = String; type Context = BaseContext; }`. Reuse `BaseContext` directly, no `TextContext` wrapper.
- D-03: Re-export from `ferro` facade: `pub use ferro_text::TextRenderer;` (plus `RenderHint` alongside the projections block).
- D-04: Register in `publish.yml` after ferro-projections (Wave 1b), before `framework` (Wave 2). Add to workspace `members` in root `Cargo.toml`.

**B. Per-intent text rendering strategy**
- D-05: One strategy function per intent dispatched on `intents[ctx.intent_index].intent.label()` (never `{:?}`).
  - Browse: entity name + domain fields identifying each item.
  - Collect: "fields to provide" framing — input fields the form gathers.
  - Process: `ctx.current_state` + guard-passing actions ("Currently *submitted*. You can: approve, cancel").
  - Summarize: headline entity + key status/metric fields in a compact sentence.
  - Track: linear state-progression statement ("Currently *shipped*").
- D-06: Output is deterministic plain text. Conversational-leaning. Reuse `field_display_name()` / `is_system_field()`. `Output = String`.
- D-07: Empty-intent input returns `Error::NoIntents` (Phase 215 variant), not `"unknown"`.

**C. Verbosity semantics**
- D-08: `Full` (default) — complete render: fields, state machine context, guard-filtered action list.
  `Brief` — headline only: entity name + intent + guard-passing action verbs (Process/Track) or primary identifying field (Browse/Collect/Summarize).
  Both snapshot-tested over the anchor fixture.

**D. Guard filtering semantics**
- D-09: An action renders unless **any** of its `ActionDef::preconditions` maps to explicit `false` in `ctx.evaluated_guards`. Absent key or `true` → render.
- D-10: Snapshot anchor fixture twice — `evaluated_guards` empty (all four actions render) and `{"is_approver": false}` (approve/reject filtered out).

**E. `FieldDef::render_hint` (CHAN-03)**
- D-11: Add `pub render_hint: Option<RenderHint>` to `FieldDef`. `enum RenderHint { AltText(String), Skip }`. Default `None`. Builder method `FieldDef::with_render_hint(...)` (or `render_hint(...)`) consistent with existing fluent builders. Derive set mirrors `FieldDef`'s existing derives (Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema).
- D-12: Renderer behavior: `Some(AltText(s))` → render alt text. `Some(Skip)` → omit field. `None` on `ImageUrl`/`Url` → label form with "(link)"/"(image)" marker.

**F. Focus / Analyze fallback**
- D-13: Focus — render fields applying D-12 rules + one-line note: media/navigational view with limited text representation.
  Analyze — render entity + field set + one-line note: time-series/trend output has no full text form. No fabricated statistics. Both fallbacks snapshot-tested.

**G. Snapshot tooling + anchor fixture**
- D-14: Use `insta` for snapshots (already a workspace dev-dep). Plain `assert_eq!` golden strings acceptable fallback.
- D-15: Copy `approval_workflow` anchor fixture from `ferro-projections/src/render/sketch/cli.rs` test module into the new crate's test module. Also construct minimal Browse/Collect/Summarize/Track/Focus/Analyze fixtures.

### Claude's Discretion
- Exact crate name (`ferro-text` recommended) and renderer type name (`TextRenderer`).
- Exact `RenderHint` derive set and whether it is serde/JsonSchema (match `FieldDef`).
- Whether to use `insta` snapshots vs inline golden strings (D-14).
- Precise conversational wording per intent.
- Whether COMP-05 `pub(crate)` sketch renderers stay as-is or `cli.rs` is removed as superseded.

### Deferred Ideas (OUT OF SCOPE)
- Voice renderer, structured-API renderer, mobile `device_class` / chart-card type.
- Inbound intent classification via `ferro-ai`.
- `ServiceDef::summary_hint` for Analyze voice narration.
- Reshaping the seven intents (frozen; CHAN-05 is a future research outcome).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CHAN-03 | `FieldDef` carries a `render_hint` (`AltText(String)` / `Skip`) so a renderer handles `ImageUrl`/`Url` fields without emitting a useless raw-URL label; absent hint preserves current behavior. | Section 4 (render_hint placement) and Section 8 (derives) confirm exactly where and how to add the field. |
| CHAN-04 | A production conversational-text `Renderer` in its own output crate projects a `ServiceDef` to text for Browse/Collect/Process/Summarize/Track, guard-filtered and verbosity-aware, with a defined tested fallback for Focus/Analyze. Re-exported via `ferro` facade; deterministic snapshot/string tests over the COMP-05 anchor fixture. | Sections 1–9 collectively specify every integration seam, crate boundary, and test shape. |
</phase_requirements>

---

## Summary

Phase 216 is a pure internal-Rust build phase: create a new output crate `ferro-text` and ship the first production non-visual `Renderer`. The surface it consumes was purpose-built in Phase 215 — `BaseContext` now carries `evaluated_guards`, `verbosity`, `intent_index`, and `current_state`; `Intent::label()` replaces the fragile `{:?}` pattern; `Error::NoIntents` replaces silent fallbacks. The renderer can be written directly against these verified Phase 215 contracts.

The only edit inside `ferro-projections` is additive: `FieldDef` gains an `Option<RenderHint>` field with `#[serde(default, skip_serializing_if = "Option::is_none")]`. Every existing `FieldDef` literal in the codebase (eleven sites in ferro-projections, one in ferro-json-ui) uses struct literal syntax that specifies all current fields by name, not `..` spread — adding an `Option` with `#[serde(default)]` does NOT break struct literal construction because Rust's struct literal syntax requires listing every field. However, since `FieldDef` does NOT use `Default`, those existing literals must add `render_hint: None` or the build fails. This is the one migration cost inside ferro-projections.

The new crate `ferro-text` is the third instance of the renderer-per-output-crate pattern (after `ferro-json-ui` for `JsonUiRenderer` and `ferro-mcp-server` for `McpRenderer`). It depends only on `ferro-projections` plus standard crates. No codec, no assembler, no native dependency risk.

**Primary recommendation:** Ship the `FieldDef::render_hint` extension in ferro-projections first (propagate `render_hint: None` to every existing struct literal), then scaffold `ferro-text` following the `ferro-json-ui` `Cargo.toml` template, implement the five cleanly-mapping intents, snapshot-test the anchor fixture with and without `is_approver: false`, and wire the facade re-export. The quality bar for this phase is the per-intent text quality (D-05/D-06), not the crate plumbing.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `RenderHint` enum + `render_hint` field | `ferro-projections` | — | Schema extension belongs in the schema crate; all renderers then read it. |
| Text rendering logic (all intents) | `ferro-text` (new output crate) | — | Rendering architecture rule: renderers live in output crates, not ferro-projections. |
| `BaseContext` / `Verbosity` / `Error::NoIntents` | `ferro-projections` | — | Already shipped in Phase 215; consumed here without modification. |
| `Intent::label()` | `ferro-projections` | — | Already shipped in Phase 215; consumed here without modification. |
| Facade re-export | `framework` | — | Existing pattern at `framework/src/lib.rs:265`. |
| Publish wave ordering | `.github/workflows/publish.yml` | — | New crate goes between Wave 1b (ferro-projections) and Wave 2 (framework). |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-projections` | workspace (0.2.56) | `Renderer` trait, `BaseContext`, `ServiceDef`, `FieldDef`, `Intent`, `Error` | The entire type surface this crate consumes. |
| `thiserror` | 1.0 | Error derive (if the crate introduces its own error type — optional) | Workspace convention for error types. |
| `serde` | 1.0 | Required because `RenderHint` must derive `Serialize`/`Deserialize` (see §4). | Workspace convention; `FieldDef` already derives serde. |
| `schemars` | 1 | `RenderHint` must derive `JsonSchema` (see §4). | `FieldDef` derives `JsonSchema`; `RenderHint` must too or the derived impl fails. |

### Dev-dependencies

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `insta` | 1 (features = ["yaml"]) | Snapshot testing | Already a workspace dev-dep in ferro-projections; add to `[dev-dependencies]` in ferro-text. |

**Installation for new crate `ferro-text/Cargo.toml`:**

```toml
[package]
name = "ferro-text"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Conversational-text renderer for Ferro service projections"
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://ferro-rs.dev"
readme = "README.md"
keywords = ["text", "renderer", "projections", "ferro"]
categories = ["web-programming", "web-programming::http-server"]

[dependencies]
ferro-projections = { path = "../ferro-projections", version = "0.2" }

[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
```

Note: `serde` / `schemars` are not direct dependencies of `ferro-text` — `RenderHint` is defined in `ferro-projections` (not in this crate). `ferro-text` only consumes the type through its `ferro-projections` dependency.

---

## Architecture Patterns

### System Architecture Diagram

```
ServiceDef + IntentScore[] + BaseContext
         |
         v
   TextRenderer::render()          (ferro-text/src/lib.rs)
         |
    intents[ctx.intent_index].intent.label()
         |
    match intent {
      "browse"    --> browse_text(service, fields, verbosity)
      "collect"   --> collect_text(service, fields, verbosity)
      "process"   --> process_text(service, ctx, verbosity)
                         |
                    guard_filter(actions, evaluated_guards)
      "summarize" --> summarize_text(service, fields, verbosity)
      "track"     --> track_text(service, ctx, verbosity)
      "focus"     --> focus_fallback(service, fields)    [render_hint applied]
      "analyze"   --> analyze_fallback(service, fields)
      _           --> same dispatch (Custom intents = best-effort)
    }
         |
         v
   Result<String, Error::NoIntents | Error::Render>
         |
         v
   ferro facade (framework/src/lib.rs)
   pub use ferro_text::TextRenderer;
```

### Recommended Project Structure

```
ferro-text/
├── Cargo.toml
└── src/
    └── lib.rs      # TextRenderer impl + per-intent strategy fns + guard_filter helper
```

A single `lib.rs` is sufficient: the renderer is one function plus helpers. Split into submodules only if the file exceeds ~400 lines.

### Pattern 1: `Renderer` trait implementation

```rust
// Source: ferro-projections/src/render/mod.rs (verified)
use ferro_projections::{
    render::{BaseContext, Renderer, field_display_name, is_system_field},
    Error, IntentScore, ServiceDef,
};

pub struct TextRenderer;

impl Renderer for TextRenderer {
    type Output = String;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<String, Error> {
        let score = intents.get(ctx.intent_index).ok_or(Error::NoIntents)?;
        match score.intent.label() {
            "browse"    => Ok(render_browse(service, ctx)),
            "collect"   => Ok(render_collect(service, ctx)),
            "process"   => Ok(render_process(service, ctx)),
            "summarize" => Ok(render_summarize(service, ctx)),
            "track"     => Ok(render_track(service, ctx)),
            "focus"     => Ok(render_focus(service, ctx)),
            "analyze"   => Ok(render_analyze(service, ctx)),
            _           => Ok(render_browse(service, ctx)), // Custom: best-effort
        }
    }
}
```

### Pattern 2: Guard filtering helper

```rust
// Source: verified from action.rs:34 (ActionDef::preconditions: Vec<String>)
// and render/mod.rs:46 (evaluated_guards: HashMap<String, bool>)
fn action_passes_guards(
    action: &ferro_projections::ActionDef,
    evaluated_guards: &std::collections::HashMap<String, bool>,
) -> bool {
    // D-09: absent key = render; only explicit false filters
    action.preconditions.iter().all(|guard_name| {
        evaluated_guards.get(guard_name.as_str()).copied().unwrap_or(true)
    })
}
```

### Pattern 3: `RenderHint` extension in `ferro-projections/src/field.rs`

CHAN-03 requires adding to `field.rs` at approximately line 57 (before `FieldDef`):

```rust
// Source: verified FieldDef derives at field.rs:59 — matches serde/JsonSchema pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RenderHint {
    /// Substitute this string in place of the raw URL/ImageUrl value.
    AltText(String),
    /// Omit this field entirely from non-visual output.
    Skip,
}
```

And in `FieldDef` (line 59):

```rust
pub struct FieldDef {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub is_list: bool,
    #[serde(default = "default_true")]
    pub readable: bool,
    #[serde(default = "default_true")]
    pub writable: bool,
    /// Non-visual rendering hint for URL/ImageUrl fields.
    /// `None` preserves current behavior: render a "(link)"/"(image)" label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_hint: Option<RenderHint>,
}
```

Builder method (consuming, matches existing `ActionDef`/`GuardDef` builder style):

```rust
impl FieldDef {
    pub fn with_render_hint(mut self, hint: RenderHint) -> Self {
        self.render_hint = Some(hint);
        self
    }
}
```

Note: the current `FieldDef` has no constructor — it is always constructed via struct literal or through `ServiceDef` builder methods (`field()`, `optional_field()`, etc.). Adding `render_hint: Option<RenderHint>` breaks all 12 existing struct-literal sites. Each must add `render_hint: None`. The planner MUST include a task to migrate these literals.

### Pattern 4: `render_hint` application for `ImageUrl`/`Url` fields

```rust
// Per D-12 — applied inside focus_fallback and as a general field-render helper
fn render_field_value(f: &ferro_projections::FieldDef) -> Option<String> {
    use ferro_projections::{FieldMeaning, render::RenderHint};
    match &f.render_hint {
        Some(RenderHint::Skip) => None,
        Some(RenderHint::AltText(s)) => Some(s.clone()),
        None => {
            // D-12: ImageUrl/Url without hint → useful label form
            let label = field_display_name(&f.name);
            match &f.meaning {
                FieldMeaning::ImageUrl => Some(format!("{label} (image)")),
                FieldMeaning::Url => Some(format!("{label} (link)")),
                _ => Some(label),
            }
        }
    }
}
```

### Anti-Patterns to Avoid

- **Using `{:?}` on `Intent`:** COMP-05 weakness #2 documents this as fragile. Always use `.label()`.
- **Adding a renderer to `ferro-projections`:** The crate-boundary rule (CLAUDE.md, ferro-projections/CLAUDE.md) prohibits this. The new crate is the fix.
- **Fabricating statistics in Analyze fallback:** `ServiceDef` has no computed values; D-13 explicitly prohibits this.
- **Panicking on empty intents:** D-07 requires returning `Error::NoIntents`.
- **Silently rendering sensitive fields:** Consider calling `is_system_field()` to drop `Identifier`/`CreatedAt`/`UpdatedAt` (matches sketch behavior).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Intent label strings | Custom `match intent { Intent::Browse => ... }` | `intent.label()` from Phase 215 | Already implemented; `{:?}` is fragile (COMP-05 weakness #2) |
| Field display names | `to_string()` on field name | `field_display_name(&f.name)` from `ferro-projections::render` | Handles snake_case → Title Case correctly |
| System field detection | Manual `match` on `FieldMeaning` | `is_system_field(&f.meaning)` from `ferro-projections::render` | Handles Identifier/CreatedAt/UpdatedAt in one place |
| Empty-intent error | `"unknown"` fallback | `Error::NoIntents` from `ferro-projections::error` | Phase 215 variant; typed and tested |
| Snapshot tooling | Custom golden file I/O | `insta` (already a workspace dev-dep in ferro-projections) | Zero new tooling; `.snap` files integrate with `cargo test` |

**Key insight:** The entire rendering support surface (`BaseContext`, `Verbosity`, `Intent::label()`, `Error::NoIntents`, `field_display_name()`, `is_system_field()`) was purpose-built in Phase 215 to be consumed by exactly this renderer. No reinvention needed.

---

## Findings by Research Area

### 1. `Renderer` Trait and `BaseContext` (verified from `ferro-projections/src/render/mod.rs`)

**`Verbosity` enum** (line 24):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    #[default]
    Full,
    Brief,
}
```
Default is `Full`. Derives `Copy` — can be passed by value.

**`BaseContext` struct** (line 37):
```rust
#[derive(Debug, Clone, Default)]
pub struct BaseContext {
    pub intent_index: usize,           // Which intent to render (0 = primary)
    pub current_state: Option<String>, // Current workflow state name
    pub evaluated_guards: HashMap<String, bool>, // guard_name -> bool
    pub verbosity: Verbosity,          // Full (default) or Brief
}
```
Default: `intent_index=0`, `current_state=None`, `evaluated_guards={}`, `verbosity=Full`. No serde derives — context is not serialized.

**`Renderer` trait** (line 58):
```rust
pub trait Renderer: Send + Sync {
    type Output;
    type Context: Default;
    fn render(&self, service: &ServiceDef, intents: &[IntentScore], ctx: &Self::Context) -> Result<Self::Output, Error>;
}
```

**Reusable helpers** (lines 91, 109):
- `pub fn field_display_name(name: &str) -> String` — snake_case → Title Case
- `pub fn is_system_field(meaning: &FieldMeaning) -> bool` — matches `Identifier | CreatedAt | UpdatedAt`

Both are `pub` and importable as `ferro_projections::render::{field_display_name, is_system_field}`.

### 2. COMP-05 Sketch Renderer (`ferro-projections/src/render/sketch/cli.rs`)

`CliSummaryRenderer` is `pub(crate)` — cannot be used from the new crate, only studied. Key behaviors:

- Uses `format!("{:?}", s.intent).to_lowercase()` for intent label — fragile, D-05/D-06 replaces this with `.label()`.
- Calls `is_system_field()` to drop infrastructure fields.
- Calls `field_display_name()` for label display.
- Lists ALL actions unconditionally — no guard filtering. This is the gap the new renderer fixes.
- Output format is a debug dump, not conversational text — the new renderer differentiates itself here.

**`approval_workflow_fixture()` verbatim** (lines 75–129 of `cli.rs`): The canonical anchor fixture — a `ServiceDef` named `"approval_workflow"` with fields `id` (Identifier), `title` (EntityName), `status` (Status), `amount` (Money); guards `has_required_fields`, `is_approver`, `is_cancellable`; state machine `approval_lifecycle` with initial `draft`, states `draft`/`submitted`/`approved`(final)/`rejected`(final)/`cancelled`(final); 5 transitions (submit guarded by `has_required_fields`, approve/reject by `is_approver`, cancel by `is_cancellable` from both `draft` and `submitted`); 4 actions (`submit` precond `has_required_fields`, `approve`/`reject` precond `is_approver`, `cancel` precond `is_cancellable`). `derive_intents()` resolves to `Process` as primary intent.

D-15 requires copying this fixture into `ferro-text`'s test module verbatim (not as a public dependency on the sketch).

### 3. `ServiceDef` / `FieldDef` / `FieldMeaning` / `ActionDef` / `GuardDef` / `Intent` Shapes

**`ServiceDef`** (`service.rs:63`): `name: String`, `display_name: Option<String>`, `description: Option<String>`, `fields: Vec<FieldDef>`, `actions: Vec<ActionDef>`, `guards: Vec<GuardDef>`, `relationships: Vec<RelationshipDef>`, `intent_hints: Vec<IntentHint>`, `state_machine: Option<StateMachine>`, plus MCP metadata fields. Derives `Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema`.

Accessors the text renderer uses:
- `service.name` / `service.display_name` — entity identifier/label
- `service.fields` — `Vec<FieldDef>` iterated for per-intent field sections
- `service.actions` — `Vec<ActionDef>` filtered by guards for Process/Collect output
- `service.state_machine` — `Option<StateMachine>` for Process/Track state context

**`FieldDef`** (`field.rs:59`): `name: String`, `data_type: DataType`, `meaning: FieldMeaning`, `required: bool` (default true), `is_list: bool` (default false), `readable: bool` (default true), `writable: bool` (default true). Derives `Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema`.

**`FieldMeaning`** (`field.rs:35`): enum with 18 known variants + `Custom(String)`. Derives `Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema`, `#[serde(rename_all = "snake_case")]`. The Focus-relevant variants are `Url` and `ImageUrl`. System variants are `Identifier`, `CreatedAt`, `UpdatedAt`.

**`ActionDef`** (`action.rs:25`): `name: String`, `display_name: Option<String>`, `description: Option<String>`, `inputs: Vec<InputDef>`, `preconditions: Vec<String>`, `effects: Vec<String>`, `transition_trigger: Option<String>`. Builder pattern with consuming `mut self -> Self` methods. The guard-filtering key is `preconditions: Vec<String>` — each element is a guard name string.

**`GuardDef`** (`action.rs:148`): `name: String`, `display_name: Option<String>`, `description: Option<String>`. The `name` field is the key used in `evaluated_guards` and `ActionDef::preconditions`.

**`Intent`** (`intent.rs:18`): enum `Browse | Focus | Collect | Process | Summarize | Analyze | Track | Custom(String)`. Derives `Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema`, `#[serde(rename_all = "snake_case")]`. `.label()` method added in Phase 215 returns `&str`.

**`StateMachine`** (`state.rs:24`): `name: String`, `display_name: Option<String>`, `description: Option<String>`, `initial_state: String`, `states: Vec<StateDef>`, `transitions: Vec<Transition>`. Accessor `events_from_state(state: &str) -> Vec<&Transition>` returns outgoing transitions for a state — directly useful for Process rendering (what actions are available from `ctx.current_state`).

**`StateDef`** (`state.rs:46`): `name: String`, `display_name: Option<String>`, `is_final: bool`, `on_enter/on_exit: Vec<String>`, `metadata: Option<Value>`. For Track: `is_final` determines terminal state display.

**`Transition`** (`state.rs:73`): `from: String`, `event: String`, `to: String`, `guard: Option<String>`, `actions: Vec<String>`, `description: Option<String>`.

### 4. `render_hint` Placement (CHAN-03)

**Current `FieldDef` derives** (`field.rs:59`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct FieldDef { ... }
```

`RenderHint` must derive the same set: `Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema` plus `#[serde(rename_all = "snake_case")]` (workspace convention for serde enums).

**Exhaustive struct literal sites requiring migration** (adding `render_hint: None`):

In `ferro-projections/src/field.rs` (test module, 4 sites): lines 295, 440, 458, 476.
In `ferro-projections/src/service.rs` (builder methods + from_model, 6 sites): lines 153, 172, 191, 212, 233, 315.
In `ferro-json-ui/src/projection/builder.rs` (test, 1 site): line 1066.

Total: **11 struct literal sites** must add `render_hint: None`. This is the only breaking change inside the existing codebase.

The `ServiceDef` builder methods (`field()`, `optional_field()`, etc.) construct `FieldDef` with all fields named — they will fail to compile until `render_hint: None` is added. Same for the `from_model()` push site.

**Serde behavior with `#[serde(default, skip_serializing_if = "Option::is_none")]`:**
- Old JSON without `render_hint` key → deserializes to `render_hint: None` (backward compatible).
- New JSON with `render_hint: {"alt_text": "My image"}` → deserializes to `Some(RenderHint::AltText("My image"))`.
- `None` is not serialized (clean JSON output).

**Builder method choice:** Since `FieldDef` has no existing builder `new()` pattern (it is always built via struct literal or `ServiceDef` methods), the builder method should be added as an `impl FieldDef` method matching `ActionDef`'s consuming style: `pub fn with_render_hint(mut self, hint: RenderHint) -> Self`. Alternatively a plain field setter. The planner can choose; `with_render_hint` is consistent with `ActionDef::with_*` style and won't collide with the field name.

### 5. New Crate Scaffolding

**Model: `ferro-json-ui/Cargo.toml`** (verified):
```toml
[package]
name = "ferro-json-ui"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "JSON-based server-driven UI schema types for Ferro"
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://ferro-rs.dev"
readme = "README.md"
keywords = ["json-ui", "sdui", "server-driven-ui", "ferro"]
categories = ["web-programming", "web-programming::http-server"]
```

`ferro-text` mirrors this template, with appropriate description/keywords.

**Root `Cargo.toml` members** (line 3 area): The current `members` array ends with `"ferro-assets"` (line 35). Add `"ferro-text"` as a new entry. Current workspace version: `0.2.56`.

**`ferro-projections` dependency line** (for `ferro-text/Cargo.toml`):
```toml
ferro-projections = { path = "../ferro-projections", version = "0.2" }
```

Note: `ferro-json-ui` has `ferro-projections` as an **optional** dependency (under `features = ["projections"]`). For `ferro-text`, it is a **required** dependency — the renderer is the whole point of the crate.

**Wave ordering flag:** `ferro-json-ui` is listed in Wave 1a (`WAVE1A_CRATES`) but it depends on `ferro-projections` (Wave 1b) as an optional feature. The optional dep avoids the ordering problem because the base crate has no hard dependency. The Wave 1a listing is acceptable for the base crate; the `projections` feature is only exercised by `ferro-rs` in Wave 2. However, `ferro-text` has `ferro-projections` as a **required** (non-optional) dependency, so it cannot go in Wave 1a — it must go in Wave 1b alongside `ferro-projections`, or in a position after Wave 1b completes. The planner should add `ferro-text` to `WAVE1B_CRATES` (line 246) or create a wait-then-publish step between 1b and Wave 2.

### 6. Facade Re-export

**Current re-export block** (`framework/src/lib.rs` lines 256–265):
```rust
// Re-export ferro-projections for service projection definitions
#[cfg(feature = "projections")]
pub use ferro_projections::{
    derive_intents, infer_meaning, ActionDef, Cardinality, DataType, Error as ProjectionsError,
    FieldDef, FieldMeaning, GuardDef, InputDef, Intent, IntentHint, IntentScore, NavigationHint,
    RelationshipDef, Renderer, ServiceDef, StateDef, StateMachine, Transition,
    Warning as ProjectionsWarning,
};
// Re-export visual renderer types from ferro-json-ui
#[cfg(feature = "projections")]
pub use ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext};
```

**What to add (D-03):**
1. Add `RenderHint` to the `ferro_projections` re-export line (since it is defined in `ferro-projections`).
2. Add a new re-export block for `ferro-text`:
```rust
#[cfg(feature = "projections")]
pub use ferro_text::TextRenderer;
```

**`framework/Cargo.toml` additions:**
- Add `ferro-text = { path = "../ferro-text", version = "0.2", optional = true }` alongside the `ferro-json-ui` optional dep.
- The existing `projections` feature already pulls `ferro-json-ui/projections`; extend it to pull `ferro-text`:
  ```toml
  projections = ["dep:ferro-projections", "dep:ferro-json-ui", "ferro-json-ui/projections", "dep:ferro-text"]
  ```

Note: `framework/src/lib.rs` uses `#[cfg(feature = "projections")]` gates. `TextRenderer` should be gated behind the same feature for consistency with `JsonUiRenderer`.

### 7. Publish.yml Waves

**Current wave assignments** (verified from `.github/workflows/publish.yml`):

- Wave 1a (`WAVE1A_CRATES`, line 211): `ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet ferro-orm ferro-audit ferro-migration ferro-assets`. Notably includes `ferro-json-ui` even though it has an optional dep on `ferro-projections`.
- Wave 1b (`WAVE1B_CRATES`, line 246): `ferro-projections ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments`. This is where `ferro-projections` lives.
- Wave 2 (`WAVE2_CRATES`, line 274): `ferro-rs ferro-mcp ferro-mcp-server ferro-mcp-oauth`. This is `framework` and the MCP crates.

**Wave ordering concern:** `ferro-json-ui` in Wave 1a has `ferro-projections` as an optional dep. CI publishes with `--no-verify`, so the optional dep doesn't trigger a dependency-ordering failure. This is a pre-existing ordering shortcut in the pipeline — Wave 1a technically shouldn't include crates with optional deps on Wave 1b crates, but it works in practice because `--no-verify` skips the crates.io index check. Flag to planner: this is a pre-existing minor ordering issue, NOT Phase 216's job to fix.

**Recommended placement for `ferro-text`:** Add to `WAVE1B_CRATES` (line 246), after `ferro-projections`. The new line becomes:
```
WAVE1B_CRATES="ferro-projections ferro-text ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments"
```

`ferro-text` depends only on `ferro-projections` (Wave 1b), so this is correct ordering. It publishes after `ferro-projections` within the same wave iteration (the loop processes crates sequentially).

### 8. Insta Snapshot Testing

**`insta` is already a dev-dependency in `ferro-projections/Cargo.toml`** (line 20):
```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
proptest = "1"
```

**Existing usage:** `ferro-projections/tests/catalog.rs` uses `insta::assert_yaml_snapshot!()` for 7 intent-score snapshots. Snapshot files are in `ferro-projections/tests/snapshots/`. This is the established pattern.

**For `ferro-text`:** Add `insta` to its own `[dev-dependencies]`:
```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
```

**Snapshot approach per D-14:** `insta` is preferred. For text renderer output, `insta::assert_snapshot!()` (plain text, no YAML) is more appropriate than `assert_yaml_snapshot!` since the output is a `String`, not a structured value. Snapshots go to `ferro-text/src/snapshots/` (insta convention for inline tests) or `ferro-text/tests/snapshots/` (for integration test files). The planner should prefer inline tests in `lib.rs` or a `tests/` file.

**Inline golden string fallback (D-14):** If the planner prefers zero snapshot management, `assert_eq!(result, "expected text\n...")` in the test body is equivalent and avoids external `.snap` files. Given that the test output is deterministic plain text (short strings), inline golden strings are an acceptable and simpler approach for this phase.

**Required test matrix for anchor fixture:**

```rust
// Test 1: approval_workflow, empty guards (all 4 actions render)
let ctx = BaseContext { evaluated_guards: HashMap::new(), ..Default::default() };
let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
// assert_snapshot! or assert_eq! — pins all 4 actions visible

// Test 2: approval_workflow, is_approver: false (approve/reject filtered)
let ctx = BaseContext {
    evaluated_guards: [("is_approver".to_string(), false)].into(),
    ..Default::default()
};
let result = TextRenderer.render(&svc, &intents, &ctx).unwrap();
// assert! result does NOT contain "approve" and "reject" action verbs
// assert! result DOES contain "submit" and "cancel"

// Test 3: Brief verbosity
let ctx = BaseContext { verbosity: Verbosity::Brief, ..Default::default() };
// assert! brief output is shorter / contains only headline

// Test 4: empty intents → Error::NoIntents
let result = TextRenderer.render(&svc, &[], &BaseContext::default());
assert!(matches!(result, Err(Error::NoIntents)));
```

### 9. Per-Intent Text Strategy (ServiceDef accessors per intent)

For each intent below, the planner writes one `render_*` function consuming `&ServiceDef` and `&BaseContext`.

**Browse:** Iterate `service.fields`, drop `is_system_field()`, render domain fields (EntityName, Status, Money, ForeignKey relationships). `Brief`: entity display name + primary EntityName field only. `Full`: entity name + all domain fields as a label list.

Relevant accessors: `service.display_name / service.name`, `service.fields` filtered by `!is_system_field(&f.meaning)`.

**Collect:** Iterate `service.fields`, show writable fields that are domain fields (`f.writable && !is_system_field(&f.meaning)`), express as "fields to fill in". Required/optional distinction is available from `f.required`. `Brief`: count of fields ("3 fields to fill in"). `Full`: label list with required markers.

Relevant accessors: `service.fields` (filter `f.writable`), `f.required`.

**Process:** Access `ctx.current_state` for current position. If `None`, use `service.state_machine.as_ref().map(|sm| sm.initial_state.as_str())` as fallback. Filter `service.actions` by guard pass. Use `StateMachine::events_from_state(current)` to cross-reference which actions correspond to outgoing transitions from the current state (optional refinement — actions are also filtered by guards). `Brief`: "Currently *{state}*. You can: {verb1}, {verb2}." `Full`: state description + field listing + full guard-passing action list with descriptions.

Relevant accessors: `ctx.current_state`, `service.state_machine`, `sm.events_from_state()`, `service.actions`, `action.preconditions`, `ctx.evaluated_guards`, `action.display_name / action.name`.

**Summarize:** Iterate `service.fields` for Money/Percentage/Quantity/Status fields (the headline metrics). `Brief`: entity name + first metric field. `Full`: entity name + sentence enumerating key metric fields.

Relevant accessors: `service.display_name / service.name`, `service.fields` filtered by metric meanings (`Money | Percentage | Quantity | Status`).

**Track:** Access `ctx.current_state` (or `sm.initial_state` as fallback). Express the current position in the lifecycle. `is_final` on the current `StateDef` determines if "completed" language should apply. `Brief`: "Currently *{state}*." `Full`: state + whether it is a terminal state + what transitions are possible (unguarded — Track has unguarded linear progression by definition).

Relevant accessors: `ctx.current_state`, `service.state_machine`, `sm.states` (to find `StateDef.is_final`), `sm.events_from_state()`.

**Focus (fallback, D-13):** Render all domain fields applying `render_hint` logic (D-12). Append one-line note: "This is a media/navigational view; full text representation is limited." `render_hint` on `ImageUrl`/`Url` fields: `AltText(s)` → use `s`; `Skip` → omit; `None` → `field_display_name + " (link)"/"(image)"`.

Relevant accessors: `service.fields`, `f.meaning`, `f.render_hint`.

**Analyze (fallback, D-13):** Render entity name + domain field set (names only, since no computed values exist in `ServiceDef`). Append one-line note: "Time-series and trend data has no full text representation in this channel." No fabricated statistics.

Relevant accessors: `service.display_name / service.name`, `service.fields` filtered by `!is_system_field()`.

---

## Common Pitfalls

### Pitfall 1: Struct literal breakage from `render_hint` field addition

**What goes wrong:** Adding `render_hint: Option<RenderHint>` to `FieldDef` causes compile errors at all 11 existing `FieldDef { ... }` struct literal sites because Rust struct literals must specify every field.

**Why it happens:** Unlike `Default` (which would allow `..Default::default()`), `FieldDef` has no `Default` impl. All construction uses explicit struct literals.

**How to avoid:** First migration task in the plan: grep for `FieldDef {`, add `render_hint: None` to each site. Then add the field to `FieldDef`. Alternatively add the field first and let compiler errors guide the migration.

**Warning signs:** `error[E0063]: missing field 'render_hint' in initializer` across multiple files.

### Pitfall 2: Incorrect guard filtering (false negative or false positive)

**What goes wrong:** An action that should render is filtered out (false positive), or a forbidden action renders (false negative).

**Why it happens:** Inverting the guard logic — treating absent keys as `false` instead of `true`.

**How to avoid:** D-09 is explicit: `evaluated_guards.get(guard_name).copied().unwrap_or(true)`. Absent = render. Only explicit `false` filters. The D-10 test pins both the empty-map (all render) and `is_approver: false` (approve/reject hidden) cases.

**Warning signs:** D-10 snapshot test failures; actions missing or appearing unexpectedly.

### Pitfall 3: Using `{:?}` for intent label in the new renderer

**What goes wrong:** `format!("{:?}", score.intent).to_lowercase()` is copied from the sketch renderer and breaks if the enum gets a custom `Debug` impl or a variant is renamed.

**Why it happens:** Copy-paste from `cli.rs` sketch without noticing weakness #2 from COMP-05.

**How to avoid:** Always use `score.intent.label()`. Tests for all 7 intents verify the label strings.

### Pitfall 4: Registering `ferro-text` in Wave 1a instead of Wave 1b

**What goes wrong:** `ferro-text` publishes before `ferro-projections` is available on crates.io, causing a dependency resolution failure.

**Why it happens:** Copying the `ferro-json-ui` entry (which is in Wave 1a) without checking that `ferro-json-ui`'s `ferro-projections` dep is optional (hence no hard ordering requirement at publish time).

**How to avoid:** `ferro-text` has `ferro-projections` as a required dep — it goes in Wave 1b, after the `sleep 30` wait for Wave 1a crates.io indexing.

### Pitfall 5: Missing facade feature gate for `ferro-text`

**What goes wrong:** `pub use ferro_text::TextRenderer;` compiles unconditionally, pulling `ferro-text` into every `ferro-rs` consumer even if they don't use projections.

**Why it happens:** Forgetting that `JsonUiRenderer` is gated behind `#[cfg(feature = "projections")]`.

**How to avoid:** Add `ferro-text` to `framework/Cargo.toml` as an optional dep and gate the re-export with `#[cfg(feature = "projections")]`, matching the existing pattern at lines 263–265 of `framework/src/lib.rs`.

---

## Code Examples

### Anchor fixture (verbatim from `ferro-projections/src/render/sketch/cli.rs:75–129`)

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
                .transition(Transition::new("draft", "submit", "submitted")
                    .guard("has_required_fields"))
                .transition(Transition::new("submitted", "approve", "approved")
                    .guard("is_approver"))
                .transition(Transition::new("submitted", "reject", "rejected")
                    .guard("is_approver"))
                .transition(Transition::new("draft", "cancel", "cancelled")
                    .guard("is_cancellable"))
                .transition(Transition::new("submitted", "cancel", "cancelled")
                    .guard("is_cancellable")),
        )
        .action(ActionDef::new("submit")
            .precondition("has_required_fields")
            .transition_trigger("submit"))
        .action(ActionDef::new("approve")
            .precondition("is_approver")
            .transition_trigger("approve"))
        .action(ActionDef::new("reject")
            .precondition("is_approver")
            .transition_trigger("reject"))
        .action(ActionDef::new("cancel")
            .precondition("is_cancellable")
            .transition_trigger("cancel"))
}
```

Note: After `render_hint` is added to `FieldDef`, the `.field()` builder method constructs `FieldDef` internally (in `ServiceDef::field()`). The builder method site is `service.rs:153` where `FieldDef { ... }` is constructed — that site must add `render_hint: None`. The fixture itself (using `.field()` builder) is unaffected.

### Insta snapshot for Process (approximate expected output, Full verbosity, no guard filter)

```
approval_workflow — process

Currently: draft (initial)
Fields: Title, Status, Amount
Available actions: submit, approve, reject, cancel
```

With `is_approver: false`:

```
approval_workflow — process

Currently: draft (initial)
Fields: Title, Status, Amount
Available actions: submit, cancel
```

The exact wording is Claude's discretion (D-06) — the snapshot pins whatever the implementation produces, not a pre-specified string.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `format!("{:?}", intent).to_lowercase()` for intent label | `intent.label()` method | Phase 215 (D-06) | Decouples label from Debug derive — stable across enum refactors |
| Silent `"unknown"` fallback for empty intents | `Error::NoIntents` typed error | Phase 215 (D-08) | Typed, testable, cross-renderer |
| All guards in `BaseContext` absent | `evaluated_guards: HashMap<String,bool>` | Phase 215 (D-03/D-04) | Enables guard-filtered rendering |
| `VisualContext` duplicated `intent_index`/`current_state` from `BaseContext` | `VisualContext { base: BaseContext, ... }` embedding | Phase 215 (D-02) | Single source of truth; `BaseContext` is the universal renderer context |

**Deprecated:**
- The COMP-05 sketch renderers (`cli.rs`, `voice.rs`, `mobile.rs`) remain `pub(crate)` research artifacts. They are NOT deprecated — they document the COMP-05 research. The planner's default is to leave them. The conversational-text renderer in `ferro-text` supersedes `CliSummaryRenderer` functionally, but the sketch stays.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Adding `render_hint: None` to each of the 11 struct literal sites is sufficient to preserve all existing serde round-trip tests for `FieldDef`. | §4 | Low — `#[serde(default, skip_serializing_if)]` handles JSON compatibility; test data without `render_hint` key will deserialize to `None`. |
| A2 | `ferro-text` placed in Wave 1b (after ferro-projections in the same loop) is sufficient — no additional wait step needed between the two Wave 1b crates. | §7 | Low — crates.io indexes within seconds on the same publish wave for dependency chains within the same wave; the 30s sleep between waves is the safety margin. |
| A3 | `framework/Cargo.toml` projections feature can simply add `"dep:ferro-text"` without also adding a `ferro-text/projections` sub-feature (since `ferro-text` has no optional features of its own). | §6 | Low — if `ferro-text` has no features, `"dep:ferro-text"` is sufficient. |

**If this table is empty:** All other claims were verified directly from source files.

---

## Open Questions

1. **Builder method name for `FieldDef::render_hint`**
   - What we know: `ActionDef` uses `pub fn precondition(mut self, ...) -> Self` (method named after the field, not `with_precondition`). `GuardDef` uses `pub fn display_name(mut self, ...) -> Self` (same pattern).
   - What's unclear: `render_hint` as a method name shadows the field name (`self.render_hint = Some(hint)`). In Rust this is legal in an `impl` block but might be surprising. `with_render_hint` is unambiguous.
   - Recommendation: Use `pub fn with_render_hint(mut self, hint: RenderHint) -> Self` for clarity, deviating slightly from the no-`with_` pattern. Claude's discretion (D-11).

2. **Whether `cli.rs` sketch is removed**
   - What we know: D-15 says copy the fixture, not depend on the sketch. The CONTEXT.md default is to leave the sketch files.
   - What's unclear: The sketch's `CliSummaryRenderer` now has a production successor. Leaving it risks confusion.
   - Recommendation: Leave as-is per default. Add a doc comment noting it is superseded by `ferro-text::TextRenderer` if desired.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 216 is a pure Rust crate-creation and code-editing phase. No external tools, services, databases, or CLIs beyond the standard Rust toolchain (`cargo`, `rustfmt`, `clippy`) are required, and those are already confirmed present in the workspace.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`#[test]`), `insta` 1.x for snapshots |
| Config file | None — cargo test discovers tests automatically |
| Quick run command | `cargo test -p ferro-text` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CHAN-03 | `render_hint: None` preserves existing `FieldDef` serde behavior | unit | `cargo test -p ferro-projections` | ❌ Wave 0: update existing serde round-trip tests to include `render_hint: None` in existing `FieldDef` construction; add a new round-trip test for `RenderHint::AltText` and `RenderHint::Skip` |
| CHAN-03 | `FieldDef::with_render_hint(AltText("..."))` sets the field | unit | `cargo test -p ferro-projections` | ❌ Wave 0: new test in `field.rs` tests module |
| CHAN-04 | `TextRenderer::render` returns `Err(Error::NoIntents)` on empty intents | unit | `cargo test -p ferro-text` | ❌ Wave 0: new `ferro-text` crate + test |
| CHAN-04 | `approval_workflow` fixture, empty guards → all 4 actions in output | snapshot | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | `approval_workflow` fixture, `is_approver: false` → approve/reject absent from output | snapshot | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | `Brief` verbosity produces shorter output than `Full` for the same fixture | unit | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | `ImageUrl` field with `None` hint renders with "(image)" suffix, not raw URL | unit | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | `Url` field with `AltText("Photo")` renders as "Photo" | unit | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | `Url` field with `Skip` is absent from output | unit | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | Focus intent produces degraded fallback text (not panic, not empty) | unit | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | Analyze intent produces degraded fallback text (not panic, not empty) | unit | `cargo test -p ferro-text` | ❌ Wave 0: new test |
| CHAN-04 | `TextRenderer` reachable from `ferro` facade: `ferro::TextRenderer` compiles | compile | `cargo check -p ferro-rs --features projections` | ❌ Wave 0: facade wiring |
| CHAN-04 | `ferro-projections` adds no renderer (`grep -r "impl Renderer" ferro-projections/src/`) | grep | manual or CI step | ❌ Wave 0: verify during implementation |
| CHAN-04 | `cargo doc -p ferro-text -Dwarnings` produces no warnings | doc | `cargo doc -p ferro-text -Dwarnings` | ❌ Wave 0: write rustdoc during implementation |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-text && cargo test -p ferro-projections`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green + `cargo doc -Dwarnings` clean before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-text/` directory and `Cargo.toml` — new crate scaffold
- [ ] `ferro-text/src/lib.rs` — `TextRenderer` impl + strategy functions + tests
- [ ] `ferro-projections/src/field.rs` — `RenderHint` enum + `render_hint` field on `FieldDef` + builder method + 7 new tests (AltText round-trip, Skip round-trip, builder method, backward-compat serde without key, `None` on Url/ImageUrl behavior)
- [ ] Update 11 existing `FieldDef { ... }` struct literals to add `render_hint: None` (6 in `service.rs`, 4 in `field.rs` tests, 1 in `ferro-json-ui/src/projection/builder.rs`)
- [ ] Root `Cargo.toml` members — add `"ferro-text"`
- [ ] `framework/Cargo.toml` — add `ferro-text` optional dep, extend `projections` feature
- [ ] `framework/src/lib.rs` — add `pub use ferro_text::TextRenderer;` + `RenderHint` to projections re-export
- [ ] `.github/workflows/publish.yml` — add `ferro-text` to `WAVE1B_CRATES`

---

## Security Domain

Security enforcement: this phase involves no authentication, authorization, input parsing from untrusted sources, or cryptography. The `TextRenderer` consumes an already-validated `ServiceDef` (a Rust struct) and produces a `String`. No ASVS categories apply. Omitting security domain section.

---

## Sources

### Primary (HIGH confidence — verified from source files in this session)

- `ferro-projections/src/render/mod.rs` — `BaseContext`, `Verbosity`, `Renderer` trait, `field_display_name`, `is_system_field` (lines 1–169, full file read)
- `ferro-projections/src/field.rs` — `FieldDef`, `FieldMeaning`, `DataType`, `infer_meaning` (lines 1–491, full file read)
- `ferro-projections/src/intent.rs` — `Intent`, `Intent::label()`, `IntentScore`, `IntentHint` (lines 1–333, full file read)
- `ferro-projections/src/action.rs` — `ActionDef`, `InputDef`, `GuardDef` (lines 1–413, full file read)
- `ferro-projections/src/service.rs` — `ServiceDef`, all builder methods (lines 1–1684, full file read)
- `ferro-projections/src/state.rs` — `StateMachine`, `StateDef`, `Transition`, `Warning` (lines 1–714, full file read)
- `ferro-projections/src/error.rs` — `Error` enum including `NoIntents` (lines 1–29, full file read)
- `ferro-projections/src/render/sketch/cli.rs` — `CliSummaryRenderer`, `approval_workflow_fixture` (lines 1–145, full file read)
- `ferro-projections/Cargo.toml` — confirmed `insta` dev-dep (line 20)
- `ferro-json-ui/Cargo.toml` — confirmed optional `ferro-projections` dep, Wave 1a classification
- `ferro-json-ui/src/projection/mod.rs` — `VisualContext`, `JsonUiRenderer` impl (lines 1–100)
- `framework/src/lib.rs` — facade re-export block (lines 250–265)
- `framework/Cargo.toml` — feature definitions, `ferro-json-ui` optional dep (lines 1–60)
- `.github/workflows/publish.yml` — Wave 1a/1b/2/3 crate lists (lines 195–315)
- `Cargo.toml` (root) — workspace members list (lines 1–50)
- `docs/research/comp-05-cross-modality-vocabulary-sketch.md` — anchor fixture, tensions, weaknesses, v14.0 implications (full file)
- `.planning/phases/216-conversational-text-renderer-output-crate/216-CONTEXT.md` — locked decisions (full file)
- `.planning/phases/215-*/215-CONTEXT.md` — Phase 215 decisions (full file)

### Secondary (MEDIUM confidence — grep verification)

- `grep -rn "FieldDef {"` across workspace — confirmed 11 struct literal sites requiring `render_hint: None` migration
- `grep -rn "assert_snapshot\|insta::"` — confirmed insta usage pattern in `ferro-projections/tests/catalog.rs`

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all types and crates verified from source files
- Architecture: HIGH — patterns verified from existing `JsonUiRenderer` and `McpRenderer` implementations
- Pitfalls: HIGH — all pitfalls verified from direct code inspection (struct literal sites counted, wave ordering read from yml)
- Anchor fixture: HIGH — verbatim from `cli.rs:75-129`

**Research date:** 2026-06-13
**Valid until:** 2026-09-13 (stable internal architecture; no external ecosystem)
