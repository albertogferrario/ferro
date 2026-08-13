# Phase 244: `#[offload]` macro → Job + payload derivation — Pattern Map

**Mapped:** 2026-08-13
**Files analyzed:** 8 (5 modified, 1 new module, 2 new test files)
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-macros/src/service.rs` | proc-macro (code-gen) | transform | self (extended) | self |
| `ferro-macros/src/offload.rs` | proc-macro helper module | transform | `ferro-macros/src/memoize.rs` | role-match |
| `ferro-macros/src/lib.rs` | proc-macro entry-point | — | self (minor) | self |
| `ferro-queue/src/db.rs` | registry type + inventory | event-driven | `framework/src/container/provider.rs` | role-match |
| `ferro-queue/src/worker.rs` | worker boot | event-driven | self (extended) | self |
| `ferro-queue/Cargo.toml` | config | — | `ferro-macros/Cargo.toml` | partial |
| `ferro-macros/tests/offload_macro.rs` | test (trybuild harness) | — | `ferro-macros/tests/action_macro.rs` | exact |
| `ferro-queue/tests/offload_round_trip.rs` | test (integration) | request-response | `ferro-queue/src/dispatcher.rs` (sync mode) | role-match |

---

## Pattern Assignments

### `ferro-macros/src/service.rs` — extended to strip `#[offload]` and emit derived items

**Role:** The primary proc-macro implementation site. The `service_impl()` function at line 107
iterates `item_trait.items` to add `Send + Sync + 'static` bounds and, optionally, emits an
`inventory::submit!` binding. Phase 244 adds a second pass over `item_trait.items` that strips
`#[offload]` from method attrs and collects their signatures for derivation.

**Analog:** `ferro-macros/src/injectable.rs` lines 168–178 (helper-attribute filter pattern) +
`ferro-macros/src/service.rs` lines 165–178 (existing `inventory::submit!` emission).

**Helper-attribute stripping pattern** (`ferro-macros/src/injectable.rs:173–178`):
```rust
// filter out a helper attribute before re-emitting the field definition
let other_attrs: Vec<_> = field
    .attrs
    .iter()
    .filter(|attr| !attr.path().is_ident("inject"))
    .collect();
```

**Analogous mutation pattern for trait method items** (to be added inside `service_impl()`
after line 109, before the bounds-adding block):
```rust
// Collect methods marked #[offload] and strip the attribute so rustc never sees it.
let mut offload_methods: Vec<OffloadMethodInfo> = Vec::new();
for item in &mut item_trait.items {
    if let syn::TraitItem::Fn(method) = item {
        let pos = method.attrs.iter().position(|a| a.path().is_ident("offload"));
        if let Some(pos) = pos {
            method.attrs.remove(pos);   // strip before re-emitting item_trait
            offload_methods.push(crate::offload::collect_info(trait_name, method)?);
        }
    }
}
```

**Existing `inventory::submit!` emission pattern** (`ferro-macros/src/service.rs:166–178`):
```rust
let impl_registration = args.impl_type.as_ref().map(|concrete_type| {
    quote! {
        #ferro::inventory::submit! {
            #ferro::container::provider::ServiceBindingEntry {
                register: || {
                    #ferro::App::bind::<dyn #trait_name>(
                        ::std::sync::Arc::new(
                            <#concrete_type as ::std::default::Default>::default()
                        );
                    ),
                    name: #trait_name_str,
                }
            }
        }
    }
});
```

**Delta for phase 244:** Insert the `offload_methods` collection loop before line 113; append the
`emit_job_items(&offload_methods)` token stream to the `expanded` `quote!` block at line 208.
The emitted token stream format is documented in the `ferro-macros/src/offload.rs` section below.

---

### `ferro-macros/src/offload.rs` — NEW helper module

**Role:** Contains all `#[offload]`-specific logic: `collect_info()`, `owned_type()`,
`emit_job_struct()`, `emit_impl_job()`. Extracted into a sibling module to keep `service.rs`
focused.

**Analog:** `ferro-macros/src/memoize.rs` — directly comparable: it also processes trait/fn
signatures (`ItemFn`, `FnArg`, `Pat`), handles the `asyncness` flag, and emits wrapper code.

