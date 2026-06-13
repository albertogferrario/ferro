# Phase 215: Non-visual rendering context — BaseContext + Intent extensions - Research

**Researched:** 2026-06-13
**Domain:** ferro-projections schema/context types, ferro-mcp label migration, ferro-json-ui VisualContext refactor
**Confidence:** HIGH — all findings from direct code inspection; no external sources needed

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Add `evaluated_guards` and `verbosity` to `BaseContext` (`ferro-projections/src/render/mod.rs`).
- **D-02:** Refactor `VisualContext` (`ferro-json-ui/src/projection/mod.rs`) to **embed** `base: BaseContext` rather than re-declaring `intent_index` / `current_state`. The fallback (flat fields + two new fields in both structs) is documented if embedding proves expensive.
- **D-03:** `evaluated_guards: HashMap<String, bool>`, keyed by precondition/guard name.
- **D-04:** Absent key = action renders (guard not-yet-evaluated). `Default` = empty map = render everything.
- **D-05:** `enum Verbosity { Brief, Full }` with `Full` as default. Lives in `ferro-projections` alongside `BaseContext`. Derive `Debug, Clone, Copy, PartialEq, Eq` (no serde unless a consumer serializes context).
- **D-06:** `impl Intent { pub fn label(&self) -> &str }` — infallible; known variants return stable lowercase strings; `Custom(s)` returns `s.as_str()`.
- **D-07:** Migrate three `ferro-mcp` call sites from `format!("{:?}", intent)` to `.label()`. Review projection_coverage.rs:173 and migrate if user-facing.
- **D-08:** Add typed variant (e.g. `Error::NoIntents`) to `ferro-projections::error::Error`; return from render entry points on empty intents slice; covered by unit test.
- **D-09:** `ferro-json-ui` `ProjectionError::EmptyIntents` path unchanged. Phase does not reroute visual renderer through `Error::NoIntents`.

