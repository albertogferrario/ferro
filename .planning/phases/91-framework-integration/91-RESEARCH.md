# Phase 91: Framework Integration — Research

**Researched:** 2026-03-01
**Domain:** Internal framework crate integration (ferro-projections → ferro)
**Confidence:** HIGH

<research_summary>
## Summary

Researched Ferro's established patterns for integrating sub-crates into the framework. Phase 91 wires ferro-projections (ServiceDef → IntentGraph → Renderer) into the framework crate, CLI, and MCP server.

The integration follows five established patterns already used by ferro-cache, ferro-events, ferro-notifications, ferro-broadcast, ferro-storage, ferro-json-ui, and ferro-lang. No new architectural patterns needed — this is a wiring phase.

**Primary recommendation:** Follow the existing feature-gated re-export pattern (`#[cfg(feature = "projections")]`), add handler helpers for projection rendering, scaffold CLI commands for ServiceDef generation, and add MCP introspection tools.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already built — ferro-projections)
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| ferro-projections | workspace | ServiceDef→Intent→Renderer pipeline | Phase 84-90 complete |
| schemars | 1.x | JSON Schema generation for all types | Integrated in Phase 85.1 |
| serde/serde_json | 1.x | Serialization for all types | Integrated in Phase 84 |
| thiserror | 1.0 | Error type derivation | Integrated in Phase 84 |

### Integration Points (framework crate)
| Component | Location | Purpose |
|-----------|----------|---------|
| Re-exports | `framework/src/lib.rs` | Public API surface |
| Handler helpers | `framework/src/http/` | Response helpers for projection rendering |
| Feature gate | `framework/Cargo.toml` | Optional `projections` feature |

### CLI Integration
| Component | Location | Purpose |
|-----------|----------|---------|
| `make:projection` | `ferro-cli/src/commands/` | Scaffold ServiceDef definition |
| Route scaffolding | `ferro-cli/src/commands/` | Generate projection endpoints |

### MCP Integration
| Component | Location | Purpose |
|-----------|----------|---------|
| `list_projections` | `ferro-mcp/src/tools/` | Inventory ServiceDefs |
| `inspect_projection` | `ferro-mcp/src/tools/` | View structure of a ServiceDef |
| `render_projection` | `ferro-mcp/src/tools/` | Call Renderer and return JSON-UI |

### No External Dependencies Needed
Phase 91 requires zero new external crates. All integration uses existing framework primitives.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: Feature-Gated Re-Export

All optional crate integrations in Ferro use Cargo feature gates. JSON-UI (`ferro-json-ui`) is the closest precedent.

**What:** Conditionally re-export ferro-projections types behind `#[cfg(feature = "projections")]`
**When to use:** Always — projections is opt-in functionality
**Precedent:** `framework/src/lib.rs` lines 63-77 (json-ui feature gate)

```rust
// framework/Cargo.toml
[features]
projections = ["dep:ferro-projections"]

[dependencies]
ferro-projections = { path = "../ferro-projections", optional = true }
```

```rust
// framework/src/lib.rs
#[cfg(feature = "projections")]
pub use ferro_projections::{
    derive_intents, infer_meaning,
    ActionDef, Cardinality, DataType, Error as ProjectionsError,
    FieldDef, FieldMeaning, GuardDef, InputDef, Intent, IntentHint,
    IntentScore, JsonUiRenderer, NavigationHint, RelationshipDef,
    RenderContext, RenderMode, Renderer, ServiceDef, StateDef,
    StateMachine, Transition, Warning as ProjectionsWarning,
};
```

### Pattern 2: Response Helper Functions

Framework provides ergonomic helpers that return `Response` (= `Result<HttpResponse, HttpResponse>`).

**What:** Helper function for rendering projections to JSON response
**Precedent:** `framework/src/http/mod.rs` lines 52-66 (`text()`, `json()`, `bytes()`)

```rust
// Projection rendering returns serde_json::Value, which maps directly to json()
// No new response type needed — json() already handles it
let renderer = JsonUiRenderer;
let intents = derive_intents(&service);
let output = renderer.render(&service, &intents, &ctx)?;
json_response!({ "projection": output })
```

### Pattern 3: Handler Parameter Extraction

Handlers use `FromRequest` trait for automatic parameter extraction from requests.

**What:** Extract ServiceDef or render parameters from request
**Precedent:** `framework/src/http/extract.rs` — `FromRequest` for Request, i32, String, etc.

```rust
#[handler]
pub async fn show_projection(req: Request, id: i32) -> Response {
    let service = build_user_service(); // ServiceDef constructed per-service
    let intents = derive_intents(&service);
    let ctx = RenderContext::default();
    let renderer = JsonUiRenderer;
    let output = renderer.render(&service, &intents, &ctx)?;
    Ok(HttpResponse::json(output))
}
```

