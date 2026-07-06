---
phase: 250-token-vocabulary-v2-default-theme-refresh
fixed_at: 2026-07-03T04:23:19Z
review_path: .planning/phases/250-token-vocabulary-v2-default-theme-refresh/250-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 250: Code Review Fix Report

**Fixed at:** 2026-07-03T04:23:19Z
**Source review:** .planning/phases/250-token-vocabulary-v2-default-theme-refresh/250-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6 (fix_scope: critical_warning — 0 Critical, 6 Warning; 3 Info out of scope)
- Fixed: 6
- Skipped: 0

## Fixed Issues

### WR-01: `duration-fast/base/slow` utility classes are not generated

**Files modified:** `ferro-json-ui/assets/input.css`, `ferro-json-ui/assets/ferro-base.css`, `ferro-json-ui/src/assets/mod.rs`
**Commit:** 72fcfb9a
**Applied fix:** Defined `duration-fast`, `duration-base`, `duration-slow` explicitly via `@utility` blocks with `var(--motion-duration-*, fallback)`; removed the three dead `--duration-*` `@theme inline` bridge entries (Tailwind does not resolve `duration-*` against a `--duration-*` namespace, so they only fooled the drift guard). Regenerated `ferro-base.css` — all three class rules now present (`.duration-fast{transition-duration:var(--motion-duration-fast,.12s)}` etc.). Strengthened the drift guard to assert `.duration-fast{`, `.duration-base{`, `.duration-slow{` class rules exist, not just the `var()` substring.

### WR-02: Reduced-motion collapse defeated by theme `<style>` cascade order

**Files modified:** `ferro-json-ui/assets/input.css`, `ferro-json-ui/assets/ferro-base.css`, `ferro-json-ui/src/assets/mod.rs`
**Commit:** 2d4c9b91
**Applied fix:** Added `!important` to the three `--motion-duration-*` collapse declarations in the `prefers-reduced-motion` block, with a comment explaining the cascade-order rationale. Regenerated and verified `!important` survives minification (`--motion-duration-fast:.01ms!important`). Added a guard assertion so a regeneration that loses `!important` fails tests.

### WR-03: `--color-ring` has no base-CSS default — v1-theme compatibility gap

**Files modified:** `ferro-json-ui/assets/input.css`, `ferro-json-ui/assets/ferro-base.css`, `ferro-json-ui/src/assets/mod.rs`
**Commit:** e6825d34
**Applied fix:** Bridged as `--color-ring: var(--color-ring, var(--color-primary))`. Regenerated and verified the fallback inlines at every use site (`.ring-ring{--tw-ring-color:var(--color-ring,var(--color-primary))}`). Added drift-guard test `ferro_base_css_ring_falls_back_to_primary`.

### WR-04: themes.md documents the retired `@theme`/Tailwind-CLI authoring model

**Files modified:** `docs/src/features/themes.md`
**Commit:** 3f6a9e61
**Applied fix:** Rewrote all four stale sections to the plain-CSS `:root` authoring model: overview bullet, Quick Start customize example (Tailwind CLI processing step replaced with a no-build-step explanation), dark-mode `@media` example (`@theme` → `:root`), and "For Theme Creators" ("Authoring format vs. deployed format" replaced with "No build step" describing verbatim `<style>` injection). Remaining `@theme` mentions are only the explicit do-not-use warnings.

### WR-05: themes.md default values stale; scaffold palette not harmonized

**Files modified:** `docs/src/features/themes.md`, `ferro-cli/src/commands/make_theme.rs`
**Commit:** 0f085876
**Applied fix:** Updated the "Default (light)" surface table and the accent row to the refreshed hue-250 values from `ferro-theme/assets/default.css`. Took the review's recommended option for the scaffold: `make:theme` now mirrors the shipped default palette (light surfaces/accent + full dark block). Updated the scaffold's dark-mode test assertion (`oklch(12%` → `oklch(15% 0.014 250)`) and the docs' dark-mode examples to the same dark values so all three surfaces agree.

### WR-06: loader.rs drift guard checks 7 of 30 tokens

**Files modified:** `ferro-theme/src/loader.rs`
**Commit:** 8fad30db
**Applied fix:** Rewrote `default_theme_returns_all_30_token_slots` to iterate `crate::token::ALL_TOKENS` and assert a `{token}:` declaration exists for each slot (suffix colon avoids prefix false-positives). token.rs ↔ default.css name drift is now structurally impossible.

## Verification

- Regenerated `ferro-base.css` via `scripts/gen-ferro-base-css.sh` after each `input.css` change; artifact committed together with its source in each commit.
- Crate-scoped tests green after each fix: `ferro-json-ui` assets tests (13 passed), `ferro-cli` make_theme tests (7 passed), `ferro-theme` tests (17 passed).
- `cargo fmt --all -- --check` clean.
- `cargo clippy -p ferro-theme -p ferro-json-ui -p ferro-cli --all-targets --all-features -- -D warnings` clean.

---

_Fixed: 2026-07-03T04:23:19Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
