---
phase: 120-cli-and-mcp-updates
plan: "05"
subsystem: ferro-cli
tags: [cli, make-json-view, v2, ai, two-pass, validation, fallback]
dependency_graph:
  requires: [120-04]
  provides: [TOOL-01]
  affects: [ferro-cli/src/commands/make_json_view.rs, ferro-cli/src/ai.rs, ferro-cli/src/templates/make.rs]
tech_stack:
  added: []
  patterns: [two-pass-ai-generation, catalog-validation-fallback, static-template-fallback]
key_files:
  created: []
  modified:
    - ferro-cli/src/commands/make_json_view.rs
    - ferro-cli/src/ai.rs
    - ferro-cli/src/templates/make.rs
decisions:
  - call_anthropic_plain added as alias for call_anthropic; call_anthropic_structured changed to return String (serializes Value internally)
  - build_json_view_pass1 and build_json_view_pass2 are prompt builders in ai.rs, not in make_json_view.rs
  - test module in make.rs moved to end of file to satisfy items_after_test_module clippy lint
metrics:
  duration: ~25 minutes
  completed: 2026-04-21
  tasks_completed: 2
  files_modified: 3
---

# Phase 120 Plan 05: make:json-view v2 Output + Two-Pass AI Summary

One-liner: `ferro make:json-view` now writes `src/views/{name}.json` with two-pass AI generation (pass1=plain plan, pass2=structured spec), catalog validation, and static fallback on failure.

## What Was Built

### Task 1: templates::json_view_template returns v2 JSON spec string

`ferro-cli/src/templates/make.rs` `json_view_template` was rewritten:

Before:
```rust
pub fn json_view_template(_name: &str, title: &str, layout: &str) -> String {
    // returned JSON without Card description, _name unused
```

After:
```rust
pub fn json_view_template(name: &str, title: &str, layout: &str) -> String {
    // returns JSON with Card props.description = "Edit src/views/{name}.json to customize this view."
```

The Card `props` now includes a `description` field referencing `src/views/{name}.json`. Four tests added at end of file (required `items_after_test_module` lint move).

### Task 2: make_json_view::run rewritten for .json output + two-pass AI + validation fallback

**ai.rs additions:**
- `call_anthropic_plain`: alias for `call_anthropic`, canonical name from Plan 05 interface contract
- `call_anthropic_structured`: signature changed from `(&serde_json::Value) -> Result<serde_json::Value, String>` to `(serde_json::Value) -> Result<String, String>` — serialization happens inside the function
- `build_json_view_pass1(name, description) -> (String, String)`: builds (system, user) prompts for Pass 1
- `build_json_view_pass2(pass1_result, schema) -> (String, String)`: builds (system, user) prompts for Pass 2

**make_json_view.rs control flow (before → after):**

Before:
```
run() → ai::generate_json_view() [monolithic: passes 1+2+validation inside ai.rs]
```

After:
```
run()
  └─ generate_with_ai()          [private fn in make_json_view.rs]
       ├─ ai::build_json_view_pass1() → ai::call_anthropic_plain()   [Pass 1]
       ├─ ai::build_json_view_pass2() → ai::call_anthropic_structured() [Pass 2]
       └─ ferro_json_ui::Spec::from_json() + global_catalog().validate() [validation + fallback]
```

**Output changes:**
- File extension: `.json` (not `.rs`)
- Default layout: `"dashboard"` (was `"app"`)
- No `mod.rs` operations (deleted `update_mod_file`)
- Usage message: `JsonUi::render_file("views/{name}.json", data)` with v2 handler pattern

## Before/After diff: control flow summary

| Aspect | Before | After |
|--------|--------|-------|
| Output file | `src/views/{name}.rs` (Rust source) | `src/views/{name}.json` (JSON spec) |
| mod.rs | Updated via `update_mod_file` | Not touched |
| AI path | `ai::generate_json_view()` (monolithic) | `generate_with_ai()` (orchestrates primitives) |
| Pass 1 | Inside `generate_json_view` | `ai::build_json_view_pass1` + `ai::call_anthropic_plain` |
| Pass 2 | Inside `generate_json_view` | `ai::build_json_view_pass2` + `ai::call_anthropic_structured` |
| Validation | Inside `generate_json_view`, returns Err | In `generate_with_ai`, falls back to static template |
| Default layout | `"app"` | `"dashboard"` |

