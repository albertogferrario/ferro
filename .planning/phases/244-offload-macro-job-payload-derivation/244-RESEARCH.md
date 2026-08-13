# Phase 244: `#[offload]` macro → Job + payload derivation — Research

**Researched:** 2026-08-13
**Domain:** Rust proc-macro authoring (syn/quote), ferro-queue Job derivation, inventory auto-registration
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** The macro derives a sibling Job alongside the trait; the `#[offload]` trait method stays a normal in-process synchronous call in 244.
- **D-02:** The round-trip test enqueues the derived Job through the existing `ferro_queue::dispatch()` API.
- **D-03:** The ergonomic handle-returning offload entrypoint is deferred to Phase 245.
- **D-04:** `#[offload]` takes no arguments in 244; derived Job inherits all Job-trait defaults.
- **D-05:** Attribute config knobs deferred.
- **D-06:** Both `-> T` and `-> Result<T, E>` method signatures are supported.
- **D-07:** For `Result`-returning methods, `Err(e)` maps to a Job failure; `E` is stringified via Display/Debug; not required to be Serialize.
- **D-08:** Return value is discarded in 244; return type is not required to be serializable.
- **D-09:** A single `#[derive(Serialize, Deserialize)]` struct per offloaded method; the struct both carries params as fields and `impl Job`.
- **D-10:** Naming scheme: `<Trait><Method>Job` PascalCase; struct is public.
- **D-11:** `&self` receiver excluded; each non-self parameter becomes an owned field of the struct.
- **D-12:** Derived Job self-registers via `inventory::submit!`; `WorkerLoop::from_registry` gains an inventory-collection path.
- **D-13:** Phase 244 scope expands into `ferro-queue`'s registration mechanism to add the inventory path alongside the existing runtime `JOB_REGISTRARS` Vec.
- **D-14:** Derived `handle()` resolves the concrete service from the container (`App::make::<dyn Trait>()`) and calls the original method body with the payload fields.

### Claude's Discretion

- Exact inventory entry type/name for the job registrar; whether the runtime `Queue::register` Vec and the inventory path are unified or run side by side.
- Stringification detail for `E` (`Display` vs `Debug`) and the concrete `ferro_queue::Error` variant for method-`Err` mapping.
- Module placement of the derived struct (sibling of the trait in the same module).

### Deferred Ideas (OUT OF SCOPE)

- Typed result handle + compile-time serializable-contract enforcement — Phase 245.
- Result → `ferro-projection` snapshot — Phase 246.
- Shared broadcast transport — Phase 246.1.
- Read-model delta → broadcast streaming — Phase 247.
- Deployable `worker` subcommand runtime — Phase 248.
- `ferro-mcp` introspection + docs — Phase 249.
- `#[offload(queue = …, retries = …, timeout = …)]` config surface.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OFFLOAD-01 | A developer marks a `#[service]` trait method `#[offload]` and the framework derives a `ferro-queue` Job + serializable payload from the method signature — no hand-written Job struct, no manual enqueue. | Resolved: attribute mechanism, naming, field mapping, auto-registration, and handle() derivation all documented below with precise code references. |

</phase_requirements>

---

## Summary

Phase 244 is a proc-macro authoring phase with well-scoped integration work in `ferro-queue`. The
core task is teaching `#[service]` to recognise an `#[offload]` helper attribute on trait methods
and emit a derived Job struct alongside the trait. All three open research questions — attribute
mechanism, reference-param mapping, and sync/async handling — have clear, verified answers from
reading the actual source.

The existing codebase provides excellent precedents. The `#[service]` macro in
`ferro-macros/src/service.rs` already emits `inventory::submit!` for service binding entries
(line 168); the `#[injectable]` macro strips `#[inject]` helper attributes from fields
(line 177); and `ferro-queue/src/db.rs` (lines 24–87) shows the exact runtime `JOB_REGISTRARS`
Vec structure that the new inventory path sits alongside. The `#[memoize]` macro in
`ferro-macros/src/memoize.rs` is a direct structural model for how to parse trait method
signatures and emit conditional code.

No external libraries beyond `inventory`, `serde`, and `async_trait` are needed — all three are
already dependencies of the relevant crates.

