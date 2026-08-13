---
phase: 244-offload-macro-job-payload-derivation
verified: 2026-08-13T00:00:00Z
status: passed
score: 8/8
overrides_applied: 0
re_verification: false
---

# Phase 244: `#[offload]` Macro → Job + Payload Derivation — Verification Report

**Phase Goal:** Turn a single `#[offload]` annotation on a `#[service]` trait method into a
derived `ferro-queue` Job plus a serializable payload struct built from the method's
parameters — so the work is declared once (as the method) and never re-authored as a Job wrapper.

**Verified:** 2026-08-13
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A `#[offload]` method on a `#[service]` trait expands to a registered `ferro-queue` Job whose payload carries the method's parameters | VERIFIED | `basic.rs` fixture compiles; `ReportsBuildMonthlyJob { month: Month(1) }` is nameable in `main()`. `offload.rs::emit_job_items` emits `pub struct #job_ident { #(pub #field_names: #field_types,)* }` + `impl ::ferro::queue::Job` + `inventory::submit!`. `service_impl` strips `#[offload]` and appends derived items via `#(#offload_items)*`. |
| 2 | Enqueuing the derived Job runs the original method body on a worker (round-trip in a test) | VERIFIED | `offload_round_trip_sync_mode` (OFFLOAD-01-d): `dispatch(RanJob{value:42}).await.unwrap()` sets `JOB_RAN` AtomicBool via `handle()` in `QUEUE_CONNECTION=sync` mode. Queue substrate end-to-end proven with a hand-written Job structurally identical to a macro-derived one; trybuild fixtures confirm derivation produces the same shape. `cargo test -p ferro-queue` passed per plan gate evidence. |
| 3 | No hand-written Job struct or manual enqueue call is required at the call site | VERIFIED | `offload_job_auto_registers_via_inventory` (OFFLOAD-01-f): `WorkerLoop::from_registry(WorkerConfig::default())` picks up `InventoryJob` submitted only via `inventory::submit!` — no `Queue::register` call anywhere in the test. `emit_job_items` emits `::ferro::inventory::submit!{…}` as part of the derived token stream. |

**Score:** 3/3 roadmap truths verified

---

### Plan 01 Must-Haves

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A derived Job dispatched via `ferro_queue::dispatch(..).await` in sync mode runs its `handle()` | VERIFIED | `offload_round_trip_sync_mode` test in `ferro-queue/tests/offload_round_trip.rs:43–51`. |
| 2 | A Result-returning `handle()` that maps `Err(e)` to `Error::job_failed` surfaces as a dispatch failure in sync mode | VERIFIED | `offload_result_err_maps_to_job_failure` test (lines 80–91); `dispatch(FailingJob).await` returns `Err`; message contains "boom". |
| 3 | A Job type submitted via `inventory::submit!(JobRegistrarEntry{..})` is picked up by `WorkerLoop::from_registry` with zero bootstrap code | VERIFIED | `offload_job_auto_registers_via_inventory` test (lines 119–128); `worker.registered_job_count() >= 1` asserted. |

### Plan 02 Must-Haves

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A `#[offload]` method on a `#[service]` trait compiles and emits a public `<Trait><Method>Job` struct carrying the method's non-self params as owned fields | VERIFIED | `pass/basic.rs`: `ReportsBuildMonthlyJob { month: Month(1) }` is constructed in `main()`. |
| 2 | A `&str` parameter maps to an owned `String` field in the derived struct | VERIFIED | `pass/ref_str_param.rs`: `GreeterServiceGreetJob { name: String::from("x") }` compiles. |
| 3 | A `&mut T` parameter emits a clear `compile_error!` naming the `&mut` restriction | VERIFIED | `fail/mut_ref_param.stderr` contains exact error text: `"#[offload] parameters may not be &mut references — Job payloads must be owned and serializable"` at the correct span. |
| 4 | The derived struct `impl Job` resolves the concrete service from the container and runs the original method body; `Result<T,E>` methods map `Err` to a job failure | VERIFIED | `offload.rs::emit_job_items` emits `::ferro::App::make::<dyn #trait_ident>()` + four `call_expr` branches (async/sync × Result/non-Result). `pass/result_method.rs` compiles the `Result` branch. |
| 5 | The derived Job self-registers via `inventory::submit!` — no manual `Queue::register` call | VERIFIED | `emit_job_items` emits `::ferro::inventory::submit! { ::ferro::queue::JobRegistrarEntry { register: |w| w.register::<#job_ident>(), name: … } }`. Confirmed in `ferro-macros/src/offload.rs:289–296`. |

