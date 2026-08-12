---
status: passed
phase: 164-json-ui-improvements-batch-3
source: [164-VERIFICATION.md]
started: 2026-05-17T03:35:00Z
updated: 2026-07-28T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Visual diff of `CardVariant::Bordered` vs `CardVariant::Elevated` rendering
expected: Bordered renders dashboard card chrome (border + shadow-sm + p-4); Elevated renders auth-page chrome (shadow-md + p-8, no border).
why_human: Unit tests assert CSS class strings only. Visual correctness on actual rendered output requires a browser comparison against the reference screenshot in `V7-RUNTIME-FRICTION.md` (`login-prod.png`).
how_to_verify:
  1. Boot a ferro app that uses both Card variants — e.g. a dashboard page with `"variant": "bordered"` and an auth/login page with `"variant": "elevated"`.
  2. In the browser, confirm: bordered card has a visible 1px border + soft shadow + ~16px padding; elevated card has a stronger shadow, no border, and ~32px padding.
  3. Optionally compare against `login-prod.png` from gestiscilo's V7-RUNTIME-FRICTION.md captures.
result: pass
evidence: >
  Ferro sample app (127.0.0.1:8090) tested 2026-07-28 via Chrome MCP screenshot.
  /auth/login (CardAppearance::Elevated): floating card centered on dark page, no visible border, strong
  drop shadow (shadow-md), generous padding (~32px). Matches expected auth-page chrome.
  /products/new (CardAppearance::Bordered, default): two side-by-side dashboard cards with a visible
  1px border, soft shadow (shadow-sm), tighter padding (~16px). Matches expected dashboard chrome.
  Both variants render distinctly and correctly — visual difference is unambiguous.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
