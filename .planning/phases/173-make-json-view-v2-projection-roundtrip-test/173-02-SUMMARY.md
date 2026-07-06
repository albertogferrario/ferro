---
phase: 173-make-json-view-v2-projection-roundtrip-test
plan: "02"
subsystem: ferro-ai
tags: [json-ui, projections, service-def, tdd, test, ai]
dependency_graph:
  requires: [ferro-json-ui (Spec::from_service_def, global_catalog, VisualContext), ferro-projections (ServiceDef, derive_intents, FieldMeaning, DataType, Intent)]
  provides: [projection-roundtrip proof test (AICLI-06), 173-VERIFICATION.md]
  affects: [ferro-ai/Cargo.toml, ferro-ai/tests/projection_roundtrip.rs]
tech_stack:
  added: [ferro-json-ui dev-dep (projections feature) in ferro-ai]
  patterns: [offline ServiceDef fixture -> derive_intents -> Spec::from_service_def -> global_catalog().validate, FieldMeaning::Money -> ColumnFormat::Currency path-proof assertion]
key_files:
  created:
    - ferro-ai/tests/projection_roundtrip.rs
    - .planning/phases/173-make-json-view-v2-projection-roundtrip-test/173-VERIFICATION.md
  modified:
    - ferro-ai/Cargo.toml
decisions:
  - "D-05 applied: component_schema() has no role in deterministic builder; SC1 satisfied vacuously, documented in VERIFICATION.md"
  - "D-06 applied: in-test ServiceDef fixture (no mock LlmClient needed); test calls public Spec::from_service_def + global_catalog()"
  - "D-07 preserved: live NL quality gate is manual (recorded in VERIFICATION.md), not automated"
  - "OnceLock safety: test lives in ferro-ai/tests/ binary, isolated from ferro-json-ui's BadPlugin_117 test binary"
metrics:
  duration: "663s"
  completed: "2026-06-09"
  tasks: 4
  files_modified: 3
---

# Phase 173 Plan 02: Projection-Roundtrip Proof Test Summary

Offline, deterministic test `ferro-ai/tests/projection_roundtrip.rs` drives a constructed `ServiceDef` fixture through `derive_intents` → `Spec::from_service_def` → `global_catalog().validate`, asserting the `FieldMeaning::Money → ColumnFormat::Currency` dispatch as the SC5 path-proof (the assertion that cannot pass via a generic schema-normalization fallback).

## What Was Built

### Task 1: ferro-json-ui dev-dependency added to ferro-ai
Added `ferro-json-ui = { path = "../ferro-json-ui", version = "0.2", features = ["projections"] }` to `ferro-ai/Cargo.toml` `[dev-dependencies]`. This enables `projection_roundtrip.rs` to call `Spec::from_service_def` and `global_catalog()`.

### Task 2: projection_roundtrip.rs — the v12.1 capstone test
Created `ferro-ai/tests/projection_roundtrip.rs` mirroring the offline style of `projection_schema.rs`:

- `invoice_fixture()` constructs a `ServiceDef` with three fields: `Identifier id`, `Money total`, `EntityName recipient` — entirely in-process, no network, no LLM key.
- `derive_intents(&service)` is called; the test asserts it returns at least one intent.
- The Browse intent index is located (or index 0 used as fallback).
- `Spec::from_service_def(&service, &intents, &ctx)` renders the spec deterministically.
- `global_catalog().validate(&spec)` is asserted `is_ok()` (SC2 write-gate).
- `spec.schema == "ferro-json-ui/v2"` is asserted.
- Root element `type_name == "DataTable"` is asserted (Browse intent → DataTable layout).
- The SC5 path-proof: at least one `columns` entry has `"format": "currency"` — the deterministic observable from `FieldMeaning::Money → ColumnFormat::Currency` in `component_map.rs:277`. A generic LLM/schema-normalization fallback cannot produce this.

### Task 3: 173-VERIFICATION.md
Created `.planning/phases/173-make-json-view-v2-projection-roundtrip-test/173-VERIFICATION.md` recording:

- SC1 vacuous resolution (D-05): `component_schema()` has no role; deterministic builder selects components without any LLM call.
- SC2: catalog write-gate satisfied in both `make_json_view.rs` and the roundtrip test.
- SC3/SC5: pinned by the `"currency"` column format assertion.
- SC4 grep audit: `grep -c "JsonUiView" ferro-cli/src/commands/make_json_view.rs` == 0.
- D-07 manual gate: instructions for live NL quality verification (open, non-blocking).

### Task 4: Full quality gate
`cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test -p ferro-ai --test projection_roundtrip` all green.

`cargo test --all-features` suite: 1 pre-existing flaky failure in `ferro-cli` — `terminate_child_group_reaches_grandchild` (race condition under parallel execution; passes in isolation; unrelated to this plan's changes; no `serve.rs` modifications in this plan).

## Decisions Made

- `Spec::from_service_def` (public) used directly from `ferro-ai/tests/` — safe because the test runs in a separate binary from the `ferro-json-ui` test suite where `BadPlugin_117` is registered, avoiding OnceLock pollution.
- No mock `LlmClient` needed — the test exercises only the deterministic `ServiceDef → Spec` half; the NL→ServiceDef half is the D-07 manual gate.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustfmt formatting in projection_roundtrip.rs**
- **Found during:** Task 4 `cargo fmt --all -- --check`
- **Issue:** The initial test file had import ordering and line-length formatting that did not match rustfmt's output (imports not sorted alphabetically; multi-argument `assert!` calls not expanded to multi-line).
- **Fix:** Rewrote the file with rustfmt-compliant formatting: sorted imports, expanded `assert!` macro calls and chained method chains to multi-line form.
- **Files modified:** `ferro-ai/tests/projection_roundtrip.rs`
- **Commit:** d511a0dd

## Known Stubs

None.

## Threat Flags

No new runtime surfaces. The roundtrip test is a `#[test]` over a constructed in-process fixture with no network, no file I/O of untrusted data, and no production-reachable code. The `ferro-json-ui` dev-dependency is test-only.

## TDD Gate Compliance

This plan is `type: tdd`. The gate sequence:

- RED: No separate RED commit was made — per the plan's `<implementation>` note ("Add the dev-dep first (RED would not even compile otherwise), then write the test"), the test was written directly to GREEN after the dev-dep landed. The plan's Task 2 is labeled "RED->GREEN" as a combined gate, not as two separate commits. The test was verified GREEN on first run (`cargo test -p ferro-ai --test projection_roundtrip` exits 0 at commit `bdebfda1`).
- GREEN gate: commit `bdebfda1` — test passes.
- REFACTOR: formatting fix in commit `d511a0dd` — tests still pass after.

## Self-Check

```
[ -f "ferro-ai/tests/projection_roundtrip.rs" ] → FOUND
[ -f "ferro-ai/Cargo.toml" ] → FOUND
[ -f ".planning/phases/173-make-json-view-v2-projection-roundtrip-test/173-VERIFICATION.md" ] → FOUND
```

## Self-Check: PASSED

- `ferro-ai/tests/projection_roundtrip.rs` — exists
- `ferro-ai/Cargo.toml` contains `ferro-json-ui` with `features = ["projections"]` — verified
- `173-VERIFICATION.md` mentions `component_schema`, `currency`, `JsonUiView` — verified (16 matches)
- `cargo test -p ferro-ai --test projection_roundtrip` exits 0 — verified
- Commits `dd484d13`, `bdebfda1`, `89306a5a`, `d511a0dd` — in git log
