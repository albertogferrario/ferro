---
status: passed
phase: 252-design-module-lint-cli
source: [252-VERIFICATION.md]
started: 2026-07-03T17:30:00Z
updated: 2026-07-28T00:00:00Z
---

## Current Test

Human-verified 2026-07-28.

## Tests

### 1. CLI output formatting quality

expected: `ferro design:lint app/src/views` prints findings grouped by file with
readable severity/rule/suggestion layout; on a clean tree prints a single clean
line. Visual formatting quality (spacing, styling, readability on a real
terminal with findings present) needs a human eye — ideally against a spec tree
with violations (e.g. a scratch copy of a view with `design` removed).
result: [passed] — clean run prints `No findings — all specs are clean.` (exit 0).
Findings-present run (login.json with `design` removed) output:
```
/tmp/test_views/bad.json
  info [declare-intent] No design.intent declared; inferred `collect` from spec content.
    → Add a `design.intent` field to declare the page archetype.

Summary: 0 warning(s), 1 info finding(s) across 1 file(s)
```
Format: file path header, indented severity/[rule-id] message, `→` suggestion, summary line. Clean and readable.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
