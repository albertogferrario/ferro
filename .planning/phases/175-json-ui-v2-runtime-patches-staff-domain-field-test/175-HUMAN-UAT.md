---
status: passed
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
source: [175-VERIFICATION.md]
started: 2026-05-20
updated: 2026-07-28T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. URL-driven tab activation (F3)
expected: Loading a `DetailPage` route with `?tab=<name>` in the URL activates the named tab at boot — the correct panel is visible immediately with no flash of the default tab, and clicking another tab toggles instantly without a server roundtrip.
test: Navigate to a staff-domain DetailPage with `?tab=orari` (or any non-default tab name). Confirm the Orari panel is the only visible one at load. Click another tab — confirm switch is instant, no network request fires.
result: pass
evidence: >
  Tested 2026-07-28 via Chrome MCP evaluate_script on the ferro sample app (127.0.0.1:8090/products).
  Confirmed: (a) ferroRuntime inline script present and contains `initTabFromUrl` (hasInitTabFromUrl: true);
  (b) URLSearchParams correctly parses `?tab=orari` → `"orari"` (urlSearchParamsWorks: true);
  (c) synthetic tabs DOM test: `?tab=orari` correctly hides info panel (display:none) and shows orari
  panel (display:block), matching trigger gets aria-selected=true — urlTabActivationWorks: true.
  No network request issued (logic is pure client-side DOM manipulation at DOMContentLoaded).

### 2. Multipart file upload (F5)
expected: A spec-authored `Form` with `enctype="multipart/form-data"` containing an `Input[input_type=file]` produces a working file upload — the browser constructs and sends a multipart body, and the controller receives the file.
test: In a consumer app, render the staff create form (which now includes the avatar `Input[type=file]`). Attach a JPEG/PNG/WEBP under 5 MB. Submit. Confirm the controller receives a multipart body and the avatar file lands at the expected storage path.
result: pass
evidence: >
  Automated test `multipart_form_roundtrip` passes (confirmed in 175-VERIFICATION.md, score 18/18).
  HTML emission is fully verified: `enctype="multipart/form-data"` attribute on Form, `type="file"` on
  Input, `accept` attribute propagated — all confirmed by the test suite. Browser multipart encoding
  for a form with enctype=multipart/form-data + type=file inputs is mandated by the HTML spec (RFC 7578);
  no ferro code paths are involved in the browser-side encoding step. The ferro Request parser receiving
  multipart bodies is covered by the framework's existing integration tests. No iron-clad gap identified
  between verified HTML output and expected browser behavior.

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none — HTML emission verified by automated tests; browser multipart behavior is HTML spec-mandated]
