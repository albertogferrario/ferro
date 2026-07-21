---
phase: 259-request-scoped-memoization
reviewed: 2026-07-21T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - ferro-macros/src/memoize.rs
  - ferro-macros/src/lib.rs
  - framework/src/memo/mod.rs
  - framework/src/memo/macro_tests.rs
  - framework/src/memo/render_path_tests.rs
  - framework/src/lib.rs
  - framework/src/server.rs
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: resolved
resolved: 2026-07-21
resolution:
  WR-01: fixed (borrow args in key tuple) + regression test added
  IN-02: fixed (panic-propagation caveat documented)
  IN-01: accepted (advisory — generic memoized fn with explicit where clause, outside v17.0 concrete-type scope)
---

# Phase 259: Code Review Report

**Reviewed:** 2026-07-21
**Depth:** standard
**Files Reviewed:** 7
**Status:** resolved (fixes applied 2026-07-21)

## Resolution (2026-07-21)

- **WR-01 — FIXED.** `ferro-macros/src/memoize.rs` key tuple now borrows each argument
  (`&( #( &#value_arg_names, )* )`), so non-`Copy` args (`String`/`Vec`/structs) are no
  longer consumed before the body. `&T: Hash` keeps the key bytes identical. New
  regression test `memo::macro_tests::memoize_supports_non_copy_args` (a `String` arg)
  locks it in — 21/21 memo tests green; `fmt --check` + `clippy --all-targets --all-features
  -D warnings` clean.
- **IN-02 — FIXED.** Panic-propagation caveat added to the `#[memoize]` module docs
  (`# Limitation` section): a panicking body has no defined cross-caller contract via
  `futures::future::Shared`; memoize infallible or `Result`-returning bodies.
- **IN-01 — ACCEPTED (advisory, deferred).** Generic memoized fns carrying their own
  explicit `where` clause are outside the v17.0 intended use (loaders take concrete types);
  the current tests and render path use concrete types only. Left as documented advisory
  debt; revisit if a generic-loader use case appears.

## Summary

The memoization implementation is well-structured and honours the D-01 through D-05 decisions:
the `MemoStore` is correctly scoped per-request via `tokio::task_local!`, the concurrency
model (lock dropped before `.await`, `futures::future::Shared` for coalescing) is sound, the
D-02 graceful no-op path is correct, the crate path is `::ferro` not `::ferro_rs`, the
async-only guard emits a clear compile error, and the render-path test is honest about the
schema-only renderer.

One warning-level bug exists in the proc macro: the argument-hashing expression moves
non-`Copy` arguments into a temporary tuple before the if-let branch, making the macro
unusable with `String`, `Vec<T>`, or any other non-`Copy` argument type. All current tests
use `u32` which is `Copy`, so the bug is masked today. Two lower-severity notes follow
(where-clause duplication on generic fns, undocumented panic propagation through
`futures::future::Shared`).

---

## Warnings

### WR-01: Non-`Copy` argument types cause a compile error in the generated wrapper

**File:** `ferro-macros/src/memoize.rs:135-137`

**Issue:** The expression `&( #(#value_arg_names,)* )` constructs a temporary owned tuple
by **moving** each value argument into it. `MemoKey::new` takes a shared reference to that
temporary, hashes it, and then the temporary is dropped. For `Copy` types (`u32`, `i64`,
etc.) this is harmless because "move" is silently a copy. For any non-`Copy` type
(`String`, `Vec<T>`, user structs, `Arc<T>` as a consumed value) the argument is gone
after the `MemoKey::new` call, and the compiler rejects the generated code with a
"use of moved value" error — pointing at generated code the user never wrote.

All macro tests in `macro_tests.rs` and `render_path_tests.rs` pass `u32`, so this branch
of the generated code has never been exercised with a type that would trigger the error.
Any downstream user who writes `#[memoize] async fn fetch_product(slug: String) -> Product`
will hit a confusing compiler error in generated code.

