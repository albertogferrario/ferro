---
status: partial
phase: 243-app-integration-e2e-envelope-guard-catalog-docs
source: [243-VERIFICATION.md]
started: 2026-06-24T10:59:14Z
updated: 2026-06-24T10:59:14Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Live `:8090/mcp` create→list→update→delete drive
expected: With the app's `order` projection flipped to CRUD and a seeded `read_write`
bearer key, an agent drives create → list → update → delete through the live
`:8090/mcp` endpoint, each verb returning a well-formed Phase 205
`CallToolResult::structured` envelope, and `delete_order` enforcing the
`request_confirm_delete_order` / `confirm_delete_order` token flow. Per 243-CONTEXT.md
D-01/D-02 this was intentionally designated a manual UAT smoke (not a CI gate); the
in-process `crud_e2e.rs` harness already exercises the same shared kernel and
authorization path automatically, so this live drive is confirmation, not the
primary gate.
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
