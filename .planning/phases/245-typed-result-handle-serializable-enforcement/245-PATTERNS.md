# Phase 245: Typed Result Handle + Serializable-Contract Enforcement — Pattern Map

**Mapped:** 2026-08-13
**Files analyzed:** 8 (4 new, 4 modified)
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-queue/src/offload.rs` | type module (new types + trait) | request-response (enqueue) | `ferro-queue/src/job.rs` + `ferro-ai/src/classifier/mod.rs` | role-match (queue-layer type) + exact (PhantomData pattern) |
| `ferro-macros/src/offload.rs` | proc-macro emission site | transform (token stream) | itself (244 emit_job_items) | exact — extend in place |
| `ferro-macros/src/service.rs` | proc-macro wiring | transform (token stream) | itself (244 collect/emit wiring) | exact — extend in place |
| `ferro-queue/src/lib.rs` | re-export shim | — | itself (current pub use block + Queueable blanket) | exact — additive |
| `framework/src/lib.rs` | re-export shim | — | itself (mod queue block, lines 224–231) | exact — additive |
| `ferro-macros/tests/ui/offload/fail/non_serializable_param.rs` | trybuild fixture | — | `tests/ui/offload/fail/mut_ref_param.rs` | exact |
| `ferro-macros/tests/ui/offload/fail/non_serializable_param.stderr` | trybuild snapshot | — | `tests/ui/offload/fail/mut_ref_param.stderr` | exact |
| `ferro-macros/tests/ui/offload/fail/non_serializable_return.rs` + `.stderr` | trybuild fixture | — | `tests/ui/offload/fail/mut_ref_param.rs` | exact |
| `ferro-macros/tests/ui/offload/pass/<new>.rs` | trybuild fixture | — | `tests/ui/offload/pass/result_method.rs` | exact |

---

## Pattern Assignments

### `ferro-queue/src/offload.rs` (new type module)

**Analog 1:** `ferro-queue/src/job.rs` — queue-layer type and trait definition pattern
**Analog 2:** `ferro-ai/src/classifier/mod.rs` — `PhantomData<fn() -> T>` pattern
**Analog 3:** `ferro-queue/src/lib.rs` — `Queueable` blanket impl pattern (lines 74–101)

**Imports pattern** — follow `ferro-queue/src/job.rs` lines 1–8:
```rust
use crate::Error;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;
```
Note: `uuid` is already in `ferro-queue/Cargo.toml` with `features = ["v4", "serde"]` — no new dependency.

**PhantomData pattern** — `framework/src/auth/provider.rs` lines 89–100 shows the project's established `fn() -> T` convention:
```rust
// `fn() -> E` keeps the struct `Send + Sync` regardless of `E`.
_marker: PhantomData<fn() -> E>,

impl<E: EntityTrait> ModelUserProvider<E> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
```
`OffloadHandle<T>` must follow this exact convention (`PhantomData<fn() -> T>`, not `PhantomData<T>`) so the handle is `Send + Sync` unconditionally.

**`OffloadSerializable` marker trait pattern** — no existing analog in the tree; build from design spec (CONTEXT.md D-05). The `Queueable` blanket impl in `ferro-queue/src/lib.rs` lines 74–101 shows the project's blanket-impl style:
```rust
// ferro-queue/src/lib.rs:74-101
pub trait Queueable: Job + serde::Serialize + serde::de::DeserializeOwned {
    fn dispatch(self) -> PendingDispatch<Self>
    where
        Self: Sized,
    {
        PendingDispatch::new(self)
    }
    // …
}

impl<T> Queueable for T where T: Job + serde::Serialize + serde::de::DeserializeOwned {}
```
`OffloadSerializable` follows the same supertrait + blanket impl structure, adding `#[diagnostic::on_unimplemented]` above the trait declaration (first use of this attribute in the tree).

**`HandleKey` newtype pattern** — `ferro-queue/src/job.rs` lines 93–117 shows how `JobPayload` uses `Uuid::new_v4()` for ID generation. `HandleKey` is a thinner newtype wrapping that same UUID-to-string conversion:
```rust
// ferro-queue/src/job.rs:93-96 — UUID v4 mint pattern
pub struct JobPayload {
    pub id: Uuid,
    // …
}
// JobPayload::new() at line 126:
id: Uuid::new_v4(),
```

**`Offloadable` trait pattern** — the `Job` trait in `ferro-queue/src/job.rs` lines 43–89 shows the `#[async_trait]`-decorated trait shape with provided method defaults (e.g. `fn name()`, `fn max_retries()`) that `Offloadable` mirrors for `.offload()`:
```rust
// ferro-queue/src/job.rs:43-51
#[async_trait]
pub trait Job: Send + Sync + 'static {
    async fn handle(&self) -> Result<(), Error>;

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    // …
}
```

