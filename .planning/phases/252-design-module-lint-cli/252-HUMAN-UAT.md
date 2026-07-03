---
status: partial
phase: 252-design-module-lint-cli
source: [252-VERIFICATION.md]
started: 2026-07-03T17:30:00Z
updated: 2026-07-03T17:30:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. CLI output formatting quality

expected: `ferro design:lint app/src/views` prints findings grouped by file with
readable severity/rule/suggestion layout; on a clean tree prints a single clean
line. Visual formatting quality (spacing, styling, readability on a real
terminal with findings present) needs a human eye — ideally against a spec tree
with violations (e.g. a scratch copy of a view with `design` removed).
result: [pending] — auto-approved under the auto chain with captured evidence:
clean run prints `No findings — all specs are clean.` (exit 0); `--json` prints
`[]`; `--deny` exits 0 on clean tree. Findings-present formatting not yet
eyeballed by a human.

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
