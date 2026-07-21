# Phase 259: Request-scoped memoization — Research

**Researched:** 2026-07-21
**Domain:** Rust proc-macro authoring, tokio task-local storage, `futures::future::Shared` coalescing, ferro render-pass wiring
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 — Store access mechanism:** `MemoStore` held in a `tokio::task_local!` (`MEMO_STORE`), scoped per-request in the server middleware chain, mirroring `TENANT_CONTEXT` (`framework/src/tenant/context.rs`, scope entered at `framework/src/server.rs:280`). `#[memoize]` reads the ambient store; does NOT require a `&Request`/`&Cx` parameter. The `Request` extensions type-map is NOT the chosen surface.

**D-02 — Out-of-scope behavior:** Graceful no-op. `MEMO_STORE.try_with(...)` returns `Err` → body runs un-memoized, no panic. Mirrors `current_tenant()` (context.rs:32–37).

**D-03 — Argument keying:** `MemoKey = (TypeId of a per-callsite zero-sized marker, u64 hash of the non-receiver arguments)`. All value args must `impl Hash`; macro emits the bound (compile-time error for non-hashable args). `&self` receiver excluded from the key for `#[service]` methods.

**D-04 — Coalescing + error semantics:** Concurrent callers coalesce via `futures::future::Shared` whose output is `Arc<dyn Any + Send + Sync>`, downcast to concrete type. Full return value cached for the request including `Err` for `Result`-returning fns.

**D-05 — Render-path wiring + proof:** Phase 259 wires the memo store into the `ServiceDef → IntentGraph → JsonUiRenderer` fetch path and ships a render-path integration test proving N intents over one key issue exactly one underlying fetch.

### Claude's Discretion

- Internal `MemoStore` types: `Mutex<HashMap<…>>` vs `RwLock`, initial capacity.
- Per-callsite zero-sized marker generation strategy inside the macro.
- Whether to emit a debug-mode warning when `#[memoize]` runs outside scope.
- Whether to expose a manual `MemoStore` API for non-macro use, or keep it crate-internal.
- Exact scope-entry point in the middleware chain (alongside vs nested within the tenant scope at server.rs:280).

### Deferred Ideas (OUT OF SCOPE)

- Cross-request / general-purpose caching (stays `ferro-cache`).
- Memoization in background / queue-worker contexts (D-02 keeps those working un-memoized).
- `LiveFragment` element + client runtime — Phase 260.
- `asset!()` macro + Iconify/Fontsource fetch — Phase 261.
- MCP catalog / `generation_context` / docs / publish — Phase 262.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LIVE-01 | Request-scoped `#[memoize]` attribute for async fn and `#[service]` methods, plus render-path fetch dedup proving N intents over one key issue one fetch | §Task-local wiring, §Macro authoring, §Render-path wiring, §MemoStore internals |
</phase_requirements>

---

## Summary

Phase 259 adds three coupled deliverables: (1) `MemoStore` in `framework`, (2) the `#[memoize]` attribute macro in `ferro-macros`, and (3) wiring into the `ServiceDef → IntentGraph → JsonUiRenderer` render pass with a render-path integration test.

The task-local pattern (D-01) is already used in four places in `framework`: `TENANT_CONTEXT` (tenant/context.rs), `CURRENT_THEME` (theme/context.rs), `REQUEST_HOST` (http/request_context.rs), and `SESSION_CONTEXT` + `SESSION_ACCESSED` (session/middleware.rs). `MEMO_STORE` follows the exact same shape. The macro is modeled after `#[action]` in `ferro-macros/src/action.rs`, which already demonstrates `syn` `ItemFn` parsing, parameter rewriting, and `quote!` code generation. The `futures::future::Shared` primitive exists in `futures` 0.3.31 (in the workspace Cargo.lock) and is available in `ferro-queue` and `ferro-ai`; it must be added to `framework/Cargo.toml`.

**Critical render-path finding (D-05):** The current `ServiceDef → JsonUiRenderer` render pass (`ferro-json-ui/src/projection/builder.rs`, `mod.rs`) is schema-only — it does NOT perform any I/O or data fetch. `Spec::from_service_def` takes a `&ServiceDef` (static schema definition) and `&[IntentScore]` (derived from schema signals, no DB) and returns a `Spec` built from component vocabulary. There is no load call in this path. The "render-path integration test" in Success Criterion #3 must therefore use a **representative memoized loader/service method** that the render pass would call in a real application, asserted via a call counter, rather than a structural fetch inside the renderer itself. The test harness must create this pattern, not discover it pre-existing.

**Primary recommendation:** Implement `MemoStore` in `framework/src/memo/` (new sub-module), `#[memoize]` in `ferro-macros/src/memoize.rs`, wire the scope in `framework/src/server.rs` alongside `REQUEST_HOST`, and create a render-path integration test that places a memoized loader function in the test and calls `derive_intents` + `JsonUiRenderer::render` twice over the same key, asserting the loader ran once.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `MemoStore` (store type + `MemoKey` + coalescing logic) | Framework crate (`framework/src/memo/`) | — | Runtime store, not schema; belongs with other request-scoped stores (session, tenant context) |
| `#[memoize]` attribute macro | `ferro-macros` crate | — | All ferro attribute macros live here; no new crate allowed |
| Task-local wiring (`MEMO_STORE` + scope entry) | Framework crate (`framework/src/memo/` or `framework/src/server.rs`) | — | Follows `REQUEST_HOST` scope pattern in server.rs |
| Public re-export (`ferro::memoize` + `ferro::MemoStore`) | `framework/src/lib.rs` + `ferro-macros` re-export | — | Consistent with all other macros (`ferro::handler`, `ferro::service`, etc.) |
| Render-path integration test | `framework/src/memo/` test module or `ferro-json-ui/tests/` | — | Must exercise the `derive_intents` + `render` path with a memoized loader |

