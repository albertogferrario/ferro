---
phase: 143
plan: "02"
subsystem: ferro-theme
tags: [ferro-theme, css, theme-tokens, tailwind-migration]
dependency_graph:
  requires: []
  provides: [plain-css-default-theme]
  affects: [framework-json-ui-theme-injection]
tech_stack:
  added: []
  patterns: [plain-css-variables, root-custom-properties, dark-mode-media-query]
key_files:
  created: []
  modified:
    - ferro-theme/assets/default.css
    - ferro-theme/src/loader.rs
decisions:
  - "Converted default.css from @theme to :root; identical token values, syntax-only change"
  - "Removed @import 'tailwindcss' — file is now standard CSS injectable without Tailwind runtime"
  - "Updated loader.rs doc comments to reflect new plain CSS format (Rule 1 auto-fix)"
metrics:
  duration: "~5 min"
  completed: "2026-04-20"
  tasks_completed: 1
  files_modified: 2
requirements_satisfied: [D-09, D-10, D-11]
---

# Phase 143 Plan 02: Convert default.css to Plain CSS Variables Summary

**One-liner:** Converted `ferro-theme/assets/default.css` from Tailwind `@theme` syntax to standard `:root { ... }` CSS variable declarations — injectable into `<style>` tags without a Tailwind runtime.

## What Was Done

Task 1 converted `ferro-theme/assets/default.css` from:
- `@import "tailwindcss"` + `@theme { ... }` wrapper (Tailwind-CDN-specific, not standard CSS)

To:
- `:root { ... }` block with all 23 semantic tokens (light mode)
- `@media (prefers-color-scheme: dark) { :root { ... } }` for system dark mode
- `[data-theme="dark"] { ... }` for explicit class-based dark toggle

All 23 token values are preserved verbatim. This is a syntax-only conversion with no value changes.

## Token Coverage Confirmation

All 23 tokens present in the new file:
- Surface (6): `--color-background`, `--color-surface`, `--color-card`, `--color-border`, `--color-text`, `--color-text-muted`
- Role (8): `--color-primary`, `--color-primary-foreground`, `--color-secondary`, `--color-secondary-foreground`, `--color-accent`, `--color-destructive`, `--color-success`, `--color-warning`
- Shape (4): `--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-full`
- Shadow (3): `--shadow-sm`, `--shadow-md`, `--shadow-lg`
- Typography (2): `--font-sans`, `--font-mono`

## File Size

| | Bytes |
|---|---|
| Before (with @import + @theme wrappers) | 2287 |
| After (plain :root blocks) | 2480 |

Size increased by 193 bytes due to the header comment and slightly more verbose `:root {` and `@media (prefers-color-scheme: dark) { :root {` wrappers replacing `@theme {`.

## Token Value Discrepancies

None. All token values are identical between the old `@theme` blocks and the new `:root` blocks. This was a pure syntax conversion.

## Test Results

All 16 ferro-theme tests pass:
- `default_theme_returns_non_empty_css_with_color_primary` — continues to pass (`--color-primary` still present inside `:root`)
- `default_theme_returns_all_none_templates` — unaffected
- All `from_path_*` tests — unaffected (they test file loading, not format)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated stale doc comments in loader.rs**
- **Found during:** Post-conversion review
- **Issue:** `Theme.css` field doc and `default_theme()` doc still described the CSS as "Tailwind v4 `@theme` syntax" — now incorrect after conversion
- **Fix:** Updated field doc to "plain CSS variable declarations (`:root { ... }`)" and method doc to note the file is safe to inject without Tailwind processing
- **Files modified:** `ferro-theme/src/loader.rs`
- **Commit:** 016a6c84

## Self-Check

Files confirmed present:
- `ferro-theme/assets/default.css` — FOUND
- `ferro-theme/src/loader.rs` — FOUND (modified)

Commits confirmed:
- `42f45d7e` — feat(143-02): convert default.css from @theme to plain CSS :root variables
- `016a6c84` — docs(143-02): update loader.rs doc comments to reflect plain CSS format

## Self-Check: PASSED
