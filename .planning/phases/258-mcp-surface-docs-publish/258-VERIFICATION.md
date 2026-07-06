---
phase: 258-mcp-surface-docs-publish
verified: 2026-07-06T18:00:00Z
status: passed
score: 11/11
overrides_applied: 0
re_verification: null
gaps: []
deferred: []
human_verification: []
---

# Phase 258: MCP Surface + Docs + Publish — Verification Report

**Phase Goal:** Agents can discover and compose POS sale screens through `ferro-mcp` without consulting source code; the full CI-exact gate is green; gestiscilo's register phase can pin the published crate. (Milestone v16.6 closer — single crates.io publish.)
**Verified:** 2026-07-06T18:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | An agent reading generation_context learns WHEN to use the Register layout template vs. a form-only Collect spec and the form-state selection contract | VERIFIED | `RegisterCompositionGuidance.when_to_use` + `form_state_contract` fields populated; `test_generation_context_has_all_sections` passes |
| 2 | generation_context lists the four register-* lint rule ids derived from the rule registry (not hand-copied) | VERIFIED | `register_rule_ids` filter array in `execute()` at generation_context.rs:336-350; derived via `design_rules().filter()`; `register_composition_drift_guard` test passes |
| 3 | json_ui_catalog BUILDER_API documents fill_viewport(bool) and .each(path, as_) | VERIFIED | Both present at json_ui_catalog.rs:361,368; `builder_api_mentions_fill_viewport_and_each` test passes |
| 4 | SelectionPanel and Numpad receive register-fill-viewport guidance in json_ui_catalog | VERIFIED | `RULE_COMPONENTS` entry: `("register-fill-viewport", &["Grid", "TileGrid", "SelectionPanel", "Numpad"])` at json_ui_catalog.rs:100-103; `design_system_component_guidance_drift_guarded` passes |
| 5 | SC-1 (catalog count 52 + all five names) is recorded as pre-existing evidence, not re-implemented | VERIFIED | 258-01-SUMMARY.md §SC-1 records `test_all_components_present ... ok` with count=52 and all five names; no count churn |
| 6 | docs/src/json-ui/components.md documents all five new components (TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad) with props tables and at least one usage example each | VERIFIED | `grep -c "^### (TileGrid\|SelectionPanel\|FilterTabs\|QuantityStepper\|Numpad)$"` returns 5; each section has a props table and a fenced JSON example |
| 7 | The docs describe the tap-to-add interaction model, the Form common-ancestor scoping, and the disable_on_submit/idempotency double-submit pointer | VERIFIED | TileGrid note: "One tap on a tile adds one unit; ALL quantity editing happens in the SelectionPanel" (components.md:1472); Form ancestor stated same line; SelectionPanel note states `disable_on_submit: true` → `data-disable-on-submit` + idempotency pointer (components.md:1499) |
| 8 | docs/src/json-ui/layouts.md documents fill_viewport and the Register layout template | VERIFIED | `grep -c "## fill_viewport\|## Register Layout Template"` returns 2; `app`/`dashboard` layout constraint documented; `register_template()` cross-link present |
| 9 | docs/src/json-ui/spec-construction.md documents the fill_viewport() and each() builder additions | VERIFIED | `SpecBuilder::fill_viewport(bool) -> Self` and `ElementBuilder::each(path, as_)` documented with explanation and Rust example at spec-construction.md:136-165 |
| 10 | mdBook build exits 0; existing pages extended, no new SUMMARY.md pages | VERIFIED | `mdbook build docs` exits 0 (re-run in verification); SUMMARY.md unchanged (D-08 constraint observed) |
| 11 | The projection-derived /cassa flip stands; CI-exact gate green; workspace bumped 0.2.88→0.2.89; master pushed; BOTH ferro-rs 0.2.89 AND ferro-payments 0.1.6 live on crates.io; gestiscilo handoff brief written | VERIFIED | `cassa.json` absent; `cassa.rs` calls `register_template()` 3×; crates.io API: ferro-rs=0.2.89, ferro-payments=0.1.6; git tag v0.2.89 on remote; GitHub Release latest=v0.2.89; CI run 28808914072 all waves green; handoff brief embedded in 258-03-SUMMARY.md |

