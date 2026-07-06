---
phase: 181
plan: 08
status: complete
wave: 7
subsystem: docs
tags: [docs, json-ui, form-validation, d-09]
commits:
  - 7cf5ad6c
  - 5f05f037
dependency_graph:
  requires: [181-02]
  provides: [docs/src/json-ui/forms.md]
  affects: [docs/src/SUMMARY.md]
tech_stack:
  added: []
  patterns: [mdbook]
key_files:
  created:
    - docs/src/json-ui/forms.md
  modified:
    - docs/src/SUMMARY.md
key-decisions:
  - "redirect_back takes Option<&str> referer, not &req — plan template used wrong signature; corrected in docs to req.header(\"referer\")"
  - "ValidationError::add takes &mut self — docs use Validator::new pattern rather than chained add calls to match actual API shape"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-31T22:55:00Z"
  tasks: 3
  files: 2
---

# Phase 181 Plan 08: Form Validation Documentation Summary

New `docs/src/json-ui/forms.md` page covering all four CONTEXT D-09 authoring patterns for JSON-UI form validation, with accurate API signatures verified against current source.

## What Was Built

### Task 1: docs/src/json-ui/forms.md (commit `7cf5ad6c`)

Created `docs/src/json-ui/forms.md` (146 lines, 4 H2 sections) covering:

1. **Blessed Path: `JsonUi::render_validation_error`** — GET handler calls `render_validation_error(&spec, &data, &ve)` to plumb error messages onto fields automatically by matching the `field` prop. POST handler calls `validator.validate()` on failure, chains `.with_old_input(&data).redirect_back(req.header("referer"))`.

2. **Escape Hatch: Manual `$data` Binding** — handler inserts `"<field>_error"` into the data map with `req.validation_error("field")` value, spec references it via `.prop("error", json!({"$data": "/<field>_error"}))`. Use when the validation key does not match the form-control `field` prop, for cross-field display, or composite keys.

3. **Flash Round-Trip on POST → GET** — `with_old_input(&data).redirect_back(req.header("referer"))` writes `_flash.new.*`; session middleware promotes to `_flash.old.*` on next request; GET handler reads `req.old("field")` for `default_value` (submission restore) and `req.validation_error("field")` for the error message.

4. **Cross-Field Validation Summary** — `if req.has_validation_errors() { ... }` conditional renders an `Alert` banner before the form fields. Documents that `has_validation_errors()` and `validation_error("field")` read the same session key and cannot disagree within one handler invocation.

Voice: neutral and instructional. No version labels (`v1`/`v2`/`legacy`). No marketing language.

### Task 2: docs/src/SUMMARY.md (commit `5f05f037`)

Inserted `- [Form Validation](json-ui/forms.md)` at line 58, between "Data Binding & Visibility" and "Layouts". No other lines changed.

### Task 3: mdbook smoke test (no commit — verification only)

`mdbook build` exited 0. `docs/book/json-ui/forms.html` confirmed present at
`/Users/alberto/repositories/albertogferrario/ferro/docs/book/json-ui/forms.html`.

## D-09 Section Inventory

| H2 Section | CONTEXT D-09 Pattern |
|---|---|
| Blessed Path: `JsonUi::render_validation_error` | Pattern 1 — blessed `render_validation_error` path |
| Escape Hatch: Manual `$data` Binding | Pattern 2 — manual `$data` escape hatch |
| Flash Round-Trip on POST → GET | Pattern 3 — `with_old_input` + `redirect_back` + `req.old` round-trip |
| Cross-Field Validation Summary | Pattern 4 — `has_validation_errors` conditional banner |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Incorrect `redirect_back` signature in plan template**
- **Found during:** Task 1 (API verification)
- **Issue:** The plan's code template showed `.redirect_back(&req)` — `redirect_back` actually takes `Option<&str>` (the Referer header value), not a `&Request`. Signature confirmed at `framework/src/validation/error.rs:116`.
- **Fix:** Used `req.header("referer")` in the docs examples, matching the actual API.
- **Files modified:** `docs/src/json-ui/forms.md`
- **Commit:** `7cf5ad6c`

## Known Stubs

None. Documentation only — no runtime stubs.

## Threat Flags

No new attack surface. Documentation only, using synthetic field names (`email`, `overage_threshold`). No secrets, no PII.

## Self-Check: PASSED

- `docs/src/json-ui/forms.md`: FOUND (`ls -la` confirmed, 6776 bytes)
- `grep -c '^## ' docs/src/json-ui/forms.md` → 4 (≥ 4 required)
- `grep -cE 'render_validation_error|\$data|with_old_input|has_validation_errors'` → 15 (≥ 4 required)
- `grep -ciE '\b(v1|v2|legacy|new form validation)\b'` → 0
- `grep -n 'json-ui/forms.md' docs/src/SUMMARY.md` → line 58, exactly 1 match
- Position check (`grep -B1 -A1 'json-ui/forms.md'`): between "Data Binding & Visibility" and "Layouts" ✓
- Commit `7cf5ad6c`: verified via git log
- Commit `5f05f037`: verified via git log
- `mdbook build` exit 0: confirmed
- `docs/book/json-ui/forms.html`: FOUND
