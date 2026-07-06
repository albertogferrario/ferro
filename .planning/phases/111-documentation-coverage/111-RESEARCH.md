# Phase 111: Documentation Coverage - Research

**Researched:** 2026-03-26
**Domain:** Technical documentation writing — Service Projections, FerroModel derive macro, ValidateRules derive macro
**Confidence:** HIGH

## Summary

Phase 111 requires creating two new documentation pages. Both fill gaps where shipped
framework features have no user-facing documentation page at all. The work is
documentation authoring, not implementation. All source code exists and compiles.

The Service Projections page (`docs/src/features/projections.md`) must explain the
`ServiceDef` builder API, the five-signal intent derivation engine (`derive_intents`),
the `JsonUiRenderer`, and the `RenderContext`/`RenderMode` types. A complete worked
example is required by the success criteria.

The derive macros page must document both `FerroModel` (SeaORM builder scaffolding)
and `ValidateRules` (declarative `#[rule(...)]` validation) with at least one
complete usage example each.

**Primary recommendation:** Write both pages to match the existing documentation
style — prose introduction, Quick Start code block, section-by-section reference,
and a "Best Practices" or "Reference" table at the end. Mirror `themes.md` and
`validation.md` for tone and structure.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DOC-01 | Service Projections user documentation page created in docs/src/features/ | ServiceDef, derive_intents, JsonUiRenderer fully read from source; worked example from app/src/projections/ available |
| DOC-02 | FerroModel derive macro documented in user docs with examples | ferro-macros/src/model.rs and app/src/models/entities/ provide complete pattern |
| DOC-03 | ValidateRules derive macro documented in user docs with examples | ferro-macros/src/validate.rs shows all rule parsing; lib.rs shows all exported rule fns |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-projections` | workspace | ServiceDef, intent engine, renderer | Gated behind `projections` feature flag in framework crate |
| `ferro-macros` | workspace | FerroModel and ValidateRules proc macros | Always available — no feature gate |
| mdBook | current | Docs toolchain | Project uses this; SUMMARY.md is mdBook format |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `sea_orm` | workspace | FerroModel requires DeriveEntityModel from SeaORM | Only relevant in FerroModel docs |

## Architecture Patterns

### Recommended Project Structure

Docs live at:

```
docs/src/
├── features/
│   ├── projections.md     <- DOC-01 (new, missing)
│   └── validation.md      <- exists; ValidateRules/FerroModel will link or add new page
├── SUMMARY.md             <- must add links to new pages
```

The SUMMARY.md must be updated to include new pages. Both entries belong under
the `# Features` section. Projections gets its own entry. FerroModel and ValidateRules
can share one page (e.g., `derive-macros.md`) or be appended to `database.md` and
`validation.md` respectively — see Open Questions.

### Pattern 1: Existing Doc Page Structure

Every existing feature page follows this structure:

```
# Feature Name

[1-2 sentence description with key differentiator]

## Overview / How It Works (optional)
[Diagram or bullet explanation of the data flow]

## Quick Start
[Minimal working code block]

## Core Concepts / [Topic-by-topic sections]
[Subsections per concept with code blocks]

## Reference Table (often at end)
| Item | Description | Example |

## Best Practices (optional)
[Numbered list]
```

Source: observed in `themes.md`, `validation.md`, `json-ui.md`, `database.md`.

### Pattern 2: Code Imports Use `ferro::` (not `ferro_projections::`)

Per STATE.md decision from Phase 110: all import paths use `ferro::` crate root.
Projections types are exported from `ferro::` when the `projections` feature is active.

```rust
// Source: framework/src/lib.rs line 228-233, app/src/projections/order.rs line 1
use ferro::{
    ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef, StateDef, StateMachine, Transition,
};
```

The `projections` feature gate is an implementation detail — users get the types
when Ferro is configured for their project. Do not document the feature flag in
user-facing docs.

### Pattern 3: ServiceDef Builder API

All projection files in the app use the same builder chain style:

