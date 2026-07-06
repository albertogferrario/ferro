# Phase 41: API Resources Basics - Research

**Researched:** 2026-02-10
**Domain:** Rust model-to-response transformation (Laravel API Resources pattern)
**Confidence:** HIGH

<research_summary>
## Summary

Researched the Rust ecosystem for model-to-response transformation patterns equivalent to Laravel API Resources. The Rust ecosystem has **no existing crate** that replicates this feature set. The closest is `dto_mapper` (compile-time DTO generation with field selection) but it lacks runtime conditional fields, request context awareness, and collection support.

The standard Rust approach is separate response structs with `From<Model>` implementations — type-safe but verbose and repetitive. Serde's `skip_serializing_if` handles static conditional fields but cannot support request-context-dependent decisions (e.g., "include email only for admins").

Ferro already has strong patterns to build on: `FerroModel`, `InertiaProps`, and `ValidateRules` derive macros all use `syn` + `quote` proc macros. The API Resources feature should follow the same pattern — a `#[derive(ApiResource)]` macro that generates a `to_resource()` method with conditional field support via a builder/map approach.

**Primary recommendation:** Build a `Resource` trait + `#[derive(ApiResource)]` macro that generates field-selecting `to_resource(&self, &Request) -> serde_json::Value` methods. Use an explicit `ResourceMap` builder (not serde sentinel values) for conditional fields. This is novel in the Rust ecosystem and fits Ferro's macro-driven DX.
</research_summary>

<standard_stack>
## Standard Stack

### Core (Internal — No External Dependencies)
| Component | Purpose | Why This Approach |
|-----------|---------|-------------------|
| `Resource` trait | Core trait defining `to_resource(&self, req: &Request) -> Value` | Mirrors Laravel's `JsonResource::toArray()`, request-aware |
| `#[derive(ApiResource)]` macro | Generates `Resource` impl from struct attributes | Matches Ferro's `FerroModel`/`InertiaProps`/`ValidateRules` pattern |
| `ResourceMap` builder | Collects fields with conditional inclusion | Avoids serde sentinel hacks; explicit, type-safe field building |
| `ResourceValue` enum | `Value(serde_json::Value)` / `Missing` | Internal mechanism for conditional field removal |

### Supporting (Existing Dependencies)
| Library | Version | Purpose | Already In Use |
|---------|---------|---------|----------------|
| `serde` | workspace | Serialization of final output | Yes |
| `serde_json` | workspace | `Value` type for flexible JSON building | Yes |
| `syn` | workspace | Proc macro parsing | Yes (ferro-macros) |
| `quote` | workspace | Proc macro code generation | Yes (ferro-macros) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom `ResourceMap` builder | Manual `Serialize` impl per resource | Manual impl is more flexible but extremely verbose; macro + builder is DX-optimal |
| `ResourceValue::Missing` sentinel | `Option<Value>` + `skip_serializing_if` | Option-based only works for static conditions; Missing sentinel supports runtime request-context decisions |
| Proc macro generation | Runtime reflection (like serde_hooks) | Runtime adds overhead, limited to skip/rename; proc macro generates optimal code |
| `dto_mapper` crate | External DTO generation | Only does static field selection; no runtime conditions, no request context, no collections |

### No New Dependencies Required
All building blocks already exist in the Ferro workspace. This is a pure internal implementation using existing `syn`/`quote`/`serde`/`serde_json`.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Module Structure
```
framework/src/
├── http/
│   └── resources/
│       ├── mod.rs           # Resource trait, ResourceMap, ResourceValue
│       ├── resource.rs      # Resource trait definition
│       └── resource_map.rs  # ResourceMap builder with conditional methods
ferro-macros/src/
├── resource.rs              # #[derive(ApiResource)] proc macro
```

### Pattern 1: Resource Trait
**What:** Core trait that defines how a model transforms to a JSON response.
**When to use:** Every model-to-response transformation.
**Design:**
```rust
pub trait Resource {
    /// Transform this model into a JSON value for API responses.
    /// The request is available for context-dependent field selection.
    fn to_resource(&self, req: &Request) -> serde_json::Value;
}
```

