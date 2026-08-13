# Phase 245: Typed Result Handle + Serializable-Contract Enforcement — Research

**Researched:** 2026-08-13
**Domain:** Rust proc-macro extension, `#[diagnostic::on_unimplemented]`, trybuild harness, ferro-queue
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** The typed handle is produced by an `.offload()` method on the already-public derived
  Job struct. The `#[offload]` trait method itself stays `-> T` in-process. `.offload()` is the
  enqueue entrypoint, not a mutation of the method signature.
- **D-02:** The macro emits `type Output = <method return success type>`. `.offload()` returns
  `Result<OffloadHandle<Self::Output>, Error>`.
- **D-03:** `.offload()` enqueues through the existing `dispatch`/`PendingDispatch` path. It wraps
  that call and mints the handle.
- **D-04:** `.offload()` and `type Output` live on a new `Offloadable` trait. The macro emits
  `impl Offloadable for <..>Job`. The `offload()` body is a provided default.
- **D-05:** Serializable enforcement uses a marker trait `OffloadSerializable: Serialize +
  DeserializeOwned` with a blanket impl and `#[diagnostic::on_unimplemented]`. Return type
  enforced via `type Output: OffloadSerializable`; parameters via a matching bound. MSRV is
  1.94.1, so the attribute is freely available.
- **D-06:** Both param-side and return-side non-serializable failures are proven by trybuild
  fixtures (extending the 244 harness).
- **D-07:** Handle key is always a fresh UUID v4 minted at enqueue, decoupled from
  `Job::idempotency_key()`.
- **D-08:** `OffloadHandle<T>` holds a `HandleKey` (UUID-backed string newtype) + `PhantomData<T>`.
  Inert in 245: exposes `.key()` / `.id()`. No `.await` / `.subscribe()`.
- **D-09:** `type Output = T` is the success type. For `Result<T, E>` the handle is
  `OffloadHandle<T>`; `E` keeps 244 D-07 treatment. For `-> T`, `Output = T`; for `-> ()`,
  `Output = ()`. Enforcement targets `Output` and parameters, never `E`.
- **D-10:** The worker's `Job::handle()` still discards the value in 245. 245 locks the typed
  contract and compile-time enforcement only; value capture is Phase 246.

### Claude's Discretion

- Exact module home for `OffloadHandle`, `Offloadable`, `OffloadSerializable`, `HandleKey` (new
  `offload` module in `ferro-queue` vs `framework`).
- Whether `OffloadHandle<T>` derives `Serialize, Deserialize, Clone, Debug`.
- The exact `#[diagnostic::on_unimplemented]` wording (message + note).
- How the parameter-side bound is expressed (per-field vs generated `where` assertion).
- The concrete UUID crate/path used for `HandleKey` generation.

### Deferred Ideas (OUT OF SCOPE)

- Result → `ferro-projection` snapshot keyed by the handle; terminal error state — Phase 246.
- Handle `.await` / `.subscribe()` resolve + streaming — Phase 247.
- `#[offload(queue = …, retries = …)]` config surface.
- Reconciling a deduped job with multiple distinct handles — Phase 246/247.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OFFLOAD-02 | Calling an offloaded method returns a typed result handle; a method whose parameter or return type is not `Serialize`/`DeserializeOwned` fails at compile time with a clear, type-naming diagnostic (this enforcement is the module-isolation boundary). | `#[diagnostic::on_unimplemented]` on `OffloadSerializable` provides the branded message. Trybuild fixtures in the existing harness prove both param-side and return-side failures. `OffloadHandle<T>` wraps the UUID key and is returned by `.offload()`. |
</phase_requirements>

---

## Summary

Phase 245 adds two related properties to the `#[offload]` derivation shipped in Phase 244: a typed
result handle (`OffloadHandle<T>`) returned by an `.offload()` enqueue method on the derived Job
struct, and compile-time enforcement that every parameter and return type crossing the offload
boundary is serializable, surfaced through a branded diagnostic.

The 244 codebase is entirely in `ferro-macros/src/offload.rs` and `ferro-macros/src/service.rs`.
`collect_info` captures method metadata (including `returns_result` already, which now also needs
to capture the success type for `type Output`). `emit_job_items` is where the new `impl Offloadable`
block and enforcement bounds are emitted. The trybuild harness in `tests/offload_macro.rs` already
covers pass and fail cases; two new fail fixtures attach to the same `compile_fail` glob.

`#[diagnostic::on_unimplemented]` was stabilized in Rust 1.78 [VERIFIED: Rust release notes].
The project's MSRV is 1.94.1 and the local toolchain is pinned to 1.94.1 (confirmed via
`rust-toolchain.toml`). The attribute is therefore freely usable without any feature gate.

UUID v4 is already a direct dependency of `ferro-queue` with the `v4` and `serde` features
enabled — no new workspace dependency is needed for `HandleKey` generation.

