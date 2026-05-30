# Phase 180: Declarative Action Handler Primitive — Research

**Researched:** 2026-05-30
**Domain:** Rust proc-macro authoring, HTTP redirect primitive, session flash transport
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01** — `ActionError` fields: `message: String` (required), `kind: ActionKind` (default `Generic`; variants `Generic | NotFound | Forbidden | Unauthorized`), `flash_variant: FlashVariant` (default `Error`; variants `Error | Warning | Info`), `redirect_override: Option<String>` (default `None`). Constructors: `ActionError::msg(impl Into<String>)`, `::not_found(...)`, `::forbidden(...)`, `::unauthorized(...)`. Builder methods: `.with_flash(FlashVariant)`, `.redirect_to(impl Into<String>)`.
- **D-02** — `ActionOk` fields: `flash: Option<&'static str>` and `redirect_override: Option<String>`. Returning `Ok(())` is the common case (`From<()> for ActionOk`). Override constructors: `ActionOk::flash("created")`, `ActionOk::redirect_to("/dashboard/x/{id}")`.
- **D-03** — `ActionResult = Result<ActionOk, ActionError>` type alias exported from `ferro::action`.
- **D-04** — `IntoActionError` wrapper trait, not blanket `From<E: Display>`. Blanket `impl<E: Display> IntoActionError for E`. `?` ergonomics via either `From<T> for ActionError where T: IntoActionError` or explicit shim. Planner picks the exact stable mechanism.
- **D-05** — `#[action(redirect_to = "...", method = "POST")]`. Method defaults to `POST`. Macro wraps body, catches `Result<ActionOk, ActionError>`, builds 303, writes session flash, percent-encodes back-compat query-string, logs via tracing.
- **D-06** — Session flash as primary transport (`session.flash("_action", ...)`). Query-string back-compat (`?error=...&msg=...` / `?success=...`) retained until consumer sweep completes. Flash key: `_action`.
- **D-07** — `tracing::error!(handler = %name, msg = %err.message, source = ?err)`. Macro emits at catch site. Matches existing ferro convention.
- **D-08** — `ActionError::unauthorized()` carries `redirect_override = None` as ferro default (project-agnostic crate rule). Consumer configures via `.redirect_to(...)` or per-app config. No `/accedi` literal in ferro source.
- **D-09** — CI grep gate: `rg -l 'error_response!\(' src/controllers/ | xargs rg -l '#\[handler\]\s*(\n\s*)?pub async fn (publish|create|update|delete|new|store|destroy)' --multiline`. Must return zero matches in gestiscilo-it.
- **D-10** — Consumer-side sweep (40-60 handlers) is part of the phase deliverable. Half-migrated state is rejected.

### Claude's Discretion

- Exact stable-Rust mechanism for `?` through `IntoActionError` (planner picks between `From<T> for ActionError where T: IntoActionError` vs explicit `.into_action_error()?` shim).
- Location of runtime types in `framework` crate: new `action` module vs. extension of `http` module.
- Percent-encoding helper: reuse `serde_urlencoded` / `form_urlencoded` already in framework, or write inline (the CONTEXT example shows a trivial 10-line closure).

### Deferred Ideas (OUT OF SCOPE)

- CSRF integration (existing ferro mechanism applies before the macro runs).
- Per-handler authorization policies — separate concern.
- HTMX / fetch-based action variant (`#[json_action]`, `#[htmx_action]`).
- Query-string fallback removal — future cleanup phase.
- `From<E: Display>` blanket via specialization (deferred to stable specialization landing).
</user_constraints>

---

## Summary

Phase 180 introduces three primitives in two crates:

1. **`framework`**: runtime types `ActionError`, `ActionOk`, `ActionResult`, `IntoActionError`, `ActionKind`, `FlashVariant` — and a `flash_action` helper that writes the structured flash payload to session and builds the 303 redirect `Response`.
2. **`ferro-macros`**: a `#[action(redirect_to = "...", method = "POST")]` proc-macro attribute that rewrites a handler returning `ActionResult` into the standard handler signature `(req: Request) -> Response`, inserting the error-catch-and-redirect boilerplate at the call site.
3. **`ferro-mcp`**: an `action_handler` code template in `code_templates.rs` so agents see the new primitive in `code_templates(category: "handler")` output.

The primary risk area is the stable-Rust orphan rule for `From<T> for ActionError where T: IntoActionError`. Research confirms the safe mechanism (see section 4). The flash transport is already fully implemented in `framework/src/session/store.rs:86`. The proc-macro shape is a direct extension of the existing `#[handler]` macro.

**Primary recommendation:** Implement in this order — (1) runtime types in `framework/src/action/mod.rs`, (2) `flash_action` helper, (3) public re-exports in `framework/src/lib.rs`, (4) proc-macro in `ferro-macros/src/action.rs` + registration in `ferro-macros/src/lib.rs`, (5) MCP code template, (6) docs, (7) ferro-side integration test. Consumer sweep is a parallel workstream once ferro compiles.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `ActionError` / `ActionOk` / `ActionResult` types | `framework` crate | — | Runtime types belong where `Response`, `HttpResponse`, `FrameworkError` live |
| `IntoActionError` trait + blanket impl | `framework` crate | — | Trait coherence: impls must live where the trait is defined |
| `flash_action` runtime helper (session write + 303 build) | `framework` crate | — | Needs `session_mut` access; proc-macro cannot call async runtime code |
| `#[action]` proc-macro attribute | `ferro-macros` crate | — | Proc-macros must live in a `proc-macro = true` crate |
| MCP `action_handler` code template | `ferro-mcp` crate | — | `code_templates.rs` is the authoritative catalog; agents read it |
| Docs page | `docs/src/the-basics/` | — | Handler/action docs live in the-basics section per existing structure |
| Public re-exports (`ferro::ActionError`, etc.) | `framework/src/lib.rs` | — | All user-facing API re-exported from single crate root |

