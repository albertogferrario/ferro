---
phase: 180
title: Declarative action handler primitive
source: gestiscilo-it 2026-05-28 publish-action field test
status: Ready for planning
gathered: 2026-05-30
---

# Phase 180 — Context

## Driver

Consumer (gestiscilo-it) is iterating on the dashboard frontend management page (`/dashboard/pagine`). The "Pubblica" row action on each card POSTs to `/dashboard/pagine/{id}/publish`. In production the request was returning 50x, and because the browser does a full-page POST, the URL stuck at the action endpoint — owners saw a 500 page at `/pagine/{id}/publish` and had to manually navigate back.

**`/pagine` is one symptom; this is a category bug.** Every action handler in every consumer app has the same shape: resolve tenant, parse params, load record, ownership check, mutate, redirect. Every fallible step short-circuits to `error_response(...)` which strands the browser at the POST URL. This phase fixes the entire category — primitive + reference migration covering every action handler the consumer ships, not just `/pagine`. Half-migrated state is not an acceptable end state; either the primitive replaces every action-handler error-wrapping site or it shouldn't ship.

## What consumer apps are forced to write today

Every action handler that mutates and redirects looks like this:

```rust
#[handler]
pub async fn publish_by_id(req: Request) -> Response {
    let business = resolve_tenant().await?;                    // ← returns 401/500 HTML on failure
    let id: i64 = req.param("id")
        .map_err(|_| error_response(400, "ID non valido"))
        .and_then(|s| s.parse().map_err(|_| error_response(400, "ID non valido")))?;
    let frontend = Page::find_by_id(id).await
        .map_err(|e| error_response(500, &format!("Errore: {e}")))?
        .ok_or_else(|| error_response(404, "Pagina non trovata"))?;
    if frontend.tenant_id != business.id {
        return Err(error_response(403, "Non autorizzato"));
    }
    match publish_page(...).await {
        Ok(v) => {
            Page::save_validation_json(...).await
                .map_err(|e| error_response(500, &format!("Errore: {e}")))?;
            Page::mark_published(...).await
                .map_err(|e| error_response(500, &format!("Errore: {e}")))?;
            Redirect::to("/dashboard/pagine?success=published").into()
        }
        Err(err_msg) => {
            let encoded: String = err_msg.chars().map(|c| { /* percent-encode */ }).collect();
            Redirect::to(format!("/dashboard/pagine?error=publish&msg={}", encoded)).into()
        }
    }
}
```

Six fallible steps. Five of them short-circuit to `error_response(...)` which returns HTML with a status code — stranding the browser at the POST URL on any failure. Only `publish_page`'s error path uses Redirect.

Consumer's shipped workaround (kept open as bridge until this phase ships):

```rust
fn pct_encode(msg: &str) -> String { /* … */ }

fn pagine_redirect(result: Result<(), String>) -> Response {
    match result {
        Ok(()) => Redirect::to("/dashboard/pagine?success=published").into(),
        Err(msg) => {
            eprintln!("publish redirect error: {msg}");
            Redirect::to(format!("/dashboard/pagine?error=publish&msg={}", pct_encode(&msg))).into()
        }
    }
}

#[handler]
pub async fn publish_by_id(req: Request) -> Response {
    let business = match resolve_tenant().await { /* manual unwrap → redirect */ };
    let id = match req.param("id").ok().and_then(|s| s.parse().ok()) { /* manual unwrap → redirect */ };

    let result = async { /* …actual handler body returning Result<(), String>… */ }.await;
    pagine_redirect(result)
}
```

Every action handler in the consumer app needs the same scaffolding:
- `publish_by_id`, `publish`, `delete_by_id`, `delete_page`, `create`, `update`
- `update_dominio`, `disconnect_dominio`
- ~30 more across cassa/ordini, cassa/prodotti, prenotazioni, staff, clienti, magazzino, settings

The boilerplate ratio is roughly **15 lines of error-wrapping per 10 lines of business logic**.

## What "fixed" looks like