---

## Standard Stack

### Core (verified in workspace)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tokio` | 1.48.0 [VERIFIED: Cargo.lock] | `task_local!`, `#[tokio::test]` | Already a workspace dependency in `framework/Cargo.toml` |
| `futures` | 0.3.31 [VERIFIED: Cargo.lock] | `future::Shared`, `future::BoxFuture`, `FutureExt` | In `ferro-queue` and `ferro-ai`; must be added to `framework/Cargo.toml` |
| `syn` | 2.x [VERIFIED: `ferro-macros/Cargo.toml`] | `ItemFn`, `ItemTrait`, `FnArg::Receiver` parsing | Already the macro crate's parser |
| `quote` | 1.x [VERIFIED: `ferro-macros/Cargo.toml`] | `quote!` token generation | Already in `ferro-macros` |
| `proc-macro2` | 1.x [VERIFIED: `ferro-macros/Cargo.toml`] | `TokenStream2`, `Span` | Already in `ferro-macros` |
| `std::sync::Mutex` + `std::collections::HashMap` | std | `MemoStore` internal map | Standard library, no new dep |
| `std::any::{Any, TypeId}` | std | Type-erased store values + callsite marker | Standard library; already used in `framework/src/http/request.rs:7` |
| `std::hash::{Hash, Hasher}` + `std::collections::hash_map::DefaultHasher` | std | Argument hashing for `MemoKey` | Standard library |

### Version Verification

```bash
# futures not yet in framework/Cargo.toml — add it:
# futures = { version = "0.3", default-features = false, features = ["std"] }
# (lock already has 0.3.31 from ferro-queue)
```

**Note:** `futures` must be added to `framework/Cargo.toml`. The workspace Cargo.lock already contains 0.3.31 (pulled by `ferro-queue`), so no new download is needed. Specify with `default-features = false, features = ["std"]` to avoid pulling in executors and I/O (consistent with how `ferro-ai/Cargo.toml` adds it). [VERIFIED: Cargo.toml survey]

---

## Architecture Patterns

### System Architecture Diagram

```
Request arrives at framework server.rs
          │
          ▼
  REQUEST_HOST.scope(host,              ← server.rs:279-281
    MEMO_STORE.scope(                   ← NEW: alongside REQUEST_HOST
      Arc::new(MemoStore::new()),
      chain.execute(request, handler)
    )
  ).await
          │
          ▼
  Handler (or middleware) calls a #[memoize] fn
          │
          ▼
  #[memoize] wrapper:
    MEMO_STORE.try_with(|store| {        ← ambient lookup
      let key = MemoKey::new::<__Marker>(hash_args)
      store.get_or_insert(key, || future)
    })
    .ok()
    .unwrap_or_else(|| run body directly)  ← D-02 graceful no-op
          │
  ┌───────┴────────┐
  │ cache MISS     │ cache HIT (or concurrent caller)
  ▼                ▼
run body       return Arc clone
insert Shared  downcast to T
future in map
poll to completion
          │
          ▼
   Arc<dyn Any + Send + Sync>
   downcast to T (or Result<T,E>)
   return to caller
          │
          ▼
  Request finishes → MEMO_STORE scope dropped → MemoStore dropped
```

### Recommended Project Structure

```
framework/src/
├── memo/
│   ├── mod.rs          # MemoStore, MemoKey, MEMO_STORE task-local, current_memo_store()
│   └── (no separate files needed; store is self-contained)
└── server.rs           # L279: add MEMO_STORE.scope() around the chain.execute call

ferro-macros/src/
├── memoize.rs          # #[memoize] implementation
└── lib.rs              # register #[proc_macro_attribute] pub fn memoize(...)
```

### Pattern 1: `tokio::task_local!` for request-scoped ambient state

Every ambient context in ferro follows this exact pattern. `MEMO_STORE` mirrors `CURRENT_THEME` (theme/context.rs) which is the simplest variant (no `RwLock` needed — see Pattern 2):

```rust
// framework/src/memo/mod.rs
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

tokio::task_local! {
    pub(crate) static MEMO_STORE: Arc<MemoStore>;
}

/// Return the current request's memo store, if inside a request context.
///
/// Returns `None` outside a request scope (background jobs, tests without scope).
pub fn current_memo_store() -> Option<Arc<MemoStore>> {
    MEMO_STORE.try_with(|s| s.clone()).ok()
}

pub(crate) fn memo_scope() -> Arc<MemoStore> {
    Arc::new(MemoStore::new())
}

pub(crate) async fn with_memo_scope<F, R>(store: Arc<MemoStore>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    MEMO_STORE.scope(store, f).await
}
```

[VERIFIED: pattern from `framework/src/theme/context.rs`]

**Key difference from `TENANT_CONTEXT`:** `MEMO_STORE` holds `Arc<MemoStore>` directly, not `Arc<RwLock<Option<...>>>`, because the store is created fresh per request (no optional initialization needed) and internal mutation goes through `MemoStore`'s own `Mutex`. This is simpler and matches the `REQUEST_HOST: String` shape.

### Pattern 2: `MemoStore` internals — Shared future coalescing