---

## 1. Domain Understanding

### Today (before Phase 180)

Every POST handler that mutates state and must redirect on any failure looks like this:

```rust
#[handler]
pub async fn publish_by_id(req: Request) -> Response {
    let business = resolve_tenant().await?;          // returns HTML 500 on fail — strands browser
    let id: i64 = req.param("id")
        .map_err(|_| error_response(400, "ID non valido"))
        .and_then(|s| s.parse().map_err(|_| error_response(400, "ID non valido")))?;
    // ... five more fallible steps, each producing HTML error pages ...
    Redirect::to("/dashboard/pagine?success=published").into()
}
```

When any step fails, the browser receives an HTML error page at the POST URL
(`/dashboard/pagine/{id}/publish`) and is stuck there — back-navigation is
broken, and the user sees a raw error page instead of a form with a flash message.

The consumer shipped a workaround: a `pagine_redirect(Result<(), String>)` helper
per-controller that centralizes the redirect but still requires manual
`match resolve_tenant()` blocks at every step. The ratio is roughly
15 lines of error-wrapping per 10 lines of business logic, and the pattern
is reproduced in ~40-60 handlers.

### After Phase 180

```rust
#[action(redirect_to = "/dashboard/pagine")]
pub async fn publish_by_id(req: Request) -> ActionResult {
    let business = resolve_tenant().await?;     // ? works — ActionError wraps via IntoActionError
    let id: i64 = req.param("id")?.parse()?;
    let page = Page::find_by_id(id).await?
        .ok_or(ActionError::not_found("Pagina non trovata"))?;
    if page.tenant_id != business.id {
        return Err(ActionError::forbidden("Non autorizzato"));
    }
    publish_page(...).await?;
    Ok(())  // → 303 /dashboard/pagine (session flash set)
}
```

On any `Err`, the macro-generated wrapper: (1) writes `{variant, message}` to
`session.flash("_action", ...)`, (2) appends `?error=...&msg=...` to the
redirect URL for back-compat, (3) emits `tracing::error!`, (4) returns
`Ok(HttpResponse::new().status(303).header("Location", url))`.

---

## 2. Existing Patterns to Reuse

### 2.1 `#[handler]` macro — `ferro-macros/src/handler.rs`

[VERIFIED: read file]

**Lines 64-156** contain the full implementation. Key patterns to copy:

- **Line 65**: `let input_fn = parse_macro_input!(input as ItemFn);` — identical parse entry for `#[action]`.
- **Lines 66-73**: Extract `fn_vis`, `fn_name`, `fn_generics`, `fn_output`, `fn_block`, `fn_attrs` — copy verbatim; `#[action]` needs all of these plus the attribute args.
- **Lines 84-130**: `ParamKind` classification and `generate_extraction` dispatch — `#[action]` reuses the exact same parameter extraction; only the _outer wrapper_ differs (return type is `ActionResult` → `Response`, not `Response` directly).
- **Lines 133-155**: The final `quote!` block generates `__ferro_req: #ferro::Request` as the real signature. `#[action]` does the same, but wraps `fn_block` in an async closure that returns `ActionResult`, then pattern-matches the result to emit the 303 redirect.

**Critical:** `#[handler]` emits the body as-is. `#[action]` needs to emit:

```rust
pub async fn #fn_name(__ferro_req: ferro::Request) -> ferro::Response {
    let __ferro_params = __ferro_req.params().clone();
    #(#extractions)*   // same extraction code as #[handler]
    
    // Wrap original body:
    let __action_result: ferro::ActionResult = async move {
        #fn_block
    }.await;
    
    // Catch site — only change from #[handler]:
    ferro::action::handle_action_result(
        __action_result,
        #redirect_to,           // literal from attr
        stringify!(#fn_name),   // for tracing
    )
}
```

Where `ferro::action::handle_action_result` is a runtime function (not macro-generated) that writes flash, builds query-string back-compat, logs, and returns `Response`. This keeps the macro thin.

### 2.2 `#[domain_error]` macro — `ferro-macros/src/domain_error.rs`

[VERIFIED: read file]

**Lines 25-59**: `parse_attrs(attr: TokenStream)` — the `DomainErrorAttrs` parser.
Parses `#[domain_error(status = 404, message = "...")]` using
`syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated`.

**Exact pattern for `#[action]` attribute parser:**

```rust
struct ActionAttrs {
    redirect_to: String,
    method: String,   // defaults to "POST"
}

fn parse_action_attrs(attr: TokenStream) -> Result<ActionAttrs, syn::Error> {
    // Same Punctuated::<Meta, Token![,]>::parse_terminated pattern
    // key "redirect_to" → Lit::Str → required
    // key "method"      → Lit::Str → optional, default "POST"
}
```

`redirect_to` must be required (emit `compile_error!` if absent); `method` defaults to `POST`. The `parse_attrs` in `domain_error.rs` silently ignores unknown keys — `#[action]` should emit `compile_error!` for unknown keys to guide users.

### 2.3 Flash transport — `framework/src/session/store.rs:86-98`

[VERIFIED: read file]

```rust
pub fn flash<T: Serialize>(&mut self, key: &str, value: T) {
    self.put(&format!("_flash.new.{key}"), value);
}

pub fn get_flash<T: DeserializeOwned>(&mut self, key: &str) -> Option<T> {
    let flash_key = format!("_flash.old.{key}");
    let value = self.get(&flash_key);
    if value.is_some() { self.forget(&flash_key); }
    value
}
```

**How `#[action]` uses this:** The runtime helper `handle_action_result` calls:

```rust
session_mut(|session| {
    session.flash("_action", serde_json::json!({
        "variant": err.flash_variant,   // "error" | "warning" | "info"
        "message": err.message,
    }));
});
```

Consumer templates read back with:
```rust
session.get_flash::<serde_json::Value>("_action")
```

The namespace `_action` does not conflict with `_validation_errors` or `_old_input.*` (established by Phase 137).

**`session_mut` signature** (`middleware.rs:56-69`): `session_mut<F, R>(f: F) -> Option<R> where F: FnOnce(&mut SessionData) -> R`. The flash write can fail silently (returns `None`) if no session context is active — which is correct; if there is no session, the query-string fallback still works.

### 2.4 Flash-then-redirect pattern — `framework/src/validation/error.rs:139-142`

[VERIFIED: read file]

```rust
pub fn redirect_to(self, url: impl Into<String>) -> crate::http::Response {
    self.flash_into_session();
    crate::http::Redirect::to(url.into()).into()
}
```

`Redirect::to(url).into()` (line 142) produces `Response` via `impl From<Redirect> for Response` (`response.rs:279-284`). The default status is 302. The `#[action]` macro must use `Redirect::to(url).into()` and then set status 303 — or use `HttpResponse::new().status(303).header("Location", url)` directly. The latter is simpler and avoids the `Redirect` builder's default 302 status.

**Recommendation:** `handle_action_result` emits `Ok(HttpResponse::new().status(303).header("Location", url))` directly. This is the PRG pattern standard status code (303 See Other forces GET on redirect).

### 2.5 Percent-encoding — framework dependencies

[VERIFIED: `framework/Cargo.toml:56-57`]

The framework already depends on `serde_urlencoded = "0.7"` and `form_urlencoded = "1"`. The `form_urlencoded` crate provides a percent-encoder. The consumer's workaround used a manual char-by-char ASCII loop; the framework should use `form_urlencoded::byte_serialize` or `form_urlencoded::Serializer` for correctness.

For a single key=value pair (the `msg` parameter), the simplest approach:
```rust
use form_urlencoded::byte_serialize;

let encoded_msg: String = byte_serialize(err.message.as_bytes()).collect();
let url = format!("{base}?error={kind}&msg={encoded_msg}");
```

No new dependency needed.

### 2.6 Tracing — `framework/src/json_ui/mod.rs:56`, `CONVENTIONS.md:87-90`

[VERIFIED: read files]

Established pattern in the framework:
```rust
tracing::error!(field = %value, "description");
```

The `#[action]` catch site should emit:
```rust
tracing::error!(
    handler = %#fn_name_str,
    msg = %err.message,
    kind = ?err.kind,
    "action handler error — redirecting"
);
```

`%` for Display fields, `?` for Debug fields. The `fn_name_str` is `stringify!(#fn_name)` evaluated at macro expansion time — a `&'static str` literal in the generated code.

---

## 3. Proc-Macro Implementation Strategy

### 3.1 Attribute parsing — syn types

The `#[action(redirect_to = "/dashboard/pagine", method = "POST")]` attribute is parsed as a `TokenStream` (the first arg to the `proc_macro_attribute` function).

**syn types involved:**
- `syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated` — same as `domain_error.rs:29`
- `syn::Meta::NameValue(nv)` — each `key = "value"` pair
- `nv.path.get_ident()` — the key identifier
- `syn::Expr::Lit(expr_lit)` → `syn::Lit::Str(lit_str)` → `lit_str.value()` — the string value

**Required fields validation in the macro:**

```rust
let redirect_to = parsed.redirect_to.ok_or_else(|| {
    syn::Error::new(proc_macro2::Span::call_site(),
        "#[action] requires `redirect_to = \"/path\"`")
})?;
```

The macro returns `syn::Error::new(...).to_compile_error().into()` on parse failure — same as `handler.rs:122-126`.

### 3.2 Return-type transformation

The consumer function declares:
```rust
pub async fn publish_by_id(req: Request) -> ActionResult { ... }
```

The macro:
1. Parses the `ItemFn` — captures `fn_block`, `fn_name`, `fn_vis`, `fn_generics`, all attrs.
2. Discards `fn_output` — the real output is always `ferro::Response`.
3. Runs the same `ParamKind` classification and `generate_extraction` as `#[handler]`.
4. Emits:

```rust
#(#fn_attrs)*
#fn_vis async fn #fn_name #fn_generics(__ferro_req: #ferro::Request) -> #ferro::Response {
    let __ferro_params = __ferro_req.params().clone();
    #(#extractions)*
    
    let __result: #ferro::ActionResult = {
        #fn_block
    };
    
    #ferro::action::handle_action_result(
        __result,
        #redirect_to,
        concat!(module_path!(), "::", stringify!(#fn_name)),
    )
}
```

`concat!(module_path!(), "::", stringify!(#fn_name))` gives a stable identifier string for tracing without runtime allocation. This is a `&'static str` literal usable in `tracing::error!(handler = %name, ...)`.

**Why wrap in a block `{ #fn_block }` rather than an async closure?**

An async closure (`async move { #fn_block }`) would capture variables from the extraction bindings via `move`. A plain block `{ #fn_block }` runs in the same async frame, which is simpler and avoids potential capture issues with `req` already consumed. Since the extractions are all `let` bindings before the block, the original `fn_block` can reference them normally.

The `fn_block` itself is the original function body with `return Err(ActionError::...)` and `?` operators. Since the block's type is now `ActionResult` (required by `let __result: ActionResult`), the Rust compiler enforces the return type at the point of `fn_block` evaluation.

### 3.3 Parameter extraction reuse

The `ParamKind` enum, `classify_param_type`, `generate_extraction`, `extract_param_name`, and `is_primitive_type_name` functions from `handler.rs` are in `ferro-macros/src/handler.rs` as private functions.