### Pattern 4: CLI Scaffolding Command

CLI commands follow a module-per-command pattern with code generation via string templates.

**What:** `ferro make:projection <name>` generates a ServiceDef module
**Precedent:** `ferro-cli/src/commands/` — make_controller, make_model, make_api

### Pattern 5: MCP Tool Module

Each MCP tool is a separate module with `pub fn execute(project_root: &Path) -> Result<T>`.

**What:** Tools for introspecting and rendering projections
**Precedent:** `ferro-mcp/src/tools/` — application_info, list_models, json_ui_inspect

### Anti-Patterns to Avoid
- **Don't create a new response type for projections** — `serde_json::Value` from Renderer maps directly to `HttpResponse::json()`, which already exists
- **Don't make projections mandatory** — feature-gate like json-ui, not like core HTTP
- **Don't couple ServiceDef to database models** — ServiceDef is schema-only, constructed in application code, not derived from SeaORM models (that's Phase 93 field test territory)
- **Don't add projection-specific middleware** — rendering is a handler concern, not a request pipeline concern
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON response from Renderer | Custom response type | `HttpResponse::json(value)` | Renderer already returns serde_json::Value |
| Error conversion | Manual error mapping | `From<ferro_projections::Error> for HttpResponse` | Framework error conversion pattern |
| Intent derivation caching | Custom cache layer | Return `Vec<IntentScore>` directly | Derivation is fast (struct analysis), no I/O |
| ServiceDef persistence | Database storage | In-code construction | ServiceDef is schema definition, not runtime data |
| Route generation | Custom route macro | Existing `get!/post!` macros | Projection endpoints are standard HTTP routes |

**Key insight:** ferro-projections outputs `serde_json::Value` by design — it already integrates with Ferro's `HttpResponse::json()`. The integration is thin wiring, not complex adaptation.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Over-Engineering the Handler Layer
**What goes wrong:** Creating elaborate ServiceDef registries, projection middleware, or automatic route mounting
**Why it happens:** Treating projections like a framework subsystem rather than a library
**How to avoid:** ServiceDef is constructed in handler code, not registered globally. Keep it simple — build, derive, render, respond.
**Warning signs:** If you're adding anything to `bootstrap.rs`, you're over-engineering

### Pitfall 2: Coupling ServiceDef to SeaORM Models
**What goes wrong:** Auto-generating ServiceDef from database models at compile time
**Why it happens:** Tempting to DRY up field definitions
**How to avoid:** ServiceDef is intentionally separate from models — it describes business semantics, not database schema. Phase 93 may explore this, but Phase 91 should NOT.
**Warning signs:** If you're reading model derive macros or migration files, stop

### Pitfall 3: Feature Gate Dependency Chain
**What goes wrong:** `projections` feature accidentally depends on `json-ui` or vice versa
**Why it happens:** JsonUiRenderer outputs ferro-json-ui compatible JSON but shouldn't require the json-ui HTML renderer
**How to avoid:** `projections` feature should only depend on `ferro-projections` crate. The JSON output is self-contained — it follows json-ui schema but doesn't import json-ui types.
**Warning signs:** If `framework/Cargo.toml` feature gate has more than one dependency

### Pitfall 4: Forgetting Error Type Aliasing
**What goes wrong:** `Error` name collision between framework and projections
**Why it happens:** Both crates export `Error`
**How to avoid:** Re-export as `ProjectionsError` (established pattern: `Error as EventError`, `Error as QueueError`, etc.)
**Warning signs:** Compilation errors about ambiguous `Error` type
</common_pitfalls>

<code_examples>
## Code Examples

### Complete Handler Pattern
```rust
// Source: Derived from established Ferro handler + projection patterns
use ferro::{
    derive_intents, HttpResponse, JsonUiRenderer, RenderContext,
    RenderMode, Renderer, Response, ServiceDef,
};

fn user_service() -> ServiceDef {
    ServiceDef::new("user")
        .display_name("User")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("email", DataType::String, FieldMeaning::Email)
        .has_many("posts", "post")
}

#[handler]
pub async fn show(id: i32) -> Response {
    let service = user_service();
    let intents = derive_intents(&service);
    let renderer = JsonUiRenderer;
    let ctx = RenderContext::default(); // Display mode, primary intent
    let output = renderer.render(&service, &intents, &ctx)?;
    Ok(HttpResponse::json(output))
}

#[handler]
pub async fn edit(id: i32) -> Response {
    let service = user_service();
    let intents = derive_intents(&service);
    let renderer = JsonUiRenderer;
    let ctx = RenderContext {
        intent_index: 0,
        current_state: None,
        mode: RenderMode::Input,
    };
    let output = renderer.render(&service, &intents, &ctx)?;
    Ok(HttpResponse::json(output))
}
```

