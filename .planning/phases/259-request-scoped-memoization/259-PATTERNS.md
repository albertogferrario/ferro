# Phase 259: Request-scoped memoization — Pattern Map

**Mapped:** 2026-07-21
**Files analyzed:** 6 new/modified files
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `framework/src/memo/mod.rs` | utility (request-scoped store) | request-response | `framework/src/http/request_context.rs` (task-local, bare value) | exact |
| `framework/src/server.rs` (modify L279-281) | middleware / wiring | request-response | `framework/src/session/middleware.rs:192-197` (nested scope pattern) | exact |
| `framework/Cargo.toml` (modify) | config | — | `ferro-queue/Cargo.toml` (`futures = "0.3"`) and `ferro-ai/Cargo.toml` (`futures = { version = "0.3", default-features = false, features = ["std"] }`) | exact |
| `ferro-macros/src/memoize.rs` | utility (proc-macro) | transform | `ferro-macros/src/action.rs` (ItemFn parse + body rewrite + quote!) | exact |
| `ferro-macros/src/lib.rs` (modify) | config (proc-macro registration) | — | existing `pub fn action(...)` / `pub fn handler(...)` registration blocks | exact |
| `framework/src/lib.rs` (modify) | config (re-export) | — | `pub use theme::{current_theme, ...}` block (L141-145) | exact |

---

## Pattern Assignments

### `framework/src/memo/mod.rs` (utility, request-response)

**Analog:** `framework/src/http/request_context.rs` — simplest task-local (bare `String` value, no `RwLock<Option<...>>`). This is the direct model for `MEMO_STORE` because the store is always present once scoped (no optional initialization needed).

**Secondary analog for `try_with` + `None` degradation:** `framework/src/tenant/context.rs:32-37`

**Task-local declaration pattern** (`framework/src/http/request_context.rs:3-5`):
```rust
tokio::task_local! {
    pub(crate) static REQUEST_HOST: String;
}
```

For `MEMO_STORE`, use `Arc<MemoStore>` directly (same bare-value shape — no `RwLock<Option<...>>`):
```rust
tokio::task_local! {
    pub(crate) static MEMO_STORE: Arc<MemoStore>;
}
```

**`try_with` reader that degrades to `None` outside scope** (`framework/src/http/request_context.rs:11-13`):
```rust
pub fn request_host() -> Option<String> {
    REQUEST_HOST.try_with(|h| h.clone()).ok()
}
```

For `MEMO_STORE`:
```rust
pub fn current_memo_store() -> Option<Arc<MemoStore>> {
    MEMO_STORE.try_with(|s| s.clone()).ok()
}
```

**`with_*_scope` helper pattern** (`framework/src/tenant/context.rs:55-60`):
```rust
pub(crate) async fn with_tenant_scope<F, R>(ctx: Arc<RwLock<Option<TenantContext>>>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    TENANT_CONTEXT.scope(ctx, f).await
}
```

**Key difference from `TENANT_CONTEXT` / `CURRENT_THEME`:** Both of those use `Arc<RwLock<Option<T>>>` because the value starts as `None` and gets populated lazily. `MEMO_STORE` is created fresh at scope entry (always `Some`), so `Arc<MemoStore>` directly is the correct shape — matching `REQUEST_HOST: String` rather than `TENANT_CONTEXT`.

**`Any + TypeId` type-erased store precedent** (`framework/src/http/request.rs:7,88-103`):
```rust
use std::any::{Any, TypeId};
// ...
extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
// ...
pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
    self.extensions.insert(TypeId::of::<T>(), Box::new(value));
}
pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
    self.extensions
        .get(&TypeId::of::<T>())
        .and_then(|boxed| boxed.downcast_ref::<T>())
}
```

`MemoStore` uses the same `TypeId` for the callsite marker and `downcast_ref` for retrieval.

**Test pattern for task-local context** (`framework/src/tenant/context.rs:111-121`):
```rust
#[tokio::test]
async fn current_tenant_returns_some_within_scope() {
    let ctx = tenant_scope();
    {
        let mut guard = ctx.write().await;
        *guard = Some(make_tenant());
    }
    let result = with_tenant_scope(ctx, async { current_tenant() }).await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().slug, "acme");
}

#[test]
fn current_tenant_returns_none_outside_scope() {
    let result = current_tenant();
    assert!(result.is_none());
}
```

