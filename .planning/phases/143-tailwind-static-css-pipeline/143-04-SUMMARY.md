---
phase: 143
plan: 04
subsystem: ferro-cli
tags: [ferro-cli, scaffolder, theme, tests, plain-css]
dependency_graph:
  requires: [143-03]
  provides: [make-theme-plain-css-scaffolder]
  affects: [ferro-cli]
tech_stack:
  added: []
  patterns: [plain-css-variables, root-block-tokens]
key_files:
  modified:
    - ferro-cli/src/commands/make_theme.rs
decisions:
  - "tokens_css_template uses :root { ... } not @theme to align with Plan 03 injection path"
  - "Dark mode uses @media (prefers-color-scheme: dark) { :root { ... } } not @theme"
  - "test_make_theme_tokens_css_has_theme_block renamed to test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax"
  - "Dark mode test assertion changed from contains('@theme') to not contains('@theme {') to avoid false match on comment text"
metrics:
  duration: ~8min
  completed: 2026-04-20
  tasks_completed: 2
  files_modified: 1
---

# Phase 143 Plan 04: Scaffolder Plain CSS Alignment Summary

Update `ferro make:theme` scaffolder to emit plain CSS variable declarations (`:root { ... }`) instead of Tailwind-CDN-specific `@import "tailwindcss"` + `@theme { ... }` syntax. Aligns the scaffolder output with Plan 02's default theme conversion and Plan 03's runtime injection contract.

## Tasks

### Task 1: Rewrite tokens_css_template and update tests (COMPLETE)

Commit: `b4033a44`
Files: `ferro-cli/src/commands/make_theme.rs`

**Template change:** `tokens_css_template()` now returns plain CSS with a `:root { ... }` block and `@media (prefers-color-scheme: dark) { :root { ... } }`. The `@import "tailwindcss"` line and `@theme { ... }` wrappers are removed. All 23 semantic tokens are preserved with their existing values.

**Test changes:**
- `test_make_theme_tokens_css_has_theme_block` renamed to `test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax`
- New assertions: `!css.contains("@import \"tailwindcss\"")`, `!css.contains("@theme {")`, `css.contains(":root {")`
- `test_make_theme_tokens_css_has_dark_mode_block`: added `!css.contains("@theme {")` assertion

**Token byte delta:** Template shrunk from 664 bytes to 648 bytes (removed `@import` line and changed `@theme {` → `:root {` wrappers; comment block is slightly longer).

**All 7 make_theme tests pass:**
```
test_make_theme_creates_directory_structure ... ok
test_make_theme_tokens_css_has_all_23_token_slots ... ok
test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax ... ok
test_make_theme_tokens_css_has_dark_mode_block ... ok
test_make_theme_theme_json_is_empty_object ... ok
test_make_theme_fails_if_directory_exists ... ok
test_make_theme_succeeds_once_fails_on_repeat ... ok
```

### Task 2: Full workspace validation (PARTIAL)

`cargo fmt --all -- --check` — PASSED
`cargo clippy --all --all-targets -- -D warnings` — PASSED
`cargo test --all-features` — FAILED (environment, not code)

The workspace test run failed on `No space left on device` (disk 100% full: 460Gi used, 299Mi available) during compilation of unrelated large crates: `async-stripe`, `aws-sdk-s3`, `ferro-storage`. No code changes failed; the failure is a build machine disk exhaustion. The ferro-cli crate (Plan 04 scope) and all previously tested crates passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Format string braces in assert! messages**
- **Found during:** Task 1 compile
- **Issue:** Assert message strings contained literal `{ ... }` which Rust's format macro interprets as format arguments, causing compile error "expected `}`, found `.`"
- **Fix:** Escaped braces as `{{...}}` in the three affected assert messages
- **Files modified:** `ferro-cli/src/commands/make_theme.rs`
- **Commit:** `b4033a44` (inline with Task 1)

**2. [Rule 1 - Bug] Dark mode @theme assertion false-match on comment**
- **Found during:** Task 1 test run
- **Issue:** `test_make_theme_tokens_css_has_dark_mode_block` asserted `!css.contains("@theme")` but the template's own file comment contains the text `Tailwind's @theme syntax`, causing the test to fail
- **Fix:** Changed assertion to `!css.contains("@theme {")` — checks for the actual at-rule syntax, not the English word in the comment
- **Files modified:** `ferro-cli/src/commands/make_theme.rs`
- **Commit:** `b4033a44` (inline with Task 1)

## Acceptance Criteria Status

- `@import "tailwindcss"` absent from `tokens_css_template` function body: CONFIRMED
- `@theme {` absent from `tokens_css_template` function body: CONFIRMED
- `:root {` present in `tokens_css_template` function body: CONFIRMED
- `@media (prefers-color-scheme: dark)` present: CONFIRMED
- All 23 tokens present (verified by `test_make_theme_tokens_css_has_all_23_token_slots`): CONFIRMED
- `test_make_theme_tokens_css_has_root_block_and_no_tailwind_syntax` passes: CONFIRMED
- `test_make_theme_tokens_css_has_dark_mode_block` passes: CONFIRMED
- `test_make_theme_tokens_css_has_all_23_token_slots` passes: CONFIRMED
- Old test name `test_make_theme_tokens_css_has_theme_block` absent from file: CONFIRMED
- `cargo build -p ferro-cli` exits 0: CONFIRMED (via test run)
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` exits 0: CONFIRMED
- `cargo test --all-features` exits 0: BLOCKED by disk exhaustion (environment issue, not code regression)

## Known Stubs

None. The template content is a complete plain-CSS token set with all 23 slots populated with real default values.

## Threat Flags

None. Plan 04 only modifies a compile-time string constant in the CLI binary. No network endpoints, auth paths, or file access patterns were added.

## Self-Check: PASSED

- File exists: `ferro-cli/src/commands/make_theme.rs` — FOUND
- Commit exists: `b4033a44` — FOUND
- Old test name absent: `grep "test_make_theme_tokens_css_has_theme_block" ferro-cli/src/commands/make_theme.rs` — NOT FOUND (correct)
- `:root {` in template: CONFIRMED
- No `@theme {` in template: CONFIRMED
