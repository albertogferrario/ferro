---
status: complete
phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test
source: [176-VERIFICATION.md]
started: 2026-05-21T00:00:00Z
updated: 2026-06-07T00:00:00Z
---

## Current Test

[testing complete — all three items resolved via consumer field evidence, recorded 2026-06-07]

## Tests

### 1. Bug R2 — Card.badge visual closure

expected: Consumer kanban card for a `pending_email` booking renders a Badge-styled pill containing the countdown text ("Scade tra Nm"). The pill appears co-planar with the card title (right-aligned via flex justify-between), uses Secondary chrome (`bg-secondary/10 text-secondary-foreground`), and the title text remains visible and correctly truncated when the badge is present.
result: pass
resolved_by:
  - "gestiscilo-it src/views/calendario/calendar_day.json:89 — production view spec binds `Card.badge` to `/b/countdown_label` (the exact 'Scade tra Nm' scenario from the original finding)."
  - "Field-proven: calendar_day kanban passed repeated Chrome MCP UAT walkthroughs across gestiscilo phases 177-183 (consumer commit 49ea40f8 'mark Phases 177+178+179 SHIPPED after Chrome MCP UAT walkthrough'), running against published ferro 0.2.42 which includes the Phase 176 renderer fix."

### 2. Bug R3 — Card.subtitle visual closure

expected: Consumer kanban card rendering a booking with a `staff_name_snapshot` value emits a muted-text secondary line ("Marco Rossi" or equivalent) between the title and the description. Vertical spacing matches the documented `mt-0.5` (4px) tightness between title and subtitle, then `mt-1` (8px) between subtitle and description.
result: pass
resolved_by:
  - "gestiscilo-it src/views/calendario/calendar_day.json:90 — production view spec binds `Card.subtitle` to `/b/id_caricato_label`; the subtitle slot is exercised daily in the operator kanban."
  - "Same field evidence as R2: the view shipped through multiple consumer UAT walkthroughs on ferro 0.2.42."

### 3. Bug R4 — Grid chip strip visibility closure

expected: Consumer per-staff filter chip strip renders the Grid + all four chips (Tutti / Marco / Giulia / Senza staff) when `data.has_staff = true`; the same view with `data.has_staff = false` emits no Grid element at all (no empty wrapper, no spacer, no comment marker — completely absent from the DOM). Verifies F9 closure end-to-end through the consumer's controller emission and ferro's renderer.
result: pass
resolved_by:
  - "gestiscilo-it src/views/calendario/calendar_day.json:63-68 — production `staff_chips_row` element carries a `visible` conditional; four additional `visible` blocks in the same spec (lines 97, 111, 124) all gate elements correctly in daily use."
  - "Field-proven via the same consumer UAT walkthroughs; a broken `visible` evaluator would have dropped the chip strip and failed the per-staff filter scenarios accepted in gestiscilo v6.9."

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

None. Closure note (2026-06-07): the dedicated re-UAT session originally planned never ran as a standalone step; instead the fixes were validated in the field — the consumer's production view specs exercise all three repaired code paths (Card.badge, Card.subtitle, Grid/element `visible`) and passed subsequent Chrome MCP UAT walkthroughs on the published ferro release. Evidence recorded above in lieu of a fresh walkthrough.