---

### `ferro-macros/src/offload.rs` (proc-macro emission site — extend 244)

**Analog:** itself — the 244 implementation is the baseline to extend. Read the file at lines 1–298 before modifying. Two extension points:

**Extension point 1: `OffloadMethodInfo` struct** (lines 48–63) — add `output_type: proc_macro2::TokenStream2` field alongside `returns_result`. The existing `returns_result` bool (lines 60–62) already flags the `Result<T, E>` case; `output_type` carries the extracted `T`:
```rust
// ferro-macros/src/offload.rs:48-63 — current shape; add output_type below returns_result
pub(crate) struct OffloadMethodInfo {
    pub job_ident: proc_macro2::Ident,
    pub method_ident: proc_macro2::Ident,
    pub field_names: Vec<proc_macro2::Ident>,
    pub field_types: Vec<TokenStream2>,
    field_forwards: Vec<FieldForward>,
    pub is_async: bool,
    pub returns_result: bool,
    // 245 adds:
    // pub output_type: TokenStream2,
}
```

**Extension point 2: `collect_info` return-type detection** (lines 170–195) — the existing block (lines 170–185) already detects `Result`. It must now also extract the success type `T`:
```rust
// ferro-macros/src/offload.rs:170-185 — existing returns_result detection
let returns_result = match &method.sig.output {
    ReturnType::Default => false,
    ReturnType::Type(_, ty) => {
        if let Type::Path(type_path) = ty.as_ref() {
            type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "Result")
                .unwrap_or(false)
        } else {
            false
        }
    }
};
```
245 extends this block to also compute `output_type: TokenStream2`: `()` for default return, `T` for bare `-> T`, and the first generic argument of `Result<T, E>` when `returns_result` is true.

**Extension point 3: `emit_job_items` struct emission** (lines 262–297) — the existing struct emission block (lines 271–274) is where the `where` clause for parameter enforcement attaches, and the `impl Offloadable` block is appended after `inventory::submit!`. Current emitted shape:
```rust
// ferro-macros/src/offload.rs:262-297 — current emit_job_items output
quote! {
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
    pub struct #job_ident {
        #( pub #field_names: #field_types, )*
    }

    #[::ferro::async_trait]
    impl ::ferro::queue::Job for #job_ident {
        fn name(&self) -> &'static str { #job_ident_str }
        async fn handle(&self) -> ::std::result::Result<(), ::ferro::queue::Error> {
            let svc = ::ferro::App::make::<dyn #trait_ident>().expect(#expect_msg);
            #call_expr
        }
    }

    ::ferro::inventory::submit! {
        ::ferro::queue::JobRegistrarEntry {
            register: |w: &mut ::ferro::queue::WorkerLoop| { w.register::<#job_ident>(); },
            name: #job_ident_str,
        }
    }
}
```
245 adds after the `inventory::submit!` block:
```rust
    // NEW in 245:
    impl ::ferro::queue::Offloadable for #job_ident {
        type Output = #output_type;
    }
```
And modifies the struct definition to add a `where` clause:
```rust
    // MODIFIED struct with parameter enforcement where clause:
    #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
    pub struct #job_ident
    where
        #( #field_types: ::ferro::queue::OffloadSerializable ),*
    {
        #( pub #field_names: #field_types, )*
    }
```
All emitted paths continue to use `::ferro::queue::*` — never `ferro_queue::` directly. This convention is already enforced throughout lines 277–297 of the existing code.

---

### `ferro-macros/src/service.rs` (proc-macro wiring — extend 244)

**Analog:** itself — lines 183–258 show the collect/emit wiring. The `offload_infos` collection loop (lines 186–200) and the emission loop (lines 246–248) are the two extension points. No structural change is needed to `service.rs` itself — `OffloadMethodInfo` gains `output_type` as a field, and `emit_job_items` reads it. The wiring in `service.rs` is unchanged:

```rust
// ferro-macros/src/service.rs:183-254 — current wiring shape (no change needed)
let mut offload_infos: Vec<crate::offload::OffloadMethodInfo> = Vec::new();
for item in &mut item_trait.items {
    if let syn::TraitItem::Fn(method) = item {
        if let Some(pos) = method.attrs.iter().position(|a| a.path().is_ident("offload")) {
            method.attrs.remove(pos);
            match crate::offload::collect_info(&trait_ident, method) {
                Ok(info) => offload_infos.push(info),
                Err(e) => return e.to_compile_error().into(),
            }
        }
    }
}

let offload_items = offload_infos
    .iter()
    .map(|info| crate::offload::emit_job_items(&trait_ident, info));

let expanded = quote! {
    #item_trait
    #impl_registration
    #fake_impl
    #( #offload_items )*
};
```
The only required change is that `collect_info` now populates `output_type` and `emit_job_items` uses it — `service.rs` itself is a pass-through.

