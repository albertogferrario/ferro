---
phase: 212-crud-handler-proc-macros
milestone: v13.1
status: scoped — discuss-phase needed before plan
created: 2026-06-12
created_by: gestiscilo Phase 202 evidence-gathering pass
renumbered: "2026-06-12 — relocated from 209 to 212 (new milestone v13.1) to resolve a phase-number collision: ferro ROADMAP already owns 209 for COMP-01 Gestiscilo Migration"
paired_consumer_phase: gestiscilo-it/app Phase 202 (see 202-EVIDENCE.md §8)
---

# Phase 212 — CRUD handler proc macros

Design proc macros for the recurring "GET form" + "POST handler" CRUD shapes that ferro framework consumers write today as 5–15 lines of boilerplate per handler. Two macros in scope, one previously-proposed macro explicitly dropped.

## Motivation

ferro is the product, not internal scaffolding for any one consumer. The first consumer (gestiscilo) hit a duplication wall in its operator controllers — 55+ tenant-resource lookups, 88+ form-error redirects, 21+ raw param-parse ladders. The existing ferro APIs that gestiscilo Phase 202 will adopt (`Request::param_as<T>`, `ValidationError::into_action_error`) already close ~75% of the boilerplate, but each handler still writes:

```rust
let business = resolve_tenant().await.map_err(|_| ActionError::msg("Sessione scaduta"))?;
let id: i64 = req.param_as("id").map_err(|_| error_response(404, "ID non valido"))?;
let resource = Resource::find_for_tenant(id, business.id).await?.ok_or_else(...)?;
```

That 3-line prelude repeats 200+ times across a single consumer. A proc macro can fold it into a route attribute. Other ferro consumers — current and future — write the same shape; the macros benefit the framework's whole consumer surface, not just gestiscilo's local LoC.

The macros do not save gestiscilo enough lines on their own to be worth a ferro phase if scoped only to gestiscilo's savings. They ARE worth a ferro phase scoped to ferro's framework-product axis.

## What's in scope

### Macro 1 — `#[resource_get]`

Wraps a GET handler that displays a single tenant-scoped resource (edit form, detail page, confirm page).

Generated prelude:
- Path param extraction (typed) — macro takes `path = "{id:i64}"` or similar
- Caller-supplied tenant resolver (configurable — defaults to a `TenantResolver` trait the consumer implements)
- Resource lookup via a `TenantScoped` trait the consumer implements on its model
- 404 dispatch on miss — uses caller-provided `on_miss` URL or Response builder

Sketch:

```rust
#[ferro::resource_get(Customer, on_miss = "/dashboard/clienti")]
pub async fn edit(req: Request, tenant: &Tenant, customer: &Customer) -> Response {
    // user body — tenant + customer already resolved, no preamble
}
```

Expanded form (approximate):

```rust
#[ferro::handler]
pub async fn edit(req: Request) -> Response {
    let tenant = req.tenant().await.map_err(...)?;
    let id: i64 = req.param_as("id").map_err(|_| ferro::dashboard_404("/dashboard/clienti"))?;
    let customer = Customer::find_for_tenant(id, tenant.id)
        .await
        .map_err(|e| ferro::dashboard_500(e))?
        .ok_or_else(|| ferro::dashboard_404("/dashboard/clienti"))?;
    edit_inner(req, &tenant, &customer).await
}
async fn edit_inner(req: Request, tenant: &Tenant, customer: &Customer) -> Response {
    // original user body
}
```

### Macro 2 — `#[resource_post]`

Wraps a POST handler that mutates a tenant-scoped resource (save edit, delete, transition state).

Generated prelude + form-error envelope:
- Same tenant + resource resolution as `#[resource_get]`
- Macro arg: `redirect_to = "..."` for the success redirect (literal or fmt template)
- Macro arg: `form_url = "..."` for the validation-failure redirect (literal or fmt template, typically the edit GET)
- New helper on `Validator`: `.validate_or_redirect(&data, &form_url)` — collapses the existing 2-line `into_action_error` chain into the validator's own `?` flow

Sketch:

```rust
#[ferro::resource_post(Customer, redirect_to = "/dashboard/clienti", form_url = "/dashboard/clienti/{id}/modifica")]
pub async fn save(req: Request, tenant: &Tenant, customer: &Customer) -> ActionResult {
    let form: ClienteForm = req.form_mut().await?;
    let data = json!({ /* ... */ });
    Validator::new(&data)
        .rules("name", rules![required()])
        .rules("email", rules![email()])
        .messages(/* ... */)
        .validate_or_redirect(&data, /* form_url synthesised by macro */)?;
    Customer::update(&data).await?;
    Ok(())
}
```

### Out of scope (explicitly dropped)