```rust
// framework/src/memo/mod.rs
use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use futures::future::{BoxFuture, Shared, FutureExt};

/// Type-erased slot in the memo map.
pub type MemoSlot = Shared<BoxFuture<'static, Arc<dyn Any + Send + Sync>>>;

/// Per-request memoization store.
///
/// Holds a map from `(callsite TypeId, argument hash)` to a shared awaitable slot.
/// The first caller inserts a pending future; concurrent callers await the same slot.
pub struct MemoStore {
    entries: Mutex<HashMap<MemoKey, MemoSlot>>,
}

/// Key for a memoized call: callsite identity + argument hash.
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct MemoKey {
    callsite: std::any::TypeId,
    args_hash: u64,
}

impl MemoKey {
    pub fn new<Marker: 'static, A: Hash>(args: &A) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut h = DefaultHasher::new();
        args.hash(&mut h);
        Self {
            callsite: std::any::TypeId::of::<Marker>(),
            args_hash: h.finish(),
        }
    }
}

impl MemoStore {
    pub fn new() -> Self {
        Self { entries: Mutex::new(HashMap::new()) }
    }

    /// Get or insert a shared future for `key`.
    ///
    /// The `make_fut` closure is called at most once per key; concurrent callers
    /// await the same `Shared` handle.
    pub fn get_or_insert(
        &self,
        key: MemoKey,
        make_fut: impl FnOnce() -> BoxFuture<'static, Arc<dyn Any + Send + Sync>>,
    ) -> MemoSlot {
        let mut map = self.entries.lock().unwrap();
        map.entry(key)
            .or_insert_with(|| make_fut().shared())
            .clone()  // Shared is Clone
    }
}
```

[ASSUMED: exact type signatures pending compiler verification; the pattern is verified against `futures` 0.3.x docs]

**`Shared` output must be `Clone`:** `futures::future::Shared` requires the wrapped future's output to be `Clone`. `Arc<dyn Any + Send + Sync>` is `Clone` (cloning the `Arc`). This is the reason `Arc<...>` wraps the concrete value rather than storing the value directly. [VERIFIED: Context7 futures 0.3.32 docs — `fn shared(self) -> Shared<Self>` requires `Self::Output: Clone`]

**Panic behavior of `Shared`:** When a `Shared` future panics, the panic propagates to the first poller; subsequent pollers on the same `Shared` will get a "inner future panicked" panic. This is acceptable for v17.0 because memoized functions that panic are a caller error (document this). If defensive handling is needed, wrap the future body in `std::panic::catch_unwind` before boxing, but this adds complexity and is Claude's discretion.

### Pattern 3: `#[memoize]` macro authoring — free `async fn`

The closest analog is `#[action]` (ferro-macros/src/action.rs), which parses `ItemFn`, rewrites the body, and preserves the public signature. For `#[memoize]` on a free `async fn`:

```rust
// ferro-macros/src/memoize.rs
// Source: modeled on action.rs and handler.rs

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn};

pub fn memoize_free_fn_impl(input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // 1. Extract components
    let fn_vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_output = &input_fn.sig.output;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;

    // 2. Collect value (non-self) parameters for hashing
    let value_params: Vec<_> = input_fn.sig.inputs.iter()
        .filter(|arg| matches!(arg, FnArg::Typed(_)))
        .collect();

    // 3. Generate a per-callsite zero-sized marker type
    // The macro uses a module-scoped static to generate unique names per expansion
    // (See Pattern 4 for the exact uniqueness strategy)
    let marker_name = quote::format_ident!("__MemoMarker_{}", fn_name);

    // 4. Emit the Hash bound on each value parameter type
    // (the bound is added to the where clause in the generated fn)

    // 5. Rewrite the body: check memo store, build key, get_or_insert, downcast
    let output = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name #fn_generics(#(#value_params),*) #fn_output {
            struct #marker_name;  // zero-sized, unique per macro site

            let __memo_key = ::ferro::memo::MemoKey::new::<#marker_name, _>(
                &(/* tuple of value args */),
            );

            if let Some(__store) = ::ferro::memo::current_memo_store() {
                let __slot = __store.get_or_insert(__memo_key, || {
                    Box::pin(async move {
                        let __result: /* return type */ = { #fn_block };
                        ::std::sync::Arc::new(__result) as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>
                    })
                });
                let __arc = __slot.await;
                return *__arc.downcast::</* return type */>().expect("MemoStore type mismatch");
            }
            // D-02: no scope → run body directly
            { #fn_block }
        }
    };
    output.into()
}
```

[ASSUMED: The exact tuple construction for the args hash and the return type extraction from `fn_output` require careful syn manipulation — see Pattern 5 for the return-type downcast challenge.]

### Pattern 4: Per-callsite uniqueness of the zero-sized marker

The `struct __MemoMarker_<fn_name>;` approach names the marker after the function, which collides if two functions have the same name in different modules. The robust approach is a counter-based unique ID using a `static AtomicUsize` in the macro crate:

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

The `struct __FerroMemoMarker17;` is emitted at the call site and is private to the generated code, so it does not pollute the user's namespace. [ASSUMED: this strategy is standard in Rust proc-macro practice; not explicitly documented in ferro existing macros but consistent with how `#[ferro_test]` generates test functions]

### Pattern 5: Return-type downcast challenge for `Result`-returning fns

D-04 says the full return value including `Err` is cached. The macro must handle two cases:

**Case A: `T` (non-Result)**
```rust
// box the return value as Arc<dyn Any>
let __arc: Arc<dyn Any + Send + Sync> = Arc::new(result);
// on retrieval:
*__arc.downcast::<T>().expect("MemoStore type invariant")
```

**Case B: `Result<T, E>` (or any return type)**
```rust
// The macro cannot distinguish Result from non-Result purely from the type token.
// Approach: always box `T` where `T` is the full return type (including Result).
// The Arc<dyn Any> holds a `Result<V, E>` when the fn returns Result<V, E>.
// Downcast: __arc.downcast::<Result<V, E>>()
```

