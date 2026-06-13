# Phase 212: CRUD Handler Proc Macros — Research

**Researched:** 2026-06-13
**Domain:** Proc-macro authoring in `ferro-macros`, framework validation + tenant + action layers
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Tenant resolved via existing `ferro::current_tenant()` / `TenantScopeProvider` — no new resolver trait.
- **D-02:** Optional `tenant = "<expr>"` escape-hatch macro arg. Default = `current_tenant()`.
- **D-03:** `TenantScoped` trait with assoc `Id: FromStr` + `find_for_tenant(id, tenant_id) -> Result<Option<Self>, _>`.
- **D-04:** Optional `find = "<path::fn>"` macro arg overrides the trait for models that don't fit.
- **D-05:** `on_miss = "/url"` → redirect on miss; omitted → generic 404 (`Response::not_found()` for GET, `ActionError::not_found(...)` for POST). No consumer-specific styling in `ferro-macros`.
- **D-06:** `#[resource_get]` emits `#[ferro::handler]` internally; `#[resource_post]` emits `#[ferro::action]`.
- **D-07:** `Validator::validate_or_redirect(self, data: &Value, url) -> Result<(), ActionError>` composing `with_old_input` + `into_action_error`.
- **D-08:** `{param}` placeholders in URL args resolve from extracted path params; unknown placeholder = compile error.
- **D-09:** Typed params as real function parameters; body in named inner fn; rustdoc `cargo expand` walkthroughs.
- **D-10:** Requirements are CRUD-01..CRUD-06 (see CONTEXT.md).

### Claude's Discretion

- Exact `TenantScoped` trait method names / assoc-type bounds beyond `Id: FromStr`.
- Whether `path = "{id:i64}"` or a positional resource-type arg drives id extraction.
- Whether the gestiscilo thread-local bridge is documentation-only or a tiny adapter.

### Deferred Ideas (OUT OF SCOPE)

- `#[confirm_page]` macro (no recurring shape found).
- Multipart upload macros.
- gestiscilo Phase 202b consumer adoption.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRUD-01 | `#[resource_get]` folds typed-param + tenant resolution + tenant-scoped lookup + 404-on-miss; tenant/resource as real typed params | D-06, D-01/D-02, D-03/D-04, D-05, D-09 — all composition surfaces verified below |
| CRUD-02 | `#[resource_post]` folds same prelude + validation-failure redirect envelope | D-06, D-07, D-08 — action composition + validator surface verified |
| CRUD-03 | `Validator::validate_or_redirect(&data, &url) -> Result<(), ActionError>` composing existing chain | `Validator::validate()` at validator.rs:158; `with_old_input` at error.rs:98; `into_action_error` at error.rs:160 — all verified |
| CRUD-04 | `TenantScoped` trait + `find =` override; tenant via existing `current_tenant()`, `tenant =` escape hatch | `current_tenant()` at context.rs:32 returns `Option<TenantContext>` — sync, verified; async-trait precedent in framework verified |
| CRUD-05 | IDE experience: typed params, named inner fn, rustdoc `cargo expand` walkthroughs, verified | inner-fn pattern and trybuild infra verified (trybuild dev-dep in ferro-macros Cargo.toml) |
| CRUD-06 | Macros exported via `ferro` facade; reference fixture; CHANGELOG + version bump | facade re-export pattern at lib.rs:329-342 verified; trybuild pass/fail structure verified |
</phase_requirements>

---

## Summary

Phase 212 ships two attribute proc macros (`#[resource_get]`, `#[resource_post]`) plus one validator helper method (`validate_or_redirect`). All three reuse exclusively existing framework surfaces — the macro crate has no new logic, only codegen composition.

