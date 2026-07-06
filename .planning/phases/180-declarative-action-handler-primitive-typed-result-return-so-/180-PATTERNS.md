# Phase 180: Declarative Action Handler Primitive — Pattern Map

**Mapped:** 2026-05-30
**Files analyzed:** 9 (6 new + 3 modified)
**Analogs found:** 8 / 9 (1 no analog: `IntoActionError` trait)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `framework/src/action/mod.rs` | utility/runtime-types | request-response | `framework/src/validation/error.rs` | role-match (flash+redirect shape identical) |
| `framework/tests/action_handler.rs` | test | request-response | `framework/tests/validation_derive.rs` | role-match |
| `ferro-macros/src/action.rs` | proc-macro | transform | `ferro-macros/src/handler.rs` | exact (parameter extraction verbatim) |
| `ferro-macros/tests/action_macro.rs` | test | — | `framework/tests/validation_derive.rs` | partial (no ferro-macros tests dir yet) |
| `ferro-mcp/src/tools/code_templates.rs` | utility | — | itself (modification, existing template pattern) | exact |
| `docs/src/the-basics/action-handlers.md` | docs | — | `docs/src/the-basics/controllers.md` | role-match |
| `framework/src/http/mod.rs` | config/module | — | itself | exact (add `pub mod action;`) |
| `framework/src/lib.rs` | config/re-export | — | lines 310-326 of itself (existing macro re-exports) | exact |
| `ferro-macros/src/lib.rs` | config/registration | — | lines 231-233 of itself (`handler` registration) | exact |

---

## Pattern Assignments

### `framework/src/action/mod.rs` (utility, request-response)

**Analog:** `framework/src/validation/error.rs`