```rust
// Source: app/src/projections/order.rs
use ferro::{ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef, StateDef, StateMachine, Transition};

pub fn service_def() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("total", DataType::Float, FieldMeaning::Money)
        .state_machine(
            StateMachine::new("order_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("completed").final_state())
                .transition(Transition::new("draft", "complete", "completed"))
        )
        .guard(GuardDef::new("is_manager").display_name("Manager Approval Required"))
        .action(ActionDef::new("approve").precondition("is_manager"))
}
```

### Pattern 4: Intent Derivation → Rendering Pipeline

The three-stage pipeline is the core concept for the Projections page:

```
ServiceDef  →  derive_intents(&service_def)  →  IntentScore[]
                                                      ↓
JsonUiRenderer.render(&service_def, &intents, &ctx)  →  serde_json::Value
```

```rust
// Source: ferro-projections/src/render/json_ui.rs (doc comment example, lines 43-60)
use ferro::{ServiceDef, DataType, FieldMeaning, derive_intents, JsonUiRenderer, Renderer, RenderContext};

let product = ServiceDef::new("product")
    .display_name("Product")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("name", DataType::String, FieldMeaning::EntityName)
    .field("price", DataType::Float, FieldMeaning::Money);

let intents = derive_intents(&product);
let renderer = JsonUiRenderer;
let json = renderer.render(&product, &intents, &RenderContext::default()).unwrap();
// json["$schema"] == "ferro-json-ui/v1"
// json["components"] is non-empty
```

### Pattern 5: FerroModel Usage

The generated entities in `app/src/models/entities/` show the canonical usage:

```rust
// Source: app/src/models/entities/users.rs
use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

Generated API (from ferro-macros/src/model.rs):

```rust
// Create
let user = User::create()
    .set_name("Alice")
    .set_email("alice@example.com")
    .insert()
    .await?;

// Selective update
let updated = user.update()
    .set_name("Alice Smith")
    .save()
    .await?;

// Clear nullable field
let updated = updated.update()
    .clear_bio()
    .save()
    .await?;

// Delete
user.delete().await?;

// Query
let users = User::query()
    .filter(Column::Email.contains("@example.com"))
    .all()
    .await?;
```

### Pattern 6: ValidateRules Usage

```rust
// Source: ferro-macros/src/lib.rs lines 488-510
use ferro::ValidateRules;

#[derive(ValidateRules)]
struct CreateUserRequest {
    #[rule(required, email)]
    email: String,

    #[rule(required, min(8))]
    password: String,

    #[rule(required, integer, min(18))]
    age: Option<i32>,
}

// Usage
let request = CreateUserRequest { ... };
request.validate()?;
```

The `#[rule(...)]` attribute accepts any rule that's exported from the `ferro::` crate
root (required, email, min, max, string, integer, between, etc.).

### Anti-Patterns to Avoid

- **Using `ferro_projections::` imports in docs:** Always use `ferro::` per STATE.md Phase 110 decision
- **Documenting the `projections` feature flag to end users:** It's an infrastructure detail
- **Treating `#[rule(...)]` syntax like the `validator` crate:** These are different systems — `ValidateRules` uses Ferro's own Laravel-style rules, not `#[validate(...)]` from the `validator` crate

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Code examples for projections | Write them from scratch | Copy from `app/src/projections/` | Examples there are tested and correct |
| FerroModel CRUD examples | Write from scratch | Adapt from `app/src/models/entities/` | These are the canonical generated patterns |
| ValidateRules examples | Write from scratch | Use pattern from ferro-macros/src/lib.rs docstring | Source of truth is the macro's own inline docs |

## Common Pitfalls

### Pitfall 1: Missing SUMMARY.md Entry

**What goes wrong:** A new `.md` file exists but is not reachable because it's not
listed in `docs/src/SUMMARY.md`. mdBook will not render unreferenced pages.
**Why it happens:** Authors write the page and forget the nav entry.
**How to avoid:** Add the SUMMARY.md entry in the same task as creating the file.
**Warning signs:** Page exists but isn't reachable via docs nav.