**FnArg iteration and asyncness detection pattern** (`ferro-macros/src/memoize.rs:87–123`):
```rust
// Split receiver from value args — receivers are excluded from the generated artifact.
let value_inputs: Vec<_> = input_fn
    .sig
    .inputs
    .iter()
    .filter(|a| !matches!(a, FnArg::Receiver(_)))
    .collect();

// Extract binding idents — only Pat::Ident patterns are supported.
for arg in &value_inputs {
    match arg {
        FnArg::Typed(pat_type) => {
            let ty = &*pat_type.ty;
            match &*pat_type.pat {
                Pat::Ident(pat_ident) => {
                    value_arg_names.push(pat_ident.ident.clone());
                    value_arg_types.push(ty);
                }
                other => {
                    return syn::Error::new_spanned(
                        other,
                        "#[memoize] arguments must be simple identifiers …",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        FnArg::Receiver(_) => {}
    }
}
```

**Asyncness detection pattern** (`ferro-macros/src/memoize.rs:61–68`):
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
For `#[offload]`, use `method.sig.asyncness.is_some()` to select between `.await` and direct call
emission — both are accepted (D-06 / RQ-3).

**Reference-type-to-owned mapping function** (new, no direct analog — use the RESEARCH.md
specification verbatim at `ferro-macros/src/offload.rs`):
```rust
fn owned_type(ty: &syn::Type) -> syn::Result<proc_macro2::TokenStream> {
    match ty {
        syn::Type::Reference(r) => {
            if r.mutability.is_some() {
                return Err(syn::Error::new_spanned(ty,
                    "#[offload] parameters may not be &mut references — \
                     Job payloads must be owned and serializable"));
            }
            match r.elem.as_ref() {
                syn::Type::Path(p) if p.path.is_ident("str") => Ok(quote! { String }),
                syn::Type::Slice(s) => { let inner = &s.elem; Ok(quote! { Vec<#inner> }) }
                other => Ok(quote! { #other }),
            }
        }
        other => Ok(quote! { #other }),
    }
}
```

**Derivation target shape — the code this module must emit** (canonical form):
```rust
// For: #[offload] async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;
// on trait Reports, with #[service(impl = ReportBuilder)]

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ReportsBuildMonthlyJob {
    pub tenant_id: i64,
    pub month: Month,
}

#[::ferro::async_trait]
impl ::ferro::queue::Job for ReportsBuildMonthlyJob {
    fn name(&self) -> &'static str {
        "ReportsBuildMonthlyJob"   // stable string literal, not type_name (Pitfall 2)
    }

    async fn handle(&self) -> ::std::result::Result<(), ::ferro::queue::Error> {
        let svc = ::ferro::App::make::<dyn Reports>()
            .expect(
                "Reports is not registered in the App container. \
                 Did you annotate the struct with #[service(impl = …)]?"
            );
        // async fn variant:
        let _ = svc.build_monthly(self.tenant_id.clone(), self.month.clone()).await;
        Ok(())
        // Result<T,E> variant replaces the two lines above with:
        // svc.build_monthly(self.tenant_id.clone(), self.month.clone()).await
        //     .map(|_| ())
        //     .map_err(|e| ::ferro::queue::Error::job_failed(
        //         "ReportsBuildMonthlyJob", format!("{e}")))
    }
}

::ferro::inventory::submit! {
    ::ferro::queue::JobRegistrarEntry {
        register: |w: &mut ::ferro::queue::WorkerLoop| {
            w.register::<ReportsBuildMonthlyJob>();
        },
        name: "ReportsBuildMonthlyJob",
    }
}
```

**Key invariants for the planner:**
- `#[derive(Clone)]` is mandatory on the struct — `handle()` calls `.clone()` on all fields.
- `fn name()` must return a string literal, not `std::any::type_name::<Self>()`, to be
  path-stable (Pitfall 2 in RESEARCH.md).
- The `impl Job` block must be wrapped with `#[::ferro::async_trait]` (Pitfall 3).
- The `inventory::submit!` path is `::ferro::inventory::submit!` — requires `inventory`
  added to `ferro-queue/Cargo.toml` as a direct dep (Pitfall 4).