---

### `ferro-queue/src/lib.rs` (re-export shim — additive)

**Analog:** itself, current lines 48–101. The export block structure to follow:
```rust
// ferro-queue/src/lib.rs:48-68 — existing pub use pattern
mod config;
mod db;
mod dispatcher;
mod error;
mod job;
mod migration;
mod worker;

pub use config::QueueConfig;
pub use db::{ … };
pub use dispatcher::{ … };
pub use error::Error;
pub use job::{Job, JobPayload};
pub use migration::CreateJobsTable;
pub use worker::{TenantScopeProvider, Worker, WorkerConfig, WorkerLoop};
```
245 adds:
```rust
mod offload;  // new module declaration
pub use offload::{HandleKey, OffloadHandle, Offloadable, OffloadSerializable};
```
The `Queueable` blanket impl at lines 74–101 shows how to write the `OffloadSerializable` blanket impl — same structural pattern.

---

### `framework/src/lib.rs` (re-export shim — additive)

**Analog:** itself, lines 224–231. The exact block to extend:
```rust
// framework/src/lib.rs:224-231 — current mod queue re-export block
pub mod queue {
    pub use ferro_queue::{
        dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook, CreateJobsTable,
        Error, FailedJobInfo, Job, JobInfo, JobPayload, JobRegistrarEntry, JobState,
        PendingDispatch, Queue, QueueConfig, QueueStats, Queueable, SingleQueueStats,
        TenantScopeProvider, Worker, WorkerConfig, WorkerLoop,
    };
}
```
245 adds `HandleKey, OffloadHandle, Offloadable, OffloadSerializable` to the `pub use ferro_queue::{ … }` list. No other structural change.

---

### `ferro-macros/tests/ui/offload/fail/non_serializable_param.rs` (trybuild fixture)

**Analog:** `ferro-macros/tests/ui/offload/fail/mut_ref_param.rs` (lines 1–19)

**Fixture structure to copy exactly:**
```rust
// ferro-macros/tests/ui/offload/fail/mut_ref_param.rs:1-19
//! Compile-fail: #[offload] on a method with a &mut parameter (OFFLOAD-01-c).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};

#[derive(Default)]
pub struct Svc;

#[service(Svc)]
#[async_trait]
pub trait MyService {
    #[offload]
    async fn mutate(&self, data: &mut String);
}

fn main() {}
```

The new `non_serializable_param.rs` follows this template. The difference: declare a local non-serializable struct (no `#[derive(Serialize, Deserialize)]`) and use it as a parameter type. Include the `use serde::{Deserialize, Serialize};` import so the compiler's failure is clearly on `OffloadSerializable`, not on a missing import.

`.stderr` snapshot: generate via `TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro`. The existing `mut_ref_param.stderr` shows the expected single-line format:
```
// ferro-macros/tests/ui/offload/fail/mut_ref_param.stderr:1-6
error: #[offload] parameters may not be &mut references — Job payloads must be owned and serializable
  --> tests/ui/offload/fail/mut_ref_param.rs:16:34
   |
16 |     async fn mutate(&self, data: &mut String);
   |                                  ^^^^^^^^^^^
```
The `non_serializable_param.stderr` will contain an `E0277` error with the `#[diagnostic::on_unimplemented]` message instead. Generate via `TRYBUILD=overwrite` — do not hand-author.

---

### `ferro-macros/tests/ui/offload/fail/non_serializable_return.rs` + `.stderr` (trybuild fixture)

**Analog:** same as above — `mut_ref_param.rs` template. The difference: the parameter type is serializable, but the return type (or its `Result<T, E>` success inner type) is a local struct without `Serialize + DeserializeOwned`. `.stderr` generated via `TRYBUILD=overwrite`.

---

### New pass fixture under `ferro-macros/tests/ui/offload/pass/` (trybuild fixture)

**Analog:** `ferro-macros/tests/ui/offload/pass/result_method.rs` (lines 1–31)

**Fixture structure to copy exactly:**
```rust
// ferro-macros/tests/ui/offload/pass/result_method.rs:1-31
//! Compile-pass: Result<T, E> return type on an #[offload] method (OFFLOAD-01-b/D-06).
#![allow(unused_imports, dead_code)]

extern crate ferro_rs as ferro;
extern crate serde;

use ferro::{async_trait, service};

#[derive(Default)]
pub struct Exporter;

#[service(Exporter)]
#[async_trait]
pub trait ExporterService {
    #[offload]
    async fn export(&self, id: i64) -> Result<(), String>;
}

#[async_trait]
impl ExporterService for Exporter {
    async fn export(&self, _id: i64) -> Result<(), String> {
        Ok(())
    }
}

fn main() {
    // Derived struct carries the i64 field; Result branch in handle() is verified
    // by compilation (the impl job handles the Result<(), String> return).
    let _job = ExporterServiceExportJob { id: 7 };
}
```

