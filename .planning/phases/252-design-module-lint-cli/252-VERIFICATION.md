---
phase: 252-design-module-lint-cli
verified: 2026-07-03T00:00:00Z
status: human_needed
score: 6/6
overrides_applied: 0
human_verification:
  - test: "Run `ferro design:lint app/src/views` from repo root"
    expected: "Findings grouped by file, with severity (warning/info), rule ID, message, and suggestion on separate indented lines; summary count at the bottom"
    why_human: "Terminal output visual formatting — color, indentation, and readability are aesthetic judgments that cannot be verified programmatically"
---

# Phase 252: design-module-lint-cli Verification Report

**Phase Goal:** Codify composition patterns as a machine-readable, testable rule set — `Spec` gains an optional `design` field (`intent` + `allow`), a pure `design::lint(&Spec)` engine implements the intent-keyed rules, and `ferro design:lint` surfaces findings from the command line.
**Verified:** 2026-07-03
**Status:** human_needed
**Re-verification:** No — initial verification

## Summary

All six ROADMAP success criteria are verified. The design module exists, is substantive, and is wired. The lint engine is pure (no I/O, no panics), 10 rules are implemented with violating+conforming test pairs, the CLI command is registered with `--json`/`--deny` flags, all three app views lint clean, and the review-fix regressions (WR-01, WR-02, WR-03) are present and passing. The only remaining item is a human assessment of terminal output formatting quality.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `Spec` accepts optional `design` field; invalid `intent` values and unknown `allow` ids are reported as findings, never errors | VERIFIED | `DesignMeta { intent: Option<String>, allow: Vec<String> }` in `spec.rs:76-84`; `design: Option<DesignMeta>` in `spec.rs:113`; tests `design_meta_unknown_intent_parses_without_error` and `unknown_allow_id_emits_warning` both pass |
| 2 | All 10 rules implemented with a violating + conforming test pair per rule | VERIFIED | `RULE_REGISTRY` contains 10 `DesignRule` entries; `rules.rs` has 30 rule tests covering all 10 rules in both directions; `cargo test -p ferro-json-ui design` reports 46 passed, 0 failed |
| 3 | Undeclared intent is inferred from spec content and reported as an info finding | VERIFIED | `lint()` None branch in `mod.rs:86-103`; inference priority in `infer.rs:22-36`; tests `undeclared_intent_with_data_table_emits_info_browse` and `undeclared_intent_no_signal_emits_info_none_inferred` pass |
| 4 | `ferro design:lint [path] [--json] [--deny]` walks spec files; exit non-zero only under `--deny` with warning-level findings | VERIFIED | `main.rs:498-508` defines `DesignLint { path, json, deny }`; `main.rs:803-804` routes to `commands::design_lint::run`; `run()` exits only when `deny && has_warning(&all)` (`design_lint.rs:132-134`); 8 CLI tests pass |
| 5 | Sample `app/` views lint clean, enforced by a test | VERIFIED | `app_views_lint_clean` test passes (`cargo test -p app design_lint`: 1 passed); login.json carries `intent: collect`, login_confirm.json carries `intent: focus`, pagamenti.json carries `intent: summarize` with a `PageHeader` element |
| 6 | Lint never affects rendering or spec validation | VERIFIED | `lint()` takes `&Spec` (immutable), returns `Vec<Finding>`, performs no I/O and no mutation; `grep` of `render.rs` finds no reference to the design module; lint is not called from any rendering path |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/spec.rs` | `DesignMeta` struct + `Spec.design` optional field | VERIFIED | `pub struct DesignMeta` at line 76; `pub design: Option<DesignMeta>` at line 113; three round-trip tests present |
| `ferro-json-ui/src/design/mod.rs` | `lint()`, `rules()`, `KNOWN_INTENTS`, drift test | VERIFIED | 315 lines; `pub fn lint` at line 61; `pub const KNOWN_INTENTS` at line 37; drift test gated on `#[cfg(all(test, feature = "projections"))]` at line 286 |
| `ferro-json-ui/src/design/types.rs` | `Finding`, `Severity`, `DesignRule` | VERIFIED | `Severity` enum with Info/Warning; `Finding` struct with rule/element_id/severity/message/suggestion; `DesignRule` fn-pointer struct |
| `ferro-json-ui/src/design/infer.rs` | `infer_intent` heuristic | VERIFIED | `pub(super) fn infer_intent(spec: &Spec) -> Option<&'static str>` at line 15; priority: KanbanBoard→process, Form→collect, DataTable/Table→browse, 2×StatCard→summarize, else None; 7 inference tests pass |
| `ferro-json-ui/src/design/rules.rs` | 10-rule `RULE_REGISTRY` | VERIFIED | `pub(super) static RULE_REGISTRY: &[DesignRule] = &[...]` with 10 entries; WR-01 fix (`!v.is_null()` on `empty_message`) at line 147-149; WR-02 fix (`!v.is_null()` on `breadcrumb`) at line 206-209 |
| `ferro-cli/src/commands/design_lint.rs` | CLI implementation (`lint_content`, `has_warning`, `run`) | VERIFIED | 318 lines; WR-03 fix (I/O errors push `FileFinding` with `severity: Warning`) at lines 104-119; 8 CLI tests pass |
| `app/src/tests/design_lint.rs` | D-17 app-views lint-clean gate | VERIFIED | `app_views_lint_clean` test walks `app/src/views/*.json` and asserts zero findings; passes |
| `app/src/views/pagamenti.json` | `design.intent` + `PageHeader` element | VERIFIED | `"design": { "intent": "summarize" }` at line 5; `page_header` element with `type: PageHeader, props.title: Pagamenti` at lines 13-16 |
| `app/Cargo.toml` | `ferro-json-ui` dev-dependency | VERIFIED | `ferro-json-ui = { path = "../ferro-json-ui" }` at line 50 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-json-ui/src/lib.rs` | design module | `pub mod design` + `pub use design::{...}` | VERIFIED | `lib.rs:35` declares `pub mod design;`; `lib.rs:65` re-exports `lint, rules, DesignMeta, DesignRule, Finding, Severity, KNOWN_INTENTS` |
| `ferro-json-ui/src/design/mod.rs` | `infer::infer_intent` | called in `lint()` None and unknown-intent branches | VERIFIED | `mod.rs:84` and `mod.rs:88` call `infer::infer_intent(spec)` |
| `ferro-json-ui/src/design/mod.rs` | `ferro_projections::Intent::label()` | feature-gated drift test | VERIFIED | drift test at line 286 guarded by `#[cfg(all(test, feature = "projections"))]` |
| `ferro-cli/src/commands/mod.rs` | design_lint module | `pub mod design_lint;` | VERIFIED | `commands/mod.rs:17` |
| `ferro-cli/src/main.rs` | `commands::design_lint::run` | `Commands::DesignLint { path, json, deny }` dispatch | VERIFIED | `main.rs:803-804` |
| `app/src/tests/design_lint.rs` | `ferro_json_ui::design::lint` | `lint(&spec)` call in test | VERIFIED | `design_lint.rs:6-7` imports; test walks all `.json` views and calls `lint` |
| `app/src/tests/mod.rs` | design_lint module | `pub mod design_lint;` | VERIFIED | `mod.rs:3` |

