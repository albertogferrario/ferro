---
phase: 250-token-vocabulary-v2-default-theme-refresh
plan: "02"
subsystem: ferro-theme, ferro-cli
tags: [tokens, css, design-system, neutral-ramp, default-theme, make-theme]
requirements: [DS-01, DS-02]

dependency_graph:
  requires: [250-01 — token constants + CSS bridge]
  provides: [default-theme-v2, make-theme-scaffold-v2, 30-slot-drift-guards]
  affects: [ferro-theme, ferro-cli]

tech_stack:
  added: []
  patterns:
    - "Cool-tinted oklch neutral ramp: low-chroma (0.004–0.02) at hue 250 in both light and dark modes"
    - "Accent harmonized to primary hue family (250) — no separate cyan (200) hue in default theme"
    - "7 new tokens appended to all 3 CSS cascade blocks (:root, @media dark, [data-theme=dark])"
    - "Scaffold keeps blank-canvas zero-chroma values; DS-02 refresh is default.css-only"

key_files:
  created: []
  modified:
    - ferro-theme/assets/default.css
    - ferro-theme/src/loader.rs
    - ferro-cli/src/commands/make_theme.rs

decisions:
  - "Scaffold (make_theme.rs) keeps original zero-chroma neutrals as blank-canvas starting point; DS-02 cool-tint refresh applies to default.css only (not the scaffold template)"
  - "Dark ring token uses oklch(65% 0.18 250) in both dark blocks for sufficient contrast against dark surfaces; light uses oklch(55% 0.2 250) matching primary"
  - "secondary-foreground retains zero-chroma values (oklch(15% 0 0) / oklch(95% 0 0)) as a role token explicitly preserved by the plan — not part of the neutral ramp"

metrics:
  duration: "344 seconds (~6 minutes)"
  completed: "2026-07-03"
  tasks_completed: 2
  files_modified: 3
---

# Phase 250 Plan 02: Default Theme Refresh + 30-Slot Scaffold Summary

Refreshed `default.css` with cool-tinted oklch neutrals (hue 250), harmonized accent from
cyan (hue 200) to primary family (hue 250), and declared all 7 new v2 tokens across all
three CSS cascade blocks; extended the `make:theme` scaffold to 30 slots and updated both
drift-guard tests to assert 30.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Refresh default.css design language + declare 7 new tokens; update loader drift guard | 194a640c | ferro-theme/assets/default.css, ferro-theme/src/loader.rs |
| 2 | Extend the make:theme scaffold to 30 slots + update the drift-guard test | 41658c29 | ferro-cli/src/commands/make_theme.rs |

## Verification Results

- `cargo test -p ferro-theme`: 17/17 passed — `loader::tests::default_theme_returns_all_30_token_slots` green
- `cargo test -p ferro-cli make_theme`: 7/7 passed — `test_make_theme_tokens_css_has_all_30_token_slots` green; existing no-tailwind and dark-mode guards still pass
- `cargo fmt --all -- --check`: passes
- Acceptance criteria greps:
  - `--color-background: oklch(99% 0.004 250)` present in default.css (cool-tinted light)
  - accent harmonized: 3 occurrences across all blocks, hue 200 count = 0
  - `--motion-duration-fast: 120ms` × 3 blocks, `--spacing: 0.25rem` × 3 blocks
  - `--color-ring` × 3 blocks; dark ring `oklch(65% 0.18 250)` in both dark blocks
  - `--font-display: var(--font-sans)` × 3 blocks
  - No Tailwind at-rules in default.css (count = 0)
  - `oklch(100% 0 0)` count = 3 (primary-foreground white, within the ≤3 limit)

## Decisions Made

1. **Scaffold keeps zero-chroma neutrals**: The `tokens_css_template()` in `make_theme.rs` is a blank-canvas starting point — theme authors customize it. The DS-02 cool-tint refresh is applied only to `default.css` (the runtime default). The `test_make_theme_tokens_css_has_dark_mode_block` guard that asserts `oklch(12%` is preserved unchanged.

2. **Dark ring contrast**: `--color-ring: oklch(65% 0.18 250)` in both dark blocks (vs. `oklch(55% 0.2 250)` in light) — higher lightness in dark mode provides sufficient contrast for focus rings against dark surfaces.

3. **secondary-foreground retained as-is**: `oklch(15% 0 0)` (light) and `oklch(95% 0 0)` (dark) are role tokens for foreground text on secondary buttons, explicitly preserved by the plan. The acceptance criteria grep for zero-chroma neutrals produces 3 hits (all `--color-secondary-foreground`) rather than 0 — these are out of scope for the neutral ramp refresh. The must-have truth ("neutral ramp background/surface/card/border/text/text-muted carries nonzero chroma") is fully satisfied.

## Deviations from Plan

### Minor: secondary-foreground counted in zero-chroma grep

The plan's acceptance criterion states `grep -c 'oklch(97% 0 0)\|...\|oklch(15% 0 0)\|oklch(50% 0 0)' returns 0`. The actual count is 3 (all `--color-secondary-foreground` lines: `oklch(15% 0 0)` in `:root`, `oklch(95% 0 0)` in both dark blocks).

These are role tokens that the plan explicitly says to leave unchanged. The must-have truth specifies only the neutral ramp (background/surface/card/border/text/text-muted), which is fully cool-tinted with nonzero chroma. The grep criterion is a slight overcount that catches a role token not in scope. No code change required.

## Known Stubs

None. All 30 tokens are declared with concrete values in `default.css`. The scaffold produces editable CSS — not a stub.

## Threat Flags

None. Changes are compile-time-embedded CSS custom property values and Rust string constants. No new network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check

### Modified files exist:
- `ferro-theme/assets/default.css` — CONFIRMED (Write tool)
- `ferro-theme/src/loader.rs` — CONFIRMED (Edit tool, fmt applied)
- `ferro-cli/src/commands/make_theme.rs` — CONFIRMED (Edit tool)

### Commits exist:
- 194a640c (Task 1) — CONFIRMED via git rev-parse
- 41658c29 (Task 2) — CONFIRMED via git rev-parse

## Self-Check: PASSED