### Re-Export Pattern (framework/src/lib.rs)
```rust
// Source: Established pattern from ferro-cache, ferro-events, etc.
#[cfg(feature = "projections")]
pub use ferro_projections::{
    derive_intents, infer_meaning,
    ActionDef, Cardinality, DataType, Error as ProjectionsError,
    FieldDef, FieldMeaning, GuardDef, InputDef, Intent, IntentHint,
    IntentScore, JsonUiRenderer, NavigationHint, RelationshipDef,
    RenderContext, RenderMode, Renderer, ServiceDef, StateDef,
    StateMachine, Transition, Warning as ProjectionsWarning,
};
```

### Route Registration
```rust
// Source: Established route pattern from app/src/routes.rs
routes! {
    // ... existing routes ...
    group!("/projections", {
        get!("/:service", controllers::projection::show).name("projections.show"),
        get!("/:service/edit", controllers::projection::edit).name("projections.edit"),
    }),
}
```

### Error Conversion
```rust
// Source: Established pattern from framework/src/http/response.rs
impl From<ferro_projections::Error> for HttpResponse {
    fn from(err: ferro_projections::Error) -> Self {
        HttpResponse::json(serde_json::json!({
            "error": err.to_string(),
            "type": "projection_error"
        })).status(500)
    }
}

impl From<ferro_projections::Error> for FrameworkError {
    fn from(err: ferro_projections::Error) -> Self {
        FrameworkError::internal(err.to_string())
    }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

No external ecosystem changes relevant — Phase 91 is internal integration. All technology decisions were made in Phases 84-90.

| Decision | Made In | Still Current |
|----------|---------|---------------|
| serde_json::Value for renderer output | Phase 90 | Yes — framework-agnostic |
| schemars for JSON Schema | Phase 85.1 | Yes — 1.x stable |
| Builder pattern (consuming self) | Phase 84 | Yes — workspace convention |
| 7 intents + Custom | Phase 88 | Yes — validated in Phase 89 |

**No deprecated/outdated patterns to worry about.**
</sota_updates>

<open_questions>
## Open Questions

1. **Should `make:projection` scaffold a full module or a single function?**
   - What we know: CLI scaffolding commands (make:controller, make:model) generate full module files
   - What's unclear: Whether projections warrant a separate file or should live in existing controllers
   - Recommendation: Scaffold as a module file (`projections/user.rs`) that exports a `fn user_service() -> ServiceDef` — consistent with make:* pattern

2. **Should MCP `render_projection` call the renderer or just inspect the ServiceDef?**
   - What we know: MCP tools are read-only introspection; rendering is deterministic
   - What's unclear: Whether agents need rendered output or just ServiceDef structure
   - Recommendation: Both — `inspect_projection` for structure, `render_projection` for rendered JSON-UI output

3. **How should the sample app demonstrate projections?**
   - What we know: app/src/controllers/ has example handlers; app demonstrates framework features
   - What's unclear: Whether to add projections to existing controllers or create new ones
   - Recommendation: Add `controllers/projection.rs` with 2-3 ServiceDef examples showing different intents
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- `framework/src/lib.rs` — re-export patterns, feature gates, macro exports
- `framework/src/http/mod.rs` + `response.rs` — Response type, helpers, error conversion
- `ferro-projections/src/lib.rs` — complete public API surface
- `ferro-cli/src/commands/` — CLI scaffolding patterns
- `ferro-mcp/src/tools/` — MCP tool implementation patterns
- `app/src/` — sample app structure and usage patterns

### Secondary (MEDIUM confidence)
- None — all findings from direct source code analysis

### Tertiary (LOW confidence)
- None — internal codebase research, no external sources needed
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: ferro-projections internal crate (Phases 84-90)
- Ecosystem: Ferro framework integration points (re-exports, HTTP, CLI, MCP)
- Patterns: Feature-gated re-exports, handler helpers, CLI scaffolding, MCP tools
- Pitfalls: Over-engineering, model coupling, feature gate chains, error aliasing

**Confidence breakdown:**
- Standard stack: HIGH — all patterns observed in existing codebase
- Architecture: HIGH — follows established Ferro conventions exactly
- Pitfalls: HIGH — derived from known framework constraints
- Code examples: HIGH — constructed from verified source patterns

**Research date:** 2026-03-01
**Valid until:** 2026-04-01 (30 days — internal patterns are stable)
</metadata>

---

*Phase: 91-framework-integration*
*Research completed: 2026-03-01*
*Ready for planning: yes*