Mirror these for `out_of_scope_is_noop` (LIVE-01-E) and `current_memo_store_returns_some_within_scope` (LIVE-01-A).

**`futures` import required — but `futures-util` already present.** `framework/Cargo.toml:77` already has `futures-util = { version = "0.3", default-features = false, features = ["sink", "std"] }`. The full `futures` crate (which re-exports `futures-util` and adds `future::Shared`) is NOT yet present in `framework/Cargo.toml`. It must be added. The exact line to add (modeled on `ferro-ai/Cargo.toml:18`):
```toml
futures = { version = "0.3", default-features = false, features = ["std"] }
```

---

### `framework/src/server.rs` — modify lines 279-281 (middleware wiring)

**Analog:** `framework/src/session/middleware.rs:192-197` — nested scope pattern (outer `SESSION_ACCESSED.scope`, inner `SESSION_CONTEXT.scope`).

**Current code at server.rs:279-281** (verified):
```rust
let response = crate::http::request_context::REQUEST_HOST
    .scope(request_host, chain.execute(request, handler))
    .await;
```

**Nested scope pattern from session/middleware.rs:192-197** (verified):
```rust
let response = SESSION_ACCESSED
    .scope(
        accessed_flag.clone(),
        SESSION_CONTEXT.scope(ctx.clone(), async move { next(request).await }),
    )
    .await;
```

**After Phase 259, server.rs:279-281 becomes:**
```rust
let __memo_store = Arc::new(crate::memo::MemoStore::new());
let response = crate::http::request_context::REQUEST_HOST
    .scope(
        request_host,
        crate::memo::MEMO_STORE.scope(__memo_store, chain.execute(request, handler)),
    )
    .await;
```

**Critical: second `chain.execute` call at server.rs:311** (verified — the fallback handler path does NOT use `REQUEST_HOST.scope`). The research notes Assumption A6: the fallback path is intentionally un-scoped for `REQUEST_HOST`; apply the same exemption to `MEMO_STORE` unless explicitly decided otherwise at implementation time.

---

### `framework/Cargo.toml` — add `futures` dependency

**Current state (verified):** `futures-util` is present (line 77) but NOT `futures` itself.

**Model from `ferro-ai/Cargo.toml:18`:**
```toml
futures = { version = "0.3", default-features = false, features = ["std"] }
```

Add this line under `[dependencies]` in `framework/Cargo.toml`. The workspace Cargo.lock already has `futures 0.3.31` (pulled by `ferro-queue`), so no new network download is required.

`futures::future::Shared` and `futures::future::BoxFuture` and `FutureExt::shared()` all live in the `futures` crate, not in `futures-util`. The existing `futures-util` dep does not provide `Shared`.

---

### `ferro-macros/src/memoize.rs` (proc-macro, transform)

**Primary analog:** `ferro-macros/src/action.rs` — the closest existing macro that parses `ItemFn`, collects `FnArg::Typed` vs `FnArg::Receiver`, rewrites the body, and preserves the signature via `quote!`.

**ItemFn parsing and component extraction pattern** (`ferro-macros/src/action.rs:210-218`):
```rust
let input_fn = parse_macro_input!(input as ItemFn);

let fn_vis = &input_fn.vis;
let fn_name = &input_fn.sig.ident;
let fn_generics = &input_fn.sig.generics;
let fn_block = &input_fn.block;
let fn_attrs = &input_fn.attrs;

let params: Vec<_> = input_fn.sig.inputs.iter().collect();
```

**`FnArg::Receiver` vs `FnArg::Typed` split** (`ferro-macros/src/action.rs:224-244`):
```rust
for param in &params {
    match param {
        FnArg::Typed(pat_type) => {
            // collect value params for processing
        }
        FnArg::Receiver(_) => {
            return syn::Error::new_spanned(
                param,
                "#[action] does not support methods with `self` receiver",
            )
            .to_compile_error()
            .into();
        }
    }
}
```

