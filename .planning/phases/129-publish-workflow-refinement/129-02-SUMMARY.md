---
phase: 129-publish-workflow-refinement
plan: "02"
subsystem: ferro-cli
tags: [schema, deploy-metadata, forward-compatibility, tests]
dependency_graph:
  requires: []
  provides:
    - ferro-cli/src/project.rs::FerroDeployMetadata.ferro_versions
    - ferro-cli/src/deploy/rewrite_ferro_version.rs::preserves_ferro_versions_override_roundtrip
  affects:
    - Any caller that pattern-matches or constructs FerroDeployMetadata
tech_stack:
  added: []
  patterns:
    - Hand-rolled toml::Value field extraction with explicit error messages
    - toml_edit byte-preserving round-trip for untouched metadata tables
key_files:
  modified:
    - ferro-cli/src/project.rs
    - ferro-cli/src/deploy/rewrite_ferro_version.rs
decisions:
  - "ferro_versions parsed but not wired into rewrite logic — schema reservation only (Phase 129 D-07..D-09)"
  - "TODO comment references Phase 129 / REPORT §14 as the future resolution point"
  - "Round-trip test proves toml_edit leaves [package.metadata.ferro.deploy.ferro_versions] byte-identical"
metrics:
  duration: "~2 min"
  completed: "2026-04-09"
  tasks: 3
  files: 2
---

# Phase 129 Plan 02: ferro_versions Schema Reservation Summary

Reserve the `ferro_versions` per-crate override schema in `FerroDeployMetadata` — parsed and round-tripped, not consumed by any rewrite logic.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add ferro_versions field + parser + unit tests | f8d99c49 | ferro-cli/src/project.rs |
| 2 | Add preserves_ferro_versions_override_roundtrip test | 78c392be | ferro-cli/src/deploy/rewrite_ferro_version.rs |
| 3 | Full crate lint + test gate | — (gate only) | — |

## What Was Built

Added `ferro_versions: Option<BTreeMap<String, String>>` to `FerroDeployMetadata` with:

- Hand-rolled `toml::Value` parse block matching the existing field-extraction pattern
- Targeted error messages: `ferro_versions must be a table` and `ferro_versions.<key> must be a string`
- TODO comment referencing Phase 129 / REPORT §14 pointing at the future wiring path
- Two unit tests in `project::tests`: `parses_ferro_versions_override` and `rejects_ferro_versions_wrong_type`
- One round-trip regression test in `deploy::rewrite_ferro_version::tests`: `preserves_ferro_versions_override_roundtrip`

No rewrite logic changed. No CLI surface changed. No doctor check added.

## Verification

All acceptance criteria met:

- `grep 'pub ferro_versions: Option<BTreeMap<String, String>>'` — present in project.rs
- `grep 'TODO(Phase 129'` — present in project.rs referencing REPORT §14
- `cargo test -p ferro-cli -- project::tests::parses_ferro_versions_override` — PASS
- `cargo test -p ferro-cli -- project::tests::rejects_ferro_versions_wrong_type` — PASS
- `cargo test -p ferro-cli -- project::tests` — all 19 tests PASS
- `cargo test -p ferro-cli -- deploy::rewrite_ferro_version::tests::preserves_ferro_versions_override_roundtrip` — PASS
- `cargo fmt --all -- --check` — exit 0
- `cargo clippy --all --all-targets -- -D warnings` — exit 0
- `cargo test -p ferro-cli --all-features` — 483 + 3 tests, 0 failures

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — the `ferro_versions` field is explicitly a schema-only reservation. It is parsed and round-tripped but intentionally not wired to any rewrite logic. The TODO comment and Phase 129 / REPORT §14 tracking reference document the intended future resolution path.

## Self-Check: PASSED