The macro should always box the **full return type** as `T`, whether it is `Result<V,E>` or `V`. This is simpler than inspecting the return type and avoids special-casing. The constraint is that `T: Any + Send + Sync`, which means the error type in `Result<V, E>` must also be `Any + Send + Sync` (i.e., `'static`). For `#[service]` methods returning `Result<_, E>` where `E: std::error::Error + Send + Sync + 'static`, this is satisfied. [ASSUMED: the exact macro code for downcast must be verified at implementation time]

**The clone-on-downcast problem:** `Arc::downcast::<T>` consumes the `Arc` if T is Sized. Since `Shared` holds the same `Arc<dyn Any>` for all concurrent callers, and `downcast` requires ownership, the macro wrapper must:

```rust
// Correct pattern:
let __arc: Arc<dyn Any + Send + Sync> = __slot.await;
// Arc<dyn Any + Send + Sync>::downcast consumes the Arc
// To call from multiple places, clone the Arc first:
let __typed: Arc<T> = Arc::downcast::<T>(__arc.clone()).expect("type invariant");
// Then: (*__typed).clone() — requires T: Clone
```

OR, the store can hold `Arc<T>` inside the `Any` box so it can be cloned cheaply. The practical solution: the macro generates code that extracts a `&T` via `downcast_ref` (not consuming the `Arc`):

```rust
let __arc: Arc<dyn Any + Send + Sync> = __slot.await;
let __ref: &T = __arc.downcast_ref::<T>().expect("type invariant");
// If T: Clone, clone it out:
__ref.clone()
```

This requires `T: Clone`. For `Result<V, E>` this is `V: Clone + E: Clone`. For service methods returning `Result<Vec<Model>, DbErr>`, both are `Clone`. Document that `#[memoize]` requires `T: Clone` on the return type. [ASSUMED: needs compiler verification]

### Pattern 6: `#[memoize]` on `#[service]` trait methods

Applying `#[memoize]` to a method inside a `#[service]` trait is different from a free `async fn`:

- The input to the macro is an `ItemFn` within an `ItemTrait` body, **not** a standalone `ItemFn`.
- `FnArg::Receiver` (`&self`) must be detected and excluded from the key.
- The method body is still an `async` block; the rewrite is the same wrapper pattern.

**Implementation approach:** Detect if the first argument is `FnArg::Receiver` using `syn`:
```rust
let is_method = input_fn.sig.inputs.first()
    .map(|a| matches!(a, FnArg::Receiver(_)))
    .unwrap_or(false);
let value_inputs: Vec<_> = input_fn.sig.inputs.iter()
    .filter(|a| !matches!(a, FnArg::Receiver(_)))
    .collect();
```

Then use `value_inputs` for the args hash and retain `self` in the final function signature unchanged.

**Trait method application:** `#[memoize]` on a trait method rewrites the method in the trait definition itself. The trait retains the same signature. The concrete impl will have the rewritten body. Alternatively, `#[memoize]` can be applied to the implementation method (not the trait declaration) — this is simpler because it avoids rewriting trait declarations and applies cleanly to `async fn` in `impl Trait`. Document that `#[memoize]` targets implementation methods, not trait declarations (consistent with `#[handler]` which cannot be applied to trait methods). [ASSUMED: the exact application site policy needs user confirmation at plan time]

### Pattern 7: Wiring the scope in `server.rs`

Current `server.rs:278–281` [VERIFIED]:
```rust
let response = crate::http::request_context::REQUEST_HOST
    .scope(request_host, chain.execute(request, handler))
    .await;
```

After Phase 259, it becomes:
```rust
let __memo = crate::memo::memo_scope();  // Arc::new(MemoStore::new())
let response = crate::http::request_context::REQUEST_HOST
    .scope(request_host,
        crate::memo::MEMO_STORE.scope(__memo, chain.execute(request, handler))
    )
    .await;
```

The memo store is nested inside `REQUEST_HOST.scope` (outer) but this is equivalent to alongside. The session middleware (`session/middleware.rs:192-197`) already shows the nested pattern: `SESSION_ACCESSED.scope(..., SESSION_CONTEXT.scope(..., body))`. Either nesting order works; nest inside `REQUEST_HOST` to keep server.rs changes minimal. [VERIFIED: server.rs:279-281, session/middleware.rs:192-197]

**IMPORTANT:** The fallback handler path in `server.rs` (around line 308-315) also calls `chain.execute` and must also be wrapped. Confirm at implementation: there are TWO `chain.execute` call sites in `server.rs`. [VERIFIED: server.rs grep shows only one `REQUEST_HOST.scope` at line 279; the fallback path at line 311 calls `chain.execute` WITHOUT the REQUEST_HOST scope — check if that is intentional before extending it]

### Pattern 8: Render-path integration test (D-05 — the key finding)

**Finding:** The current `ServiceDef → JsonUiRenderer` render pass is SCHEMA-ONLY. [VERIFIED: `ferro-json-ui/src/projection/builder.rs` — `from_service_def` takes `&ServiceDef`, `&[IntentScore]`, `&VisualContext`; calls `global_catalog()` for validation; does NOT call any async loader or I/O function.]

`derive_intents()` (ferro-projections/src/derive.rs) also works purely on `&ServiceDef` field signals — no DB, no async, no I/O. [VERIFIED: derive.rs:75–80]

The render-path integration test for Success Criterion #3 must therefore be constructed, not discovered. The approach:

1. Create a `#[memoize]` async fn `load_product_data(id: u32) -> Vec<String>` with a call counter (`Arc<AtomicUsize>`).
2. In the test, enter a `MEMO_STORE.scope(...)`.
3. Call this loader twice with the same `id` (simulating two intents fetching the same underlying data), assert `counter.load() == 1`.
4. Additionally, call `JsonUiRenderer.render(...)` twice for the same `ServiceDef` with different `intent_index` values, with the loader called inside each render path (via a test helper that wraps the render pass), and assert counter == 1.

