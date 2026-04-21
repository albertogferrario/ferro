---
phase: 143
plan: "01"
subsystem: ferro-json-ui
tags: [ferro-json-ui, tailwind, css, build-tooling]
dependency_graph:
  requires: []
  provides: [FERRO_BASE_CSS, ferro-json-ui/assets/ferro-base.css, scripts/gen-ferro-base-css.sh]
  affects: [ferro-json-ui, framework]
tech_stack:
  added: [Tailwind v4 standalone CLI (build-time only)]
  patterns: [include_str! asset embedding, @source inline() safelist]
key_files:
  created:
    - ferro-json-ui/assets/input.css
    - ferro-json-ui/assets/ferro-base.css
    - ferro-json-ui/src/assets.rs
    - scripts/gen-ferro-base-css.sh
  modified:
    - ferro-json-ui/src/lib.rs
    - .github/workflows/ci.yml
decisions:
  - "Asset location: ferro-json-ui/assets/ — mirrors ferro-theme/assets/ pattern"
  - "Dedicated assets.rs module for include_str! — consistent with ferro-theme/src/loader.rs"
  - "Bootstrap placeholder CSS committed; must be replaced by running gen-ferro-base-css.sh after CLI install"
  - "clippy::const_is_empty suppressed on test — const evaluated at compile time, lint FP in test context"
metrics:
  duration_minutes: 25
  completed_date: "2026-04-20"
  tasks_completed: 3
  tasks_total: 5
  files_created: 4
  files_modified: 2
---

# Phase 143 Plan 01: Tailwind Static CSS Asset Pipeline Summary

Static CSS asset pipeline for ferro-json-ui: `input.css` Tailwind entry file, `ferro-base.css` pre-built output (committed), `FERRO_BASE_CSS: &'static str` embedded via `include_str!`, regeneration shell script, and CI drift check.

## Tasks Completed

| Task | Name | Commit | Status |
|------|------|--------|--------|
| 1 | Install Tailwind v4 standalone CLI | — | Blocked (human-action gate) |
| 2 | Create input.css, gen script, bootstrap ferro-base.css | db47e2c0 | Done (placeholder CSS) |
| 3 | Add assets.rs, embed FERRO_BASE_CSS, re-export from lib.rs | 84769991 | Done |
| 4 | Add CI job for ferro-base.css drift check | 05b356d8 | Done |
| 5 | Human verification | — | Awaiting |

## What Was Built

### ferro-json-ui/assets/input.css
Tailwind v4 entry file with `@import "tailwindcss"`, `@source` directives scanning `ferro-json-ui/src` and `framework/src`, and an `@source inline(...)` safelist covering all 23 semantic token classes (bg-background, bg-primary, text-text-muted, rounded-*, shadow-*, font-sans, font-mono, etc.).

### ferro-json-ui/assets/ferro-base.css
Bootstrap placeholder CSS (4,751 bytes) covering the key utility classes ferro-json-ui components emit. Contains `flex`, `bg-primary`, `rounded-md`, `font-sans`, and the full semantic token vocabulary as CSS custom property references. **Must be replaced by running `bash scripts/gen-ferro-base-css.sh` after installing the Tailwind v4 CLI.**

### ferro-json-ui/src/assets.rs
Module-level `pub const FERRO_BASE_CSS: &str = include_str!("../assets/ferro-base.css")`. Unit test `ferro_base_css_non_empty` asserts the constant is non-empty and contains `flex`. `include_str!` guarantees UTF-8 validity at compile time (satisfies D-17).

### scripts/gen-ferro-base-css.sh
Executable shell script: checks for `tailwindcss` on PATH, runs `tailwindcss -i ferro-json-ui/assets/input.css -o ferro-json-ui/assets/ferro-base.css --minify`, reports byte count. Idempotent by design.

### .github/workflows/ci.yml — ferro-base-css-drift job
New CI job installs Tailwind v4 Linux x64 binary, regenerates CSS to `/tmp/ferro-base.generated.css`, diffs against committed file. Fails with `::error::ferro-base.css is out of date. Run 'bash scripts/gen-ferro-base-css.sh' locally and commit the result.`

## Output Metrics

- **ferro-base.css byte size:** 4,751 bytes (bootstrap placeholder; actual Tailwind output will be larger)
- **Tailwind CLI version used:** Not yet installed on dev machine — Task 1 gate pending
- **Classes added to @source inline() beyond initial list:** None — initial safelist covers the 23-token semantic vocabulary
- **render.rs runtime string-concat assemblies found (A2):** None detected via grep of `format!` calls producing partial class names in `ferro-json-ui/src/render.rs`. All class strings appear as complete literals.
- **cargo test --all-features:** PASSED (all test results ok, 0 failures)