**The load-bearing design question** is how the macro learns the concrete tenant type and resource type. Because `current_tenant()` returns `Option<TenantContext>` (the framework's fixed concrete type), the tenant binding in the user's signature must be `tenant: &TenantContext`. For the resource type, the macro reads the type from the user's typed parameter declaration (e.g. `customer: &Customer`) and emits a `TenantScoped::find_for_tenant(id, tenant.id).await?` call using that type. No macro arg needed for the common case — the type is inferred from the parameter.

**AFIT vs `async_trait`**: the framework uses `async_trait` throughout for async trait methods (Rust edition 2021, MSRV 1.88). The new `TenantScoped` trait should follow the same pattern (`#[async_trait]`) for consistency, even though AFIT is stable in 1.75+. This avoids the boxing concern being invisible to consumers — `async_trait` is already re-exported from `ferro` (`framework/src/lib.rs:268`).

**Macro composition strategy**: `#[resource_get]`/`#[resource_post]` do NOT call `handler_impl`/`action_impl` at the Rust function level. They call them at the token level — the generated output includes `#[::ferro::handler]` or `#[::ferro::action(...)]` as an outer attribute on the generated wrapper fn. This is the standard multi-layer attribute pattern; it works because proc-macro attributes are applied in source order and the outer macro fully rewrites the item. The `handler`/`action` codegen does not need to be refactored.

**Primary recommendation:** Implement `#[resource_get]` / `#[resource_post]` as attribute macros in a new `ferro-macros/src/resource_get.rs` and `ferro-macros/src/resource_post.rs` following the exact parse patterns in `action.rs`. Add `validate_or_redirect` as a plain method on `Validator` in `framework/src/validation/validator.rs`. Define `TenantScoped` in `framework/src/tenant/` and export it through the `ferro` facade alongside the macros.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tenant resolution | API / Backend middleware | — | `TenantMiddleware` sets task-local context; `current_tenant()` reads it. The macro emits a call to this existing layer. |
| Resource lookup (DB) | API / Backend | — | `TenantScoped::find_for_tenant` is an async DB query. Lives in model layer, called from generated handler code. |
| Path param extraction | API / Backend | — | `Request::param_as<T>` at request.rs:177 is a synchronous parse from already-decoded route params. |
| Validation + redirect envelope | API / Backend | — | `Validator` + `ValidationError` + `ActionError` — all framework types; generated code emits calls to them. |
| Macro codegen | Compile-time (`ferro-macros`) | — | Proc-macro crate; generates the handler/action wrapper at compile time. |
| Facade re-export | API / Backend (`framework`) | — | `framework/src/lib.rs` — the single public surface consumers import from. |

---

## Standard Stack

### Core (all already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `syn` | 2.x (full + parsing features) | Parse `TokenStream` into AST | Already in `ferro-macros/Cargo.toml` [VERIFIED: Cargo.toml] |
| `quote` | 1.x | Generate `TokenStream2` from templates | Already in ferro-macros [VERIFIED: Cargo.toml] |
| `proc-macro2` | 1.x | Token manipulation | Already in ferro-macros [VERIFIED: Cargo.toml] |
| `async-trait` | 0.1 | `#[async_trait]` on `TenantScoped` | Already in `framework/Cargo.toml` [VERIFIED: grep] |
| `trybuild` | 1.x | Compile-pass / compile-fail UI tests | Already a dev-dep in `ferro-macros/Cargo.toml` [VERIFIED: Cargo.toml] |

No new dependencies. All required crates are already in the workspace.

### Installed Parse Pattern (no darling, manual only)
`ferro-macros` uses **manual `syn` parsing** throughout — no `darling`. The `action.rs` parser uses `syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated` to iterate over `Meta::NameValue` entries. The new macros must follow the same pattern for consistency. [VERIFIED: ferro-macros/src/action.rs:76-134]

---

## D-01 / D-02: Tenant Resolution Surface

### `current_tenant()` signature
```rust
// framework/src/tenant/context.rs:32 [VERIFIED]
pub fn current_tenant() -> Option<TenantContext>
```

- Synchronous (no `.await`). Reads a `tokio::task_local!` `Arc<RwLock<Option<TenantContext>>>` via `try_with`.
- Returns `None` when called outside `TenantMiddleware` scope.
- `TenantContext` is a concrete struct: `{ id: i64, slug: String, name: String, plan: Option<String>, ... }` [VERIFIED: framework/src/tenant/mod.rs:46]
- Re-exported as `ferro::current_tenant` [VERIFIED: framework/src/lib.rs:133]

### Generated tenant resolution code (default path, D-01)

```rust
// Generated by the macro (quote! sketch)
let __tenant: ::ferro::TenantContext = ::ferro::current_tenant()
    .ok_or_else(|| ::ferro::FrameworkError::domain(
        "No tenant context. Ensure this route is behind TenantMiddleware.", 400
    ))?;
let #tenant_pat: &#tenant_ty = &__tenant;
```

The macro reads the user's parameter name (e.g. `tenant`) and type (e.g. `TenantContext`) from the fn signature. Since `current_tenant()` always returns `TenantContext`, the generated binding is always `&TenantContext` — the type annotation in the user's signature must match or the compiler rejects it. This is the correct behavior.

### D-02: `tenant = "expr"` escape hatch

When the user writes `#[resource_get(Customer, tenant = "resolve_tenant().await")]`, the macro emits:
```rust
let __tenant: ::ferro::TenantContext = { resolve_tenant().await }
    .map_err(|e| ::ferro::ActionError::msg(e.to_string()))?;
```
The `tenant = "..."` value is emitted verbatim as a Rust expression in a block — the consumer is responsible for the type matching `TenantContext` (or whatever type their `tenant:` parameter declares).

### gestiscilo's `resolve_tenant()` bridge

`resolve_tenant()` in gestiscilo returns `Result<Business, HttpResponse>`, where `Business` is gestiscilo's own model — not `TenantContext`. The `tenant = "resolve_tenant().await"` escape hatch handles this case: the user declares `tenant: &Business` in the signature and provides the expr. No ferro-side adapter is needed. **Documentation-only bridge** is the correct answer — the escape hatch suffices. [VERIFIED: gestiscilo dashboard.rs:77]

---

## D-03 / D-04: `TenantScoped` Trait

### Design
```rust
// To live in framework/src/tenant/scoped.rs (new file)
#[async_trait]
pub trait TenantScoped: Sized + Send + Sync {
    /// The type of the resource's primary key.
    type Id: std::str::FromStr + Send;

    /// Look up one record owned by `tenant_id`.
    ///
    /// Returns `Ok(None)` when not found (triggers 404 / redirect-on-miss).
    /// Returns `Err` on infrastructure failures (DB error, etc.).
    async fn find_for_tenant(
        id: Self::Id,
        tenant_id: i64,
    ) -> Result<Option<Self>, ::ferro::FrameworkError>;
}
```

- `tenant_id: i64` matches `TenantContext.id: i64` [VERIFIED: tenant/mod.rs:48]
- `FrameworkError` is the return error type — consistent with `RouteBinding::from_route_param` [VERIFIED: database/route_binding.rs:63]
- `#[async_trait]` used (not AFIT) — consistent with every async trait in `framework/src/` [VERIFIED: grep over framework/src/]
- Trait lives in `framework` (not `ferro-orm`) — it's a lookup contract tied to the tenant layer, not a general ORM primitive

### D-04: `find = "expr"` override

When `find = "Customer::lookup"` is provided, the macro emits:
```rust
let __resource = Customer::lookup(__resource_id, __tenant.id).await
    .map_err(|e| /* 500 shape */)?;
```
The expr must be a path to a function with signature `async fn(Id, i64) -> Result<Option<R>, E>` where `E: Into<ActionError>` (or the user handles errors themselves). This is permissive — the generated code emits a `.map_err` that converts to the miss/error arms.

---

## D-05: 404 / Miss Strategy

### `#[resource_get]` miss path
`Response` is `Result<HttpResponse, HttpResponse>` [VERIFIED: http/response.rs:16]. A 404 response is:
```rust
// Generated code for on_miss omitted:
return Err(::ferro::HttpResponse::new().status(404));
```
There is **no `HttpResponse::not_found()` constructor** — the framework does not expose one as a named method on `HttpResponse`. The generated code uses `.status(404)` directly, which is idiomatic ferro. [VERIFIED: grep over framework/src/http/response.rs — no `fn not_found`]

For `on_miss = "/url"`, the miss becomes:
```rust
return Err(::ferro::HttpResponse::new()
    .status(302)
    .header("Location", "/url"));
// Or equivalently: return ::ferro::Redirect::to("/url").into();
```
`Redirect::to(path)` is the cleaner option [VERIFIED: http/response.rs:272].

### `#[resource_post]` miss path
`ActionError::not_found(msg)` exists and sets `kind: ActionKind::NotFound` [VERIFIED: http/action.rs:133]:
```rust
// Generated: on_miss omitted
return Err(::ferro::ActionError::not_found("Resource not found"));
// Generated: on_miss = "/url"
return Err(::ferro::ActionError::not_found("Resource not found")
    .redirect_to("/url"));
```
`ActionError::redirect_to` is a builder [VERIFIED: http/action.rs:172].

---

## D-06: Macro Composition — How to Emit `#[handler]`/`#[action]`

### The canonical pattern

The approach is to emit the nested attribute as a token in the output `quote!`. This is the standard multi-attribute composition pattern for proc macros in Rust:

```rust
// In resource_get.rs (quote! sketch)
let output = quote! {
    #[::ferro::handler]           // <-- emitted as outer attr on the generated fn
    #fn_vis async fn #fn_name(__ferro_req: ::ferro::Request) -> ::ferro::Response {
        // ... prelude (tenant resolve + resource lookup) ...
        // body delegates to inner fn:
        __#fn_name_inner(__ferro_req_ref, &__tenant, &__resource).await
    }

    async fn #inner_fn_name(
        #req_pat: &mut ::ferro::Request,
        #tenant_pat: &#tenant_ty,
        #resource_pat: &#resource_ty,
    ) -> ::ferro::Response {
        #fn_block
    }
};
```

When `rustc` processes the output, it sees `#[::ferro::handler]` on the generated wrapper fn and applies `handler_impl` to it. Since the generated wrapper already has the canonical `(__ferro_req: ferro::Request) -> Response` shape, `handler_impl` processes it without further extraction (the single `Request` param is passed through). This avoids any double-extraction issue.

**Risk**: `handler_impl` classifies the `Request` param as `ParamKind::Request` and emits `let req = __ferro_req;` (move). Since the inner fn takes `&mut Request`, the generated wrapper passes `&mut __ferro_req` (not ownership). This means the wrapper fn generated by `resource_get` must NOT declare `req: Request` in its signature — it handles the request internally. The user's original `req:` parameter is satisfied by the inner-fn argument. [VERIFIED: handler.rs:86-113, utils.rs:156-163]

**Alternative (simpler)**: generate the full handler wrapper directly, inlining what `handler_impl` would produce, without re-emitting `#[::ferro::handler]`. This avoids the double-attribute complexity. Given that `handler_impl` for a single `Request` param is just a thin shell (rename to `__ferro_req`, emit body), inlining is straightforward. **Recommended approach**: inline the handler/action boilerplate directly rather than emitting a nested attribute. The `action_impl` output shape is documented in `action.rs:17-28` and is ~10 lines to replicate.

### `#[resource_post]` must replicate the action wrapper shape

The generated fn must:
1. Declare `(__ferro_req: ::ferro::Request) -> ::ferro::Response`
2. Rebind as `let mut __ferro_req = __ferro_req;`
3. Emit the `__action_result: ActionResult = async move { __inner_body }.await;`
4. Call `::ferro::http::action::handle_action_result(__action_result, redirect_to_lit, concat!(...), &mut __ferro_req)`

[VERIFIED: action.rs:256-278]

---

## D-07: `validate_or_redirect` on `Validator`

### Exact composition
```rust
// Add to framework/src/validation/validator.rs

impl<'a> Validator<'a> {
    /// Validate and, on failure, flash per-field errors + old input and return
    /// an [`ActionError`] redirecting to `url`.
    ///
    /// Composes the existing `with_old_input` + `into_action_error` chain:
    ///
    /// ```ignore
    /// // Before:
    /// if let Err(e) = validator.validate() {
    ///     return Err(e.with_old_input(&data).into_action_error(&form_url));
    /// }
    ///
    /// // After:
    /// Validator::new(&data)
    ///     .rules(...)
    ///     .validate_or_redirect(&data, &form_url)?;
    /// ```
    pub fn validate_or_redirect(
        self,
        data: &serde_json::Value,
        url: impl Into<String>,
    ) -> Result<(), crate::http::action::ActionError> {
        self.validate()
            .map_err(|e| e.with_old_input(data).into_action_error(url))
    }
}
```

**Exact types, verified:**
- `Validator::validate(self) -> Result<(), ValidationError>` — consumes `self` [VERIFIED: validator.rs:158]
- `ValidationError::with_old_input(mut self, data: &serde_json::Value) -> Self` — takes `&Value` [VERIFIED: error.rs:98]
- `ValidationError::into_action_error(self, url: impl Into<String>) -> ActionError` — exact signature [VERIFIED: error.rs:160]
- The composition compiles: `validate()` → `map_err(|e| e.with_old_input(data).into_action_error(url))` → `Result<(), ActionError>`

**Import path for `ActionError` in `validate_or_redirect`**: `crate::http::action::ActionError` — same path used by `into_action_error` at error.rs:160.

---

## D-08: Form-URL `{param}` Placeholder Synthesis

### What the macro knows at compile time

The macro signature is `#[resource_get(Customer, on_miss = "/url/{id}")]`. The macro arg provides the resource type. It does NOT know the route path (`/customers/{id}/edit`) at macro expansion time — the route is registered separately by the router.