### Pitfall 2: Wrong Import Style

**What goes wrong:** Examples show `use ferro_projections::ServiceDef` or
`use ferro::validation::Validator`.
**Why it happens:** Looking at internal source code instead of user-facing export paths.
**How to avoid:** All types come from `ferro::` root. See framework/src/lib.rs for
the canonical export list. Validation rules come from `ferro::{required, email, ...}`.
**Warning signs:** Import has a double-colon path with more than two segments.

### Pitfall 3: Conflating ValidateRules with validator crate

**What goes wrong:** Docs show `#[validate(email)]` instead of `#[rule(email)]`.
**Why it happens:** The existing `validation.md` documents the `validator` crate's
`#[validate]` attribute. `ValidateRules` is a different, Ferro-native macro using
`#[rule]`.
**How to avoid:** `ValidateRules` uses `#[rule(...)]` attribute and Ferro's own rule
functions. The `validator` crate uses `#[validate(...)]` and is still valid for
`#[request]` form structs. They are parallel systems.

### Pitfall 4: Confusing Intents With Actions

**What goes wrong:** Docs describe intents as "what the user can do" rather than
"what the service structurally IS."
**Why it happens:** The word "intent" has UX connotations.
**How to avoid:** Use the wording from the source code: intents answer "what IS this
service?" based on structural signals, not user capabilities. Actions (ActionDef) are
what users can do.

## Code Examples

### Intent Derivation (full pipeline)

```rust
// Source: ferro-projections/src/derive.rs derive_intents docstring (lines 63-74)
use ferro::{ServiceDef, DataType, FieldMeaning, derive_intents};

let product = ServiceDef::new("product")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("name", DataType::String, FieldMeaning::EntityName)
    .field("price", DataType::Float, FieldMeaning::Money);

let scores = derive_intents(&product);
// scores[0] is the primary intent with highest confidence
// scores[0].intent, scores[0].confidence, scores[0].matching_signals
```

### Intent Override

```rust
// Source: ferro-projections/src/intent.rs (IntentHint docs)
use ferro::{ServiceDef, DataType, FieldMeaning, Intent, IntentHint};

let service = ServiceDef::new("my_service")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .intent_hint(IntentHint::Primary(Intent::Process))   // force Process intent
    .intent_hint(IntentHint::Exclude(Intent::Browse));   // exclude Browse intent
```

### RenderContext Customization

```rust
// Source: ferro-projections/src/render/mod.rs (RenderContext struct, lines 29-50)
use ferro::{RenderContext, RenderMode};

let ctx = RenderContext {
    intent_index: 0,           // use primary intent
    current_state: Some("submitted".to_string()), // current workflow state
    mode: RenderMode::Input,   // render editable form
    templates: None,           // use default layouts
};
```

### FerroModel — Complete Entity

```rust
// Source: app/src/models/entities/users.rs (canonical generated pattern)
use ferro::FerroModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, FerroModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub author_id: i32,
    pub slug: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

### ValidateRules — Complete Example

```rust
// Source: ferro-macros/src/lib.rs lines 488-510 (ValidateRules docstring)
use ferro::ValidateRules;

#[derive(ValidateRules)]
struct RegistrationRequest {
    #[rule(required, email)]
    email: String,

    #[rule(required, min(8))]
    password: String,

    #[rule(required, string, min(2), max(50))]
    name: String,

    #[rule(required, integer, min(18), max(120))]
    age: i32,

