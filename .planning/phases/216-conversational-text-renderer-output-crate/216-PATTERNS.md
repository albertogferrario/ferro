# Phase 216: Conversational-text Renderer (output crate) - Pattern Map

**Mapped:** 2026-06-13
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-text/Cargo.toml` | config (new crate) | — | `ferro-json-ui/Cargo.toml` | exact |
| `ferro-text/src/lib.rs` | renderer (new crate) | request-response (transform) | `ferro-json-ui/src/projection/mod.rs` + `ferro-projections/src/render/sketch/cli.rs` | exact (trait impl) + role-match (strategy shape) |
| `ferro-projections/src/field.rs` | schema extension | — | same file: `FieldMeaning` enum + existing `FieldDef` struct | exact (same file, same derive pattern) |
| `ferro-projections/src/service.rs` (6 sites) | mechanical literal migration | — | same file lines 153, 172, 191, 212, 233, 315 | exact |
| `ferro-projections/src/field.rs` tests (4 sites) | mechanical literal migration | — | same file lines 295, 440, 458, 476 | exact |
| `ferro-json-ui/src/projection/builder.rs` (1 site) | mechanical literal migration | — | same file line 1066 | exact |
| `framework/src/lib.rs` | facade re-export | — | same file lines 263–265 (JsonUiRenderer block) | exact |
| `framework/Cargo.toml` | config (dep + feature) | — | same file line 18, 43 (`projections` feature + ferro-json-ui dep) | exact |
| `Cargo.toml` (root members) | config | — | same file lines 18–34 (member list) | exact |
| `.github/workflows/publish.yml` | CI registration | — | same file line 246 (`WAVE1B_CRATES`) | exact |

---

## Pattern Assignments

### `ferro-text/Cargo.toml` (new crate, config)

**Analog:** `ferro-json-ui/Cargo.toml` (full file)

**Imports/structure pattern** (lines 1–28 of ferro-json-ui/Cargo.toml):

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

[features]
projections = ["dep:ferro-projections", "dep:ferro-theme"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = { version = "1", features = ["derive"] }
strum = { version = "0.26", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
jsonschema = { version = "0.46", default-features = false }
ferro-projections = { path = "../ferro-projections", version = "0.2", optional = true }
ferro-theme = { path = "../ferro-theme", version = "0.2", optional = true }

[dev-dependencies]
serde_json = "1.0"
```

**What `ferro-text/Cargo.toml` copies vs changes:**
- Copy: header fields (`version.workspace`, `edition.workspace`, `license.workspace`, `repository`, `homepage`, `readme`, `categories`)
- Change: `name = "ferro-text"`, `description`, `keywords`
- Change: `[features]` block — `ferro-text` has no features (ferro-projections is a required dep, not optional)
- Change: `[dependencies]` — only `ferro-projections = { path = "../ferro-projections", version = "0.2" }` (required, not optional); no serde/schemars here (RenderHint lives in ferro-projections, not this crate)
- Change: `[dev-dependencies]` — `insta = { version = "1", features = ["yaml"] }`

**Target Cargo.toml:**

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

---

### `ferro-text/src/lib.rs` (new crate, renderer + tests)

**Analog 1 — Renderer trait impl shape:** `ferro-json-ui/src/projection/mod.rs` lines 22–109

**Crate-level doc + module import pattern** (ferro-json-ui/src/projection/mod.rs lines 1–28):

```rust
use ferro_projections::render::{BaseContext, Renderer};
use ferro_projections::Error;
use ferro_projections::IntentScore;
use ferro_projections::ServiceDef;
```

**`impl Renderer` pattern** (ferro-json-ui/src/projection/mod.rs lines 95–109):

```rust
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    type Output = Spec;
    type Context = VisualContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &VisualContext,
    ) -> Result<Spec, Error> {
        Spec::from_service_def(service, intents, ctx).map_err(|e| Error::Render(e.to_string()))
    }
}
```

**What `TextRenderer` copies vs changes:**
- Copy: unit struct, `impl Renderer` skeleton, method signature shape
- Change: `type Output = String; type Context = BaseContext;` (no VisualContext wrapper — D-02)
- Change: body dispatches to per-intent strategy fns instead of delegating to a builder