The "render-path wiring" is therefore the **addition of a memoized data-loader layer** that a real multi-intent render would use, wired so the projection builder can call it. Since `Spec::from_service_def` is synchronous and pure, the loader is invoked by the caller that drives the render (the handler, or a test that simulates it), not by the renderer internally. The test proves that when a caller uses `#[memoize]` to fetch data before passing it to the renderer, the memo store deduplicates the fetch.

This is an honest and verifiable proof of Success Criterion #3. [VERIFIED: analysis of ferro-json-ui/src/projection/builder.rs:56-124 and ferro-projections/src/derive.rs:75-80]

### Anti-Patterns to Avoid

- **Storing `Box<dyn Future>` directly in the map (not `Shared`):** Without `Shared`, concurrent callers would race to poll the same future, causing panics or double-execution. Always store `Shared<BoxFuture<...>>`. [VERIFIED: futures 0.3 docs]
- **Using `RwLock` for `MemoStore.entries`:** Write-on-miss only; `Mutex` suffices and avoids the async RwLock overhead. `std::sync::Mutex` (sync, not `tokio::sync::Mutex`) is correct because the critical section is non-async (map lookup + insert, then drop the lock before `.await`). [ASSUMED: standard practice]
- **Blocking inside `Mutex` guard while awaiting the future:** Acquire lock → clone the `Shared` handle → release lock → then `.await` the clone. Never hold the mutex guard across an `.await`. [ASSUMED: standard async Rust rule]
- **Applying `#[memoize]` to the trait declaration:** Apply it to implementation methods to avoid rewriting trait interfaces. Document clearly.
- **Using `unwrap()` on downcast:** Use `.expect("MemoStore type invariant: {type_name}")` with the type name for easier debugging.
- **Forgetting the fallback handler path in server.rs:** The fallback handler also calls `chain.execute` and needs the MEMO_STORE scope if memoized fns can be called from fallback handlers.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Concurrent-callers coalescing | Custom "already running" flag + channels | `futures::future::Shared` | `Shared` handles the exact case: first caller drives the future, all clones await the same result; works with tokio's executor |
| Type-erased key-value store | Custom enum dispatch | `std::any::Any + TypeId` | Already used in `Request::extensions` (request.rs:7,88-103); same pattern throughout the codebase |
| Argument hashing | Custom hash function | `std::hash::DefaultHasher` via `Hash` derive bound | Sufficient for within-request dedup; no need for cryptographic or cross-request-stable hashing |
| Unique callsite IDs | File+line macros, string interning | `TypeId` of a per-macro-expansion zero-sized struct | Zero runtime cost, guaranteed unique per macro expansion, no string allocation |

**Key insight:** `futures::future::Shared` is the only correct primitive for the coalescing requirement. Anything hand-rolled would re-implement either `Shared` (hard) or a channel-based rendezvous (complex, heavier). The `std::any` module handles type erasure with zero unsafe code.

---

## Common Pitfalls

### Pitfall 1: `Mutex` guard held across `.await`

**What goes wrong:** `std::sync::Mutex::lock()` returns a guard. If the guard is held while the code calls `.await`, tokio may move the task to a different thread, which panics because `MutexGuard` is `!Send`.

**Why it happens:** Writing `let __slot = map.lock().unwrap().entry(key).or_insert_with(|| fut.shared()).clone(); __slot.await` looks fine but the compiler allows the guard to live to the end of the statement (not to `await`). If restructured naively, the guard is held at the `.await`.

**How to avoid:** In `get_or_insert`, explicitly drop the guard before returning:
```rust
let slot = {
    let mut map = self.entries.lock().unwrap();
    map.entry(key).or_insert_with(|| make_fut().shared()).clone()
};  // guard dropped here
slot.await  // called outside the lock
```
[VERIFIED: this is the correct pattern; the borrow checker enforces it if the guard is in a separate `let` block]

**Warning signs:** `tokio` runtime panics with "MutexGuard is not Send" or "cannot hold mutex guard across await".

### Pitfall 2: `Shared` future panic propagation

**What goes wrong:** If the future inside a `Shared` panics, all waiting callers receive the panic. `Shared` wraps the output, not the panicking behavior. Future callers (after the panic) will also panic when they try to poll the `Shared` because the inner future's slot is poisoned.

**Why it happens:** `futures::future::Shared` stores the output as an `Option`; if the future panics, the panic propagates through `SharedFuture::poll` to all current and future pollers.

**How to avoid:** Document that `#[memoize]` applied to functions that can panic in their body is caller error. For defensive V1, add to RESEARCH and docs; do not add `catch_unwind` (overengineered for v17.0). Future direction: wrap body in `catch_unwind` and store `Result<Arc<T>, PanicInfo>`.

### Pitfall 3: `--all-features` clippy vs local clippy

**What goes wrong:** `cargo clippy --all-targets -- -D warnings` (without `--all-features`) passes locally, but CI runs `--all-features` which enables `projections` and compiles paths that only exist under that feature.

**How to avoid:** Run `cargo clippy --all --all-targets --all-features -- -D warnings` exactly as CI does before committing. The `projections` feature in `framework/Cargo.toml` enables `ferro-projections` and `ferro-json-ui` paths; the render-path integration test will only compile under `--all-features`. [VERIFIED: `framework/Cargo.toml` line 18, `ferro-json-ui/projections` feature]

### Pitfall 4: `#[memoize]` on non-`async fn`

**What goes wrong:** The macro only makes sense for `async fn` — a synchronous function cannot have concurrent callers that would benefit from coalescing.