## v1 Removal Verification

```
grep -rn "Spec::builder|Element::new|JsonUiView" ferro-cli/src/commands/make_json_view.rs
→ 0 hits

grep -rn "Spec::builder|Element::new|JsonUiView" ferro-cli/src/templates/make.rs
→ 1 hit: test marker strings in assertion (not actual API usage)

grep -rn "Spec::builder|Element::new|JsonUiView" ferro-cli/src/ai.rs
→ 0 hits
```

Note: `ferro-cli/src/templates/module.rs` contains v1 API calls (`Spec::builder`, `Element::new`) — these are pre-existing and out of scope for this plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wave 1 agent did not add `call_anthropic_plain`, `build_json_view_pass1`, `build_json_view_pass2` to ai.rs**
- **Found during:** Task 2 verification of must_haves
- **Issue:** Plan 05 requires `make_json_view.rs` to call `ai::call_anthropic_plain`, `ai::build_json_view_pass1`, `ai::build_json_view_pass2`. Wave 1's `ai.rs` had only `call_anthropic` (not `_plain`) and `call_anthropic_structured` returning `serde_json::Value` not `String`.
- **Fix:** Added `call_anthropic_plain` as thin alias for `call_anthropic`. Changed `call_anthropic_structured` to accept `serde_json::Value` by value and return `Result<String, String>` (serializes internally). Added `build_json_view_pass1` and `build_json_view_pass2` as prompt builder functions.
- **Files modified:** `ferro-cli/src/ai.rs`
- **Commit:** 5baf31db

**2. [Rule 1 - Clippy] items_after_test_module lint failure**
- **Found during:** Task 1 clippy run
- **Issue:** Test module was added at line 133 of `make.rs` with ~650 lines of public functions after it. Clippy `-D warnings` rejects `items_after_test_module`.
- **Fix:** Moved test module to end of file (line 785+).
- **Files modified:** `ferro-cli/src/templates/make.rs`
- **Commit:** 5baf31db (included in same commit)

**3. [Rule 1 - Fmt] cargo fmt diffs after initial commits**
- **Found during:** Post-commit fmt check
- **Fix:** Applied `cargo fmt -p ferro-cli`, committed as style commit.
- **Commit:** 1968b916

## Threat Mitigation Verification

| Threat | Mitigation Present |
|--------|-------------------|
| T-120-13: Malicious AI spec output | `Spec::from_json` + `global_catalog().validate()` gate in `generate_with_ai`; fallback to static template on failure |
| T-120-14: Path traversal via name | `is_valid_identifier` rejects non-`[A-Za-z0-9_]` names; `.json` extension hardcoded in `format!` |
| T-120-15: AI output logged in error case | Only `CatalogError` enum values printed, not raw AI response body |
| T-120-16: Two API round trips | `call_anthropic_plain` 60s timeout, `call_anthropic_structured` 90s timeout |

## Suggested Manual Smoke Commands

```bash
# Requires a ferro project directory structure
cd /tmp && mkdir ferro-test && cd ferro-test && mkdir -p src/views

# Static fallback (no API key needed)
cargo run -p ferro-cli --bin ferro -- make:json-view Demo --no-ai
cat src/views/demo.json | python3 -m json.tool   # must be valid JSON

# AI path (requires ANTHROPIC_API_KEY)
export ANTHROPIC_API_KEY=sk-ant-...
cargo run -p ferro-cli --bin ferro -- make:json-view Dashboard --description "analytics overview with KPIs"
cat src/views/dashboard.json | python3 -m json.tool
```

## Self-Check

Files exist:
- ferro-cli/src/commands/make_json_view.rs: EXISTS
- ferro-cli/src/ai.rs: EXISTS
- ferro-cli/src/templates/make.rs: EXISTS

Commits:
- d29a7405: feat(120-05): rewrite json_view_template to return v2 JSON spec string
- 5baf31db: feat(120-05): rewrite make_json_view for .json output + two-pass AI + validation fallback
- 1968b916: style(120-05): apply cargo fmt to ferro-cli changes

## Self-Check: PASSED