For `#[memoize]`, the `Receiver` branch must NOT error — it must exclude `&self` from the key instead. Use:
```rust
let value_inputs: Vec<_> = input_fn.sig.inputs.iter()
    .filter(|a| !matches!(a, FnArg::Receiver(_)))
    .collect();
```

**Error return pattern** (used throughout `action.rs`):
```rust
return syn::Error::new_spanned(token, "message").to_compile_error().into();
```

Use this for: non-`async fn` input, non-`Pat::Ident` argument patterns (v17.0 limitation).

**`quote!` body rewrite pattern** (`ferro-macros/src/action.rs:256-278`):
```rust
let output = quote! {
    #(#fn_attrs)*
    #fn_vis async fn #fn_name #fn_generics(__ferro_req: #ferro::Request) -> #ferro::Response {
        let mut __ferro_req = __ferro_req;
        let __ferro_params = __ferro_req.params().clone();
        #(#extractions)*
        let __action_result: #ferro::ActionResult = async move { #fn_block }.await;
        #ferro::http::action::handle_action_result(...)
    }
};
output.into()
```

For `#[memoize]`, the generated function preserves the original signature (same vis, name, generics, inputs, output) and wraps the body:
```rust
let output = quote! {
    #(#fn_attrs)*
    #fn_vis async fn #fn_name #fn_generics(#(#all_inputs),*) #fn_output
    where
        /* Hash bound on each value param type */
        /* Clone + Send + Sync + 'static bound on return type */
    {
        struct #marker_name;
        let __key = #ferro::memo::MemoKey::new::<#marker_name, _>(&(#(#value_arg_names),*));
        if let Some(__store) = #ferro::memo::current_memo_store() {
            let __slot = __store.get_or_insert(__key, move || {
                ::std::boxed::Box::pin(async move {
                    let __result = { #fn_block };
                    ::std::sync::Arc::new(__result)
                        as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>
                })
            });
            let __arc = __slot.await;
            return __arc.downcast_ref::<#return_ty>()
                .expect("MemoStore type invariant")
                .clone();
        }
        { #fn_block }
    }
};
```

**Callsite uniqueness via `static AtomicUsize`** (pattern from research — not yet used in ferro macros, but standard proc-macro practice):
```rust
// ferro-macros/src/memoize.rs
use std::sync::atomic::{AtomicUsize, Ordering};
static MEMO_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn memoize_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let n = MEMO_COUNTER.fetch_add(1, Ordering::Relaxed);
    let marker_name = quote::format_ident!("__FerroMemoMarker{n}");
    // ...
}
```

**`async fn` guard** (pattern from research — action.rs does not check this; enforce it for `#[memoize]`):
```rust
if input_fn.sig.asyncness.is_none() {
    return syn::Error::new_spanned(
        &input_fn.sig,
        "#[memoize] can only be applied to `async fn`",
    )
    .to_compile_error()
    .into();
}
```

**Imports block for `memoize.rs`** (modeled on `action.rs:50-55`):
```rust
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType};
```

---

### `ferro-macros/src/lib.rs` — add `#[memoize]` registration

**Analog:** every existing attribute macro registration in `lib.rs`. The exact registration for `#[action]` (lines 266-269) is the template:
```rust
#[proc_macro_attribute]
pub fn action(attr: TokenStream, input: TokenStream) -> TokenStream {
    action::action_impl(attr, input)
}
```