- `#[confirm_page]` macro — the gestiscilo evidence pass found ZERO recurring confirm-page view shape (202-EVIDENCE.md §1 Pattern E). Destructive actions are inline buttons that POST directly to `/elimina`. No macro target.

- Multipart upload macros — gestiscilo has 4 distinct upload domains (operator file, customer self-upload, staff photo, document signing) with different shapes. Separate macros if warranted, not part of 212.

## Consumer evidence (what the macros must support)

The full duplication survey is in `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/202-adopt-ferro-crud-macros/202-EVIDENCE.md`. Key data ferro should plan against:

**Tenant-resolution call sites (Pattern A + 112 'Sessione scaduta' map_err)** — 244 `resolve_tenant()` calls across gestiscilo controllers. Every POST handler opens with the same 3-line shape. The macro must work with consumer-defined tenant resolution (gestiscilo's is a thread-local; other apps may use middleware or extractors).

**Resource lookup (Pattern A 55 + Pattern C 39)** — `find_for_tenant(id, tenant_id)` is the gestiscilo signature. Other consumers may have different signatures. The macro depends on a `TenantScoped` trait the consumer implements.

**Form-error redirect (Pattern B 129)** — 16 sites already use the modern chain, 73 use a single-Response shape, 72 use the legacy discard idiom. The new `validate_or_redirect` validator helper should make the modern chain ergonomic to the point that the legacy idiom doesn't tempt anyone.

## Open design questions for discuss-phase

1. **Tenant resolution coupling**: how does the macro know how to resolve the tenant? Options:
   - **A**: a `TenantResolver` trait on `Request` that consumers implement once
   - **B**: a macro argument `tenant = "expr"` that takes a Rust expression to evaluate
   - **C**: a runtime extension type registered at app boot — `#[resource_get(Customer)]` looks up `req.extensions::<TenantResolver>()`

2. **Resource trait shape**: gestiscilo's models all use `find_for_tenant(id: i64, tenant_id: i64) -> Result<Option<Self>, Error>`. Make that the required trait signature? Or accept a function-pointer macro arg `find = "Customer::find_for_tenant"`?

3. **404 strategy**: should the macro emit gestiscilo-style `dashboard_error_view` HTML, or a generic 404? Two paths:
   - **A**: macro emits a plain ferro `Response::not_found()` and the consumer is expected to install a 404 handler middleware that styles it
   - **B**: macro arg `on_miss = "url"` redirects, `on_miss = handler_path` calls a consumer-provided function

4. **Macro composition with `#[handler]` / `#[action]`**: `#[resource_get]` wraps `#[handler]`; `#[resource_post]` wraps `#[action]`. Should the macros emit those directly, or be standalone? Outer-most-attribute convention matters for cargo expand readability.

5. **`Validator::validate_or_redirect` shape**: is this a method on `Validator`, on `ValidationError`, or a free function? Where does the `&data` round-trip live? See `framework/src/validation/error.rs:160` for the existing `into_action_error` pattern that's likely the right composition target.

6. **Form URL synthesis**: when `form_url` is a fmt template with `{id}` placeholders, where do the placeholders come from? The macro can read them from the route params it already extracted (`id` from `path = "{id:i64}"`), but other placeholders (e.g. `{from_query}`) need explicit syntax.

7. **Editor experience**: jump-to-definition, rustdoc rendering, IDE autocomplete — proc macros often break these. The 212 design needs an "IDE experience" section before plan-write.

## Deliverables when 212 ships

- `ferro_macros::resource_get` + `ferro_macros::resource_post` proc macros, exported via the `ferro` facade
- `Validator::validate_or_redirect(&data, &url) -> Result<(), ActionError>` helper that composes with the macro
- A `TenantResolver` trait (or the chosen shape from §1) — small surface, well-documented
- A `TenantScoped` trait (or chosen shape from §2) — small surface
- Reference example app or test fixture showing the macros in use
- Rustdoc with `cargo expand` walkthroughs for the two non-trivial expansions
- ferro CHANGELOG entry + version bump (workspace 0.2.54 → 0.2.55 at minimum)

## Paired consumer phase

`gestiscilo-it/app` Phase 202 (consumer-side adoption sweep) ships independently of 212 — it adopts the EXISTING ferro APIs (`param_as`, `into_action_error`, `tenant_resource_or_404` helper) and doesn't depend on the macros. After 212 publishes, an optional Phase 202b in gestiscilo adopts the macros on top of the cleaned-up consumer code. The split lets the boilerplate-removal value ship fast and decouples it from the proc-macro design timeline.

## Next step

`/gsd-discuss-phase 212` (run inside the ferro repo) to lock the seven open questions in §"Open design questions". Plan-phase follows discuss-phase.
