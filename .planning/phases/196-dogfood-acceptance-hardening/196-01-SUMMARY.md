---
phase: 196-dogfood-acceptance-hardening
plan: 01
subsystem: ferro-mcp
tags: [checkpoint-projection, cap-reduction, SC-3, D-05]
dependency_graph:
  requires: []
  provides: [MAX_NEXT_STEPS const, next_steps capped at 5, next_steps_cap_at_five test]
  affects: [ferro-mcp/src/tools/checkpoint_projection.rs, docs/src/agents/checkpoint-projection.md]
tech_stack:
  added: []
  patterns: [named const for magic number, SC-3 test pinning]
key_files:
  created: []
  modified:
    - ferro-mcp/src/tools/checkpoint_projection.rs
    - docs/src/agents/checkpoint-projection.md
decisions:
  - "Cap value expressed via MAX_NEXT_STEPS const — not an inline literal — so search-and-replace is impossible to miss"
  - "Single renamed test (not a new test alongside the old one) — one test, one assertion, no duplication"
metrics:
  duration: ~5 min
  completed: 2026-06-10
  tasks_completed: 3
  files_modified: 2
requirements: [CHK-10]
---

# Phase 196 Plan 01: Reduce next_steps Cap 10 → 5 Summary

**One-liner:** `MAX_NEXT_STEPS: usize = 5` const + cap-at-5 guard + `next_steps_cap_at_five` test satisfying SC-3 across code and docs atomically.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2 | Introduce MAX_NEXT_STEPS const, change cap guard, update docstrings, rename test | b1ad1726 | ferro-mcp/src/tools/checkpoint_projection.rs |
| 3 | Update agent-facing doc cap references | 92303a58 | docs/src/agents/checkpoint-projection.md |

## Changes Made

### ferro-mcp/src/tools/checkpoint_projection.rs

- **Line 71** (Verdict doc): `cap 10` → `cap 5`
- **Above `aggregate_next_steps`**: Added `/// Maximum number of ranked next_steps returned in a verdict.\nconst MAX_NEXT_STEPS: usize = 5;`
- **Docstring** (was line 737): `Cap at 10.` → `Cap at 5.`
- **Cap guard** (was line 763): `result.len() == 10` → `result.len() == MAX_NEXT_STEPS`
- **Test renamed**: `next_steps_cap_at_10` → `next_steps_cap_at_five`; body uses 7 findings and asserts `== 5`

### docs/src/agents/checkpoint-projection.md

- **Line 61** (Verdict table): `capped at 10` → `capped at 5`
- **Line 125** (next_steps section): `capped at 10 entries` → `capped at 5 entries`

## Verification

- `grep -c "== 10" ferro-mcp/src/tools/checkpoint_projection.rs` → 0
- `grep -rc "capped at 10|cap 10|Cap at 10|10 entries"` across both files → 0
- `cargo test -p ferro-mcp checkpoint_projection` → 38 passed; 0 failed
- `cargo fmt --all -- --check` → clean
- `cargo clippy --all --all-targets -- -D warnings` → clean

## Deviations from Plan

None — plan executed exactly as written. Tasks 1 and 2 were committed together since they touch the same file and form one atomic unit (const + guard + test rename).

## Known Stubs

None.

## Threat Flags

None. This plan changes a numeric constant, a test, and documentation in read-only introspection tooling. No new input surface, no auth, no data egress.

## Self-Check: PASSED

- [x] `ferro-mcp/src/tools/checkpoint_projection.rs` — modified and committed at b1ad1726
- [x] `docs/src/agents/checkpoint-projection.md` — modified and committed at 92303a58
- [x] Both commits verified in git log
