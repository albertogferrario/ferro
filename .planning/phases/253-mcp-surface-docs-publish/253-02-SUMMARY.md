---
phase: 253-mcp-surface-docs-publish
plan: "02"
subsystem: ferro-mcp
tags: [design-system, mcp, json-ui, drift-guard, agent-context]
dependency_graph:
  requires: [253-01, 251-component-variant-discipline-interactive-state-pass, 252-design-module-lint-cli, 250-token-vocabulary-v2-default-theme-refresh]
  provides: [DS-07-part2, design-system-mcp-surface]
  affects: [ferro-mcp/src/tools/json_ui_catalog.rs, ferro-mcp/src/tools/generation_context.rs]
tech_stack:
  added: [ferro-theme dep in ferro-mcp, strum dep in ferro-mcp]
  patterns: [strum VariantArray drift-proof enum enumeration, bidirectional RULE_COMPONENTS drift guard, static token table with count drift guard]
key_files:
  created: []
  modified:
    - ferro-mcp/Cargo.toml
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - ferro-mcp/src/tools/generation_context.rs
    - ferro-mcp/src/service.rs
    - ferro-mcp/src/tools/design_lint.rs
decisions:
  - "json_ui_catalog guidance is component-keyed (RULE_COMPONENTS → DesignVocabulary.component_guidance); intent-keyed grouping lives only in generation_context per D-06 division of labor"
  - "RULE_COMPONENTS is explicit static (not text-scanning rule prose) — bidirectionally drift-guarded: mapped ids ⊆ registry AND registry ⊆ mapped ids AND component names ⊆ builtin catalog set"
  - "Variant/Tone/Size values derived via strum::VariantArray — zero hand-listed arrays, drift structurally impossible (D-05)"
  - "CardAppearance intentionally excluded from vocabulary: lacks strum derives, is structural Card chrome not a weight/status/size axis"
  - "Token descriptions in generation_context use a static parallel table guarded by count test against ferro_theme::token::ALL_TOKENS"
metrics:
  duration: "632 seconds (~11 minutes)"
  completed: "2026-07-04"
  tasks_completed: 2
  files_modified: 5
---

# Phase 253 Plan 02: MCP Surface Design System Extensions Summary

Design system surfaces wired into the two agent-context MCP tools as additive, backward-compatible fields: `json_ui_catalog` now carries per-component design guidance derived from a drift-guarded static mapping; `generation_context` now carries a compact design-system summary with the 30-slot token vocabulary, per-intent patterns, and canonical variants.

## What Was Built

### Task 1: json_ui_catalog design_system field (commit `0b0a5aa7`)

**`DesignVocabulary` struct** added to `json_ui_catalog.rs`:
- `variant_values`, `tone_values`, `size_values`: derived via `strum::VariantArray` from the canonical `Variant`, `Tone`, `Size` enums — no hand-listed arrays, drift impossible by construction.
- `component_guidance`: `HashMap<String, Vec<DesignRuleRef>>` keyed by component type name, inverted from the explicit `RULE_COMPONENTS` static (rule id → builtin component names).

**`RULE_COMPONENTS` static** maps all 10 design rule IDs to the builtin component names they govern. Bidirectional drift guard (in `design_system_component_guidance_drift_guarded` test):
1. Every mapped rule id exists in `design::rules()` (no stale ids).
2. Every registry rule id is mapped (no missing ids when a rule is added).
3. Every component name is a real builtin (via the catalog output at test time).

**Dependencies added** to `ferro-mcp/Cargo.toml`: `ferro-theme = { path = "../ferro-theme", version = "0.2" }`, `strum = { version = "0.26" }`.

**Tests added**: `design_system_vocabulary_present` (5 variant, 4 tone, 3 size values; spot-checks "primary", "destructive", "md"), `design_system_component_guidance_drift_guarded` (bidirectional id + component name validation).

### Task 2: generation_context design_system field (commit `8bb0b608`)

**`DesignSystemSummary` struct** added to `generation_context.rs`:
- `tokens: &'static [TokenInfo]`: 30-entry static table (`DESIGN_TOKEN_DESCRIPTIONS`), one-liner purpose per CSS variable, same order as `ferro_theme::token::ALL_TOKENS`.
- `intent_patterns: HashMap<String, Vec<IntentPattern>>`: rules from `design::rules()` grouped by intent; rules with empty `intents` go into an `"all"` bucket (D-06 intent-keyed view, kept out of catalog per division of labor).
- `canonical_variants: CanonicalVariants`: variant/tone/size lists via strum, same source as Task 1.
- `docs: &'static str`: pointer to `docs/src/design-system/`.

**Count drift guard test** (`token_description_count_matches_all_tokens`): asserts `DESIGN_TOKEN_DESCRIPTIONS.len() == ferro_theme::token::ALL_TOKENS.len()` — fails immediately if a token is added to the vocabulary without updating the descriptions.

**`test_generation_context_has_all_sections` extended** with design_system assertions (30 tokens, non-empty intent_patterns, non-empty canonical_variants, non-empty docs).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Format] Pre-existing rustfmt drift in Plan 01 ferro-mcp files**
- **Found during:** Task 1 pre-commit `cargo fmt --all -- --check`
- **Issue:** `ferro-mcp/src/service.rs` and `ferro-mcp/src/tools/design_lint.rs` had non-canonical rustfmt formatting committed in Plan 01 (lines too long, multi-argument function calls not split).
- **Fix:** `cargo fmt -p ferro-mcp` applied before the Task 1 commit; both files staged and committed alongside the task 1 changes.
- **Files modified:** `ferro-mcp/src/service.rs`, `ferro-mcp/src/tools/design_lint.rs`
- **Commit:** `0b0a5aa7`

## Self-Check

### Created files exist:
- SUMMARY.md: this file

### Modified files exist:
- `ferro-mcp/Cargo.toml`: `ferro-theme` and `strum` deps present ✓
- `ferro-mcp/src/tools/json_ui_catalog.rs`: `DesignVocabulary`, `RULE_COMPONENTS`, `design_system` field ✓
- `ferro-mcp/src/tools/generation_context.rs`: `DesignSystemSummary`, `DESIGN_TOKEN_DESCRIPTIONS`, `design_system` field ✓

### Commits exist:
- `0b0a5aa7` — Task 1
- `8bb0b608` — Task 2

### Tests:
- `cargo test -p ferro-mcp json_ui_catalog`: 19 passed (incl. `design_system_vocabulary_present`, `design_system_component_guidance_drift_guarded`)
- `cargo test -p ferro-mcp generation_context`: 5 passed (incl. `token_description_count_matches_all_tokens`, extended `test_generation_context_has_all_sections`)

## Self-Check: PASSED