**Primary recommendation:** Place all new types (`OffloadHandle`, `Offloadable`, `OffloadSerializable`,
`HandleKey`) in a new `ferro-queue/src/offload.rs` module, exported from `ferro-queue/src/lib.rs`,
and added to the `::ferro::queue { … }` re-export block in `framework/src/lib.rs`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `OffloadHandle<T>` type + `HandleKey` newtype | `ferro-queue` | — | Handle represents an enqueued job identity; naturally co-located with the queue's Job/PendingDispatch types |
| `Offloadable` trait (`.offload()` + `type Output`) | `ferro-queue` | — | Supertrait of `Job`; the impl is emitted by the macro, but the trait itself must live in the queue crate |
| `OffloadSerializable` marker trait + blanket impl | `ferro-queue` | — | Enforces the wire contract at the queue boundary; `Serialize + DeserializeOwned` is already central to `ferro-queue` |
| Macro emission (`impl Offloadable`, enforcement bounds) | `ferro-macros` | — | `emit_job_items` in `offload.rs` already owns all derived code for `#[offload]` |
| `::ferro::queue::*` re-export of new types | `framework` | — | Convention established in 244; `framework/src/lib.rs` re-exports `ferro_queue` items under `::ferro::queue` |
| Trybuild fixtures (param-fail, return-fail) | `ferro-macros/tests` | — | Two new files in `tests/ui/offload/fail/`; attached to the existing `offload_macro_ui` harness |

---

## Standard Stack

