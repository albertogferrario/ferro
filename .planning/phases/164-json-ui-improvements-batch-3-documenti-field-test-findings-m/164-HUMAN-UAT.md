---
status: partial
phase: 164-json-ui-improvements-batch-3
source: [164-VERIFICATION.md]
started: 2026-05-17T03:35:00Z
updated: 2026-05-17T03:35:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Visual diff of `CardVariant::Bordered` vs `CardVariant::Elevated` rendering
expected: Bordered renders dashboard card chrome (border + shadow-sm + p-4); Elevated renders auth-page chrome (shadow-md + p-8, no border).
why_human: Unit tests assert CSS class strings only. Visual correctness on actual rendered output requires a browser comparison against the reference screenshot in `V7-RUNTIME-FRICTION.md` (`login-prod.png`).
how_to_verify:
  1. Boot a ferro app that uses both Card variants — e.g. a dashboard page with `"variant": "bordered"` and an auth/login page with `"variant": "elevated"`.
  2. In the browser, confirm: bordered card has a visible 1px border + soft shadow + ~16px padding; elevated card has a stronger shadow, no border, and ~32px padding.
  3. Optionally compare against `login-prod.png` from gestiscilo's V7-RUNTIME-FRICTION.md captures.
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
