# Phase 215: Non-visual rendering context — BaseContext + Intent extensions - Pattern Map

**Mapped:** 2026-06-13
**Files analyzed:** 7 modified files
**Analogs found:** 7 / 7

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-projections/src/render/mod.rs` | schema/context type extension | transform | itself (existing `BaseContext`) | self-extension |
| `ferro-projections/src/intent.rs` | domain enum + method | transform | `ferro-projections/src/field.rs` (`FieldMeaning` enum) | role-match |
| `ferro-projections/src/error.rs` | error enum extension | — | itself + `ferro-json-ui/src/projection/error.rs` | self-extension |
| `ferro-json-ui/src/projection/mod.rs` | context struct refactor | transform | `ferro-projections/src/render/sketch/cli.rs` (`BaseContext` embed pattern) | role-match |
| `ferro-json-ui/src/projection/builder.rs` | field-access migration | transform | itself (struct-literal tests at lines 846–1133) | self-migration |
| `ferro-mcp/src/tools/render_projection.rs` | MCP tool label migration | request-response | `ferro-mcp/src/tools/generate_projection.rs` (parallel label site) | exact |
| `ferro-mcp/src/tools/generate_projection.rs` | MCP tool label migration | request-response | `ferro-mcp/src/tools/render_projection.rs` (parallel label site) | exact |
| `ferro-mcp/src/tools/projection_coverage.rs` | MCP tool label migration | request-response | `ferro-mcp/src/tools/render_projection.rs` (parallel label site) | exact |

---

## Pattern Assignments

### `ferro-projections/src/render/mod.rs` — add `evaluated_guards`, `verbosity`, `Verbosity`

**Analog:** itself — the existing `BaseContext` and `RenderMode` (in `ferro-json-ui`) establish the derive set and Default conventions.

**Existing struct to extend** (lines 21–27):
```rust
#[derive(Debug, Clone, Default)]
pub struct BaseContext {
    /// Which intent to render (0 = primary). Index into the `intents` slice.
    pub intent_index: usize,
    /// Current workflow state name (relevant for Process/Track intents).
    pub current_state: Option<String>,
}
```

**Change shape — add two fields:**
```rust
#[derive(Debug, Clone, Default)]
pub struct BaseContext {
    pub intent_index: usize,
    pub current_state: Option<String>,
    // NEW: guard-name → evaluated result; absent key = render the action (D-03/D-04)
    pub evaluated_guards: std::collections::HashMap<String, bool>,
    // NEW: text detail level; Full preserves current behavior (D-05)
    pub verbosity: Verbosity,
}
```

`HashMap<String, bool>` implements `Default` (empty map). `Verbosity` with `#[default]` on `Full` means `BaseContext::default()` is backward-compatible — existing callers need no changes.

`std::collections::HashMap` is already used in the workspace; no new Cargo dependency.

**New `Verbosity` enum — closest style analog is `RenderMode`** (`ferro-json-ui/src/projection/mod.rs` lines 31–38):
```rust
// ANALOG (ferro-json-ui/src/projection/mod.rs lines 31-38):
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    Display,
    Input,
}
```

`Verbosity` does NOT get serde (BaseContext has no serde; stay consistent). Drop `Serialize, Deserialize` and `#[serde(...)]`. Add `#[default]` instead:

```rust
// Verbosity — copy RenderMode's structure, strip serde, add #[default]:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    #[default]
    Full,
    Brief,
}
```

**Existing test to extend** (lines 97–102) — add two assertions:
```rust
#[test]
fn base_context_default() {
    let ctx = BaseContext::default();
    assert_eq!(ctx.intent_index, 0);
    assert!(ctx.current_state.is_none());
    // NEW assertions:
    assert!(ctx.evaluated_guards.is_empty());
    assert_eq!(ctx.verbosity, Verbosity::Full);
}
```

---

### `ferro-projections/src/intent.rs` — add `Intent::label()`

**Analog for `as_str` / string-accessor pattern:** `ferro-json-ui/src/projection/builder.rs` line 485 uses `.clone()` on `ctx.current_state`, and `ferro-projections/src/render/sketch/cli.rs` line 26 uses `.as_deref().unwrap_or(&service.name)` — the workspace idiom for borrowing `&str` from enum data. For a match-based `&str` method on an enum, the closest pattern in the codebase is the `parse_intent` function in `ferro-mcp/src/tools/render_projection.rs` lines 462–474, which maps `Intent` variant names to strings.

**Existing test pattern** (`intent.rs` lines 113–123 — serde round-trip proves snake_case labels):
```rust
#[test]
fn intent_snake_case_serialization() {
    assert_eq!(
        serde_json::to_string(&Intent::Browse).unwrap(),
        r#""browse""#
    );
    assert_eq!(
        serde_json::to_string(&Intent::Summarize).unwrap(),
        r#""summarize""#
    );
}
```
The serde-serialized strings are the canonical label values `label()` must return.

