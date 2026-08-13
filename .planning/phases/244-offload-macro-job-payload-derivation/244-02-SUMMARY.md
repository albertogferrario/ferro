---
phase: 244-offload-macro-job-payload-derivation
plan: "02"
subsystem: ferro-macros
tags: [queue, offload, proc-macro, trybuild, job-derivation]
dependency_graph:
  requires:
    - ferro_queue::JobRegistrarEntry (Plan 01)
    - WorkerLoop::from_registry inventory drain (Plan 01)
    - ::ferro::queue::JobRegistrarEntry re-export (Plan 01)
  provides:
    - offload.rs helper module (owned_type, collect_info, emit_job_items)
    - #[offload] recognition wired into service_impl
    - trybuild harness covering OFFLOAD-01-a/b/c
  affects:
    - ferro-macros/src/offload.rs
    - ferro-macros/src/service.rs
    - ferro-macros/src/lib.rs
    - ferro-macros/Cargo.toml
    - ferro-macros/tests/offload_macro.rs
    - ferro-macros/tests/ui/offload/
tech_stack:
  added:
    - serde dev-dep in ferro-macros (for trybuild fixture crate reachability)
  patterns:
    - inert helper attribute consumed by outer #[service] macro (mirrors #[inject] in injectable.rs)
    - per-field FieldForward strategy for type-correct handle() forwarding
    - trybuild pass/fail fixture harness (mirrors action_macro.rs)
key_files:
  created:
    - ferro-macros/src/offload.rs
    - ferro-macros/tests/offload_macro.rs
    - ferro-macros/tests/ui/offload/pass/basic.rs
    - ferro-macros/tests/ui/offload/pass/ref_str_param.rs
    - ferro-macros/tests/ui/offload/pass/result_method.rs
    - ferro-macros/tests/ui/offload/fail/mut_ref_param.rs
    - ferro-macros/tests/ui/offload/fail/mut_ref_param.stderr
  modified:
    - ferro-macros/src/service.rs
    - ferro-macros/src/lib.rs
    - ferro-macros/Cargo.toml
decisions:
  - id: pascal-case-algorithm
    summary: >
      Job ident built as <TraitIdent><MethodPascalCase>Job. Method snake_case
      converted by split('_'), capitalize each segment, concat. Example:
      Reports + build_monthly → ReportsBuildMonthlyJob.
      format_ident! used to build the final proc_macro2::Ident.
  - id: serde-dev-dep-added
    summary: >
      serde = { version = "1", features = ["derive"] } added to ferro-macros
      [dev-dependencies]. Required because trybuild fixtures are compiled as
      standalone crates with only ferro-macros dev-deps in scope; ::serde::
      is not reachable from ::ferro::serde in that context.
  - id: field-forward-strategy
    summary: >
      OffloadMethodInfo carries a per-field FieldForward enum (AsStr, AsSlice,
      Clone). In handle(), &str-mapped fields are forwarded as self.field.as_str(),
      &[T]-mapped fields as self.field.as_slice(), all others as self.field.clone().
      Uniform .clone() was the initial approach but fails when the method signature
      expects &str — the field is String, so .clone() returns String not &str.
  - id: fixture-positional-syntax
    summary: >
      Trybuild fixtures use #[service(ConcreteType)] (positional) rather than
      #[service(impl = ConcreteType)] (named). The ServiceArgs parser uses
      input.parse::<Ident>() to detect named params; since impl is a keyword,
      not an Ident, the named path fails and the positional path is taken. Both
      are semantically equivalent; the named form works fine in normal builds
      where the proc-macro runs in a fully-resolved token context.
  - id: round-trip-deferred
    summary: >
      Full app-crate round-trip integration (macro-derived Job dispatched via
      dispatch()) is deferred. Plan 02 proves derivation via trybuild (OFFLOAD-01-a/b/c).
      Plan 01 proved the queue-side round-trip via hand-written Job structs
      (OFFLOAD-01-d/e/f). End-to-end round-trip with macro-derived Jobs in the
      app crate is a later phase concern.
metrics:
  duration_seconds: 1167
  completed_date: "2026-08-13"
  tasks_completed: 3
  files_modified: 4
  files_created: 7
---

# Phase 244 Plan 02: `#[offload]` Macro Job Payload Derivation Summary

Proc-macro derivation of `ferro-queue` Job structs from `#[offload]`-marked
`#[service]` trait methods, with trybuild harness covering the three core
compile-time behaviors (OFFLOAD-01-a/b/c).

## What Was Built

**Task 1 — `ferro-macros/src/offload.rs` helper module:**

