# Action Handlers

`#[action]` is the attribute macro for POST-style handlers that mutate state
and then redirect. It is a sibling of `#[handler]` — choose `#[action]` when
the handler's natural completion is a 303 redirect (success or error);
choose `#[handler]` for HTMX partials, JSON endpoints, GET pages, or anything
that does not redirect on every code path.

## When to use `#[action]`

Use `#[action]` when the handler:

- Receives a POST (or PUT / PATCH / DELETE) form submission.
- Mutates state (creates, updates, deletes a record).
- Redirects to a list or detail page on success.
- Should redirect (often back to the same page) on error, surfacing the
  failure as a flash message.

Keep `#[handler]` for handlers that render a response: GET pages, JSON APIs,
HTMX partials, file downloads.

## Quick example

```rust
use ferro::{action, ActionError, ActionResult, Request};

#[action(redirect_to = "/dashboard/pages")]
pub async fn publish_by_id(req: Request, id: i64) -> ActionResult {
    let page = Page::find_by_id(id).await?
        .ok_or(ActionError::not_found("Page not found"))?;
    page.publish().await?;
    Ok(())
}
```

On success the user is redirected to `/dashboard/pages` with a 303. On any
`Err`, the user is redirected to the same URL with `?error=<kind>&msg=<text>`
appended and a session flash entry written under the `_action` slot.

## The macro shape

```rust
#[action(redirect_to = "/path", method = "POST")]
```

- `redirect_to` — **required**. The success-path redirect target. Must be a
  same-origin path (starting with `/`).
- `method` — optional. Default `"POST"`. Currently informational; it documents
  the intended method but does not affect routing. Wire the route in your
  `routes.rs` as usual.

The user-written signature uses `req: Request` (not `&mut Request`). The macro
generates the mutable binding internally — `Request` passes by value in the
function signature, and the macro rebinds it as `&mut Request` before handing
it to the user body.

## Return type — `ActionResult`

```rust
pub type ActionResult = Result<(), ActionError>;
```

The success expression is `Ok(())` — there is no helper type to construct.
The error side is `ActionError` (described below).

## `?` ergonomics

Inside an `#[action]` body, `?` works directly on:

- `Result<_, String>`
- `Result<_, &'static str>`
- `Result<_, FrameworkError>`
- `Result<_, sea_orm::DbErr>`

The macro wraps the body in an `async move { ... }.await` block typed as
`ActionResult`, so `?` propagates to `ActionResult`, not to the outer
`Response`-returning function. This is what makes bare `?` work on the
error types above.

For any other error type implementing `std::fmt::Display`, use the
`.action_err()` extension method:

```rust
use ferro::ActionResultExt;

let value = some_external_call()
    .action_err()?;
```

`.action_err()` converts any `Display` error into an `ActionError::msg(e.to_string())`.

## `ActionError` — constructors and builders

```rust
ActionError::msg("Something went wrong");           // kind = Generic
ActionError::not_found("Page not found");           // kind = NotFound
ActionError::forbidden("You don't own this page");  // kind = Forbidden
ActionError::unauthorized("Please sign in");        // kind = Unauthorized
```

Builder methods (consuming `mut self -> Self`):

```rust
ActionError::msg("Saved as draft")
    .with_flash(FlashVariant::Warning)         // override the banner variant
    .redirect_to("/dashboard/pages/123/edit"); // override the redirect target
```

`redirect_to(...)` on `ActionError` is the **error-side** redirect override.
The most common use is `unauthorized()` — ferro does not know your
application's sign-in path, so configure it explicitly:

```rust
if !user.is_authenticated() {
    return Err(ActionError::unauthorized("Please sign in")
        .redirect_to("/your-login-path"));
}
```

> Ferro intentionally does not ship a default authentication redirect.
> Consumer applications supply the path. This keeps `ferro-*` crates
> project-agnostic (no hardcoded sign-in routes).

## Success-side overrides

`Request` exposes two setter methods for overriding the success path:

```rust
#[action(redirect_to = "/dashboard/pages")]
pub async fn create(req: Request) -> ActionResult {
    let new_page = Page::create(...).await?;
    req.redirect_to(format!("/dashboard/pages/{}", new_page.id));
    req.flash("created");
    Ok(())
}
```

- `req.redirect_to(url)` — overrides the configured `redirect_to` on the
  success path. Validated as same-origin; external URLs are silently ignored
  (see Security below).