**Imports pattern** (error.rs lines 1-9):
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
```

For `action/mod.rs`, adapt to:
```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;
```

**thiserror struct pattern** — `AppError` in `framework/src/error.rs` lines 64-113:
```rust
// error.rs:64-68
#[derive(Debug, Clone)]
pub struct AppError {
    message: String,
    status_code: u16,
}
```
`ActionError` follows same shape but uses `thiserror::Error` derive. Copy the constructor naming pattern from `AppError::not_found`, `::unauthorized`, `::forbidden` (error.rs lines 86-103):
```rust
pub fn not_found(message: impl Into<String>) -> Self {
    Self::new(message).status(404)
}
pub fn bad_request(message: impl Into<String>) -> Self {
    Self::new(message).status(400)
}
pub fn unauthorized(message: impl Into<String>) -> Self {
    Self::new(message).status(401)
}
pub fn forbidden(message: impl Into<String>) -> Self {
    Self::new(message).status(403)
}
```

**Builder pattern** — CLAUDE.md: "Builder pattern: `with_*` methods taking `mut self → Self` (consuming)". Pattern seen in `AppError::status(mut self, code: u16) -> Self` (error.rs line 80):
```rust
pub fn status(mut self, code: u16) -> Self {
    self.status_code = code;
    self
}
```

**Flash-then-redirect pattern** — `framework/src/validation/error.rs:139-142` (EXACT pattern to replicate):
```rust
pub fn redirect_to(self, url: impl Into<String>) -> crate::http::Response {
    self.flash_into_session();
    crate::http::Redirect::to(url.into()).into()
}
```
The `handle_action_result` runtime helper follows the same shape. Note: use `HttpResponse::new().status(303)` directly instead of `Redirect` (which defaults to 302).

**Session flash write pattern** — `framework/src/validation/error.rs:148-158`:
```rust
fn flash_into_session(self) {
    let errors = self.errors;
    let old = self.old_input;
    crate::session::session_mut(|session| {
        session.flash("_validation_errors", &errors);
        // ...
    });
}
```
The `handle_action_result` helper calls `crate::session::session_mut(|session| { session.flash("_action", payload); })` using the same pattern.

**`session.flash()` and `session_mut` signatures** — `framework/src/session/store.rs:86-98`:
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

**`session_mut` call signature** (middleware.rs): `session_mut<F, R>(f: F) -> Option<R> where F: FnOnce(&mut SessionData) -> R`. Returns `Option<R>` — flash write may return `None` if no session; this is acceptable.

**Open-redirect validation** — `framework/src/validation/error.rs:172-179`:
```rust
fn is_same_origin(url: &str) -> bool {
    if url.starts_with('/') {
        return true;
    }
    false
}
```
`handle_action_result` applies the same `is_same_origin` check to `redirect_override` before using it. Reject and fall back to the macro's `redirect_to` on failure, emitting `tracing::warn!`.

**Serde enum pattern** — CLAUDE.md: "Serde enums: `#[serde(rename_all = "snake_case")]`". For `ActionKind` and `FlashVariant`:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    #[default]
    Generic,
    NotFound,
    Forbidden,
    Unauthorized,
}
```

**Percent-encoding** — `framework/Cargo.toml` already has `form_urlencoded = "1"`. Use:
```rust
use form_urlencoded::byte_serialize;
let encoded_msg: String = byte_serialize(err.message.as_bytes()).collect();
```

**`From<()> for ActionOk`** — no exact analog in codebase; use standard Rust pattern:
```rust
impl From<()> for ActionOk {
    fn from(_: ()) -> Self { ActionOk::default() }
}
```

---

### `ferro-macros/src/action.rs` (proc-macro, transform)

**Analog:** `ferro-macros/src/handler.rs` (EXACT — parameter extraction reused verbatim)

**Entry point pattern** (handler.rs lines 64-75):
```rust
pub fn handler_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let ferro = ferro();

    let fn_vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_output = &input_fn.sig.output;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    // ...
}
```
`action_impl(attr: TokenStream, input: TokenStream)` copies this verbatim, PLUS parses `attr` for `redirect_to`/`method` before parsing `input`.

**Attribute parsing pattern** (domain_error.rs lines 25-59):
```rust
fn parse_attrs(attr: TokenStream) -> DomainErrorAttrs {
    let mut result = DomainErrorAttrs::default();

    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let metas = match syn::parse::Parser::parse(parser, attr) {
        Ok(metas) => metas,
        Err(_) => return result,
    };

    for meta in metas {
        if let Meta::NameValue(nv) = meta {
            let key = nv.path.get_ident().map(|i| i.to_string());

            match key.as_deref() {
                Some("status") => {
                    if let Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Int(lit_int) = &expr_lit.lit {
                            if let Ok(val) = lit_int.base10_parse::<u16>() {
                                result.status = val;
                            }
                        }
                    }
                }
                Some("message") => {
                    if let Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            result.message = Some(lit_str.value());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    result
}
```
For `#[action]`, `redirect_to` uses `Lit::Str` (like `message`). The parser must emit `compile_error!` if `redirect_to` is absent (unlike `domain_error.rs` which silently ignores missing fields). Use `syn::Error::new(proc_macro2::Span::call_site(), "...")`.

**Compile-error emission pattern** (handler.rs lines 122-128):
```rust
FnArg::Receiver(_) => {
    return syn::Error::new_spanned(
        param,
        "#[handler] does not support methods with self receiver",
    )
    .to_compile_error()
    .into();
}
```

**Parameter extraction** — `ParamKind`, `classify_param_type`, `generate_extraction`, `extract_param_name`, `is_primitive_type_name` from handler.rs lines 17-270. These are currently private to `handler.rs`. The recommended approach is to move them to `ferro-macros/src/utils.rs` (file exists but currently only contains `levenshtein_distance`). Move as `pub(crate)` functions.

**Generated code pattern** (handler.rs lines 133-155):
```rust
quote! {
    #(#fn_attrs)*
    #fn_vis #async_token fn #fn_name #fn_generics(__ferro_req: #ferro::Request) #fn_output {
        let __ferro_params = __ferro_req.params().clone();
        #(#extractions)*
        #fn_block
    }
}
```
For `#[action]`, replace `#fn_block` with the catch wrapper:
```rust
quote! {
    #(#fn_attrs)*
    #fn_vis async fn #fn_name #fn_generics(__ferro_req: #ferro::Request) -> #ferro::Response {
        let __ferro_params = __ferro_req.params().clone();
        #(#extractions)*

        let __action_result: #ferro::ActionResult = {
            #fn_block
        };

        #ferro::action::handle_action_result(
            __action_result,
            #redirect_to,
            concat!(module_path!(), "::", stringify!(#fn_name)),
        )
    }
}
```
Note: `fn_output` is DISCARDED (replaced by `-> #ferro::Response`). `asyncness` is always forced to `async` for `#[action]` handlers.

**`ferro()` helper** (handler.rs lines 12-14):
```rust
fn ferro() -> TokenStream2 {
    quote!(::ferro)
}
```
Copy verbatim into `action.rs`.

---

### `ferro-macros/tests/action_macro.rs` (test, proc-macro integration)

**No existing ferro-macros/tests/ directory.** The directory must be created. The testing pattern follows `framework/tests/validation_derive.rs`.

**Import pattern** (validation_derive.rs lines 5-8):
```rust
extern crate ferro_rs as ferro;
use ferro_rs::validation::Validatable;
use ferro_rs::ValidateRules;
use serde::{Deserialize, Serialize};
```
For `action_macro.rs`:
```rust
extern crate ferro_rs as ferro;
use ferro::{ActionError, ActionOk, ActionResult, FlashVariant, ActionKind};
```

**Test structure** (validation_derive.rs lines 23-44):
```rust
#[test]
fn test_basic_validation_passes() {
    let request = BasicRequest { /* ... */ };
    assert!(request.validate().is_ok());
}
```

**Note on trybuild tests:** No existing `ferro-macros/tests/ui/` directory. Create fresh. Trybuild test structure uses `trybuild::TestCases` — no existing analog in this repo to copy from, use trybuild crate's documented pattern.

---

### `framework/tests/action_handler.rs` (test, integration)

**Analog:** `framework/tests/validation_derive.rs` (role-match) and `framework/tests/pipeline_order.rs` (feature-gating pattern)

**Crate alias pattern** (validation_derive.rs line 5):
```rust
extern crate ferro_rs as ferro;
```

**Struct + test pattern** (validation_derive.rs lines 14-44):
```rust
#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct BasicRequest {
    #[rule(required)]
    name: String,
}

#[test]
fn test_basic_validation_passes() {
    let request = BasicRequest { name: "John Doe".to_string() };
    assert!(request.validate().is_ok());
}
```

**Response inspection pattern** (pipeline_order.rs lines 20-25):
```rust
fn html_body(result: ferro_rs::http::Response) -> String {
    match result {
        Ok(r) => r.body().to_string(),
        Err(r) => r.body().to_string(),
    }
}
```
For `action_handler.rs`, tests inspect the 303 Location header on the returned `Response`. Create a helper:
```rust
fn location_header(result: ferro_rs::Response) -> String {
    match result {
        Ok(r) => r.header("Location").unwrap_or_default().to_string(),
        Err(r) => r.header("Location").unwrap_or_default().to_string(),
    }
}
```

**Important:** `handle_action_result` requires a live session context (via `session_mut`). Tests that invoke it directly may need to initialize the session store (same setup as `ferro_test` attribute in the framework). Check if `testing::TestDatabase` or a lightweight session mock is needed for integration tests.

---

### `ferro-mcp/src/tools/code_templates.rs` (modification, utility)

**Analog:** Itself — `handler_templates()` function at lines 81-159.

**`CodeTemplate` struct pattern** (code_templates.rs lines 82-126):
```rust
CodeTemplate {
    name: "index_handler".to_string(),
    category: "handler".to_string(),
    description: "List all resources with pagination using ResourceCollection".to_string(),
    code: r#"#[handler]
pub async fn index(req: Request) -> Response {
    // ...
}"#.to_string(),
    imports: vec![
        "use ferro::{handler, Request, Response, ...};".to_string(),
    ],
    placeholders: vec![
        Placeholder {
            name: "{{Entity}}".to_string(),
            description: "Model name in PascalCase".to_string(),
            example: "User".to_string(),
        },
    ],
},
```

New template to add in `handler_templates()`:
```rust
CodeTemplate {
    name: "action_handler".to_string(),
    category: "handler".to_string(),
    description: "POST action handler that mutates state and redirects (PRG pattern). Use for any POST endpoint that mutates-then-redirects.".to_string(),
    code: r#"#[action(redirect_to = "/dashboard/{{resource}}")]
pub async fn {{action}}(req: Request) -> ActionResult {
    let id: i64 = req.param("id")?.parse()?;
    let record = {{Entity}}::find_by_id(id).await?
        .ok_or(ActionError::not_found("{{Entity}} not found"))?;
    // perform mutation
    {{Entity}}::save(record).await?;
    Ok(())
}"#.to_string(),
    imports: vec![
        "use ferro::{action, ActionError, ActionOk, ActionResult, Request};".to_string(),
        "use crate::entities::{{entity}}::Entity as {{Entity}};".to_string(),
    ],
    placeholders: vec![
        Placeholder {
            name: "{{resource}}".to_string(),
            description: "Dashboard resource path segment".to_string(),
            example: "prodotti".to_string(),
        },
        Placeholder {
            name: "{{action}}".to_string(),
            description: "Handler function name".to_string(),
            example: "publish_by_id".to_string(),
        },
        Placeholder {
            name: "{{Entity}}".to_string(),
            description: "Model name in PascalCase".to_string(),
            example: "Product".to_string(),
        },
        Placeholder {
            name: "{{entity}}".to_string(),
            description: "Model name in snake_case".to_string(),
            example: "product".to_string(),
        },
    ],
},
```

---

### `framework/src/http/mod.rs` (modification, config)

**Analog:** Itself — existing module declarations at lines 1-11.

**Module declaration pattern** (http/mod.rs lines 1-10):
```rust
mod body;
pub mod cookie;
mod extract;
mod form_request;
mod multipart;
mod request;
pub mod request_context;
pub mod resources;
mod response;
```
Add `pub mod action;` in the same list. However, the RESEARCH.md recommends `framework/src/action/mod.rs` as a top-level framework module (not nested under `http`). Verify in planning: the file is `framework/src/action/mod.rs` so the declaration goes in `framework/src/lib.rs` as `pub mod action;`, NOT in `http/mod.rs`. The "Modified files" list in the task description says `framework/src/http/mod.rs` — this may be wrong if the module lands at crate root. Planner must resolve: top-level `pub mod action` in `lib.rs` vs `http/action.rs`. Research says `framework/src/action/mod.rs`.

---

### `framework/src/lib.rs` (modification, re-export)

**Analog:** Lines 310-326 of itself — existing macro re-exports.

**Macro re-export pattern** (lib.rs lines 310-322):
```rust
pub use ferro_macros::domain_error;
pub use ferro_macros::ferro_test;
pub use ferro_macros::handler;
pub use ferro_macros::inertia_response;
pub use ferro_macros::injectable;
pub use ferro_macros::redirect;
pub use ferro_macros::request;
pub use ferro_macros::service;
pub use ferro_macros::ApiResource;
pub use ferro_macros::FerroModel;
pub use ferro_macros::FormRequest as FormRequestDerive;
pub use ferro_macros::InertiaProps;
pub use ferro_macros::ValidateRules;
```
Add `pub use ferro_macros::action;` in the same block.

**Runtime type re-export pattern** (lib.rs lines 105-110):
```rust
pub use http::{
    bytes, json, request_host, text, validate_mime, validate_size, Cookie, CookieOptions,
    FormRequest, FromParam, FromRequest, HttpResponse, InertiaRedirect, MultipartForm,
    PaginationLinks, PaginationMeta, Redirect, Request, Resource, ResourceCollection, ResourceMap,
    Response, ResponseExt, SameSite, UploadedFile,
};
```
Add separate:
```rust
pub mod action;
pub use action::{
    ActionError, ActionKind, ActionOk, ActionResult, ActionResultExt, FlashVariant, IntoActionError,
};
```

---

### `ferro-macros/src/lib.rs` (modification, registration)

**Analog:** Lines 231-233 of itself — `handler` proc-macro registration.

**Registration pattern** (lib.rs lines 230-233):
```rust
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, input: TokenStream) -> TokenStream {
    handler::handler_impl(attr, input)
}
```
Add:
```rust
mod action;

#[proc_macro_attribute]
pub fn action(attr: TokenStream, input: TokenStream) -> TokenStream {
    action::action_impl(attr, input)
}
```
Place the `mod action;` declaration in the `mod` block at lines 13-26, alongside `mod handler;`.

---

## Shared Patterns

### Session flash write
**Source:** `framework/src/validation/error.rs:148-158`, `framework/src/session/store.rs:86-88`
**Apply to:** `framework/src/action/mod.rs` (`handle_action_result`), `framework/tests/action_handler.rs` (test setup)
```rust
crate::session::session_mut(|session| {
    session.flash("_action", &payload);
});
```

### Open-redirect mitigation
**Source:** `framework/src/validation/error.rs:172-179`
**Apply to:** `framework/src/action/mod.rs` (`handle_action_result` — validate `redirect_override`)
```rust
fn is_safe_redirect(url: &str) -> bool {
    url.starts_with('/')
}
```
If `redirect_override` fails: fall back to `redirect_to`, emit `tracing::warn!(rejected_url = %url, "redirect_override rejected: not same-origin")`.

### thiserror Error derive pattern
**Source:** `framework/src/error.rs:7-8`, `ferro-macros/src/domain_error.rs` (generated pattern)
**Apply to:** `framework/src/action/mod.rs` (`ActionError`)
```rust
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct ActionError {
    pub message: String,
    // ...
}
```

### Builder consuming pattern
**Source:** `framework/src/error.rs:80-83` (`AppError::status`)
**Apply to:** `ActionError::with_flash()`, `ActionError::redirect_to()`
```rust
pub fn with_flash(mut self, variant: FlashVariant) -> Self {
    self.flash_variant = variant;
    self
}
```

### Tracing error emission
**Source:** existing framework tracing usage — `tracing::error!(field = %value, "description")`
**Apply to:** `handle_action_result` catch site
```rust
let __safe_msg: String = err.message.chars()
    .map(|c| if c.is_control() { ' ' } else { c })
    .collect();
tracing::error!(
    handler = %handler_name,
    msg = %__safe_msg,
    kind = ?err.kind,
    "action handler error — redirecting"
);
```
Note: emit `::tracing::error!` (absolute path) in macro-generated code, not `tracing::error!`. The crate is available transitively via `ferro`.

### Proc-macro ferro() path helper
**Source:** `ferro-macros/src/handler.rs:12-14`
**Apply to:** `ferro-macros/src/action.rs`
```rust
fn ferro() -> TokenStream2 {
    quote!(::ferro)
}
```

### Integration test crate alias
**Source:** `framework/tests/validation_derive.rs:5`
**Apply to:** `framework/tests/action_handler.rs`, `ferro-macros/tests/action_macro.rs`
```rust
extern crate ferro_rs as ferro;
```

---

## No Analog Found

| File | Role | Reason |
|------|------|--------|
| `IntoActionError` trait (in `action/mod.rs`) | utility trait | No existing extension trait in the framework for error conversion. Pattern is newly introduced. Planner designs from scratch using the trait + blanket impl described in RESEARCH.md §4.6. |
| `ferro-macros/tests/ui/action/*.rs` | trybuild UI tests | No existing trybuild tests in `ferro-macros/`. Create fresh. Pattern: `trybuild::TestCases::new().compile_fail("tests/ui/action/*.rs")`. Add `trybuild` to `ferro-macros/[dev-dependencies]`. |
| `docs/src/the-basics/action-handlers.md` | docs | No existing handler action docs. New page. Closest analog structurally: `docs/src/the-basics/controllers.md`. |

---

## Key Implementation Notes for Planner

1. **Parameter extraction refactor required:** `ParamKind`, `classify_param_type`, `generate_extraction`, `extract_param_name`, `is_primitive_type_name` are currently private in `ferro-macros/src/handler.rs`. They MUST be moved to `ferro-macros/src/utils.rs` as `pub(crate)` before `action.rs` can reuse them. This is a required prerequisite step in the plan.

2. **Module location decision:** RESEARCH.md recommends `framework/src/action/mod.rs` (top-level module in the `framework` crate). The task description says `framework/src/http/action.rs`. These are different. Top-level `action` module is cleaner (matches `http`, `validation`, `session` siblings). Planner resolves; the patterns above assume top-level `pub mod action` in `framework/src/lib.rs`.

3. **`From<sea_orm::DbErr> for ActionError`:** Needs feature-gate matching `framework/src/error.rs:454` pattern. Planner verifies the exact feature flag name (`"database"` or `"sea-orm"`).

4. **Macro re-export for `#[action]`:** Verified pattern at `framework/src/lib.rs:312` — `pub use ferro_macros::handler;`. New `pub use ferro_macros::action;` goes in the same block. Users write `use ferro::action;` in their controller files.

5. **`concat!(module_path!(), "::", stringify!(#fn_name))`** in generated code gives a `&'static str` handler name for tracing without runtime allocation. This is superior to `stringify!(#fn_name)` alone.

6. **Session context in tests:** `handle_action_result` calls `session_mut(...)`. In tests without a real request context, session writes return `None` (silently) — the query-string fallback still works. Tests can verify the 303 response and query-string params without needing a live session.

---

## Metadata

**Analog search scope:** `ferro-macros/src/`, `framework/src/`, `framework/tests/`
**Files scanned:** 11
**Pattern extraction date:** 2026-05-30