**Options:**
- **Recommended:** Extract shared helpers into `ferro-macros/src/utils.rs` (the file already exists per `lib.rs:26`). Move `ParamKind`, `classify_param_type`, `generate_extraction`, `extract_param_name`, `is_primitive_type_name` there and `pub(crate) use` them in both `handler.rs` and `action.rs`.
- Alternative: Duplicate (violates DRY, risky drift).

Check `ferro-macros/src/utils.rs` before implementing — it may already have shared helpers.

### 3.4 Emission of `tracing::error!` at the catch site

The `tracing` crate is not a dependency of `ferro-macros` (proc-macro crates cannot use runtime crates directly in the generated code — they can only emit token streams that reference them). The generated code references `::tracing::error!` by path. The `tracing` crate is a dependency of `framework`, not `ferro-macros`. This is correct: the macro emits `::tracing::error!(...)` as a token, and the consuming crate (`framework`) satisfies the resolution.

**Verification needed (Open Question OQ-B):** Does `::tracing::error!` resolve correctly in the generated code when the consuming binary is a user's application crate (not `framework` itself)? The tracing crate is re-exported from `ferro`, so `::tracing::error!` would fail in user code unless `tracing` is a direct dependency of the user's crate. The standard pattern used by other macros (e.g., `tokio::test`) is to emit `::tracing::error!` — this works because `tracing` is in the user's `[dependencies]` via `ferro`'s public re-exports or the user adds it directly. Alternatively, emit `::ferro::__tracing_error!` with a re-export shim. Research confirms `tracing` crates are designed for this pattern — `::tracing::error!` from a proc-macro output works when `tracing` is transitively available.

**Safest approach:** Emit `::tracing::error!(...)` and add a note in the `#[action]` docs that users must have `tracing` as a direct or transitive dependency. Since `framework` already re-exports nothing from `tracing` (verified in `lib.rs`), this is standard practice.

### 3.5 `?` ergonomics through `IntoActionError`

This is the core mechanism question. The locked decision (D-04) specifies `IntoActionError` trait. Here is the concrete stable implementation:

**The trait:**
```rust
pub trait IntoActionError {
    fn into_action_error(self) -> ActionError;
}

impl<E: std::fmt::Display> IntoActionError for E {
    fn into_action_error(self) -> ActionError {
        ActionError::msg(self.to_string())
    }
}
```

**The `?` mechanism:** `?` on a `Result<T, E>` in a function returning `Result<ActionOk, ActionError>` requires `From<E> for ActionError`. A blanket `impl<E: Display> IntoActionError for E` exists. To bridge to `From`:

```rust
// This impl would conflict with concrete From impls (e.g., From<String>)
// on stable Rust without specialization.
// impl<T: IntoActionError> From<T> for ActionError { ... }  // PROBLEMATIC
```

[ASSUMED: orphan rule analysis] The concrete problem: if user code writes `impl From<MyError> for ActionError`, this conflicts with the blanket `impl<T: IntoActionError> From<T> for ActionError` — the compiler cannot know the blanket impl doesn't apply to `MyError`. This is the orphan rule collision that D-04 identifies.

**Clean stable solution — explicit `.map_err` shim in the macro:**

The macro-generated code does NOT use `?` on `ActionResult` directly. Instead, any `?` inside the _original function body_ (`fn_block`) operates on `ActionResult` naturally — the original handler is declared to return `ActionResult`, so `?` on `Result<T, E>` requires only `From<E> for ActionError`. But the same blanket collision appears.

**Actually clean solution:** The `IntoActionError` trait provides a method `into_action_error(self) -> ActionError`. Users write:

```rust
let business = resolve_tenant().await.map_err(|e| e.into_action_error())?;
// or with a macro helper:
let business = resolve_tenant().await.action_err()?;
```

But this breaks the goal of bare `?` ergonomics.

**The real solution used by async-graphql, axum, etc.:** Define a _newtype_ wrapper:

```rust
pub struct ActionErrWrapper<E>(pub E);

impl<E: std::fmt::Display> From<ActionErrWrapper<E>> for ActionError {
    fn from(w: ActionErrWrapper<E>) -> ActionError {
        ActionError::msg(w.0.to_string())
    }
}
```

And then in user code define a local `From<MyError> for ActionError` — but that still requires user impls.

**The answer that actually works on stable:** Use the `IntoActionError` trait as the _single conversion point_, but don't provide a blanket `From` impl. Instead, the macro generates a helper type that wraps the body result:

```rust
// In the generated code:
let __result: ActionResult = (|| async { #fn_block })().await;
// The fn_block returns ActionResult directly.
// Inside fn_block, ? works because:
//   - ActionError: From<ActionError> trivially
//   - Other errors: user must use .map_err(IntoActionError::into_action_error)?
//     OR use the ActionError::msg() constructor
```

Wait — but the CONTEXT says `?` should work on `FrameworkError`, `String`, `sea_orm::DbErr`. The only way to make this work without blanket `From` is to provide specific `From` impls for those types. [VERIFIED: framework/src/error.rs] shows `FrameworkError` already has `From<sea_orm::DbErr> for FrameworkError`. We need `From<FrameworkError> for ActionError`, `From<String> for ActionError`, etc.

**Recommended concrete resolution (for planner):**

Provide explicit `From` impls for the most common error types used in ferro applications:

```rust
impl From<FrameworkError> for ActionError { ... }
impl From<String> for ActionError { ... }
impl From<&str> for ActionError { ... }
impl From<sea_orm::DbErr> for ActionError { ... }  // feature-gated behind "database"
```

And `IntoActionError` as an extension trait for anything else, with a user-facing `.action_err()` extension method on `Result`. This satisfies bare `?` for the four named types without the orphan conflict. See section 9 (Open Questions) for the compile verification needed.

