---
status: partial
phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test
source: [176-VERIFICATION.md]
started: 2026-05-21T00:00:00Z
updated: 2026-05-21T00:00:00Z
---

## Current Test

[awaiting human testing via gestiscilo-it consumer chrome-mcp re-UAT against patched local-path ferro dependency]

## Tests

### 1. Bug R2 — Card.badge visual closure

expected: Consumer kanban card for a `pending_email` booking renders a Badge-styled pill containing the countdown text ("Scade tra Nm"). The pill appears co-planar with the card title (right-aligned via flex justify-between), uses Secondary chrome (`bg-secondary/10 text-secondary-foreground`), and the title text remains visible and correctly truncated when the badge is present.
result: [pending]

### 2. Bug R3 — Card.subtitle visual closure

expected: Consumer kanban card rendering a booking with a `staff_name_snapshot` value emits a muted-text secondary line ("Marco Rossi" or equivalent) between the title and the description. Vertical spacing matches the documented `mt-0.5` (4px) tightness between title and subtitle, then `mt-1` (8px) between subtitle and description.
result: [pending]

### 3. Bug R4 — Grid chip strip visibility closure

expected: Consumer per-staff filter chip strip renders the Grid + all four chips (Tutti / Marco / Giulia / Senza staff) when `data.has_staff = true`; the same view with `data.has_staff = false` emits no Grid element at all (no empty wrapper, no spacer, no comment marker — completely absent from the DOM). Verifies F9 closure end-to-end through the consumer's controller emission and ferro's renderer.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