**New impl block to add after the `Intent` enum** (after line 36):
```rust
impl Intent {
    /// Stable, lowercase string label for this intent.
    ///
    /// Known variants return a `'static str`; `Custom(s)` returns
    /// `s.as_str()` (lifetime bound to the enum value).
    pub fn label(&self) -> &str {
        match self {
            Intent::Browse    => "browse",
            Intent::Focus     => "focus",
            Intent::Collect   => "collect",
            Intent::Process   => "process",
            Intent::Summarize => "summarize",
            Intent::Analyze   => "analyze",
            Intent::Track     => "track",
            Intent::Custom(s) => s.as_str(),
        }
    }
}
```

Return type is `&str` (not `&'static str`) because `Custom(s)` borrows from `self`. Known-variant arms return string literals which coerce to `&str`. This is the only way to unify the two lifetimes in Rust.

**New test to add in `intent.rs` `mod tests`:**
```rust
#[test]
fn intent_label_known_variants() {
    assert_eq!(Intent::Browse.label(), "browse");
    assert_eq!(Intent::Focus.label(), "focus");
    assert_eq!(Intent::Collect.label(), "collect");
    assert_eq!(Intent::Process.label(), "process");
    assert_eq!(Intent::Summarize.label(), "summarize");
    assert_eq!(Intent::Analyze.label(), "analyze");
    assert_eq!(Intent::Track.label(), "track");
}

#[test]
fn intent_label_custom_returns_inner_string() {
    assert_eq!(Intent::Custom("reporting".into()).label(), "reporting");
}
```

---

### `ferro-projections/src/error.rs` — add `Error::NoIntents`

**Analog:** the existing four variants in this file (lines 1–13) and `ProjectionError` in `ferro-json-ui/src/projection/error.rs` (line 21, `EmptyIntents` variant with a unit form).

**Existing enum** (lines 1–13 — copy the attribute style exactly):
```rust
use thiserror::Error;

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
}
```

**Closest structural analog for a unit variant:** `ProjectionError::EmptyIntents` in `ferro-json-ui/src/projection/error.rs` line 21:
```rust
/// Caller supplied an empty intents slice — no projection target exists.
#[error("cannot project service with no intents")]
EmptyIntents,
```

**Change shape — append one variant:**
```rust
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
    // NEW: returned by render entry points on empty intents slice (D-08)
    #[error("cannot render service with no intents")]
    NoIntents,
}
```

`NoIntents` is a unit variant (no payload). Naming convention: state description, not shape description (`NoIntents` not `EmptyIntents`). The `ferro-json-ui` visual path keeps its own `ProjectionError::EmptyIntents` (D-09: unchanged).

**New test to add in `ferro-projections/src/render/mod.rs` `mod tests` (or inline in `error.rs`):**
```rust
#[test]
fn no_intents_error_message() {
    let err = crate::error::Error::NoIntents;
    assert_eq!(err.to_string(), "cannot render service with no intents");
}
```
This satisfies success criterion 3 with zero fixture setup — just construct the variant and assert `Display`.

---

### `ferro-json-ui/src/projection/mod.rs` — embed `base: BaseContext` in `VisualContext`

**Analog:** `ferro-projections/src/render/sketch/cli.rs` — `CliSummaryRenderer` uses `BaseContext` directly as its `Context` type. The embed pattern makes `VisualContext` a superset of `BaseContext`.

**Existing `VisualContext` struct** (lines 44–68 — the complete current shape):
```rust
#[derive(Debug, Clone)]
pub struct VisualContext {
    /// Which intent to render (0 = primary). Index into the `intents` slice.
    pub intent_index: usize,
    /// Current workflow state name (relevant for Process/Track intents).
    pub current_state: Option<String>,
    /// Display or Input mode.
    pub mode: RenderMode,
    /// Optional theme template overrides.
    pub templates: Option<ThemeTemplates>,
}

impl Default for VisualContext {
    fn default() -> Self {
        Self {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Display,
            templates: None,
        }
    }
}
```

**Change shape — collapse `intent_index` + `current_state` into embedded `BaseContext`:**
```rust
#[derive(Debug, Clone)]
pub struct VisualContext {
    /// Modality-agnostic context (intent index, state, guards, verbosity).
    pub base: BaseContext,
    /// Display or Input mode.
    pub mode: RenderMode,
    /// Optional theme template overrides.
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

Do NOT add `#[derive(Default)]` — keep the hand-written impl because `mode: RenderMode::Display` cannot be derived.

**Import change required** — add `BaseContext` to the use statement at the top of `mod.rs`:
```rust
use ferro_projections::render::{BaseContext, Renderer};
```

