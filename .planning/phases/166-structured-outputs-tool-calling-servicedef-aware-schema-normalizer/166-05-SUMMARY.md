---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
plan: "05"
subsystem: ferro-ai, ci
tags: [publish-wave, wave-ordering, sc7, fmt, clippy, cargo-test, phase-gate, final-plan]
dependency_graph:
  requires: [166-01, 166-02, 166-03, 166-04]
  provides: [publish-wave-fix, sc7-gate, aisdk-02, aisdk-03]
  affects: [.github/workflows/publish.yml]
tech_stack:
  added: []
  patterns: [publish-wave-dag-ordering, phase-gate-serialized]
key_files:
  created: []
  modified:
    - .github/workflows/publish.yml
decisions:
  - "WAVE1B_CRATES reordered so ferro-projections precedes ferro-ai; both remain in WAVE1B — ferro-projections depends only on ferro-theme (WAVE1A), so no wave promotion needed"
  - "Dep comment block updated to document ferro-ai -> ferro-projections edge (Phase 166) for future publish-order audits"
metrics:
  duration: "~598 seconds (~10 minutes)"
  completed: "2026-06-08T04:25:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 1
---

# Phase 166 Plan 05: Publish-wave Fix + Full Phase Gate (SC#7) Summary

Publish-wave ordering corrected for the new `ferro-ai -> ferro-projections` dependency added in Phase 166 Plan 01; full workspace gate (`fmt + clippy -D warnings + cargo test --all-features`) confirms SC#7 green with all Phase 166 tests passing and the existing `Classifier<T>` suite intact.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Reorder WAVE1B so ferro-projections precedes ferro-ai | 3d521cb2 | .github/workflows/publish.yml |
| 2 | Full phase gate — fmt + clippy + cargo test --all-features (SC#7) | (verification only — no source changes) | — |

## Decisions Made

**WAVE1B reorder, no wave promotion (Task 1):** `ferro-projections` already resided in WAVE1B (it depends on `ferro-theme`, a WAVE1A crate). Moving it to the front of `WAVE1B_CRATES` is sufficient — no wave boundary change needed. The loop iterates the string left-to-right, so list order is publish order.

**Dep comment updated (Task 1):** The dependency comment block directly above `WAVE1B_CRATES` now shows `ferro-ai -> ferro-events, ferro-projections (Phase 166)` so future publish-order audits can trace the new edge without reading `ferro-ai/Cargo.toml`.

## Verification Results

### Task 1 — publish.yml ordering
- `grep -n "WAVE1B_CRATES=" .github/workflows/publish.yml | grep -q "ferro-projections ferro-ai"` — OK
- `grep -c "WAVE1A_CRATES=\|WAVE2_CRATES=\|WAVE3_CRATES=" .github/workflows/publish.yml` — 3 (unchanged)
- YAML parse via `python3 -c "import yaml,sys; yaml.safe_load(...)"` — exits 0

### Task 2 — SC#7 full gate
- `cargo fmt --all -- --check` — exits 0 (no output)
- `cargo clippy --all --all-targets -- -D warnings` — exits 0 (`Finished` with no warnings)
- `cargo test --all-features` — all test suites green, zero failures
- `cargo test -p ferro-ai classifier` — **8/8 green** (SC#7: existing Classifier<T> tests intact)
- Phase 166 gate tests — all present and green:
  - `schema_probe_field_meaning_any_of_shape` ok
  - `schema_probe_intent_any_of_shape` ok
  - `servicedef_schema_accepts_all_known_intent_variants` ok
  - `servicedef_schema_accepts_minimal_servicedef` ok
  - `servicedef_schema_accepts_all_known_field_meaning_variants` ok
  - `servicedef_schema_rejects_invalid_field_meaning` ok
  - `servicedef_schema_rejects_invalid_intent` ok
  - `complete_returns_typed_result` ok
  - `tool_registry_enforces_max_iterations` ok

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Mitigations Verified

| Threat | Mitigation | Verified By |
|--------|------------|-------------|
| T-166-PUB-01 (broken release pipeline) | WAVE1B reordered; ferro-projections precedes ferro-ai; DAG satisfied | grep + YAML parse + Task 1 commit 3d521cb2 |
| T-166-GATE-01 (regression slips through) | `cargo test --all-features` green; clippy `-D warnings` clean | Task 2 full gate |

## Threat Surface Scan

No new network endpoints, auth paths, or file access introduced. Only `.github/workflows/publish.yml` modified — a CI config file with no runtime trust boundary impact.

## Self-Check: PASSED

- `.github/workflows/publish.yml` WAVE1B contains `ferro-projections ferro-ai` in that order: confirmed
- `ferro-projections` dependency comment documented under WAVE1B: confirmed
- WAVE1A_CRATES, WAVE2_CRATES, WAVE3_CRATES count = 3 (unchanged): confirmed
- YAML parses cleanly: confirmed
- `cargo fmt --all -- --check` exits 0: confirmed
- `cargo clippy --all --all-targets -- -D warnings` exits 0: confirmed
- `cargo test --all-features` exits 0 (all suites green): confirmed
- `cargo test -p ferro-ai classifier` 8/8 green: confirmed
- Commit 3d521cb2 exists: confirmed