### Core (all already in workspace — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `uuid` | 1 (features: `v4`, `serde`) | UUID v4 generation + serde for `HandleKey` | Already a direct dependency of `ferro-queue` with both features enabled [VERIFIED: ferro-queue/Cargo.toml] |
| `serde` | 1 | `Serialize`/`DeserializeOwned` bounds for `OffloadSerializable` | Already in the workspace |
| `trybuild` | 1 | UI test harness for compile-fail and compile-pass fixtures | Already a dev-dependency of `ferro-macros` [VERIFIED: ferro-macros/Cargo.toml] |
| `#[diagnostic::on_unimplemented]` | stable ≥ 1.78 | Branded compile-time diagnostic on `OffloadSerializable` | Stable in Rust 1.78; MSRV is 1.94.1 [VERIFIED: local `rustc --version`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::marker::PhantomData` | stdlib | Carry `T` in `OffloadHandle<T>` without owning it | Required for the typed handle; well-established pattern in the codebase (`ferro-ai/src/classifier/mod.rs`) |
| `proc_macro2` / `quote` / `syn` | existing | Emit `impl Offloadable` + enforcement bounds in macro | Already used in `emit_job_items` |

**No new Cargo dependencies introduced in this phase.**

---

## Architecture Patterns

### System Architecture Diagram

```
#[service] macro expansion (ferro-macros/src/service.rs)
         |
         v
collect_info() [offload.rs] ──── reads return type, params
         |
         | (OffloadMethodInfo: adds output_type: TokenStream2)
         v
emit_job_items() [offload.rs]
    ├── existing: #[derive(Serialize, Deserialize)] struct <Trait><Method>Job { ... }
    ├── existing: impl ::ferro::queue::Job for <..>Job { handle() discards value }
    ├── existing: inventory::submit! { ... }
    ├── NEW: impl ::ferro::queue::Offloadable for <..>Job {
    │       type Output = <success_type>;
    │       // offload() is a provided default on the trait
    │   }
    └── NEW: where-clause bounds on struct OR per-field bounds (parameter enforcement)

::ferro::queue namespace (framework/src/lib.rs mod queue)
    ├── existing: Job, PendingDispatch, Error, dispatch, ...
    └── NEW: OffloadHandle, Offloadable, OffloadSerializable, HandleKey

ferro-queue/src/offload.rs [NEW MODULE]
    ├── OffloadSerializable trait + blanket impl
    ├── HandleKey newtype (wraps String, minted from Uuid::new_v4())
    ├── OffloadHandle<T> struct (HandleKey + PhantomData<T>)
    └── Offloadable trait (type Output: OffloadSerializable; async fn offload() with default body)

trybuild fixtures [extend existing harness]
    tests/ui/offload/fail/non_serializable_param.rs   + .stderr
    tests/ui/offload/fail/non_serializable_return.rs  + .stderr
```

### Recommended Project Structure (additions only)

```
ferro-queue/src/
├── offload.rs        # NEW — OffloadHandle, Offloadable, OffloadSerializable, HandleKey
└── lib.rs            # add pub mod offload; pub use offload::{...}

framework/src/
└── lib.rs            # add to mod queue { pub use ferro_queue::{OffloadHandle, Offloadable, OffloadSerializable, HandleKey, ...} }

ferro-macros/src/
└── offload.rs        # extend collect_info + emit_job_items

ferro-macros/tests/ui/offload/fail/
├── non_serializable_param.rs    # NEW
├── non_serializable_param.stderr
├── non_serializable_return.rs   # NEW
└── non_serializable_return.stderr
```

### Pattern 1: `OffloadSerializable` with `#[diagnostic::on_unimplemented]`

**What:** A marker trait with a blanket impl; the attribute ensures rustc emits the branded
message instead of the default "the trait `Serialize` is not implemented" chain.

**When to use:** The only type requiring this attribute is `OffloadSerializable` itself. Every
other bound in the codebase uses plain trait bounds.

**Example (design sketch — exact wording is discretion):**

```rust
// Source: CONTEXT.md D-05 + RFC 3368 (diagnostic namespace, stable 1.78)
#[diagnostic::on_unimplemented(
    message = "`{Self}` crosses the #[offload] isolation boundary \
               and must be `Serialize + DeserializeOwned`",
    note = "offloaded parameters and return types travel as a queue \
            payload; implement `Serialize` and `DeserializeOwned` for \
            `{Self}` to seal the module across the isolation boundary"
)]
pub trait OffloadSerializable: serde::Serialize + serde::de::DeserializeOwned {}

impl<T: serde::Serialize + serde::de::DeserializeOwned> OffloadSerializable for T {}
```

**Key property:** `{Self}` interpolation in `message`/`note` strings is confirmed to expand to the
concrete type that fails the bound. This is the mechanism that satisfies OFFLOAD-02's "type-naming
diagnostic" requirement [CITED: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-diagnosticon_unimplemented-attribute].

### Pattern 2: `Offloadable` trait with provided default body

**What:** `Offloadable` has `type Output` and an `async fn offload()` with a provided body.
The macro emits only the `type Output` associated type per method; the `offload()` body
executes identically for every `impl`.

**Example:**

```rust
// Source: CONTEXT.md D-04
#[::ferro::async_trait]
pub trait Offloadable: ::ferro::queue::Job {
    type Output: ::ferro::queue::OffloadSerializable;

    async fn offload(self) -> Result<::ferro::queue::OffloadHandle<Self::Output>, ::ferro::queue::Error>
    where
        Self: ::serde::Serialize + ::serde::de::DeserializeOwned + Sized,
    {
        let key = ::ferro::queue::HandleKey::new();
        ::ferro::queue::PendingDispatch::new(self).dispatch().await?;
        Ok(::ferro::queue::OffloadHandle::new(key))
    }
}
```

**Macro emission (per method):**

```rust
// Source: pattern derived from existing emit_job_items() in ferro-macros/src/offload.rs
impl ::ferro::queue::Offloadable for #job_ident {
    type Output = #output_type; // success type extracted from return sig
}
```

### Pattern 3: `OffloadHandle<T>` definition

**What:** Inert typed wrapper around a `HandleKey`. `PhantomData<T>` carries the success type
without runtime cost. `T` need not be `Serialize` for the handle to be serializable (the handle
only holds the key string, which is always serializable).

**Example:**

```rust
// Source: established PhantomData pattern; see ferro-ai/src/classifier/mod.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OffloadHandle<T> {
    key: HandleKey,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> OffloadHandle<T> {
    pub fn new(key: HandleKey) -> Self {
        Self { key, _phantom: std::marker::PhantomData }
    }

    pub fn key(&self) -> &str { self.key.as_str() }
    pub fn id(&self) -> &HandleKey { &self.key }
}
```

Note: using `PhantomData<fn() -> T>` (a function-pointer phantom) instead of `PhantomData<T>`
makes `OffloadHandle<T>: Send + Sync` unconditionally, which is required since `T` may not be
`Send`. For 245's purposes (inert handle, `T` not resolved), this is the safer variance choice.
`PhantomData<T>` works if `T: Send`, but `fn() -> T` is always `Send + Sync` regardless of `T`.

### Pattern 4: `HandleKey` newtype

**What:** A transparent `String` newtype backed by `Uuid::new_v4().to_string()`. Thin wrapper so
downstream phases can add methods (e.g., `parse()` for persistence lookup) without breaking callsites.

**Example:**

```rust
// uuid crate already in ferro-queue with features ["v4", "serde"] — no new dependency
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct HandleKey(String);

impl HandleKey {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

### Pattern 5: Parameter-side enforcement

**What:** The 244 Job struct derives `#[derive(Serialize, Deserialize)]`, which already makes
non-serializable parameter types fail at the `derive` site. However, the error message comes from
serde-derive, not from `OffloadSerializable`. To fire the branded message for parameters, the
macro must emit an explicit bound.

**Two options and the winner:**

| Approach | Mechanism | Branded message fires? | Notes |
|----------|-----------|----------------------|-------|
| Per-field bound on struct fields | `struct Foo where Field: OffloadSerializable` | Yes — rustc checks `OffloadSerializable` before serde-derive sees it | Clean; same site as serde check |
| Const/fn assertion | `const _: fn() = || { let _: &dyn OffloadSerializable = ... }` | Yes | More indirection, error points to the assertion not the field |

**Recommended approach: per-field `where` clause on the derived struct.**

The macro already uses field types in `emit_job_items`. The additional where clause is:

```rust
// Emitted inside emit_job_items alongside the struct definition
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct #job_ident where #( #field_types: ::ferro::queue::OffloadSerializable ),*
{
    #( pub #field_names: #field_types, )*
}
```

When a field type does not implement `OffloadSerializable`, rustc fires the
`#[diagnostic::on_unimplemented]` message from `OffloadSerializable`, naming `{Self}` as the
concrete field type. The serde derive also fails, but rustc typically surfaces the first
unsatisfied bound first, so the branded message appears before the serde chain.

**Caveat:** With a blanket `impl<T: Serialize + DeserializeOwned> OffloadSerializable for T {}`,
any type that is both `Serialize` and `DeserializeOwned` automatically satisfies
`OffloadSerializable`. The `#[diagnostic::on_unimplemented]` message fires only for types that
satisfy *neither* (or one but not the other). For the "param is not serializable at all" case,
the message fires correctly and names the type.

### Anti-Patterns to Avoid

- **Emitting `OffloadHandle` / `Offloadable` paths as direct `ferro_queue::` crate paths.** All
  generated code must use `::ferro::queue::*` so it resolves in any consumer crate that depends on
  `ferro-rs` only. The convention is established in 244 and must be preserved.
- **Adding a `#[serde(bound = "")]` attribute to `OffloadHandle<T>`.** The handle is always
  serializable regardless of `T` (it only holds the UUID key string). Using `fn() -> T` in
  `PhantomData` marks the phantom as covariant and always `Send + Sync`; omit any serde bound
  on the phantom field since `#[serde(skip)]` removes it from the serde path entirely.
- **Firing `OffloadSerializable` on the error type `E`.** D-09 is explicit: enforcement targets
  the success type and parameters, never `E`. The macro's existing `returns_result` flag already
  distinguishes the success type; D-09 requires the macro to strip the `E` wrapper before
  assigning `type Output`.
- **Placing `OffloadHandle`/`Offloadable` in `framework/src/` rather than `ferro-queue/src/`.** These
  types are semantically queue-layer types. Placing them in `ferro-queue` keeps the dependency
  graph acyclic (the macro crate depends on `ferro-rs` only as a dev-dependency for trybuild).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| UUID v4 generation for `HandleKey` | Custom random-string generator | `uuid::Uuid::new_v4()` | `uuid` is already in `ferro-queue` with `v4` + `serde` features; RFC 4122 compliant; serde support built in |
| Branded compile-time diagnostic | `compile_error!()` macro emission from proc-macro | `#[diagnostic::on_unimplemented]` on a trait | The attribute fires at the Rust type-checker level, produces better span information, and supports `{Self}` interpolation |
| Tracking the type parameter in `OffloadHandle<T>` at zero runtime cost | A `Box<dyn Any>` or type-erased storage | `PhantomData<fn() -> T>` | Zero-size, no allocations, enforces compile-time type tracking for the 247 subscribe path |
| Snapshot the `.stderr` for trybuild | Copy-paste from terminal | `TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro` | Standard trybuild workflow; already documented in the harness comment in `tests/offload_macro.rs` |

---

## Runtime State Inventory

> Phase 245 is not a rename/refactor/migration phase. This section is omitted.

---

## Verification in Depth: The Three Genuine Unknowns

### Unknown 1: `#[diagnostic::on_unimplemented]` mechanics

**Stabilization:** The attribute is stable since Rust 1.78.0 (2024-05-02), landed via RFC 3368
(the `#[diagnostic]` tool attribute namespace). It requires no feature gate on Rust ≥ 1.78. The
project MSRV is 1.94.1; the local toolchain is pinned to exactly 1.94.1. No compatibility issue
exists [VERIFIED: rustc 1.94.1 confirmed via `rustc --version`; stabilization confirmed via
reference docs].

**Interaction with blanket impl:** The attribute is placed on the trait declaration. With a blanket
`impl<T: Serialize + DeserializeOwned> OffloadSerializable for T {}`, the attribute fires
whenever Rust cannot satisfy `T: OffloadSerializable` — which happens exactly when `T` does not
implement one or both of `Serialize` / `DeserializeOwned`. The branded message therefore fires for
the correct case without interference from the blanket impl.

**`{Self}` interpolation:** The `message` and `note` fields of `#[diagnostic::on_unimplemented]`
support several interpolation variables. `{Self}` expands to the concrete type that failed to
satisfy the trait bound [CITED: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-diagnosticon_unimplemented-attribute].
For a type `MyStruct` that is not `Serialize`, the message becomes:
"`MyStruct` crosses the #[offload] isolation boundary and must be `Serialize + DeserializeOwned`".

**How it appears in trybuild `.stderr`:** The attribute causes rustc to emit an `error[E0277]`
diagnostic with the custom message in the primary span. The trybuild `.stderr` format captures
the full rustc diagnostic output. An example structure (based on confirmed rustc behavior for this
attribute):

```
error[E0277]: `MyStruct` crosses the #[offload] isolation boundary and must be `Serialize + DeserializeOwned`
  --> tests/ui/offload/fail/non_serializable_param.rs:XX:YY
   |
XX |     #[offload]
   |     ^^^^^^^^^^ the trait `OffloadSerializable` is not implemented for `MyStruct`
   |
   = note: offloaded parameters and return types travel as a queue payload; implement `Serialize` and `DeserializeOwned` for `MyStruct` to seal the module across the isolation boundary
   = help: the trait `OffloadSerializable` is implemented for all types that implement `Serialize + DeserializeOwned`
```

**Important gotcha:** The exact secondary lines (`note`, `help`) may vary across Rust versions.
Trybuild matches `.stderr` content against the captured diagnostic; version-sensitive lines are a
known source of breakage. Two strategies:

1. Generate `.stderr` via `TRYBUILD=overwrite` on the pinned 1.94.1 toolchain (the standard
   workflow, already documented in the harness comment).
2. Accept that `.stderr` snapshots must be regenerated if the toolchain is ever bumped. This is
   expected behavior and not a defect in the test design.

The `message` text (the custom branded line) is the most stable part — it is controlled entirely
by the attribute argument, not by rustc heuristics.

### Unknown 2: Parameter-side vs return-side enforcement — making the SAME branded diagnostic fire

**Return-side (straightforward):** `type Output: OffloadSerializable` on `Offloadable` — rustc
checks this bound when the `impl Offloadable for XJob` is emitted with a concrete `Output` type.
If `Output` does not implement `OffloadSerializable`, the branded diagnostic fires at the `impl`
site.

**Parameter-side (requires explicit bound):** The 244 Job struct already derives `Serialize,
Deserialize`. A non-serializable parameter type fails at the `derive` site with serde's own
message, not the branded message. To fire the branded message, the macro must emit a `where`
clause on the struct:

```rust
// Pattern: where clause on the derived struct in emit_job_items
#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct #job_ident
where
    #( #field_types: ::ferro::queue::OffloadSerializable ),*
{
    #( pub #field_names: #field_types, )*
}
```

**Which diagnostic fires first?** When a field type fails both `OffloadSerializable` and serde,
rustc may emit multiple errors. The `where` bound is checked at the struct definition point,
before the serde derive macro's internal bounds are checked. In practice, the `E0277` from the
`where` clause appears first, and the custom `#[diagnostic::on_unimplemented]` message is
attached to it. However, the serde error may also appear in the same compilation.

**Trybuild implication:** The `.stderr` for the param-side fixture must capture all errors emitted
by that compilation. `TRYBUILD=overwrite` captures the full output, so the snapshot reflects
exactly what rustc says for the 1.94.1 toolchain. The branded message for `OffloadSerializable`
will be present in the output, satisfying SC#2's "clear, type-naming message" requirement.

**Alternative (const assertion):** A generated hidden const/fn that asserts the bound:

```rust
const _: fn() = || {
    fn assert_offload_serializable<T: ::ferro::queue::OffloadSerializable>() {}
    assert_offload_serializable::<#field_type>();
};
```

This fires the branded message at the assertion site, with a span pointing into the generated code
rather than the trait definition site. It is more indirection and the span is less clear. The
`where`-clause approach is preferred.

**D-09 return type extraction:** For `-> Result<T, E>`, the macro must extract `T` as the
`Output`, not `Result<T, E>`. The existing `returns_result` bool in `OffloadMethodInfo` signals
that extraction is needed; the macro must also capture the inner success type token stream. The
extraction can be done by matching the `Type::Path` generic args when `returns_result` is true:

```rust
// Pseudocode for collect_info extension
let output_type: TokenStream2 = match &method.sig.output {
    ReturnType::Default => quote! { () },
    ReturnType::Type(_, ty) => {
        if returns_result {
            // Extract T from Result<T, E>
            extract_result_ok_type(ty)?
        } else {
            quote! { #ty }
        }
    }
};
```

### Unknown 3: Trybuild harness extension — exact mechanics

**Harness file:** `ferro-macros/tests/offload_macro.rs` (read and confirmed). It contains a
single `#[test]` function that calls:
```rust
t.pass("tests/ui/offload/pass/*.rs");
t.compile_fail("tests/ui/offload/fail/*.rs");
```

The new fail fixtures are added by creating two new files in `tests/ui/offload/fail/`:
- `non_serializable_param.rs` — a trait with an `#[offload]` method whose parameter type does not
  implement `Serialize + DeserializeOwned`
- `non_serializable_return.rs` — a trait with an `#[offload]` method whose return type (or its
  success type, if `Result<T, E>`) does not implement `Serialize + DeserializeOwned`

Each `.rs` file must be accompanied by a `.stderr` snapshot file of exactly the same base name.
The `compile_fail` glob `"tests/ui/offload/fail/*.rs"` picks them up automatically — no changes
to `offload_macro.rs` itself are needed.

**Generating `.stderr` files:** Run:
```bash
TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro
```
This command is already documented in the `offload_macro.rs` harness comment. Trybuild captures
the full rustc stderr output and writes it to the matching `.stderr` file. The generated snapshot
must be committed to git.

**How trybuild matches `.stderr`:** Trybuild compares the captured stderr against the snapshot
line by line. It normalizes some version-specific paths but is otherwise an exact match. The
standard recommendation is: generate on the pinned toolchain, commit, and regenerate whenever the
toolchain is bumped.

**Fixture structure:** All existing pass fixtures use the same pattern:
```rust
extern crate ferro_rs as ferro;
extern crate serde;
use ferro::{async_trait, service};
// ... serde derives, Default impl, service/impl pair ...
fn main() { /* structural assertion */ }
```
The new fail fixtures follow the same template but omit `#[derive(Serialize, Deserialize)]` on the
relevant type to trigger the enforcement.

---

## Common Pitfalls

### Pitfall 1: Emitting `ferro_queue::` paths instead of `::ferro::queue::` in macro output

**What goes wrong:** Generated code fails to compile in any crate that depends on `ferro-rs` but
not directly on `ferro-queue`.

**Why it happens:** The macro developer refers to the underlying crate path (`ferro_queue::`) out
of familiarity, forgetting that macro output is emitted into the consumer's crate context.

**How to avoid:** All generated `TokenStream2` in `emit_job_items` uses `::ferro::queue::Foo`
paths. The 244 code confirms this pattern — search for `::ferro::queue` in `offload.rs`. New
types (`OffloadHandle`, `Offloadable`, `OffloadSerializable`, `HandleKey`) follow the same rule.

**Warning signs:** A consumer crate fails with `error[E0433]: failed to resolve: use of
undeclared crate or module 'ferro_queue'`.

### Pitfall 2: `PhantomData<T>` making `OffloadHandle<T>: !Send` when `T: !Send`

**What goes wrong:** `OffloadHandle<T>` cannot be sent across threads, breaking async dispatch.

**Why it happens:** `PhantomData<T>` adopts `T`'s `Send`/`Sync` variance. If `T` is not `Send`,
neither is `PhantomData<T>`, and therefore neither is `OffloadHandle<T>`.

**How to avoid:** Use `PhantomData<fn() -> T>` (function pointer phantom). A `fn() -> T` type is
always `Send + Sync` regardless of `T`. This makes `OffloadHandle<T>` unconditionally
`Send + Sync`, which is correct since the handle holds only a UUID string (not a `T` value) and
must be sendable across async task boundaries.

**Warning signs:** Clippy or rustc emit `Send` bound violations when `OffloadHandle<T>` is used
in an `async` context with a non-`Send` `T`.

### Pitfall 3: `type Output: OffloadSerializable` fires during `impl Offloadable` emission, not at call site

**What goes wrong:** The diagnostic fires at the wrong span — deep inside the macro expansion
rather than at the trait method definition — making it harder for users to locate the offending type.

**Why it happens:** The `impl Offloadable for XJob { type Output = MyNonSerializableType; }`
is emitted inside the `#[service]` macro expansion. The error span points to the macro invocation
site, not to the return type in the trait definition.

**How to avoid:** This is partly unavoidable with proc-macro generated code; the branded message
text compensates by naming `{Self}` (the concrete type). Consider emitting a `span` annotation
from the original return-type `Span` if possible, using `proc_macro2::Span` tracking from
`collect_info`. At minimum, the `#[diagnostic::on_unimplemented]` message names the type, which
satisfies SC#2's "type-naming message" requirement.

**Warning signs:** Error points to the `#[service]` attribute line rather than the return type.
This is cosmetically suboptimal but not a functional failure of the enforcement.

### Pitfall 4: Serde `#[serde(skip)]` on `PhantomData` causes serde to omit the field — handle remains serializable

**What goes wrong:** Without special handling, serde requires `T: Serialize` on `OffloadHandle<T>`
because `PhantomData<T>` appears in the struct. This contradicts the design intent (the handle
must be serializable even when `T` is not).

**Why it happens:** Serde-derive generates a bound `T: Serialize` on any field whose type mentions
`T`, including `PhantomData<T>`.

**How to avoid:** Annotate the phantom field with `#[serde(skip)]`. Serde then excludes the field
entirely from its bound generation. Since `PhantomData` is zero-sized and skipped, serde only
needs to serialize/deserialize the `HandleKey` field, which is always serializable. The
`#[serde(skip)]` annotation is the canonical solution for phantom fields.

### Pitfall 5: trybuild `.stderr` snapshot is rustc-version-sensitive

**What goes wrong:** CI fails on a future Rust bump because rustc changed a diagnostic phrasing
that trybuild matches literally.

**Why it happens:** Trybuild does an exact match on the captured stderr. Rustc's secondary notes
and `help` lines change between versions.

**How to avoid:** Regenerate via `TRYBUILD=overwrite` whenever the toolchain is bumped. The
`rust-toolchain.toml` pins the toolchain to 1.94.1; the snapshot is stable for as long as that pin
holds. Flag this in the PLAN as a known maintenance concern.

---

## Code Examples

### Existing `collect_info` function — current shape (244)

From `ferro-macros/src/offload.rs` (read verbatim):
- `OffloadMethodInfo` fields: `job_ident`, `method_ident`, `field_names`, `field_types`,
  `field_forwards`, `is_async`, `returns_result`.
- `returns_result` is detected at lines 171–185 by checking if the last path segment is `Result`.
- **245 extends `OffloadMethodInfo` with `output_type: TokenStream2`** — the success type for
  `type Output`. For `Result<T, E>`, this is `T`; for bare `-> T`, this is `T`; for `-> ()` or
  default, this is `()`.

### Existing `emit_job_items` function — current shape (244)

From `ferro-macros/src/offload.rs` (read verbatim):
- Emits: derived Job struct with `#[derive(Serialize, Deserialize)]`, `impl Job`, `inventory::submit!`.
- **245 adds:** `where` clause on the struct for parameter enforcement, and an `impl Offloadable`
  block with `type Output = #output_type`.
- All emitted paths use `::ferro::queue::*` and `::ferro::*` — this convention must continue.

### Existing trybuild harness (244)

From `ferro-macros/tests/offload_macro.rs` (read verbatim):
```rust
#[test]
fn offload_macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/offload/pass/*.rs");
    t.compile_fail("tests/ui/offload/fail/*.rs");
}
```

New fail fixtures are added under `tests/ui/offload/fail/` and picked up by the existing glob.

### `::ferro::queue::*` re-export block — current shape (244)

From `framework/src/lib.rs` lines 223–231 (read verbatim):
```rust
pub mod queue {
    pub use ferro_queue::{
        dispatch, dispatch_later, dispatch_to, register_tenant_capture_hook, CreateJobsTable,
        Error, FailedJobInfo, Job, JobInfo, JobPayload, JobRegistrarEntry, JobState,
        PendingDispatch, Queue, QueueConfig, QueueStats, Queueable, SingleQueueStats,
        TenantScopeProvider, Worker, WorkerConfig, WorkerLoop,
    };
}
```

**245 adds** `OffloadHandle, Offloadable, OffloadSerializable, HandleKey` to this list.

---

## Module Home Decision (Claude's Discretion)

**Recommendation: new `ferro-queue/src/offload.rs` module, re-exported from `ferro-queue/src/lib.rs`.**

Rationale:
- `OffloadHandle`, `Offloadable`, `OffloadSerializable`, and `HandleKey` are semantically
  queue-layer types — they define the enqueue contract and the identity of the enqueued work.
- `ferro-queue/src/lib.rs` already exports `Job`, `PendingDispatch`, `Queueable`, etc. Adding the
  new types to the same crate keeps the dependency graph clean.
- The `framework/src/lib.rs` re-export under `mod queue { pub use ferro_queue::{...} }` is already
  the established mechanism for surfacing queue types as `::ferro::queue::*`. No structural change
  to `framework` is needed — only additions to the `pub use` list.
- Placing them in `framework/src/` would require `ferro-queue` to depend on `framework` (or vice
  versa), creating a cycle. Placing them in `ferro-queue` is acyclic.

`HandleKey` should also live in `ferro-queue/src/offload.rs` rather than a separate file, since
it is used exclusively as the key inside `OffloadHandle`.

---

## `OffloadHandle<T>` Serde Derivation Decision (Claude's Discretion)

**Recommendation: derive `Serialize, Deserialize, Clone, Debug` on `OffloadHandle<T>`.**

Justification:
- `#[serde(skip)]` on the phantom field makes derivation unconditional on `T`.
- Phase 247 requires the handle to travel to the client as the subscription key; serde is required
  for that transport.
- `Clone` is required for the handle to be used after `.offload()` without consuming it.
- `Debug` is standard on all public framework types.
- `PartialEq, Eq` are worth including for test assertions (comparing handles returned by mocked
  dispatches).

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies beyond the project's existing workspace; uuid, serde,
trybuild, and the Rust toolchain are already present and verified).