- `req.flash(key)` — records a success flash key. On the next page render,
  the consumer template reads the `_action` flash slot to surface the
  success message.

Both setters take effect only on the success path. On `Err`, they are
ignored; `ActionError`'s own `redirect_override` and `flash_variant` apply
instead.

The override surface is split by code path: success-side overrides live on
`Request` (where the success handler has access); error-side overrides live
on `ActionError` (where the error is constructed). This makes each override
discoverable at its natural call site.

## Flash transport

`#[action]` writes a JSON payload to the session flash slot `_action`:

```json
{
  "variant": "error",
  "message": "Page not found"
}
```

`variant` is one of `error`, `warning`, `info`, or `success`. Templates
use this to choose the banner's CSS class.

Consumer templates read the flash via:

```rust
let action_flash = session.get_flash::<serde_json::Value>("_action");
// Pass to the view.
```

`get_flash` ages the value — the next render after the redirect sees it;
subsequent renders do not.

### Back-compat query string

For templates not yet wired to read session flash, `#[action]` also appends
a query string to the redirect URL:

- Error path: `?error=<kind>&msg=<percent-encoded-message>`
- Success path: `?success=<flash-key-or-1>`

where `<kind>` is the snake_case `ActionKind` value: `generic`, `not_found`,
`forbidden`, `unauthorized`. This fallback exists for incremental migration
and may be removed in a future ferro release once session flash reading is
universal in consumer templates.

## Security

The action runtime applies three mitigations. Consumers should be aware of
each.

### Flash message escaping (T-180-01)

`ActionError::message` is treated as untrusted user-facing display text. It
may originate from a database error reflecting a query parameter, a parsed
form field, or any other input the handler touches.

**Consumer templates MUST HTML-escape the flash message before rendering.**
Ferro stores the message verbatim in the JSON flash payload; escaping is
the template's responsibility.

Safe rendering example:

```html
{% if action_flash %}
  <div class="flash flash--{{ action_flash.variant }}">
    {{ action_flash.message | escape }}
  </div>
{% endif %}
```

### Open-redirect mitigation (T-180-02)

Both the error-side `ActionError::redirect_to(...)` and the success-side
`req.redirect_to(...)` are validated as same-origin at the moment the
redirect is built. URLs not starting with `/`, or starting with `//`
(scheme-relative), are silently rejected and the request falls back to the
configured `#[action(redirect_to = "...")]` target. A `tracing::warn!`
records the rejection.

To redirect to an external URL, use a dedicated `#[handler]` that constructs
the redirect explicitly.

### Log-injection mitigation (T-180-03)

`tracing::error!` is called on the error path with the handler name and
error message. Control characters in the message (`\n`, `\r`, `\t`, NUL,
etc.) are stripped before the call so structured log sinks see a single-line
value. This is defense-in-depth — JSON log formatters escape control
characters anyway, but plain-text sinks benefit.

The query-string fallback (`?msg=...`) uses standard percent-encoding so
control characters in the URL are encoded, not interpreted.

## Before / after

A typical action handler before `#[action]`:

### Before

```rust
#[handler]
pub async fn publish_by_id(req: Request, id: i64) -> Response {
    let page = Page::find_by_id(id).await
        .map_err(|e| error_response(500, &format!("DB error: {e}")))?
        .ok_or_else(|| error_response(404, "Page not found"))?;
    if page.tenant_id != current_tenant_id {
        return Err(error_response(403, "Not authorized"));
    }
    match page.publish().await {
        Ok(()) => Redirect::to("/dashboard/pages?success=published").into(),
        Err(err_msg) => {
            let encoded: String = pct_encode(&err_msg);
            Redirect::to(format!("/dashboard/pages?error=publish&msg={}", encoded)).into()
        }
    }
}
```

On any failure the browser is stranded at the POST URL displaying a 500 page.

### After

```rust
#[action(redirect_to = "/dashboard/pages")]
pub async fn publish_by_id(req: Request, id: i64) -> ActionResult {
    let page = Page::find_by_id(id).await?
        .ok_or(ActionError::not_found("Page not found"))?;
    if page.tenant_id != current_tenant_id {
        return Err(ActionError::forbidden(""));
    }
    page.publish().await?;
    Ok(())
}
```

Every failure is a 303 redirect to `/dashboard/pages` with a flash message
and a back-compat query string. The browser does not strand at the POST URL.

## See also

- [Controllers](./controllers.md) — for the routing layer that mounts handlers.
- [Request & Response](./request-response.md) — for non-redirecting response types.