### Pattern 2: ResourceMap Builder
**What:** A builder that collects field name-value pairs, supporting conditional inclusion. Produces a `serde_json::Value` (object) at the end.
**When to use:** Inside `to_resource()` implementations (generated or manual).
**Design:**
```rust
pub struct ResourceMap {
    fields: Vec<(String, ResourceValue)>,
}

enum ResourceValue {
    Value(serde_json::Value),
    Missing,
}

impl ResourceMap {
    pub fn new() -> Self { ... }

    /// Always include this field.
    pub fn field(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self { ... }

    /// Include field only when condition is true.
    pub fn when(mut self, key: &str, condition: bool, value: impl FnOnce() -> serde_json::Value) -> Self { ... }

    /// Include field only when condition is false.
    pub fn unless(mut self, key: &str, condition: bool, value: impl FnOnce() -> serde_json::Value) -> Self { ... }

    /// Conditionally merge multiple fields.
    pub fn merge_when(mut self, condition: bool, fields: impl FnOnce() -> Vec<(&str, serde_json::Value)>) -> Self { ... }

    /// Include field only if the Option is Some.
    pub fn when_some<T: Into<serde_json::Value>>(mut self, key: &str, value: &Option<T>) -> Self { ... }

    /// Finalize: strip Missing values, return JSON object.
    pub fn build(self) -> serde_json::Value { ... }
}
```

### Pattern 3: Derive Macro for Common Case
**What:** `#[derive(ApiResource)]` generates a `Resource` impl that includes annotated fields.
**When to use:** When the resource is a straightforward field selection from a model (80% case).
**Design:**
```rust
#[derive(ApiResource)]
#[resource(model = "user::Model")]
pub struct UserResource {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[resource(rename = "member_since")]
    pub created_at: String,
    #[resource(skip)]        // Never include (e.g., password hash)
    pub password: String,
}
```

Generates:
```rust
impl Resource for UserResource {
    fn to_resource(&self, _req: &Request) -> serde_json::Value {
        ResourceMap::new()
            .field("id", json!(self.id))
            .field("name", json!(self.name))
            .field("email", json!(self.email))
            .field("member_since", json!(self.created_at))
            // password skipped
            .build()
    }
}

impl From<user::Model> for UserResource {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            email: model.email,
            created_at: model.created_at,
            password: model.password,
        }
    }
}
```

### Pattern 4: Manual Resource for Complex Cases
**What:** Implement `Resource` trait manually when you need request-context-dependent logic.
**When to use:** When conditional fields depend on request state (auth, roles, query params).
**Design:**
```rust
pub struct UserResource {
    model: user::Model,
}

impl Resource for UserResource {
    fn to_resource(&self, req: &Request) -> serde_json::Value {
        let is_admin = req.user_is_admin(); // hypothetical

        ResourceMap::new()
            .field("id", json!(self.model.id))
            .field("name", json!(self.model.name))
            .when("email", is_admin, || json!(self.model.email))
            .when("created_at", is_admin, || json!(self.model.created_at))
            .merge_when(is_admin, || vec![
                ("role", json!("admin")),
                ("permissions", json!(["read", "write", "delete"])),
            ])
            .build()
    }
}
```

### Pattern 5: Handler Integration
**What:** Resources integrate with Ferro's response system so handlers can return them directly.
**When to use:** Every API endpoint returning a resource.
**Design:**
```rust
#[handler]
pub async fn show(req: Request, user: User) -> Response {
    let resource = UserResource::from(user);
    Ok(resource.to_response(&req))
    // Or with data wrapping:
    // Ok(resource.to_wrapped_response(&req))
    // → {"data": {"id": 1, "name": "...", ...}}
}
```

### Pattern 6: Data Wrapping
**What:** Configurable `{"data": ...}` envelope around resource output.
**When to use:** API responses following JSON:API conventions.
**Design:**
```rust
// On the Resource trait:
fn to_response(&self, req: &Request) -> HttpResponse {
    json_response!(self.to_resource(req))
}

fn to_wrapped_response(&self, req: &Request) -> HttpResponse {
    json_response!({"data": self.to_resource(req)})
}

// Or with additional metadata:
fn to_response_with(&self, req: &Request, additional: serde_json::Value) -> HttpResponse {
    let mut response = json!({"data": self.to_resource(req)});
    if let (Some(obj), Some(add)) = (response.as_object_mut(), additional.as_object()) {
        obj.extend(add.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    json_response!(response)
}
```