**How to avoid:** Check `input_fn.sig.asyncness.is_some()` at the top of `memoize_impl` and emit `compile_error!("...#[memoize] can only be applied to async fn...")` for sync fns. [VERIFIED: `#[action]` (action.rs) does not check this, but `#[handler]` correctly handles the sync case; `#[memoize]` should enforce async-only]

### Pitfall 5: Return type must implement `Clone` for downcast

**What goes wrong:** The macro clones the inner value from the `Arc` via `downcast_ref::<T>().clone()`. If `T` is not `Clone`, the generated code does not compile.

**How to avoid:** Emit `T: Clone` as a where-clause bound in the generated function signature. The compile error ("the trait `Clone` is not implemented for `T`") is surfaced at the caller's macro site, which is acceptable.

### Pitfall 6: `ferro-macros` tokio dev-dependency for `#[tokio::test]`

**What goes wrong:** `ferro-macros/Cargo.toml` already has `tokio` as a dev-dependency. The memoize macro tests that use `#[tokio::test]` with `MEMO_STORE.scope(...)` need `tokio` in scope.

**How to avoid:** Tests for the MACRO's output are typically done in an integration test crate or in the consuming `framework` crate, not in `ferro-macros` directly (proc-macro crates cannot call their own macros in unit tests). The framework tests use the macro as a consumer. [VERIFIED: `ferro-macros/Cargo.toml` line 26 has `tokio` dev dep with `macros, rt` features]

---

## Code Examples

### Example 1: Full task-local context pattern (verified)

```rust
// Source: framework/src/theme/context.rs (exact pattern to mirror)
tokio::task_local! {
    static CURRENT_THEME: Arc<RwLock<Option<Arc<Theme>>>>;
}

pub fn current_theme() -> Option<Arc<Theme>> {
    CURRENT_THEME
        .try_with(|ctx| ctx.try_read().ok().and_then(|guard| guard.clone()))
        .ok()
        .flatten()
}
```

For `MEMO_STORE` the simpler shape applies (no `RwLock<Option>` needed; the store is always `Some`):

```rust
tokio::task_local! {
    pub(crate) static MEMO_STORE: Arc<MemoStore>;
}

pub fn current_memo_store() -> Option<Arc<MemoStore>> {
    MEMO_STORE.try_with(|s| s.clone()).ok()
}
```

### Example 2: Scope entry in server.rs (verified location)

```rust
// Source: framework/src/server.rs:278-281 (current)
let response = crate::http::request_context::REQUEST_HOST
    .scope(request_host, chain.execute(request, handler))
    .await;

// After Phase 259 (new):
let __memo_store = Arc::new(crate::memo::MemoStore::new());
let response = crate::http::request_context::REQUEST_HOST
    .scope(
        request_host,
        crate::memo::MEMO_STORE.scope(__memo_store, chain.execute(request, handler)),
    )
    .await;
```

### Example 3: `get_or_insert` without holding Mutex across `.await`

```rust
// Correct: lock scope ends before .await
pub fn get_or_insert(&self, key: MemoKey, make_fut: impl FnOnce() -> BoxFuture<'static, Arc<dyn Any + Send + Sync>>) -> MemoSlot {
    let slot = {
        let mut map = self.entries.lock().unwrap();
        map.entry(key).or_insert_with(|| make_fut().shared()).clone()
    }; // MutexGuard dropped here
    slot  // returned; caller awaits outside
}
```

### Example 4: `#[memoize]` on free async fn — generated output shape

```rust
// User writes:
#[memoize]
pub async fn load_products(category_id: u32) -> Vec<Product> { /* db query */ }

// Macro generates (conceptually):
pub async fn load_products(category_id: u32) -> Vec<Product>
where
    u32: std::hash::Hash,
    Vec<Product>: Clone + Send + Sync + 'static,
{
    struct __FerroMemoMarker42;  // unique per expansion
    let __key = ::ferro::memo::MemoKey::new::<__FerroMemoMarker42, _>(&category_id);

    if let Some(__store) = ::ferro::memo::current_memo_store() {
        let __slot = __store.get_or_insert(__key, move || {
            Box::pin(async move {
                let __result: Vec<Product> = {
                    /* original body */
                };
                ::std::sync::Arc::new(__result) as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>
            })
        });
        let __arc = __slot.await;
        return __arc.downcast_ref::<Vec<Product>>()
            .expect("MemoStore type invariant")
            .clone();
    }
    /* body un-memoized (D-02 graceful no-op) */
}
```

### Example 5: Table test pattern (verified from ferro-mcp-server tests)

```rust
// Source: ferro-mcp-server/tests/intent_loop.rs:439 (AtomicUsize counter pattern)
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn memoize_runs_body_once_per_key() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    #[memoize]
    async fn load(id: u32, _counter: Arc<AtomicUsize>) -> u32 {
        _counter.fetch_add(1, Ordering::SeqCst);
        id * 2
    }

    let store = Arc::new(crate::memo::MemoStore::new());
    let result = crate::memo::MEMO_STORE.scope(store, async {
        let a = load(1, counter.clone()).await;
        let b = load(1, counter.clone()).await; // same args → should hit cache
        let c = load(2, counter.clone()).await; // different arg → recomputes
        (a, b, c)
    }).await;

    assert_eq!(result.0, 2);
    assert_eq!(result.1, 2);  // same as a
    assert_eq!(result.2, 4);
    assert_eq!(counter.load(Ordering::SeqCst), 2); // ran twice (id=1 once, id=2 once)
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Request extensions type-map (`Request::insert`) | Task-local (`tokio::task_local!`) | D-01 locked in 259-CONTEXT.md | No `Request` handle needed in deep call trees; consistent with TENANT/THEME/SESSION pattern |
| No memoization in ferro render path | `#[memoize]` + `MemoStore` | Phase 259 | N intents over one key issue one fetch |
| `futures::future::Shared` not in `framework` | Add `futures` to `framework/Cargo.toml` | Phase 259 | Enables the coalescing primitive |

