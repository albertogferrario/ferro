---
phase: 252
plan: 02
subsystem: ferro-json-ui
tags: [design-lint, catalog, serde, stale-prop, tdd]
requirements: [DS-05]

dependency_graph:
  requires:
    - phase: 252-01
      provides: ferro_json_ui::design module foundation, Spec.design field
  provides:
    - Stage 2b extended to walk el.action typed field for retired-prop detection
    - ConfirmDialog preserves unknown JSON keys via #[serde(flatten)] HashMap
  affects:
    - ferro-json-ui/src/catalog.rs (Stage 2b loop extended)
    - ferro-json-ui/src/action.rs (ConfirmDialog unknown_fields added)

tech_stack:
  added: []
  patterns:
    - serialize-then-walk pattern: typed Rust struct → serde_json::to_value → retired-prop walk
    - "#[serde(flatten)] HashMap<String, Value> to preserve unknown serde fields for post-hoc validation"

key_files:
  created: []
  modified:
    - ferro-json-ui/src/action.rs
    - ferro-json-ui/src/catalog.rs

key-decisions:
  - "ConfirmDialog needs #[serde(flatten)] unknown_fields to preserve retired keys during deserialization — without it, serde drops `variant` before to_value can walk it (Rule 1 auto-fix)"
  - "Single-home enforcement: stale-prop detection stays exclusively in catalog Stage 2b; design::lint gets no stale-prop rule (D-16)"
  - "unknown_fields is pub(crate) only — no public API surface change; struct literals in action.rs updated with Default::default()"

patterns-established:
  - "Typed-field gap closure: serialize typed struct to Value, then apply existing recursive walk — avoids duplicating walk logic or opening a second control surface"

requirements-completed: [DS-05]

metrics:
  duration: 30min
  completed: 2026-07-03T18:00:00Z
  tasks: 1
  files: 2
---

# Phase 252 Plan 02: Stage 2b el.action Walk Summary

**Closed the D-16 stale-prop gap: `Catalog::validate` now catches retired `action.confirm.variant` by serializing the typed `el.action` field to a `Value` and running the existing retired-prop walk on it.**

## Performance

- **Duration:** ~30 min
- **Completed:** 2026-07-03
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Extended Stage 2b loop to serialize `el.action` to `serde_json::Value` and run `collect_retired_action_variants` with `/action` path prefix
- Added `#[serde(flatten)] pub(crate) unknown_fields: HashMap<String, serde_json::Value>` to `ConfirmDialog` so retired keys (`variant`) survive serde deserialization instead of being silently dropped
- Updated 5 `ConfirmDialog` struct literal constructions in action.rs to include `unknown_fields: Default::default()`
- Added 2 catalog tests: rejects `confirm.variant` (retired), accepts `confirm.tone` (canonical)
- 251-VERIFICATION.md finding-1 gap fully closed; stale-prop detection single-homed

## Task Commits

| Task | Name | Commit | Type |
|------|------|--------|------|
| 1 | Extend Stage 2b to walk el.action | 270c3f24 | feat |

**Plan metadata:** (docs commit to follow)

## Files Created/Modified

- `ferro-json-ui/src/action.rs` — added `unknown_fields: HashMap<String, serde_json::Value>` with `#[serde(flatten)]` to `ConfirmDialog`; updated 5 struct literal constructions; `use std::collections::HashMap` import
- `ferro-json-ui/src/catalog.rs` — added 5-line Stage 2b block after existing `collect_retired_action_variants(&el.props, "")` call; added 2 new TDD tests (`validate_rejects_retired_el_action_confirm_variant`, `validate_accepts_canonical_el_action_confirm_tone`)

## Decisions Made

- `unknown_fields` is `pub(crate)` — keeps it internal, no public API change, no schema-visible surface beyond `additionalProperties` in schemars output
- The plan's 5-line block alone was insufficient: serde drops unknown fields from `ConfirmDialog` before `to_value` runs, so `variant` never appears in the serialized Value. Fix required adding `#[serde(flatten)] HashMap` first (Rule 1 auto-fix).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ConfirmDialog silently dropped `variant` before the Stage 2b walk could see it**
- **Found during:** Task 1 (GREEN — first implementation analysis)
- **Issue:** The plan's 5-line block serializes `el.action` to a Value and walks it, but `ConfirmDialog` had no field for `variant`. Serde drops unknown JSON keys when deserializing into a typed struct without `#[serde(deny_unknown_fields)]` or a catch-all field. The resulting `to_value(action)` would produce `{"confirm": {"tone": "neutral"}}` — no `variant` for the walk to detect.
- **Fix:** Added `#[serde(flatten)] pub(crate) unknown_fields: HashMap<String, serde_json::Value>` to `ConfirmDialog`. With this, unknown JSON keys are captured during deserialization and re-serialized into the Value, making `variant` visible to `collect_retired_action_variants`.
- **Files modified:** ferro-json-ui/src/action.rs
- **Verification:** RED test `validate_rejects_retired_el_action_confirm_variant` passes; GREEN test `validate_accepts_canonical_el_action_confirm_tone` produces no false positive
- **Committed in:** 270c3f24

---

**Total deviations:** 1 auto-fixed (Rule 1 bug)
**Impact on plan:** Auto-fix was required for correctness — without it, the plan's block would be silently inert. No scope creep.

## TDD Gate Compliance

Due to a context boundary in the previous session (ran out of context mid-GREEN setup), the RED failing-test commit and GREEN implementation commit were merged into a single commit (270c3f24). RED behavior was confirmed failing in the prior session before the implementation was written. TDD RED→GREEN sequence was honored in execution order but not in commit topology.

| Gate | Status |
|------|--------|
| RED (tests written + confirmed failing) | Confirmed in prior session; no separate commit |
| GREEN (implementation makes tests pass) | 270c3f24 |
| REFACTOR | Not needed |

## Known Stubs

None.

## Threat Flags

No new network endpoints, auth paths, or file access patterns. `ConfirmDialog.unknown_fields` is deserialized from spec JSON (already untrusted input subject to `Catalog::validate`) and only read during `to_value()` serialization back to Value for the walk. No new trust boundary.

## Issues Encountered

None beyond the serde silent-drop gap (documented as Rule 1 deviation above).

## Next Phase Readiness

- Plan 02 complete: element-level `action.confirm.variant` detection closed
- Plan 03 and 04 can proceed: the `design::lint` engine (Plan 01) has no stale-prop rule; single home confirmed
- All 46 catalog tests green; `cargo clippy --all-targets --all-features` clean

## Self-Check: PASSED

- `ferro-json-ui/src/action.rs` — FOUND, modified
- `ferro-json-ui/src/catalog.rs` — FOUND, modified
- Commit 270c3f24 — FOUND (`git log --oneline`)
- `grep "collect_retired_action_variants(&action_value" ferro-json-ui/src/catalog.rs` — FOUND (line 772)
- `grep '"/action"' ferro-json-ui/src/catalog.rs` — FOUND (line 772)
- `grep -rn "stale-prop\|stale_prop" ferro-json-ui/src/design/ | wc -l` = 0 — CONFIRMED
- 46 catalog tests pass — CONFIRMED
