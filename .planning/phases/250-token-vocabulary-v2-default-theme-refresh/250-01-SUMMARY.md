---
phase: 250-token-vocabulary-v2-default-theme-refresh
plan: "01"
subsystem: ferro-theme, ferro-json-ui
tags: [tokens, css, tailwind, motion, design-system]
requirements: [DS-01]

dependency_graph:
  requires: []
  provides: [token-vocabulary-v2, ferro-base-css-v2-utilities, motion-reduced-motion-block]
  affects: [ferro-theme, ferro-json-ui]

tech_stack:
  added: []
  patterns:
    - "var(--semantic-name, <fallback>) bridge in @theme inline for v1-safe new tokens"
    - "@source inline safelist for runtime-concatenated utility names"
    - "prefers-reduced-motion block with 0.01ms (keeps transitionend/animationend firing)"

key_files:
  created: []
  modified:
    - ferro-theme/src/token.rs
    - ferro-json-ui/assets/input.css
    - ferro-json-ui/assets/ferro-base.css
    - ferro-json-ui/src/assets/mod.rs

decisions:
  - "Motion bridge uses var(--motion-duration-fast, 120ms) fallback pattern — v1 themes that omit the new slots still render correctly (SC1/D-05)"
  - "No --spacing entry in @theme inline — Tailwind v4 generates spacing utilities natively from var(--spacing) without a bridge entry (Pitfall 4 avoided)"
  - "0.01ms for prefers-reduced-motion (not 0ms) preserves transitionend/animationend listener firing (D-07)"

metrics:
  duration: "500 seconds (~8 minutes)"
  completed: "2026-07-03"
  tasks_completed: 3
  files_modified: 4
---

# Phase 250 Plan 01: Token Vocabulary v2 Surface Summary

Expose the 7 new DS-01 token constants in Rust and wire them into the Tailwind
pipeline with v1-safe fallbacks; regenerated stylesheet carries the new utilities
and a `prefers-reduced-motion` collapse block.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add 7 v2 token constants and grow ALL_TOKENS to 30 | f26fa6d0 | ferro-theme/src/token.rs |
| 2 | Bridge new tokens into Tailwind namespaces + safelist + reduced-motion | c755e714 | ferro-json-ui/assets/input.css |
| 3 | Regenerate ferro-base.css and pin regression test | 4ea70c60 | ferro-json-ui/assets/ferro-base.css, ferro-json-ui/src/assets/mod.rs |

## Verification Results

- `cargo test -p ferro-theme`: 17/17 passed — `token::tests::all_tokens_len_is_30` green
- `cargo test -p ferro-json-ui`: `assets::tests::ferro_base_css_contains_motion_duration_fallback` green
- Fmt: `cargo fmt --all -- --check` passes
- Post-regen grep assertions: duration-fast, ease-base, ring-ring, font-display, var(--motion-duration-fast,), prefers-reduced-motion — all found in ferro-base.css

## Decisions Made

1. **v1-safe fallback pattern**: Motion tokens use `var(--motion-duration-fast, 120ms)` in `@theme inline` (not the self-referential `var(--motion-duration-fast)`) — v1 themes that do not define `--motion-*` still resolve the generated utilities. This is the SC1 structural guarantee.

2. **No `--spacing` bridge entry**: Tailwind v4 generates spacing utilities natively as `calc(var(--spacing) * N)` without needing a `@theme inline` entry. Adding one would create a circular reference (RESEARCH Pitfall 4).

3. **0.01ms for reduced-motion**: Using `0.01ms` instead of `0ms` or `transition: none` keeps `transitionend`/`animationend` event listeners firing, preventing JS interaction bugs (D-07).

## Deviations from Plan

### Environmental Issue (non-blocking)

**ENOSPC during crate-scoped sweep**

The plan calls for a crate-scoped sweep (`cargo test -p ferro-theme -p ferro-json-ui`) after all tasks. The disk volume reached 100% capacity (418GiB/460GiB) during the sweep, preventing the 14MB `ferro-json-ui` test binary from being linked.

- **Impact**: The sweep could not be run as a combined command.
- **Mitigation**: Both crate test suites ran individually before the binary was deleted. All acceptance criteria tests passed:
  - `token::tests::all_tokens_len_is_30` — confirmed passing in the full `ferro-theme` 17-test run
  - `assets::tests::ferro_base_css_contains_motion_duration_fallback` — confirmed passing in the `ferro-json-ui` run
- **Resolution**: Plan 03's full CI gate (`cargo test --all-features`) will run on a clean CI runner with adequate disk. No code quality issue exists.

### Task 2 and 3 CSS commit split

The plan suggested committing `input.css` and regenerated `ferro-base.css` in the same commit. They were committed separately per the task structure (Task 2: `input.css`; Task 3: `ferro-base.css`). The CI check compares the final HEAD state, so both files are consistent at the point CI evaluates the branch.

## Known Stubs

None. All 7 new token constants are fully defined, the bridge entries resolve to real CSS values via fallback, and the regenerated stylesheet is the committed artifact.

## Threat Flags

None. The changes are CSS custom property constants and Rust string literals — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Self-Check

### Created files exist:
- N/A (no new files created)

### Modified files confirmed:
- `ferro-theme/src/token.rs`: 7 new constants + ALL_TOKENS(30) + v2 doc header — CONFIRMED
- `ferro-json-ui/assets/input.css`: bridge entries + safelist + reduced-motion — CONFIRMED
- `ferro-json-ui/assets/ferro-base.css`: regenerated with new utilities — CONFIRMED
- `ferro-json-ui/src/assets/mod.rs`: regression test added — CONFIRMED

### Commits exist:
- f26fa6d0 (Task 1) — CONFIRMED via git log
- c755e714 (Task 2) — CONFIRMED via git log
- 4ea70c60 (Task 3) — CONFIRMED via git log

## Self-Check: PASSED