**Note on design spec discrepancy:** The authoritative design spec (`docs/superpowers/specs/2026-07-21-live-projection-surface-design.md §1`) says "A `MemoStore` lives in the request extensions type-map (`Request::insert`...)". The CONTEXT.md D-01 overrides this with the task-local pattern. The planner must follow D-01 (task-local). The spec was written before the discuss-phase clarified the access mechanism.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `MemoStore` should use `std::sync::Mutex` (not `tokio::sync::Mutex`) because the critical section is non-async (no `.await` inside the lock) | Standard Stack / Pattern 2 | If wrong: deadlock potential in single-threaded tokio context; fix by switching to `tokio::sync::Mutex` and making `get_or_insert` async |
| A2 | `#[memoize]` is applied to implementation methods, not trait declarations (analogous to `#[handler]`) | Pattern 6 | If the team decides to apply it to trait declarations, macro must parse `TraitItemFn` not `ItemFn` |
| A3 | Return type `T: Clone` is acceptable as a compiler-enforced constraint | Pattern 5 | If user wants to memoize non-Clone returns, they must wrap in `Arc<T>` themselves |
| A4 | The `DefaultHasher` is collision-resistant enough for within-request keying | Pattern 2 | Extremely unlikely to matter at within-request scale; `DefaultHasher` is not stable across processes/versions but MemoStore is per-request so this is fine |
| A5 | `Shared` future panic propagates to all current and future pollers | Pitfall 2 | Risk is low for v17.0 since the store is dropped with the request; document and defer `catch_unwind` |
| A6 | The fallback handler path in `server.rs` (around line 310) does NOT currently receive `REQUEST_HOST` scope; same exemption applies to `MEMO_STORE` | Pattern 7 | If fallback handlers need memo store, the scope entry must be added to the fallback path too |

---

## Open Questions (RESOLVED)

1. **Multi-argument hash tuple construction in the macro**
   - What we know: Multiple value arguments need to be combined into a single hashable value.
   - What's unclear: The macro must construct a tuple `(arg1, arg2, ...)` from the parsed `FnArg::Typed` items. `syn::Pat::Ident` names are available; creating a `(a, b, c)` expression in `quote!` is straightforward but must handle the case where one argument's binding uses a complex pattern.
   - Recommendation: Restrict initial implementation to `Pat::Ident` argument patterns (most common), emit `compile_error!` for tuple/struct destructuring arguments. Document as a limitation.
   - RESOLVED: `Pat::Ident`-only value args; non-`Pat::Ident` patterns emit a `to_compile_error()` naming the offending arg (Plan 259-02, Task 1, step 6). Documented as the v17.0 limitation.

2. **`#[memoize]` on trait method vs impl method**
   - What we know: The macro can be applied to `async fn` in either location.
   - What's unclear: Should the macro work on trait declarations (rewriting the trait body) or only on impl methods? Trait declaration rewriting is harder (must parse `TraitItemFn` not `ItemFn`) and changes the trait's public API shape.
   - Recommendation: Claude's discretion — apply to impl methods only for v17.0.
   - RESOLVED: Impl methods only for v17.0 (parse `ItemFn`, exclude `FnArg::Receiver` from the key); trait-declaration rewriting is out of scope. Proven by the `service_method_memoized` test (Plan 259-02, Task 2).

3. **Render-path wiring integration point for D-05**
   - What we know: The current `ServiceDef → JsonUiRenderer` render pass is schema-only (no data fetch).
   - What's unclear: Does D-05 intend to ADD a memoized loader to the render-pipeline call chain, or is the test harness standalone (memoized loader called before `render()`)?
   - Recommendation: The test harness uses a memoized loader that is called before (not inside) the render pass, simulating how a real multi-intent handler would work. This is the honest implementation; document it in the plan.
   - RESOLVED: Constructed-loader harness — a `#[memoize]`d loader called around a genuine multi-intent render (`derive_intents` + real `JsonUiRenderer`), asserting a single underlying fetch; no fabricated in-renderer fetch (Plan 259-03, Task 1). The render pass stays schema-only per D-05.

---

## Environment Availability

All dependencies are within the workspace:

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `tokio` (task_local, rt, macros) | `MEMO_STORE` task-local | ✓ | 1.48.0 | — |
| `futures` (Shared, BoxFuture) | `MemoStore` | Indirect ✓ (in lock) | 0.3.31 | Must add to `framework/Cargo.toml` |
| `syn` (full, parsing) | `#[memoize]` macro | ✓ | 2.x | — |
| `quote` | `#[memoize]` macro | ✓ | 1.x | — |
| `proc-macro2` | `#[memoize]` macro | ✓ | 1.x | — |
| `ferro-projections` (feature=projections) | render-path test | ✓ | workspace | Tests need `--all-features` |
| `ferro-json-ui` (feature=projections) | render-path test | ✓ | workspace | Tests need `--all-features` |

**Missing dependencies with no fallback:**
- None.

