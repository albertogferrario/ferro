---
phase: 263-projection-native-inertia-substrate
plan: 02
subsystem: framework
tags: [rust, guards, authorization, mcp, projections, visibility-filter]

requires:
  - phase: 263-01
    provides: schema_contract pure derivation in ferro-projections (SUBST-01)

provides:
  - framework::permitted_actions(service, evaluated_guards) -> Vec<String> — feature-gated on projections, re-exported as ferro::permitted_actions
  - Single guard-visibility evaluation site (grep-verifiable): only framework/src/permitted_actions.rs
  - ferro-mcp-server render_action_tool calls framework::permitted_actions (inline loop deleted)
  - guard_visibility_unchanged_after_lift regression test in renderer.rs

affects:
  - 263-03 (projection_read relocation — also touches framework/src/lib.rs)
  - 263-04 (Inertia from_projection — calls ferro::permitted_actions)
  - 263-05 (single-source parity tests)

tech-stack:
  added: []
  patterns:
    - "Guard-visibility filter as a standalone framework fn: permitted_actions(service, evaluated_guards) — not inline in consumer code"
    - "Feature-gate new framework modules on projections mirroring the write module pattern"
    - "Confirmation-gated render fns unified to same single evaluation site via service lookup before guard check"

key-files:
  created:
    - framework/src/permitted_actions.rs
  modified:
    - framework/src/lib.rs
    - ferro-mcp-server/src/renderer.rs

key-decisions:
  - "All three guard-loop sites in renderer.rs (render_action_tool + render_request_confirm_tool + render_confirm_tool) converted to use permitted_actions — the acceptance criterion grep for == Some(&false) applies across all sites, not only the main function"
  - "render_confirm_tool made _services/_service_name params live (service lookup added) to enable permitted_actions call — consistent with render_request_confirm_tool"
  - "permitted_actions rustdoc explicitly marks it as a VISIBILITY filter, not an authorization gate — per T-263-03 threat mitigation"

patterns-established:
  - "Single guard-evaluation site: all guard-visibility filtering in ferro-mcp-server routes through framework::permitted_actions"
  - "Deviation Rule 2 applied: render_confirm_tool's formerly-unused _services/_service_name params activated for consistency — no inline loop anywhere in renderer.rs"

requirements-completed: [SUBST-02]

duration: 5min
completed: 2026-07-27
---

# Phase 263 Plan 02: Guard-Visibility Lift to framework::permitted_actions Summary

**Guard-visibility filter extracted from ferro-mcp-server into `framework::permitted_actions(service, evaluated_guards) -> Vec<String>`, deleting all three inline loops in renderer.rs and leaving exactly one `== Some(&false)` evaluation site in the framework.**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-07-27T12:10:11Z
- **Completed:** 2026-07-27T12:15:36Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Created `framework/src/permitted_actions.rs` with `pub fn permitted_actions(service: &ServiceDef, evaluated_guards: &HashMap<String, bool>) -> Vec<String>`, feature-gated on `projections`, re-exported from `framework/src/lib.rs` as `ferro::permitted_actions`
- Three unit tests cover all deny-semantics cases: `hides_action_when_guard_is_false`, `absent_guard_key_allows_action`, `explicit_true_allows_action`
- Deleted all three inline `for precondition in &action.preconditions { if ... == Some(&false) }` loops from `ferro-mcp-server/src/renderer.rs` — `render_action_tool`, `render_request_confirm_tool`, `render_confirm_tool`
- Added `guard_visibility_unchanged_after_lift` regression test proving the refactor preserves exact prior filtering behavior

## Task Commits

1. **Task 1: Create framework::permitted_actions with inline unit tests** - `0d0cd4fb` (feat)
2. **Task 2: Refactor render_action_tool to call framework::permitted_actions + pin MCP no-regression** - `91745759` (refactor)

## Files Created/Modified

