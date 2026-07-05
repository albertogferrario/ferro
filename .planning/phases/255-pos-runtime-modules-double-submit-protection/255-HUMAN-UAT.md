---
status: partial
phase: 255-pos-runtime-modules-double-submit-protection
source: [255-VERIFICATION.md]
started: 2026-07-05T13:00:00Z
updated: 2026-07-05T13:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Form guard live-fires on /cassa confirm

expected: Open /cassa in a browser with the dev server running. Click "Conferma
ordine" once — the button disables (visually dims, `disabled` attribute set)
and exactly one POST to /cassa/conferma fires. Click again before navigation —
no second POST. Navigate back to /cassa via browser back button (bfcache) —
the confirm button is re-enabled and a new submit goes through.
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