```rust
#[action(redirect_to = "/dashboard/pagine")]
pub async fn publish_by_id(req: Request) -> ActionResult {
    let business = resolve_tenant().await?;
    let id: i64 = req.param("id")?.parse()?;
    let frontend = Page::find_by_id(id).await?
        .ok_or(ActionError::msg("Pagina non trovata"))?;
    if frontend.tenant_id != business.id {
        return Err(ActionError::forbidden());
    }
    publish_page(...).await?;
    Page::save_validation_json(...).await?;
    Page::mark_published(...).await?;
    spawn_screenshot_capture(...);
    Ok(())  // → 303 /dashboard/pagine?success=…
}
```

Ferro catches the `Err`, percent-encodes, logs to stderr, returns 303 to `redirect_to` with `?error=…&msg=…`. Blanket `From<E: Display>` on `ActionError` so `?` works on `FrameworkError`, `String`, `sea_orm::DbErr`, anything.

## Design surface (planner to refine)

1. **`ActionError`** — error type. Required fields: `message: String`. Optional fields: `kind: ActionKind` (Generic/NotFound/Forbidden/Unauthorized), `flash_variant: FlashVariant` (Error/Warning/Info), `redirect_override: Option<String>` (for cases where the natural redirect target depends on the error, e.g. unauthorized → `/accedi`).

2. **`ActionResult`** — `Result<ActionOk, ActionError>` where `ActionOk` carries optional `flash: Option<&'static str>` and `redirect_override: Option<String>` for success-side overrides (e.g. created → `/dashboard/X/{new_id}`).

3. **Attribute `#[action(redirect_to = "...", method = "POST")]`** — macro on async handlers. Wraps the body, catches Result, builds the 303. Method default `POST` (matches the existing `#[handler]` discoverability).

4. **Flash transport** — query string is simple but messages can be long/sensitive. Alternative: signed cookie (`__flash`) consumed on next render. Planner picks; query-param is what consumer already uses, lowest churn.

5. **`From<E: Display>` blanket** — controversial because it conflicts with `From<String>`. Standard workaround is a wrapper trait `IntoActionError`. Planner picks the ergonomic shape.

6. **Logging** — `tracing::error!(handler = ..., msg = ..., source = ...)`. Use existing ferro logging conventions.

7. **Auth-failure routing** — `resolve_tenant()` failure should route to `/accedi`, not the configured `redirect_to`. Mechanism: `ActionError::unauthorized()` carries a `redirect_override` that ferro respects, OR ferro inspects the source error type. Planner picks.

## Implementation Decisions (auto-locked 2026-05-30)

Recommended defaults for the seven design-surface gray areas. Planner may override with rationale, but these are the lock state going into research and planning.

### Error type and result shape

- **D-01 — `ActionError` fields.** All four: `message: String` (required), `kind: ActionKind` (default `Generic`; variants `Generic | NotFound | Forbidden | Unauthorized`), `flash_variant: FlashVariant` (default `Error`; variants `Error | Warning | Info`), `redirect_override: Option<String>` (default `None`). Constructors: `ActionError::msg(impl Into<String>)`, `::not_found(...)`, `::forbidden(...)`, `::unauthorized(...)`. Builder methods: `.with_flash(FlashVariant)`, `.redirect_to(impl Into<String>)`.
- **D-02 — `ActionOk` fields.** `flash: Option<&'static str>` and `redirect_override: Option<String>`. Returning `Ok(())` is the common case (`From<()> for ActionOk`). Override constructors: `ActionOk::flash("created")`, `ActionOk::redirect_to("/dashboard/x/{id}")`.
- **D-03 — `ActionResult = Result<ActionOk, ActionError>`.** Type alias exported from `ferro::action` (or wherever the primitive lands).

### Error conversion ergonomics