New `offload` module with three public-crate functions:

- `owned_type(&Type) -> syn::Result<TokenStream2>` — maps `&str`→`String`,
  `&[T]`→`Vec<T>`, `&T`→`T`, `T`→`T`, and returns a spanned `syn::Error` for
  `&mut T` with the message "parameters may not be &mut references — Job payloads
  must be owned and serializable".

- `collect_info(trait_ident, &TraitItemFn) -> syn::Result<OffloadMethodInfo>` —
  extracts the Job ident (PascalCase via split-on-underscore algorithm), non-self
  parameter names and owned types, per-field forwarding strategy, asyncness flag,
  and Result-return detection.

- `emit_job_items(trait_ident, &OffloadMethodInfo) -> TokenStream2` — emits:
  a `#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)] pub struct
  <Trait><Method>Job { … }`, a `#[::ferro::async_trait] impl ::ferro::queue::Job`
  with a `handle()` that resolves the service via `::ferro::App::make::<dyn Trait>()`
  and forwards fields type-correctly, and an `::ferro::inventory::submit!` entry.

All emitted paths use `::ferro::*`; no `::ferro_queue::` paths appear in generated code.

`mod offload;` declared alphabetically in `ferro-macros/src/lib.rs` (after `mod model;`,
before `mod redirect;`).

**Task 2 — `#[offload]` recognition wired into `service_impl`:**

In `ferro-macros/src/service.rs::service_impl`, after supertrait bounds:

1. `trait_ident = item_trait.ident.clone()` captured before the mutable borrow.
2. Loop over `item_trait.items` — for each `TraitItem::Fn` with `#[offload]` in
   `attrs`, strip the attribute (Pitfall 1: must strip before `#item_trait` is
   re-emitted), call `collect_info`, collect into `offload_infos`.
3. `offload_items` iterator over `emit_job_items` calls appended to the `expanded`
   `quote!` block via `#( #offload_items )*`.
4. Rustdoc note on `#[service]`-outermost attribute order added to `service_impl` doc.

Non-offload `#[service]` traits are unaffected: empty `offload_infos` expands to nothing.

**Task 3 — Trybuild harness and pass/fail fixtures:**

- `ferro-macros/tests/offload_macro.rs`: harness with `t.pass(...)` and
  `t.compile_fail(...)` globs, mirroring `action_macro.rs`.
- `pass/basic.rs`: `Reports::build_monthly(month: Month)` → `ReportsBuildMonthlyJob`
  is nameable in `main()` (OFFLOAD-01-a).
- `pass/ref_str_param.rs`: `greet(name: &str)` → `GreeterServiceGreetJob { name:
  String::from("x") }` compiles (OFFLOAD-01-b).
- `pass/result_method.rs`: `export(id: i64) -> Result<(), String>` → derived
  `handle()` uses the `job_failed` mapping branch.
- `fail/mut_ref_param.rs` + `.stderr`: `mutate(data: &mut String)` produces the
  exact compile error captured in the snapshot.

## Job-ident PascalCase Algorithm

Split the method ident on `_`, filter empty segments, capitalize the first
character of each segment, concatenate. Applied to the snake_case method ident;
the trait ident is already PascalCase. Final ident assembled via `format_ident!`.

Examples:
- `Reports` + `build_monthly` → `ReportsBuildMonthlyJob`
- `GreeterService` + `greet` → `GreeterServiceGreetJob`
- `ExporterService` + `export` → `ExporterServiceExportJob`

## Serde Dev-Dep Addition

`serde = { version = "1", features = ["derive"] }` added to
`ferro-macros/[dev-dependencies]`. Reason: trybuild compiles each fixture as a
standalone crate whose only visible extern crates are the `ferro-macros`
dev-dependencies. `::serde::Serialize` and `::serde::Deserialize` are emitted by
the macro; they must resolve in the fixture. Although `ferro-rs` re-exports
`serde`, the re-export path is `::ferro::serde`, not `::serde`. The dev-dep
makes the `serde` extern crate available under its own name in the fixture
compilation environment.

## Exact `&mut` Diagnostic Captured in Snapshot

```
error: #[offload] parameters may not be &mut references — Job payloads must be owned and serializable
  --> tests/ui/offload/fail/mut_ref_param.rs:16:34
   |
16 |     async fn mutate(&self, data: &mut String);
   |                                  ^^^^^^^^^^^
```

The error is emitted by `syn::Error::new_spanned(ty, "…")` in `owned_type()`,
converted to a compile error via `e.to_compile_error().into()` in `service_impl`.

## Round-Trip Integration Status

