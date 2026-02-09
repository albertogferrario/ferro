# Phase 26: Action System - Research

**Researched:** 2026-02-09
**Domain:** Server-driven UI action resolution — mapping JSON-UI action declarations to Ferro handler routes
**Confidence:** HIGH

<research_summary>
## Summary

Researched how the existing JSON-UI action schema connects to Ferro's routing and handler infrastructure. Phase 26 is internal framework plumbing: the action types (`Action`, `ConfirmDialog`, `ActionOutcome`) are fully defined in `ferro-json-ui/src/action.rs`; Ferro's named route registry (`register_route_name` / `route()`) already maps route names to URL patterns. The gap is the resolver that transforms `"users.store"` handler references into executable URLs with the correct HTTP method, plus builder ergonomics.

Server-driven UI systems (Airbnb Ghost Platform, Lyft Canvas) universally resolve actions server-side before sending JSON to the client. The action JSON should contain the resolved URL, not just the handler name — the client should never need to resolve route names. This is the key architectural insight: resolution happens at render time in the handler, not at action execution time in the frontend.

**Primary recommendation:** Build an `ActionResolver` that resolves `Action.handler` strings to URLs using Ferro's existing `route()` function at view render time. The resolved URL replaces or supplements the handler reference in the JSON sent to the client. Form submissions collect field values and POST to the resolved URL. Confirmation dialogs and outcomes are handled client-side based on the declarative JSON.
</research_summary>

<standard_stack>
## Standard Stack

No external libraries needed. This phase uses existing Ferro infrastructure:

### Core (Already Built)
| Component | Location | Purpose |
|-----------|----------|---------|
| `Action` struct | `ferro-json-ui/src/action.rs:68-80` | Action declaration with handler, method, confirm, outcomes |
| `ActionOutcome` enum | `ferro-json-ui/src/action.rs:52-65` | Post-action behavior (redirect, refresh, show_errors, notify) |
| `ConfirmDialog` struct | `ferro-json-ui/src/action.rs:32-39` | Pre-action confirmation |
| `HttpMethod` enum | `ferro-json-ui/src/action.rs:20-29` | HTTP method for the action request |
| `route()` function | `framework/src/routing/router.rs:98-107` | Resolves named route to URL with params |
| `register_route_name()` | `framework/src/routing/router.rs:74-81` | Route name → path registration |
| `ComponentNode.action` | `ferro-json-ui/src/component.rs:449-463` | Action attachment point on any component |
| `FormProps.action` | `ferro-json-ui/src/component.rs:169-176` | Required action for form components |
| `TableProps.row_actions` | `ferro-json-ui/src/component.rs:152-167` | Per-row actions on tables |

### Supporting (Already Built)
| Component | Location | Purpose |
|-----------|----------|---------|
| `JsonUiView` | `ferro-json-ui/src/view.rs:31-42` | View container with data + components |
| `JsonUi::render()` | `framework/src/json_ui/mod.rs:45-103` | HTML response renderer |
| `JsonUi::render_json()` | `framework/src/json_ui/mod.rs:109-116` | JSON response renderer |
| Request `input()` | `framework/src/http/request.rs:304-311` | Form data deserialization |
| `FormRequest` trait | `framework/src/http/form_request.rs:14-104` | Validated form handling |
| `SavedInertiaContext` | `framework/src/inertia/context.rs:27-56` | Context saving before request consumption |

### Nothing to Install
This phase is pure Rust framework code using existing dependencies (serde, serde_json, hyper).
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: Server-Side Action Resolution

**What:** Resolve handler names to URLs before sending JSON to the client. The client never resolves routes.

**When to use:** Always — this is the core pattern for Phase 26.

**How it works:**

1. Developer writes: `Action { handler: "users.store", method: Post, ... }`
2. At render time, resolver walks the component tree
3. For each `Action`, resolves `"users.store"` → `"/users"` using `route()`
4. Resolved URL is included in the JSON sent to the client
5. Client sends form data to the resolved URL with the specified HTTP method

**Why server-side:** The client (Phase 28 HTML renderer, or future JS client) should be a thin execution layer. Route resolution is a server concern — the client just needs a URL to POST to.

### Pattern 2: Action Resolution in the Render Pipeline

**What:** Resolution happens as part of `JsonUi::render()` / `JsonUi::render_json()`, not as a separate step.

**Rationale:** If resolution is separate, developers will forget to call it. Making it part of the render pipeline ensures all actions are always resolved.