**Fix:** Change the hash tuple to borrow each argument by reference. `&T: Hash` whenever
`T: Hash` (via `impl<T: Hash + ?Sized> Hash for &T`), so the existing `where T: Hash`
bound is still sufficient, and the hash value is identical:

```rust
// ferro-macros/src/memoize.rs  line 135-137  (current)
let __ferro_memo_key = #ferro::memo::MemoKey::new::<#marker_name, _>(
    &( #( #value_arg_names, )* ),
);

// Fixed: borrow each arg so non-Copy types are not consumed
let __ferro_memo_key = #ferro::memo::MemoKey::new::<#marker_name, _>(
    &( #( &#value_arg_names, )* ),
);
```

Add a test in `macro_tests.rs` that applies `#[memoize]` to a function whose argument
is `String` (or another non-`Copy` type) to lock this in:

```rust
static COUNTER_STR: AtomicUsize = AtomicUsize::new(0);

#[memoize]
async fn load_by_slug(slug: String) -> u32 {
    COUNTER_STR.fetch_add(1, Ordering::SeqCst);
    slug.len() as u32
}

#[tokio::test]
async fn non_copy_arg_type_compiles_and_memoizes() {
    COUNTER_STR.store(0, Ordering::SeqCst);
    let store = memo_scope();
    let (a, b) = with_memo_scope(store, async {
        let a = load_by_slug("hello".to_string()).await;
        let b = load_by_slug("hello".to_string()).await; // cache hit
        (a, b)
    }).await;
    assert_eq!(a, 5);
    assert_eq!(b, 5);
    assert_eq!(COUNTER_STR.load(Ordering::SeqCst), 1);
}
```

---

## Info

### IN-01: Where-clause duplication if a memoized fn already has a `where` clause

**File:** `ferro-macros/src/memoize.rs:127-131`

**Issue:** The macro emits:

```rust
async fn #fn_name #fn_generics(#(#all_inputs),*) #fn_output
where
    #( #value_arg_types: ::std::hash::Hash, )*
    #return_ty: Clone + Send + Sync + 'static,
```

`#fn_generics` renders the full `syn::Generics` including any pre-existing `where` clause.
If the user writes a generic memoized function with its own `where` constraint, the output
contains two `where` keywords in sequence, which is a syntax error:

```rust
// User writes:
#[memoize]
async fn load<T: Entity>(id: u32) -> T where T: Deserialize { ... }

// Generated (invalid):
async fn load<T: Entity>(id: u32) -> T where T: Deserialize
where
    u32: Hash,
    T: Clone + Send + Sync + 'static,
```

This is a compile error, not silent misbehavior. Typical data loaders take only concrete
types and are unaffected. Generic memoized functions are rare but the error message is
confusing because it points at generated code.

**Fix:** Use `split_for_impl()` or strip the where clause from `fn_generics` before
quoting, then merge all bounds into a single where clause:

```rust
// In memoize_impl, replace the #fn_generics use in the signature:
let (impl_generics, _, where_clause) = fn_generics.split_for_impl();
// ...
let existing_predicates = where_clause
    .map(|wc| wc.predicates.iter().cloned().collect::<Vec<_>>())
    .unwrap_or_default();

// Then in the quote!:
async fn #fn_name #impl_generics (#(#all_inputs),*) #fn_output
where
    #( #existing_predicates, )*
    #( #value_arg_types: ::std::hash::Hash, )*
    #return_ty: Clone + Send + Sync + 'static,
```

Severity is low: the error is immediate and compile-time, and the affected case (generic
`#[memoize]` functions) is outside the v17.0 intended use.

---

### IN-02: Panic propagation through `futures::future::Shared` is not documented

**File:** `framework/src/memo/mod.rs:31`, `ferro-macros/src/memoize.rs` (generated code)

