---
phase: 250-token-vocabulary-v2-default-theme-refresh
plan: "03"
subsystem: docs, ferro-theme
tags: [tokens, design-system, documentation, visual-verification]
requirements: [DS-02]

dependency_graph:
  requires: [250-01 — token constants + regenerated ferro-base.css, 250-02 — refreshed default.css + 30-slot scaffold]
  provides: [themes-md-v2-reference, type-scaling-recipe, visual-sign-off-default-theme, phase-250-ci-gate-green]
  affects: [docs, ferro-theme]

tech_stack:
  added: []
  patterns:
    - "Token reference tables in themes.md mirror ferro-theme/src/token.rs constants exactly (names + defaults)"
    - "Root font-size documented as the type-scaling mechanism — no per-size type tokens by design"

key_files:
  created: []
  modified:
    - docs/src/features/themes.md

decisions:
  - "Visual sign-off approved with zero oklch nudges — default.css ships exactly as refreshed in Plan 02"
  - "Focus rings are not visually checkable this phase by design: no component emits ring classes until Phase 251; the --color-ring token and its default ship here"
  - "Dark Mode section's @theme-inside-@media doc drift left as-is (plan marked it optional, out of required scope)"

metrics:
  duration: "5737 seconds (~95 minutes, dominated by the full CI gate)"
  completed: "2026-07-03"
  tasks_completed: 2
  files_modified: 1
---

# Phase 250 Plan 03: v2 Docs + Visual Sign-off Summary

themes.md now documents the full 30-slot v2 vocabulary with density/motion/focus-ring/display-font tables and the root-font-size type-scaling recipe; the refreshed default theme passed operator visual sign-off in light and dark, and the full CI-exact gate is green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Document token vocabulary v2 + type-scaling recipe in themes.md | 3d114d16 | docs/src/features/themes.md |
| 2 | Visual sign-off of refreshed default theme + full CI gate | (no code change — approval gate) | ferro-theme/assets/default.css (approved as-is) |

## Verification Results

### CI-exact gate (run sequentially, one cargo command at a time)

| Command | Exit code |
|---|---|
| `cargo fmt --all -- --check` | 0 |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | 0 |
| `cargo test --all-features` | 0 (full suite green, including `test_make_theme_tokens_css_has_all_30_token_slots`, `default_theme_returns_all_30_token_slots`, `ferro_base_css_contains_motion_duration_fallback`) |

The `cargo test` run regenerated `docs/protocol/schemas/*.json` (Phase 94 export test); that unrelated churn was discarded via `git checkout` per project convention. Two initial `cargo test --all-features` attempts were SIGKILLed (exit 137, OOM during dependency compilation); the third attempt with warm incremental cache completed cleanly — environment issue, not a defect.

### Docs acceptance criteria

- `grep -c '30 semantic token slots'` returns 2; stale `23`-count references: 0
- `--motion-duration-fast`, `--spacing`, `--color-ring`, `--font-display` all present in token tables
- `## Type Scaling` section present with `font-size: 14px` recipe
- Typography token-name bug fixed: `--font-family-sans`/`--font-family-mono` → `--font-sans`/`--font-mono` (0 stale occurrences; names now match `ferro-theme/src/token.rs` constants)
- Backward-compat migration note added at Token Reference intro (every valid v1 theme remains a valid v2 theme unchanged)

### Visual sign-off (operator approved)

Server start approved by operator; fresh binary served on :8090 (a stale pre-refresh instance was killed first — `default.css` is `include_str!`-embedded, so the old process carried the old theme). Chrome MCP screenshots at 1440x900 (login + pagamenti pages, light and dark) presented to the operator; retained locally in `app/tmp/` (not committed).

Assessment against the design bar:
- Cool-tinted neutrals visible in both modes — no flat grey
- Single hue-250 focal accent; the previous cyan (hue 200) accent is gone
- Dark reads dark-not-gloomy: tinted, contrast-separated surfaces, lightness not crushed
- Radii, shadows, spacing, and table striping intact relative to the pre-refresh baseline
- Focus rings not visually checkable this phase by design: no component emits ring classes until Phase 251; the `--color-ring` token and default ship in this phase

Operator typed "approved" — zero oklch nudges requested; `ferro-theme/assets/default.css` unchanged from Plan 02 (commit 194a640c).

## Deviations from Plan

### Environmental: OOM SIGKILL on first two cargo test attempts

`cargo test --all-features` was killed (signal 9, exit 137) twice during dependency compilation (`itertools`, `utoipa`, `ferro-bundle`). Not a code defect — memory pressure during parallel compilation. Each attempt made incremental progress; the third run compiled and ran the full suite green with exit 0. No retry-limit concern: the same command, no code changes between attempts.

No other deviations — plan executed as written.

## Known Stubs

None. The documentation is complete and matches the shipped constants; `default.css` carries concrete values for all 30 slots.

## Threat Flags

None. Changes are documentation-only plus an approval gate over existing CSS values; no network, auth, storage, or schema surface.

## Self-Check

### Modified files exist:
- `docs/src/features/themes.md` — CONFIRMED (Edit tool; all greps pass)

### Commits exist:
- 3d114d16 (Task 1) — CONFIRMED via git log

## Self-Check: PASSED
