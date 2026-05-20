---
status: partial
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
source: [175-VERIFICATION.md]
started: 2026-05-20
updated: 2026-05-20
---

## Current Test

[awaiting human testing — both items are browser-side confirmations of already-verified HTML output]

## Tests

### 1. URL-driven tab activation (F3)
expected: Loading a `DetailPage` route with `?tab=<name>` in the URL activates the named tab at boot — the correct panel is visible immediately with no flash of the default tab, and clicking another tab toggles instantly without a server roundtrip.
test: Navigate to a staff-domain DetailPage with `?tab=orari` (or any non-default tab name). Confirm the Orari panel is the only visible one at load. Click another tab — confirm switch is instant, no network request fires.
result: [pending]

### 2. Multipart file upload (F5)
expected: A spec-authored `Form` with `enctype="multipart/form-data"` containing an `Input[input_type=file]` produces a working file upload — the browser constructs and sends a multipart body, and the controller receives the file.
test: In a consumer app, render the staff create form (which now includes the avatar `Input[type=file]`). Attach a JPEG/PNG/WEBP under 5 MB. Submit. Confirm the controller receives a multipart body and the avatar file lands at the expected storage path.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps

[none yet — these are integration confirmations of already-verified HTML emission, not implementation gaps]
