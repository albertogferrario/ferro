---
phase: 252
plan: 01
subsystem: ferro-json-ui
tags: [design-lint, spec, tdd, design-module]
requirements: [DS-05]

dependency_graph:
  requires: []
  provides:
    - ferro_json_ui::design::lint
    - ferro_json_ui::design::rules
    - ferro_json_ui::design::KNOWN_INTENTS
    - ferro_json_ui::design::Finding
    - ferro_json_ui::design::Severity
    - ferro_json_ui::design::DesignRule
    - ferro_json_ui::spec::DesignMeta
    - ferro_json_ui::Spec::design (optional field)
  affects:
    - ferro-json-ui/src/spec.rs (Spec struct extended)
    - ferro-json-ui/src/lib.rs (new pub use exports)

tech_stack:
  added: []
  patterns:
    - TDD (RED → GREEN per task)
    - static fn-pointer rule registry (zero-cost iteration)
    - feature-gated drift test (#[cfg(all(test, feature = "projections"))])
    - skip_serializing_if for optional wire fields

key_files:
  created:
    - ferro-json-ui/src/design/mod.rs
    - ferro-json-ui/src/design/types.rs
    - ferro-json-ui/src/design/rules.rs
    - ferro-json-ui/src/design/infer.rs
  modified:
    - ferro-json-ui/src/spec.rs
    - ferro-json-ui/src/catalog.rs
    - ferro-json-ui/src/lib.rs

decisions:
  - "DesignMeta defined in spec.rs (not design/types.rs) and re-exported from design/ — keeps spec.rs self-contained per PATTERNS pitfall 1"
  - "infer_intent uses Vec<&str> collect then contains() — required by clippy contains-over-iter-any lint"
  - "SpecWire internal deserialization struct updated alongside Spec struct literal sites"

metrics:
  duration: 7m
  completed: 2026-07-03T17:31:00Z
  tasks: 3
  files: 7
---

# Phase 252 Plan 01: Design module foundation Summary

Pure `ferro_json_ui::design` module with `Spec.design` wire field, typed `Finding`/`Severity`/`DesignRule`, a working `lint()` engine (intent resolution + allow-list validation + empty rule dispatch), and intent inference heuristic. The compile-and-test skeleton Plans 02–04 build on.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add DesignMeta + Spec.design | a6736675 | spec.rs, catalog.rs |
| 2 | Create design module + lib export | 090a4f93 | design/{mod,types,rules,infer}.rs, lib.rs |
| 3 | Engine + inference unit tests | 508a089e | design/mod.rs, design/infer.rs |
| — | fmt/clippy fixes | 92167a88 | design/infer.rs, design/mod.rs, lib.rs |

## Verification

- `cargo build -p ferro-json-ui` exits 0 (D-07: compiles without projections feature)
- `cargo build -p ferro-json-ui --features projections` exits 0
- `cargo test -p ferro-json-ui design` — 16 tests pass (6 engine + 7 inference + 3 spec)
- `cargo test -p ferro-json-ui --features projections drift` — drift test passes
- `cargo doc -p ferro-json-ui --no-deps` — zero warnings
- `cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] SpecWire internal struct needed design field**
- **Found during:** Task 1 (first GREEN build attempt)
- **Issue:** `Spec::from_json` deserializes through an internal `SpecWire` struct; adding `design` to `Spec` without adding it to `SpecWire` and the two Spec literal sites caused compile errors
- **Fix:** Added `#[serde(default)] design: Option<DesignMeta>` to `SpecWire`; added `design: None` to SpecBuilder's `build()` and `design: raw.design` to the from_json constructor site
- **Files modified:** ferro-json-ui/src/spec.rs, ferro-json-ui/src/catalog.rs (2 test literal sites)
- **Commit:** a6736675

**2. [Rule 1 - Bug] Clippy: contains() preferred over iter().any() for Vec<&str>**
- **Found during:** Post-Task 3 CI gate check
- **Issue:** `types.iter().any(|t| *t == "KanbanBoard")` and similar patterns trigger clippy `clippy::iter_over_hash_type` / contains-efficiency lint
- **Fix:** Replaced three `iter().any()` calls with `contains()` in infer.rs
- **Files modified:** ferro-json-ui/src/design/infer.rs
- **Commit:** 92167a88

## Known Stubs

None — the empty `RULE_REGISTRY` is intentional (Plans 02–03 populate it). The engine is fully functional; it just has no composition rules yet. `lint()` already produces the engine-level findings (`declare-intent` inference, unknown-intent warning, unknown-`allow` warning).

## Threat Flags

No new network endpoints, auth paths, or file access patterns introduced. `lint()` is pure in-process computation consuming an already-deserialized `Spec`.

## Self-Check: PASSED

- `ferro-json-ui/src/design/mod.rs` — FOUND
- `ferro-json-ui/src/design/types.rs` — FOUND
- `ferro-json-ui/src/design/rules.rs` — FOUND
- `ferro-json-ui/src/design/infer.rs` — FOUND
- Commit a6736675 — FOUND (`git log --oneline`)
- Commit 090a4f93 — FOUND
- Commit 508a089e — FOUND
- Commit 92167a88 — FOUND