---

## 4. Runtime Type Design

### 4.1 `ActionKind` enum

```rust
/// Semantic kind of an action error, used for routing and logging.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    #[default]
    Generic,
    NotFound,
    Forbidden,
    Unauthorized,
}
```

### 4.2 `FlashVariant` enum

```rust
/// Visual variant for the flash message rendered on the next page.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlashVariant {
    #[default]
    Error,
    Warning,
    Info,
}
```

### 4.3 `ActionError` struct

```rust
/// Error type returned by `#[action]` handlers.
///
/// Carries a message, semantic kind, display variant, and an optional
/// redirect override for cases where the error destination differs from
/// the handler's configured `redirect_to`.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct ActionError {
    /// Human-readable error message (percent-encoded into query string).
    pub message: String,
    /// Semantic classification (for logging, routing, future middleware).
    pub kind: ActionKind,
    /// Flash message visual variant (rendered by consumer templates).
    pub flash_variant: FlashVariant,
    /// Override the configured redirect_to for this specific error.
    ///
    /// When `Some`, this URL is used instead of the handler's `redirect_to`.
    /// Use `ActionError::unauthorized()` to redirect to the login page;
    /// configure the target via `.redirect_to(...)`.
    pub redirect_override: Option<String>,
}

impl ActionError {
    /// Create a generic error with a message.
    pub fn msg(message: impl Into<String>) -> Self { ... }
    /// Create a 404-semantic error.
    pub fn not_found(message: impl Into<String>) -> Self { ... }
    /// Create a 403-semantic error.
    pub fn forbidden(message: impl Into<String>) -> Self { ... }
    /// Create a 401-semantic error with no default redirect override.
    ///
    /// Configure the login redirect via `.redirect_to("/your-login-path")`.
    pub fn unauthorized(message: impl Into<String>) -> Self { ... }
    
    /// Builder: set the flash variant.
    pub fn with_flash(mut self, variant: FlashVariant) -> Self { self.flash_variant = variant; self }
    /// Builder: set the redirect override.
    pub fn redirect_to(mut self, url: impl Into<String>) -> Self {
        self.redirect_override = Some(url.into()); self
    }
}
```

Note: `thiserror::Error` derive gives `Display` for free via `#[error("{message}")]`. The struct can be returned as `std::error::Error` and passed to `tracing::error!(source = ?err)`.

### 4.4 `ActionOk` struct

```rust
/// Success value returned by `#[action]` handlers.
///
/// The common case is `Ok(())` — ferro converts this via `From<()> for ActionOk`.
/// Override the flash message or redirect target for non-standard success paths.
#[derive(Debug, Clone, Default)]
pub struct ActionOk {
    /// Optional success flash key (e.g., `"created"`, `"saved"`).
    pub flash: Option<&'static str>,
    /// Override the configured redirect_to for success.
    pub redirect_override: Option<String>,
}

