---
phase: 259-request-scoped-memoization
plan: "01"
subsystem: framework
tags: [memoization, task-local, request-scoped, coalescing, futures]
dependency_graph:
  requires: []
  provides:
    - "ferro::MemoStore — per-request coalescing memo store"
    - "ferro::MemoKey — (TypeId callsite, u64 args_hash) key"
    - "ferro::current_memo_store() — ambient store reader, None outside scope"
    - "MEMO_STORE task-local — scoped per request in server.rs"
  affects:
    - framework/src/server.rs (per-request scope entry added)
    - framework/src/lib.rs (pub mod memo + pub use memo::)
tech_stack:
  added:
    - "futures = { version = \"0.3\", default-features = false, features = [\"std\"] } — provides futures::future::Shared for coalescing"
  patterns:
    - "tokio::task_local! bare-value shape (mirrors REQUEST_HOST: String, not RwLock<Option<T>>)"
    - "MEMO_STORE.try_with(|s| s.clone()).ok() — graceful None outside scope (D-02)"
    - "Mutex guard released in block before .await (Pitfall 1 avoided)"
    - "Shared<BoxFuture<Arc<dyn Any + Send + Sync>>> — type-erased coalescing slot"
key_files:
  created:
    - framework/src/memo/mod.rs
  modified:
    - framework/Cargo.toml
    - framework/src/lib.rs
    - framework/src/server.rs
decisions:
  - "Used Arc<MemoStore> directly in task-local (not Arc<RwLock<Option<T>>>) — store is always present once scoped, matching REQUEST_HOST shape"
  - "memo_scope() / with_memo_scope() kept pub(crate) with #[allow(dead_code)] — used by tests and future plans, not yet called from server.rs directly"
  - "server.rs inlines Arc::new(MemoStore::new()) at scope entry rather than calling memo_scope() — keeps the scope entry readable inline"
  - "Fallback handler path intentionally not wrapped (Assumption A6 / D-02)"
  - "pub use memo:: placed before pub use sea_orm:: in lib.rs to satisfy rustfmt alphabetical ordering"
metrics:
  duration: "~10 minutes"
  completed: "2026-07-21"
  tasks: 2
  files: 4
requirements: [LIVE-01]
---

# Phase 259 Plan 01: MemoStore + MEMO_STORE Task-Local Summary

Request-scoped memo store with `futures::future::Shared` coalescing, held in a `tokio::task_local!`, scoped per request in `server.rs`, re-exported at `::ferro::memo`.

## What Was Built

**Task 1 — MemoStore + MemoKey + MEMO_STORE task-local + futures dep** (`937e2142`)

- `framework/Cargo.toml`: added `futures = { version = "0.3", default-features = false, features = ["std"] }` (already in Cargo.lock via ferro-queue; no new download).
- `framework/src/memo/mod.rs` (new, 290 lines): complete implementation of the `<interfaces>` contract:
  - `MEMO_STORE: Arc<MemoStore>` task-local via `tokio::task_local!`
  - `MemoKey { callsite: TypeId, args_hash: u64 }` with `MemoKey::new::<Marker, A>(args)`
  - `MemoStore { entries: Mutex<HashMap<MemoKey, MemoSlot>> }` with `get_or_insert` that releases the lock before returning (Pitfall 1 — no guard held across `.await`)
  - `MemoSlot = Shared<BoxFuture<'static, Arc<dyn Any + Send + Sync>>>`
  - `current_memo_store() -> Option<Arc<MemoStore>>` via `try_with(...).ok()`
  - `memo_scope()` / `with_memo_scope()` pub(crate) helpers
  - `impl Default for MemoStore` (satisfies clippy `new_without_default`)
  - 7 unit tests covering all six required behaviors
- `framework/src/lib.rs`: `pub mod memo;` added alongside other `pub mod` declarations.

**Task 2 — MEMO_STORE scope in server.rs + re-export** (`55816dc4`)

- `framework/src/server.rs`: primary handler chain now wrapped in `MEMO_STORE.scope(Arc::new(MemoStore::new()), ...)` nested inside the existing `REQUEST_HOST.scope`; fallback path at ~line 315 carries an explicit exemption comment (D-02 intentional).
- `framework/src/lib.rs`: `pub use memo::{current_memo_store, MemoKey, MemoStore};` in the re-export block, ordered before `pub use sea_orm::` per rustfmt.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo test -p ferro-rs --lib memo` | 7/7 pass (hit, miss, coalesce, out-of-scope, err-cached, drop, scope-helper) |
| `cargo build -p ferro-rs` | OK |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy -p ferro-rs --all-targets -- -D warnings` | OK (0 errors) |
| `cargo doc -p ferro-rs --no-deps` | OK (0 warnings) |
| `grep MEMO_STORE.scope framework/src/server.rs` | line 283 — primary path scoped |
| `grep "pub use memo::" framework/src/lib.rs` | line 128 — public API re-exported |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Rustdoc broken intra-doc link on `MEMO_STORE`**
- **Found during:** Task 2 doc build
- **Issue:** `[`MEMO_STORE`]` in the `//!` module doc was parsed as an intra-doc link; `MEMO_STORE` is `pub(crate)` so rustdoc emitted `warning: unresolved link to MEMO_STORE`
- **Fix:** Changed `[`MEMO_STORE`]` to plain backtick `` `MEMO_STORE` `` (not a link — just inline code)
- **Files modified:** `framework/src/memo/mod.rs`
- **No separate commit** — folded into Task 2 commit

**2. [Rule 1 - Bug] rustfmt ordering — `pub use memo::` after `pub use sea_orm::`**
- **Found during:** Task 2 fmt check
- **Issue:** First placement of `pub use memo::` was after `pub use sea_orm::`, which violates rustfmt's alphabetical ordering (`m` < `s`)
- **Fix:** Moved `pub use memo::` to appear before `pub use sea_orm::`
- **Files modified:** `framework/src/lib.rs`
- **No separate commit** — folded into Task 2 commit

**3. [D-02 preservation] `memo_scope()` / `with_memo_scope()` not called from server.rs**
- The plan's `<interfaces>` block lists both as `pub(crate)`. The server.rs scope entry inlines `Arc::new(crate::memo::MemoStore::new())` directly rather than calling `memo_scope()` — this is consistent with how `tenant/context.rs` helpers are also unused in some cases (annotated `#[allow(dead_code)]`).
- Added `#[allow(dead_code)]` with a comment noting they are used by Plan 02/03.
- Plan 02 (the `#[memoize]` macro) will call `current_memo_store()` directly; `memo_scope()` / `with_memo_scope()` may be used in Plan 03 integration tests.

## Known Stubs

None. All public API is fully implemented and tested. The `memo_scope()` and `with_memo_scope()` helpers are complete but currently `#[allow(dead_code)]` until consumed by Plan 02/03.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The `T-259-01` (cross-request isolation) mitigation is proven by the `dropped_store_has_no_prior_entries` test.

## Self-Check: PASSED

- `framework/src/memo/mod.rs` exists: FOUND
- `framework/Cargo.toml` contains `futures = { version = "0.3"`: FOUND
- `framework/src/lib.rs` contains `pub mod memo`: FOUND  
- `framework/src/lib.rs` contains `pub use memo::`: FOUND
- `framework/src/server.rs` contains `MEMO_STORE.scope`: FOUND
- Commit `937e2142` exists: FOUND
- Commit `55816dc4` exists: FOUND
- All 7 memo unit tests pass: VERIFIED