---

### `ferro-macros/src/lib.rs` — no structural change required

**Role:** Proc-macro crate entry point. The `service` entry point (`#[proc_macro_attribute]`)
is already registered at line ~107. No new `#[proc_macro_attribute]` is needed because
`#[offload]` is an inert helper attribute consumed by `#[service]` (RQ-1).

**Analog:** Existing `lib.rs` — the only delta is a `mod offload;` declaration to expose the
new helper module to `service.rs`.

**Existing entry-point pattern** (`ferro-macros/src/lib.rs`) — follow the same `use crate::...`
import style used by all other submodule calls in `lib.rs`.

---

### `ferro-queue/src/db.rs` — add `JobRegistrarEntry` type and `inventory::collect!`

**Role:** Holds the `Queue` global and the runtime `JOB_REGISTRARS` static (lines 24–87). Phase
244 adds the inventory-based registration type and collection call alongside the existing runtime
Vec.

**Analog:** `framework/src/container/provider.rs` lines 44–66 — `ServiceBindingEntry`,
`SingletonEntry`, and their `inventory::collect!` calls.

**`ServiceBindingEntry` + `inventory::collect!` pattern** (`framework/src/container/provider.rs:44–66`):
```rust
/// Entry for inventory-collected service bindings (trait → impl)
pub struct ServiceBindingEntry {
    pub register: fn(),
    pub name: &'static str,
}

pub struct SingletonEntry {
    pub register: fn(),
    pub name: &'static str,
}

inventory::collect!(ServiceBindingEntry);
inventory::collect!(SingletonEntry);
```

**Delta for `ferro-queue/src/db.rs`** — insert after line 88 (after `Queue::apply_registrars`):
```rust
/// Entry for inventory-collected job registrations.
///
/// Emitted by `#[offload]` methods via `inventory::submit!`; collected by
/// `WorkerLoop::from_registry` alongside the runtime `JOB_REGISTRARS` Vec.
pub struct JobRegistrarEntry {
    /// Registers the derived Job type with the given WorkerLoop.
    pub register: fn(&mut crate::WorkerLoop),
    /// Job struct ident string for diagnostics.
    pub name: &'static str,
}

inventory::collect!(JobRegistrarEntry);
```

**Note:** `inventory` must be added to `ferro-queue/Cargo.toml` as a direct dependency first
(see Cargo.toml section below).

---

### `ferro-queue/src/worker.rs` — extend `from_registry` to drain inventory

**Role:** `WorkerLoop::from_registry` at line 206 currently creates a `WorkerLoop` and calls
`Queue::apply_registrars` (which drains the runtime `JOB_REGISTRARS` Vec). Phase 244 adds a
second pass that drains the `inventory::iter::<JobRegistrarEntry>` collection.

**Existing `from_registry` pattern** (`ferro-queue/src/worker.rs:206–209`):
```rust
pub fn from_registry(config: WorkerConfig) -> Self {
    let mut w = Self::new(config);
    crate::db::Queue::apply_registrars(&mut w);
    w
}
```

**Delta** — replace lines 206–209 with:
```rust
pub fn from_registry(config: WorkerConfig) -> Self {
    let mut w = Self::new(config);
    crate::db::Queue::apply_registrars(&mut w);   // runtime Vec path (unchanged)
    // Inventory path: drain all JobRegistrarEntry items submitted by #[offload]
    for entry in inventory::iter::<crate::db::JobRegistrarEntry> {
        (entry.register)(&mut w);
    }
    w
}
```

**`WorkerLoop::register` signature** (`ferro-queue/src/worker.rs:175–199`) — the register closure
inside each `JobRegistrarEntry` calls `w.register::<J>()`, which stores a `JobHandler` keyed by
`type_name`. For derived Jobs that override `fn name()`, the key stored is still `type_name`
(handler dispatch uses `job_row.job_type` from the DB, not `job.name()`). Cross-check: at
`worker.rs:179`, `let type_name = std::any::type_name::<J>().to_string()` is the key. The
derived Job should therefore also override `fn name()` to return the same string that
`type_name::<ReportsBuildMonthlyJob>()` produces in the app crate — or, more safely, accept
the type_name default and document the module-rename caveat (RESEARCH.md Pitfall 2).

---

### `ferro-queue/Cargo.toml` — add `inventory` dependency

**Role:** `ferro-queue` does not currently depend on `inventory`. The `JobRegistrarEntry` type and
`inventory::collect!` call both live in `ferro-queue/src/db.rs`, so `inventory` must be a direct
dep of `ferro-queue`.

**Analog:** `framework/Cargo.toml` — look for the `inventory` dep line there (the framework
already re-exports it at `framework/src/lib.rs:311`). The version to match is `"0.3"`.

**Delta** — add to `[dependencies]` in `ferro-queue/Cargo.toml`:
```toml
inventory = "0.3"
```

No feature flags required. `ferro-macros/Cargo.toml` requires no changes (the macro generates
`::ferro::inventory::submit!` token paths; it does not link `inventory` itself at
expansion time).

---

### `ferro-macros/tests/offload_macro.rs` — trybuild harness (NEW)

**Role:** Trybuild test entry point for `#[offload]` compilation fixtures. Declares pass and fail
glob patterns; trybuild executes each fixture in isolation as a separate `cargo build`.

