---
phase: 181
plan: "01"
subsystem: framework/json_ui
tags: [tdd, red-state, test-infrastructure, json-ui, form-validation]
dependency_graph:
  requires: []
  provides: [181-01-red-evidence]
  affects: [framework/src/json_ui/mod.rs]
tech_stack:
  added: []
  patterns: [html_body-over-response_body, pipeline-integration-test]
key_files:
  created: []
  modified:
    - framework/src/json_ui/mod.rs
decisions:
  - "html_body() is the correct helper for assertions on rendered HTML tags; response_body() captures Debug repr including data-view JSON and produces false positives"
  - "Tests require --all-features flag to be compiled (render_with_errors tests are under full feature set)"
metrics:
  duration: "~12 minutes"
  completed: "2026-05-31"
  tasks_completed: 2
  files_modified: 1
---

# Phase 181 Plan 01: Wave 0 Test Infrastructure Summary

Wave 0 test infrastructure establishing the RED state for the JSON-UI form-control error rendering bug. Four tests upgraded or added to `framework/src/json_ui/mod.rs`, all failing on master HEAD, proving the two pipeline root causes before any fix lands in Plan 02.

## One-liner

Pipeline-level TDD RED gate: 2 upgraded + 2 new tests proving `attach_errors` field-name mismatch and `resolve_expressions` runtime-data scoping failures.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Upgrade `render_with_errors_populates_form_fields` + `render_validation_error_accepts_framework_type` to use `html_body` and assert on `<p id="err-*">` DOM shape | `96367695` | `framework/src/json_ui/mod.rs` |
| 2 | Add `pipeline_data_binding_error_prop_renders_p_tag` (D-07a) + `pipeline_render_validation_error_renders_p_tag` (D-07b) | `e9bbf853` | `framework/src/json_ui/mod.rs` |

## Wave 0 RED State Evidence

All four tests FAIL on master HEAD (worktree base `577b67f2`). This is the load-bearing proof that Plan 02's Fix A + Fix B actually close the bug.

### Test 1: `render_with_errors_populates_form_fields` (upgraded)

```
thread 'json_ui::tests::render_with_errors_populates_form_fields' panicked at framework/src/json_ui/mod.rs:828:9:
error <p> must appear below name input; got: <!DOCTYPE html>
...
<div id="ferro-json-ui" data-view="...&quot;errors&quot;:[&quot;Name is required&quot;]...">
<div class="flex flex-wrap gap-4 ..."><!-- ferro-json-ui: failed to decode Form props: ... --></div>
</div>
```

Root cause visible in data-view: `"errors":["Name is required"]` (plural array written by `attach_errors`) — `InputProps.error: Option<String>` never populated because field name and shape don't match.

```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 528 filtered out
```

### Test 2: `render_validation_error_accepts_framework_type` (upgraded)

```
thread 'json_ui::tests::render_validation_error_accepts_framework_type' panicked at framework/src/json_ui/mod.rs:897:9:
error <p> must appear below name input; got: <!DOCTYPE html>
...
```

```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 528 filtered out
```

### Test 3: `pipeline_data_binding_error_prop_renders_p_tag` (new, D-07a)

```
thread 'json_ui::tests::pipeline_data_binding_error_prop_renders_p_tag' panicked at framework/src/json_ui/mod.rs:922:9:
error paragraph must appear below the input; got: <!DOCTYPE html>
...
```

Root cause: `resolve_expressions` reads `spec.data` only; runtime `data` argument containing `"email_error": "must be valid"` never reaches expression resolution. `{"$data": "/email_error"}` resolves to `Value::Null`.

```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 528 filtered out
```

### Test 4: `pipeline_render_validation_error_renders_p_tag` (new, D-07b)

```
thread 'json_ui::tests::pipeline_render_validation_error_renders_p_tag' panicked at framework/src/json_ui/mod.rs:952:9:
error paragraph must appear below the input; got: <!DOCTYPE html>
...
```

Root cause: `attach_errors` writes `"errors": ["must be valid"]` (plural array); `InputProps.error: Option<String>` deserializes with `error: None` because serde ignores the unknown `errors` field.

```
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 527 filtered out
```

## Acceptance Criteria Verification

- `grep -n 'html_body(ok_response(result))' framework/src/json_ui/mod.rs` — 10 matches, including the 4 test functions in scope
- `grep -n 'fn pipeline_data_binding_error_prop_renders_p_tag'` — exactly 1 match (line 906)
- `grep -n 'fn pipeline_render_validation_error_renders_p_tag'` — exactly 1 match (line 934)
- `grep -n 'r#"<p id="err-'` — 6 matches (2 per upgraded test, 1 per new test)
- `cargo test --no-run -p ferro-rs --all-features` — succeeds (all 4 tests compile)
- All 4 tests exit NON-ZERO with assertion failure on `<p id="err-` substring

## Deviations from Plan

### Auto-discovery: tests require `--all-features`

The plan specified `cargo test -p framework --lib ...` but the package name is `ferro-rs` and the render_with_errors tests are only compiled under `--all-features`. Adjusted all verification commands to use `-p ferro-rs --lib --all-features`. No code change required.

## Known Stubs

None — this plan adds test code only, no production stubs.

## Threat Flags

None — test-only changes, no new attack surface.

## Self-Check: PASSED

- `framework/src/json_ui/mod.rs` — modified, exists: FOUND
- Commit `96367695` — exists: FOUND
- Commit `e9bbf853` — exists: FOUND
- All 4 tests compile: PASSED (`cargo test --no-run -p ferro-rs --all-features`)
- All 4 tests FAIL on master HEAD: CONFIRMED