    #[rule(nullable, url)]
    website: Option<String>,
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual SeaORM ActiveModel construction everywhere | `#[derive(FerroModel)]` scaffolds create/update/delete/query | Phase 84-93 | Remove ~80% of model CRUD boilerplate |
| `Validator::new(&data).rules(...)` for every struct | `#[derive(ValidateRules)]` with `#[rule(...)]` per field | Phase 84-93 | Validation co-located with type definition |
| Hardcoded JSON-UI component trees per route | `ServiceDef` + `derive_intents` + `JsonUiRenderer` | Phase 89 | Structural layout derived from data semantics |

## Open Questions

1. **FerroModel and ValidateRules: separate page or appended to existing pages?**
   - What we know: `database.md` already documents SeaORM model patterns without FerroModel; `validation.md` documents the fluent `Validator::new()` API without `ValidateRules`
   - What's unclear: Whether to add a new `derive-macros.md` page, append to existing pages, or create separate pages per macro
   - Recommendation: A single page `docs/src/features/derive-macros.md` covering both `FerroModel` and `ValidateRules` keeps them findable together. Alternatively, append a "FerroModel" section to `database.md` and a "ValidateRules" section to `validation.md`. Either satisfies DOC-02 and DOC-03. The planner should pick one approach and be consistent.

2. **Service Projections feature flag mention**
   - What we know: `projections` is an optional Cargo feature on the `framework` crate
   - What's unclear: Whether scaffolded apps have it enabled by default
   - Recommendation: Check ferro-cli scaffold templates for the default Cargo.toml to verify if projections feature is on by default. If yes, no mention needed. If not, a brief note in the "Getting Started" part of the projections page is appropriate.

## Validation Architecture

> `workflow.nyquist_validation` is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust native) |
| Config file | workspace `Cargo.toml` |
| Quick run command | `cargo test --all-features -p ferro-projections` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DOC-01 | `docs/src/features/projections.md` exists and is linked in SUMMARY.md | manual-only (file system check + visual review) | n/a | ❌ Wave 0 |
| DOC-02 | FerroModel documented with example | manual-only (content review) | n/a | ❌ Wave 0 |
| DOC-03 | ValidateRules documented with example | manual-only (content review) | n/a | ❌ Wave 0 |

All three requirements are documentation deliverables. There is no automated test that
verifies doc content quality. The verification step (`/gsd:verify-work`) is a human
review that checks the success criteria directly:
- DOC-01: file exists, SUMMARY.md has the link, the pipeline section and worked example are present
- DOC-02: page with FerroModel has `#[derive(FerroModel)]` example
- DOC-03: page with ValidateRules has `#[rule(...)]` example

### Wave 0 Gaps

None — no test infrastructure needed for documentation-only work. The existing `cargo
test --all-features` suite remains the full suite gate, and doc examples do not need
to be runnable (they use `ignore` or live in prose blocks, not `doctest` blocks).

## Sources

### Primary (HIGH confidence)
- `ferro-projections/src/lib.rs` — public exports, module structure
- `ferro-projections/src/service.rs` — ServiceDef builder API (all methods)
- `ferro-projections/src/derive.rs` — derive_intents function and signal system
- `ferro-projections/src/intent.rs` — Intent enum, IntentScore, IntentHint
- `ferro-projections/src/field.rs` — DataType, FieldMeaning, infer_meaning
- `ferro-projections/src/render/mod.rs` — Renderer trait, RenderContext, RenderMode
- `ferro-projections/src/render/json_ui.rs` — JsonUiRenderer with inline example
- `ferro-macros/src/model.rs` — FerroModel proc macro implementation
- `ferro-macros/src/validate.rs` — ValidateRules proc macro implementation
- `ferro-macros/src/lib.rs` — docstrings for both macros (canonical usage examples)
- `framework/src/lib.rs` lines 226-303 — all ferro:: export paths
- `app/src/projections/` — six real-world ServiceDef examples
- `app/src/models/entities/users.rs` — canonical FerroModel entity
- `docs/src/SUMMARY.md` — current nav structure
- `docs/src/features/*.md` — doc page style conventions

### Secondary (MEDIUM confidence)
- `.planning/REQUIREMENTS.md` — DOC-01/02/03 requirements text
- `.planning/STATE.md` — Phase 110 decision: ferro:: crate root imports everywhere

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all types verified directly from source code
- Architecture: HIGH — page structure verified from existing docs, import patterns from STATE.md and framework exports
- Pitfalls: HIGH — derived from direct source inspection and project decisions

**Research date:** 2026-03-26
**Valid until:** 2026-04-25 (docs do not change until framework changes)
