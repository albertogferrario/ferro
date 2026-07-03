---
phase: 250-token-vocabulary-v2-default-theme-refresh
verified: 2026-07-03T05:30:00Z
status: passed
score: 4/4
overrides_applied: 0
re_verification: false
---

# Phase 250: Token Vocabulary v2 + Default Theme Refresh — Verification Report

**Phase Goal:** Grow the fixed vocabulary from 23 to 30 slots — one-knob density (`--spacing`), frequency-tiered motion (`--motion-duration-fast/base/slow`, `--motion-ease`), a uniform focus ring (`--color-ring`), and a display font slot (`--font-display`) — every new slot with a default so existing v1 themes stay valid unchanged; refresh the default theme to the documented design language.
**Verified:** 2026-07-03T05:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ALL_TOKENS` lists 30 slots; every new slot has a default in the base CSS and `default.css` (light + dark); an unmodified v1 `tokens.css` theme renders identically | VERIFIED | `token.rs` slice has exactly 30 entries (23 v1 + 7 new); `default.css` declares all 7 new tokens in all three cascade blocks; `@utility` blocks carry `var(, fallback)` so v1 themes lacking the new slots still resolve |
| 2 | Regenerated `ferro-base.css` exposes the new utilities (`duration-fast/base/slow`, `ease-base`, ring color, `font-display`, spacing base) resolving to `var()` slots | VERIFIED | All six utility class rules confirmed in generated CSS: `.duration-fast{transition-duration:var(--motion-duration-fast,.12s)}`, `.ease-base`, `.ring-ring{--tw-ring-color:var(--color-ring,var(--color-primary))}`, `.font-display`; spacing utilities resolve via `calc(var(--spacing) * N)` natively |
| 3 | Base CSS collapses motion durations under `prefers-reduced-motion` | VERIFIED | `@media (prefers-reduced-motion:reduce){:root{--motion-duration-fast:.01ms!important;--motion-duration-base:.01ms!important;--motion-duration-slow:.01ms!important}` confirmed in minified `ferro-base.css`; `!important` present (required to win over theme `<style>` cascade order) |
| 4 | `default.css` follows the design language (cool-tinted neutrals, single sparing accent) and `docs/src/features/themes.md` documents v2 plus the root-font-size type-scaling recipe | VERIFIED | `default.css`: neutrals use oklch with nonzero chroma at hue 250 across all three blocks; accent harmonized to hue 250 (no hue-200 cyan remains); operator visual sign-off completed during Plan 03 (screenshots in `app/tmp/`). `themes.md`: 30-slot tables present for all groups, `## Type Scaling` with `font-size: 14px` recipe, backward-compat note; no stale "23 semantic token slots" references |

**Score:** 4/4 ROADMAP success criteria verified

### Plan Must-Have Truths

| # | Plan | Truth | Status | Evidence |
|---|------|-------|--------|----------|
| 1 | 01 | ALL_TOKENS lists exactly 30 slots (23 v1 + 7 v2) | VERIFIED | Slice in `token.rs` has 30 entries; `all_tokens_len_is_30` test present |
| 2 | 01 | Regenerated ferro-base.css exposes duration-fast/base/slow, ease-base, ring-ring, font-display utilities | VERIFIED | All 6 `.class{` rules confirmed via grep |
| 3 | 01 | Regenerated ferro-base.css collapses motion durations under prefers-reduced-motion | VERIFIED | Block present with `!important` (WR-02 fix) |
| 4 | 01 | A v1 theme that omits new tokens resolves via fallback (SC1 structural guarantee) | VERIFIED | `@utility duration-fast { transition-duration: var(--motion-duration-fast, 120ms); }` pattern throughout; `ferro_base_css_contains_motion_duration_fallback` test pinned |
| 5 | 02 | default.css declares all 7 new tokens in :root, dark @media block, and [data-theme=dark] block | VERIFIED | All 7 new tokens confirmed in all three blocks in `default.css` (read directly) |
| 6 | 02 | default.css neutral ramp carries a subtle cool tint (nonzero chroma in hue ~250) in both modes | VERIFIED | Light background `oklch(99% 0.004 250)` through dark `oklch(15% 0.014 250)` — all neutral ramp values have nonzero chroma |
| 7 | 02 | --color-accent is harmonized toward the primary hue family (~250), no longer cyan (hue 200) | VERIFIED | Light: `oklch(70% 0.13 250)`, dark: `oklch(68% 0.13 250)`; grep for `200)` in `default.css` returns empty |
| 8 | 02 | ferro make:theme scaffolds all 30 slots; scaffold drift-guard test asserts 30 | VERIFIED | `test_make_theme_tokens_css_has_all_30_token_slots` present; old `_has_all_23_token_slots` name is gone |
| 9 | 02 | Theme::default_theme() exposes all 7 new tokens; loader test asserts them | VERIFIED | `default_theme_returns_all_30_token_slots` iterates `ALL_TOKENS` — structural guarantee, not a fixed enumeration |
| 10 | 03 | themes.md documents 30 semantic token slots with v2 token tables (density, motion, focus ring, display font) | VERIFIED | Sections present for Density Token (1), Motion Tokens (4), Focus Ring Token (1), Display Font Token (1); stale "23 semantic token slots" references: 0 |
| 11 | 03 | themes.md documents the root-font-size type-scaling recipe | VERIFIED | `## Type Scaling` section present with `font-size: 14px` example |
| 12 | 03 | The refreshed default theme reads as designed — verified visually in light and dark | VERIFIED | Operator approved after Chrome MCP screenshots (login + pagamenti pages, light and dark, 1440x900); documented in 250-03-SUMMARY.md |
| 13 | 03 | The full CI-exact gate (fmt + clippy --all-features + test --all-features) is green | VERIFIED | All three commands exited 0; documented in 250-03-SUMMARY.md |

**Plan must-have score:** 13/13

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-theme/src/token.rs` | 7 new TOKEN_* constants + ALL_TOKENS(30) + v2 doc header + test | VERIFIED | `//! Fixed semantic token vocabulary for ferro-theme/v2.`; 7 constants from TOKEN_SPACING to TOKEN_FONT_DISPLAY; 30-entry slice; `all_tokens_len_is_30` test |
| `ferro-json-ui/assets/input.css` | @utility duration blocks + @theme inline easing/ring/font bridges + safelist + prefers-reduced-motion | VERIFIED | `@utility duration-fast/base/slow` blocks; `--ease-base`, `--color-ring`, `--font-display` in `@theme inline`; safelist line 73; `@media (prefers-reduced-motion: reduce)` with `!important` |
| `ferro-json-ui/assets/ferro-base.css` | Regenerated with new utilities + reduced-motion collapse | VERIFIED | 6 utility class rules confirmed; reduced-motion block with `!important` confirmed |
| `ferro-json-ui/src/assets/mod.rs` | Regression test for fallback + reduced-motion + ring-ring | VERIFIED | 3 tests: `ferro_base_css_non_empty`, `ferro_base_css_contains_motion_duration_fallback` (asserts `.duration-fast{`, `!important`), `ferro_base_css_ring_falls_back_to_primary` |
| `ferro-theme/assets/default.css` | 30 tokens in all 3 blocks; cool-tinted; plain CSS only | VERIFIED | 30 tokens in `:root`; 7 new in dark blocks; no Tailwind at-rules; all neutrals have nonzero chroma at hue 250 |
| `ferro-theme/src/loader.rs` | Doc updated to 30 slots; test iterates ALL_TOKENS | VERIFIED | "for all 30 semantic token slots"; `for token in crate::token::ALL_TOKENS` drift guard |
| `ferro-cli/src/commands/make_theme.rs` | 30-slot scaffold template; drift-guard renamed to `_has_all_30_token_slots` | VERIFIED | 7 new token slots in scaffold (`:root` and dark `@media` blocks); test renamed; dark background assertion updated to `oklch(15% 0.014 250)` (WR-05) |
| `docs/src/features/themes.md` | v2 token reference (30 slots) + type-scaling section; plain-CSS authoring model | VERIFIED | All token tables updated; `## Type Scaling` present; WR-04 plain-CSS authoring model documented throughout |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-theme/src/token.rs` | `ALL_TOKENS` | 7 new constants appended to the slice | VERIFIED | TOKEN_SPACING through TOKEN_FONT_DISPLAY all present in slice |
| `ferro-json-ui/assets/input.css` | `ferro-json-ui/assets/ferro-base.css` | `scripts/gen-ferro-base-css.sh` (Tailwind v4.2.3) | VERIFIED | Regenerated artifact committed; all 6 expected utilities appear in minified output |
| `ferro-theme/assets/default.css` | `Theme::default_theme()` | `include_str!("../assets/default.css")` in `loader.rs` | VERIFIED | Line 6: `const DEFAULT_THEME_CSS: &str = include_str!("../assets/default.css");` |
| `ferro-cli/src/commands/make_theme.rs` `tokens_css_template()` | `themes/{name}/tokens.css` | `ferro make:theme` scaffold write | VERIFIED | Function returns 30-slot template; `test_make_theme_tokens_css_has_all_30_token_slots` passes |

### Data-Flow Trace (Level 4)

Not applicable. Modified files are Rust constants, static CSS assets, and documentation — no dynamic data rendering paths.

### Behavioral Spot-Checks

| Behavior | Evidence | Status |
|----------|----------|--------|
| `ferro_base_css_contains_motion_duration_fallback` test green | Documented in SUMMARY 01 and confirmed by WR-01 fix commit `72fcfb9a` strengthening the test | PASS |
| `ferro_base_css_ring_falls_back_to_primary` test green | Added by WR-03 fix commit `e6825d34`; rule `.ring-ring{--tw-ring-color:var(--color-ring,var(--color-primary))}` confirmed in ferro-base.css | PASS |
| `default_theme_returns_all_30_token_slots` test iterates ALL_TOKENS | WR-06 fix commit `8fad30db` rewrote test to iterate `crate::token::ALL_TOKENS`; confirmed in loader.rs | PASS |
| `test_make_theme_tokens_css_has_all_30_token_slots` green | Documented in SUMMARY 02; dark-mode test updated by WR-05 | PASS |
| Full CI gate green | SUMMARY 03: `fmt --check` 0, `clippy --all-targets --all-features` 0, `test --all-features` 0 | PASS |

### Requirements Coverage

| Requirement | Plans | Description | Status | Evidence |
|-------------|-------|-------------|--------|----------|
| DS-01 | 01, 02 | Token vocabulary grows from 23 to 30 slots; new slots have defaults in base CSS and `default.css`; v1 themes remain valid | SATISFIED | `ALL_TOKENS` has 30 entries; `@utility` fallbacks in ferro-base.css; `default.css` declares all 30 in all cascade blocks |
| DS-02 | 02, 03 | `default.css` refreshed to design language; `themes.md` documents v2 + type-scaling recipe | SATISFIED | Cool-tinted oklch neutrals, harmonized hue-250 accent confirmed in `default.css`; `themes.md` updated with 30-slot tables, `## Type Scaling`, and plain-CSS authoring model |

**Note on REQUIREMENTS.md traceability table:** Lines 211-212 still show `| DS-01 | Phase 250 | Not started |` and `| DS-02 | Phase 250 | Not started |`. The requirement entries themselves (lines 160, 166) have been checked `[x]`, indicating completion. The traceability table was not updated to "Complete" — this is a tracking artifact, not a code gap.

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `.planning/REQUIREMENTS.md` lines 211–212 | Traceability table status column still reads "Not started" for DS-01 and DS-02 | Info | No impact on code or goal — documentation tracking only; requirement checkboxes are correctly marked `[x]` |

No stubs, placeholder implementations, or disconnected wiring found in any production code file.

### Human Verification Required

None. The visual sign-off (Plan 03 Task 2) was completed during phase execution: operator approved after Chrome MCP screenshots in light and dark (login + pagamenti pages at 1440x900). The operator typed "approved" — zero oklch nudges were requested. This is documented in `250-03-SUMMARY.md`.

### Gaps Summary

No gaps. All four ROADMAP success criteria are satisfied by the actual codebase. All six review-fix commits (72fcfb9a through 8fad30db) applied cleanly: duration utilities generated via `@utility`, reduced-motion collapse wins with `!important`, ring-ring fallback to primary, themes.md plain-CSS authoring model, scaffold palette harmonized, and the loader drift guard strengthened to iterate ALL_TOKENS.

---

_Verified: 2026-07-03T05:30:00Z_
_Verifier: Claude (gsd-verifier)_