---

## Validation Architecture

`config.json` does not set `workflow.nyquist_validation` (key absent) — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `trybuild 1` (UI tests) + `cargo test --all-features` (unit tests) |
| Config file | `ferro-macros/Cargo.toml` (dev-dependency: `trybuild = "1"`) |
| Quick run command | `cargo test -p ferro-macros --test offload_macro` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OFFLOAD-02a | `.offload()` returns `OffloadHandle<T>` typed on success type | Unit / compile-pass | `cargo test -p ferro-macros --test offload_macro` (pass fixture) | ❌ Wave 0 — new pass fixture |
| OFFLOAD-02b | Non-serializable parameter fails with branded diagnostic | Compile-fail (trybuild) | `cargo test -p ferro-macros --test offload_macro` | ❌ Wave 0 — `non_serializable_param.rs` + `.stderr` |
| OFFLOAD-02c | Non-serializable return type fails with branded diagnostic | Compile-fail (trybuild) | `cargo test -p ferro-macros --test offload_macro` | ❌ Wave 0 — `non_serializable_return.rs` + `.stderr` |
| OFFLOAD-02d | `OffloadHandle<T>.key()` returns the UUID string | Unit | `cargo test -p ferro-queue` (unit test in offload.rs) | ❌ Wave 0 — unit test on `HandleKey` |
| OFFLOAD-02e | `OffloadHandle<T>` serializes / deserializes regardless of `T: !Serialize` | Unit | `cargo test -p ferro-queue` | ❌ Wave 0 — serde round-trip test |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-macros --test offload_macro && cargo test -p ferro-queue`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-queue/src/offload.rs` — new module (OffloadHandle, HandleKey, Offloadable, OffloadSerializable)
- [ ] `ferro-macros/tests/ui/offload/fail/non_serializable_param.rs` + `.stderr`
- [ ] `ferro-macros/tests/ui/offload/fail/non_serializable_return.rs` + `.stderr`
- [ ] `ferro-macros/tests/ui/offload/pass/` — one new pass fixture proving `.offload()` return type
- [ ] Unit tests in `ferro-queue/src/offload.rs` (`HandleKey::new()` is UUID, serde round-trip)

