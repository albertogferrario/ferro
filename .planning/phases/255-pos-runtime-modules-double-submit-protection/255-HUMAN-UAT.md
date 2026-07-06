---
status: passed
phase: 255-pos-runtime-modules-double-submit-protection
source: [255-VERIFICATION.md]
started: 2026-07-05T13:00:00Z
updated: 2026-07-05T14:45:00Z
---

## Current Test

[complete]

## Tests

### 1. Form guard live-fires on /cassa confirm

expected: Open /cassa in a browser with the dev server running. Click "Conferma
ordine" once — the button disables (visually dims, `disabled` attribute set)
and exactly one POST to /cassa/conferma fires. Click again before navigation —
no second POST. Navigate back to /cassa via browser back button (bfcache) —
the confirm button is re-enabled and a new submit goes through.
result: PASS (2026-07-05, live Chrome via chrome-devtools MCP against the dev
server on :8090; screenshot `app/tmp/255-uat-cassa.png`). Evidence:
- Guard bound on load: `btn._submitted === false` (initDisableOnSubmit ran),
  button inside `form[action="/cassa/conferma"][method="post"]`.
- Synthetic pass (navigation intercepted downstream of the guard): click 1 →
  submit allowed, button `disabled` + `opacity-50` + `_submitted: true`;
  click 2 on the disabled button → no submit event fired; `form.requestSubmit()`
  (Enter-key path) → blocked by the guard via `preventDefault`.
- Real network pass: one click produced exactly ONE
  `POST /cassa/conferma` (302) followed by exactly one `GET /cassa` (PRG);
  button disabled the same tick as the click.
- Back-navigation (`performance` navigationType `back_forward`): button
  re-enabled, `_submitted` reset to false, no dim classes; a fresh submit went
  through the guard exactly once and latched again (D-15 outcome verified).
- Observation (non-blocking): the demo `conferma` handler redirects with 302
  rather than 303; PRG behavior is correct in practice. Cosmetic only — the
  handler is a plain-redirect demo slated for the Phase 257 projection flip.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