- **D-04 — `IntoActionError` wrapper trait, not blanket `From<E: Display>`.** The blanket conflicts with `From<String>` and `From<&str>` (orphan rule violation in user code). Implementation:
  ```rust
  pub trait IntoActionError { fn into_action_error(self) -> ActionError; }
  impl<E: Display> IntoActionError for E { fn into_action_error(self) -> ActionError { ActionError::msg(self.to_string()) } }
  ```
  Then `?` works through a thin shim: either a `From<T> for ActionError where T: IntoActionError` (specialization not required since `IntoActionError` is sealed-via-blanket) or an explicit `.into_action_error()?` call. Planner picks the exact mechanism that compiles cleanly on stable; the surface area (`?` usable on `FrameworkError`, `String`, `sea_orm::DbErr`, `anyhow::Error`) is the requirement.

### Macro shape

- **D-05 — `#[action(redirect_to = "...", method = "POST")]`.** `method` default `POST` (matches `#[handler]` discoverability convention). `redirect_to` is required for the success-path 303 target. Macro wraps the body, catches `Result<ActionOk, ActionError>`, builds the 303, percent-encodes flash messages, writes to session flash, logs to stderr via tracing.

### Flash transport

- **D-06 — Session flash, not query-string and not signed cookie.** Ferro already owns the mechanism at `framework/src/session/store.rs:86` (`session.flash(key, value)`, aged on next request, namespaced under `_flash.new.*` / `_flash.old.*`). The macro writes `{variant, message}` to the `_action` flash slot; the next render reads it. Rationale: avoids URL pollution and length limits; avoids re-implementing signed-cookie crypto; consumer's current query-string approach (`?error=…&msg=…`) is phased out across the migration sweep.
  - **Back-compat for templates not yet flash-aware:** macro ALSO appends `?error=...&msg=...` (error case) or `?success=...` (success case) to the redirect target. Consumers keep working during the sweep; phase 180+1 (or a later cleanup phase) deletes the query-string fallback once every dashboard view reads session flash.

### Logging

- **D-07 — `tracing::error!(handler = %name, msg = %err.message, source = ?err)`.** Matches existing ferro convention (`framework/src/middleware/rate_limit.rs` and rest of framework already use tracing). Macro emits the call at the catch site so the consumer's controller doesn't need an explicit `eprintln!`. Span attaches handler name (from `fn_name`) and route (from request context if available).

### Auth-failure routing

- **D-08 — `ActionError::unauthorized()` carries `redirect_override = Some("/accedi")` by default.** Builder-overridable via `.redirect_to("/login")`. Macro inspects `redirect_override` on the error; if `Some`, uses it instead of the configured `#[action(redirect_to = ...)]`. This generalizes: any error variant can carry an override (e.g. `Forbidden` → `/dashboard` instead of staying on the same page), which is more flexible than special-casing auth.
  - **Default `/accedi` is consumer-specific (gestiscilo Italian copy).** Ferro itself MUST NOT hardcode `/accedi`. The recommended ferro-level default is `None`, with consumers configuring per-action (`.redirect_to(...)`) or per-app via a `FerroConfig::auth_redirect` setting. Planner picks the config surface; project-agnostic crates rule from CLAUDE.md applies.

### Migration acceptance (hard gate)

- **D-09 — Zero workaround helpers, zero `error_response!(` in POST handler bodies.** CI grep enforcement:
  ```
  rg -l 'error_response!\(' src/controllers/ | xargs rg -l '#\[handler\]\s*(\n\s*)?pub async fn (publish|create|update|delete|new|store|destroy)' --multiline
  ```
  Phase is not complete until this returns zero matches in the gestiscilo-it consumer.
- **D-10 — Sweep is part of the phase deliverable, not a follow-up.** The consumer-side migration of ~40-60 handlers runs in the same friction-loop iteration that ships the ferro primitive. Half-migrated state is rejected.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Ferro framework (existing patterns to harmonize with)

