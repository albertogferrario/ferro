---
phase: 181
plan: 07
status: complete
wave: 7
subsystem: cross-repo-audit
tags: [d-08-audit, phase-gate, gestiscilo, cross-repo, clippy, fmt]
commits:
  - ef4d3712
  - e38babd5
  - 3a3f86f0
  - d362223c
files_modified:
  - .planning/phases/181-json-ui-input-error-prop-inline-render/181-07-AUDIT.md
  - ferro-json-ui/src/render/form.rs
  - framework/src/http/multipart.rs
  - framework/src/http/request.rs
  - framework/src/json_ui/mod.rs
  - framework/tests/action_handler.rs
key-decisions:
  - "D-08 closed cleanly — Bucket B empty; no gestiscilo consumer reads attach_errors plural array shape"
  - "Manual UAT auto-approved as DEFERRED-TO-OPERATOR; documented as release-time gate per feedback_friction_loop_release_cadence.md"
  - "Phase gate required two pre-fixes: cargo fmt (Wave 2 commits lacked formatting run) + uninlined_format_args clippy lint in render_switch (peer_ring_class inlined into format string)"
  - "No #[allow(...)] suppressions added — all clippy issues fixed at root"
metrics:
  duration: "~25 minutes"
  completed: "2026-05-31T21:00:00Z"
  tasks: 3
  files: 6
---

# Phase 181 Plan 07: D-08 Cross-Repo Audit + Phase Gate Summary

D-08 cross-repo audit confirmed no gestiscilo consumer depends on the pre-fix `errors: Vec<String>` shape. Canonical pre-commit gate passed after two formatting/lint fixes inherited from Wave 2 commits. Phase 181 (all 7 plans) is complete pending operator browser UAT before ferro publish.

## What Was Built

### Task 1: D-08 Grep Audit (commit `ef4d3712`)

Created `.planning/phases/181-json-ui-input-error-prop-inline-render/181-07-AUDIT.md` with the D-08 cross-repo audit results.

