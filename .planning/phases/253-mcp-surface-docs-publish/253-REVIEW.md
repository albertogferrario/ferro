---
phase: 253
status: issues
depth: standard
files_reviewed: 28
critical: 0
warnings: 2
info: 1
reviewed: 2026-07-04
---

# Phase 253: Code Review

**Findings: 0 Critical / 2 Warning / 1 Info** (published 0.2.85 pre-dates this review; fixes ride the next patch)

## WR-01 [Warning] `col-span-2/3/4` missing from Tailwind safelist

`ferro-json-ui/assets/input.css:72` — `render_grid` builds base col-span classes
via `format!("col-span-{}", span.clamp(2, 4))` (containers.rs), so the Tailwind
source scanner never sees them as literals and they are absent from the
generated `ferro-base.css`. `md:col-span-2/3` survive only because
`render_form_section` contains them as literals. Effect: spans on a
`columns > 1` grid have no visual effect at the base breakpoint.
**Fix:** add `col-span-2 col-span-3 col-span-4` to the `@source inline(...)`
safelist and regenerate `ferro-base.css`.

## WR-02 [Warning] Wrong MCP tool name in published docs

`docs/src/design-system/linting.md:50` — references `spec_lint`; the registered
tool is `design_lint`. User-facing factual error in the guide the gestiscilo
Phase 232 sweep follows. **Fix:** rename to `design_lint`.

## IN-01 [Info] No test covers base `col-span-N` generation

`grid_spans_wrap_children_with_col_span` uses `columns: 1`, so the base-span
branch never runs; the fill test generates `col-span-2` but never asserts on
it. WR-01 was therefore invisible to the suite. **Fix:** add a `columns > 1`
span case asserting the base class.

## Confirmed correct

design_lint XOR/path handling and CLI-identical FileFinding contract;
RULE_COMPONENTS bidirectional drift guard (11 rules); destructive-confirmation
`action.confirm` check incl. entry-level-confirm regression; token count guard
(30); D-09 patterns.md drift test (11 sections); ferro-fill CSS selector chain
for both dashboard-family layouts; fill+scrollable suppression; class-merge
wrapper; project-agnostic crate check; SUMMARY.md registration; dark-mode
primary contrast (~70 L* points); 252 deferred items IN-01/IN-02 verified done.
