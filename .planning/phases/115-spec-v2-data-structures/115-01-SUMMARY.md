---
phase: 115-spec-v2-data-structures
plan: 01
subsystem: ui
tags: [json-ui, spec-v2, sdui, serde, thiserror, validation]

# Dependency graph
requires:
  - phase: 114-v11-5-closeout
    provides: stable ferro-json-ui v1 surface (action.rs, visibility.rs) to reuse in Element
provides:
  - Spec v2 type foundation (Spec, Element, SpecBuilder, ElementBuilder, SpecError)
  - SCHEMA_VERSION = "ferro-json-ui/v2" constant
  - MAX_NESTING_DEPTH = 3 constant
  - Spec::from_json parse-time structural validator (ID format, root, dangling refs, cycles, depth)
  - 7 ok fixtures + 11 reject fixtures under ferro-json-ui/tests/fixtures/
  - tests/round_trip.rs and tests/reject.rs integration suites
affects:
  - 115-02-delete-v1 (will unalias SCHEMA_VERSION and remove v1 types)
  - 116-flat-element-renderer (consumes Spec)
  - 117-catalog-and-schema (reads Element.type_name against catalog)
  - 119-page-loader (calls Spec::from_json on untrusted JSON)

# Tech tracking
tech-stack:
  added: [thiserror = "1.0"]
  patterns:
    - "Flat element map with ID-keyed references (Vercel json-render-style, SDUI 3-tier cap)"
    - "Parse-time structural validation: duplicate -> ID format -> root -> dangling -> cycle -> depth"
    - "Custom MapAccess visitor rejects duplicate JSON object keys via sentinel-in-error-message pattern"
    - "thiserror-derived error enum with structured variant payloads (path: Vec<String>, not strings)"
    - "Consuming builder with mut self -> Self (SpecBuilder, ElementBuilder)"

key-files:
  created:
    - ferro-json-ui/src/spec.rs
    - ferro-json-ui/tests/round_trip.rs
    - ferro-json-ui/tests/reject.rs
    - ferro-json-ui/tests/fixtures/ok/ (7 JSON fixtures)
    - ferro-json-ui/tests/fixtures/reject/ (11 JSON fixtures)
  modified:
    - ferro-json-ui/Cargo.toml (added thiserror dep)
    - ferro-json-ui/src/lib.rs (additive: pub mod spec + re-exports; SCHEMA_VERSION aliased to SCHEMA_VERSION_V2 to avoid v1 collision)

key-decisions:
  - "Plan 01 is strictly additive. v1 types (JsonUiView, Component, ComponentNode) remain intact; v2 lives alongside. Plan 02 deletes v1."
  - "SCHEMA_VERSION re-exported as SCHEMA_VERSION_V2 alias to sidestep the v1 name collision; Plan 02 unaliases once view.rs is gone."
  - "Element::new() returns ElementBuilder (not Self). clippy::new_ret_no_self allowed locally with justification — the fluent-builder ergonomics take precedence."
  - "Duplicate-ID detection rides on serde::de::Error::custom via a sentinel string (__FERRO_DUPLICATE_ID__). The sentinel is rewrapped into SpecError::DuplicateId at the from_json boundary — avoids forking serde_json."

patterns-established:
  - "ferro-json-ui Spec v2 shape: flat HashMap<String, Element> keyed by ID with a single root pointer. Children are string refs, not nested structures."
  - "Validation order: IDs -> root exists -> no dangling -> no cycles -> depth <= 3. Cheapest/most specific checks first; graph traversals last."
  - "Parse-time rejection of malformed input via typed SpecError variants. from_json never panics on arbitrary input."

requirements-completed: [SPEC-01, SPEC-02, SPEC-03]

# Metrics
duration: ~35min
completed: 2026-04-18
---

# Phase 115 Plan 01: Spec v2 Data Structures Summary

**Flat Spec/Element type foundation with parse-time structural validation (duplicate, ID format, root, dangling, cycle, depth) and an 18-fixture test corpus — additive alongside v1, zero regressions.**