### Claude's Discretion
- Exact variant name (`Error::NoIntents` vs `Error::EmptyIntents`).
- Exact `Verbosity` derive set.
- Whether `Verbosity`/`evaluated_guards` get serde.
- Whether `Intent::label()` returns `&'static str` for known variants via a `match` (must return `&str` to accommodate `Custom`'s borrowed string).

### Deferred Ideas (OUT OF SCOPE)
- `device_class` / `MobileContext` — dropped from v14.0 text-renderer-first milestone scope.
- `FieldDef::render_hint` (AltText/Skip) — CHAN-03, Phase 216.
- The conversational-text `Renderer` — CHAN-04, Phase 216.
- Intent vocabulary reshaping — CHAN-05, research outcome.
- Voice / structured-API / inbound `ferro-ai` classification channels — later.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CHAN-01 | `BaseContext` carries `evaluated_guards` (guard→bool map) and `verbosity` (`Brief`/`Full`). Existing visual/MCP renderers compile unchanged. | D-01 through D-05 fully specify the change; verified current `BaseContext` shape and all consumer sites. |
| CHAN-02 | `Intent::label() -> &str` replaces `format!("{:?}", intent)` across renderers; empty intent slice returns typed error, not `"unknown"`. | D-06 through D-09 specify the change; all `{:?}` call sites verified; existing `Error` enum inspected. |
</phase_requirements>

---

## Summary

Phase 215 extends three types in `ferro-projections` (the renderer-free schema crate) and migrates label call sites in `ferro-mcp`. The structural changes are narrow and mechanically clear: two new fields on `BaseContext`, one new method on `Intent`, one new variant on `Error`. The blast-radius concern is `VisualContext` (D-02 embedding): its struct has nine struct-literal construction sites across the workspace, all of which must migrate from flat-field access to `ctx.base.intent_index` style — or gain `..Default::default()` fill-in for the two new `BaseContext` fields if the planner uses the fallback approach.

The `format!("{:?}", intent)` label sites are **five in number** (not three as CONTEXT.md counted): three in `ferro-mcp/src/tools/` are user-facing label fields; two in `ferro-projections/tests/catalog.rs` and two in sketch renderers (`pub(crate)`) are internal test/debug uses. CONTEXT.md's D-07 correctly targets the `ferro-mcp` tools; the catalog test and sketch uses are internal and the planner may leave them or migrate opportunistically. The `intent_layout.rs:163/167` uses are `assert!` macro error messages in tests, confirmed not labels.

`McpRenderer` in `ferro-mcp-server/src/renderer.rs` ignores the `intents` slice entirely (`_intents`). It derives no label from intent. Success criterion 4 ("ferro-mcp-server build and tests pass unchanged") is satisfied as long as `BaseContext` and `Renderer` trait signatures remain source-compatible, which they will under the extension-only approach.

**Primary recommendation:** Implement D-01 through D-09 in three logical tasks: (1) extend `ferro-projections` types (`BaseContext` + `Verbosity`, `Intent::label()`, `Error::NoIntents`), (2) refactor `VisualContext` to embed `base: BaseContext` and migrate `builder.rs` access sites, (3) migrate `ferro-mcp` tool label call sites. All three tasks are independently verifiable with `cargo test -p ferro-projections`, `cargo test -p ferro-json-ui`, and `cargo test -p ferro-mcp` respectively.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `BaseContext` extension (`evaluated_guards`, `verbosity`) | `ferro-projections` (schema crate) | — | Modality-agnostic; must not live in any renderer crate |
| `Verbosity` enum definition | `ferro-projections` | — | Consumed by future Phase 216 renderer; must be available before any renderer imports it |
| `Intent::label()` method | `ferro-projections` | — | Lives on the `Intent` type; all renderers consume it |
| `Error::NoIntents` variant | `ferro-projections` | — | Modality-agnostic error; renderer-free crate owns the error enum |
| `VisualContext` embedding refactor | `ferro-json-ui` | `ferro-projections` | `VisualContext` is visual-only; embedding is a structural cleanup in its owner crate |
| `builder.rs` field-access migration | `ferro-json-ui` | — | Internal to the visual rendering pipeline |
| MCP label call-site migration | `ferro-mcp` | — | `ferro-mcp` tools own the `IntentInfo.intent` string field |

---

## Standard Stack

No new dependencies introduced in this phase. All changes are to existing types in existing crates.

### Existing crates involved

| Crate | Role in Phase | Key file(s) |
|-------|--------------|-------------|
| `ferro-projections` | Owner of all new/extended types | `src/render/mod.rs`, `src/intent.rs`, `src/error.rs` |
| `ferro-json-ui` | VisualContext embed + builder.rs migration | `src/projection/mod.rs`, `src/projection/builder.rs` |
| `ferro-mcp` | Label call-site migration only | `src/tools/render_projection.rs`, `generate_projection.rs`, `projection_coverage.rs` |
| `ferro-mcp-server` | No changes; must still compile | `src/renderer.rs` |

### Cargo features note

`builder.rs` is gated `#![cfg(feature = "projections")]`. Any test for the `VisualContext` embed that touches `from_service_def_with_catalog` must run with `--features projections` or `--all-features`. [VERIFIED: builder.rs line 17]

---

## Architecture Patterns

### System Architecture Diagram

```
ferro-projections (schema/context crate — renderer-free)
  BaseContext          ← add: evaluated_guards + verbosity + Verbosity enum
  Intent::label()      ← add: stable string labels for all variants
  Error::NoIntents     ← add: typed error for empty intents slice
      │
      ├─── ferro-json-ui (visual renderer output crate)
      │       VisualContext  ← embed base: BaseContext (D-02)
      │       builder.rs     ← ctx.intent_index → ctx.base.intent_index
      │       builder.rs     → Error::NoIntents never reached here (D-09)
      │
      ├─── ferro-mcp (MCP developer tools — consumes projections)
      │       render_projection.rs:94,102   ← {?}intent → .label()
      │       generate_projection.rs:89     ← {?}intent → .label()
      │       projection_coverage.rs:173    ← {?}intent → .label()
      │
      └─── ferro-mcp-server (consumer app MCP output crate)
              McpRenderer    — NO CHANGES: ignores intents slice entirely
```

### Pattern 1: BaseContext extension (additive, backward-compatible)

`BaseContext` currently derives `Debug, Clone, Default` and has NO serde derives. [VERIFIED: render/mod.rs line 21] Adding fields that implement `Default` keeps `BaseContext::default()` backward-compatible automatically.

```rust
// Source: ferro-projections/src/render/mod.rs (current state, verified)
#[derive(Debug, Clone, Default)]
pub struct BaseContext {
    pub intent_index: usize,         // existing
    pub current_state: Option<String>, // existing
    // NEW:
    pub evaluated_guards: HashMap<String, bool>,  // Default = empty = render all
    pub verbosity: Verbosity,                      // Default = Full = current behavior
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    #[default]
    Full,
    Brief,
}
```

`HashMap<String, bool>` implements `Default` (empty map). `Verbosity` with `#[default]` on `Full` means `BaseContext::default()` produces `evaluated_guards: HashMap::new(), verbosity: Verbosity::Full` — preserving all current behavior. [VERIFIED: BaseContext has no Serialize derive; adding HashMap does not introduce serde dependency]

`std::collections::HashMap` is in std — no new Cargo.toml dependency needed.

### Pattern 2: Intent::label() — match-based, not Debug-derived

`Intent` derives `#[serde(rename_all = "snake_case")]` and `Debug`. The serde-serialized strings are the canonical label strings: `"browse"`, `"focus"`, `"collect"`, `"process"`, `"summarize"`, `"analyze"`, `"track"`. [VERIFIED: intent.rs lines 13-36; serde round-trip tests confirm snake_case output]

```rust
// Source: ferro-projections/src/intent.rs — add impl block
impl Intent {
    /// Stable, lowercase string label for this intent.
    ///
    /// Known variants return a `'static str`; `Custom(s)` returns
    /// `s.as_str()` (lifetime bound to the enum value).
    pub fn label(&self) -> &str {
        match self {
            Intent::Browse => "browse",
            Intent::Focus => "focus",
            Intent::Collect => "collect",
            Intent::Process => "process",
            Intent::Summarize => "summarize",
            Intent::Analyze => "analyze",
            Intent::Track => "track",
            Intent::Custom(s) => s.as_str(),
        }
    }
}
```

Return type is `&str` (not `&'static str`) because `Custom(s)` borrows from `self`. Known-variant arms return string literals (which are `&'static str` coerced to `&str`). This is idiomatic Rust for mixed-lifetime return types. [VERIFIED: similar pattern to `display_name.as_deref().unwrap_or(&service.name)` in codebase]

### Pattern 3: Error::NoIntents — extend existing thiserror enum

```rust
// Source: ferro-projections/src/error.rs (current state, verified)
#[derive(Error, Debug)]
pub enum Error {
    #[error("service definition error: {0}")]
    Definition(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    // NEW:
    #[error("cannot render service with no intents")]
    NoIntents,
}
```

Naming recommendation: `NoIntents` (matches the error condition precisely). `EmptyIntents` would also work. The decision is Claude's discretion (CONTEXT.md). `NoIntents` is chosen because it reads as a state description ("there are no intents") rather than a shape description ("the slice is empty") — more meaningful to a renderer author.

### Pattern 4: VisualContext embedding (D-02)

Current `VisualContext` (verified, mod.rs lines 45-68):
```rust
pub struct VisualContext {
    pub intent_index: usize,         // duplicates BaseContext
    pub current_state: Option<String>, // duplicates BaseContext
    pub mode: RenderMode,             // visual-only
    pub templates: Option<ThemeTemplates>, // visual-only
}
```

After embedding:
```rust
pub struct VisualContext {
    pub base: BaseContext,             // collapses intent_index + current_state
    pub mode: RenderMode,
    pub templates: Option<ThemeTemplates>,
}

impl Default for VisualContext {
    fn default() -> Self {
        Self {
            base: BaseContext::default(),
            mode: RenderMode::Display,
            templates: None,
        }
    }
}
```

All existing tests use `VisualContext::default()` or struct-literal syntax. The struct-literal sites are the migration cost.

### Anti-Patterns to Avoid

- **Don't add serde to `Verbosity`** unless a concrete consumer in this phase serializes it. `BaseContext` has no serde derives and adding serde to fields would be inconsistent. `Verbosity` is a runtime signal, not a protocol type.
- **Don't route the visual renderer through `Error::NoIntents`** (D-09 is explicit). `ProjectionError::EmptyIntents` remains the visual path error. The new `Error::NoIntents` is for Phase 216's text renderer.
- **Don't add `#[derive(Default)]` to VisualContext** (remove the handwritten `Default` impl risk): keep the handwritten `impl Default` since it sets `mode: RenderMode::Display` which cannot be derived.
- **Don't migrate `ferro-projections/tests/catalog.rs` debug-format uses**: those are internal test snapshot strings (lines 660, 1090) and sketch renderer uses (`pub(crate)`). Only the `ferro-mcp` tools produce user-facing label strings.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HashMap for evaluated_guards | Custom guard-result type | `std::collections::HashMap<String, bool>` | Already used throughout workspace; zero-cost default (empty map); keyed by the existing `ActionDef::preconditions` / `GuardDef::name` strings (no new ID scheme) |
| thiserror error variant | Manual `fmt::Display` impl | `#[error("cannot render service with no intents")]` on `Error::NoIntents` | Consistent with all four existing variants in `error.rs` |

---

## Common Pitfalls

### Pitfall 1: VisualContext struct-literal sites break on embedding

**What goes wrong:** After embedding, `VisualContext { intent_index: ..., current_state: ..., mode: ..., templates: ... }` is a compile error — the fields `intent_index` and `current_state` no longer exist directly on `VisualContext`.

**Blast radius:** 9 struct-literal sites verified:
- `ferro-json-ui/src/projection/builder.rs`: lines 846, 870, 891, 944, 983, 1011, 1093, 1133 (all in `#[cfg(test)]`)
- `ferro-json-ui/src/projection/mod.rs`: line 174 (test)
- External struct-literal callers: `ferro-ai/tests/projection_roundtrip.rs:33`, `ferro-mcp/tests/agent_harness.rs:275`, `ferro-mcp/src/tools/render_projection.rs:72`

All 9 builder.rs sites are in `mod tests`. The 3 external sites are in integration tests and one production tool.

**How to avoid:** Two migration strategies:
1. **Preferred (embedding):** Update all sites to `VisualContext { base: BaseContext { intent_index: ..., current_state: ..., ..Default::default() }, mode: ..., templates: ... }`. Sites using `..VisualContext::default()` fill-in work as-is since `Default` still exists.
2. **Fallback (flat, D-02 note):** Keep `intent_index` and `current_state` on `VisualContext` directly, add `evaluated_guards` and `verbosity` to both `BaseContext` and `VisualContext`. No struct-literal migration needed, but the duplication persists.

**Warning signs:** Compile errors with "no field `intent_index` on type `VisualContext`" are expected and good — they enumerate every site that needs migration.

### Pitfall 2: builder.rs field access sites

**What goes wrong:** `ctx.intent_index` and `ctx.current_state` in `builder.rs` (non-test production code) must become `ctx.base.intent_index` and `ctx.base.current_state`.

**Affected sites in production code of builder.rs:**
- Line 67: `ctx.intent_index` in `from_service_def` bounds check
- Line 94: `ctx.intent_index` in `from_service_def_with_catalog` (`.get(ctx.intent_index)`)
- Line 100: `ctx.mode` (unchanged — visual-only)
- Line 103–108: `ctx.templates` (unchanged — visual-only)
- Line 109: `ctx` passed to `build_display_spec` (signature changes if VisualContext changes)
- Line 485: `ctx.current_state.clone()` in `emit_kanban_root`

**How to avoid:** Search `ctx\.intent_index\|ctx\.current_state` in builder.rs after applying the embed — compiler catches missed sites.

### Pitfall 3: `approve_workflow` fixture already exists — no re-creation needed

**What goes wrong:** Phase 216 CONTEXT references the `approval_workflow` anchor fixture from COMP-05. Planner might spec a new fixture creation.

**Reality:** The fixture exists in three places in `ferro-projections/src/render/sketch/` (cli.rs, voice.rs, mobile.rs) and in `ferro-projections/tests/catalog.rs` (line 120). For the `Error::NoIntents` unit test in this phase, any minimal fixture (even an empty-fields `ServiceDef`) suffices — the test only needs to call a render entry point with `&[]` and assert the `Error::NoIntents` result.

**How to avoid:** Do not spec a fixture-creation task. The `NoIntents` test can use `ServiceDef::new("x")` inline.

### Pitfall 4: `McpRenderer` context type is `McpContext`, not `BaseContext`

**What goes wrong:** Planner might assume success criterion 4 requires changes to `ferro-mcp-server/src/renderer.rs`.

**Reality:** `McpRenderer::render` ignores both `intents` and `ctx` (both are `_` prefixed). [VERIFIED: renderer.rs lines 27-28 use `_intents` and `_ctx`]. It never derives an intent label. The only change needed for it to pass unchanged is that the `Renderer` trait signature remains compatible, which it will since `BaseContext` extension is additive.

**How to avoid:** Include renderer.rs in the compile-check step but schedule no changes to it.

### Pitfall 5: `catalog.rs` and sketch renderer `{:?}` uses are NOT migration targets

**What goes wrong:** grep returns 8 `format!("{:?}", *intent)` sites. CONTEXT.md D-07 mentions three in ferro-mcp. Planner might expand scope to all 8.

**Reality per site:**
- `ferro-mcp/src/tools/render_projection.rs:94,102` — `IntentInfo.intent: String` (user-facing JSON output field) → **MIGRATE**
- `ferro-mcp/src/tools/generate_projection.rs:89` — same `IntentInfo.intent` field → **MIGRATE**
- `ferro-mcp/src/tools/projection_coverage.rs:173` — `ModelCoverage.primary_intent: Option<String>` (user-facing coverage report) → **MIGRATE** (D-07 "review and migrate if user-facing" — this is user-facing)
- `ferro-projections/tests/catalog.rs:660` — internal `redacted_signals()` snapshot helper, test-only → **LEAVE** (internal debug string, not a label)
- `ferro-projections/tests/catalog.rs:1090` — internal test assertion message → **LEAVE**
- `ferro-projections/src/render/sketch/cli.rs:29` — `pub(crate)` research sketch → **OPTIONAL** (sketch renderers could also adopt `.label()`, but they are research artifacts)
- `ferro-projections/src/render/sketch/mobile.rs:29` — same → **OPTIONAL**

So the mandatory migration is **4 sites** (render_projection.rs:94, :102; generate_projection.rs:89; projection_coverage.rs:173). The planner should scope 4, not 3, to include projection_coverage.rs.

### Pitfall 6: `intent_layout.rs:163,167` are `assert!` error messages, not labels

**What goes wrong:** The CONTEXT.md D-07 mentions these lines. Planner might include them in the label migration.

**Reality:** [VERIFIED: intent_layout.rs] Lines 163 and 167 are:
```
"intent {intent:?} has empty display slots"
"intent {intent:?} has no outer container layout"
```
These are `assert!` panic messages in unit tests. They use `{:?}` for debug output of the iterator variable `intent` (not an `IntentScore.intent` field). These are internal test diagnostics, not user-facing label strings. Leave unchanged.

---

## Code Examples

### Empty-intent error test (for the `Error::NoIntents` success criterion)

```rust
// In ferro-projections/src/render/mod.rs, add to existing #[cfg(test)] mod tests
#[test]
fn render_error_on_empty_intents() {
    // Any renderer that implements the Renderer trait with BaseContext
    // should return Error::NoIntents (or equivalent) on empty slice.
    // For this phase, test via TemplateRenderer (uses BaseContext directly).
    use crate::render::template::TemplateRenderer;
    use crate::render::Renderer;

    let service = crate::service::ServiceDef::new("x");
    let renderer = TemplateRenderer;
    // TemplateRenderer ignores intents today; the NoIntents check
    // must be added to the render path or tested via a new thin entry point.
    // See Open Questions for the exact wiring decision.
}
```

Note: The exact test wiring depends on whether `Error::NoIntents` is checked inside `TemplateRenderer::render`, inside a new helper, or left for the Phase 216 renderer to use (with a standalone free-function test). See Open Questions.

### Intent::label() migration pattern

Before (in `ferro-mcp/src/tools/render_projection.rs`):
```rust
// Lines 91-98: all_intents population
let all_intents: Vec<IntentInfo> = intents
    .iter()
    .map(|is| IntentInfo {
        intent: format!("{:?}", is.intent),  // BEFORE
        ...
    })
    .collect();
```

After:
```rust
let all_intents: Vec<IntentInfo> = intents
    .iter()
    .map(|is| IntentInfo {
        intent: is.intent.label().to_string(),  // AFTER
        ...
    })
    .collect();
```

Note: `.to_string()` is needed because `IntentInfo.intent` is a `String`, not `&str`. `.label()` returns `&str`; `.to_string()` allocates once.

---

## Open Questions (RESOLVED)

1. **Where exactly is `Error::NoIntents` checked and returned?**
   - What we know: `TemplateRenderer` ignores the intents slice entirely today (lines 70-75 of template.rs use `_intents`). The sketch renderers check `intents.get(ctx.intent_index)` and fall through to `"unknown"` (cli.rs:29). No current renderer returns an error on empty intents.
   - What's unclear: Does the CONTEXT.md D-08/D-09 mean (a) add a free function / entry-point wrapper that checks for empty and returns `Error::NoIntents` before dispatching to `Renderer::render`, or (b) add the check inside `TemplateRenderer::render` and each sketch renderer, or (c) just define the variant and write a standalone unit test that constructs the error directly?
   - Recommendation: Option (c) is the minimum to satisfy success criterion 3 ("tested"). The planner should spec `Error::NoIntents` as a well-documented, tested variant used by Phase 216's renderer. A single unit test constructing `Err(Error::NoIntents)` and asserting its `to_string()` message ("cannot render service with no intents") satisfies the SC. The sketch renderers can be updated to return it on empty intents for good hygiene, but that is not required by D-09.
   - **RESOLVED:** Plan 01 Task 2 takes option (c) — `Error::NoIntents` is defined and unit-tested standalone (constructing the error and asserting its `to_string()` message), NOT wired into the visual render path (D-09 keeps `ProjectionError::EmptyIntents`).

2. **Should `builder.rs` `emit_kanban_root` use `ctx.base.current_state` after embedding?**
   - What we know: `emit_kanban_root(service, ctx)` at line 403 takes `&VisualContext`. It uses `ctx.current_state.clone()` at line 485.
   - What's unclear: If the planner chooses the embedding path, `emit_kanban_root`'s signature could stay `&VisualContext` (and access `ctx.base.current_state`) or change to `&BaseContext` (since it only uses `current_state`). The first is simpler.
   - Recommendation: Keep the signature as `&VisualContext` and update the access to `ctx.base.current_state.clone()`. No signature change.
   - **RESOLVED:** Plan 02 Task 1 keeps `emit_kanban_root`'s `&VisualContext` signature and migrates the access to `ctx.base.current_state.clone()` (no signature change).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `insta` (snapshots in catalog.rs) |
| Config file | `Cargo.toml` per crate; no separate test config |
| Quick run (projections) | `cargo test -p ferro-projections` |
| Quick run (json-ui) | `cargo test -p ferro-json-ui --all-features` |
| Quick run (mcp) | `cargo test -p ferro-mcp` |
| Full suite | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| CHAN-01 | `BaseContext::default()` has `evaluated_guards = {}`, `verbosity = Full` | unit | `cargo test -p ferro-projections` | Extend existing `base_context_default` test in `render/mod.rs:98` |
| CHAN-01 | `Verbosity::Full` is the default | unit | `cargo test -p ferro-projections` | New test asserting `Verbosity::default() == Verbosity::Full` |
| CHAN-01 | `VisualContext::default()` still works (embed backward compat) | unit | `cargo test -p ferro-json-ui --all-features` | Existing `visual_context_default_has_sensible_values` test passes unchanged |
| CHAN-01 | `JsonUiRenderer::render` builds and tests pass | compile + unit | `cargo test -p ferro-json-ui --all-features` | All existing builder.rs tests |
| CHAN-02 | `Intent::label()` returns lowercase stable strings for all 7 variants | unit | `cargo test -p ferro-projections` | New test: assert each known variant + `Custom("foo")` |
| CHAN-02 | `Intent::label()` for `Custom` returns the inner string | unit | `cargo test -p ferro-projections` | Part of same test |
| CHAN-02 | Empty intents → `Error::NoIntents` (typed, testable) | unit | `cargo test -p ferro-projections` | New test: construct `Err(Error::NoIntents)`, assert `to_string()` |
| CHAN-02 | `ferro-mcp` tools produce lowercase label strings | unit | `cargo test -p ferro-mcp` | Existing `projection_coverage` tests; add assertion on `primary_intent` format |
| CHAN-02 | `ferro-mcp-server` still compiles and tests pass | compile + unit | `cargo test -p ferro-mcp-server` | No changes to renderer.rs; compile check sufficient |

### Wave 0 Gaps (test infrastructure already exists — no gaps)

None. All changes fit into existing `#[cfg(test)] mod tests` blocks in their respective crates. No new test files, fixtures, or framework installations needed.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is purely code/type changes within the existing Rust workspace. No external tools, services, databases, or CLIs beyond the Rust toolchain are needed.

---

## Sources

### Primary (HIGH confidence — direct code inspection)
- `ferro-projections/src/render/mod.rs` — `BaseContext` definition (line 22), derives, existing tests
- `ferro-projections/src/intent.rs` — `Intent` enum (line 18), serde attributes, existing tests
- `ferro-projections/src/error.rs` — `Error` enum (lines 1-13)
- `ferro-projections/src/action.rs` — `ActionDef::preconditions` (line 34), `GuardDef` (line 149)
- `ferro-projections/src/render/template.rs` — `TemplateRenderer` uses `BaseContext` (lines 68, 74)
- `ferro-projections/src/render/sketch/cli.rs` — `{:?}` label site (line 29), `approval_workflow_fixture`
- `ferro-projections/src/render/sketch/mobile.rs` — same pattern (line 29)
- `ferro-json-ui/src/projection/mod.rs` — `VisualContext` (lines 45-68), `JsonUiRenderer` (lines 98-112)
- `ferro-json-ui/src/projection/builder.rs` — all `ctx.intent_index`/`ctx.current_state` access sites, struct-literal sites
- `ferro-json-ui/src/projection/error.rs` — `ProjectionError::EmptyIntents` (line 21)
- `ferro-json-ui/src/projection/intent_layout.rs` — confirmed lines 163/167 are test assert messages
- `ferro-mcp/src/tools/render_projection.rs` — `{:?}` sites at lines 94, 102
- `ferro-mcp/src/tools/generate_projection.rs` — `{:?}` site at line 89
- `ferro-mcp/src/tools/projection_coverage.rs` — `{:?}` site at line 173
- `ferro-mcp-server/src/renderer.rs` — `McpRenderer` confirmed to ignore `_intents` and `_ctx`
- `ferro-projections/tests/catalog.rs` — confirmed internal debug format uses at lines 660, 1090
- `.planning/phases/215-non-visual-rendering-context-basecontext-intent-extensions/215-CONTEXT.md` — locked decisions D-01..D-09

---

## Assumptions Log

> All claims are VERIFIED from direct code inspection. No ASSUMED entries.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**This table is empty:** All claims in this research were verified by reading the actual source files.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps; all types inspected
- Architecture: HIGH — all construction sites counted; no guessing
- Pitfalls: HIGH — verified against actual code; struct-literal count is exact

**Research date:** 2026-06-13
**Valid until:** 2026-07-13 (codebase is stable; these files change slowly)
