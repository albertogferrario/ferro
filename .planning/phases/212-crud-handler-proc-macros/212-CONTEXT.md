---
phase: 212-crud-handler-proc-macros
milestone: v13.1
status: Ready for planning
created: 2026-06-12
gathered: 2026-06-13
created_by: gestiscilo Phase 202 evidence-gathering pass
renumbered: "2026-06-12 — relocated from 209 to 212 (new milestone v13.1) to resolve a phase-number collision: ferro ROADMAP already owns 209 for COMP-01 Gestiscilo Migration"
paired_consumer_phase: gestiscilo-it/app Phase 202 (see 202-EVIDENCE.md §8)
---

# Phase 212: CRUD Handler Proc Macros - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Ship two route-attribute proc macros plus one validator helper that fold the recurring
tenant-scoped CRUD prelude ferro consumers write by hand today:

- **`#[resource_get]`** — wraps a GET handler displaying a single tenant-scoped resource (edit
  form, detail, confirm page): typed path-param extraction → tenant resolution → tenant-scoped
  lookup → 404-on-miss, leaving only the user's body.
- **`#[resource_post]`** — wraps a POST handler mutating a tenant-scoped resource: the same
  prelude plus a validation-failure redirect envelope.
- **`Validator::validate_or_redirect(&data, &url)`** — collapses the existing
  `into_action_error` chain into the validator's own `?` flow.

**Killer feature:** the prelude folds into a single route attribute *while tenant and resource
stay real, typed function parameters* (IDE jump-to-def + autocomplete keep working) and the macro
reuses ferro's existing tenant and validation layers — compression without a new mental model or a
duplicate control surface. That is the substance-first win (compressive + conceptual), not raw
line count.

**Out of scope (explicitly dropped):**
- `#[confirm_page]` macro — the gestiscilo evidence pass found ZERO recurring confirm-page shape
  (202-EVIDENCE §1 Pattern E); destructive actions POST directly to `/elimina`. No macro target.
- Multipart upload macros — gestiscilo has 4 distinct upload domains with different shapes;
  separate macros if warranted, not part of 212.
- Reducing one consumer's LoC as the success metric — this is scoped to ferro's framework-product
  axis (any consumer benefits), not gestiscilo's local savings.

</domain>

<decisions>
## Implementation Decisions

The seven open design questions from the scoping doc are resolved below. Each reuses an existing
ferro surface where one exists, keeping the core small (continuous conceptual coherence).

### Tenant resolution (Q1)
- **D-01:** The macro resolves the tenant through the **existing** tenant layer —
  `ferro::current_tenant()` / the configured `TenantScopeProvider`
  (`framework/src/lib.rs:133`) — NOT a new `TenantResolver` trait. ferro already owns
  "how is the tenant resolved"; adding a second knob would duplicate that control surface
  (rejected per the no-duplicate-control-surface principle). Consumers whose tenant lives
  elsewhere configure the existing provider/middleware.
- **D-02:** Provide an optional escape-hatch macro arg `tenant = "<expr>"` (a Rust expression /
  function path) for consumers not on the standard provider. Default = the existing
  `current_tenant()` surface; the arg is the exception, not the norm. gestiscilo's thread-local
  `resolve_tenant()` is bridged to `current_tenant()` (or passed via this arg) — research confirms
  which.

### Resource lookup trait (Q2)
- **D-03:** A small **`TenantScoped` trait** the consumer implements on its model is the default:
  an associated `Id` type (bound `FromStr`, so not hardcoded to `i64`) and
  `find_for_tenant(id: Self::Id, tenant_id) -> Result<Option<Self>, _>`. The macro calls it with
  zero config in the common case.
- **D-04:** An optional `find = "<path::fn>"` macro arg overrides the lookup for models that don't
  fit the trait signature. Trait-default keeps the common case zero-config; the arg is the escape
  hatch.

