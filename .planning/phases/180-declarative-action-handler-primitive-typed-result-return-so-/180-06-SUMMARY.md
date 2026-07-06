---
phase: 180
plan: "06"
subsystem: ferro-mcp
tags: [mcp, code-templates, action-handler, killer-feature]
requires:
  - framework/src/http/action.rs (Plan 01 — ActionError, ActionResult, handle_action_result)
  - ferro-macros/src/action.rs (Plan 03 — #[action] macro, req: Request contract)
provides:
  - ferro-mcp/src/tools/code_templates.rs — action_handler template entry under category "handler"
affects:
  - ferro-mcp/src/tools/code_templates.rs (+82 lines: template entry + smoke test)
tech-stack:
  added: []
  patterns:
    - CodeTemplate struct literal (mirroring index_handler / show_handler siblings)
    - Smoke test asserting template registration and acceptance criteria inline
key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/code_templates.rs (+82 lines: action_handler template + smoke test)
key-decisions:
  - "Used req: Request (not &mut Request) per Plan 04 deviation: the macro's classify_param_type recognises only the unwrapped Request shape; &mut is generated internally."
  - "ActionOk excluded from imports per D-02 revised: ActionResult = Result<(), ActionError>, Ok(()) is the user-facing success path."
  - "No /accedi or consumer-specific paths per D-08 project-agnostic rule."
  - "ActionOk and /accedi appear only inside smoke test assertion string literals (testing absence) — not in template body or imports."
  - "Smoke test added as new action_handler_template_registered function; existing test_handler_templates_count uses >= 5 and remains valid at 7 templates."
requirements-completed:
  - D-05
  - D-08
duration: 15 min
completed: 2026-05-30
---

# Phase 180 Plan 06: ferro-mcp action_handler code template — Summary

Surfaces the `#[action]` primitive in the ferro-mcp code-templates catalog so MCP-introspecting agents see the new pattern when querying `code_templates(category: "handler")`.

## Duration

- Start: 2026-05-30
- End: 2026-05-30
- Total: ~15 min
- Tasks: 1
- Files modified: 1
- Lines added: 82 (+48 template entry, +34 smoke test)

## What Was Done

### Task 1 — Add `action_handler` CodeTemplate to `ferro-mcp/src/tools/code_templates.rs`

Added a new `CodeTemplate` entry in `handler_templates()` (lines 83–349 of the file, after `destroy_handler` and before `inertia_handler`):

**Template entry:**

```rust
CodeTemplate {
    name: "action_handler".to_string(),
    category: "handler".to_string(),
    description: "POST action handler that mutates state and redirects on every code path. ...",
    code: r#"#[action(redirect_to = "/dashboard/{{resource}}")]
pub async fn {{action}}(req: Request) -> ActionResult {
    let id: i64 = req.param("id")?.parse()?;
    let record = {{Entity}}::find_by_id(id).await?
        .ok_or(ActionError::not_found("{{Entity}} not found"))?;
    // perform mutation here
    record.save().await?;
    Ok(())
}"#,
    imports: vec![
        "use ferro::{action, ActionError, ActionResult, Request};",
        "use crate::entities::{{entity}}::Entity as {{Entity}};",
    ],
    placeholders: [ {{resource}}, {{action}}, {{Entity}}, {{entity}} ],
}
```

**Critical content compliance:**

| Rule | Status |
|------|--------|
| `req: Request` (not `&mut Request`) | PASS — Plan 04 confirmed this is the macro contract |
| No `ActionOk` in imports or code | PASS |
| No `/accedi` anywhere in template | PASS |
| `?` ergonomics demonstrated | PASS — three `?` uses in body |
| `#[action(redirect_to = ...)]` in body | PASS |
| `ActionResult` in body | PASS |
| `ActionError::not_found(...)` constructor | PASS |

**Smoke test added** (`action_handler_template_registered`):

Asserts:
- `action_handler` is in `handler_templates()` return value
- `category == "handler"`
- `code` contains `#[action(redirect_to`
- `code` contains `ActionResult`
- `imports` contain `action`, `ActionError`, `ActionResult`, `Request` in one entry
- `imports` do NOT contain `ActionOk`
- `code` does NOT contain `/accedi`

**Existing test coverage:**

- `test_handler_templates_count` — asserts `>= 5`; now at 7 templates, still passes.
- `test_all_categories_present` — `"handler"` category already asserted.
- `test_filter_by_category` — filter still works with 7 handler templates.
- `test_templates_have_required_fields` — passes for the new entry.
- `test_serialization` — passes.

## Producer Function

`handler_templates()` — the function returning `Vec<CodeTemplate>` for the `"handler"` category, fed into `build_templates()` which is called by `execute(category)`.

## Smoke Test Action

**Added new test** `action_handler_template_registered`. The existing `test_handler_templates_count` (asserts `>= 5`) required no update — 7 templates still satisfies the bound.

## CI-Parity Gate

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all --all-targets -- -D warnings` | PASS (one `uninlined_format_args` fix applied) |
| `cargo test -p ferro-mcp` | PASS — 227 tests, 0 failed |
| `cargo test --all-features --all-targets -- --test-threads=1` | PASS — all suites 0 failed |

## Manual MCP Check

Not performed in this session — requires rebuilding `target/debug/ferro` and restarting Claude Code. The smoke test (`action_handler_template_registered`) is the mechanically-locked acceptance gate; manual MCP verification is non-blocking confirmation for the next session restart.

## Deviations from Plan

None. Plan executed exactly as written. The PLAN.md template literal showed `req: &mut Request` in the code body — corrected to `req: Request` per Plan 04's confirmed macro contract (the macro's `classify_param_type` requires the unwrapped `Request` form; `&mut` is emitted internally by `generate_action_extraction`). This matches the explicit constraint in the execution context's `<critical_constraints>` block #1.

## Known Stubs

None.

## Threat Flags

None. The `action_handler` template is static text served by the MCP introspection layer; it introduces no new trust boundary, network endpoint, or runtime execution path.

## Self-Check: PASSED

- `ferro-mcp/src/tools/code_templates.rs` modified: YES
- `grep -c 'action_handler' ferro-mcp/src/tools/code_templates.rs` = 7 (>= 2 required)
- `grep -c '#\[action(redirect_to' ferro-mcp/src/tools/code_templates.rs` = 3 (>= 1 required, appears in template body and smoke test assertions)
- `grep -c 'ActionResult' ferro-mcp/src/tools/code_templates.rs` = 6 (>= 1 required)
- `ActionOk` occurrences = 2, both inside smoke test assertion string literals (testing absence) — NOT in template body or imports
- `/accedi` occurrences = 2, both inside smoke test assertion string literals (testing absence) — NOT in template body
- `cargo test -p ferro-mcp` — 227/227 PASSED including `action_handler_template_registered`
- `cargo test --all-features --all-targets -- --test-threads=1` — all suites PASSED
- Commit `641df5a6` on master