- `framework/src/permitted_actions.rs` — new module: `pub fn permitted_actions(...)`, 3 unit tests, rustdoc visibility-only warning
- `framework/src/lib.rs` — added `#[cfg(feature = "projections")] pub mod permitted_actions` + `pub use` re-export
- `ferro-mcp-server/src/renderer.rs` — `use ferro_rs::permitted_actions` import; all three inline guard loops replaced; `GuardDef` added to test imports; `guard_visibility_unchanged_after_lift` regression test added; `render_confirm_tool` params made live

## Decisions Made

- All three guard-loop sites converted (not only `render_action_tool`): the acceptance criterion `grep -q "== Some(&false)"` must return only the framework line across both `framework/src` and `ferro-mcp-server/src`, so all three sites in `renderer.rs` were converted.
- `render_confirm_tool`'s `_services`/`_service_name` params were activated (underscores removed, service lookup added) to enable the `permitted_actions` call consistently. This is a small Rule 2 fix — the old inline loop was already reading `action.preconditions` which is equivalent; the new call reads through the service, same data.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Converted render_request_confirm_tool and render_confirm_tool guard loops**
- **Found during:** Task 2 (renderer.rs refactor)
- **Issue:** Plan action explicitly named `render_action_tool` (lines 229-233) but the acceptance criterion requires `grep -q "== Some(&false)"` across `framework/src` AND `ferro-mcp-server/src` to return ONLY the framework line — two more inline loops existed in the `#[cfg(feature = "confirmation")]` functions
- **Fix:** Moved service lookup before guard check in `render_request_confirm_tool`; added service lookup to `render_confirm_tool` (making `_services`/`_service_name` params live); both now call `permitted_actions(service, &ctx.evaluated_guards)`
- **Files modified:** `ferro-mcp-server/src/renderer.rs`
- **Verification:** `grep -rn "== Some(&false)" framework/src ferro-mcp-server/src` returns exactly one line (framework)
- **Committed in:** `91745759` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 — missing consistency across confirmation-gated functions)
**Impact on plan:** Required for acceptance criterion compliance. No scope creep.

## Issues Encountered

- Package name mismatch: the `framework/` directory contains crate named `ferro-rs`, not `framework`. Initial `cargo test -p framework` failed; corrected to `cargo test -p ferro-rs`. Resolved immediately.

## Stub Tracking

None — no stub values or placeholder text in the created/modified files.

## Threat Flags

None — no new network endpoints, auth paths, file access, or schema changes introduced. The refactor moves logic between compilation units without changing the trust boundary or data flow.

## Self-Check

**Commits exist:**
- `0d0cd4fb` — feat(263-02): add framework::permitted_actions visibility filter
- `91745759` — refactor(263-02): lift guard-visibility loop out of render_action_tool into framework

**Files exist:**
- `framework/src/permitted_actions.rs` — created
- `framework/src/lib.rs` — modified (pub mod + pub use under projections feature)
- `ferro-mcp-server/src/renderer.rs` — modified (import + 3 loops replaced + regression test)

**Single evaluation site:**
- `grep -rn "== Some(&false)" framework/src ferro-mcp-server/src` → only `framework/src/permitted_actions.rs:29`

**Tests:**
- `cargo test -p ferro-rs --features projections permitted_actions` → 3/3 passed
- `cargo test -p ferro-mcp-server` → 58 unit + 14 integration = 72 passed, 0 failed

## Self-Check: PASSED

## Next Phase Readiness

- `ferro::permitted_actions` is exported and ready for the Inertia `from_projection` delivery helper (Plan 04)
- Plan 03 (`projection_read` relocation) also modifies `framework/src/lib.rs` — sequenced to Wave 2, no conflict with this plan's commits
- SUBST-02 satisfied: one guard-evaluation site, MCP `tools/list` output unchanged, `ferro::permitted_actions` available for Inertia wave

---
*Phase: 263-projection-native-inertia-substrate*
*Completed: 2026-07-27*
