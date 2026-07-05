---
phase: 255-pos-runtime-modules-double-submit-protection
plan: 05
subsystem: ui
tags: [ferro-json-ui, runtime, double-submit, form-guards, POS, SC-4, ES5, docs]

# Dependency graph
requires:
  - phase: 255-04
    provides: setupNumpad/setupFilters runtime modules + SC-1/2/3 inline tests + mod.rs wiring pattern

provides:
  - ferro-json-ui/src/component.rs: ButtonProps.disable_on_submit additive Option<bool>
  - ferro-json-ui/src/render/atoms.rs: render_button_inner emits data-disable-on-submit when Some(true)
  - ferro-json-ui/src/runtime/form_guards.rs: double-submit guard inside setupFormGuards + data-numpad-input in initNumberGuard
  - ferro-json-ui/src/runtime/mod.rs: SC-4 inline test (runtime_wires_disable_on_submit)
  - app/src/views/cassa.json: btn_confirm carries disable_on_submit:true
  - docs/src/features/write-kernel.md: Double-submit protection for forms section

affects:
  - 256 (render functions rely on data-numpad-input being in the number guard)
  - Any form with a submit button that opts in to disable_on_submit

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Additive Option<bool> serde field with skip_serializing_if = Option::is_none (established pattern)
    - Conditional HTML attribute emission via let attr = if Some(true) { " attr" } else { "" }
    - btn._submitted on-element flag (not closure variable) for bfcache-safe guard reset
    - submit event binding (not click) for double-submit prevention (D-14)
    - pageshow + event.persisted bfcache restore handler (D-15)
    - ES5 var/function style throughout (no arrows, template literals, let/const in JS)

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/projection/component_map.rs
    - ferro-json-ui/src/runtime/form_guards.rs
    - ferro-json-ui/src/runtime/mod.rs
    - app/src/views/cassa.json
    - docs/src/features/write-kernel.md

key-decisions:
  - "D-15: submitted flag lives on btn element (btn._submitted), not a closure variable, so the pageshow bfcache handler can reset it and the next submit fires normally"
  - "D-13: double-submit guard lives INSIDE setupFormGuards (no new setup function, no new dispatcher entry)"
  - "D-14: binding is on the form submit event, never on button click"
  - "D-16: disable_on_submit is additive Option<bool>; absent or Some(false) produces no data-attribute"
  - "CSS regen: no change — opacity-50/cursor-not-allowed already in bundle from prior form_guards.rs initTextEqualsGuard/initNumberGuard"
  - "component_map.rs build_relationship_button_props: disable_on_submit: None (struct initializer fix, Rule 1)"

requirements-completed: [POS-08]

# Metrics
duration: ~15min
completed: 2026-07-05
---

# Phase 255 Plan 05: Double-Submit Protection (POS-08) Summary

**ButtonProps.disable_on_submit -> data-disable-on-submit in render_button; setupFormGuards extended with a bfcache-safe submit-event guard; data-numpad-input added to initNumberGuard; /cassa demo wired; write-kernel.md documents the layered client-guard + server-idempotency-key + PRG pattern. Full CI-exact gate green.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-07-05T13:24:20Z
- **Completed:** 2026-07-05T13:37:20Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

**Task 1 — ButtonProps.disable_on_submit + render_button emission (TDD):**
- RED: two failing tests added (`render_button_emits_disable_on_submit`, `render_button_omits_disable_on_submit_by_default`)
- GREEN: `pub disable_on_submit: Option<bool>` added to `ButtonProps` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
- `render_button_inner`: `let disable_on_submit_attr = if props.disable_on_submit == Some(true) { " data-disable-on-submit" } else { "" };` interpolated into the button format string
- Auto-fix (Rule 1): `build_relationship_button_props` struct literal in `component_map.rs` gained `disable_on_submit: None` to compile
- Both tests green; backward-compatible (absent field = absent attribute)

**Task 2 — Double-submit guard in form_guards.rs + SC-4 inline test (TDD):**
- RED: `runtime_wires_disable_on_submit` test in mod.rs asserts bundle contains `data-disable-on-submit`
- GREEN (form_guards.rs):
  - `initNumberGuard`: added `var numpadInputs = form.querySelectorAll('input[data-numpad-input]');` + merge loop (D-05)
  - `setupFormGuards`: double-submit block added after the existing guard loop
    - Collects all `button[data-disable-on-submit]` and calls `initDisableOnSubmit` per button
    - `initDisableOnSubmit`: resolves form via `btn.closest('form')` fallback to `getElementById(btn.getAttribute('form'))`, binds submit event, first submit sets `btn._submitted = true` + disables; second submit `e.preventDefault()`
    - `window.addEventListener('pageshow', ...)`: if `e.persisted`, resets `_submitted = false` + re-enables all disable-on-submit buttons (D-15)
  - All ES5 style (var/function, no arrows, no template literals)
- SC-4 test green; no new setup function; no new dispatcher entry; 753 unit tests pass

**Task 3 — /cassa demo + docs + CSS regen + CI gate:**
- `app/src/views/cassa.json`: `btn_confirm` gains `"disable_on_submit": true`
- `docs/src/features/write-kernel.md`: new section "Double-submit protection for forms" — three-layer pattern (client guard / server dedupe idempotency_key / PRG), summary table, frames client guard as UX affordance (T-255-11)
- CSS regen: `ferro-base.css` unchanged (all classes already in bundle)
- Full CI-exact gate: `cargo fmt --all -- --check` + `cargo clippy --all --all-targets --all-features -- -D warnings` + `cargo test --all-features` + `cargo doc --no-deps` — all green
- SC-0 global grep: zero hits (no regression)