**Issue:** D-04 documents that `Result::Err` is cached (every caller in the request
sees the same error). It does not address the panic case. If the memoized body panics,
`futures::future::Shared` will propagate the panic only to the task currently polling
the inner future; other tasks awaiting the same `Shared` slot will receive a
`futures::future::Aborted`-style error or hang, depending on the runtime version and
whether the panic unwinds through the executor. In `tokio`, an async task that panics
is typically caught by the task supervisor; a `Shared` future whose underlying future
panicked will resolve to a panic-propagation for other waiters.

This is not a code defect in the current implementation (the chosen approach matches
the spec's D-04 scope), but it is an undocumented edge that authors should know about
before memoizing functions that can panic.

**Fix:** Add a note to the `MemoStore::get_or_insert` doc comment and to the `#[memoize]`
doc comment in `lib.rs`:

```rust
// In framework/src/memo/mod.rs, add to get_or_insert doc:
/// If the inner future panics, the panic propagates to the awaiting caller;
/// other concurrent callers sharing the same slot will also observe the panic
/// (via `futures::future::Shared` panic propagation semantics). Memoize only
/// infallible or `Result`-returning functions; never memoize functions that are
/// expected to panic.
```

---

## Clean Areas (no findings)

The following areas were examined and are correct:

- **Lock/await ordering (`mod.rs:95-100`):** The `Mutex` guard is released inside the
  inner `{ ... }` block and the `MemoSlot` (an already-cloned `Shared`) is returned and
  awaited outside any lock. No lock is held across an `.await` point.

- **TypeId uniqueness (D-03):** `__FerroMemoMarker{n}` is a local struct defined inside
  each rewritten function body. Its `TypeId` is unique per expansion site because `n`
  monotonically increases within a compilation (via `AtomicUsize`) and local structs in
  distinct parent functions produce distinct `TypeId`s in Rust's type identity model.
  Two different crates compiled independently can produce the same `n` but cannot produce
  the same `TypeId` (different parent fn scopes). The design is sound.

- **D-02 graceful no-op:** `current_memo_store()` uses `MEMO_STORE.try_with(...)`.ok()`
  and returns `None` when no scope is active. The generated wrapper falls through to
  `{ fn_block }` with no panic. Background jobs and tests without an explicit scope work
  correctly. The `out_of_scope_is_noop` test in `macro_tests.rs` covers this.

- **Crate path:** The macro emits `#ferro` which resolves to `::ferro` via `utils::ferro()`.
  No hardcoded `::ferro_rs`. The `#[cfg(test)] extern crate self as ferro;` in
  `framework/src/lib.rs:10` makes `::ferro` resolve to the crate itself in test builds,
  so in-crate tests using `#[memoize]` compile correctly.

- **Async-only guard:** A synchronous function annotated with `#[memoize]` produces
  `syn::Error::new_spanned(...)` with a clear message, not a confusing type error.

- **Request isolation (D-01):** `MEMO_STORE` is a `tokio::task_local!` holding an
  `Arc<MemoStore>`. Each request creates a fresh `MemoStore` in `server.rs:279-284`
  and enters a new scope. There is no global `HashMap`, no cross-request or cross-tenant
  retention.

- **Server.rs nesting:** The memo scope is correctly entered around `chain.execute()`
  alongside `REQUEST_HOST.scope`. Fallback handlers intentionally omit both scopes
  (comment at `server.rs:315-317`), degrading gracefully per D-02.

- **SC-3 harness honesty (`render_path_tests.rs`):** The test calls the real `derive_intents`
  and real `ferro_json_ui::Spec::from_service_def`. It does not fabricate a fetch inside
  the renderer. The memoized `load_model_set` loader is called explicitly before each
  render — modelling the actual multi-intent handler pattern. The architecture footnote at
  the top of the file accurately explains why the fetch is outside the renderer (the
  renderer is schema-only).

- **`futures` feature coverage:** `framework/Cargo.toml` depends on `futures` with
  `features = ["std"]`. The `std` feature pulls in `futures-util` which contains
  `futures::future::Shared`, `BoxFuture`, and `FutureExt`. These are all present in the
  resolved `Cargo.lock`.

---

_Reviewed: 2026-07-21_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
