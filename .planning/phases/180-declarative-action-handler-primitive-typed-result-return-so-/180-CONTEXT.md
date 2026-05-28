---
phase: 180
title: Declarative action handler primitive
source: gestiscilo-it 2026-05-28 publish-action field test
status: context-only
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
