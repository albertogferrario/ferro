---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
plan: 01
subsystem: ui
tags: [json-ui, serde, iteration, wire-format, ferro-json-ui]

# Dependency graph
requires:
  - phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
    provides: "Spec/Element/SpecBuilder v2 structural validator; serde round-trip test conventions"
provides:
  - "EachDirective wire-format type in ferro-json-ui::spec"
  - "Element.each: Option<EachDirective> field (serde rename \"$each\", skip_serializing_if)"
  - "Three serde round-trip tests covering EachDirective and Element.$each presence/absence"
affects:
  - 163-02
  - 163-03
  - 163-04

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dollar-prefixed directive keys (\"$each\") for spec-level metadata on Element entries — convention shared with $data/$expr from prior phases"
    - "Optional iteration directive expressed as a sibling field on Element, not a wrapper enum — keeps the Element shape stable for catalog validators"

key-files:
  created: []
  modified:
    - "ferro-json-ui/src/spec.rs"
    - "ferro-json-ui/src/catalog.rs"
    - "ferro-json-ui/src/render/atoms.rs"
    - "ferro-json-ui/src/render/data.rs"
    - "ferro-json-ui/src/render/form.rs"
    - "ferro-json-ui/src/render/mod.rs"

key-decisions:
  - "EachDirective.path is JSONPath-style slash-separated; semantics align with the existing data::resolve_path used by Visibility (no new path engine)"
  - "EachDirective.as_ uses serde rename(\"as\") rather than renaming the Rust field — keeps the canonical wire keyword while avoiding the Rust keyword collision"
  - "Element.each lives as a sibling Optional field, not a wrapper enum, because catalog/validator code already pattern-matches Element directly; introducing an enum would force a worse downstream refactor for zero gain"
  - "Plan 01 leaves the directive inert (no resolver) — expansion lands in Plan 03 per CONTEXT D-01; resource bounds and reserved-name validation are deferred to Plans 03/04 per the threat register"

patterns-established:
  - "Wire-format directive expansion order: spec-level Option<T> field with rename(\"$key\") + skip_serializing_if to keep no-op specs free of metadata noise"
  - "TDD gate for serde-round-trip features: RED commit asserts both the dot-access (parsed.each) and the wire key presence (reserialized.get(\"$each\")) so the field is end-to-end visible, not just deserializable"

requirements-completed: []

# Metrics
duration: 4min
completed: 2026-05-16
---

# Phase 163 Plan 01: Add `$each` iteration directive wire-format type — Summary

**Adds `EachDirective` struct and `Element.each: Option<EachDirective>` to `ferro-json-ui::spec`, enabling JSON specs to carry `"$each": { "path": "/orders", "as": "order" }` through serde round-trip; resolver expansion is the next step.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-16T17:20:35Z
- **Completed:** 2026-05-16T17:24:33Z
- **Tasks:** 1 (TDD)
- **Files modified:** 6

## Accomplishments

- Introduced `EachDirective { path: String, as_: String }` with `#[serde(rename = "as")]` so the on-wire key matches `163-CONTEXT.md` D-01 verbatim.
- Added `Element.each: Option<EachDirective>` with `#[serde(rename = "$each", default, skip_serializing_if = "Option::is_none")]`; existing five Element fields untouched.
- Threaded the new field through `ElementBuilder` (struct field + `Element::new()` default + `build()`) so all existing Spec-builder call sites continue compiling without changes.
- Added three inline serde round-trip tests (`each_directive_round_trips`, `element_with_each_round_trips`, `element_without_each_omits_field`) covering positive presence, key naming, and skip-serializing semantics.

## Task Commits

1. **Task 1 RED — Failing serde tests for `$each`** — `99371816` (test)
2. **Task 1 GREEN — `EachDirective` type + `Element.each` field + builder/literal updates** — `6c5f9fe7` (feat)

REFACTOR was evaluated and skipped — the introduced code is already minimal (struct + one field + builder threading); no duplication to consolidate.

## Files Created/Modified

- `ferro-json-ui/src/spec.rs` — Added `EachDirective` struct, `Element.each` field, `ElementBuilder.each` field with default-None threading through `build()`, three serde round-trip tests.
- `ferro-json-ui/src/catalog.rs` — Updated three Element-struct-literal test helpers to include `each: None` (Rule 3 — required for compilation; no behavior change).
- `ferro-json-ui/src/render/atoms.rs` — Updated one Element-struct-literal test helper to include `each: None`.
- `ferro-json-ui/src/render/data.rs` — Updated one `mk_element` test helper to include `each: None`.
- `ferro-json-ui/src/render/form.rs` — Updated one `mk_element` test helper to include `each: None`.
- `ferro-json-ui/src/render/mod.rs` — Updated one `mk_element` test helper to include `each: None`.

## Decisions Made

- **Element shape: add a sibling field, not a wrapper enum.** A wrapper enum (e.g. `Element::Single(ElementBody) | Element::Each(EachWrap)`) would invalidate every downstream pattern-match in catalog/render. Sibling field keeps the existing surface stable and matches how `action` and `visible` are already modeled.
- **Resolver expansion deliberately out of scope.** Plan 01 ships only the wire-format type. The directive is documented as inert; the rustdoc on `EachDirective` and the threat register both flag the resolve-time concerns that Plans 03/04 will address. Mixing wire-format and resolver in the same plan would have made the diff harder to review.
- **Reuse `data::resolve_path` semantics for `EachDirective.path`** — no new path grammar is introduced. This matches D-04 in `163-CONTEXT.md` ("reuse the existing visibility expression evaluator … do NOT add a parallel expression engine") even though D-04 is scoped to `$if`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Six Element-struct-literal test helpers required `each: None` to compile**
- **Found during:** Task 1 GREEN — initial `cargo build -p ferro-json-ui --tests` after adding `Element.each`
- **Issue:** Adding a public field to `Element` is a breaking change for every `Element { … }` struct literal in the crate. The plan called out only `spec.rs`, but five other test helpers (one each in `catalog.rs` × 3 sites, `render/atoms.rs`, `render/data.rs`, `render/form.rs`, `render/mod.rs`) instantiate `Element` directly and would have failed compilation.
- **Fix:** Added `each: None,` to each of the six sites. No behavior change; tests still construct the same shape.
- **Files modified:** `ferro-json-ui/src/catalog.rs`, `ferro-json-ui/src/render/atoms.rs`, `ferro-json-ui/src/render/data.rs`, `ferro-json-ui/src/render/form.rs`, `ferro-json-ui/src/render/mod.rs`
- **Verification:** `cargo build -p ferro-json-ui --tests --all-features` exits 0; the full ferro-json-ui suite (432 unit tests + 11 + 8 + 5 doctest groups) remains green.
- **Committed in:** `6c5f9fe7` (Task 1 GREEN commit — single atomic change so the type addition and its call-site updates land together).

---

**Total deviations:** 1 auto-fixed (1 Rule 3 — blocking, structurally required for compilation)
**Impact on plan:** Mechanical struct-literal update only. Zero scope creep — the same six call sites would have needed the change regardless of who made it; bundling them with the GREEN commit keeps history bisectable.

## Issues Encountered

None. The plan's TDD gate sequence (RED test commit → GREEN implementation commit) ran clean. Acceptance-criteria greps and the full test suite both passed without iteration.

## TDD Gate Compliance

- `test(163-01): …` commit at `99371816` (RED gate — three new tests failed to compile as required).
- `feat(163-01): …` commit at `6c5f9fe7` (GREEN gate — same three tests now pass; full suite green).
- REFACTOR gate evaluated and intentionally skipped (no duplication or simplification opportunity in the minimal diff).

## User Setup Required

None.

## Next Phase Readiness

- **Plan 02 (`$if` directive):** Element.each precedent — sibling Optional field with `#[serde(rename = "$key", skip_serializing_if = "Option::is_none")]` — is the pattern Plan 02 should follow for `$if`.
- **Plan 03 (resolver):** `EachDirective.path` and `EachDirective.as_` are publicly readable; the resolver in Plan 03 can call `data::resolve_path(spec.data, &each.path)` directly and bind `each.as_` as the loop-variable scope prefix.
- **Plan 04 (validation):** Validator can pattern-match on `Element.each.is_some()` and read `each.as_` to enforce the reserved-name list (D-12 in CONTEXT). `EachDirective`'s rustdoc already cites the future `SpecError::EachAsReservedName` variant by name.

Threat surface unchanged — the directive is inert at Plan 01 (T-163-01-01 mitigation explicitly deferred to Plan 04; T-163-01-02 accepted per the plan).

## Self-Check: PASSED

- `ferro-json-ui/src/spec.rs` — FOUND (modified)
- `ferro-json-ui/src/catalog.rs` — FOUND (modified)
- `ferro-json-ui/src/render/atoms.rs` — FOUND (modified)
- `ferro-json-ui/src/render/data.rs` — FOUND (modified)
- `ferro-json-ui/src/render/form.rs` — FOUND (modified)
- `ferro-json-ui/src/render/mod.rs` — FOUND (modified)
- Commit `99371816` (RED) — FOUND in `git log --oneline`
- Commit `6c5f9fe7` (GREEN) — FOUND in `git log --oneline`

---
*Phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test*
*Plan: 01*
*Completed: 2026-05-16*