- `ferro-macros/src/handler.rs` — `#[handler]` macro structure; `#[action]` follows the same TokenStream-input/ItemFn-parse/quote-output shape
- `ferro-macros/src/domain_error.rs` — closest existing analog: attribute macro that builds an error type + `HttpError` impl + `From<T> for FrameworkError`. `#[action]` should compose cleanly with `#[domain_error]` types.
- `ferro-macros/src/lib.rs` — public macro re-exports; new `action` proc-macro registers here
- `framework/src/http/response.rs` — `Response = Result<HttpResponse, HttpResponse>`; macro return type
- `framework/src/http/redirect.rs` — `Redirect::to(url).into()` — the 303 builder
- `framework/src/validation/error.rs:103-142` — `ValidationError::redirect_back()` / `redirect_to(url)` flash-then-redirect pattern; `#[action]` follows the same shape on the error path
- `framework/src/session/store.rs:84-118` — `session.flash(key, value)`, `get_flash`, aging mechanism (`_flash.new.*` → `_flash.old.*`)
- `framework/src/lib.rs` — public re-exports; new `action` / `ActionError` / `ActionOk` / `ActionResult` surface lands here

### Consumer evidence (gestiscilo-it audit, 2026-05-28)

- gestiscilo `src/controllers/pages.rs` — the publish-action 500 incident that drove this phase; `pagine_redirect` workaround helper lives here
- gestiscilo `src/controllers/cassa/`, `clienti.rs`, `magazzino/`, `staff.rs`, `settings/`, `documenti/` — full migration target inventory (see existing "Full inventory" table above)

### Project guidance

- `.planning/VISION.md` — projection/intent core abstraction; action handler primitive sits at the routing/HTTP layer, must not muddle projections
- `CLAUDE.md` § "Project-agnostic crates" — `ferro-macros` and `framework` MUST NOT hardcode consumer strings (no `"gestiscilo"`, no `/accedi` literal in framework defaults); auth-redirect defaults come from consumer config

### Prior phases (no conflicts identified)

- Phase 179 (DataTable RawHtml-free heterogeneous rows) — independent; no shared surface
- No prior phase locked decisions about action handlers, error response shapes, or flash transport that this phase needs to respect.

## Existing Code Insights

### Reusable assets

- **`#[handler]` proc-macro infrastructure** (`ferro-macros/src/handler.rs`) — parameter extraction scaffold (ParamKind enum, FromParam / RouteBinding / FromRequest dispatch). `#[action]` reuses the same parameter classification; only the return-type handling differs.
- **`#[domain_error]` proc-macro** (`ferro-macros/src/domain_error.rs`) — proven pattern for attribute-driven error type generation with status/message attrs. Mental model for the `#[action(redirect_to = "...", method = "POST")]` parser.
- **Session flash** (`framework/src/session/store.rs`) — already implements the exact mechanism `#[action]` needs; no new transport infrastructure required.
- **`Redirect::to(url).into()` → 303** (`framework/src/http/redirect.rs`) — the macro emits this on both success and error paths.
- **`ValidationError::with_old_input(...).redirect_to(url)`** (`framework/src/validation/error.rs:139`) — proves the flash-then-redirect-with-303 idiom is already idiomatic ferro.

### Established patterns

- **Builder pattern with `with_*` methods** consuming `mut self → Self` (CLAUDE.md: "Builder pattern: `with_*` methods taking `mut self` → `Self` (consuming)"). `ActionError` builder follows this: `.with_flash(...)`, `.redirect_to(...)`.
- **`thiserror` derive, one Error enum per crate** (per project memory). `ActionError` is a regular struct (not an enum) but uses `thiserror::Error` for the `Display` / `Error` impl.
- **`fmt::Error` chain preserved via `%w`** — when `IntoActionError` wraps a source error, preserve the chain.

### Integration points

- Public surface: `framework/src/lib.rs` re-exports `ferro::action`, `ferro::ActionError`, `ferro::ActionOk`, `ferro::ActionResult`.
- Macro registration: `ferro-macros/src/lib.rs` adds `#[proc_macro_attribute] pub fn action(attr, input) -> TokenStream`.
- ferro-mcp `code_templates` tool — add an `action_handler` template so MCP introspection shows agents the new primitive.
- Documentation: `docs/src/` needs a new page on `#[action]` (per CLAUDE.md "Always update docs when framework changes").

## Specific Ideas

