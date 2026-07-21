---
phase: 259-request-scoped-memoization
plan: "02"
subsystem: ferro-macros + framework
tags: [memoization, proc-macro, request-scoped, coalescing, task-local]
dependency_graph:
  requires:
    - "259-01: MemoStore, MemoKey, current_memo_store(), MEMO_STORE task-local"
  provides:
    - "#[memoize] attribute macro at ::ferro::memoize"
    - "ferro-macros/src/memoize.rs: memoize_impl with async guard, &self exclusion, Hash/Clone bounds"
    - "framework/src/memo/macro_tests.rs: 5 end-to-end tests (SC-1, SC-2, LIVE-01, D-02, D-04)"
  affects:
    - ferro-macros/src/memoize.rs (new)
    - ferro-macros/src/lib.rs (mod memoize + proc_macro_attribute registration)
    - framework/src/memo/macro_tests.rs (new)
    - framework/src/memo/mod.rs (#[cfg(test)] mod macro_tests registration)
    - framework/src/lib.rs (pub use ferro_macros::memoize + #[cfg(test)] extern crate self as ferro)
tech_stack:
  added: []
  patterns:
    - "static AtomicUsize MEMO_COUNTER: per-expansion unique marker name mint"
    - "struct __FerroMemoMarker{n}: zero-sized per-callsite TypeId marker"
    - "FnArg::Receiver filter: &self excluded from args hash, preserved in signature"
    - "Pat::Ident restriction: v17.0 limits to simple identifier patterns"
    - "#[cfg(test)] extern crate self as ferro: enables in-crate tests using macros that generate ::ferro:: paths"
key_files:
  created:
    - ferro-macros/src/memoize.rs
    - framework/src/memo/macro_tests.rs
  modified:
    - ferro-macros/src/lib.rs
    - framework/src/memo/mod.rs
    - framework/src/lib.rs
decisions:
  - "Used #[cfg(test)] extern crate self as ferro in framework/src/lib.rs so in-crate tests using #[memoize] (which generates ::ferro::memo:: paths) resolve correctly without widening pub(crate) scope of memo helpers"
  - "In-crate test module (src/memo/macro_tests.rs) chosen over integration tests (framework/tests/) because memo_scope() and with_memo_scope() are pub(crate) and cannot be reached from the tests/ directory"
  - "Trailing-comma tuple form &(#(#value_arg_names,)*) for the args hash: uniformly produces a tuple for 0, 1, and N args — avoids special-casing the single-arg parenthesised-expression vs 1-tuple ambiguity"
  - "v17.0 Pat::Ident restriction: non-identifier argument patterns emit compile_error! naming the offending arg"
metrics:
  duration: "~25 minutes"
  completed: "2026-07-21"
  tasks: 2
  files: 5
requirements: [LIVE-01]
---

# Phase 259 Plan 02: `#[memoize]` Proc-Macro Summary

`#[memoize]` attribute macro in `ferro-macros` rewriting async fns to look up the per-request `MemoStore`, coalescing concurrent callers via `futures::future::Shared`, with full end-to-end proof from `ferro-rs`.

## What Was Built

**Task 1 — `#[memoize]` proc-macro + registration + re-export** (`9193e39f`)

- `ferro-macros/src/memoize.rs` (155 lines): `memoize_impl(attr, input) -> TokenStream`:
  - `static MEMO_COUNTER: AtomicUsize` — mints a unique `__FerroMemoMarker{n}` per expansion
  - async-only guard: `syn::Error::new_spanned(..., "#[memoize] can only be applied to `async fn`")`
  - `FnArg::Receiver` filter: `&self`/`&mut self` excluded from value arg list (D-03); full `all_inputs` retained in the emitted signature
  - `Pat::Ident` restriction: non-identifier patterns emit a named `compile_error!` (v17.0 limitation)
  - `where` clause: `#(#value_arg_types: ::std::hash::Hash,)*` + `#return_ty: Clone + Send + Sync + 'static`
  - Generated body: `::ferro::memo::MemoKey::new::<#marker_name, _>(&(#(#value_arg_names,)*))` → `current_memo_store()` lookup → `get_or_insert` → `.await` → `downcast_ref::<#return_ty>().clone()` on HIT; direct `{ #fn_block }` on D-02 fallthrough
  - Uses `crate::utils::ferro()` → `quote!(::ferro)` — no hardcoded `::ferro_rs`
- `ferro-macros/src/lib.rs`: `mod memoize;` + `#[proc_macro_attribute] pub fn memoize(...)` with rustdoc
- `framework/src/lib.rs`: `pub use ferro_macros::memoize;` (between `injectable` and `redirect`, alphabetically) + `#[cfg(test)] extern crate self as ferro;` (see Deviations)

**Task 2 — End-to-end macro tests** (`49d54a6d`)

- `framework/src/memo/macro_tests.rs` (165 lines): 5 named tests:

  | Test | Criterion | Result |
  |------|-----------|--------|
  | `memoize_runs_body_once_same_args` | SC-1: same args hit cache, distinct args recompute | pass |
  | `concurrent_callers_coalesce` | SC-2: `tokio::join!` on same key runs body once | pass |
  | `service_method_memoized` | LIVE-01: impl method, different `&self` instances, same id → counter == 1 | pass |
  | `out_of_scope_is_noop` | D-02: no panic outside scope, body runs each call | pass |
  | `err_is_cached` | D-04: `Result::Err` cached, counter == 1, same Err twice | pass |

- `framework/src/memo/mod.rs`: `#[cfg(test)] mod macro_tests;` registered

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build -p ferro-macros` | OK |
| `cargo build -p ferro-rs` | OK |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy -p ferro-macros -p ferro-rs --all-targets -- -D warnings` | OK (0 warnings) |
| `cargo test -p ferro-rs --lib memo::macro_tests --all-features` | 5/5 pass |
| `grep "pub use ferro_macros::memoize;" framework/src/lib.rs` | FOUND (line 353) |
| `grep "memo::MemoKey" ferro-macros/src/memoize.rs` | FOUND (line 135) |
| `grep "FnArg::Receiver" ferro-macros/src/memoize.rs` | FOUND (lines 80, 108) |
| `grep "can only be applied to" ferro-macros/src/memoize.rs` | FOUND (line 53) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `::ferro` path unresolvable in in-crate `#[memoize]` tests**

- **Found during:** Task 2 first test run
- **Issue:** The macro generates `::ferro::memo::MemoKey::new` and `::ferro::memo::current_memo_store()`. Within `ferro-rs` itself (the crate IS `ferro`), `::ferro` is not available as an external crate path — it resolves only for downstream consumers. In-crate `#[memoize]`-annotated functions emitted compile errors: `could not find 'ferro' in the list of imported crates`.
- **Fix:** Added `#[cfg(test)] extern crate self as ferro;` at the top of `framework/src/lib.rs`. This makes `::ferro` resolve to `crate` in test builds only, without affecting the production binary or API surface. This is the canonical Rust pattern for crates that test their own proc-macros in-tree (used by e.g. `serde`, `tokio`).
- **Files modified:** `framework/src/lib.rs`
- **Commit:** `49d54a6d` (folded into Task 2 commit)

**2. [Rule 1 - Bug] rustfmt reformatted `mod memoize` declaration position**

- **Found during:** `cargo fmt --all` run after Task 2 file creation
- **Issue:** The `mod memoize;` line in `ferro-macros/src/lib.rs` was placed after `mod describe;`; rustfmt sorted it alphabetically to between `mod injectable;` and `mod model;`.
- **Fix:** Applied automatically by `cargo fmt --all`. No logic changed.
- **Files modified:** `ferro-macros/src/lib.rs`
- **Commit:** `49d54a6d`

**3. [Rule 1 - Bug] rustfmt reformatted `macro_tests.rs` comment indent**

- **Found during:** `cargo fmt --all` run
- **Issue:** A comment `// Body ran exactly once — &self exclusion confirmed.` inside the `service_method_memoized` test was placed on a line where rustfmt reindented it as a continuation of the preceding `assert` alignment.
- **Fix:** Applied automatically by `cargo fmt --all`. The comment now reads correctly after format.
- **Files modified:** `framework/src/memo/macro_tests.rs`
- **Commit:** `49d54a6d`

## Known Stubs

None. All 5 criteria are proven through the real `#[memoize]` macro expansion. The `memo_scope()` / `with_memo_scope()` helpers transition from `#[allow(dead_code)]` to actively used (by the macro tests) — the `#[allow(dead_code)]` attributes can be removed in Plan 03 or remain as harmless annotations.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes. Generated code accesses only the per-request task-local `MEMO_STORE`; no new trust boundary beyond Plan 01.

T-259-04 mitigation verified: `service_method_memoized` test proves `&self` is excluded from the key (two different `ProductLoader` instances, same `id`, counter == 1).

## Self-Check: PASSED

- `ferro-macros/src/memoize.rs` exists: FOUND
- `ferro-macros/src/lib.rs` contains `pub fn memoize`: FOUND (line 295)
- `ferro-macros/src/lib.rs` contains `mod memoize`: FOUND (line 20)
- `ferro-macros/src/lib.rs` contains `memoize::memoize_impl`: FOUND (line 296)
- `framework/src/lib.rs` contains `pub use ferro_macros::memoize;`: FOUND (line 353)
- `framework/src/memo/macro_tests.rs` exists: FOUND
- `framework/src/memo/mod.rs` contains `mod macro_tests`: FOUND (line 141)
- Commit `9193e39f` exists: FOUND
- Commit `49d54a6d` exists: FOUND
- All 5 macro tests pass: VERIFIED