**Approach:** The macro extracts the param name from the user's resource parameter (e.g. `customer: &Customer` → param name `customer`, and the `TenantScoped::Id` type is extracted from the trait impl or assumed to be `i64`). For URL synthesis, the macro reads the user-declared params in the fn signature (e.g. `id: i64` or inferred from `customer`'s declared type) and replaces `{id}` or `{customer}` in the URL string at codegen time using `format!`.

**Concrete resolution rule:**
- The macro parses URL strings in all three args (`on_miss`, `redirect_to`, `form_url`) for `{name}` placeholders.
- Recognized names: the resource param name (e.g. `customer`) and any additional primitive params in the signature (future extension).
- At codegen time, emit: `let __on_miss_url = format!("/dashboard/clienti/{}", __resource_id);`
- An unrecognized `{xyz}` placeholder emits `compile_error!("unknown path param {xyz} in on_miss — declared params are: ...")`.

**At parse time**, the macro knows `__resource_id` (the extracted ID) so the format works. The ID extraction happens before the URL synthesis code.

[VERIFIED: D-08 analysis consistent with `Request::param_as` at request.rs:177 and the attr-parser pattern in action.rs]

---

## D-09: IDE Experience — Inner Function Pattern

### Pattern
The user's body moves into a `__fn_name_inner` async fn with real typed params:

```rust
// User writes:
#[resource_get(Customer, on_miss = "/dashboard/clienti")]
pub async fn edit(req: Request, tenant: &TenantContext, customer: &Customer) -> Response {
    // user body
}

// Macro generates (simplified):
pub async fn edit(__ferro_req: ::ferro::Request) -> ::ferro::Response {
    // ... prelude ...
    __edit_inner(&mut __ferro_req, &__tenant, &__customer).await
}

async fn __edit_inner(
    req: &mut ::ferro::Request,       // user's `req` param, real type
    tenant: &::ferro::TenantContext,  // user's `tenant` param, real type
    customer: &Customer,              // user's `customer` param, real type
) -> ::ferro::Response {
    // user's original body here
}
```

rust-analyzer can jump to `__edit_inner` and sees typed params. The user's body is unchanged. The only difference is that `req` is `&mut Request` in the inner fn (same as `#[action]`'s pattern).

### Trybuild infrastructure
- `trybuild = "1"` is already a dev-dep in `ferro-macros/Cargo.toml` [VERIFIED]
- Existing pattern: `tests/action_macro.rs` + `tests/ui/action/pass/*.rs` + `tests/ui/action/fail/*.rs` [VERIFIED]
- New test file: `tests/resource_macro.rs` with matching `tests/ui/resource/pass/` and `tests/ui/resource/fail/` directories
- Pattern for updating .stderr snapshots: `TRYBUILD=overwrite cargo test -p ferro-macros --test resource_macro`

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Async trait methods | Custom polling / hand-impl Future | `#[async_trait]` | Already standard in framework; avoids AFIT boxing surprise |
| Attribute parsing | Hand-roll token iterator | `syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated` | Already the pattern in action.rs:76 |
| Action result dispatch | Re-implement 303 redirect logic | `ferro::http::action::handle_action_result` | Already handles flash, same-origin check, log, back-compat query |
| Validation error redirect | Two-line discard+redirect idiom | `ValidationError::into_action_error(url)` | Already in framework/error.rs:160; `validate_or_redirect` is a thin wrapper |
| Tenant lookup failure | Custom error response | `FrameworkError::domain("...", 400)?` | Consistent with existing `TenantContext::from_request` pattern |