### Data-Flow Trace (Level 4)

Not applicable — all artifacts in this phase are a pure engine, CLI tool, and tests. No dynamic data rendering. `lint(&spec)` takes a parsed spec struct and returns findings; there is no data source to trace beyond the spec itself.

### Behavioral Spot-Checks

Targeted tests run per prompt instructions (CPU-serialization rule: one at a time):

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All design module tests pass (rules + engine + inference) | `cargo test -p ferro-json-ui design` | 46 passed, 0 failed | PASS |
| CLI design_lint tests pass (including WR-03 regression) | `cargo test -p ferro-cli design_lint` | 8 passed, 0 failed | PASS |
| D-17 app-views lint-clean gate | `cargo test -p app design_lint` | 1 passed, 0 failed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| DS-05 | 252-01 through 252-04 | `Spec.design` field + pure lint engine + ~10 intent-keyed rules + violating/conforming test pairs per rule | SATISFIED | All 10 rules in `RULE_REGISTRY`; 46 design tests pass; engine is pure (no I/O, no panic) |
| DS-06 | 252-05 | `ferro design:lint [path] [--json] [--deny]` CLI | SATISFIED (functional) | CLI registered in main.rs; `--json` and `--deny` flags present; exit logic correct; 8 CLI tests pass; visual output format is the single human item |