**Existing test to update** (lines 162–168 — two assertions change field path):
```rust
// BEFORE:
fn visual_context_default_has_sensible_values() {
    let ctx = VisualContext::default();
    assert_eq!(ctx.intent_index, 0);
    assert!(ctx.current_state.is_none());
    assert_eq!(ctx.mode, RenderMode::Display);
    assert!(ctx.templates.is_none());
}

// AFTER:
fn visual_context_default_has_sensible_values() {
    let ctx = VisualContext::default();
    assert_eq!(ctx.base.intent_index, 0);
    assert!(ctx.base.current_state.is_none());
    assert_eq!(ctx.mode, RenderMode::Display);
    assert!(ctx.templates.is_none());
}
```

---

### `ferro-json-ui/src/projection/builder.rs` — migrate field-access sites

**Change shape:** all `ctx.intent_index` → `ctx.base.intent_index`, all `ctx.current_state` → `ctx.base.current_state`. The four remaining visual-only fields (`ctx.mode`, `ctx.templates`) are unchanged.

**Production code field-access sites** (verified by research):

| Line | Before | After |
|------|--------|-------|
| 67 | `ctx.intent_index` | `ctx.base.intent_index` |
| 94 | `ctx.intent_index` | `ctx.base.intent_index` |
| 485 | `ctx.current_state.clone()` | `ctx.base.current_state.clone()` |

Lines 100 (`ctx.mode`) and 103–108 (`ctx.templates`) are visual-only — unchanged.

**Test struct-literal sites** — 8 sites in `mod tests` (lines 846, 870, 891, 944, 983, 1011, 1093, 1133). They use two patterns:

**Pattern A — explicit field construction (e.g. line 846):**
```rust
// BEFORE:
let ctx = VisualContext {
    intent_index: intents.iter().position(...).unwrap_or(0),
    mode: RenderMode::Display,
    ..Default::default()
};

// AFTER:
let ctx = VisualContext {
    base: BaseContext {
        intent_index: intents.iter().position(...).unwrap_or(0),
        ..Default::default()
    },
    mode: RenderMode::Display,
    ..Default::default()
};
```

**Pattern B — `..Default::default()` fill-in only (e.g. line 944):**
```rust
// BEFORE:
let ctx = VisualContext {
    intent_index: intents.iter().position(|i| matches!(i.intent, Intent::Browse)).unwrap_or(0),
    mode: RenderMode::Display,
    templates: Some(templates),
    ..Default::default()
};

// AFTER:
let ctx = VisualContext {
    base: BaseContext {
        intent_index: intents.iter().position(|i| matches!(i.intent, Intent::Browse)).unwrap_or(0),
        ..Default::default()
    },
    mode: RenderMode::Display,
    templates: Some(templates),
    ..Default::default()
};
```

**External struct-literal sites** (not in builder.rs — also need migration):
- `ferro-ai/tests/projection_roundtrip.rs:33`
- `ferro-mcp/tests/agent_harness.rs:275`
- `ferro-mcp/src/tools/render_projection.rs:72`

The compiler will catch all missed sites with "no field `intent_index` on type `VisualContext`" — use compiler errors as the migration checklist.

**`builder.rs` feature gate note:** this file is `#![cfg(feature = "projections")]`. Run tests with `cargo test -p ferro-json-ui --all-features`.

---

### `ferro-mcp/src/tools/render_projection.rs` — migrate `{:?}` label sites at lines 94 and 102

**Analog:** `ferro-mcp/src/tools/generate_projection.rs` line 89 (identical pattern, parallel migration).

**Line 94 — `all_intents` population** (lines 91–98):
```rust
// BEFORE:
let all_intents: Vec<IntentInfo> = intents
    .iter()
    .map(|is| IntentInfo {
        intent: format!("{:?}", is.intent),
        confidence: is.confidence,
        signals: is.matching_signals.clone(),
    })
    .collect();

// AFTER:
let all_intents: Vec<IntentInfo> = intents
    .iter()
    .map(|is| IntentInfo {
        intent: is.intent.label().to_string(),
        confidence: is.confidence,
        signals: is.matching_signals.clone(),
    })
    .collect();
```

**Line 102 — `RenderResult.intent` field** (lines 100–103):
```rust
// BEFORE:
Ok(RenderResult {
    service_name: detail.service_name,
    intent: format!("{:?}", selected.intent),
    ...
})

// AFTER:
Ok(RenderResult {
    service_name: detail.service_name,
    intent: selected.intent.label().to_string(),
    ...
})
```

`.to_string()` is required because `IntentInfo.intent` is `String`; `.label()` returns `&str`.

