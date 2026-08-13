---
phase: 245-typed-result-handle-serializable-enforcement
plan: "02"
subsystem: ferro-macros
tags: [offload, typed-handle, serializable-contract, compile-time-enforcement, trybuild]
dependency_graph:
  requires: [245-01]
  provides: [offload-macro-impl-offloadable, offload-macro-param-enforcement, offload-macro-return-enforcement]
  affects: [ferro-macros/src/offload.rs]
tech_stack:
  added: []
  patterns:
    - "#[serde(bound = \"\")] on derived struct to suppress serde's auto-generated where-clause, leaving OffloadSerializable as the sole enforcement path"
    - "output_type: OffloadSerializable on both the struct where-clause and impl Offloadable where-clause to force the branded #[diagnostic::on_unimplemented] message for return-type violations"
    - "trybuild pass fixture with fn assert_output_is_report::<J: Offloadable<Output = Report>>() for compile-time type-equality proofs without runtime dispatch"
key_files:
  created:
    - ferro-macros/tests/ui/offload/pass/offload_handle.rs
    - ferro-macros/tests/ui/offload/fail/non_serializable_param.rs
    - ferro-macros/tests/ui/offload/fail/non_serializable_param.stderr
    - ferro-macros/tests/ui/offload/fail/non_serializable_return.rs
    - ferro-macros/tests/ui/offload/fail/non_serializable_return.stderr
  modified:
    - ferro-macros/src/offload.rs
decisions:
  - "output_type added to both the struct where-clause and the impl Offloadable where-clause — both placements are required: the struct where-clause ensures the branded diagnostic fires even before the impl is evaluated; the impl where-clause is the semantically correct enforcement point"
  - "serde(bound = \"\") mitigation applied to suppress serde-derive's own where-clause generation on the derived struct, which would otherwise produce duplicate serde bounds that fire before OffloadSerializable"
  - "Derived Job ident naming convention confirmed: <Trait><MethodPascalCase>Job — e.g. ReportsServiceBuildMonthlyJob for trait ReportsService method build_monthly"
metrics:
  duration_seconds: 658
  completed_date: "2026-08-13T15:15:45Z"
  tasks_completed: 3
  files_changed: 6
requirements: [OFFLOAD-02]
---

# Phase 245 Plan 02: Typed Output Handle + Serializable Enforcement — Summary

`OffloadMethodInfo.output_type` capture, `impl Offloadable { type Output }` emission, param+return `OffloadSerializable` where-clause enforcement, and three trybuild fixtures (pass/offload_handle, fail/non_serializable_param, fail/non_serializable_return).

## One-liner

`#[offload]` macro extended with `output_type` extraction (Result<T,E>→T; bare→T; default→()), `impl ::ferro::queue::Offloadable` emission, and a two-site `OffloadSerializable` where-clause (struct + impl) that fires the branded isolation-boundary diagnostic for both param and return violations.

## Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2 | output_type capture + impl Offloadable + param where-clause | d9a5efb0 | ferro-macros/src/offload.rs |
| 3 | trybuild fixtures (pass/offload_handle, fail/param, fail/return) + .stderr generation | 722dfbf2 | ferro-macros/src/offload.rs, 5 new test files |

Note: Tasks 1 and 2 were committed together because separating them produced a `dead_code` warning under `-D warnings` (the `output_type` field is unused until `emit_job_items` consumes it in Task 2). The combined commit is atomic with respect to the lint gate.

## Branded-Message Primacy Check (Open Question 1 — REQUIRED EMPIRICAL RESULT)

**Mitigation needed: YES**

**First-run diagnosis (pre-mitigation):** Both fail fixtures produced serde's default `the trait bound XType: serde::Serialize is not satisfied` as the primary `error[E0277]` line. The branded `OffloadSerializable` "isolation boundary" message appeared later in the error stream (errors 13–14 of ~15 for the param case; absent entirely for the return case).

**Root cause:** The `Offloadable` supertrait bounds (`Serialize + DeserializeOwned + Sized`) live at the trait level in `ferro-queue`. The `service` macro expansion checks those supertrait bounds on the concrete service type, which triggers serde's own `E0277` messages before the macro-emitted `OffloadSerializable` struct where-clause gets a chance to fire.

**Mitigations applied (two-stage):**

1. `#[serde(bound = "")]` on the derived struct: suppresses serde-derive's own auto-generated where-clause. Without this, serde would add redundant `field_type: Serialize + Deserialize` constraints to the derived struct in addition to the `OffloadSerializable` bound — producing duplicate serde messages at the struct level.

2. `output_type: ::ferro::queue::OffloadSerializable` added to the struct where-clause (in addition to the impl where-clause): this makes `RawReport: OffloadSerializable` a direct bound on the derived struct, which causes rustc to emit the branded `#[diagnostic::on_unimplemented]` message for the return case. Without this, the return-type path only reached `OffloadSerializable` via the `Offloadable::Output` associated-type chain, which does not trigger `#[diagnostic::on_unimplemented]`.

