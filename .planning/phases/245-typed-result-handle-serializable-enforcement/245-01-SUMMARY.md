---
phase: 245-typed-result-handle-serializable-enforcement
plan: "01"
subsystem: ferro-queue
tags: [offload, typed-handle, serializable-contract, compile-time-enforcement]
dependency_graph:
  requires: [244-01, 244-02]
  provides: [ferro-queue::OffloadSerializable, ferro-queue::HandleKey, ferro-queue::OffloadHandle, ferro-queue::Offloadable]
  affects: [framework/src/lib.rs, ferro-queue/src/lib.rs]
tech_stack:
  added: []
  patterns:
    - "PhantomData<fn() -> T> for Send+Sync phantom regardless of T"
    - "#[serde(skip)] on phantom field to decouple T's serde bounds from the wrapper"
    - "#[diagnostic::on_unimplemented] for branded compile-time error messages"
    - "Default delegating to new() to satisfy clippy::new_without_default"
key_files:
  created:
    - ferro-queue/src/offload.rs
  modified:
    - ferro-queue/src/lib.rs
    - framework/src/lib.rs
decisions:
  - "Offloadable supertrait bounds are: crate::Job + Serialize + DeserializeOwned + Sized — placed at the trait level so the macro can emit bare impl Offloadable for XJob { type Output = ...; } with no extra where-clause"
  - "serde_json is already a regular (non-dev) dependency of ferro-queue — no new dependency needed for the test's serde_json::to_string / from_str calls"
  - "Rustfmt requires alphabetical order within pub use lists — OffloadSerializable before Offloadable in both ferro-queue/src/lib.rs and framework/src/lib.rs"
metrics:
  duration_seconds: 206
  completed_date: "2026-08-13T15:02:20Z"
  tasks_completed: 2
  files_changed: 3
requirements: [OFFLOAD-02]
---

# Phase 245 Plan 01: Offload Types Foundation — Summary

Four offload primitives added to `ferro-queue` and surfaced through `::ferro::queue::*`.

## One-liner

`OffloadSerializable` marker trait with `#[diagnostic::on_unimplemented]` + `HandleKey` UUID v4 newtype + `OffloadHandle<T>` with `fn() -> T` phantom and `#[serde(skip)]` + `Offloadable` async trait with provided `offload()` default.

## Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create ferro-queue/src/offload.rs with four types + tests | 327a1b3c | ferro-queue/src/offload.rs, ferro-queue/src/lib.rs |
| 2 | Wire re-exports through ::ferro::queue | 8c7fa626 | framework/src/lib.rs |

## Verification Results

- `cargo test -p ferro-queue offload::tests` — 2/2 passed (OFFLOAD-02d + OFFLOAD-02e)
- `cargo build -p ferro-rs -p ferro-queue` — exit 0
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-rs -p ferro-queue --all-targets -- -D warnings` — clean

## Decisions for Plan 02

**Offloadable supertrait bounds (Plan 02 macro note):**

The `Offloadable` trait is declared as:
```rust
pub trait Offloadable: crate::Job + Serialize + DeserializeOwned + Sized { ... }
```
All three extra bounds (`Serialize`, `DeserializeOwned`, `Sized`) are at the trait level — not on `offload()` itself — because `PendingDispatch::new` requires them at its `impl<J> PendingDispatch<J> where J: Job + Serialize + DeserializeOwned` level. This placement means the macro can emit exactly:
```rust
impl ::ferro::queue::Offloadable for XJob {
    type Output = SomeType;
}
```
with no additional `where`-clause. The `Sized` bound is required because `offload(self)` takes `self` by value and `async_trait` boxes the future.

**serde_json dev-dependency status:**

`serde_json` is a regular (non-dev) dependency in `ferro-queue/Cargo.toml` (version `"1"`, no `dev-dependencies` entry needed). The test's `serde_json::to_string` and `from_str` calls compile without any Cargo.toml change.

## Deviations from Plan

### Auto-fixed issues

**1. [Rule 1 - Bug] Rustfmt alphabetical ordering in pub use lists**
- **Found during:** Task 1 commit preparation / Task 2 commit preparation
- **Issue:** Rustfmt requires identifiers in `pub use` lists to be alphabetically sorted. Both initial edits placed `Offloadable` before `OffloadSerializable`, which fmt rejected.
- **Fix:** Reordered to `OffloadSerializable, Offloadable` in both `ferro-queue/src/lib.rs` and `framework/src/lib.rs`.
- **Files modified:** ferro-queue/src/lib.rs, framework/src/lib.rs
- **Commits:** included in 327a1b3c and 8c7fa626

## Known Stubs

None. The four types are fully implemented for Phase 245 scope. `OffloadHandle<T>` is intentionally inert (no resolve/subscribe surface) by design decision D-08; Phases 246 and 247 add those methods.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes. The surface is new Rust types with compile-time bounds only, matching the plan's threat model (T-245-01 through T-245-03 all accepted or mitigated at compile time).

## Self-Check: PASSED

- ferro-queue/src/offload.rs: FOUND
- ferro-queue/src/lib.rs contains `mod offload;`: FOUND
- ferro-queue/src/lib.rs contains `pub use offload::`: FOUND
- framework/src/lib.rs contains `OffloadHandle`: FOUND
- Commit 327a1b3c: FOUND
- Commit 8c7fa626: FOUND
- cargo test -p ferro-queue offload::tests: 2 passed
- cargo build -p ferro-rs -p ferro-queue: exit 0