**Analog:** `ferro-macros/tests/action_macro.rs` (exact structural match).

**Pattern to copy** (`ferro-macros/tests/action_macro.rs:1–15`):
```rust
//! Trybuild UI tests for the `#[offload]` proc-macro.
//!
//! - `tests/ui/offload/pass/*.rs` — fixtures that MUST compile cleanly.
//! - `tests/ui/offload/fail/*.rs` + `*.stderr` — fixtures that MUST emit the
//!   exact compile error captured in the matching `.stderr` snapshot.
//!
//! Update `.stderr` snapshots after intentional message changes:
//!     TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro

#[test]
fn offload_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/offload/pass/*.rs");
    t.compile_fail("tests/ui/offload/fail/*.rs");
}
```

**Fixture file structure to create:**

`tests/ui/offload/pass/basic.rs` — minimal valid `#[offload]` on an async method:
```rust
//! Compile-pass: minimal #[offload] on an async trait method.
#![allow(unused_imports)]
extern crate ferro_rs as ferro;
use ferro::service;
use ferro::async_trait;

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Month(pub u32);

pub struct ReportBuilder;
impl Default for ReportBuilder { fn default() -> Self { Self } }

#[service(impl = ReportBuilder)]
#[async_trait]
pub trait Reports {
    #[offload]
    async fn build_monthly(&self, month: Month);
}

#[async_trait]
impl Reports for ReportBuilder {
    async fn build_monthly(&self, _month: Month) {}
}

fn main() {}
```

Analog: `ferro-macros/tests/ui/action/pass/minimal.rs` (same `extern crate ferro_rs as ferro`
preamble, same `fn main() {}` trailer).

`tests/ui/offload/fail/mut_ref_param.rs` + `mut_ref_param.stderr` — `&mut T` compile error:
```rust
//! Compile-fail: #[offload] on a method with &mut parameter.
#![allow(unused_imports)]
extern crate ferro_rs as ferro;
use ferro::service;
use ferro::async_trait;

pub struct Svc;
impl Default for Svc { fn default() -> Self { Self } }

#[service(impl = Svc)]
#[async_trait]
pub trait MyService {
    #[offload]
    async fn mutate(&self, data: &mut String);
}

fn main() {}
```

Analog: `ferro-macros/tests/ui/action/fail/missing_redirect_to.rs` + `.stderr` (same file
structure, `.stderr` contains the exact `error:` line produced by `syn::Error::new_spanned`).

---

### `ferro-queue/tests/offload_round_trip.rs` — integration round-trip test (NEW)

**Role:** Verifies that a derived (or hand-written-in-test) Job can be dispatched in sync mode
and that `handle()` runs. Does not require a database — `QUEUE_CONNECTION=sync` short-circuits
through `dispatch_immediately()` in `PendingDispatch::dispatch()`.

**Analog:** `ferro-queue/src/dispatcher.rs` lines 85–127 (the `dispatch_immediately` path) +
`ferro-queue/src/db.rs` tests setup pattern (serial_test, AtomicBool side-effect assertion).

**`dispatch` free function signature** (`ferro-queue/src/dispatcher.rs` — confirmed at lib.rs:62`):
```rust
pub async fn dispatch<J: Job + Serialize + DeserializeOwned>(job: J) -> Result<(), Error>
```
Called as `ferro::queue::dispatch(MyJob { … }).await.unwrap()`.

**Sync-mode dispatch test pattern** — copy structure from dispatcher test internals:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use ferro::queue::{dispatch, Job, Error};
use ferro::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestServiceDoWorkJob { value: i32 }

static JOB_RAN: AtomicBool = AtomicBool::new(false);

#[async_trait]
impl Job for TestServiceDoWorkJob {
    fn name(&self) -> &'static str { "TestServiceDoWorkJob" }
    async fn handle(&self) -> Result<(), Error> {
        JOB_RAN.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
#[serial_test::serial]
async fn offload_round_trip_sync_mode() {
    std::env::set_var("QUEUE_CONNECTION", "sync");
    JOB_RAN.store(false, Ordering::SeqCst);
    dispatch(TestServiceDoWorkJob { value: 42 }).await.unwrap();
    assert!(JOB_RAN.load(Ordering::SeqCst), "handle() must have run");
}
```

**`serial_test::serial`** is already in `ferro-queue/Cargo.toml` dev-dependencies (line 29).
`tokio = { version = "1", features = ["full", "test-util"] }` is also present (line 28).

---

## Shared Patterns

### `inventory::submit!` path convention
**Source:** `ferro-macros/src/service.rs:168`
**Apply to:** All `quote!` blocks in `ferro-macros/src/offload.rs` that emit registration code.
```rust
// Reference inventory through the ferro re-export, matching the #[service] convention.
::ferro::inventory::submit! { … }    // for JobRegistrarEntry
```

### `App::make` container resolution
**Source:** `ferro-macros/src/service.rs:170–172` (`App::bind` pattern) + `framework/src/app.rs:431`
**Apply to:** The `handle()` body emitted by `ferro-macros/src/offload.rs`.
```rust
let svc = ::ferro::App::make::<dyn TraitName>()
    .expect("TraitName is not registered …");
```

### `#[async_trait]` on derived `impl Job` blocks
**Source:** `ferro-queue/src/lib.rs:71` (`pub use async_trait::async_trait`)
            + `ferro-queue/src/job.rs:43–44`
**Apply to:** Every `impl ::ferro::queue::Job for <Derived>Job` block emitted by the macro.
```rust
#[::ferro::async_trait]
impl ::ferro::queue::Job for ReportsBuildMonthlyJob { … }
```

### `Error::job_failed` for `Result<T, E>` mapping
**Source:** `ferro-queue/src/error.rs:39–45, 82–85`
**Apply to:** The `Result<T, E>` branch of the derived `handle()` body.
```rust
// Error::JobFailed { job: String, message: String }
// Constructor helper at error.rs:82:
// pub fn job_failed(job: impl Into<String>, message: impl Into<String>) -> Self
svc.method(…).await
    .map(|_| ())
    .map_err(|e| ::ferro::queue::Error::job_failed("TraitMethodJob", format!("{e}")))
```

### `#[derive(Debug, Clone, Serialize, Deserialize)]` on derived structs
**Source:** `ferro-queue/src/job.rs:21–22` (`ProcessImage` example in doc-comment)
            + `ferro-queue/src/job.rs:191–192` (`TestJob` in module tests)
**Apply to:** Every derived `<Trait><Method>Job` struct. `Clone` is mandatory — `handle()`
calls `.clone()` on all fields. `Debug` matches the framework idiom for all Job types.

---

## No Analog Found

No files in scope lack a reasonable analog. All patterns have direct precedents in the codebase.

---

## Metadata

**Analog search scope:** `ferro-macros/src/`, `ferro-queue/src/`, `framework/src/container/`,
`ferro-macros/tests/`, `ferro-queue/Cargo.toml`
**Files scanned:** 11 source files read in full
**Pattern extraction date:** 2026-08-13