---

## Security Domain

This phase introduces no authentication, authorization, session management, input validation from
untrusted sources, cryptographic operations, or data persistence. The enforcement is purely at
compile time. ASVS categories V2–V6 do not apply to a proc-macro extension that produces type
bounds. No security section needed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `{Self}` interpolation in `#[diagnostic::on_unimplemented]` expands to the concrete failing type | Verification — Unknown 1 | Message would omit the type name; SC#2 "type-naming message" requirement unmet |
| A2 | The `where`-clause bound on the struct fires the branded message *before* or *alongside* serde's own error, making it visible in the trybuild output | Verification — Unknown 2 | The branded message may be buried under serde errors; SC#2 still technically met (message present) but less prominent |
| A3 | `PhantomData<fn() -> T>` makes `OffloadHandle<T>: Send + Sync` unconditionally | Code Examples — Pattern 3 | If wrong, async contexts with `T: !Send` break; use `PhantomData<T>` + explicit unsafe impls instead |

**Items A1 and A2** should be validated by running `TRYBUILD=overwrite` on the new fail fixtures as
Wave 0 work and inspecting the captured `.stderr` before committing.

---

## Open Questions

1. **Does the `#[diagnostic::on_unimplemented]` message appear as the primary error line or only as a note?**
   - What we know: the attribute is intended to replace the default message for `E0277`.
   - What's unclear: whether rustc emits the custom `message` as the primary error line or
     demotes it to a note when other errors also fire (e.g., serde-derive failures on the same type).
   - Recommendation: generate via `TRYBUILD=overwrite` and inspect; if the message is demoted,
     consider whether serde-derive's bound should be suppressed by emitting `#[serde(bound = "")]`
     on the struct so only `OffloadSerializable` fires.