Audit ran against gestiscilo-it at `/Users/alberto/repositories/gestiscilo-it/app/src/` (not the plan's assumed `../gestiscilo-it/` sibling path — the two repos are in separate namespace subdirectories under `/Users/alberto/repositories/`).

**Bucket A (non-issue):** 3 hits of the literal `"errors"` key in settings controller data objects. All three are top-level data keys containing per-field error strings (e.g. `"errors": {"name": req.validation_error("name"), ...}`), consumed by spec files via `{"$data": "/errors/field"}` path bindings through the `render_file` path (which already merges data before resolution). Not reading the `attach_errors` plural array output.

**Bucket B (BLOCKER if non-empty):** Empty. `rg '\.errors' app/src/` returned zero hits. D-08 closed cleanly.

**Bucket C (migration candidates):** 15 `.prop("error", json!({"$data": "/field_error"}))` sites, all in `cassa/products.rs`. These are the escape-hatch path ($data binding) that Phase 181 Fix A enables. All 15 will produce live inline error `<p>` elements once the ferro path-dep is vendored into gestiscilo. Migration to `JsonUi::render_validation_error` is a cosmetic follow-up, not a blocker.

Note: CONTEXT mentioned ~30 sites; current count is 15. Reduction is consistent with Phase 175 product-edit form refactoring.

### Task 2: Manual UAT Auto-approved with DEFERRED-TO-OPERATOR status (commit `ef4d3712`)

Per `--auto` mode handling: `checkpoint:human-verify` auto-approved. UAT cannot be executed from auto mode (requires browser + gestiscilo dev server + path-dep repoint).

`181-07-AUDIT.md` contains a `## Manual UAT — Representative Sample` section with explicit `DEFERRED-TO-OPERATOR` status for all 5 forms:

| # | Form | Status |
|---|------|--------|
| 1 | cassa/products edit (discovery surface) | DEFERRED-TO-OPERATOR |
| 2 | calendario/bookings new | DEFERRED-TO-OPERATOR |
| 3 | settings general | DEFERRED-TO-OPERATOR |
| 4 | staff member create/edit | DEFERRED-TO-OPERATOR |
| 5 | documenti upload (file input ring check) | DEFERRED-TO-OPERATOR |

Per `feedback_friction_loop_release_cadence.md`: UAT is a **release-time gate**, not a build-time blocker. Ferro must NOT publish Phase 181 until operator UAT is complete.

### Task 3: Pre-commit Gate (commits `e38babd5`, `3a3f86f0`, `d362223c`)

Pre-commit gate required two fixes before passing:

**Fix 1 — `cargo fmt` (`e38babd5`):** Wave 2 commits (Plans 03-06) were made without running `cargo fmt --all`. 5 files reformatted: `ferro-json-ui/src/render/form.rs`, `framework/src/http/multipart.rs`, `framework/src/http/request.rs`, `framework/src/json_ui/mod.rs`, `framework/tests/action_handler.rs`. Changes were pure formatting (expanded inline ternaries and long `assert!` chains per rustfmt style).

**Fix 2 — `cargo clippy` (`3a3f86f0`):** `uninlined_format_args` lint (`-D warnings`) triggered on `ferro-json-ui/src/render/form.rs:762` — the `peer_ring_class` local variable was passed as a positional arg to `format!` in `render_switch`. Fixed by inlining it as `{peer_ring_class}` directly in the format string. No `#[allow(...)]` suppressions added.

**Gate result:**

| Step | Outcome |
|------|---------|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all --all-targets -- -D warnings` | PASS |
| `cargo test --all-features` | PASS — **2812 passed, 0 failed, 437 ignored** |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] cargo fmt failures from Wave 2 commits**
- **Found during:** Task 3 (first gate run)
- **Issue:** Plans 03-06 committed without running `cargo fmt --all`. `cargo fmt --check` failed on 5 files with expanded ternary and assert! formatting.
- **Fix:** `cargo fmt --all` then separate commit before re-running the gate.
- **Files modified:** `ferro-json-ui/src/render/form.rs`, `framework/src/http/multipart.rs`, `framework/src/http/request.rs`, `framework/src/json_ui/mod.rs`, `framework/tests/action_handler.rs`
- **Commit:** `e38babd5`

**2. [Rule 1 - Bug] clippy uninlined_format_args in render_switch**
- **Found during:** Task 3 (first gate run)
- **Issue:** `render_switch` in Plan 05 used positional-arg format! for `peer_ring_class` — rejected by `clippy::uninlined_format_args` under `-D warnings`.
- **Fix:** Inline `peer_ring_class` as `{peer_ring_class}` directly in the format string at `form.rs:762`.
- **Files modified:** `ferro-json-ui/src/render/form.rs`
- **Commit:** `3a3f86f0`

### Scope Note

Gestiscilo-it is not at the plan's assumed sibling path `../gestiscilo-it/` relative to ferro. It is at `/Users/alberto/repositories/gestiscilo-it/`. The audit ran against the correct path; documented in AUDIT.md.

## Known Stubs

None. AUDIT.md captures the full D-08 verdict. The UAT deferral is documented as an explicit release-time gate, not a silent stub.

## Threat Flags

No new threat surface. This plan is verification-only (audit + gate). No new code paths introduced.

## Self-Check: PASSED

- `.planning/phases/181-json-ui-input-error-prop-inline-render/181-07-AUDIT.md`: FOUND
- Commit `ef4d3712` (Tasks 1+2): verified via git log
- Commit `e38babd5` (fmt fixes): verified via git log
- Commit `3a3f86f0` (clippy fix): verified via git log
- Commit `d362223c` (phase gate results): verified via git log
- `## Bucket A`, `## Bucket B`, `## Bucket C` sections in AUDIT.md: confirmed (4 `##` sections minimum)
- `## Manual UAT` section in AUDIT.md: confirmed
- `## Phase Gate` section in AUDIT.md: confirmed
- Test gate: 2812 passed, 0 failed