### Anti-Patterns to Avoid
- **Abusing serde for runtime conditions:** Thread-local context or `serde_hooks` for request-dependent serialization is fragile and opaque. Use explicit `ResourceMap` builder instead.
- **Generating serde `Serialize` impls with conditional logic:** The `Serialize` trait doesn't receive request context. Don't fight serde — build on top of it.
- **Inheritance-based resource hierarchies:** Rust doesn't have inheritance. Use trait composition and struct embedding instead.
- **Returning raw models from API endpoints:** Leaks internal schema, includes sensitive fields (password hashes), couples DB schema to API contract.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON value construction | Custom JSON string building | `serde_json::json!()` macro | Handles escaping, nesting, type conversion correctly |
| Field name case conversion | Manual string manipulation | `serde(rename_all)` or existing `SerdeCase` in ferro-cli | Already battle-tested in type generator |
| Proc macro attribute parsing | Raw `syn` attribute iteration | `darling` crate or Ferro's existing patterns | Reduces boilerplate, handles error reporting |
| Response envelope (`{"data": ...}`) | Per-handler wrapping | Trait method on `Resource` | Consistency across all resource responses |

**Key insight:** The transformation logic itself must be custom-built (no Rust crate does this), but the building blocks — JSON construction, serialization, proc macro tooling — are all mature and already in the workspace. The novel work is the `Resource` trait design, `ResourceMap` builder, and `ApiResource` derive macro.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Fighting Serde Instead of Building On Top
**What goes wrong:** Trying to make `serde::Serialize` handle runtime conditional fields leads to thread-local hacks or complex custom serializers.
**Why it happens:** Serde is designed for compile-time-known serialization shapes. Runtime conditions don't fit its model.
**How to avoid:** Build the `ResourceMap` → `serde_json::Value` pipeline separately. Only use serde for the final `Value` → JSON string step (which it handles perfectly).
**Warning signs:** Reaching for `serialize_with`, thread-local state, or `serde_hooks` for request-dependent logic.

### Pitfall 2: Over-Engineering the Derive Macro
**What goes wrong:** Trying to handle every Laravel feature in the macro (conditional fields, nested resources, collections) makes it complex and brittle.
**Why it happens:** Wanting feature parity with Laravel from day one.
**How to avoid:** The derive macro handles the 80% case (static field selection + `From<Model>`). Complex cases use manual `Resource` trait implementation with `ResourceMap` builder. This mirrors how Ferro's `#[handler]` macro handles common cases while allowing manual `FromRequest` for complex extraction.
**Warning signs:** Macro attribute syntax becoming as complex as the code it generates.

### Pitfall 3: Forgetting Request Context in Design
**What goes wrong:** Designing `to_resource()` without `&Request` parameter, then retrofitting it when conditional-on-auth fields are needed.
**Why it happens:** The simple case (static field selection) doesn't need the request, so it's tempting to omit.
**How to avoid:** Include `&Request` in the `Resource` trait signature from day one. The derive macro can generate `_req` for cases that don't use it.
**Warning signs:** Resources that can't conditionally include fields based on who's requesting.

### Pitfall 4: Breaking TypeScript Type Generation
**What goes wrong:** API Resources produce JSON that doesn't match the TypeScript types generated by `ferro generate:types`.
**Why it happens:** Resources transform fields (rename, skip, compute) but the type generator reads the original Rust struct.
**How to avoid:** Either (a) make the derive macro generate type hints for the type generator, or (b) have Resources derive their own TypeScript-compatible types. This should be considered in design but can be deferred to Phase 42 or later.
**Warning signs:** Frontend TypeScript errors about missing/unexpected fields from API responses.

### Pitfall 5: Not Supporting `From<Model>` Ergonomics
**What goes wrong:** Users have to manually construct resource structs field-by-field in every handler.
**Why it happens:** Forgetting to generate `From<Model>` alongside the Resource implementation.
**How to avoid:** The `#[derive(ApiResource)]` macro should generate both `Resource` impl AND `From<Model>` impl when `#[resource(model = "...")]` is specified.
**Warning signs:** Boilerplate-heavy handler code that defeats the purpose of the abstraction.
</common_pitfalls>

<code_examples>
## Code Examples

### Basic Resource Definition (Derive Macro)
```rust
// Source: Ferro design (novel — no existing Rust equivalent)
use ferro::{ApiResource, Resource};

#[derive(ApiResource)]
#[resource(model = "entities::users::Model")]
pub struct UserResource {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[resource(rename = "member_since")]
    pub created_at: String,
}
// password, remember_token, updated_at excluded by not listing them
```