## Task Commits

| Task | Type | Hash | Description |
|------|------|------|-------------|
| Task 1 RED | test | 5c488201 | Failing tests for disable_on_submit |
| Task 1 GREEN | feat | 976d849e | ButtonProps.disable_on_submit + render_button emission |
| Task 2 RED | test | a24f241f | Failing SC-4 inline test in mod.rs |
| Task 2 GREEN | feat | 8ac85ae0 | Double-submit guard + data-numpad-input |
| Task 3 | feat | bd21cefb | /cassa demo + write-kernel.md docs + CI gate |

## Files Created/Modified

- `ferro-json-ui/src/component.rs` — `ButtonProps.disable_on_submit: Option<bool>`
- `ferro-json-ui/src/render/atoms.rs` — `data-disable-on-submit` attribute emission + 2 tests
- `ferro-json-ui/src/projection/component_map.rs` — `disable_on_submit: None` in struct init (Rule 1 fix)
- `ferro-json-ui/src/runtime/form_guards.rs` — double-submit guard + `data-numpad-input` in number guard
- `ferro-json-ui/src/runtime/mod.rs` — SC-4 `runtime_wires_disable_on_submit` test
- `app/src/views/cassa.json` — `btn_confirm` gains `disable_on_submit: true`
- `docs/src/features/write-kernel.md` — "Double-submit protection for forms" section

## Decisions Made

- `btn._submitted` stored on the button element itself (not a closure variable) so the `pageshow` bfcache handler can find and reset it — this is the D-15 safety property: navigating back clears the guard.
- Double-submit guard lives inside `setupFormGuards` (D-13): no new setup function, no new dispatcher entry, no drift-list changes needed in mod.rs.
- The client guard is explicitly framed as a UX affordance (T-255-11); `dispatch_write` idempotency is the authoritative dedupe.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing struct field in component_map.rs**
- **Found during:** Task 1 GREEN (cargo compile)
- **Issue:** `build_relationship_button_props` constructs `ButtonProps` with an explicit field list; adding `disable_on_submit` to the struct caused `E0063: missing field` at the call site.
- **Fix:** Added `disable_on_submit: None` to the struct literal in `component_map.rs:353`.
- **Files modified:** `ferro-json-ui/src/projection/component_map.rs`
- **Commit:** 976d849e (included in GREEN commit)

## Known Stubs

None — the double-submit guard is fully wired. The server-side `idempotency_key` pattern is documented; wiring it into a specific form is the responsibility of the consumer handler, not this framework phase.

## Threat Flags

No new attack surface beyond the plan's threat model. T-255-10 and T-255-11 are addressed:
- **T-255-10** (double-submit / replayed POST): mitigated by the layered pattern — client guard (data-disable-on-submit), server dedupe (dispatch_write idempotency_key), PRG.
- **T-255-11** (client-guard bypass): documented and transferred — the write-kernel.md section explicitly states the client guard is a UX affordance and the idempotency hook is the authoritative control.

## Self-Check

### Files exist:

- `ferro-json-ui/src/component.rs` with `pub disable_on_submit: Option<bool>` ✓
- `ferro-json-ui/src/render/atoms.rs` with `data-disable-on-submit` in `render_button_inner` ✓
- `ferro-json-ui/src/render/atoms.rs` with `render_button_emits_disable_on_submit` test ✓
- `ferro-json-ui/src/render/atoms.rs` with `render_button_omits_disable_on_submit_by_default` test ✓
- `ferro-json-ui/src/runtime/form_guards.rs` with `data-numpad-input` in `initNumberGuard` ✓
- `ferro-json-ui/src/runtime/form_guards.rs` with `button[data-disable-on-submit]` selector ✓
- `ferro-json-ui/src/runtime/form_guards.rs` with `addEventListener('submit'` ✓
- `ferro-json-ui/src/runtime/form_guards.rs` with `persisted` (bfcache reset) ✓
- `ferro-json-ui/src/runtime/form_guards.rs` with `_submitted = false` (2 occurrences) ✓
- `ferro-json-ui/src/runtime/mod.rs` with `runtime_wires_disable_on_submit` test ✓
- `app/src/views/cassa.json` with `"disable_on_submit": true` on btn_confirm ✓
- `docs/src/features/write-kernel.md` with "Double-submit protection for forms" section ✓
- `docs/src/features/write-kernel.md` with `idempotency_key` ✓

### Commits exist:

- `5c488201` ✓
- `976d849e` ✓
- `a24f241f` ✓
- `8ac85ae0` ✓
- `bd21cefb` ✓

### Tests:

- `cargo test -p ferro-json-ui --all-features`: 753 tests, 0 failed ✓
- `render_button_emits_disable_on_submit` ... ok ✓
- `render_button_omits_disable_on_submit_by_default` ... ok ✓
- `runtime_wires_disable_on_submit` ... ok ✓
- Full workspace `cargo test --all-features`: 0 failed ✓

## Self-Check: PASSED

---
*Phase: 255-pos-runtime-modules-double-submit-protection*
*Completed: 2026-07-05*