2. **Should the `Offloadable::offload()` provided default be `async fn` (requiring `#[async_trait]`) or return a boxed future?**
   - What we know: `ferro-queue::Job` already uses `#[async_trait]`. The 245 CONTEXT.md uses
     `async fn offload()` syntax.
   - What's unclear: whether `#[async_trait]` must be applied to `Offloadable` itself, and whether
     this affects macro emission.
   - Recommendation: apply `#[async_trait]` to `Offloadable` (same pattern as `Job`), which makes
     the provided default body work correctly with `.await` internally.

---

## Sources

### Primary (HIGH confidence)

- `ferro-macros/src/offload.rs` — read verbatim; all 244 function shapes, `OffloadMethodInfo` fields,
  and emitted paths confirmed
- `ferro-macros/src/service.rs` — read verbatim; confirms collect/emit wiring, lines 183–254
- `ferro-macros/tests/offload_macro.rs` — harness structure confirmed
- `ferro-macros/tests/ui/offload/fail/mut_ref_param.rs` + `.stderr` — fail fixture convention confirmed
- `ferro-macros/tests/ui/offload/pass/basic.rs`, `ref_str_param.rs`, `result_method.rs` — pass
  fixture convention confirmed
- `ferro-queue/src/job.rs` — `Job` trait shape (L44), `idempotency_key` (L86), `uuid` usage confirmed
- `ferro-queue/src/dispatcher.rs` — `PendingDispatch<J>` (L26), `.dispatch()` async (L85),
  bounds `J: Job + Serialize + DeserializeOwned` (L36) confirmed