### Using a Resource in a Handler
```rust
use ferro::{handler, Request, Response};

#[handler]
pub async fn show(req: Request, user: entities::users::Model) -> Response {
    let resource = UserResource::from(user);
    Ok(resource.to_response(&req))
}

// With data wrapping: {"data": {"id": 1, ...}}
#[handler]
pub async fn show_wrapped(req: Request, user: entities::users::Model) -> Response {
    let resource = UserResource::from(user);
    Ok(resource.to_wrapped_response(&req))
}
```

### Manual Resource with Conditional Fields
```rust
use ferro::{Resource, ResourceMap, Request};
use serde_json::json;

pub struct AdminUserResource {
    model: entities::users::Model,
}

impl AdminUserResource {
    pub fn new(model: entities::users::Model) -> Self {
        Self { model }
    }
}

impl Resource for AdminUserResource {
    fn to_resource(&self, req: &Request) -> serde_json::Value {
        let is_admin = req.session().get::<bool>("is_admin").unwrap_or(false);

        ResourceMap::new()
            .field("id", json!(self.model.id))
            .field("name", json!(self.model.name))
            .field("email", json!(self.model.email))
            .when("created_at", is_admin, || json!(self.model.created_at))
            .when("updated_at", is_admin, || json!(self.model.updated_at))
            .merge_when(is_admin, || vec![
                ("role", json!("admin")),
                ("last_login", json!(self.model.last_login_at)),
            ])
            .build()
    }
}
```

### ResourceMap Builder Pattern
```rust
// Source: Ferro design — inspired by Laravel's MissingValue mechanism
use serde_json::{json, Value, Map};

pub struct ResourceMap {
    fields: Vec<(String, ResourceValue)>,
}

enum ResourceValue {
    Present(Value),
    Missing,
}

impl ResourceMap {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field(mut self, key: &str, value: Value) -> Self {
        self.fields.push((key.to_string(), ResourceValue::Present(value)));
        self
    }

    pub fn when(mut self, key: &str, condition: bool, value: impl FnOnce() -> Value) -> Self {
        if condition {
            self.fields.push((key.to_string(), ResourceValue::Present(value())));
        }
        // When false: field is simply not added (no sentinel needed)
        self
    }

    pub fn merge_when(mut self, condition: bool, fields: impl FnOnce() -> Vec<(&str, Value)>) -> Self {
        if condition {
            for (key, value) in fields() {
                self.fields.push((key.to_string(), ResourceValue::Present(value)));
            }
        }
        self
    }

    pub fn when_some<T: serde::Serialize>(mut self, key: &str, value: &Option<T>) -> Self {
        if let Some(v) = value {
            self.fields.push((key.to_string(), ResourceValue::Present(json!(v))));
        }
        self
    }

    pub fn build(self) -> Value {
        let mut map = Map::new();
        for (key, value) in self.fields {
            if let ResourceValue::Present(v) = value {
                map.insert(key, v);
            }
        }
        Value::Object(map)
    }
}
```

### Data Wrapping with Additional Metadata
```rust
// Source: Laravel's additional() pattern adapted for Rust
#[handler]
pub async fn show(req: Request, user: entities::users::Model) -> Response {
    let resource = UserResource::from(user);
    Ok(resource.to_response_with(&req, json!({
        "meta": {
            "version": "v1",
            "deprecated_fields": []
        }
    })))
}
// Output: {"data": {"id": 1, ...}, "meta": {"version": "v1", ...}}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw model serialization | Separate response structs + `From` | Ongoing Rust convention | Type safety, decoupled API contract |
| `serde_hooks` for runtime field control | Explicit builder patterns | 2024+ | Cleaner, no serde abuse |
| `dto_mapper` for static DTOs | Still current for simple cases | 2023+ | Good for static selection, insufficient for conditional |

**New tools/patterns to consider:**
- **`darling` crate for proc macro attribute parsing:** More ergonomic than raw `syn` attribute parsing. Used by many popular derive macros. Could simplify `#[derive(ApiResource)]` implementation.
- **`serde_json::Map` for ordered field output:** Using `Map<String, Value>` preserves insertion order, ensuring field order in JSON output matches resource definition order.

