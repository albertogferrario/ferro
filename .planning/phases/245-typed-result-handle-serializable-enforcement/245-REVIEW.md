---
phase: 245-typed-result-handle-serializable-enforcement
reviewed: 2026-08-13T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - ferro-queue/src/offload.rs
  - ferro-queue/src/lib.rs
  - framework/src/lib.rs
  - ferro-macros/src/offload.rs
  - ferro-macros/tests/ui/offload/pass/offload_handle.rs
  - ferro-macros/tests/ui/offload/fail/non_serializable_param.rs
  - ferro-macros/tests/ui/offload/fail/non_serializable_return.rs
  - docs/src/features/queues.md
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 245: Code Review Report

**Reviewed:** 2026-08-13
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

The phase adds a typed offload result handle (`OffloadHandle<T>`), a compile-time
serializable marker (`OffloadSerializable`), an opaque enqueue identity
(`HandleKey`), and an `Offloadable` enqueue trait, together with the macro
extension that emits `type Output` and the per-field `OffloadSerializable`
`where`-clause. The core design goals verify out:

- **Path hygiene is clean.** Every emitted path in `emit_job_items` resolves
  through `::ferro::queue::*`, `::ferro::async_trait`, `::ferro::App`,
  `::ferro::inventory`, `::serde::*`, `::std::result::Result` — no bare
  `ferro_queue::` path is emitted. Confirmed against `emit_job_items`
  (ferro-macros/src/offload.rs:281-329).
- **`Result<T, E>` success-type extraction is correct.** `collect_info` matches
  the last path segment `Result`, then takes the first `GenericArgument::Type`
  via `find_map`, which yields `T` and never `E`
  (ferro-macros/src/offload.rs:174-202). `E` is never bound to
  `OffloadSerializable` anywhere — the struct `where`-clause bounds only field
  types and `#output_type`, and the `Offloadable` impl bounds only
  `#output_type` (ferro-macros/src/offload.rs:293-327). The error path
  string-serializes `E` via `format!("{e}")` (offload.rs:257-272), matching the
  documented contract.
- **The `PhantomData<fn() -> T>` handle is `Send + Sync` regardless of `T`**, and
  the `#[serde(skip)]` on the phantom correctly drops the serde `T: Serialize`
  requirement, so the handle round-trips when `T: !Serialize`. The two in-module
  tests exercise exactly these properties and are well targeted
  (offload.rs:125-151).
- **`HandleKey` uses `Uuid::new_v4()`** and the version is asserted in the test
  (offload.rs:45-47, 130-135). No `unwrap()`/`expect()`/`panic!` in the
  non-test library code. The only `.expect()` in emitted code is the container
  lookup in the derived `handle()`, which predates this phase and carries a
  descriptive message.

Two warnings concern soundness of the derived-trait bounds, not correctness of
the happy path; three info items are minor.

## Warnings

### WR-01: `OffloadHandle<T>` leaks `T: Clone / Debug / PartialEq / Eq` bounds through std derives

**File:** `ferro-queue/src/offload.rs:71`
**Issue:** The handle derives `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]`.
`#[serde(skip)]` correctly suppresses the *serde* bound on `T`, but the four
standard derives (`Clone`, `Debug`, `PartialEq`, `Eq`) still synthesize
`impl<T: Clone> Clone`, `impl<T: Debug> Debug`, etc. — the classic std-derive
over-constraint. The presence of a `PhantomData<fn() -> T>` field does not
exempt `T`: std derives bound every generic parameter unconditionally. Verified
empirically — `OffloadHandle<SerOnly>::clone()` fails to resolve when `SerOnly`
is `Serialize + DeserializeOwned` but not `Clone`.

The consequence: the trait bound the phase actually enforces is
`T: OffloadSerializable`, i.e. `Serialize + DeserializeOwned` only. A success
type that is serde-serializable but not `Clone`/`Eq`/`Debug` (common — e.g. a
`Report` containing an `f64`, which is not `Eq`; or any type deliberately left
non-`Clone`) yields an `OffloadHandle<T>` that cannot be cloned, debugged, or
compared. This partially contradicts the module docstring's stated intent that
the handle work "regardless of `T`" (offload.rs:68-70). It also means
`OffloadHandle<Report>` in the pass test only compiles because `Report` happens
to derive all four; a serde-only success type would compile at the `.offload()`
call but produce a handle missing these impls.