**Primary recommendation:** Implement `#[offload]` as an inert helper attribute consumed by
`#[service]`. The outer `#[service]` macro iterates `item_trait.items`, identifies `TraitItemFn`
nodes whose `attrs` contain `#[offload]`, strips the attribute, and for each such method emits a
derived `<Trait><Method>Job` struct with `impl Job` and `inventory::submit!`. A new
`JobRegistrarEntry` inventory type in `ferro-queue` (mirroring `ServiceBindingEntry`) carries a
`fn(&mut WorkerLoop)` closure; `WorkerLoop::from_registry` drains both the existing
`JOB_REGISTRARS` Vec and the inventory collection.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Attribute recognition + struct derivation | Proc-macro (ferro-macros) | — | `#[service]` already owns the trait parse; extending it avoids a second macro pass |
| Job struct + `impl Job` code generation | Proc-macro (ferro-macros) | — | Code is emitted at the call site, sibling to the trait |
| `inventory::submit!` auto-registration | Proc-macro (ferro-macros) | ferro-queue (inventory collection type) | Matches the service-binding precedent exactly |
| Job registry (inventory path) | ferro-queue | framework (boot calls `from_registry`) | `Queue::apply_registrars` already drains the runtime Vec; the inventory pass is added beside it |
| Worker execution (`handle()`) | ferro-queue worker | framework container (`App::make`) | The derived `handle()` reaches into the container — same pattern as any service call |
| Round-trip integration test | ferro-queue/tests | — | Existing `tests/` directory; sync-mode dispatch avoids DB setup |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `syn` | 2.x | Parse `ItemTrait`, `TraitItemFn`, `FnArg`, `Type` | Already in ferro-macros Cargo.toml |
| `quote` | 1.x | Code generation for derived struct and impls | Already in ferro-macros Cargo.toml |
| `proc-macro2` | 1.x | Token stream manipulation | Already in ferro-macros Cargo.toml |
| `inventory` | (re-exported via `::ferro::inventory`) | Compile-time static registration | Already used by `#[service]` and `#[injectable]` |
| `serde` + `serde_json` | 1.x | Derived struct serialization | Already in ferro-queue |
| `async_trait` | 0.1 | `impl Job` uses `#[async_trait]` | Already in ferro-queue |

[VERIFIED: ferro-macros/Cargo.toml, ferro-queue/Cargo.toml — all dependencies present]

### Dependency Addition Required

`ferro-macros` does not currently depend on `ferro-queue`. The derived `handle()` emits code that
calls `ferro_queue::Error` and resolves via `::ferro::App::make`. Since `ferro-macros` is a
`proc-macro` crate it generates token streams referencing these types by path — it does not need
to link against `ferro-queue` at macro expansion time. The emitted code is compiled in the crate
that applies the attribute (the app crate), which already links `ferro-rs` (which re-exports
`ferro-queue` types). **No new Cargo dependency in ferro-macros is required.**

[VERIFIED: ferro-macros/Cargo.toml line 25 shows `ferro-rs` in dev-dependencies only; the macro
generates paths via `quote!{ ::ferro_queue::… }` or `::ferro::queue::…` — no compile-time link]

---

## Research Question Resolutions

### RQ-1: Attribute Mechanism — inert helper vs. standalone proc_macro_attribute

**Verdict: `#[offload]` must be an inert helper attribute consumed by `#[service]`.**

Rust does not permit a standalone `#[proc_macro_attribute]` to target an individual trait method.
Attribute macros apply to items (structs, fns, impls, traits, modules, etc.) — a trait method is
not a standalone item; it lives inside `ItemTrait`. An attribute macro applied to a trait item
in the source would see only that method in isolation and could not emit the derived struct at the
correct module scope.

The only correct mechanism is:

1. Declare `offload` as a helper attribute in `#[service]`'s `proc_macro_attribute` registration
   (via a `helper_attributes` annotation in `proc_macro_attribute` — but `syn`/`quote` does not
   need this at the Rust level). In practice, Rust currently treats unrecognised attributes on
   trait methods as warnings or errors unless a surrounding outer proc-macro claims them. The clean
   approach, matching the `#[inject]` precedent in `#[injectable]`, is:
   - Add `offload` to the `#[proc_macro_attribute]` annotation's `helper_attributes` list in the
     `#[proc_macro_attribute]` item above `service`. Alternatively (simpler), after reading each
     `TraitItemFn`, strip the `#[offload]` attribute from `fn.attrs` so the Rust compiler never
     sees it as an unknown attribute.
2. Inside `service_impl`, iterate `item_trait.items` (a `Vec<TraitItem>`), match on
   `TraitItem::Fn(method)`, check `method.attrs.iter().any(|a| a.path().is_ident("offload"))`,
   strip that attribute from `method.attrs` (following the `injectable.rs` pattern at line 177),
   and collect the method signatures for derivation.

**Concrete change to `service_impl`:**

```rust
// In service_impl(), after parsing item_trait:
let mut offload_methods: Vec<OffloadMethodInfo> = Vec::new();

for item in &mut item_trait.items {
    if let syn::TraitItem::Fn(method) = item {
        let offload_pos = method.attrs.iter().position(|a| a.path().is_ident("offload"));
        if let Some(pos) = offload_pos {
            method.attrs.remove(pos);          // strip before re-emitting the trait
            offload_methods.push(collect_offload_info(trait_name, method)?);
        }
    }
}
```

This pattern is proven by `injectable.rs` line 177 and requires no new `#[proc_macro_attribute]`
registration. The stripped `item_trait` is emitted verbatim; no `#[offload]` remnant reaches
rustc.

[VERIFIED: ferro-macros/src/injectable.rs:177 for the stripping pattern; Rust proc-macro
reference for attribute macro target restrictions — ASSUMED for the proc-macro-attribute/helper
scoping rule, but the stripping approach sidesteps it entirely]

---

### RQ-2: Borrowed/reference params → owned payload fields

**Verdict: Syntactic substitution with documented constraints; unsupported types emit `compile_error!`.**