- The acceptance test in the existing context (publish_by_id migration diff) is the canonical "API is right" check. Planner designs against that diff.
- Sweep ordering during the migration: start with `pages.rs` (the originating incident), then verify with the consumer that 1 handler converted cleanly, then batch-convert the rest module-by-module.
- The macro should leave non-redirecting handlers (HTMX fetch, JSON API) on `#[handler]` — no forced migration of API endpoints.

## Deferred Ideas

- **CSRF integration** — out of scope (existing ferro mechanism applies before the macro runs).
- **Per-handler authorization policies** — out of scope; separate concern, separate phase if needed.
- **HTMX / fetch-based action variant** — out of scope here; if needed later, a sibling `#[json_action]` or `#[htmx_action]` macro can compose, but the current phase ships only the redirect variant.
- **Query-string fallback removal** — D-06 includes a back-compat query-string fallback. A future cleanup phase deletes it once every consumer template reads session flash.
- **`From<E: Display>` blanket via specialization** — if/when stable specialization lands, the `IntoActionError` shim could be replaced. Not blocking.

## Migration story for consumer

`#[handler]` keeps working for GET / API / non-redirecting handlers. `#[action]` is opt-in for POST handlers that mutate-then-redirect. However, for the gestiscilo-it consumer this is **not** opt-in à-la-carte: the phase deliverable includes a sweep migrating every action handler in the consumer to the new primitive in one coordinated change.

### Full inventory of consumer action-handler sites (gestiscilo-it)

Audit performed 2026-05-28 (paths relative to `src/controllers/`):

| Module | Handlers to migrate |
|---|---|
| `pages.rs` | `create`, `update`, `delete_page`, `publish`, `publish_by_id`, `delete_by_id`, `update_dominio`, `disconnect_dominio` |
| `cassa/ordini.rs` | every POST handler (new, update, change-status, delete) |
| `cassa/prodotti.rs` | every POST handler (new, update, delete, link, unlink, image upload) |
| `cassa/pagamenti.rs` | Stripe-config POSTs |
| `calendario/prenotazioni.rs` | every POST handler |
| `clienti.rs` | every POST handler |
| `magazzino/*.rs` | every POST handler across items + units |
| `staff.rs` | every POST handler |
| `settings/*.rs` | every POST handler across modules, hours, services |
| `documenti/*.rs` | template + instance POSTs (excluding the public upload-token flow, which has its own UX) |

Rough count: ~40-60 handlers across the consumer. Planner should grep the consumer for `error_response(` in controllers as the discovery query; any site that returns that helper inside a POST handler is in-scope.

### Acceptance: zero workaround helpers left in the consumer

After this phase ships and the consumer-side sweep runs, the following must be true in the gestiscilo-it repo:

- The local `pagine_redirect` helper in `src/controllers/pages.rs` is deleted.
- No other module has invented its own `*_redirect` / `*_action_response` helper.
- `error_response(` no longer appears inside any POST handler body (it remains valid for GET handlers that genuinely want to render an error page, e.g. 404 routes).
- The codemod / grep that finds the workaround pattern returns zero matches.

If even one POST handler still hand-rolls the error→redirect dance, the phase is not done. Half-migrated state means new contributors won't know which pattern to follow and the boilerplate grows back.

## Out of scope

- CSRF — already handled by ferro elsewhere.
- Per-handler authorization policies — separate concern.
- HTMX / fetch-based actions that don't need redirect — those should keep using `#[handler]`.

## Acceptance test the planner should design against

Take consumer's `publish_by_id` from gestiscilo-it `src/controllers/pages.rs` post-workaround. Migrating to `#[action]` should:

- Delete the local `pagine_redirect` helper
- Delete the per-handler `match resolve_tenant()` / `match id_opt` ceremony
- Replace `.map_err(|e| format!(...))` chains with bare `?`
- Reduce the handler from ~60 lines to ~20 lines
- Behave identically: 303 to `/dashboard/pagine?error=publish&msg=…` on any failure, full stderr log captured

If the planner can demonstrate that diff on real consumer code, the API is right.