### Anti-Patterns Found

| File | Location | Pattern | Severity | Impact |
|------|----------|---------|----------|--------|
| `ferro-json-ui/src/design/rules.rs` | Line 298 | `FIELD_TYPES` includes `"Textarea"` with no registered builtin component | Info | Dead constant entry — catalog validation rejects `Textarea` type before lint runs; no runtime impact (IN-01 from code review, deferred) |
| `ferro-cli/src/commands/design_lint.rs` | `print_human` fn | "No findings — all specs are clean" emitted even when zero files were linted | Info | Misleading message in edge case; does not affect correctness or `--deny` exit (IN-02 from code review, deferred) |

No blocker anti-patterns found. Review-fix regression tests for WR-01 (`list_empty_state_conforming_data_bound_empty_message`) and WR-02 (`breadcrumb_on_subpages_conforming_data_bound_breadcrumb_prop`) are present and passing.

### Human Verification Required

#### 1. Human-readable CLI output formatting

**Test:** From the repo root, run `cargo run --bin ferro -- design:lint app/src/views` (or the installed binary `ferro design:lint app/src/views`).
**Expected:** The output groups findings by file with bold underlined file paths, severity labels (warning in yellow, info in cyan), rule ID in dim brackets, message, and a dimmed "→ suggestion" line. A summary count line appears at the bottom. If all three views carry valid `design.intent`, no findings are emitted and the output reads "No findings — all specs are clean."
**Why human:** Terminal output color, indentation, and overall readability are visual judgments that cannot be verified programmatically. The code structure is correct but the rendered output needs a human eye.

### Pre-existing Flaky Test (Informational)

`commands::serve::tests::spawn_child_with_prefix_uses_new_process_group` fails under the full parallel test suite due to a timing race unrelated to Phase 252 changes. Documented in `deferred-items.md`. Not a Phase 252 gap.

### Review Fixes Verified

| Finding | Fix | Regression Test | Status |
|---------|-----|----------------|--------|
| WR-01: `check_list_empty_state` false positive for `$data`-bound `empty_message` | Changed `.and_then(|v| v.as_str()).is_some()` to `.map(|v| !v.is_null()).unwrap_or(false)` | `list_empty_state_conforming_data_bound_empty_message` | PASS |
| WR-02: `check_breadcrumb_on_subpages` false positive for `$data`-bound `breadcrumb` prop | Changed `.and_then(|v| v.as_array()).map(|a| !a.is_empty()).unwrap_or(false)` to `.map(|v| !v.is_null()).unwrap_or(false)` | `breadcrumb_on_subpages_conforming_data_bound_breadcrumb_prop` | PASS |
| WR-03: File I/O errors silently swallowed in CLI walker | Changed `Err(_) => continue` to push a `FileFinding` with `severity: Warning` | `has_warning_true_for_file_read_finding` | PASS |

Commits `0c7fada3` (WR-01+WR-02) and `f3b71901` (WR-03) confirmed present in git history.

---

_Verified: 2026-07-03_
_Verifier: Claude (gsd-verifier)_