**Key insight:** This phase generates NO new runtime logic. Every arm of the generated code calls an existing framework function. The macro is pure codegen glue.

---

## Common Pitfalls

### Pitfall 1: Double-Attribute Explosion
**What goes wrong:** Emitting `#[::ferro::handler]` on the generated fn AND having `handler_impl` re-extract params causes duplicate param bindings, leading to "variable already declared" or unexpected shadowing.
**Why it happens:** `handler_impl` scans ALL params and emits `let x = ...` for each. If the generated fn also has `__ferro_req`, `handler_impl` emits `let req = __ferro_req;` again.
**How to avoid:** Inline the handler/action boilerplate directly (D-06 analysis). Do not emit `#[::ferro::handler]` on the outer fn. The generated fn already has the `(__ferro_req: Request) -> Response` shape.
**Warning signs:** Compiler error "use of moved value" or "cannot find value `__ferro_req` in this scope" after initial impl.

### Pitfall 2: Tenant Type Mismatch
**What goes wrong:** User writes `tenant: &Business` (gestiscilo's type) but the macro emits `current_tenant()` which returns `TenantContext`. The binding fails to compile.
**Why it happens:** `current_tenant()` is hardcoded to return `Option<TenantContext>`. The macro cannot coerce types.
**How to avoid:** The `tenant = "expr"` escape hatch must be used when the tenant type differs from `TenantContext`. Document this prominently. The generated code should use the *declared type* from the user's parameter — not a hardcoded `TenantContext` — when the escape hatch is active.
**Warning signs:** Type mismatch error on the `let #tenant_pat: &#tenant_ty = &__tenant;` binding.

### Pitfall 3: Borrow After Move in `#[resource_post]`
**What goes wrong:** `#[action]` keeps `__ferro_req` alive by binding `req` as `&mut Request`. If `resource_post` tries to use `let req = __ferro_req;` (move), `handle_action_result(&mut __ferro_req)` will fail.
**Why it happens:** The action runtime needs `&mut __ferro_req` after the body returns.
**How to avoid:** Follow `action.rs` pattern exactly: bind `req` as `&mut ::ferro::Request = &mut __ferro_req` in the inner fn invocation. [VERIFIED: action.rs:165-170]
**Warning signs:** "use of moved value `__ferro_req`" at the `handle_action_result` call.

### Pitfall 4: `async_trait` + `TenantScoped` in consumer crate
**What goes wrong:** Consumer forgets `#[async_trait]` on their `impl TenantScoped for Customer`, causing a "method is not `async`" error or a missing return type.
**Why it happens:** `async_trait` is required on both the trait def AND each impl.
**How to avoid:** Document in rustdoc. Consider making the derive macro emit `#[::ferro::async_trait]` automatically via a `#[derive(TenantScoped)]` or a clear example.
**Warning signs:** Compile error mentioning "the `async` keyword is not allowed in trait definitions" or future type mismatch.

### Pitfall 5: `{param}` Placeholder at Wrong Scope
**What goes wrong:** User writes `on_miss = "/clienti/{name}"` where `name` is not a path param but a field on the resource. The macro cannot access it at URL synthesis time (param extraction happens only for path params).
**Why it happens:** The macro only knows path params (extracted from the route and fn signature), not resource fields.
**How to avoid:** The `compile_error!` on unknown placeholders (D-08) catches this at compile time. Document that only path-param names are valid placeholders.

---

## Code Examples

### `#[resource_get]` — full expansion sketch (CRUD-01)

```rust
// User writes:
#[resource_get(Customer, on_miss = "/dashboard/clienti/{id}")]
pub async fn edit(req: Request, tenant: &TenantContext, customer: &Customer) -> Response {
    let data = json!({ "customer": customer });
    JsonUi::render_file("src/views/clienti/modifica.json", data)
}

// Macro generates (approximately):
pub async fn edit(__ferro_req: ::ferro::Request) -> ::ferro::Response {
    let mut __ferro_req = __ferro_req;
    let __ferro_params = __ferro_req.params().clone();

    // Step 1: extract resource ID (param name inferred from resource param name "customer")
    let __resource_id: i64 = {
        let __v = __ferro_params.get("id")  // or "customer" — planner decides
            .ok_or_else(|| ::ferro::HttpResponse::new().status(400))?;
        __v.parse().map_err(|_| ::ferro::HttpResponse::new().status(400))?
    };

    // Step 2: tenant resolution (D-01)
    let __tenant: ::ferro::TenantContext = ::ferro::current_tenant()
        .ok_or_else(|| ::ferro::HttpResponse::new()
            .status(400)
            .set_body("No tenant context"))?;

    // Step 3: resource lookup (D-03)
    let __resource_opt = <Customer as ::ferro::TenantScoped>::find_for_tenant(
        __resource_id, __tenant.id
    ).await.map_err(|e| ::ferro::HttpResponse::new().status(500))?;

    // Step 4: miss handling (D-05)
    let __resource = match __resource_opt {
        Some(r) => r,
        None => {
            let __miss_url = format!("/dashboard/clienti/{}", __resource_id);
            return Err(::ferro::HttpResponse::new()
                .status(302)
                .header("Location", &__miss_url));
        }
    };

    // Step 5: delegate to inner fn with user's typed params
    __edit_inner(&mut __ferro_req, &__tenant, &__resource).await
}

async fn __edit_inner(
    req: &mut ::ferro::Request,
    tenant: &::ferro::TenantContext,
    customer: &Customer,
) -> ::ferro::Response {
    // --- original user body ---
    let data = json!({ "customer": customer });
    JsonUi::render_file("src/views/clienti/modifica.json", data)
    // --- end user body ---
}
```

### `#[resource_post]` — full expansion sketch (CRUD-02)

```rust
// User writes:
#[resource_post(Customer,
    redirect_to = "/dashboard/clienti",
    form_url = "/dashboard/clienti/{id}/modifica")]
pub async fn save(req: Request, tenant: &TenantContext, customer: &Customer) -> ActionResult {
    let data = json!({ "name": "test" });
    Validator::new(&data).rules("name", rules![required()])
        .validate_or_redirect(&data, /* form_url */)?;
    Ok(())
}

// Macro generates (abbreviated):
pub async fn save(__ferro_req: ::ferro::Request) -> ::ferro::Response {
    let mut __ferro_req = __ferro_req;
    let __ferro_params = __ferro_req.params().clone();

    // Steps 1-4 identical to resource_get...
    // (id extraction, tenant resolution, TenantScoped lookup, miss handling)

    // Step 5: form_url synthesis
    let __form_url = format!("/dashboard/clienti/{}/modifica", __resource_id);

    // Step 6: delegate to inner fn
    let __action_result: ::ferro::ActionResult =
        async move {
            __save_inner(&mut __ferro_req, &__tenant, &__resource, &__form_url).await
        }.await;

    ::ferro::http::action::handle_action_result(
        __action_result,
        "/dashboard/clienti",   // redirect_to literal
        concat!(module_path!(), "::", stringify!(save)),
        &mut __ferro_req,
    )
}

async fn __save_inner(
    req: &mut ::ferro::Request,
    tenant: &::ferro::TenantContext,
    customer: &Customer,
    __form_url: &str,       // synthesized, passed in — not a user param
) -> ::ferro::ActionResult {
    // NOTE: __form_url is available as a hidden binding for validate_or_redirect
    // The user can use it explicitly: .validate_or_redirect(&data, __form_url)
    let data = json!({ "name": "test" });
    Validator::new(&data).rules("name", rules![required()])
        .validate_or_redirect(&data, __form_url)?;
    Ok(())
}
```

**Open design question for planner:** How does the user reference `__form_url` inside their body? Options:
1. Pass `form_url` as a hidden named parameter (e.g. `form_url: &str`) injected by the macro — visible in autocomplete but not in the user's written signature.
2. Bind it as a `let form_url = ...` statement before invoking the inner fn, and make the inner fn take it.
3. The user always passes the URL explicitly to `validate_or_redirect` and `form_url` is only used for the `?` auto-redirect if `#[resource_post]` handles validation itself (not left to the user).

The CONTEXT.md sketch shows the user calling `.validate_or_redirect(&data, /* form_url synthesised by macro */)` — suggesting option 1 or 2. **Planner should decide.** Option 2 (hidden `__form_url` binding passed as an extra inner-fn param) is recommended — clean, no new user-facing concept.

---

## Architecture Patterns

### Recommended File Layout
```
ferro-macros/src/
├── resource_get.rs     # #[resource_get] impl (new)
├── resource_post.rs    # #[resource_post] impl (new)
└── lib.rs              # two new #[proc_macro_attribute] registrations

framework/src/
├── tenant/
│   ├── scoped.rs       # TenantScoped trait (new file; export from tenant/mod.rs)
│   └── mod.rs          # pub use scoped::TenantScoped;
└── validation/
    └── validator.rs    # validate_or_redirect method added
```

### Attribute Parser Pattern (from action.rs:76-134)
```rust
// The canonical parser for k = "v" style macro args in ferro-macros:
fn parse_resource_get_attrs(attr: TokenStream) -> Result<ResourceGetAttrs, syn::Error> {
    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let metas = syn::parse::Parser::parse(parser, attr)?;
    for meta in metas {
        match meta {
            Meta::NameValue(nv) => { /* match key, extract Lit::Str */ }
            Meta::Path(p) => { /* positional resource type arg (first arg = type) */ }
            other => return Err(syn::Error::new_spanned(other, "only k=\"v\" or Type supported")),
        }
    }
    // ...
}
```
The first positional arg (`Customer` in `#[resource_get(Customer, ...)]`) arrives as `Meta::Path`. Extract it as the resource type token stream for use in `<#resource_ty as ::ferro::TenantScoped>` calls.

### Anti-Patterns to Avoid
- **Emitting `#[ferro::handler]` on the generated fn**: causes double-extraction. Inline the wrapper shape.
- **Hardcoding `TenantContext` in the generated binding**: breaks the `tenant =` escape hatch. Always use `#tenant_ty` extracted from the user's signature.
- **Inventing new error types**: use `FrameworkError::domain` and `ActionError::not_found` — existing constructors.
- **Adding a `tenant_id` method to models**: that's a trait-inheritance smell. The macro passes `tenant.id` directly.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | trybuild 1.x (already dev-dep in ferro-macros) |
| Config file | none — trybuild uses Cargo.toml dev-deps |
| Quick run command | `cargo test -p ferro-macros --test resource_macro` |
| Full suite command | `cargo test --all-features -p ferro-macros && cargo test --all-features -p ferro-rs` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRUD-01 | `#[resource_get]` compiles with typed params | compile-pass | `cargo test -p ferro-macros --test resource_macro` | ❌ Wave 0 |
| CRUD-01 | `#[resource_get]` 404-on-miss behavior | unit (with mock TenantScoped) | `cargo test -p ferro-rs validation::tests` | ❌ Wave 0 |
| CRUD-02 | `#[resource_post]` compiles with redirect_to | compile-pass | `cargo test -p ferro-macros --test resource_macro` | ❌ Wave 0 |
| CRUD-03 | `validate_or_redirect` returns Ok on passing validation | unit | `cargo test -p ferro-rs validation::validator::tests` | ❌ Wave 0 |
| CRUD-03 | `validate_or_redirect` returns Err(ActionError) on failure | unit | `cargo test -p ferro-rs validation::validator::tests` | ❌ Wave 0 |
| CRUD-04 | `TenantScoped` trait impl compiles | compile-pass | `cargo test -p ferro-macros --test resource_macro` | ❌ Wave 0 |
| CRUD-05 | `cargo expand` output includes typed inner fn params | manual (cargo expand) | `cargo expand --test resource_macro` | ❌ Wave 0 |
| CRUD-06 | macros importable via `ferro::resource_get` | compile-pass | fixture test in pass/ | ❌ Wave 0 |

### Compile-fail tests needed (trybuild)
| Failure Case | File | Captures |
|-------------|------|---------|
| `#[resource_post]` missing `redirect_to` | `fail/resource_post_missing_redirect_to.rs` | `compile_error!` message |
| `{xyz}` placeholder not in declared params | `fail/resource_get_unknown_placeholder.rs` | `compile_error!` message |
| `#[resource_get]` on non-async fn | `fail/resource_get_not_async.rs` | `compile_error!` message |

### Wave 0 Gaps
- [ ] `ferro-macros/tests/resource_macro.rs` — test harness file (mirrors action_macro.rs)
- [ ] `ferro-macros/tests/ui/resource/pass/minimal_get.rs` — minimal compile-pass fixture
- [ ] `ferro-macros/tests/ui/resource/pass/minimal_post.rs`
- [ ] `ferro-macros/tests/ui/resource/fail/` — compile-fail fixtures + .stderr snapshots
- [ ] `framework/src/validation/validator.rs` — `validate_or_redirect` unit tests (add to existing `mod tests`)

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | yes | Flash data uses existing `session_mut` infrastructure — no new session writes |
| V4 Access Control | yes | Tenant isolation via `find_for_tenant(id, tenant_id)` — records not owned by tenant return None (miss path) |
| V5 Input Validation | yes | Path param parsing via `FromStr` on `TenantScoped::Id`; `validate_or_redirect` uses existing Validator |
| V6 Cryptography | no | — |

### Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Cross-tenant resource access (IDOR) | Information Disclosure | `TenantScoped::find_for_tenant(id, tenant_id)` — DB query includes `AND tenant_id = ?`; miss returns 404/redirect |
| Open redirect via `on_miss` / `redirect_to` | Elevation of Privilege | `is_same_origin` check already in `ActionError::redirect_override` path (action.rs:249) and ValidationError chain; `on_miss` URL is a string literal — compile-time known, not user input |
| Session flash injection | Tampering | Delegated to `ValidationError::flash_into_session` + `handle_action_result` — existing vetted code path; T-180-01/T-180-02/T-180-03 mitigations apply |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The resource param name in the fn signature (e.g. `customer`) is also the path param name (e.g. `{customer}` or `{id}` in the route) | D-08 | Planner must decide the param-name convention: either the resource param name IS the path param, or a separate `id = "id"` macro arg is needed. Low risk — compile error surfaces it. |
| A2 | Inner fn approach does not break rustdoc generation for the outer fn | D-09 | rustdoc shows `__edit_inner` as a private fn — this is acceptable for the IDE experience goal. If the outer fn has no useful doc, rustdoc is less useful. Risk: minor. |
| A3 | `TenantScoped` lives in `framework` (not `ferro-orm`) | D-03/D-04 | If `ferro-orm` is the right home (it handles ORM primitives), a crate dependency edge is added from `framework` to `ferro-orm`. Currently `framework` does not depend on `ferro-orm`. LOW risk — `TenantScoped` can stay in `framework/src/tenant/` without any new dep. |

**Non-assumed claims**: all major API signatures, return types, and composition paths were verified by reading source files in this session.

---

## Open Questions

1. **Path param name convention for the resource ID**
   - What we know: `Request::param_as("id")` is the path param extraction. The macro must know what name to use.
   - What's unclear: Is the path param always named `"id"`, or is it the resource param name (`"customer"`, `"booking"`, etc.)? The CONTEXT.md sketch shows `customer: &Customer` but the URL placeholder is `{id}`.
   - Recommendation: Planner should decide between (a) always use `"id"` as the path param name convention, (b) infer from the resource param name, or (c) add an `id = "id"` macro arg. Option (a) is simplest and consistent with most REST conventions.

2. **`form_url` binding visibility in user body**
   - What we know: D-08 says `{param}` placeholders synthesize at codegen time. The user writes `validate_or_redirect(&data, /* form_url */)` inside the body.
   - What's unclear: The synthesized URL string must be accessible in the user's body. Whether it's a hidden `let __form_url = ...` binding or an extra inner-fn parameter named `form_url:` is a planner choice.
   - Recommendation: Planner introduces `form_url` as an invisible extra parameter to the inner fn (injected after the user's declared params) — consistent with D-09 (user doesn't write it, macro injects it).

3. **`TenantScoped::Id` default type**
   - What we know: the assoc type is `Id: FromStr`. Most gestiscilo entities use `i64`.
   - What's unclear: whether the macro needs a `#[resource_get(Customer, id_type = "i64")]` override or always infers from the `TenantScoped::Id` assoc type.
   - Recommendation: Always infer from the trait impl. The macro emits `<Customer as TenantScoped>::Id` as the parse target. No extra macro arg needed.

---

## Sources

### Primary (HIGH confidence)
- `ferro-macros/src/action.rs` — complete action macro source, parse pattern, code generation shape [VERIFIED in session]
- `ferro-macros/src/handler.rs` — handler macro source, param extraction pattern [VERIFIED]
- `ferro-macros/src/utils.rs` — ParamKind, classify_param_type, generate_extraction [VERIFIED]
- `framework/src/validation/validator.rs` — Validator API, validate() signature [VERIFIED]
- `framework/src/validation/error.rs` — ValidationError, with_old_input, into_action_error signatures [VERIFIED]
- `framework/src/http/action.rs` — ActionError, ActionResult, handle_action_result, ActionKind::NotFound [VERIFIED]
- `framework/src/http/request.rs:177` — param_as<T: FromStr> [VERIFIED]
- `framework/src/tenant/context.rs:32` — current_tenant() signature (sync, returns Option<TenantContext>) [VERIFIED]
- `framework/src/tenant/mod.rs` — TenantContext struct (id: i64, slug, name, plan) [VERIFIED]
- `ferro-macros/Cargo.toml` — syn/quote/proc-macro2 versions, trybuild dev-dep [VERIFIED]
- `framework/src/lib.rs` — facade re-export pattern for macros and tenant types [VERIFIED]

### Secondary (MEDIUM confidence)
- `gestiscilo-it/app/src/controllers/dashboard.rs:77` — resolve_tenant() shape (returns `Result<Business, HttpResponse>`, async) [VERIFIED via direct read]
- `gestiscilo-it/app/.planning/phases/202-adopt-ferro-crud-macros/202-EVIDENCE.md` — consumer duplication counts, pattern shapes [VERIFIED via direct read]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates already in workspace, versions verified
- Architecture: HIGH — all composition surfaces verified by reading source; one open question (path param naming) flagged for planner
- Pitfalls: HIGH — derived from actual code structure and known `#[action]` constraints

**Research date:** 2026-06-13
**Valid until:** 2026-07-13 (stable layer — proc-macro / validation APIs change infrequently)