**Approach:** The `JsonUi::render()` method already receives the view and data. It should resolve all actions in the component tree before serializing. This means either:
- (a) Mutating a clone of the view before serialization
- (b) Adding a `url` field to `Action` that gets populated during render
- (c) A `ResolvedAction` output type in the JSON separate from the schema type

Option (b) is cleanest: `Action` gains an optional `url` field that's `None` during construction and populated during render.

### Pattern 3: Row Action URL Templates

**What:** Table row actions need per-row URL generation with row data substitution.

**Example:** `"users.show"` with row `{id: 5}` → `"/users/5"`

**How:** Row actions include parameter bindings like `{id}` that map to column keys. During render, these become URL templates the client can resolve per-row, or the server pre-resolves for each data row.

For server-rendered HTML (Phase 28), the server resolves per-row. For JSON API consumers, a URL template like `/users/{id}` with a `params` mapping is more practical.

### Pattern 4: Form Action = Form Submission Target

**What:** `FormProps.action` defines where the form submits. The resolved URL becomes the form's `action` attribute, and `Action.method` becomes the form method (or `_method` override for PUT/PATCH/DELETE).

**HTML output (Phase 28):**
```html
<form action="/users" method="POST">
  <!-- fields -->
</form>

<form action="/users/5" method="POST">
  <input type="hidden" name="_method" value="PUT">
  <!-- fields -->
</form>
```

### Anti-Patterns to Avoid
- **Client-side route resolution:** The client should never need access to the route registry
- **Requiring manual URL construction:** `Action` should keep the `handler` reference for developer ergonomics; the URL is derived automatically
- **Eager resolution at Action construction time:** Resolution must happen at render time because the route registry is populated during app startup
- **Coupling resolution to a specific output format:** The resolver should work for both HTML rendering and JSON API output
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Route name → URL mapping | Custom registry | Existing `route()` function | Already handles param substitution |
| Route name → HTTP method mapping | Separate method registry | Existing `RouteInfo` from `get_registered_routes()` | Already tracks method per route for introspection |
| HTML form method override | Custom middleware | Standard `_method` hidden field pattern | Universal convention, Phase 28 renderer handles it |
| Form data parsing | Custom parser | Existing `req.input()` / `FormRequest` | Already handles JSON and form-urlencoded |
| Validation error display | Custom error format | Existing `Validator` → `ActionOutcome::ShowErrors` | Validation system already produces structured errors |

**Key insight:** The entire action execution pipeline (form submit → handler → response) already works via Ferro's standard HTTP handling. Phase 26 only needs to add the resolution layer that connects JSON-UI declarations to existing infrastructure.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Resolving Actions Before Route Registry is Ready
**What goes wrong:** `route("users.store", &[])` returns `None` because routes haven't been registered yet
**Why it happens:** Resolution called during app initialization, before `routes!` macro runs
**How to avoid:** Only resolve during request handling (inside handlers), never at startup
**Warning signs:** `None` URL on actions that have valid route names

### Pitfall 2: Missing Route Names
**What goes wrong:** Developer writes `handler: "users.store"` but the route isn't named
**How to avoid:** Validate during resolution — log warning or return error for unresolvable handlers
**Warning signs:** Silently `None` URLs in rendered JSON

### Pitfall 3: Parameter Mismatch in Row Actions
**What goes wrong:** Action references `"users.show"` (needs `{id}`) but no param binding provided
**Why it happens:** Table row action URL needs row data, but the connection isn't explicit
**How to avoid:** Row action params should map to column keys explicitly
**Warning signs:** URL contains literal `{id}` instead of substituted value

### Pitfall 4: Form Method Confusion
**What goes wrong:** HTML forms only support GET/POST natively. PUT/PATCH/DELETE need `_method` override
**Why it happens:** Forgetting that HTML `<form method>` doesn't support all HTTP methods
**How to avoid:** The renderer (Phase 28) must add `_method` hidden field for non-GET/POST methods
**Warning signs:** PUT/DELETE actions treated as POST without method override
</common_pitfalls>

<code_examples>
## Code Examples

### Existing Route Resolution
```rust
// Source: framework/src/routing/router.rs:98-107
pub fn route(name: &str, params: &[(&str, &str)]) -> Option<String> {
    let registry = ROUTE_REGISTRY.get()?.read().ok()?;
    let path_pattern = registry.get(name)?;

    let mut url = path_pattern.clone();
    for (key, value) in params {
        url = url.replace(&format!("{{{}}}", key), value);
    }
    Some(url)
}
```

