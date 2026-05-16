---
status: partial
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
source: [163-VERIFICATION.md, 163-REVIEW.md]
started: 2026-05-16T00:00:00Z
updated: 2026-05-16T00:00:00Z
---

## Current Test

[awaiting human decision on WR-01 codemod multi-root handler]

## Tests

### 1. Codemod multi-root handler behavior (WR-01)
expected: Either (a) root wraps page-title and login-form in a Group/Fragment container, or (b) multi-root handlers are rejected as Unsupported with a TODO marker. Currently root is 'page-title' and login-form/email/password/submit are orphaned (unreachable from root).
result: [pending — design decision required]
file: ferro-cli/src/commands/json_ui_migrate_v1.rs:253-286
fixture_demonstrating_bug: ferro-cli/tests/fixtures/migrate_v1/out_auth_login_form.json
related: integration test `codemod_one_handler_emits_spec_and_rewrites_controller` passes only because the fixture encodes the same bug
why_human: WR-01 from code review — the codemod emits a structurally valid JSON spec (serde-clean) but semantically wrong; elements are unreachable from root. The correct fix requires a design decision about which repair approach to take.

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps

### G-163-01: Codemod multi-root handler produces orphaned elements
status: failed
source: 163-REVIEW.md WR-01
plan: 163-07
file: ferro-cli/src/commands/json_ui_migrate_v1.rs
lines: 253-286
description: When a v1 handler contains multiple top-level nodes (e.g., `page-title` outside a `login-form`), the codemod sets `root` to the first node and silently orphans the rest. Running the codemod on a real multi-root controller would produce a page rendering only the first element.

repair_options:
  - id: A
    label: Wrap in Group/Fragment container
    description: Add a synthetic `Group` element as root that contains all top-level nodes as children. Requires `Group` component to exist in the v2 catalog (it does not yet — would need a parallel plan to add it).
    scope: larger — adds new component to spec surface
  - id: B
    label: Reject as Unsupported with TODO marker (recommended)
    description: Treat multi-root handlers like the existing runtime-branching case — emit a `// TODO: codemod could not auto-translate` marker on the controller. Aligns with D-11 ("best-effort; cases the codemod cannot translate get a TODO marker, not a silent skip"). Smaller diff, more conservative.
    scope: targeted fix — single match in codemod path + fixture rewrite + test update

after_decision_required:
  - Update fixture `ferro-cli/tests/fixtures/migrate_v1/out_auth_login_form.json` to reflect the chosen behavior (Option A: emit a Group root; Option B: delete the fixture and replace integration test with TODO-marker assertion)
  - Update integration test `codemod_one_handler_emits_spec_and_rewrites_controller`
  - Update fixture `in_auth.rs` if Option B (or add a separate single-root fixture)