## Performance

- **Tasks:** 2 (both completed)
- **Files created:** 20 (spec.rs + 2 integration tests + 18 fixtures)
- **Files modified:** 3 (Cargo.toml, lib.rs, Cargo.lock)
- **Total test count added:** 36 new test assertions (17 unit + 8 round-trip + 11 reject)
- **Test counts after plan:**
  - `cargo test -p ferro-json-ui --lib` -> 488 passed (17 new spec:: + 471 pre-existing)
  - `cargo test -p ferro-json-ui --test round_trip` -> 8 passed
  - `cargo test -p ferro-json-ui --test reject` -> 11 passed

## Accomplishments

- `Spec`, `Element`, `SpecError`, `SCHEMA_VERSION`, `MAX_NESTING_DEPTH` shipped in `ferro-json-ui/src/spec.rs`
- `Spec::from_json` rejects every malformed shape enumerated in CONTEXT.md D-09/D-11 with a typed variant and never panics on arbitrary input (fuzz-resistant by construction — recursion is depth-bounded at 4 frames by MAX_NESTING_DEPTH=3 pre-check)
- `SpecBuilder` / `ElementBuilder` fluent API (consuming `mut self -> Self`) — first `.element(id, _)` sets root if not explicitly set
- Duplicate-ID detection via custom `ElementsMap` serde visitor; `serde_json::Map`'s silent-overwrite default is defeated
- 18 fixture JSON files + 2 integration test modules pin the structural contract for downstream plans

## Task Commits

1. **Task 1: Create spec.rs with types, builders, validation, and from_json; register module additively in lib.rs; add thiserror dependency** — `71608b0d` (feat)
2. **Task 2: Create 18 fixture JSON files and two integration tests (round_trip.rs, reject.rs)** — `c89481df` (test)

## Files Created/Modified

### Created
- `ferro-json-ui/src/spec.rs` (610 LoC) — Spec/Element/SpecError/builders/validation
- `ferro-json-ui/tests/round_trip.rs` — 8 integration tests (7 round-trip + builder parity)
- `ferro-json-ui/tests/reject.rs` — 11 variant-specific SpecError assertions
- `ferro-json-ui/tests/fixtures/ok/minimal_single_element.json`
- `ferro-json-ui/tests/fixtures/ok/three_level_nested.json`
- `ferro-json-ui/tests/fixtures/ok/with_actions.json`
- `ferro-json-ui/tests/fixtures/ok/with_visibility.json`
- `ferro-json-ui/tests/fixtures/ok/with_plugin_named_type.json`
- `ferro-json-ui/tests/fixtures/ok/with_data_payload.json`
- `ferro-json-ui/tests/fixtures/ok/omitted_optional_fields.json`
- `ferro-json-ui/tests/fixtures/reject/missing_root.json`
- `ferro-json-ui/tests/fixtures/reject/dangling_child.json`
- `ferro-json-ui/tests/fixtures/reject/simple_cycle.json`
- `ferro-json-ui/tests/fixtures/reject/self_cycle.json`
- `ferro-json-ui/tests/fixtures/reject/four_level_nesting.json`
- `ferro-json-ui/tests/fixtures/reject/invalid_id_space.json`
- `ferro-json-ui/tests/fixtures/reject/invalid_id_empty.json`
- `ferro-json-ui/tests/fixtures/reject/invalid_id_digit_start.json`
- `ferro-json-ui/tests/fixtures/reject/invalid_id_too_long.json` (129-char key)
- `ferro-json-ui/tests/fixtures/reject/invalid_child_ref_format.json`
- `ferro-json-ui/tests/fixtures/reject/duplicate_id.json`

### Modified
- `ferro-json-ui/Cargo.toml` — added `thiserror = "1.0"` under `[dependencies]`
- `ferro-json-ui/src/lib.rs` — added `pub mod spec;` and a new re-export block:
  ```rust
  pub use spec::{
      Element, ElementBuilder, Spec, SpecBuilder, SpecError, MAX_NESTING_DEPTH,
      SCHEMA_VERSION as SCHEMA_VERSION_V2,
  };
  ```
  Nothing removed. Plan 02 will unalias `SCHEMA_VERSION_V2` when v1 is deleted.