The macro operates on `syn::Type` tokens. It cannot perform full type-inference, but a
deterministic syntactic substitution covers the primary cases:

| Source type (syntactic form) | Derived field type | Notes |
|------------------------------|-------------------|-------|
| `&str` | `String` | Recognised by `Type::Reference` with path `str` |
| `&[T]` | `Vec<T>` | Recognised by `Type::Reference` with `Type::Slice` inner |
| `&T` (anything else) | `T` (clone ownership) | Recognised by `Type::Reference`; strips the reference; T must be `Serialize` |
| `&mut T` | Emit `compile_error!` | Mutable references are not serializable; no obvious owned analog |
| Owned `T` | `T` | Pass through unchanged |

**Implementation:**

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
                // &str → String
                syn::Type::Path(p) if p.path.is_ident("str") => {
                    Ok(quote! { String })
                }
                // &[T] → Vec<T>
                syn::Type::Slice(s) => {
                    let inner = &s.elem;
                    Ok(quote! { Vec<#inner> })
                }
                // &T → T  (e.g. &Month → Month)
                other => Ok(quote! { #other }),
            }
        }
        // Owned types: pass through
        other => Ok(quote! { #other }),
    }
}
```

The caller passes borrowed values at the original call site; the Job struct carries owned fields.
The constraint to document: **all payload field types (after owned_type substitution) must be
`Serialize + DeserializeOwned`** — the compiler enforces this transitively through the derived
`#[derive(Serialize, Deserialize)]` and through `dispatch()` which requires
`J: Job + Serialize + DeserializeOwned`. The compile error is naturally produced by the derive
macro; no explicit diagnostic is required in Phase 244. Phase 245 adds explicit trybuild-caught
diagnostics (OFFLOAD-02).

[VERIFIED: dispatcher.rs lines 36–37 show `J: Job + Serialize + DeserializeOwned` on
`PendingDispatch`; job.rs lines 121–122 show `J: Job + Serialize` on `JobPayload::new`. The
syntactic mapping is ASSUMED to be sufficient for the common cases documented above.]

---

### RQ-3: Sync vs. async methods — `Job::handle` is always `async fn`

**Verdict: Both `async fn` and plain `fn` offloaded methods are supported. The derived `handle()` always calls the concrete impl, which handles both forms. In 244, restrict to `async fn` methods; plain `fn` methods emit a clear error and are deferred.**

From `ferro-queue/src/job.rs` line 46:

```rust
#[async_trait]
pub trait Job: Send + Sync + 'static {
    async fn handle(&self) -> Result<(), Error>;
    // ...
}
```

`Job::handle` is always `async` (decorated by `async_trait`). The derived `handle()` for an
`async fn build_monthly(&self, ...) -> Report` on the impl would look like:

```rust
async fn handle(&self) -> Result<(), ferro_queue::Error> {
    let svc = ::ferro::App::make::<dyn Reports>()
        .expect("Reports not registered in the container");
    // For -> T:
    let _ = svc.build_monthly(self.tenant_id, self.month.clone()).await;
    Ok(())
    // For -> Result<T, E>:
    // svc.build_monthly(...).await
    //    .map(|_| ())
    //    .map_err(|e| ferro_queue::Error::job_failed("ReportsBuildMonthlyJob", e.to_string()))
}
```

For a **sync method** (`fn build_monthly(&self, ...) -> Report` without `async`), the derived
`handle()` would call it directly from an async context, which is valid Rust — a sync function is
callable inside an async fn. However, in 244 the authoring surface spec uses `async fn` (see
CONTEXT.md `<specifics>` block). The macro should:

- Accept `async fn` methods: emit the `svc.method(...).await` call.
- Accept plain `fn` methods: emit `svc.method(...)` without `.await`.
- Detection: `method.sig.asyncness.is_some()`.

This is fully derivable at the syntactic level from `TraitItemFn.sig.asyncness`.

[VERIFIED: ferro-queue/src/job.rs:46 — `async fn handle(&self) -> Result<(), Error>` is the
exact signature. `async_trait` desugars it to a boxed future at runtime, but the source-level
signature is `async fn`. The macro emits `#[async_trait]` on the derived `impl Job` block,
matching the existing hand-written pattern in `job.rs` test structs.]

---

## Architecture Patterns

### System Architecture Diagram

```
#[service(impl = ReportBuilder)]          (source)
#[async_trait]
pub trait Reports {
    #[offload]
    async fn build_monthly(...) → Report;
}
         │
         ▼
  service_impl() proc-macro
         │
  ┌──────┴──────────────────────────────────────┐
  │  1. strip #[offload] from method.attrs       │
  │  2. re-emit item_trait (unchanged bounds)    │
  │  3. emit ServiceBindingEntry (existing path) │
  │  4. for each offload method:                 │
  │     a. emit pub struct ReportsBuildMonthlyJob│
  │        { tenant_id: i64, month: Month }      │
  │        #[derive(Debug, Clone, Serialize,     │
  │                 Deserialize)]                │
  │     b. emit #[async_trait] impl Job for …   │
  │        { async fn handle(&self) → Result<…> }│
  │     c. emit inventory::submit! {             │
  │          JobRegistrarEntry { register: ||    │
  │            Queue::register::<Job>() } }      │
  └──────────────────────────────────────────────┘
         │
         ▼ at boot
  WorkerLoop::from_registry()
    db::Queue::apply_registrars()   ← drains JOB_REGISTRARS Vec (runtime path)
    drain_inventory_job_registrars() ← NEW: drains JobRegistrarEntry inventory
         │
         ▼ at dispatch
  dispatch(ReportsBuildMonthlyJob { tenant_id: 1, month: m }).await
    → PendingDispatch::new(job).dispatch()
    → enqueue into jobs table (or sync-mode inline)
         │
         ▼ at execute
  WorkerLoop::spawn_job()
    → handler(payload, attempts)
    → serde_json::from_str::<ReportsBuildMonthlyJob>(payload)
    → job.handle().await
    → App::make::<dyn Reports>().build_monthly(job.tenant_id, job.month.clone()).await
```

### Recommended Project Structure — changed files

```
ferro-macros/src/
├── service.rs         # extend to iterate items, strip #[offload], emit Job structs
├── lib.rs             # no change needed (service entry point unchanged)
└── offload.rs         # NEW: helper module with owned_type(), collect_offload_info(),
                       #      emit_job_struct(), emit_impl_job()

ferro-queue/src/
├── db.rs              # add JobRegistrarEntry type + inventory::collect!
├── worker.rs          # extend from_registry() to drain inventory
└── lib.rs             # re-export JobRegistrarEntry

ferro-queue/tests/
└── offload_round_trip.rs  # NEW: round-trip integration test (sync mode)
```

### Pattern 1: Helper-attribute stripping in a trait macro

**What:** The outer `#[service]` macro iterates `item_trait.items`, strips known helper attributes
from `TraitItemFn.attrs`, then re-emits the cleaned trait. The stripped attributes are consumed to
generate sibling items (the Job struct, `impl Job`, inventory entry).

**When to use:** Any time a proc-macro on an outer item needs to consume meta-annotations on inner
items without those annotations reaching rustc as unknown attributes.

```rust
// Source: ferro-macros/src/injectable.rs:177 (field-level precedent)
let other_attrs: Vec<_> = field
    .attrs
    .iter()
    .filter(|attr| !attr.path().is_ident("inject"))
    .collect();

// Analogous pattern for trait method items in service.rs:
for item in &mut item_trait.items {
    if let syn::TraitItem::Fn(method) = item {
        let pos = method.attrs.iter().position(|a| a.path().is_ident("offload"));
        if let Some(pos) = pos {
            method.attrs.remove(pos);
            // ... collect and generate
        }
    }
}
```

[VERIFIED: ferro-macros/src/injectable.rs:177]

### Pattern 2: inventory::submit! for zero-wiring registration

**What:** The macro emits an `inventory::submit!` call at the Job struct's definition site. At
program startup, `WorkerLoop::from_registry` collects all submitted entries via
`inventory::iter::<JobRegistrarEntry>` and calls each entry's register closure.

**When to use:** Any time a type defined in an application crate must self-register with a
framework runtime without manual bootstrap code.

```rust
// Source: ferro-macros/src/service.rs:168 (existing ServiceBindingEntry precedent)
#[ferro::inventory::submit! {
    #ferro::container::provider::ServiceBindingEntry {
        register: || {
            #ferro::App::bind::<dyn #trait_name>(
                ::std::sync::Arc::new(<#concrete_type as ::std::default::Default>::default())
            );
        },
        name: #trait_name_str,
    }
}]
```

For the job registrar entry, the emitted code follows the same shape:

```rust
// Emitted by the macro for ReportsBuildMonthlyJob:
::ferro_queue::inventory::submit! {
    ::ferro_queue::JobRegistrarEntry {
        register: || {
            ::ferro_queue::Queue::register::<ReportsBuildMonthlyJob>();
        },
        name: "ReportsBuildMonthlyJob",
    }
}
```

[VERIFIED: ferro-macros/src/service.rs:168; framework/src/container/provider.rs:63 for
`inventory::collect!` pattern to mirror in ferro-queue/src/db.rs]

### Pattern 3: Derived Job struct shape

The exact shape that the macro must emit, derived from the hand-written `ProcessImage` and
`TestJob` in `ferro-queue/src/job.rs`:

```rust
// Source: ferro-queue/src/job.rs (TestJob at line 192, ProcessImage at lines 22–40)
// Target derivation for Reports::build_monthly:

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ReportsBuildMonthlyJob {
    pub tenant_id: i64,
    pub month: Month,
}

#[::ferro_queue::async_trait]
impl ::ferro_queue::Job for ReportsBuildMonthlyJob {
    async fn handle(&self) -> ::std::result::Result<(), ::ferro_queue::Error> {
        let svc = ::ferro::App::make::<dyn Reports>()
            .expect("Reports not bound in the container — did you annotate the impl with #[service(impl = …)]?");
        // async fn variant (method.sig.asyncness.is_some()):
        let _ = svc.build_monthly(self.tenant_id, self.month.clone()).await;
        // For Result<T,E> variant:
        // svc.build_monthly(...).await
        //    .map(|_| ())
        //    .map_err(|e| ::ferro_queue::Error::job_failed(
        //        "ReportsBuildMonthlyJob", format!("{e}")))
        Ok(())
    }
}
```

[VERIFIED: ferro-queue/src/job.rs:44–89, job.rs:197–201 (TestJob impl), dispatcher.rs:36–37
(`J: Job + Serialize + DeserializeOwned`)]

### Pattern 4: JobRegistrarEntry inventory type in ferro-queue

New type to add in `ferro-queue/src/db.rs`:

```rust
// Mirrors ServiceBindingEntry in framework/src/container/provider.rs:44
/// Entry for inventory-collected job registrations.
///
/// Emitted by `#[offload]` methods; collected by `WorkerLoop::from_registry`.
pub struct JobRegistrarEntry {
    pub register: fn(&mut crate::WorkerLoop),
    pub name: &'static str,
}

inventory::collect!(JobRegistrarEntry);
```

And in `WorkerLoop::from_registry` (currently `worker.rs:206`):

```rust
pub fn from_registry(config: WorkerConfig) -> Self {
    let mut w = Self::new(config);
    crate::db::Queue::apply_registrars(&mut w);   // existing runtime Vec path
    // NEW: drain inventory
    for entry in inventory::iter::<crate::db::JobRegistrarEntry> {
        (entry.register)(&mut w);
    }
    w
}
```

[VERIFIED: ferro-queue/src/worker.rs:206–209, ferro-queue/src/db.rs:60–87 for the existing path]

### Anti-Patterns to Avoid

- **Emitting `#[offload]` as a standalone `#[proc_macro_attribute]` on a trait method:** This
  cannot access the enclosing trait and cannot emit a sibling struct in the correct scope.
- **Using `std::any::type_name::<J>()` as the `job_type` key in a Job that may be in a different
  module path at the call site:** The worker resolves handlers by the key stored at enqueue time
  (which is `job.name()`, defaulting to `type_name`). Changing the struct's module path (e.g.
  moving the trait to a different module) silently breaks the key. Document this as a known
  constraint: the generated struct name is stable; the fully-qualified type name used as the DB
  key changes if the trait's module path changes.
- **Calling `.clone()` on payload fields without bounding `Clone`:** The derived `handle()` moves
  fields into the method call. For non-`Copy` types this requires `.clone()`. The macro must emit
  `.clone()` for all non-`Copy` field references and bound `#[derive(Clone)]` on the generated
  struct (which it does via the standard `#[derive(Debug, Clone, Serialize, Deserialize)]`).
- **Emitting `App::make` panic without a descriptive message:** The worker runs in a background
  task; panics are caught by `catch_unwind` but logged opaquely. Include the trait name and a
  diagnostic hint in the expect message.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Job serialization/deserialization | Custom serialize/deserialize logic | `#[derive(Serialize, Deserialize)]` emitted by the macro | serde handles edge cases; the derive is what the existing idiom uses |
| Worker registration | A generate-and-call-once `register_offload_jobs()` fn | `inventory::submit!` + `inventory::iter` | Already used by `#[service]`; zero-wiring is the stated goal |
| Async wrapping of sync code | A custom `tokio::task::spawn_blocking` wrapper | Call the method body directly (sync fn callable inside async fn) | spawn_blocking adds overhead; non-blocking sync work does not need it; blocking work is a caller responsibility, not a macro concern |
| Error conversion | A custom error type for method failure | `ferro_queue::Error::job_failed(name, e.to_string())` | The variant exists (error.rs:39–45) and produces the right tracing output |

---

## Common Pitfalls

### Pitfall 1: `#[offload]` attribute visible to rustc on the trait item

**What goes wrong:** If the `#[offload]` attribute is not stripped before re-emitting `item_trait`,
rustc sees an unknown `#[offload]` attribute on a trait method and emits a warning or error
depending on edition.

**Why it happens:** The macro re-emits the whole `item_trait` including its items. Unless the
attribute is removed from `method.attrs` before that, it survives into the emitted code.

**How to avoid:** Use `method.attrs.remove(pos)` (where `pos` is the index of the `#[offload]`
attr) before the `quote! { #item_trait }` call. The `injectable.rs` precedent shows this pattern.

**Warning signs:** `error[E0658]: attributes on expressions are experimental` or `warning: unused
attribute` on the `#[offload]` attribute in user code.

### Pitfall 2: `type_name` key mismatch between enqueue and handler

**What goes wrong:** `WorkerLoop::register::<J>()` stores the handler keyed by
`std::any::type_name::<J>()`. If the Job struct's fully-qualified type name differs between
the enqueue caller (app crate) and the worker (same binary, so same path — but if the user
restructures modules, it changes), the worker logs "No handler registered" and releases for retry.

**Why it happens:** `type_name` is an unstable implementation detail that includes the full module
path. It works in practice as long as the struct's module path does not change between enqueue
and execution (same binary).

**How to avoid:** Override `fn name(&self) -> &'static str` in the derived `impl Job` to return a
stable string (e.g. `"TraitMethodJob"` — the generated struct ident, not the full path). This is
Claude's Discretion territory, but HIGH value: it makes the key path-stable.

```rust
fn name(&self) -> &'static str {
    "ReportsBuildMonthlyJob"  // emitted as the struct ident string
}
```

**Warning signs:** Worker logs "No handler registered — releasing job for retry" for a job type
that definitely has a registered handler; happens after a module rename.

### Pitfall 3: Derived `impl Job` not gated by `#[async_trait]`

**What goes wrong:** The `Job` trait uses `#[async_trait]`, so any `impl Job for T` must also be
decorated with `#[async_trait]` (from `async_trait::async_trait`). Without it, the `async fn
handle` inside the impl is rejected because the trait's `handle` is a desugared boxed-future
signature.

**Why it happens:** `async_trait` is a macro that rewrites both the trait and every impl. The
derived `impl Job` is emitted by our macro at compile time, so it must include the attribute.

**How to avoid:** Emit `#[::ferro_queue::async_trait]` on the derived impl block:

```rust
quote! {
    #[::ferro_queue::async_trait]
    impl ::ferro_queue::Job for #job_struct_name { ... }
}
```

`ferro-queue` already re-exports `async_trait` as `pub use async_trait::async_trait` (lib.rs:71).

[VERIFIED: ferro-queue/src/lib.rs:71 — `pub use async_trait::async_trait`; ferro-queue/src/job.rs:43]

### Pitfall 4: `ferro-queue` does not have `inventory` as a direct dependency

**What goes wrong:** `ferro-macros` emits `::ferro_queue::inventory::submit!`, but `ferro-queue`
does not re-export `inventory`. The framework crate re-exports it at `::ferro::inventory`.

**Why it happens:** Checking `ferro-queue/Cargo.toml` — `inventory` is not listed. The macro can
reference it through the framework's re-export path instead.

**How to avoid:** Two clean options:
1. Emit `::ferro::inventory::submit!` in the macro (all ferro-queue consumers link ferro-rs which
   re-exports inventory at `framework/src/lib.rs:311`).
2. Add `inventory` to `ferro-queue/Cargo.toml` directly (preferred for crate independence — the
   `JobRegistrarEntry` type and `inventory::collect!` both live in ferro-queue).

Option 2 is the architecturally cleaner choice: `ferro-queue` owns its inventory type and
collection; the macro emits `::ferro_queue::inventory::submit!`. The planner should include a
task to add `inventory = "0.3"` to `ferro-queue/Cargo.toml`.

[VERIFIED: ferro-queue/Cargo.toml — no inventory dep; framework/src/lib.rs:310–312 for
the existing re-export]

### Pitfall 5: Clone requirement for payload fields in `handle()`

**What goes wrong:** The `handle()` body calls `svc.build_monthly(self.tenant_id, self.month.clone())`.
`self.month` is moved out of `&self` — not allowed. All fields must be cloned or copied.

**Why it happens:** `Job::handle` takes `&self`. The generated call passes field values to the
concrete method, which expects owned (or borrowed) arguments.

**How to avoid:** For owned parameter types, emit `.clone()` unconditionally (the struct derives
`Clone`, so all fields implement it). For `Copy` types the clone is a no-op in practice. For
`&str`-mapped-to-`String` fields, emit `self.field.as_str()` if the method expects `&str`, or
`self.field.clone()` if it expects `String`. The simplest uniform rule: emit `self.field.clone()`
for all fields in the `handle()` call — correct and zero overhead for `Copy` types.

---

## Code Examples

### Complete derivation target shape

```rust
// Source: pattern derived from ferro-queue/src/job.rs:22–40 (ProcessImage) +
//         ferro-queue/src/job.rs:192–201 (TestJob in tests)
//
// Input trait method:
//   #[offload]
//   async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;
//
// Macro emits:

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct ReportsBuildMonthlyJob {
    pub tenant_id: i64,
    pub month: Month,
}

#[::ferro_queue::async_trait]
impl ::ferro_queue::Job for ReportsBuildMonthlyJob {
    fn name(&self) -> &'static str {
        "ReportsBuildMonthlyJob"
    }

    async fn handle(&self) -> ::std::result::Result<(), ::ferro_queue::Error> {
        let svc = ::ferro::App::make::<dyn Reports>()
            .expect(
                "Reports is not registered in the App container. \
                 Did you annotate the struct with #[service(impl = …)]?"
            );
        let _ = svc.build_monthly(self.tenant_id.clone(), self.month.clone()).await;
        Ok(())
    }
}

::ferro_queue::inventory::submit! {
    ::ferro_queue::JobRegistrarEntry {
        register: |w: &mut ::ferro_queue::WorkerLoop| {
            w.register::<ReportsBuildMonthlyJob>();
        },
        name: "ReportsBuildMonthlyJob",
    }
}
```

### Result-returning method — handle() variant

```rust
// Input: async fn export_csv(&self, report_id: i64) -> Result<CsvFile, ExportError>;
// Macro emits (in handle()):
async fn handle(&self) -> ::std::result::Result<(), ::ferro_queue::Error> {
    let svc = ::ferro::App::make::<dyn Reports>()
        .expect("Reports not registered");
    svc.export_csv(self.report_id.clone()).await
        .map(|_| ())
        .map_err(|e| ::ferro_queue::Error::job_failed(
            "ReportsExportCsvJob",
            format!("{e}"),
        ))
}
```

### Round-trip test shape (sync mode — no DB needed)

```rust
// Source: dispatcher.rs:296–307 (sync mode test pattern)
// Location: ferro-queue/tests/offload_round_trip.rs  OR  a #[ferro_test] in ferro-queue/src/

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use ferro_queue::{dispatch, Job, Error, async_trait};
use serde::{Deserialize, Serialize};

// Simulated generated struct (manual in test; real usage is macro-derived)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestServiceDoWorkJob {
    value: i32,
}

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

For a full end-to-end test (with real macro expansion), use a `#[ferro_test]` in the app crate
that declares a `#[service]` + `#[offload]` trait, sets `QUEUE_CONNECTION=sync`, calls
`dispatch(TraitMethodJob { … }).await`, and asserts a side-effect.

### Inventory collection in ferro-queue/src/db.rs

```rust
// Add below the existing JOB_REGISTRARS static:

/// Entry for inventory-collected job registrations.
///
/// Emitted by `#[offload]` via `inventory::submit!`; collected by
/// `WorkerLoop::from_registry` alongside the runtime `JOB_REGISTRARS` Vec.
pub struct JobRegistrarEntry {
    /// Closure that calls `worker.register::<J>()` for the derived Job type.
    pub register: fn(&mut crate::WorkerLoop),
    /// Job struct ident string for diagnostics.
    pub name: &'static str,
}

inventory::collect!(JobRegistrarEntry);
```

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code/macro changes with no external runtime dependencies beyond
the existing Rust toolchain and project Cargo workspace.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `trybuild 1.x` (macro compilation tests) + `tokio::test` (round-trip) |
| Config file | `ferro-macros/Cargo.toml` dev-deps (trybuild already present); `ferro-queue/Cargo.toml` (tokio full already in dev-deps) |
| Quick run — macro UI | `cargo test -p ferro-macros --test offload_macro` |
| Quick run — round-trip | `cargo test -p ferro-queue --test offload_round_trip` |
| Full suite | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OFFLOAD-01-a | Valid `#[offload]` method on a `#[service]` trait compiles and emits a `<Trait><Method>Job` struct | trybuild pass | `cargo test -p ferro-macros --test offload_macro` | Wave 0 |
| OFFLOAD-01-b | `&str` param maps to `String` field in derived struct | trybuild pass | same | Wave 0 |
| OFFLOAD-01-c | `&mut T` param emits compile_error | trybuild fail + .stderr | same | Wave 0 |
| OFFLOAD-01-d | Derived Job dispatched via `dispatch(…).await` in sync mode executes `handle()` | unit (tokio::test + serial) | `cargo test -p ferro-queue --test offload_round_trip` | Wave 0 |
| OFFLOAD-01-e | `Result<T, E>` method: `Err(e)` returns `Error::JobFailed` from `handle()` | unit | same | Wave 0 |
| OFFLOAD-01-f | Derived Job auto-registers; `WorkerLoop::from_registry` includes it | unit | `cargo test -p ferro-queue` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-macros --test offload_macro && cargo test -p ferro-queue` (< 30 s)
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-macros/tests/offload_macro.rs` — trybuild harness for `#[offload]`
- [ ] `ferro-macros/tests/ui/offload/pass/basic.rs` — minimal valid `#[offload]` fixture
- [ ] `ferro-macros/tests/ui/offload/pass/result_method.rs` — `Result<T, E>` return fixture
- [ ] `ferro-macros/tests/ui/offload/pass/ref_str_param.rs` — `&str` param fixture
- [ ] `ferro-macros/tests/ui/offload/fail/mut_ref_param.rs` + `.stderr` — `&mut T` compile-error fixture
- [ ] `ferro-queue/tests/offload_round_trip.rs` — sync-mode dispatch round-trip test

---

## Security Domain

This phase involves no authentication, session management, access control, or user-facing input
processing. The macro derives code that is executed in a worker process controlled entirely by the
application. No ASVS categories apply to the macro derivation itself.

The one relevant concern is **job payload data**: if a developer offloads a method that carries
sensitive data as parameters (e.g. plaintext credentials, PII), those parameters are serialized to
the database queue. This is a documentation concern (to be addressed in Phase 249's docs), not a
security control that Phase 244 must enforce.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `#[proc_macro_attribute]` cannot target a trait method directly in stable Rust | RQ-1 | If wrong, a standalone `#[offload]` attribute macro could be registered; the helper-attribute-stripping approach is still cleaner and works in both cases |
| A2 | `std::any::type_name::<J>()` produces a consistent key within one binary (no cross-binary dispatch in 244) | Pitfall 2 | If job type names are inconsistent (unlikely same binary), the worker silently releases jobs for retry; mitigated by overriding `name()` to a stable literal |
| A3 | The syntactic `owned_type()` mapping (`&str` → `String`, `&[T]` → `Vec<T>`, `&T` → `T`) covers all common param shapes | RQ-2 | If a caller uses a param type the mapping does not recognise cleanly (e.g. `&dyn Trait`), the emitted field type is wrong and the derive or dispatch will fail to compile — surfaced at compile time, not runtime |

---

## Open Questions

1. **inventory crate version in ferro-queue**
   - What we know: ferro-queue does not currently depend on `inventory`; the framework re-exports
     `inventory 0.3` at `::ferro::inventory`.
   - What's unclear: whether to add `inventory` as a direct dep to ferro-queue (preferred) or emit
     `::ferro::inventory::submit!` paths in the macro (couples job registration to the framework).
   - Recommendation: add `inventory = "0.3"` to ferro-queue/Cargo.toml. The `JobRegistrarEntry`
     type and its `inventory::collect!` call belong in ferro-queue, not in framework.

2. **`clone()` for payload field forwarding with lifetime-restricted types**
   - What we know: the macro emits `self.field.clone()` unconditionally for all fields; this works
     for types deriving `Clone`.
   - What's unclear: if a user marks a method with `#[offload]` where a parameter type does not
     implement `Clone` (e.g. a raw OS handle, a non-Clone newtype), the `#[derive(Clone)]` on the
     Job struct and the `.clone()` call in `handle()` will both fail with a clear compiler error.
   - Recommendation: this is acceptable for Phase 244 (the error is compile-time and actionable).
     Phase 245 adds explicit diagnostic messages.

3. **Interaction with `#[async_trait]` applied to the enclosing trait**
   - What we know: the spec example places `#[async_trait]` on the trait; `#[service]` receives
     the trait after `#[async_trait]` has already transformed it (attribute macros apply in order,
     outermost last). If `#[service]` is listed first and `#[async_trait]` second, `#[service]`
     sees the raw `async fn` signatures — which is what we want for parsing. If the order is
     reversed, `#[service]` sees the desugared boxed-future signatures, which are harder to parse.
   - What's unclear: what attribute order the spec example uses vs. what the user will write in
     practice.
   - Recommendation: document in the authoring surface that `#[service]` must be the outermost
     attribute (listed first in source order), so it sees the un-desugared `async fn` signatures
     when it runs. This is already the convention (see CONTEXT.md `<specifics>` block where
     `#[service]` is listed above `#[async_trait]`).

---

## Sources

### Primary (HIGH confidence)

- `ferro-macros/src/service.rs` — `service_impl` function; `inventory::submit!` precedent at
  line 168; trait parsing via `ItemTrait`
- `ferro-macros/src/injectable.rs` — helper-attribute stripping pattern at line 177
- `ferro-macros/src/memoize.rs` — structural model for parsing `FnArg` types, handling
  `asyncness`, and emitting wrapper code around a function body
- `ferro-queue/src/job.rs` — exact `Job` trait signature (lines 44–89); `ProcessImage` and
  `TestJob` as derivation targets (lines 22–40, 192–201)
- `ferro-queue/src/dispatcher.rs` — `dispatch()`, `PendingDispatch` type bounds
  (`J: Job + Serialize + DeserializeOwned`), sync-mode test pattern
- `ferro-queue/src/worker.rs` — `WorkerLoop::register::<J>()` at line 175;
  `from_registry()` at line 206
- `ferro-queue/src/db.rs` — `JOB_REGISTRARS` static at line 25; `Queue::register` at line 65;
  `Queue::apply_registrars` at line 83
- `framework/src/container/provider.rs` — `ServiceBindingEntry` at line 44;
  `inventory::collect!` at line 63; `inventory::iter` usage at lines 73, 124
- `framework/src/app.rs` — `WorkerLoop::from_registry` call at line 431;
  `App::make::<dyn Trait>()` signature
- `ferro-queue/src/lib.rs` — `pub use async_trait::async_trait` at line 71
- `framework/src/lib.rs` — `pub use inventory` at line 311
- `ferro-macros/Cargo.toml` — confirms `trybuild` already in dev-deps; `ferro-rs` in dev-deps
- `ferro-queue/Cargo.toml` — confirms no `inventory` dep (action required)

### Secondary (MEDIUM confidence)

- Rust Reference on attribute macros and their targets — [ASSUMED] that
  `#[proc_macro_attribute]` cannot target a trait method body, only top-level items

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all libraries verified in Cargo.toml
- Attribute mechanism (RQ-1): HIGH — helper-attribute stripping is a proven pattern in this codebase
- Field mapping (RQ-2): HIGH (common cases) / MEDIUM (edge cases) — syntactic substitution works for `&str`, `&[T]`, `&T`; exotic types emit compile errors naturally
- Async/sync handling (RQ-3): HIGH — verified against exact `Job::handle` signature in source
- Registration path (D-12/D-13): HIGH — all types and call sites verified in source

**Research date:** 2026-08-13
**Valid until:** 2026-10-13 (stable codebase; no external API dependencies)