**Action required:**
- Add `futures = { version = "0.3", default-features = false, features = ["std"] }` to `framework/Cargo.toml` under `[dependencies]`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `#[tokio::test]` (tokio 1.48.0) + `#[test]` for sync tests |
| Config file | No separate config — cargo test discovers tests in `framework/src/memo/mod.rs` |
| Quick run command | `cargo test -p ferro-rs memo --all-features` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LIVE-01-A | `#[memoize]` fn body runs at most once per `(callsite, args)` within a request; distinct args recompute | Unit (async) | `cargo test -p ferro-rs memo::tests::memoize_runs_body_once_same_args --all-features` | ❌ Wave 0 |
| LIVE-01-B | Two concurrent callers of same `(callsite, args)` share one computation (coalescing) | Unit (async, concurrent) | `cargo test -p ferro-rs memo::tests::concurrent_callers_coalesce --all-features` | ❌ Wave 0 |
| LIVE-01-C | Store is dropped with the request | Unit | `cargo test -p ferro-rs memo::tests::store_dropped_after_scope --all-features` | ❌ Wave 0 |
| LIVE-01-D | N intents over one key issue exactly one underlying fetch (render-path integration) | Integration (async, projections feature) | `cargo test -p ferro-rs memo::tests::render_path_single_fetch --all-features` | ❌ Wave 0 |
| LIVE-01-E | Out-of-scope call (no store) runs un-memoized, no panic | Unit | `cargo test -p ferro-rs memo::tests::out_of_scope_is_noop --all-features` | ❌ Wave 0 |
| LIVE-01-F | Non-hashable argument is rejected at compile time | Compile-error (doc/negative test) | Manual or `trybuild` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-rs memo --all-features`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `framework/src/memo/mod.rs` — covers LIVE-01-A through LIVE-01-E
- [ ] `framework/src/memo/mod.rs` `#[cfg(feature = "projections")]` submodule — covers LIVE-01-D
- [ ] `ferro-macros/src/memoize.rs` — the proc macro implementation
- [ ] Framework install: `futures = { version = "0.3", default-features = false, features = ["std"] }` added to `framework/Cargo.toml`

*(No existing test infrastructure covers any of these; all are new.)*

---

## Security Domain

`security_enforcement` not explicitly set in `.planning/config.json` → treat as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | n/a (no auth surface) |
| V3 Session Management | No | n/a (memo store is per-request, not per-session) |
| V4 Access Control | No | n/a (memoization is an optimization, not a gate) |
| V5 Input Validation | Yes (limited) | Argument hash collision is not a security boundary — the store is request-scoped and an attacker cannot craft a collision that leaks data across requests |
| V6 Cryptography | No | `DefaultHasher` is used for optimization keying, not security; no cryptographic hash needed |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Hash collision causing wrong cached value within a request | Tampering | Acceptable risk: within-request only, `DefaultHasher` collisions are astronomically rare for practical arg values; document as known limitation |
| Shared future leaking data across requests | Information Disclosure | Not possible by construction: `MemoStore` is created fresh per request and dropped when the scope exits; task-local storage is per-task |
| `downcast` panic leaking type information in production | Denial of Service | Wrap with `.expect("...")` messages that do not expose user data; panics from type mismatches indicate macro bugs, not user input |

**Security summary:** The memoization store is a correctness/performance primitive with no auth, session, or access-control implications. The primary concern is the "Shared future panic" scenario (Pitfall 2), which is a reliability concern, not a security one.

---

## Sources

### Primary (HIGH confidence)

- `framework/src/tenant/context.rs` — `TENANT_CONTEXT` task-local pattern (verbatim model for `MEMO_STORE`)
- `framework/src/theme/context.rs` — `CURRENT_THEME` simpler variant (direct model for `MEMO_STORE` without RwLock)
- `framework/src/session/middleware.rs:192-197` — nested scope pattern
- `framework/src/server.rs:278-281` — scope entry point for `REQUEST_HOST`
- `framework/src/http/request.rs:7,88-103` — `Any + TypeId` type-erased extensions (precedent for the pattern)
- `ferro-macros/src/action.rs` — `ItemFn` parsing, body rewriting, `quote!` generation (direct model)
- `ferro-macros/src/utils.rs:88-121` — `FnArg::Receiver` detection pattern
- `ferro-json-ui/src/projection/builder.rs:56-124` — confirms render pass is schema-only (critical D-05 finding)
- `ferro-projections/src/derive.rs:75-80` — confirms `derive_intents` is schema-only
- `Cargo.lock` — `futures = 0.3.31`, `tokio = 1.48.0`, `syn = 2.x`, `quote = 1.x`
- `ferro-macros/Cargo.toml` — `syn = { version = "2", features = ["full", "parsing"] }`, `proc-macro2 = "1"`, `quote = "1"`, `tokio` dev dep

### Secondary (MEDIUM confidence)

- Context7 `futures 0.3.32` docs — `Shared` requires `Output: Clone`; `FutureExt::shared()` API shape
- `ferro-mcp-server/tests/intent_loop.rs:439` — `Arc<AtomicUsize>` counter test pattern (verified in codebase)
- `app/src/tests/mcp_write_dispatch.rs:447` — additional counter pattern example

### Tertiary (LOW confidence — verify at implementation)

- Macro callsite uniqueness via `static AtomicUsize` counter — standard proc-macro practice, not explicitly used in ferro macros today
- `std::sync::Mutex` vs `tokio::sync::Mutex` choice for `MemoStore.entries` — based on "no async in critical section" reasoning; confirm at implementation

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified in Cargo.lock/Cargo.toml
- Architecture: HIGH — patterns directly observed from working ferro code
- `MemoStore` internals / `Shared` behavior: MEDIUM — `futures` docs verified but exact macro code requires compiler verification
- Render-path finding: HIGH — directly verified by reading builder.rs; the schema-only nature is unambiguous
- Pitfalls: HIGH — classic async Rust patterns confirmed by existing ferro test patterns

**Research date:** 2026-07-21
**Valid until:** 2026-08-21 (stable Rust crate versions; tokio task-local API does not change)