**Score:** 11/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp/src/tools/generation_context.rs` | register_composition guidance field + drift guard | VERIFIED | `RegisterCompositionGuidance` struct (6 fields), `RegisterRuleRef` struct, `register_composition` field on `GenerationContext`, `register_composition_drift_guard` test — all present and passing |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | BUILDER_API fill_viewport/each additions + RULE_COMPONENTS register-fill-viewport fix | VERIFIED | `fill_viewport` appears 5× (BUILDER_API + test), `.each(` appears; register-fill-viewport maps to Grid/TileGrid/SelectionPanel/Numpad; fill-viewport-layout-unknown maps to Grid |
| `docs/src/json-ui/components.md` | Five new ### component sections under ## Commerce Components | VERIFIED | 5 sections confirmed; each with props table following Tile format anchor; between Commerce and Kanban sections |
| `docs/src/json-ui/layouts.md` | fill_viewport + Register Layout Template sections | VERIFIED | Both `## fill_viewport` and `## Register Layout Template` present; register_template() Rust snippet lifted from cassa.rs |
| `docs/src/json-ui/spec-construction.md` | Builder API additions (fill_viewport, each) | VERIFIED | `### Builder API additions` subsection present; both methods documented with lint-rule pointers and cross-links |
| `Cargo.toml` | workspace version = "0.2.89" | VERIFIED | Line 47: `version = "0.2.89"` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| generation_context.rs register_composition.lint_rules | ferro_json_ui::design::rules() | filter-by-id derivation in execute() | WIRED | `design_rules().iter().filter(|r| register_rule_ids.contains(&r.id))` at line 342-350; test passes |
| register_composition_drift_guard test | global_catalog() / design::rules() / FERRO_RUNTIME_JS | registry-iterator lookups + substring assertions | WIRED | Test uses `global_catalog().components_sorted()`, `ferro_json_ui::design::rules()`, and `ferro_json_ui::FERRO_RUNTIME_JS.contains()`; passes |
| docs Register Layout Template section | register_template() helper + /cassa sample | cross-link + code example | WIRED | layouts.md references `register_template()` with Rust snippet from cassa.rs; cross-links to TileGrid/SelectionPanel sections |
| component sections form_id / fill_viewport notes | the runtime contract described in generation_context | compositional-constraint notes | WIRED | `form_id` pairing documented in TileGrid + SelectionPanel; `fill_viewport: true` requirement stated with link to layouts.md |
| Cargo.toml 0.2.89 bump commit | CI publish.yml waves | push to master triggers publish | WIRED | CI run 28808914072: all 5 waves (1a→1b→1c→2→3) green; ferro-rs 0.2.89 + ferro-payments 0.1.6 on crates.io confirmed via API |

### Data-Flow Trace (Level 4)

Not applicable — this phase modifies MCP output functions (read-only advisory context generation) and documentation. No components that render dynamic user data from a database were modified.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SC-1 catalog count=52 (pre-existing) | `cargo test -p ferro-mcp -- test_all_components_present` | ok (1 passed) | PASS |
| BUILDER_API documents fill_viewport + each | `cargo test -p ferro-mcp -- builder_api_mentions_fill_viewport_and_each` | ok (1 passed) | PASS |
| RULE_COMPONENTS drift guard (3-direction) | `cargo test -p ferro-mcp -- design_system_component_guidance_drift_guarded` | ok (1 passed) | PASS |
| register_composition drift guard | `cargo test -p ferro-mcp -- register_composition_drift_guard` | ok (1 passed) | PASS |
| generation_context section completeness | `cargo test -p ferro-mcp -- test_generation_context_has_all_sections` | ok (1 passed) | PASS |
| mdBook build exits 0 | `mdbook build docs` | HTML book written to docs/book | PASS |
| ferro-rs 0.2.89 on crates.io | curl crates.io API | max_version: 0.2.89 | PASS |
| ferro-payments 0.1.6 on crates.io | curl crates.io API | max_version: 0.1.6 | PASS |
| git tag v0.2.89 on remote | `gh api repos/.../git/refs/tags/v0.2.89` | refs/tags/v0.2.89 | PASS |
| /cassa flip stands | `ls app/src/views/cassa.json` + `grep register_template cassa.rs` | file absent; 3 matches in cassa.rs | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| POS-12 | Plan 258-01, 258-02 | MCP + docs surface extended — json_ui_catalog entries, generation_context register guidance, docs/src updates | SATISFIED | json_ui_catalog BUILDER_API + RULE_COMPONENTS updated; GenerationContext.register_composition added; 5 component sections + layouts/spec-construction docs written; mdBook green |
| POS-13 | Plan 258-03 | /cassa flip stands, CI-exact gate green, single crates.io publish closes milestone | SATISFIED | cassa.json absent; CI run 28808914072 all green; ferro-rs 0.2.89 + ferro-payments 0.1.6 on crates.io; git tag v0.2.89; gestiscilo handoff brief in 258-03-SUMMARY.md |

**REQUIREMENTS.md traceability table note:** The bottom traceability table in REQUIREMENTS.md shows "Not started" for POS-12 and POS-13 — this is a stale table entry. The requirement bullets above the table carry `[x]` marks reflecting actual completion; the plan/summary frontmatter fields (`requirements: [POS-12]`, `requirements-completed: [POS-12]`) also reflect completion. The table was not updated as part of this phase's execution (pre-existing tracking gap, not a gap in the phase).

### Anti-Patterns Found

From the 258-REVIEW.md (0 critical / 5 warnings, all advisory):

| File | Finding | Severity | Impact on Must-Haves |
|------|---------|----------|----------------------|
| docs/src/json-ui/spec-construction.md:145-147 | WR-01: Builder-API doc example uses `ferro::json_ui::Spec` (not a re-exported path) and `SpecBuilder::new()` (private constructor) | Warning | None — the documentation of what fill_viewport() and each() do is accurate; only the code example's import path is broken |
| ferro-mcp/src/tools/generation_context.rs:594-606 | WR-02: Attribute drift guard checks only 5 of 13 REGISTER_DATA_ATTRIBUTES | Warning | None — all 13 attributes exist in the runtime (reviewer verified); drift coverage is partial but guidance content is correct |
| ferro-mcp/src/tools/generation_context.rs:581-592 | WR-03: Rule-id drift guard check is vacuous — iterates derived lint_rules back against design::rules() | Warning | None — content is correct; the count assertion in the section test catches removal; logic issue only in the test |
| docs/src/json-ui/components.md:1010-1018 | WR-04: Button props table omits `disable_on_submit` prop | Warning | None — SelectionPanel section (components.md:1499) fully describes the disable_on_submit usage and data-disable-on-submit guard; must-have "docs describe the double-submit pointer" is met there |
| docs/src/json-ui/components.md:25-36 | WR-05: Component Overview table at top of file lists only Tile under Commerce; TileGrid/SelectionPanel/FilterTabs/QuantityStepper/Numpad absent | Warning | None — must-have requires dedicated sections with props tables and examples, which all exist; the overview table is a discoverability gap, not a missing-documentation gap |

**Blocker count: 0.** All 5 warnings are docs/test-hardening advisory items. The reviewer confirmed 0 critical issues and noted that "Since 0.2.89 is already published, these feed a docs/tests patch, not a re-publish."

### Human Verification Required

None. All must-haves are verifiable programmatically. The phase's outcomes (code, tests, docs, publish) have been fully confirmed via automated checks and API calls.

### Gaps Summary

No gaps. All 11 observable truths are verified. Both POS-12 and POS-13 are satisfied. The five REVIEW warnings are advisory and do not block any must-have.

The 5 advisory warnings from the code review (WR-01 through WR-05) are candidates for a follow-up docs/test-hardening patch:
- WR-01: Fix `spec-construction.md` builder example (use `ferro::{Spec, SpecBuilder}` and `Spec::builder()`)
- WR-02: Extend drift guard to cover all 13 REGISTER_DATA_ATTRIBUTES by deriving from the array
- WR-03: Replace vacuous rule-id check with assertions against the hardcoded source array
- WR-04: Add `disable_on_submit` and `form` rows to the Button props table
- WR-05: Extend the Commerce Overview table with the five register components

None of these require a version bump — they are post-publish documentation corrections.

---

_Verified: 2026-07-06T18:00:00Z_
_Verifier: Claude (gsd-verifier)_