## Deviations from Plan

### Auto-fixed Issues

None.

### Planned Deviations

**1. [Rule 3 - Blocking] Bootstrap placeholder CSS instead of CLI-generated output**
- **Found during:** Task 2
- **Issue:** Task 1 is a `checkpoint:human-action` gate requiring the Tailwind v4 CLI binary installed at `/usr/local/bin/tailwindcss`. The binary was not present and network download was not permitted in this execution context.
- **Fix:** Created a hand-crafted `ferro-base.css` placeholder (4,751 bytes) containing the key utility classes ferro-json-ui components use, expressed as CSS custom property references matching the ferro-theme token vocabulary. The placeholder allows the codebase to compile and tests to pass. It is a functionally valid CSS file but lacks Tailwind's full variant coverage (hover:, dark:, sm:, responsive variants).
- **Required follow-up:** After installing the Tailwind v4 CLI (`curl -sLo /usr/local/bin/tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64 && chmod +x /usr/local/bin/tailwindcss`), run `bash scripts/gen-ferro-base-css.sh` and commit the regenerated file. The CI drift job will then enforce that the committed file matches CLI output going forward.
- **Files modified:** ferro-json-ui/assets/ferro-base.css
- **Commit:** db47e2c0

**2. [Rule 1 - Bug] clippy::const_is_empty suppression in test**
- **Found during:** Task 3 post-commit lint
- **Issue:** Clippy's `const_is_empty` lint fires on `assert!(!FERRO_BASE_CSS.is_empty(), ...)` because clippy can prove at compile time that the const is non-empty, making the check trivially true. With `-D warnings`, this is a compile error in test mode.
- **Fix:** Added `#[allow(clippy::const_is_empty)]` on the test function. The test is intentionally documenting the invariant (non-empty embedded CSS) rather than being a pure runtime check.
- **Files modified:** ferro-json-ui/src/assets.rs
- **Commit:** 84769991

## Known Stubs

- **ferro-json-ui/assets/ferro-base.css** — Bootstrap placeholder. Contains manually curated utility classes and CSS custom property mappings, but lacks full Tailwind variant coverage (responsive prefixes, hover:, focus:, dark: variants). Replace by running `bash scripts/gen-ferro-base-css.sh` after installing the Tailwind v4 CLI. The CI drift check job will fail until this is done (it generates the real output and diffs against the placeholder).

## Plan 03 Readiness

`ferro_json_ui::FERRO_BASE_CSS` is exported as a `pub &'static str`. Plan 03 can consume it immediately:

```rust
let css = ferro_json_ui::FERRO_BASE_CSS;
// css.as_bytes() is &'static [u8] — safe for Bytes::from_static
```

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. The `FERRO_BASE_CSS` constant is a compile-time embedding — no file I/O at request time. CI drift job downloads from GitHub Releases over HTTPS (T-143-02, accepted per threat model).

## Self-Check

- [x] `ferro-json-ui/assets/input.css` exists and contains `@import "tailwindcss"`
- [x] `ferro-json-ui/assets/ferro-base.css` exists, non-empty, contains `flex`, `bg-primary`, `rounded-md`, `font-sans`
- [x] `ferro-json-ui/src/assets.rs` exists with `include_str!` and unit test
- [x] `ferro-json-ui/src/lib.rs` contains `pub mod assets;` and `pub use assets::FERRO_BASE_CSS;`
- [x] `scripts/gen-ferro-base-css.sh` exists and is executable
- [x] `.github/workflows/ci.yml` contains `ferro-base-css-drift`, `tailwindcss-linux-x64`, `diff -u ferro-json-ui/assets/ferro-base.css`, `scripts/gen-ferro-base-css.sh`
- [x] YAML parses cleanly
- [x] `cargo build -p ferro-json-ui` exits 0
- [x] `cargo test -p ferro-json-ui assets::tests::ferro_base_css_non_empty` exits 0, 1 passed
- [x] `cargo fmt --all -- --check` exits 0
- [x] `cargo clippy --all --all-targets -- -D warnings` exits 0
- [x] `cargo test --all-features` exits 0
- Commits: db47e2c0, 84769991, 05b356d8

## Self-Check: PASSED