**Fix:** Add explicit hand-written impls that do not bound `T`, or use a derive
helper that honors the phantom. Minimal hand-written form:
```rust
#[derive(Serialize, Deserialize)]
pub struct OffloadHandle<T> {
    key: HandleKey,
    #[serde(skip)]
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Clone for OffloadHandle<T> {
    fn clone(&self) -> Self {
        Self { key: self.key.clone(), _phantom: PhantomData }
    }
}
impl<T> std::fmt::Debug for OffloadHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OffloadHandle").field("key", &self.key).finish()
    }
}
impl<T> PartialEq for OffloadHandle<T> {
    fn eq(&self, other: &Self) -> bool { self.key == other.key }
}
impl<T> Eq for OffloadHandle<T> {}
```
(The serde `#[derive(Serialize, Deserialize)]` may stay — `#[serde(skip)]`
already frees it from `T` bounds.) If cross-`T` `Clone`/`Eq` genuinely is not
required in Phase 245, then at minimum tighten the docstring to say the handle
round-trips (serde) regardless of `T` but that `Clone`/`Eq`/`Debug` still
require them on `T`, so the claim and the code agree.

### WR-02: `Offloadable::offload()` mints a `HandleKey` that never travels with the job

**File:** `ferro-queue/src/offload.rs:118-122`
**Issue:** `offload()` mints a fresh `HandleKey`, then enqueues via
`PendingDispatch::new(self).dispatch()` and returns `OffloadHandle::new(key)`.
The key is generated *after* / independently of the enqueue and is never written
into the job payload or the queue row. The returned handle therefore identifies
a key that the eventual worker has no way to observe: when the job runs on a
background worker, nothing in the persisted payload carries this key, so a future
result-path (Phase 246) that resolves by `handle.key()` will not be able to
correlate the worker's output back to the handle the caller holds.

This may be an intentional Phase-245 seam — the docstring frames the handle as
"inert" and defers resolve/subscribe to 246/247 — but as written the key is a
local random string with no persisted linkage, which is a latent correctness gap
the result-path phase will have to unwind rather than build on. Note also that
enqueue and key-minting are not atomic: `dispatch()` can succeed while the
returned key is unrelated to any stored identity.

**Fix:** Either (a) thread the minted key into the enqueue so it is persisted
with the job (e.g. carry it on the payload / a dedicated column) before the
result path is built, or (b) if deferring is deliberate, add an explicit note on
`offload()` that the key is not yet persisted and carries no worker-side linkage
in Phase 245, so Phase 246 does not assume the handle key is already durable.
Minting the key *before* dispatch and passing it through also removes the
non-atomicity between enqueue success and handle identity.

## Info

### IN-01: Duplicated doc-comment `/// Re-export async_trait for convenience` on unrelated items

**File:** `ferro-queue/src/lib.rs:71,74`
**Issue:** The doc comment above the `offload::*` re-export (line 71-72) reads
"Re-export async_trait for convenience" but the item re-exports
`HandleKey, OffloadHandle, OffloadSerializable, Offloadable`. It is a copy of the
comment on the genuine `async_trait` re-export directly below (line 74-75). The
label is inaccurate for the offload re-export.
**Fix:** Change line 71 to describe the offload surface, e.g.
`/// Typed offload handle and serializable-contract primitives.`

### IN-02: `HandleKey` docstring cross-links `Job::idempotency_key()` which is a method, not an associated path

**File:** `ferro-queue/src/offload.rs:38`
**Issue:** The intra-doc link `[`Job::idempotency_key()`](crate::Job::idempotency_key)`
targets `crate::Job::idempotency_key`. Trait-method intra-doc links resolve, but
CI runs `cargo doc -Dwarnings` (per project conventions), so if the method is
provided-with-default and the link path is slightly off it will surface as a doc
warning at build time. Worth a quick `cargo doc --no-deps` confirmation rather
than assuming it resolves.
**Fix:** Verify with `cargo doc --no-deps -p ferro-queue` that no broken-intra-doc-link
warning is emitted; adjust to `crate::job::Job::idempotency_key` if needed.

### IN-03: Docs example shows `.offload()` on a hand-constructed Job struct, but the authoring surface implies the call originates from the service method

**File:** `docs/src/features/queues.md:216-223`
**Issue:** The typed-handle example constructs
`ReportsServiceBuildMonthlyJob { tenant_id, month }.offload()`. This is
technically accurate against the emitted API (the struct fields are `pub` and
`Offloadable::offload` is a provided default), and matches the pass test's
construction. It is a minor prose/mental-model mismatch only: the surrounding
narrative frames the trait method as "the single source of truth", yet the
example bypasses the method and instantiates the derived struct by hand. Not a
technical inaccuracy — the signature `.offload() -> Result<OffloadHandle<T>, Error>`
matches offload.rs:118 exactly.
**Fix:** Optional — add one line clarifying that the derived struct is the
enqueue vehicle and that a future phase may add a method-level `.offload()`
sugar, so readers do not expect `reports_service.build_monthly(..).offload()`.

---

_Reviewed: 2026-08-13_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