### Current Action Schema
```rust
// Source: ferro-json-ui/src/action.rs:68-80
pub struct Action {
    pub handler: String,
    pub method: HttpMethod,
    pub confirm: Option<ConfirmDialog>,
    pub on_success: Option<ActionOutcome>,
    pub on_error: Option<ActionOutcome>,
}
```

### How Actions Attach to Components
```rust
// Source: ferro-json-ui/src/component.rs:449-463
pub struct ComponentNode {
    pub key: String,
    pub component: Component,
    pub action: Option<Action>,       // Any component can have an action
    pub visibility: Option<Visibility>,
}

// Forms have a required action
pub struct FormProps {
    pub action: Action,               // Where the form submits
    pub fields: Vec<ComponentNode>,
    pub method: Option<HttpMethod>,
}

// Tables have optional per-row actions
pub struct TableProps {
    pub columns: Vec<Column>,
    pub data_path: String,
    pub row_actions: Option<Vec<Action>>,  // Per-row actions
    // ...
}
```

### Handler → Form Submission Pattern (Already Works)
```rust
// Source: ferro-cli templates, verified pattern
pub async fn store(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);
    let form: CreateUserRequest = req.input().await?;
    // validate, process, respond...
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

Server-driven UI action resolution is a well-established pattern:

| Pattern | Status | Used By |
|---------|--------|---------|
| Server-side URL resolution | Standard practice | Airbnb Ghost, Lyft Canvas, all major SDUI |
| Declarative outcomes (redirect/refresh/notify) | Standard | Modern SDUI frameworks |
| Confirmation dialogs in action schema | Common | Enterprise SDUI, form builders |
| Form method spoofing (`_method`) | Universal convention | Laravel, Rails, Ferro (Inertia) |

No new tools or patterns needed. The approach Ferro's action schema already takes aligns with industry standard.
</sota_updates>

<open_questions>
## Open Questions

1. **Should `Action.url` be populated at render time or should a separate `ResolvedAction` exist?**
   - Adding `url: Option<String>` to `Action` is simplest — it's `None` during construction, `Some(...)` after resolution
   - A separate type avoids optional fields but adds complexity
   - Recommendation: Add `url` field to `Action` — simpler, works for both HTML and JSON output

2. **How should unresolvable actions be handled?**
   - Option A: Error at render time (strict)
   - Option B: Warning log, leave URL as None (lenient)
   - Option C: Pass handler string as-is to client (raw)
   - Recommendation: Option A for development, with a clear error message pointing to the missing route name

3. **Table row action params: template or pre-resolved?**
   - For JSON API: URL template (`/users/{id}`) with column key mapping is practical
   - For HTML render: Pre-resolved per row from data
   - Recommendation: Both — `url` field can contain template for JSON, resolver iterates data for HTML
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- `ferro-json-ui/src/action.rs` — Full action schema with tests
- `ferro-json-ui/src/component.rs` — ComponentNode, FormProps, TableProps with action fields
- `ferro-json-ui/src/view.rs` — JsonUiView structure
- `framework/src/routing/router.rs:74-107` — Route name registration and URL resolution
- `framework/src/json_ui/mod.rs` — Current render pipeline
- `framework/src/http/request.rs:272-311` — Form data parsing (input/json/form)
- `framework/src/http/form_request.rs` — FormRequest validation trait

### Secondary (MEDIUM confidence)
- Server-driven UI patterns from Airbnb Ghost Platform, Lyft Canvas — industry standard for action resolution
- JSON Forms (jsonforms.io) — form submission patterns for JSON-schema-driven UIs
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Ferro JSON-UI action resolution
- Ecosystem: Internal framework only (no external deps needed)
- Patterns: Server-side URL resolution, form submission, declarative outcomes
- Pitfalls: Route registry timing, param mismatches, form method spoofing

**Confidence breakdown:**
- Standard stack: HIGH — all infrastructure already exists in codebase
- Architecture: HIGH — server-side resolution is universal SDUI pattern
- Pitfalls: HIGH — derived from concrete code analysis of existing router
- Code examples: HIGH — directly from source code

**Research date:** 2026-02-09
**Valid until:** 2026-03-11 (30 days — internal framework, no external dependencies to go stale)
</metadata>

---

*Phase: 26-action-system*
*Research completed: 2026-02-09*
*Ready for planning: yes*
