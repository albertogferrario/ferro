---
status: partial
phase: 181-json-ui-input-error-prop-inline-render
source: [181-VERIFICATION.md, 181-07-AUDIT.md]
started: 2026-05-31T23:35:00Z
updated: 2026-05-31T23:35:00Z
---

## Current Test

[awaiting human testing — repoint gestiscilo to local ferro path-dep, walk 5 representative forms]

## Tests

### 1. cassa/products — operator product-edit form (discovery surface)
expected: Submit `/dashboard/cassa/prodotti/{id}/modifica` with `overage_threshold=2` and `overage_price` empty. An inline `<p id="err-overage_price">` (and/or `err-overage_threshold`) renders below the "Soglia sovrapprezzo" field with the cross-field validation message "Per il sovrapprezzo, compila sia la soglia che il prezzo". The input carries `aria-invalid="true"`. Form value pre-fills via `req.old(...)`.
result: [pending]

### 2. calendario/bookings — any booking new/edit form
expected: Submit empty. Inline `<p id="err-{field}">` renders below each required field with `aria-invalid="true"`. Form pre-fills via `req.old(...)`.
result: [pending]

### 3. settings — staff or general settings form
expected: Submit with invalid input. Inline `<p>` renders below each offending field.
result: [pending]

### 4. staff — staff member create/edit form
expected: Submit with duplicate email. Inline `<p id="err-email">` renders below the email field with `aria-invalid="true"`.
result: [pending]

### 5. documenti — document upload form
expected: Submit empty. Inline `<p>` renders below the file input. The `<input type="file">` carries a `ring-destructive` ring (Plan 06 D-06 parity) AND `aria-invalid="true"`.
result: [pending]

### 6. D-05 cross-field summary (orthogonal check)
expected: For each of the 5 forms above, also confirm whether the top-of-page validation toast (`toast_validation`) renders when the handler adds it to `root_children`. Per RESEARCH §Suspect 3, `has_validation_errors()` and `validation_error()` now read the same key — the cross-field symptom should auto-resolve at the ferro layer. If the toast does NOT render despite per-field errors rendering, document the consumer-side handler that fails to add `toast_validation` (consumer-side bug, not ferro bug).
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps

[No gaps recorded yet — populated as operator walks the forms]

## How to Run

```bash
# Repoint gestiscilo to local ferro path-dep
cd ../gestiscilo-it
# Edit Cargo.toml: ferro = { path = "../ferro" }  (or similar workspace deps)
cargo build
# Start gestiscilo dev server
ferro serve  # or the gestiscilo-specific command
```

Then walk forms 1-5 in a browser at the appropriate URLs. Record PASS/FAIL findings inline in `181-07-AUDIT.md § Manual UAT — Representative Sample` table. Update this file's `## Tests` results when complete. Run `/gsd-verify-work 181` to re-verify after browser testing is done.

## Release Gate

Per `feedback_friction_loop_release_cadence.md`: ferro Phase 181 must NOT be published to crates.io until this UAT is complete. The friction-loop convention is one publish at the end of the loop after gestiscilo consumes the fix.
