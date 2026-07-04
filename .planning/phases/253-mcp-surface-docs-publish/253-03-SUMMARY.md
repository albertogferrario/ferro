---
phase: 253-mcp-surface-docs-publish
plan: "03"
subsystem: docs
tags: [docs, design-system, drift-guard, ds-08]
dependency_graph:
  requires: [253-01, 253-02]
  provides: [docs/src/design-system, D-09-drift-guard]
  affects: [docs/src/SUMMARY.md, ferro-json-ui/src/design/mod.rs]
tech_stack:
  added: []
  patterns: [mdBook chapter, verbatim-rationale copy, bi-directional drift guard]
key_files:
  created:
    - docs/src/design-system/principles.md
    - docs/src/design-system/tokens.md
    - docs/src/design-system/variants.md
    - docs/src/design-system/patterns.md
    - docs/src/design-system/linting.md
  modified:
    - docs/src/SUMMARY.md
    - ferro-json-ui/src/design/mod.rs
decisions:
  - "Rationale text copied verbatim from rules.rs into patterns.md — D-09 drift guard enforces this bi-directionally"
  - "Drift test uses plain #[cfg(test)] (not projections feature gate) — patterns.md is a plain file with no projections dependency"
  - "tokens.md owns the 30-slot vocabulary reference; features/themes.md owns the authoring recipe — cross-link, no duplication"
  - "variants.md cross-links components.md migration table rather than duplicating it"
metrics:
  duration: "~5 minutes"
  completed: "2026-07-04"
  tasks: 2
  files: 7
---

# Phase 253 Plan 03: Design System Chapter Summary

Design system documentation chapter (DS-08 part 1 of 3): five `docs/src/design-system/` pages covering principles, token reference, variant vocabulary, pattern catalog, and linting guide; registered in `SUMMARY.md`; drift-guarded by a bi-directional test.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Write five design-system docs pages + register in SUMMARY.md | ab5565d1 | docs/src/design-system/*.md, docs/src/SUMMARY.md |
| 2 | Add D-09 patterns.md <-> registry drift test | b78e3e17 | ferro-json-ui/src/design/mod.rs |

## What Was Built

**Five documentation pages** in `docs/src/design-system/`:

- `principles.md` — Three pillars: semantic tokens, intent-keyed patterns, lint as diagnostics. Each section links to the relevant detail page.
- `tokens.md` — Reference table for all 30 token slots grouped by category (surface/role/shape/shadow/typography/density/motion/focus/display). Opens with a cross-link to `features/themes.md` for the authoring recipe.
- `variants.md` — Canonical `variant`/`tone`/`size` enum values with one-line meanings. Opens with a cross-link to `json-ui/components.md` for the migration table.
- `patterns.md` — Per-rule catalog for all 10 design rules. Each section contains: rule id (as plain text for the drift guard), title, rationale copied verbatim from `rules.rs`, applicable intents, conforming JSON example, violating JSON example, and `allow` snippet.
- `linting.md` — CLI usage (`ferro design:lint`, `[path]`, `--json`, `--deny`), `design_lint` MCP tool reference, full output shape table, and allow-rule instructions.

**SUMMARY.md** — Design System chapter block inserted after the JSON-UI block (line 73), before `# Agents`.

**D-09 drift test** in `ferro-json-ui/src/design/mod.rs` — `docs_drift_tests::patterns_md_matches_rule_registry` asserts:
- Forward: every rule id in `design::rules()` appears in `patterns.md`
- Reverse: every `## \`id\`` header in `patterns.md` maps to a known rule id

Test runs under plain `#[cfg(test)]`, verified: 1/1 passed. All prior 47 design tests still pass.

## Verification

- `ls docs/src/design-system/` — five pages: linting.md, patterns.md, principles.md, tokens.md, variants.md
- All 10 rule ids present in patterns.md (verified via grep loop)
- 30 token rows in tokens.md (`grep -c "^| \`--"` = 30)
- `grep -n "design-system/principles.md" docs/src/SUMMARY.md` → line 77
- `grep -n "features/themes.md" docs/src/design-system/tokens.md` → cross-link present
- `grep -ni "legacy\|v2 vs" docs/src/design-system/*.md` → no framing violations
- `cargo test -p ferro-json-ui patterns_md_matches_rule_registry` → 1 passed

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all pages contain substantive content sourced from the rule registry and existing token/enum definitions.

## Threat Flags

None — documentation pages and a test that reads a repo-local file; no runtime component, no untrusted input.

## Self-Check: PASSED

- docs/src/design-system/principles.md: FOUND
- docs/src/design-system/tokens.md: FOUND
- docs/src/design-system/variants.md: FOUND
- docs/src/design-system/patterns.md: FOUND
- docs/src/design-system/linting.md: FOUND
- Commit ab5565d1: FOUND (Task 1)
- Commit b78e3e17: FOUND (Task 2)
- D-09 test: 1 passed, 0 failed
