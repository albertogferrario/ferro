---
phase: 164
plan: "02"
subsystem: ferro-cli
tags: [regression-test, codemod, json-ui, http-method, d-19-f2]
dependency_graph:
  requires: []
  provides: [D-19-F2-locked]
  affects: [ferro-cli/tests/json_ui_migrate_v1.rs]
tech_stack:
  added: []
  patterns: [TDD regression test, fixture-driven integration test]
key_files:
  created:
    - ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs
  modified:
    - ferro-cli/tests/json_ui_migrate_v1.rs
decisions:
  - Use `fields: vec![make_node_with_action(...)]` inside a single-root `Form` component to get all five verb actions into one codemod-compatible handler (codemod requires exactly one top-level node)
  - Test asserts both presence of uppercase and absence of lowercase via substring match (handles both compact and pretty-printed JSON)
  - Fixture handler named `show` so codemod emits `src/views/in_all_verbs/show.json`
metrics:
  duration: "~15m"
  completed: "2026-05-17T01:05:27Z"
  tasks_completed: 2
  files_changed: 2
---

# Phase 164 Plan 02: Uppercase HTTP Method Regression Test Summary

One-liner: Regression test locking the codemod's uppercase HTTP method emission (POST/GET/PUT/PATCH/DELETE) via a five-verb fixture and a new `codemod_emits_uppercase_http_methods` test.

## Objective Achieved

D-19/F2 (V7-RUNTIME-FRICTION): verified and locked the codemod's uppercase HTTP method contract. The 26 gestiscilo specs that required a `sed` workaround were hand-authored, not codemod output. The codemod at `ferro-cli/src/commands/json_ui_migrate_v1.rs:521-528` already emits uppercase — this plan adds a regression test that will fail immediately if anyone reverts that mapping.

## Codemod Audit Result

```
grep -n '=> "POST"\|=> "GET"\|=> "PUT"\|=> "PATCH"\|=> "DELETE"' \
  ferro-cli/src/commands/json_ui_migrate_v1.rs
522:        "post" => "POST",
523:        "get" => "GET",
524:        "put" => "PUT",
525:        "patch" => "PATCH",
526:        "delete" => "DELETE",
```

Five arms confirmed, all uppercase. Production code NOT modified.

## New Fixture

**`ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs`**

- Single-root handler (`show`) with one top-level `Form` component
- Five child buttons via `make_node_with_action`, one per verb: POST, GET, PUT, PATCH, DELETE
- Handler name `show` → codemod emits `src/views/in_all_verbs/show.json`

## New Regression Test

**`codemod_emits_uppercase_http_methods`** in `ferro-cli/tests/json_ui_migrate_v1.rs`

- Runs the codemod on `in_all_verbs.rs` fixture
- Reads the emitted `src/views/in_all_verbs/show.json`
- Asserts each of POST, GET, PUT, PATCH, DELETE appears in the JSON
- Asserts none of post, get, put, patch, delete appears in the JSON

### Verb Coverage Matrix

| Verb | Uppercase present | Lowercase absent |
|------|-------------------|-----------------|
| POST | asserted | asserted |
| GET | asserted | asserted |
| PUT | asserted | asserted |
| PATCH | asserted | asserted |
| DELETE | asserted | asserted |

## Commits

| Task | Description | Commit |
|------|-------------|--------|
| 1+2 | Add fixture and regression test | dd890ff7 |

## Deviations from Plan

None — plan executed exactly as written.

The plan specified two artifact paths (`in_post_action.rs` as primary, `in_all_verbs.rs` as optional consolidated). Executor judgement: a single consolidated fixture (`in_all_verbs.rs`) is simpler and covers all five verbs in one file. The PLAN frontmatter lists `in_post_action.rs` but notes `in_all_verbs.rs` as the preferred consolidated alternative. Only `in_all_verbs.rs` was created.

## Self-Check

### Created files exist

- `ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs` — FOUND
- `ferro-cli/tests/json_ui_migrate_v1.rs` (modified) — FOUND

### Commits exist

- dd890ff7 — FOUND (verified via `git log --oneline -1`)

### Test passes

- `cargo test -p ferro-cli --test json_ui_migrate_v1 codemod_emits_uppercase_http_methods` — PASSED

### Pre-commit gate

- `cargo fmt --all -- --check` — CLEAN
- `cargo clippy --all --all-targets -- -D warnings` — CLEAN
- `cargo test -p ferro-cli` — 516+7+11+4+3 = 541 tests, 0 failures

## Self-Check: PASSED