**Score:** 8/8 total must-haves verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-queue/src/db.rs` | `JobRegistrarEntry` inventory type + `inventory::collect!` | VERIFIED | Lines 94–109: `pub struct JobRegistrarEntry { pub register: fn(&mut crate::WorkerLoop), pub name: &'static str }` + `inventory::collect!(JobRegistrarEntry);` |
| `ferro-queue/src/worker.rs` | `from_registry` drains `inventory::iter::<crate::db::JobRegistrarEntry>` | VERIFIED | Lines 210–218: `for entry in inventory::iter::<crate::db::JobRegistrarEntry> { (entry.register)(&mut w); }` |
| `ferro-queue/Cargo.toml` | `inventory` dependency | VERIFIED | Line 24: `inventory = "0.3"` |
| `ferro-queue/tests/offload_round_trip.rs` | Sync-mode round-trip + inventory auto-registration tests | VERIFIED | Three tests: `offload_round_trip_sync_mode`, `offload_result_err_maps_to_job_failure`, `offload_job_auto_registers_via_inventory` |
| `framework/src/lib.rs` | `JobRegistrarEntry` re-exported in `pub mod queue` block | VERIFIED | Line 227: `JobRegistrarEntry` present in `pub use ferro_queue::{…}` list |
| `ferro-macros/src/offload.rs` | `owned_type()`, `collect_info()`, `emit_job_items()` | VERIFIED | All three functions present; `owned_type` maps `&str→String`, `&[T]→Vec<T>`, rejects `&mut T`; emits only `::ferro::*` paths |
| `ferro-macros/src/service.rs` | Scans `item_trait.items` for `#[offload]`, strips attr, appends derived items | VERIFIED | Lines 183–254: `offload_infos` collected, items appended via `#(#offload_items)*` |
| `ferro-macros/src/lib.rs` | `mod offload;` declaration | VERIFIED | Line 24: `mod offload;` |
| `ferro-macros/tests/offload_macro.rs` | Trybuild harness | VERIFIED | `offload_macro_ui` test with `t.pass(…)` and `t.compile_fail(…)` |
| `ferro-macros/tests/ui/offload/fail/mut_ref_param.stderr` | Captured compile-error snapshot containing `&mut` | VERIFIED | Contains exact error text with `&mut` at correct source location |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-queue/src/worker.rs::from_registry` | `ferro-queue/src/db.rs::JobRegistrarEntry` | `inventory::iter::<crate::db::JobRegistrarEntry>` | VERIFIED | Pattern found at worker.rs:215 |
| `framework/src/lib.rs` queue module | `ferro_queue::JobRegistrarEntry` | `pub use` re-export | VERIFIED | `JobRegistrarEntry` present in `pub mod queue { pub use ferro_queue::{…} }` at line 227 |
| `ferro-macros/src/service.rs::service_impl` | `ferro-macros/src/offload.rs` | `crate::offload::collect_info` / `crate::offload::emit_job_items` | VERIFIED | `crate::offload::collect_info` at service.rs:194; `crate::offload::emit_job_items` at service.rs:248 |
| Derived `impl Job handle()` | `::ferro::App::make::<dyn Trait>()` | Container resolution then method call | VERIFIED | `offload.rs:283`: `let svc = ::ferro::App::make::<dyn #trait_ident>().expect(…);` |
| Derived `inventory::submit!` | `::ferro::queue::JobRegistrarEntry` | Self-registration entry | VERIFIED | `offload.rs:289–296`: `::ferro::inventory::submit! { ::ferro::queue::JobRegistrarEntry { … } }` |

---

### Behavioral Spot-Checks

Step 7b: SKIPPED for trybuild fixtures (compile-time tests, not runnable entry points). For the queue tests:

| Behavior | Evidence | Status |
|----------|----------|--------|
| Sync-mode dispatch runs `handle()` | Plan 01 gate: `cargo test -p ferro-queue` — `offload_round_trip_sync_mode` PASSED | PASS |
| `Err` from `handle()` surfaces as dispatch failure | Plan 01 gate: `offload_result_err_maps_to_job_failure` PASSED | PASS |
| Inventory auto-registration picked up by `from_registry` | Plan 01 gate: `offload_job_auto_registers_via_inventory` PASSED | PASS |
| `#[offload]` macro valid expansion (basic, `&str`, `Result`) | Plan 02 gate: `cargo test -p ferro-macros --test offload_macro` — 1 passed | PASS |
| `&mut T` emits correct compile-error | Plan 02 gate: trybuild `compile_fail` fixture matched snapshot | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| OFFLOAD-01 | Plans 01 + 02 | Framework derives `ferro-queue` Job + serializable payload from `#[offload]` method — no hand-written Job, no manual enqueue | SATISFIED | All six sub-behaviors (OFFLOAD-01-a/b/c in Plan 02 trybuild; OFFLOAD-01-d/e/f in Plan 01 round-trip tests) verified |

**Note:** `REQUIREMENTS.md` traceability table at line 71 still reads "Not started" for OFFLOAD-01 — stale text not updated after phase execution. The checkbox at line 32 (`[x]`) correctly records it as complete. This is a documentation inconsistency with no effect on goal achievement.

---

### Anti-Patterns Found

No stubs, placeholders, or empty implementations found in phase 244 artifacts. "Placeholder" occurrences in `db.rs` are all SQL placeholder helper comments (unrelated to stub code). No `TODO`/`FIXME`/`XXX` in any phase file.

The one pre-existing clippy warning in `app/src/tests/permitted_actions_parity.rs` (introduced at Phase 263, fixed at commit `8b2f946a`) is outside phase 244 scope and has been resolved.

---

### Human Verification Required

None. All success criteria are verifiable programmatically through the test suite and static code inspection. No visual rendering, real-time behavior, or external service integration is involved.

---

## Gaps Summary

No gaps. All eight must-haves (three roadmap success criteria + five from plan frontmatter, deduplicated to eight distinct items) are fully verified against the actual codebase. Both plan gate runs passed clean (`fmt --check`, `clippy --all --all-targets -D warnings`, targeted test suites). All commits are on-disk and confirmed.

---

_Verified: 2026-08-13_
_Verifier: Claude (gsd-verifier)_