**Analog 2 — Strategy function shape + helpers:** `ferro-projections/src/render/sketch/cli.rs` lines 14–64

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
        use std::fmt::Write;

        let title = service.display_name.as_deref().unwrap_or(&service.name);
        let intent_label = intents
            .get(ctx.intent_index)
            .map(|s| format!("{:?}", s.intent).to_lowercase())  // <-- DO NOT copy this; use .label() instead
            .unwrap_or_else(|| "unknown".to_string());           // <-- DO NOT copy; return Error::NoIntents

        let mut out = String::new();
        let _ = writeln!(out, "{title} [{intent_label}]");

        // Domain fields (drop system fields).
        for f in &service.fields {
            if !is_system_field(&f.meaning) {
                let _ = writeln!(out, "  - {} ({:?})", field_display_name(&f.name), f.meaning);
            }
        }
        // ...
        Ok(out)
    }
}
```

**Anti-patterns from cli.rs to avoid (both documented in RESEARCH.md):**
- `format!("{:?}", s.intent).to_lowercase()` — use `s.intent.label()` instead (Phase 215 D-06)
- `.unwrap_or_else(|| "unknown")` for empty intents — return `Error::NoIntents` instead (Phase 215 D-07)
- Listing all actions without guard filter — filter by `evaluated_guards` (D-09)

**Reusable helpers imported from ferro-projections** (ferro-projections/src/render/mod.rs lines 91–113):

```rust
pub fn field_display_name(name: &str) -> String {
    name.split('_')
        .map(|word| { /* Title Case */ })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn is_system_field(meaning: &FieldMeaning) -> bool {
    matches!(
        meaning,
        FieldMeaning::Identifier | FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt
    )
}
```

Import these as `ferro_projections::render::{field_display_name, is_system_field}` — do not reimplement.

**Renderer test pattern** (ferro-json-ui/src/projection/mod.rs lines 126–192):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::render::BaseContext;
    use ferro_projections::{derive_intents, DataType, FieldMeaning, ServiceDef};

    fn sample_service() -> ServiceDef {
        ServiceDef::new("product")
            .display_name("Product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
    }

    #[test]
    fn render_empty_intents_returns_render_error() {
        let service = sample_service();
        let result = JsonUiRenderer.render(&service, &[], &VisualContext::default());
        assert!(matches!(result, Err(Error::Render(_))));
    }
}
```

**Anchor fixture to copy verbatim** (ferro-projections/src/render/sketch/cli.rs lines 75–129):

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

Note: after `render_hint` is added to `FieldDef`, the `.field()` builder methods on `ServiceDef` (which construct `FieldDef` internally in `service.rs`) will be the migration sites, not the fixture itself. The fixture calls the builder API, so it is unaffected by the literal migration.

---

### `ferro-projections/src/field.rs` — `RenderHint` enum + `render_hint` field (CHAN-03)

**Analog: `FieldMeaning` enum + `FieldDef` struct** (ferro-projections/src/field.rs lines 23–72)

**FieldMeaning derive pattern** (lines 30–56) — copy this derive set for `RenderHint`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldMeaning {
    Identifier,
    // ...
    #[serde(untagged)]
    Custom(String),
}
```

**FieldDef struct as-is** (lines 59–72) — the struct to extend:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
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
}
```

**Target additions:**

1. Add `RenderHint` enum before `FieldDef` (same file, similar placement to `FieldMeaning`):

```rust
/// Non-visual rendering hint for URL/ImageUrl fields.
///
/// Applied by non-visual renderers (e.g., `TextRenderer`) to handle fields
/// that have no meaningful text representation without context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RenderHint {
    /// Substitute this string in place of the raw URL/ImageUrl value.
    AltText(String),
    /// Omit this field entirely from non-visual output.
    Skip,
}
```

2. Extend `FieldDef` with `render_hint` field (add after `writable`):

```rust
    /// Non-visual rendering hint. `None` preserves current behavior.
    /// Used by `TextRenderer` and future non-visual renderers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_hint: Option<RenderHint>,
```

3. Add builder method to `impl FieldDef` (consuming, matching `ActionDef`'s `with_*` style):

```rust
impl FieldDef {
    pub fn with_render_hint(mut self, hint: RenderHint) -> Self {
        self.render_hint = Some(hint);
        self
    }
}
```

**Public re-export pattern** (ferro-projections/src/lib.rs line 16):

```rust
pub use field::{infer_meaning, DataType, FieldDef, FieldMeaning};
```

Add `RenderHint` to this line:

```rust
pub use field::{infer_meaning, DataType, FieldDef, FieldMeaning, RenderHint};
```

---

### FieldDef literal migration — 11 sites adding `render_hint: None`

**All 11 sites are exhaustive struct literals that must add `render_hint: None`.** The error the compiler emits without it: `error[E0063]: missing field 'render_hint' in initializer`.

**Site group 1: `ferro-projections/src/service.rs` — 6 builder method sites**

Each looks like (line 153, `.field()` method):

```rust
self.fields.push(FieldDef {
    name: name.into(),
    data_type,
    meaning,
    required: true,
    is_list: false,
    readable: true,
    writable: true,
    // ADD:
    render_hint: None,
});
```

The pattern is identical for all 6 sites (`.field()` at line 153, `.optional_field()` at line 172, `.list_field()` at line 191, `.read_only_field()` at line 212, `.write_only_field()` at line 233, `from_model()` push at line 315). Each adds `render_hint: None` as the last field before the closing `}`.

The `from_model()` site (line 315) sets `writable: !is_system` — same pattern, same `render_hint: None` addition:

```rust
def.fields.push(FieldDef {
    name: field.name.clone(),
    data_type,
    meaning,
    required: !field.is_nullable,
    is_list: false,
    readable: true,
    writable: !is_system,
    // ADD:
    render_hint: None,
});
```

**Site group 2: `ferro-projections/src/field.rs` — 4 test sites**

Each is a struct literal in a `#[test]` function (lines 295, 440, 458, 476). Representative (line 295):

```rust
let field = FieldDef {
    name: "total".to_string(),
    data_type: DataType::Float,
    meaning: FieldMeaning::Money,
    required: true,
    is_list: false,
    readable: true,
    writable: true,
    // ADD:
    render_hint: None,
};
```

All four test sites follow the same struct literal shape. Each adds `render_hint: None`.

**Site group 3: `ferro-json-ui/src/projection/builder.rs` — 1 test site (line 1066)**

```rust
let field = ferro_projections::FieldDef {
    name: "x".into(),
    data_type: DataType::String,
    meaning: FieldMeaning::Email,
    required: false,
    is_list: false,
    readable: true,
    writable: true,
    // ADD:
    render_hint: None,
};
```

---

### `framework/src/lib.rs` — facade re-export (lines 255–265)

**Analog: existing projections + json-ui re-export block** (lines 255–265):

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

**Target changes:**

1. Add `RenderHint` and `Verbosity` to the `ferro_projections` re-export line (after `FieldMeaning`):

```rust
pub use ferro_projections::{
    derive_intents, infer_meaning, ActionDef, Cardinality, DataType, Error as ProjectionsError,
    FieldDef, FieldMeaning, GuardDef, InputDef, Intent, IntentHint, IntentScore, NavigationHint,
    RelationshipDef, RenderHint, Renderer, ServiceDef, StateDef, StateMachine, Transition,
    Verbosity, Warning as ProjectionsWarning,
};
```

Note: `Verbosity` and `BaseContext` are in `ferro_projections::render`, not the top-level `ferro_projections` namespace. Check `ferro-projections/src/lib.rs` line 20 — `pub use render::{BaseContext, Renderer}` is already re-exported at the crate root; `Verbosity` is in `render` mod but not in the current crate root re-export. The planner must check whether `Verbosity` needs adding to `ferro-projections/src/lib.rs` first, then re-export from framework.

2. Add a new re-export block immediately after the `ferro_json_ui` line:

```rust
// Re-export text renderer from ferro-text
#[cfg(feature = "projections")]
pub use ferro_text::TextRenderer;
```

---

### `framework/Cargo.toml` — dependency + feature

**Analog: existing `projections` feature + `ferro-json-ui` optional dep** (lines 18, 43):

```toml
[features]
projections = ["dep:ferro-projections", "dep:ferro-json-ui", "ferro-json-ui/projections"]

[dependencies]
ferro-json-ui = { path = "../ferro-json-ui", version = "0.2", optional = true }
```

**Target changes:**

1. Add `ferro-text` optional dep (alongside `ferro-json-ui` on line 43):

```toml
ferro-text = { path = "../ferro-text", version = "0.2", optional = true }
```

2. Extend `projections` feature (line 18) to pull `ferro-text`:

```toml
projections = ["dep:ferro-projections", "dep:ferro-json-ui", "ferro-json-ui/projections", "dep:ferro-text"]
```

---

### `Cargo.toml` (root) — workspace members

**Analog: existing members list** (lines 3–35):

```toml
[workspace]
resolver = "2"
members = [
    "framework",
    "app",
    "ferro-cli",
    ...
    "ferro-assets",
]
```

**Target change:** Add `"ferro-text"` as a new entry. Placement: anywhere in the list (alphabetical or at end of list, after `"ferro-assets"`):

```toml
    "ferro-assets",
    "ferro-text",
```

---

### `.github/workflows/publish.yml` — CI wave registration

**Analog: `WAVE1B_CRATES` line** (line 246):

```
WAVE1B_CRATES="ferro-projections ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments"
```

**Target change:** Insert `ferro-text` immediately after `ferro-projections` (since it depends on ferro-projections and must publish after it in the same wave):

```
WAVE1B_CRATES="ferro-projections ferro-text ferro-ai ferro-stripe ferro-whatsapp ferro-notifications ferro-reservation ferro-projection ferro-deployments"
```

Also update the comment block above (lines 241–246) to document the new dep:

```
# ferro-text         -> ferro-projections
```

**Wave placement rationale:** `ferro-text` has a required (non-optional) dep on `ferro-projections`. It cannot go in Wave 1a (which `ferro-json-ui` is in, but only because `ferro-json-ui`'s dep is optional). Placing it in Wave 1b after `ferro-projections` is correct; the sequential loop ensures `ferro-projections` publishes before `ferro-text` within the same wave.

---

## Shared Patterns

### Consuming `BaseContext` fields

**Source:** `ferro-projections/src/render/mod.rs` lines 36–50

```rust
#[derive(Debug, Clone, Default)]
pub struct BaseContext {
    pub intent_index: usize,
    pub current_state: Option<String>,
    pub evaluated_guards: HashMap<String, bool>,
    pub verbosity: Verbosity,
}
```

**Apply to:** `ferro-text/src/lib.rs` — the renderer accesses `ctx.intent_index`, `ctx.current_state`, `ctx.evaluated_guards`, `ctx.verbosity` directly. No wrapping needed (D-02).

**Guard filtering pattern** (D-09 — absent key = render, only explicit `false` filters):

```rust
fn action_passes_guards(
    action: &ActionDef,
    evaluated_guards: &HashMap<String, bool>,
) -> bool {
    action.preconditions.iter().all(|guard_name| {
        evaluated_guards.get(guard_name.as_str()).copied().unwrap_or(true)
    })
}
```

### Error handling

**Source:** `ferro-projections/src/error.rs` lines 1–18

```rust
pub enum Error {
    Definition(String),
    Validation(String),
    Render(String),
    Serialization(#[from] serde_json::Error),
    /// Empty intents slice — no render target. (D-08)
    NoIntents,
}
```

**Apply to:** `ferro-text/src/lib.rs` — return `Err(Error::NoIntents)` when `intents.get(ctx.intent_index)` returns `None`. Do not return `Ok("unknown".to_string())`.

### Serde annotation pattern for optional fields

**Source:** `ferro-projections/src/field.rs` lines 64–71

```rust
#[serde(default = "default_true")]
pub required: bool,
#[serde(default)]
pub is_list: bool,
```

**Apply to:** `render_hint` field in `FieldDef`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub render_hint: Option<RenderHint>,
```

`#[serde(default)]` ensures old JSON without the key deserializes to `None`. `skip_serializing_if` keeps JSON output clean when hint is absent.

### Consuming builder style (no `with_` prefix)

**Source:** `ferro-projections/src/action.rs` — `ActionDef` uses:

```rust
pub fn precondition(mut self, guard_name: impl Into<String>) -> Self {
    self.preconditions.push(guard_name.into());
    self
}
```

**Apply to:** `FieldDef` builder method. The RESEARCH.md recommends `with_render_hint` to avoid shadowing the field name; either `with_render_hint` or `render_hint` (method name) is acceptable per Claude's discretion (D-11). The consuming `mut self -> Self` shape is mandatory.

---

## No Analog Found

All files have analogs in the codebase. No files require falling back to RESEARCH.md patterns exclusively.

---

## Metadata

**Analog search scope:** `ferro-json-ui/`, `ferro-projections/`, `framework/`, `Cargo.toml` (root), `.github/workflows/publish.yml`
**Files scanned:** 14 source files read directly
**Pattern extraction date:** 2026-06-13

**Key structural observations:**
- `ferro-text` is the third renderer output crate; the `ferro-json-ui/src/projection/mod.rs` `impl Renderer` block is the exact structural template for `ferro-text/src/lib.rs`.
- `ferro-projections/src/render/sketch/cli.rs` provides the `approval_workflow_fixture` to copy verbatim (lines 75–129) and the per-field iteration pattern to learn from — but two of its patterns (intent label via `{:?}`, silent `"unknown"` fallback) must NOT be copied.
- The 11 `FieldDef` struct literal sites are the only breaking-change migration cost in existing code. All are mechanical `render_hint: None` additions.
- `Verbosity` is currently `pub` in `ferro-projections::render` but is NOT in the crate root re-export (`ferro-projections/src/lib.rs` line 20 re-exports only `BaseContext` and `Renderer`). The planner must add `Verbosity` to the crate root re-export before the facade can re-export it from `framework`.
