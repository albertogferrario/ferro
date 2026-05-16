---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: "08"
subsystem: ferro-json-ui
tags: [strum, ergonomics, variant-enums, type-safety]
dependency_graph:
  requires: ["162-07"]
  provides: ["strum::AsRefStr on all six variant enums"]
  affects: ["ferro-json-ui/src/component.rs", "ferro-json-ui/src/action.rs"]
tech_stack:
  added: ["strum 0.26 (derive feature)"]
  patterns: ["TDD RED/GREEN per-task", "strum::AsRefStr + #[strum(serialize_all)]"]
key_files:
  created: []
  modified:
    - ferro-json-ui/Cargo.toml
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/action.rs
decisions:
  - "Pin strum at 0.26 directly in ferro-json-ui/Cargo.toml (not a workspace dep); strum is not used by other crates yet"
  - "Use #[strum(serialize_all = \"snake_case\")] to mirror #[serde(rename_all = \"snake_case\")] — both must agree or the round-trip test catches the drift"
  - "Consolidated test variant_enums_strum_matches_serde_wire_format covers all four component.rs enums; separate test covers action.rs enums"
metrics:
  duration: "~10 min"
  completed: "2026-05-16"
  tasks_completed: 3
  files_modified: 3
---

# Phase 162 Plan 08: strum::AsRefStr Variant Enum Derives Summary

Adds `strum::AsRefStr` derive to all six variant enums in `ferro-json-ui`, giving consumers typed `.as_ref()` → snake_case ergonomics (D-11, D-12). JSON wire format is unchanged.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add strum 0.26 dep to Cargo.toml | 10f42e5f | ferro-json-ui/Cargo.toml |
| 2 (RED) | Failing tests for component.rs enums | b5dd71c2 | ferro-json-ui/src/component.rs |
| 2 (GREEN) | AsRefStr derives for Alert/Badge/Button/Toast | 23ed0f7e | ferro-json-ui/src/component.rs |
| 3 (RED) | Failing tests for action.rs enums | 4be356cd | ferro-json-ui/src/action.rs |
| 3 (GREEN) | AsRefStr derives for Dialog/Notify | 166658e6 | ferro-json-ui/src/action.rs |

## What Was Built

`strum 0.26` with the `derive` feature was added to `ferro-json-ui/Cargo.toml`. Six variant enums received `#[derive(..., strum::AsRefStr)]` and `#[strum(serialize_all = "snake_case")]`:

- `ferro-json-ui/src/component.rs`: `ButtonVariant`, `AlertVariant`, `BadgeVariant`, `ToastVariant`
- `ferro-json-ui/src/action.rs`: `DialogVariant`, `NotifyVariant`

The serde `rename_all = "snake_case"` attributes were left untouched. The strum attribute is a parallel compile-time mapping that adds `AsRef<str>` without touching serialization.

Two test suites pin the strum-vs-serde contract:
- `variant_enums_strum_matches_serde_wire_format` (component.rs) — iterates every variant of all four component enums
- `dialog_notify_variant_strum_matches_serde` (action.rs) — iterates every variant of both action enums

## Decisions Made

- Direct version pin (`strum = { version = "0.26", features = ["derive"] }`) in `ferro-json-ui/Cargo.toml`; no workspace-level entry needed since no other crate uses strum yet.
- `#[strum(serialize_all = "snake_case")]` placed on the enum type (not per-variant), matching the serde attribute pattern already present.
- TDD RED/GREEN executed per task: tests committed in failing state first, then derives added to make them pass.

## Deviations from Plan

None — plan executed exactly as written. `cargo fmt` reformatted multi-derive lines to the project's preferred multi-line style, which is the expected outcome.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The six derives are purely compile-time trait implementations on existing types. T-162-08-01 (strum/serde string drift) is mitigated by the round-trip tests added in this plan.

## Self-Check: PASSED

- ferro-json-ui/Cargo.toml contains strum 0.26: confirmed
- 4 strum::AsRefStr derives in component.rs: confirmed (`grep -c` = 4)
- 2 strum::AsRefStr derives in action.rs: confirmed (`grep -c` = 2)
- All commits exist: 10f42e5f, b5dd71c2, 23ed0f7e, 4be356cd, 166658e6
- `cargo test -p ferro-json-ui --all-features`: 434 passed, 0 failed