- `ferro-queue/src/lib.rs` — full public API confirmed; `Queueable` blanket impl pattern noted
- `ferro-queue/Cargo.toml` — `uuid = { version = "1", features = ["v4", "serde"] }` confirmed
- `framework/src/lib.rs` lines 223–231 — `mod queue` re-export block confirmed
- `framework/Cargo.toml` — `uuid = { version = "1", features = ["v4"] }` confirmed
- `rust-toolchain.toml` — `channel = "1.94.1"` confirmed
- `rustc --version` — `rustc 1.94.1` confirmed locally

### Secondary (MEDIUM confidence)

- Rust Reference on `#[diagnostic::on_unimplemented]`: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-diagnosticon_unimplemented-attribute — confirms `{Self}` interpolation and stabilization
- RFC 3368 (diagnostic tool attribute namespace) — stabilized in Rust 1.78.0

### Tertiary (LOW confidence — ASSUMED)

- A3: `PhantomData<fn() -> T>` `Send + Sync` behavior described from training knowledge; should
  be verified by the implementer with a quick test if uncertain.

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new dependencies; all existing crates verified in tree
- Architecture: HIGH — all files read verbatim; patterns derived from actual 244 code
- `#[diagnostic::on_unimplemented]` mechanics: HIGH (stabilization) / MEDIUM (exact stderr format)
- Trybuild harness extension: HIGH — harness file read, pattern confirmed from existing fixtures
- Parameter-side enforcement: MEDIUM — the `where`-clause approach is sound, but exact stderr
  appearance with serde errors co-present requires Wave 0 validation
- Pitfalls: HIGH — derived from concrete code analysis

**Research date:** 2026-08-13
**Valid until:** Stable for the pinned 1.94.1 toolchain; revisit if toolchain is bumped.