The new pass fixture extends the `fn main()` assertion to also verify the `.offload()` method exists and that its return type is `OffloadHandle<Output>`. The assertion must be structural (compile-time type check), not a runtime assertion.

---

## Shared Patterns

### `#[async_trait]` on traits with provided async defaults
**Source:** `ferro-queue/src/job.rs:43` — `#[async_trait] pub trait Job`
**Apply to:** `Offloadable` trait definition in `ferro-queue/src/offload.rs`

The `Offloadable::offload()` provided default body uses `.await` internally (calls `PendingDispatch::dispatch().await`). Apply `#[async_trait]` to the `Offloadable` trait declaration exactly as `Job` does, so the provided body compiles.

### `::ferro::queue::*` path convention in macro output
**Source:** `ferro-macros/src/offload.rs:277-296` — every emitted path uses `::ferro::queue::Foo`
**Apply to:** all new `quote!` blocks in `emit_job_items` (the `impl Offloadable` block and the struct `where` clause)

Example from existing code (line 277):
```rust
#[::ferro::async_trait]
impl ::ferro::queue::Job for #job_ident {
```
New emission must follow the same convention:
```rust
impl ::ferro::queue::Offloadable for #job_ident {
    type Output = #output_type;
}
```
Never emit `ferro_queue::Offloadable` — the consumer crate may not have `ferro-queue` as a direct dependency.

### `PhantomData<fn() -> T>` for `Send + Sync` unconditionally
**Source:** `framework/src/auth/provider.rs:89–100` — established project convention with comment `// 'fn() -> E' keeps the struct Send + Sync regardless of E`
**Apply to:** `OffloadHandle<T>._phantom` field in `ferro-queue/src/offload.rs`

```rust
// framework/src/auth/provider.rs:89-91
pub struct ModelUserProvider<E: EntityTrait> {
    // `fn() -> E` keeps the struct `Send + Sync` regardless of `E`.
    _marker: PhantomData<fn() -> E>,
}
```

### `#[serde(skip)]` on phantom fields
**Source:** RESEARCH.md Pitfall 4 (no existing example in tree — this is the first serde-skipped phantom in the queue layer)
**Apply to:** `OffloadHandle<T>._phantom` field

Without `#[serde(skip)]`, serde-derive generates a bound `T: Serialize` on the phantom field, making `OffloadHandle<T>` non-serializable when `T: !Serialize`. Since the handle only serializes `HandleKey` (a `String`), the phantom is always excluded:
```rust
#[serde(skip)]
_phantom: std::marker::PhantomData<fn() -> T>,
```

### `Uuid::new_v4()` for ID minting
**Source:** `ferro-queue/src/job.rs:126` — `id: Uuid::new_v4()`
**Apply to:** `HandleKey::new()` in `ferro-queue/src/offload.rs`

`uuid` is already in `ferro-queue/Cargo.toml` with `features = ["v4", "serde"]`. No new dependency.

### Blanket impl over supertrait bounds
**Source:** `ferro-queue/src/lib.rs:101` — `impl<T> Queueable for T where T: Job + serde::Serialize + serde::de::DeserializeOwned {}`
**Apply to:** `OffloadSerializable` blanket impl in `ferro-queue/src/offload.rs`

```rust
// ferro-queue/src/lib.rs:101
impl<T> Queueable for T where T: Job + serde::Serialize + serde::de::DeserializeOwned {}
```
`OffloadSerializable` blanket follows the same one-liner style:
```rust
impl<T: serde::Serialize + serde::de::DeserializeOwned> OffloadSerializable for T {}
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `#[diagnostic::on_unimplemented]` usage | trait attribute | — | No existing use of this attribute anywhere in the tree; first use in 245. Pattern comes from Rust Reference + CONTEXT.md D-05. |

---

## Metadata

**Analog search scope:** `ferro-macros/src/`, `ferro-macros/tests/`, `ferro-queue/src/`, `framework/src/`, `ferro-ai/src/`
**Files scanned:** 12 (offload.rs, service.rs, offload_macro.rs, all ui/offload fixtures, job.rs, dispatcher.rs, ferro-queue/lib.rs, framework/lib.rs, ferro-ai/classifier/mod.rs, auth/provider.rs)
**Pattern extraction date:** 2026-08-13