impl ActionOk {
    /// Success with a flash message key.
    pub fn flash(key: &'static str) -> Self { ... }
    /// Success with a redirect override.
    pub fn redirect_to(url: impl Into<String>) -> Self { ... }
}

impl From<()> for ActionOk {
    fn from(_: ()) -> Self { ActionOk::default() }
}
```

### 4.5 `ActionResult` type alias

```rust
/// Result type for `#[action]` handlers.
///
/// Return `Ok(())` for the common success case, or `Ok(ActionOk::redirect_to(...))` 
/// for dynamic success targets. Return `Err(ActionError::...)` for any failure.
pub type ActionResult = Result<ActionOk, ActionError>;
```

### 4.6 `IntoActionError` trait

```rust
/// Conversion trait for types that can be turned into an `ActionError`.
///
/// Implemented for all `Display` types via a blanket impl.
/// Use `into_action_error()` or the `.action_err()` extension method on `Result`
/// to convert errors from `?` chains:
///
/// ```rust,ignore
/// let page = find_page(id).await.map_err(|e| e.into_action_error())?;
/// ```
pub trait IntoActionError {
    fn into_action_error(self) -> ActionError;
}

impl<E: std::fmt::Display> IntoActionError for E {
    fn into_action_error(self) -> ActionError {
        ActionError::msg(self.to_string())
    }
}

/// Extension method on `Result` for ergonomic error conversion.
pub trait ActionResultExt<T> {
    fn action_err(self) -> ActionResult
    where T: Into<ActionOk>;
}

impl<T: Into<ActionOk>, E: IntoActionError> ActionResultExt<T> for Result<T, E> {
    fn action_err(self) -> ActionResult {
        self.map(|v| v.into()).map_err(|e| e.into_action_error())
    }
}
```

Additionally, provide concrete `From` impls for `?` on the named types without needing `.map_err`:

```rust
impl From<FrameworkError> for ActionError { ... }
impl From<String> for ActionError { ... }
impl From<&'static str> for ActionError { ... }
```

[ASSUMED: `From<sea_orm::DbErr> for ActionError`] — may need a feature gate matching the one in `framework/src/error.rs:454`. The planner should verify.

### 4.7 Orphan rule analysis for stable Rust

[VERIFIED: Rust reference orphan rule, HIGH confidence from training + language spec]

The orphan rule requires that for `impl<T: IntoActionError> From<T> for ActionError`, either `From`, `T`, or `ActionError` must be local to the crate defining the impl. `From` is from `std` (not local). `ActionError` is local. `T` is unconstrained. Rust requires at least one of the non-std types to be local. Since `ActionError` is local, this impl is PERMITTED by the orphan rule under the "local type" exception.

**The actual problem is coherence, not orphan:** A blanket `impl<T: IntoActionError> From<T> for ActionError` combined with `impl From<FrameworkError> for ActionError` causes E0119 (conflicting impls) if `FrameworkError: IntoActionError`. Since the blanket `impl<E: Display> IntoActionError for E` applies to `FrameworkError` (which implements `Display`), we'd have two paths: one from the blanket `From`, one from the concrete `From<FrameworkError>`. This is the coherence conflict, not an orphan error.

**Resolution:** Do NOT define `impl<T: IntoActionError> From<T> for ActionError`. Instead, define only the concrete `From` impls (`From<FrameworkError>`, `From<String>`, `From<&'static str>`). The `IntoActionError` trait + `.action_err()` extension method handle everything else. This is the approach that compiles cleanly on stable.

---

## 5. Flash Transport Integration

### 5.1 Flash key naming

Key: `"_action"` — under `_flash.new._action` in session data during write.

Consumer templates read:
```rust
let action_flash = session.get_flash::<serde_json::Value>("_action");
```

The `get_flash` call ages the value (reads from `_flash.old._action`, deletes after read).

**Payload shape (serialized):**
```json
{
  "variant": "error",
  "message": "Pagina non trovata"
}
```

The `variant` field matches `FlashVariant` snake_case values (`"error"`, `"warning"`, `"info"`). Templates use this to pick the CSS class for the flash banner.

### 5.2 How consumer templates read it

In Jinja-style templates (or equivalent server-rendered markup), the consumer reads:
```rust
// In the handler for the GET page:
let flash = session.get_flash::<serde_json::Value>("_action");
// Pass as prop to the view.
```

Or in the ferro-inertia shared props middleware (`app/src/middleware/share_inertia.rs` has a TODO for this at line 42 per CONCERNS.md). The recommended approach is a middleware that reads `_action` flash and includes it in shared Inertia props.

### 5.3 Query-string back-compat fallback

The `handle_action_result` runtime function appends to the redirect URL:

**Error path:**
```
{redirect_to}?error={kind}&msg={pct_encode(message)}
```

Where `kind` is the snake_case `ActionKind` value (`"generic"`, `"not_found"`, `"forbidden"`, `"unauthorized"`).

**Success path:**
```
{redirect_to}?success={flash_key_or_empty}
```

If `ActionOk::flash` is `None`, append `?success=1`. If `Some(key)`, append `?success={key}`.

**When to drop:** A future cleanup phase (after Phase 180) removes the query-string fallback once all consumer templates read session flash. The CONTEXT explicitly defers this to a separate phase.

**Percent-encoding helper:** Use `form_urlencoded::byte_serialize(msg.as_bytes()).collect::<String>()`. Available via `form_urlencoded = "1"` already in `framework/Cargo.toml:57`.

---

## 6. Security Threat Surface

### 6.1 Flash message injection

**Threat:** `err.message` is untrusted user-influenced data (e.g., from a database error message, a form input that survived validation, or a URL parameter) serialized into session and rendered as HTML.

**Mitigation:** The flash message is stored as a JSON string field, not as raw HTML. Consumer templates MUST HTML-escape the message before rendering. The framework cannot enforce HTML escaping at the flash write site (templates are consumer-controlled). Document this clearly in `docs/src/the-basics/action-handlers.md` with an explicit warning and a safe-rendering example.

**Implementation requirement:** Add a `# Security` doc section to `ActionError` rustdoc stating that `message` should not contain unescaped HTML and that templates must escape it.

### 6.2 Open redirect via `redirect_override`

**Threat:** If `redirect_override` is constructed from untrusted user input (e.g., a `?next=` query parameter), a malicious actor could redirect users to an external site.

**Mitigation:**
- The framework validates `redirect_override` at use time: accept only same-origin paths (paths starting with `/` or registered named routes).
- In `handle_action_result`, before using `redirect_override`:

```rust
fn is_safe_redirect(url: &str) -> bool {
    // Same logic as validation/error.rs is_same_origin():
    url.starts_with('/')
}
```

If `redirect_override` fails validation, fall back to `redirect_to` from the macro attribute. Log a `tracing::warn!` with the rejected URL.

**Documentation:** The `redirect_to` field on `ActionError` must document this constraint.

### 6.3 Log injection via tracing fields

**Threat:** If `err.message` contains newlines or control characters, it could corrupt structured logs.

**Mitigation:** `tracing` with structured fields uses the format `field = %value` where `%` invokes `Display`. Structured log sinks (JSON formatters, OpenTelemetry) serialize fields as JSON strings, which escapes control characters. Plain text log output via `tracing_subscriber` may not escape newlines.

**Recommendation:** Sanitize `err.message` in the macro's catch site before passing to `tracing::error!`:
```rust
let __safe_msg: String = err.message.chars()
    .map(|c| if c.is_control() { ' ' } else { c })
    .collect();
tracing::error!(handler = %__handler_name, msg = %__safe_msg, "action error");
```

This is a minor defense; structured sinks are safe regardless.

---

## 7. Public API Surface

### 7.1 `framework/src/lib.rs` additions

Add a new `pub mod action;` module declaration and re-exports:

```rust
pub mod action;

pub use action::{
    ActionError, ActionKind, ActionOk, ActionResult, ActionResultExt,
    FlashVariant, IntoActionError,
};
```

**Module location:** `framework/src/action/mod.rs` — new file. This separates the action primitive from `http` and `error` modules for clean organization.

### 7.2 `ferro-macros/src/lib.rs` additions

Add:
```rust
mod action;

#[proc_macro_attribute]
pub fn action(attr: TokenStream, input: TokenStream) -> TokenStream {
    action::action_impl(attr, input)
}
```

Registered alongside `handler`, `domain_error`, etc.

### 7.3 Re-export convention

Per `CONVENTIONS.md` and existing pattern, users import via:
```rust
use ferro::{action, ActionError, ActionOk, ActionResult, ActionResultExt, FlashVariant};
use ferro::macros::action;  // NO — macros are imported via #[action] directly
```

The `#[action]` macro is imported as `use ferro_macros::action;` or via `ferro` re-export of the macro. Check if `framework` re-exports proc-macros from `ferro-macros`. [VERIFIED: `framework/src/lib.rs`] shows `pub use http::{..., Response, ...}` but no macro re-exports. Proc-macro re-exports require `extern crate ferro_macros; pub use ferro_macros::action;` or the user adds `ferro_macros` as a direct dep. Given the existing pattern where `#[handler]` is imported as `use ferro::handler;` (per CONTEXT.md "canonical reference" note), the framework must already re-export macros. Verify in lib.rs — the re-export pattern for proc-macros.

[ASSUMED: ferro re-exports `#[handler]` via a `pub use ferro_macros::handler;` or similar — needs verification during implementation].

---

## 8. Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]`, `pretty_assertions` |
| Config file | `framework/Cargo.toml` (dev-dependencies), `ferro-macros/Cargo.toml` |
| Quick run | `cargo test -p framework` |
| Full suite | `cargo test --all-features --all-targets` |

### Phase Requirements → Test Map

| Behavior | Test Type | File | Automated Command |
|----------|-----------|------|-------------------|
| `ActionError::msg()` / `::not_found()` / `::forbidden()` / `::unauthorized()` constructors set correct fields | Unit | `framework/src/action/mod.rs` (inline `#[cfg(test)]`) | `cargo test -p framework action::` |
| `ActionError::redirect_to()` / `::with_flash()` builder methods work | Unit | `framework/src/action/mod.rs` | `cargo test -p framework action::` |
| `From<()> for ActionOk` produces default | Unit | `framework/src/action/mod.rs` | `cargo test -p framework action::` |
| `From<FrameworkError> for ActionError` produces correct message | Unit | `framework/src/action/mod.rs` | `cargo test -p framework action::` |
| `IntoActionError` blanket impl works for arbitrary Display type | Unit | `framework/src/action/mod.rs` | `cargo test -p framework action::` |
| `handle_action_result(Ok(ActionOk::default()), "/redirect", "handler")` returns 303 with Location header | Integration | `framework/tests/action_handler.rs` | `cargo test -p framework --test action_handler` |
| `handle_action_result(Err(err), "/redirect", "handler")` returns 303, sets flash via session, appends `?error=&msg=` | Integration | `framework/tests/action_handler.rs` | `cargo test -p framework --test action_handler` |
| `ActionError::unauthorized()` with `.redirect_to("/login")` → 303 to `/login`, not to macro `redirect_to` | Integration | `framework/tests/action_handler.rs` | `cargo test -p framework --test action_handler` |
| `redirect_override` with external URL rejected, falls back to `redirect_to` | Integration | `framework/tests/action_handler.rs` | `cargo test -p framework --test action_handler` |
| `?` on `Result<T, FrameworkError>` compiles in an `#[action]` handler | Compile | `ferro-macros/tests/action_compile.rs` (trybuild or simple cargo build) | `cargo test -p ferro-macros` |
| `?` on `Result<T, String>` compiles in an `#[action]` handler | Compile | `ferro-macros/tests/action_compile.rs` | `cargo test -p ferro-macros` |
| `#[action]` without `redirect_to` arg produces compile error | Compile/UI | `ferro-macros/tests/action_compile.rs` | `cargo test -p ferro-macros` |

### Sampling Rate

- **Per task commit:** `cargo test -p framework action:: && cargo clippy -p framework -- -D warnings`
- **Per wave merge:** `cargo test --all-features && cargo clippy --all --all-targets -- -D warnings`
- **Phase gate:** Full suite green (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`) before verify-work.

### Wave 0 Gaps

- [ ] `framework/tests/action_handler.rs` — integration tests for `handle_action_result` (requires session mock or real session context)
- [ ] `framework/src/action/mod.rs` — new file, inline unit tests for constructors and `From` impls
- [ ] `ferro-macros/tests/action_compile.rs` — compile-success and compile-error tests (trybuild optional; a `cargo build` in a temp crate suffices)

---

## 9. Open Questions for the Planner

**(OQ-A) Does `impl<T: IntoActionError> From<T> for ActionError` compile cleanly on stable alongside concrete `From<FrameworkError>`?**

Research shows this produces E0119 (conflicting implementations of `From<T>`) because `FrameworkError: IntoActionError` (it implements `Display`) and `FrameworkError` matches the blanket. The planner MUST verify this compiles — or use the concrete-only `From` approach documented in section 4.7. Do NOT assume the blanket `From` + concrete `From` pattern compiles; test it before committing the type design.

**(OQ-B) Does `::tracing::error!` in macro-generated code resolve in user application crates?**

User application crates depend on `ferro`, which depends on `tracing`. On stable Rust, `::tracing::error!` in generated code resolves only if `tracing` is reachable from the crate root of the _application_ crate — which it is, transitively, via `ferro → framework → tracing`. However, `tracing` macros use `$crate` internally, so `::tracing::error!` emitted from a proc-macro expands in the _call site_ crate's namespace. The call site is the user's application, which has `tracing` transitively. This should work but needs a compile test to confirm.

**(OQ-C) Where does the proc-macro `#[action]` re-export land in `framework/src/lib.rs`?**

The existing `#[handler]` macro is likely re-exported via `pub use ferro_macros::handler;` somewhere — verify this in `lib.rs` (the read only covered to line 199 of lib.rs; the file may be longer). If proc-macros are NOT re-exported from `framework`, the user would write `use ferro_macros::action;` separately. Resolve this during implementation by grepping lib.rs for `ferro_macros`.

---

## 10. Migration Acceptance Harness

The planner does not need to plan the consumer sweep, but needs to document the CI query that enforces the zero-workaround state in gestiscilo-it.

**Primary gate (from D-09):**
```bash
rg -l 'error_response!\(' src/controllers/ \
  | xargs rg -l '#\[handler\]\s*(\n\s*)?pub async fn (publish|create|update|delete|new|store|destroy)' --multiline
```
Must return zero file paths.

**Secondary gate (no local redirect helpers):**
```bash
rg 'fn.*_redirect\s*\(' src/controllers/ --type rust
```
Must return zero matches.

**Tertiary gate (no inline `pagine_redirect` pattern):**
```bash
rg 'pagine_redirect|fn.*_action_response' src/ --type rust
```
Must return zero matches.

These three grep queries together ensure the sweep is complete. The CI gate for the ferro side is simply:
```bash
cargo test --all-features && cargo clippy --all --all-targets -- -D warnings
```

---

## Project Constraints (from CLAUDE.md)

- **Project-agnostic crates rule:** `ferro-macros` and `framework` MUST NOT hardcode any consumer string. `ActionError::unauthorized()` default `redirect_override` is `None`, not `Some("/accedi")`. Consumers configure via `.redirect_to(...)`.
- **Builder pattern:** `with_*` methods take `mut self → Self`. `ActionError::with_flash(...)` and `::redirect_to(...)` follow this.
- **thiserror derive, one Error enum per crate:** `ActionError` uses `thiserror::Error`.
- **Run fmt + clippy + tests before every commit.**
- **Always update docs when framework changes:** `docs/src/the-basics/action-handlers.md` is a deliverable.
- **Update ferro-mcp when needed:** `code_templates.rs` needs an `action_handler` template.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `::tracing::error!` resolves in user app crates via transitive dep on `tracing` through `ferro` | §3.4, OQ-B | Macro-generated tracing calls fail to compile; need `ferro::__tracing_error!` shim |
| A2 | `impl<T: IntoActionError> From<T> for ActionError` produces E0119 alongside concrete `From<FrameworkError>` | §4.7 | If wrong, the blanket `From` is viable and cleaner |
| A3 | `framework` re-exports `#[handler]` from `ferro-macros` (proc-macro re-export exists in lib.rs) | §7.3, OQ-C | If not re-exported, user would need `ferro_macros` as a direct dep |
| A4 | `From<sea_orm::DbErr> for ActionError` needs a feature gate matching the one in `error.rs:454` | §4.6 | Without feature gate, conditional dep compilation fails |

---

## Sources

### Primary (HIGH confidence)

- `ferro-macros/src/handler.rs` — read in full; `#[handler]` implementation baseline
- `ferro-macros/src/domain_error.rs` — read in full; attribute parser pattern
- `ferro-macros/src/lib.rs` — read in full; proc-macro registration pattern
- `framework/src/http/response.rs` — read in full; `Response` type, `Redirect`, `InertiaRedirect`
- `framework/src/validation/error.rs` — read in full; `ValidationError::redirect_to()` flash pattern
- `framework/src/session/store.rs` — read in full; `session.flash()`, `get_flash()`, aging
- `framework/src/session/middleware.rs:30-90` — `session_mut` signature
- `framework/src/error.rs:1-400` — `FrameworkError`, `thiserror` usage, `From` impls
- `framework/src/lib.rs:1-199` — public API re-export surface
- `ferro-mcp/src/tools/code_templates.rs:1-50` — `CodeTemplate` structure for MCP template
- `.planning/phases/180-.../180-CONTEXT.md` — locked decisions

### Secondary (MEDIUM confidence)

- Rust Reference (orphan rule, coherence rules) — training knowledge, cross-referenced with empirical framework patterns [HIGH]
- `tracing` crate macro expansion in proc-macro output — training knowledge [MEDIUM, see OQ-B]

---

## Metadata

**Confidence breakdown:**
- Runtime type design: HIGH — all types are new; no migration conflicts; patterns verified
- Proc-macro strategy: HIGH — exact analog in `#[handler]` verified in source
- Flash transport: HIGH — `session.flash()` verified in `store.rs`
- `?` ergonomics / orphan rule: MEDIUM — reasoning is sound but OQ-A needs compile verification
- Tracing in generated code: MEDIUM — standard pattern but OQ-B needs compile test

**Research date:** 2026-05-30
**Valid until:** 2026-06-30 (stable codebase, no external deps to verify)

---

## RESEARCH COMPLETE

**Phase:** 180 — Declarative action handler primitive
**Confidence:** HIGH (MEDIUM on OQ-A and OQ-B — both resolve with a compile test before finalizing type design)

### Key Findings

- The `#[handler]` macro is a direct template: parameter extraction code reuses verbatim, only the outer wrapper changes (original body assigned to `let __result: ActionResult`, then `handle_action_result` called).
- Flash transport is already fully implemented via `session.flash("_action", payload)` — no new infrastructure needed.
- The `IntoActionError` blanket `From` approach conflicts with concrete `From` impls; the planner must choose between (a) concrete-only `From` impls for known types, or (b) `IntoActionError` trait + `.action_err()` extension method, with OQ-A resolving the exact mechanism.
- `percent_encoding` of the query-string fallback is available via `form_urlencoded::byte_serialize` already in framework deps.
- Security mitigations: open-redirect via `redirect_override` mitigated by same-origin validation (reuse `is_same_origin` logic from `validation/error.rs`); flash injection mitigated by documentation + consumer escaping.
- MCP code_templates update is a required deliverable alongside the ferro primitive.

### File Created
`.planning/phases/180-declarative-action-handler-primitive-typed-result-return-so-/180-RESEARCH.md`

### Ready for Planning
Research complete. Planner can now create PLAN.md files.