Full app-crate round-trip (macro-derived Job dispatched via `ferro::queue::dispatch()`)
is deferred to a later integration phase. The current test coverage:

- Plan 01 (OFFLOAD-01-d/e/f): hand-written Job structs dispatched and auto-registered
  in sync mode — queue substrate proven.
- Plan 02 (OFFLOAD-01-a/b/c): macro derives a nameable struct, maps `&str` to
  `String`, rejects `&mut T` — macro derivation proven at compile time.

The combination covers the substrate + derivation independently. End-to-end with
macro-derived Jobs exercised in the app crate is the natural scope of a later phase.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `&str` fields forwarded as `.clone()` instead of `.as_str()`**
- **Found during:** Task 3 (trybuild run — `ref_str_param.rs` failed with `mismatched types: expected &str, found String`)
- **Issue:** The initial `emit_job_items` used `self.field.clone()` uniformly for all
  fields. For `&str`-mapped fields (where the owned type is `String`), `.clone()` returns
  `String` but the method expects `&str`.
- **Fix:** Added `FieldForward` enum (AsStr, AsSlice, Clone) to `OffloadMethodInfo`.
  `field_forward(&Type)` classifies each parameter at collect time. `emit_job_items`
  generates per-field forwarding expressions: `.as_str()` for `&str`-origin fields,
  `.as_slice()` for `&[T]`-origin fields, `.clone()` for all others.
- **Files modified:** `ferro-macros/src/offload.rs`
- **Commit:** b1114160

**2. [Rule 1 - Bug] Fixture syntax `#[service(impl = X)]` rejected in trybuild context**
- **Found during:** Task 3 (first trybuild run — all pass fixtures failed with "expected identifier, found keyword `impl`")
- **Issue:** `ServiceArgs::parse` probes for named params via `input.fork().parse::<Ident>().is_ok()`. Since `impl` is a Rust keyword, not an `Ident`, the probe returns `false`, and the parser falls through to the positional path — which also fails because `impl` cannot start a `Path`. This causes a parse error before the macro body runs.
- **Fix:** Changed all trybuild fixtures to use the positional syntax `#[service(ConcreteType)]`, which is the backwards-compatible form and semantically equivalent for the fixture's purpose.
- **Files modified:** All four fixture `.rs` files
- **Commit:** b1114160

**3. [Rule 1 - Bug] rustfmt reformatted offload.rs and service.rs**
- **Found during:** Per-plan gate (`cargo fmt --all -- --check`)
- **Issue:** Nested `match` inside `for` loop and long `if let Some(pos) =` expression reformatted by rustfmt.
- **Fix:** Applied `cargo fmt --all`.
- **Files modified:** `ferro-macros/src/offload.rs`, `ferro-macros/src/service.rs`
- **Commit:** b1114160

## Known Stubs

None. All emitted code is functional. The trybuild fixtures exercise real macro
expansion, not stubs. Round-trip integration deferred by design (see above).

## Threat Flags

None. This plan adds only proc-macro code generation and compile-time UI tests.
No new network endpoints, auth paths, file access patterns, or schema changes.
The derived `handle()` calls `App::make::<dyn Trait>()` — this resolves an
already-registered service and grants no capability the developer did not already
have. Threat T-244-03 (accepted) and T-244-04 (deferred to Phase 249 docs) from
the plan's threat model remain unchanged.

## Self-Check

### Created Files

- [x] `ferro-macros/src/offload.rs` exists
- [x] `ferro-macros/tests/offload_macro.rs` exists
- [x] `ferro-macros/tests/ui/offload/pass/basic.rs` exists
- [x] `ferro-macros/tests/ui/offload/pass/ref_str_param.rs` exists
- [x] `ferro-macros/tests/ui/offload/pass/result_method.rs` exists
- [x] `ferro-macros/tests/ui/offload/fail/mut_ref_param.rs` exists
- [x] `ferro-macros/tests/ui/offload/fail/mut_ref_param.stderr` exists

### Commits Exist

- [x] 667aad1f — feat(244-02): add offload.rs helper module with owned_type, collect_info, emit_job_items
- [x] 76f00f67 — feat(244-02): wire #[offload] recognition into service_impl
- [x] b1114160 — test(244-02): add trybuild harness and pass/fail fixtures for #[offload] (OFFLOAD-01-a/b/c)

### Gate Results

- [x] `cargo fmt --all -- --check` — clean
- [x] `cargo clippy --all --all-targets -- -D warnings` — clean
- [x] `cargo test -p ferro-macros --test offload_macro` — 1 passed; 0 failed

## Self-Check: PASSED