**Deprecated/outdated:**
- **Thread-local serialization context:** Armin Ronacher's 2021 pattern of using thread-local storage for serde context is now widely considered an anti-pattern. Explicit context passing (our `&Request` parameter) is preferred.
- **`serde_hooks` for conditional fields:** The crate itself suggests "you likely wouldn't need this." Direct `Value` construction is preferred.
</sota_updates>

<open_questions>
## Open Questions

1. **TypeScript type generation for Resources**
   - What we know: Current type generator parses `#[derive(InertiaProps)]` structs. Resources define a different field shape than the underlying model.
   - What's unclear: Should `ApiResource` structs also generate TypeScript types? Or is that only needed for Inertia props?
   - Recommendation: Defer to Phase 42 or later. API consumers typically use OpenAPI/Swagger for types, not generated TypeScript. If needed, the `ApiResource` derive could emit type hints similar to `InertiaProps`.

2. **Nested Resources in Phase 41 vs 42**
   - What we know: Phase 41 is "basics" (field selection), Phase 42 is "advanced" (relationships, pagination, collections).
   - What's unclear: Should a resource be able to nest another resource in Phase 41? E.g., `UserResource` containing `ProfileResource`.
   - Recommendation: Support it naturally — since resources produce `serde_json::Value`, nesting is just `resource.to_resource(req)` as a field value. No special machinery needed.

3. **Resource naming convention**
   - What we know: Laravel uses `UserResource`, `PostResource` suffix convention.
   - What's unclear: Where should resource files live? `src/resources/`? `src/http/resources/`?
   - Recommendation: Follow Ferro convention — `src/resources/` in user apps, with CLI `make:resource` scaffolding. Framework internals in `framework/src/http/resources/`.

4. **Simplified ResourceMap vs MissingValue sentinel**
   - What we know: Laravel uses `MissingValue` sentinel objects that are stripped during `filter()`. Our `when()` can simply not add the field.
   - What's unclear: Are there cases where we need the sentinel pattern (field must exist in the map but be removed later)?
   - Recommendation: Start with the simple approach (don't add field when condition is false). The sentinel pattern is only needed if we support merge operations where field position matters — unlikely in Phase 41. Can add later if needed.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Ferro codebase exploration — handler patterns, FerroModel/InertiaProps/ValidateRules derive macros, type generation, framework public API
- [Laravel 12.x Eloquent: API Resources](https://laravel.com/docs/12.x/eloquent-resources) — full feature set, toArray(), conditional fields, collections, pagination
- [Serde Field Attributes](https://serde.rs/field-attrs.html) — skip_serializing_if, rename, flatten
- [Serde Manual Serialize Implementation](https://serde.rs/impl-serialize.html) — custom serialization for dynamic fields

### Secondary (MEDIUM confidence)
- [dto_mapper crate](https://github.com/douggynix/dto_mapper) — closest existing Rust DTO generation, verified on crates.io
- [serde_with skip_serializing_none](https://docs.rs/serde_with/latest/serde_with/attr.skip_serializing_none.html) — bulk Option field handling
- [Armin Ronacher: Abusing Serde](https://lucumr.pocoo.org/2021/11/14/abusing-serde/) — thread-local context pattern (as anti-pattern reference)
- [Axum IntoResponse trait](https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html) — Rust web framework response patterns

### Tertiary (LOW confidence — needs validation)
- [serde_hooks](https://github.com/anatols/serde_hooks) — runtime serialization hooks (niche, limited adoption)
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust trait-based model-to-response transformation
- Ecosystem: serde, serde_json, syn, quote, dto_mapper, serde_hooks, serde_with
- Patterns: Resource trait, builder pattern, derive macro generation, From<Model> conversion
- Pitfalls: Serde abuse for runtime conditions, over-engineering macros, missing request context

**Confidence breakdown:**
- Standard stack: HIGH — internal implementation with existing dependencies, no new crates needed
- Architecture: HIGH — patterns match Ferro's existing FerroModel/InertiaProps/ValidateRules macro approach
- Pitfalls: HIGH — based on direct analysis of Rust ecosystem gaps and Laravel implementation details
- Code examples: HIGH — designed from Ferro's existing patterns and Laravel's proven API

**Research date:** 2026-02-10
**Valid until:** 2026-03-10 (30 days — Ferro internal patterns are stable)
</metadata>

---

*Phase: 41-api-resources-basics*
*Research completed: 2026-02-10*
*Ready for planning: yes*