### 404 / miss strategy (Q3)
- **D-05:** `on_miss = "/url"` macro arg → redirect to that URL on a lookup miss (the common
  dashboard case). If `on_miss` is omitted → emit a **generic** miss: `Response::not_found()` for
  `#[resource_get]`, `ActionError::not_found(...)` for `#[resource_post]`. The macro MUST NOT emit
  any consumer-specific styling (e.g. gestiscilo's `dashboard_error_view`) — `ferro-macros` is a
  project-agnostic crate; consumers style the generic 404 via their own middleware.

### Macro composition (Q4)
- **D-06 (refined by research):** `#[resource_get]` / `#[resource_post]` **inline the
  handler/action boilerplate directly** — they do NOT emit a nested `#[ferro::handler]` /
  `#[ferro::action]` attribute on the generated wrapper. The user still writes a single attribute
  at the call site, and the original body still moves into a clearly-named inner fn
  (e.g. `__{name}_inner`) so `cargo expand` reads cleanly and rust-analyzer sees real types.
  **Why the change from the discuss-phase default:** the discuss-phase guess was "emit
  `#[handler]`/`#[action]` internally". Research (212-RESEARCH §D-06, Pitfall 1) found the
  `action_impl`/`handler_impl` output for a single `Request` param is a ~10-line shell that is
  trivial to inline, and that re-emitting the nested attribute risks a double param-extraction
  interaction. Inlining is the recommended, self-contained, expand-readable approach. The
  reference shape is `ferro-macros/src/http/action.rs:17-28` (`action_impl`) and the `handler`
  fn at `ferro-macros/src/lib.rs:232`. This is a how-to-implement refinement; the user-observable
  surface (one attribute → folded prelude → typed params) is unchanged.

### `validate_or_redirect` shape (Q5)
- **D-07:** A **method on `Validator`** that consumes `self` and returns
  `Result<(), ferro::ActionError>`, composing the EXISTING error path — no new logic:
  ```rust
  pub fn validate_or_redirect(self, data: &Value, url: impl Into<String>)
      -> Result<(), ActionError> {
      self.validate().map_err(|e| e.with_old_input(data).into_action_error(url))
  }
  ```
  It reuses `ValidationError::with_old_input` + `into_action_error`
  (`framework/src/validation/error.rs:160`) so per-field errors + old input flash exactly as the
  modern chain does today. `&data` is passed because `with_old_input` needs the original payload.

### Form-URL synthesis (Q6)
- **D-08:** When `redirect_to` / `form_url` / `on_miss` is a fmt template with `{param}`
  placeholders, the macro substitutes them from the **path params it already extracted** (the
  resource id and any declared path params) via `format!`. A placeholder that does NOT correspond
  to an extracted path param is a **compile error** with a clear message — no query/body/session
  magic; the consumer builds that URL in the body and passes it explicitly.

### IDE / editor experience (Q7)
- **D-09:** Design for rust-analyzer: (a) the resolved bindings (`tenant: &Tenant`,
  `customer: &Customer`) are REAL typed parameters in the user's signature — the macro reads them
  from the signature, it does not invent hidden names — so autocomplete and jump-to-def work;
  (b) the body lives in a named inner fn; (c) rustdoc ships `cargo expand` walkthroughs for both
  expansions. The plan MUST include an explicit "IDE experience" verification (expand + a
  rust-analyzer-style type check on the bindings).

### Requirement labels (derive in REQUIREMENTS.md)
- **D-10 — CRUD-01..CRUD-06:**
  - **CRUD-01** — `#[resource_get]` folds typed-param + tenant resolution + tenant-scoped lookup +
    404-on-miss; tenant/resource surface as real typed params.
  - **CRUD-02** — `#[resource_post]` folds the same prelude + the validation-failure redirect
    envelope.
  - **CRUD-03** — `Validator::validate_or_redirect(&data, &url) -> Result<(), ActionError>`
    composing the existing `with_old_input` + `into_action_error`.
  - **CRUD-04** — `TenantScoped` trait (assoc `Id: FromStr` + `find_for_tenant`) with a `find =`
    override; tenant resolved via the existing `current_tenant()`/`TenantScopeProvider` (no new
    control surface), with a `tenant = expr` escape hatch.
  - **CRUD-05** — IDE experience preserved: typed params, named inner fn, rustdoc `cargo expand`
    walkthroughs, verified.
  - **CRUD-06** — macros exported via the `ferro` facade; a reference example/test fixture using
    both macros; CHANGELOG entry + workspace version bump (next patch, 0.2.56+ — 0.2.55 already
    published).

### Claude's Discretion
- Exact `TenantScoped` trait method names/associated-type bounds beyond `Id: FromStr`.
- Whether `path = "{id:i64}"` syntax or a positional resource-type arg drives id extraction —
  planner picks the cleanest macro surface consistent with D-08.
- Whether the bridge from gestiscilo's thread-local `resolve_tenant()` to `current_tenant()` is
  documentation-only or a tiny adapter (research determines).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Consumer evidence (the duplication this folds)
- `/Users/alberto/repositories/gestiscilo-it/app/.planning/phases/202-adopt-ferro-crud-macros/202-EVIDENCE.md`
  — the full duplication survey (Pattern A tenant resolution ×244, Pattern A/C resource lookup,
  Pattern B form-error redirect ×129). Cross-repo absolute path; read if the sibling repo is
  present. The key counts are summarized in the "Consumer evidence" section below.

### Composition targets in the ferro surface
- `ferro-macros/src/lib.rs:232` (`#[handler]`) and `:265` (`#[action]`) — the attribute macros
  `#[resource_get]`/`#[resource_post]` emit (D-06).
- `framework/src/validation/error.rs:160` — `ValidationError::into_action_error(url)` +
  `with_old_input` — the exact composition `validate_or_redirect` reuses (D-07).
- `framework/src/validation/validator.rs` — `Validator` API (`new`, `rules`, `validate`,
  `with_error`); the new `validate_or_redirect` method lives here.
- `framework/src/http/action.rs` — `ActionError` (`msg`, `validation_failed`, `not_found`,
  `redirect_to`) and `ActionResult` (D-05, D-07).
- `framework/src/http/request.rs:177` — `Request::param_as<T: FromStr>` (typed path extraction)
  and the `extensions::<T>()` type-map.
- `framework/src/lib.rs:133` — the existing tenant surface: `current_tenant`,
  `TenantScopeProvider`, `FrameworkTenantScopeProvider`, resolvers (D-01/D-02). Do not add a
  parallel tenant-resolution knob.

### Phase scope
- `.planning/ROADMAP.md` §"Phase 212: CRUD Handler Proc Macros" — goal + dependencies.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`#[handler]` / `#[action]` attribute macros** (`ferro-macros/src/lib.rs`) — `#[resource_get]`
  and `#[resource_post]` wrap these rather than re-implementing the HTTP plumbing.
- **Tenant layer** — `current_tenant()` / `TenantScopeProvider` / `FrameworkTenantScopeProvider`
  already decide tenant resolution structurally (v12.6). The macro binds to it.
- **`ValidationError::into_action_error` + `with_old_input`** — the modern form-error chain
  `validate_or_redirect` composes; no new error path.
- **`Request::param_as<T>`** — typed path extraction the macro emits.
- **`ActionError` / `ActionResult`** — the macro's miss + validation arms return these.

### Established Patterns
- Attribute proc macros emit `#[handler]`/`#[action]`-wrapped fns (consistent with existing macro
  layering).
- User-facing symbols are re-exported from the `ferro` facade (`framework/src/lib.rs`).
- Project-agnostic `ferro-*` crates: no hardcoded consumer identity/styling (D-05).

### Integration Points
- `ferro-macros/src/lib.rs` — the two new `#[proc_macro_attribute]` fns.
- `framework/src/validation/validator.rs` — the new `validate_or_redirect` method.
- A new small trait module (`TenantScoped`) — likely in `ferro-orm` or `framework` per the planner.
- `framework/src/lib.rs` — facade re-exports of the macros + trait.
- A reference example/test fixture exercising both macros (CRUD-06).

</code_context>

<specifics>
## Specific Ideas

### Macro surface sketches (from the scoping doc — the intended call-site ergonomics)
```rust
#[ferro::resource_get(Customer, on_miss = "/dashboard/clienti")]
pub async fn edit(req: Request, tenant: &Tenant, customer: &Customer) -> Response { /* body */ }

#[ferro::resource_post(Customer,
    redirect_to = "/dashboard/clienti",
    form_url = "/dashboard/clienti/{id}/modifica")]
pub async fn save(req: Request, tenant: &Tenant, customer: &Customer) -> ActionResult {
    let form: ClienteForm = req.form_mut().await?;
    let data = json!({ /* ... */ });
    Validator::new(&data)
        .rules("name", rules![required()])
        .validate_or_redirect(&data, /* form_url synthesised by macro */)?;
    Customer::update(&data).await?;
    Ok(())
}
```

### Consumer evidence the macros must support (from 202-EVIDENCE)
- **Tenant resolution** — 244 `resolve_tenant()` call sites; every POST opens with the same
  3-line shape. Macro must work with consumer-defined tenant resolution (gestiscilo uses a
  thread-local; others use middleware/extractors) → D-01/D-02.
- **Resource lookup** — `find_for_tenant(id, tenant_id)` is gestiscilo's signature; others differ
  → `TenantScoped` trait + `find =` override (D-03/D-04).
- **Form-error redirect** — 129 sites (16 modern chain, 73 single-Response, 72 legacy discard).
  `validate_or_redirect` should make the modern chain ergonomic enough that the legacy idiom
  stops tempting anyone (D-07).

</specifics>

<deferred>
## Deferred Ideas

- `#[confirm_page]` macro — no recurring shape found; dropped (see Phase Boundary).
- Multipart upload macros — 4 distinct upload domains; separate future phases if warranted.
- **gestiscilo Phase 202b** — the consumer-side adoption of these macros ships AFTER 212
  publishes, on top of the already-cleaned consumer code. Cross-repo; not this phase. (gestiscilo
  Phase 202 adopts the existing `param_as`/`into_action_error` APIs and does not depend on 212.)

None of the above belongs in Phase 212 — discussion stayed within the macro-design scope.

</deferred>

---

*Phase: 212-crud-handler-proc-macros*
*Context gathered: 2026-06-13*