**Existing test that pins the format** (line 488 — must update the expected value):
```rust
// BEFORE (test):
all_intents: vec![IntentInfo {
    intent: "Browse".to_string(),  // Debug format = PascalCase
    ...
}],

// AFTER (test):
all_intents: vec![IntentInfo {
    intent: "browse".to_string(),  // label() format = lowercase
    ...
}],
```

---

### `ferro-mcp/src/tools/generate_projection.rs` — migrate `{:?}` label site at line 89

**Analog:** `ferro-mcp/src/tools/render_projection.rs` (migrated above — exact same pattern).

**Lines 86–93** (the `intent_infos` mapping):
```rust
// BEFORE:
let intent_infos: Vec<IntentInfo> = intents
    .iter()
    .map(|score| IntentInfo {
        intent: format!("{:?}", score.intent),
        confidence: score.confidence,
        signals: score.matching_signals.clone(),
    })
    .collect();

// AFTER:
let intent_infos: Vec<IntentInfo> = intents
    .iter()
    .map(|score| IntentInfo {
        intent: score.intent.label().to_string(),
        confidence: score.confidence,
        signals: score.matching_signals.clone(),
    })
    .collect();
```

---

### `ferro-mcp/src/tools/projection_coverage.rs` — migrate `{:?}` label site at line 173

**Analog:** `ferro-mcp/src/tools/render_projection.rs` (migrated above).

**Lines 171–176** in `derive_primary_intent`:
```rust
// BEFORE:
if let Some(primary) = intents.first() {
    (
        Some(format!("{:?}", primary.intent)),
        Some(primary.confidence),
    )

// AFTER:
if let Some(primary) = intents.first() {
    (
        Some(primary.intent.label().to_string()),
        Some(primary.confidence),
    )
```

This is the user-facing `ModelCoverage.primary_intent: Option<String>` field — a label, not debug output. Research confirms it is in scope for D-07 migration.

---

## Shared Patterns

### `thiserror` unit variant style
**Source:** `ferro-json-ui/src/projection/error.rs` line 21 (`EmptyIntents`)
**Apply to:** `ferro-projections/src/error.rs` new `NoIntents` variant
```rust
/// Caller supplied an empty intents slice — no projection target exists.
#[error("cannot project service with no intents")]
EmptyIntents,
```
Copy the doc comment + `#[error("...")]` attribute style verbatim, changing only the message text and variant name.

### `Default` on non-`#[derive(Default)]` structs
**Source:** `ferro-json-ui/src/projection/mod.rs` lines 59–67 (`impl Default for VisualContext`)
**Apply to:** `VisualContext` after embedding — keep the hand-written impl, do NOT switch to `#[derive(Default)]`. The `mode: RenderMode::Display` field has no `Default` impl on `RenderMode`.

### `BaseContext` as the direct `Context` type for a renderer
**Source:** `ferro-projections/src/render/sketch/cli.rs` lines 14–23
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
Phase 216's text renderer will follow this exact pattern (different `Output` type, same `Context = BaseContext` wiring).

### Enum `label()` method returning `&str` with mixed lifetime
**Source:** `ferro-projections/src/render/sketch/cli.rs` line 26 (`.as_deref().unwrap_or(&service.name)`) and `ferro-projections/src/intent.rs` lines 93–95 (`.as_str()` usage in tests)

The idiomatic return type for a method that returns `'static` strings for known variants and a borrowed `&str` for a `Custom(String)` variant is `&str` (not `&'static str`). Rust unifies the lifetimes automatically.

---

## No Analog Found

None — all modified files have clear analogs in the codebase.

---

## Migration Count Summary

| File | Change Type | Sites |
|------|------------|-------|
| `ferro-projections/src/render/mod.rs` | additive (2 fields + 1 enum + test extensions) | 1 struct, 1 new enum, 1 test to extend + 1 new test |
| `ferro-projections/src/intent.rs` | additive (impl block + 2 tests) | 1 new impl, 2 new tests |
| `ferro-projections/src/error.rs` | additive (1 variant) | 1 variant append |
| `ferro-json-ui/src/projection/mod.rs` | struct reshape (2 fields → 1 embedded struct) | struct def + Default impl + 1 import + 1 test update |
| `ferro-json-ui/src/projection/builder.rs` | field-access migration | 3 prod + 8 test + 3 external struct-literal sites |
| `ferro-mcp/src/tools/render_projection.rs` | label string migration | 2 prod sites + 1 test string update |
| `ferro-mcp/src/tools/generate_projection.rs` | label string migration | 1 prod site |
| `ferro-mcp/src/tools/projection_coverage.rs` | label string migration | 1 prod site |

---

## Metadata

**Analog search scope:** `ferro-projections/`, `ferro-json-ui/`, `ferro-mcp/`, `ferro-mcp-server/`
**Files scanned:** 11 source files read directly
**Pattern extraction date:** 2026-06-13