For `#[memoize]`:
```rust
mod memoize;

/// Mark an `async fn` or `async` impl method for request-scoped memoization.
///
/// The function body runs at most once per `(callsite, arguments)` per request.
/// Concurrent callers for the same key within one request coalesce onto a
/// single shared computation. Outside a request context the body runs normally
/// with no caching (graceful no-op).
///
/// All value arguments must implement `Hash`. The return type must implement
/// `Clone + Send + Sync + 'static`. Applied to impl methods, `&self` is
/// excluded from the key.
///
/// # Example
///
/// ```rust,ignore
/// #[memoize]
/// pub async fn load_product(id: u32) -> Vec<String> { /* db query */ }
/// ```
#[proc_macro_attribute]
pub fn memoize(attr: TokenStream, input: TokenStream) -> TokenStream {
    memoize::memoize_impl(attr, input)
}
```

Add `mod memoize;` alongside the existing `mod action;` declaration (line 13).

---

### `framework/src/lib.rs` — re-export memo API

**Analog:** `framework/src/lib.rs:141-145` (theme re-export block):
```rust
#[cfg(feature = "theme")]
pub use theme::{
    current_theme, DefaultResolver, HeaderThemeResolver, TenantThemeResolver, ThemeMiddleware,
    ThemeResolver,
};
```

For memo (unconditional — not feature-gated, memo is always available):
```rust
pub mod memo;
pub use memo::{current_memo_store, MemoKey, MemoStore};
```

Place alongside the `pub mod session;` and `pub mod tenant;` declarations in the module section (lines 36-38 area), then add the `pub use` in the re-export section below. The `MEMO_STORE` task-local itself stays `pub(crate)` — only the public-facing API (`current_memo_store`, `MemoKey`, `MemoStore`) is re-exported.

---

## Shared Patterns

### Task-local ambient context (applies to `memo/mod.rs` and `server.rs`)

**Source:** `framework/src/http/request_context.rs` (bare-value shape) and `framework/src/tenant/context.rs` (with `try_with` degradation pattern)

The invariant across all four existing task-locals (`REQUEST_HOST`, `TENANT_CONTEXT`, `CURRENT_THEME`, `SESSION_CONTEXT`):
1. Declare with `tokio::task_local! { ... }`.
2. Expose a public `current_*()` function that uses `.try_with(...).ok()` — never `.with(...)` which panics outside scope.
3. Expose a `pub(crate) *_scope()` factory and `pub(crate) async fn with_*_scope(...)` helper.
4. Enter scope in `server.rs` (or middleware) by nesting `.scope(value, future).await`.

### `quote!` body rewrite for proc-macros (applies to `ferro-macros/src/memoize.rs`)

**Source:** `ferro-macros/src/action.rs:256-278`

Key conventions:
- Preserve `#(#fn_attrs)*` at top of generated fn.
- Use `__ferro_`-prefixed internal bindings to avoid name collisions with user code.
- Return `output.into()` at the end.
- Return `syn::Error::new_spanned(...).to_compile_error().into()` for macro errors (never panic).

### `Arc` + `TypeId` type erasure (applies to `MemoStore` internals)

**Source:** `framework/src/http/request.rs:7,88-103`

The `downcast_ref::<T>()` pattern is established in `Request::get`. The same pattern applies to `MemoStore` retrieval. Use `.expect("MemoStore type invariant")` with a descriptive message, not `.unwrap()`.

### `#[tokio::test]` async test with task-local scope (applies to `memo/mod.rs` tests)

**Source:** `framework/src/tenant/context.rs:111-121` and `framework/src/theme/context.rs:77-87`

Both use `with_*_scope(scope, async { ... }).await` as the test harness. Mirror this for `MEMO_STORE.scope(Arc::new(MemoStore::new()), async { ... }).await`.

---

## Key Observation: `futures` vs `futures-util`

`framework/Cargo.toml` already has `futures-util` (line 77) but `future::Shared` lives in the `futures` crate (which re-exports `futures-util`). The two are NOT interchangeable for this purpose. `futures = { version = "0.3", default-features = false, features = ["std"] }` must be added — this is the only new dependency required for the entire phase.

---

## No Analog Found

All files have close analogs. No files require falling back to RESEARCH.md patterns alone.

---

## Metadata

**Analog search scope:** `framework/src/`, `ferro-macros/src/`, `ferro-json-ui/src/projection/`, `ferro-projections/src/`
**Files read for verification:** `framework/src/http/request_context.rs`, `framework/src/tenant/context.rs`, `framework/src/theme/context.rs`, `framework/src/session/middleware.rs:185-200`, `framework/src/server.rs:270-320`, `framework/src/http/request.rs:1-110`, `ferro-macros/src/action.rs`, `ferro-macros/src/utils.rs:80-122`, `ferro-macros/src/lib.rs`, `ferro-macros/Cargo.toml`, `framework/Cargo.toml`, `framework/src/lib.rs`, `ferro-json-ui/src/projection/builder.rs:50-125`
**Pattern extraction date:** 2026-07-21