- `Cargo.lock` — thiserror graph pulled in

## Decisions Made

Followed CONTEXT.md D-01..D-32 and PATTERNS.md A..K exactly. Three implementation-level judgements that were not pre-locked:

1. **Single-file layout for `spec.rs` (not split into `spec/mod.rs + validate.rs + builder.rs`).** CONTEXT.md D-"Claude's Discretion" explicitly allowed either. ~600 LoC stays readable in one file and keeps cross-referencing between `from_json`, `validate_structure`, and `SpecBuilder::build` trivial.
2. **Duplicate-ID detection via sentinel string** (`__FERRO_DUPLICATE_ID__` smuggled through `serde::de::Error::custom`). Alternative would be a forked `serde_json::Map` with custom deserialization at the `SpecWire.elements` level — this is simpler and kept all logic in-crate.
3. **`ElementBuilder::build` is `pub(crate)`.** Plan specified this; calling `.build()` on an `ElementBuilder` from outside the crate is nonsensical since there's no public path that consumes a bare `Element` — always via `SpecBuilder::element`.

## Deviations from Plan

None — plan executed exactly as written.

One minor mechanical addition: a `#[allow(clippy::new_ret_no_self)]` attribute on `Element::new`. The plan's `<interfaces>` block locks the signature as `pub fn new(type_name: impl Into<String>) -> ElementBuilder` (not `Self`). Clippy's `new_ret_no_self` lint fires on this and CI enforces `-D warnings`. A local `allow` with a rustdoc note explaining the fluent-builder rationale is the minimum-surface fix — documented at spec.rs line 177.

## Issues Encountered

None. Initial clippy run flagged `new_ret_no_self` (noted above); fixed inline before the Task 1 commit.

## Self-Check: PASSED

**Files verified present:**
- `ferro-json-ui/src/spec.rs` FOUND
- `ferro-json-ui/tests/round_trip.rs` FOUND
- `ferro-json-ui/tests/reject.rs` FOUND
- 7/7 ok fixtures FOUND
- 11/11 reject fixtures FOUND

**Commits verified:**
- `71608b0d` (Task 1 — feat) FOUND in git log
- `c89481df` (Task 2 — test) FOUND in git log

**Acceptance gates (run 2026-04-18):**
- `cargo build --all-targets` -> 0
- `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` -> 0
- `cargo fmt --all -- --check` -> 0
- `cargo test -p ferro-json-ui --lib` -> 488 passed, 0 failed
- `cargo test -p ferro-json-ui --test round_trip` -> 8 passed, 0 failed
- `cargo test -p ferro-json-ui --test reject` -> 11 passed, 0 failed
- `cargo test --all-features` -> 0 failures workspace-wide

## Workspace Status

Compiles green. v1 surface (`JsonUiView`, `Component`, `ComponentNode`, `SCHEMA_VERSION = "ferro-json-ui/v1"`) untouched. No regressions in pre-existing 471 ferro-json-ui unit tests.

## Next Phase Readiness

- Plan 02 (delete v1, unalias SCHEMA_VERSION) is unblocked. It needs to:
  1. Remove `pub mod view` and `view.rs`
  2. Remove the v1 `pub use view::{JsonUiView, SCHEMA_VERSION}` line in lib.rs
  3. Change `SCHEMA_VERSION as SCHEMA_VERSION_V2` to the unaliased `SCHEMA_VERSION` in the v2 re-export block
  4. Rewrite callers in `framework/src/json_ui/mod.rs`, `app/`, `ferro-mcp` templates
- Phase 116 (flat-element renderer) has the Spec/Element types it needs to consume
- Phase 117 (catalog) can build on Element.type_name being an unrestricted string

---
*Phase: 115-spec-v2-data-structures*
*Completed: 2026-04-18*