**Post-mitigation result:** Both `.stderr` files contain "isolation boundary" (param: 4 occurrences, return: 4 occurrences). Both name the offending type in the branded message line.

**Confirmed branded message text (for Plan 03 to quote verbatim):**

Param case (`non_serializable_param.stderr`):
```
error[E0277]: `RawHandle` crosses the #[offload] isolation boundary and must be `Serialize + DeserializeOwned`
  = note: offloaded parameters and return types travel as a queue payload; implement `Serialize` and `DeserializeOwned` for `RawHandle` to seal the module across the isolation boundary
```

Return case (`non_serializable_return.stderr`):
```
error[E0277]: `RawReport` crosses the #[offload] isolation boundary and must be `Serialize + DeserializeOwned`
  = note: offloaded parameters and return types travel as a queue payload; implement `Serialize` and `DeserializeOwned` for `RawReport` to seal the module across the isolation boundary
```

**Message primacy note:** The branded message is not the *first* `error[E0277]` in either stderr — serde's own supertrait bounds from the `Offloadable` trait level fire first (those are emitted by the `service` macro expansion, before the derived struct's where-clause). The branded message is one of the later errors in the list. This is an inherent constraint of the architecture: the `Offloadable` supertrait requires `Serialize + DeserializeOwned`, so any non-serializable type will always produce at least those two serde E0277s before the `OffloadSerializable` diagnostic fires. Plan 03 documentation should note this ordering when quoting the error.

## Derived Job Ident Naming Convention (confirmed)

Pattern: `<TraitPascalCase><MethodPascalCase>Job`

Examples confirmed from pass fixtures:
- `ExporterService` + `export` → `ExporterServiceExportJob` (from `result_method.rs`)
- `ReportsService` + `build_monthly` → `ReportsServiceBuildMonthlyJob` (from `offload_handle.rs`)

The `to_pascal_case` function handles snake_case method names; the trait ident is already PascalCase.

## Verification Results

- `cargo test -p ferro-macros --test offload_macro` (match mode, post-mitigation): **1 passed, 0 failed** — all 4 pass fixtures compile, all 3 fail fixtures match their `.stderr` snapshots
- `cargo build -p ferro-macros`: exit 0
- `cargo test -p ferro-rs --lib`: 514 passed, 0 failed
- `cargo fmt --all -- --check`: clean
- `cargo clippy -p ferro-macros --all-targets -- -D warnings`: clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tasks 1+2 committed together to satisfy -D warnings lint gate**
- **Found during:** Task 1 commit attempt
- **Issue:** `output_type` field added in Task 1 is unused until Task 2 consumes it in `emit_job_items`. Under `cargo clippy --all-targets -- -D warnings`, `dead_code` is promoted to an error.
- **Fix:** Executed Task 2 before committing Task 1, then committed both as a single atomic unit.
- **Files modified:** ferro-macros/src/offload.rs
- **Commit:** d9a5efb0

**2. [Rule 2 - Missing critical functionality] `#[serde(bound = "")]` mitigation for branded-message primacy**
- **Found during:** Task 3, post-TRYBUILD=overwrite inspection
- **Issue:** Without the mitigation, serde-derive generates its own where-clause on the derived struct (`field_type: Serialize + Deserialize`), producing duplicate serde errors that fire before `OffloadSerializable`. The branded message was present but buried.
- **Fix:** Added `#[serde(bound = "")]` to the derived struct in `emit_job_items` to suppress serde's auto-generated where-clause.
- **Files modified:** ferro-macros/src/offload.rs
- **Commit:** 722dfbf2

**3. [Rule 2 - Missing critical functionality] `output_type: OffloadSerializable` on struct where-clause for return-type branded message**
- **Found during:** Task 3, branded-message primacy check for return case
- **Issue:** After the `#[serde(bound = "")]` mitigation, the return-type `.stderr` still lacked "isolation boundary" — the `OffloadSerializable` diagnostic only fires when the type appears as a *direct* bound on the failing type, not via the `Offloadable::Output` associated-type chain. The impl-level `where #output_type: OffloadSerializable` bound was insufficient on its own.
- **Fix:** Added `#output_type: ::ferro::queue::OffloadSerializable` to the struct where-clause (in addition to the impl where-clause). This makes `RawReport: OffloadSerializable` a direct struct-level constraint, triggering the `#[diagnostic::on_unimplemented]` message.
- **Files modified:** ferro-macros/src/offload.rs
- **Commit:** 722dfbf2

## Known Stubs

None. All three behaviors are fully implemented and verified:
- `type Output` extraction and emission (Tasks 1+2)
- Param `OffloadSerializable` where-clause (Task 2)
- Return `OffloadSerializable` where-clause on both struct and impl (Task 2, deviation 3)
- Three trybuild fixtures with inspected `.stderr` snapshots (Task 3)

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes. The extension is purely compile-time token transformation and trybuild fixtures, consistent with the plan's threat model (T-245-04 through T-245-06).

## Self-Check: PASSED
