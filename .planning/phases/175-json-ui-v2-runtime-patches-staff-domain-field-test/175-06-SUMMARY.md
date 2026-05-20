---
phase: 175-json-ui-v2-runtime-patches-staff-domain-field-test
plan: "06"
subsystem: ferro-json-ui
tags: [json-ui, forms, file-upload, multipart, enctype, security]
dependency_graph:
  requires: []
  provides:
    - InputType::File variant with serde wire format "file"
    - InputProps.accept field for browser-side MIME filter hint
    - FormProps.enctype field for multipart encoding
    - render_input File arm emitting <input type="file"> without value=""
    - render_form enctype attribute emission on opening <form> tag
  affects:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-json-ui/src/projection/component_map.rs
tech_stack:
  added: []
  patterns:
    - TDD RED/GREEN sequence for two coupled emitter changes shipped in one plan (D-F5)
    - Negative test assertion for security invariant (no value="" on file inputs)
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-json-ui/src/projection/component_map.rs
decisions:
  - D-F5 honored: InputType::File rendering and FormProps.enctype propagation land in the same plan with a single end-to-end test
  - File inputs do not emit value="" — browser security restriction pinned by negative assertion in test
  - accept attribute is advisory only — server-side MIME validation is the consumer's responsibility (documented in field doccomment and SUMMARY threat section)
metrics:
  duration: "~12 minutes"
  completed: "2026-05-20"
  tasks: 2
  files: 4
---

# Phase 175 Plan 06: File Input + Multipart Form Encoding Summary

One-liner: `InputType::File` variant + `accept` prop + `FormProps.enctype` field ship together with end-to-end test proving the avatar-upload form produces a multipart-capable HTML form.

## What Was Built

F5 in the phase research identified two coupled runtime gaps — neither was useful alone:

- **F5a**: `InputType` had no `File` variant. A spec with `"input_type": "file"` silently fell back to `Text` via serde's unknown-variant behavior, producing `<input type="text">`.
- **F5b**: `FormProps` had no `enctype` field. A spec with `"enctype": "multipart/form-data"` silently dropped the attribute; the form rendered with browser-default `application/x-www-form-urlencoded` encoding, making file uploads impossible.

Per D-F5, both gaps land in the same plan. Three tests pin the behavior:

1. `input_file_renders_file_type_and_accept` — file input emits `type="file"`, the `accept` attribute, and does NOT emit `value=""`.
2. `form_enctype_emitted_when_set` — `enctype` appears on the opening `<form>` tag when set; absent when not set.
3. `multipart_form_roundtrip` — end-to-end: Form with `enctype="multipart/form-data"` + `Input[file]` + `Input[text]` + Button renders a complete multipart-capable form.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 (RED) | `29926b33` | test(175-06): add red-state tests for InputType::File + FormProps.enctype |
| 2 (GREEN) | `5662f318` | feat(175-06): add InputType::File + accept prop + FormProps.enctype emission |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Struct literal initializations in projection pipeline**

- **Found during:** Task 2 (clippy pass after implementation)
- **Issue:** `FormProps` and `InputProps` are constructed as struct literals in `projection/builder.rs` and `projection/component_map.rs`. Adding new fields to the structs caused compile errors (`E0063: missing field`) in those sites.
- **Fix:** Added `enctype: None` to the `FormProps` literal in `builder.rs`; added `accept: None` to the `InputProps` literal in `component_map.rs`. Both default to `None` — the projection pipeline does not auto-derive file input or multipart encoding; consumers configure those via spec authoring.
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`, `ferro-json-ui/src/projection/component_map.rs`
- **Commit:** `5662f318` (included in implementation commit)

**2. [Rule 2 - Formatting] rustfmt reformat of test code**

- **Found during:** Task 1 post-edit fmt check
- **Issue:** Method chain in `input_file_renders_file_type_and_accept` test and a `find('>')` call in `form_enctype_emitted_when_set` did not match rustfmt's expected line-break points.
- **Fix:** `cargo fmt --all` applied; no semantic change.
- **Commit:** `5662f318` (formatting folded into implementation commit)

## Security Notes

**T-175-06-01 / T-175-06-05 (accept attribute):** The `accept` attribute on file inputs is a client-side advisory hint. Browsers use it to pre-filter the file picker UI, but they do not enforce the constraint — any file can be submitted regardless of `accept`. The field doccomment in `InputProps` documents this boundary explicitly. **Consumers must validate uploaded file MIME types server-side** at the controller layer.

**T-175-06-04 (no value="" on file inputs):** The `InputType::File` render arm intentionally omits the `value` attribute. Browser security policy prevents JavaScript from reading or pre-setting file input values. Emitting `value=""` would either be silently ignored or trigger a browser console error. The omission is pinned by the negative assertion in `input_file_renders_file_type_and_accept`.

**T-175-06-03 (CSRF on multipart forms):** The `enctype` attribute changes how the browser encodes the body but does not affect CSRF protection. Multipart-encoded form endpoints remain subject to the framework's CSRF middleware. No change to CSRF behavior in this plan.

## Known Stubs

None. All new fields are fully wired: `InputType::File` renders `<input type="file">`, `accept` emits the attribute, and `enctype` propagates to the `<form>` opening tag.

## Threat Flags

No new security surface introduced beyond what is documented in the plan's threat model and the Security Notes section above. All identified threats have `mitigate` or `accept` disposition and are handled as documented.

## Self-Check

### Files exist
- `ferro-json-ui/src/component.rs` — FOUND (contains `File,` variant, `accept: Option<String>`, `enctype: Option<String>`)
- `ferro-json-ui/src/render/form.rs` — FOUND (contains `InputType::File =>` arm, `enctype_attr`, `type=\"file\"`)
- `ferro-json-ui/src/projection/builder.rs` — FOUND (contains `enctype: None`)
- `ferro-json-ui/src/projection/component_map.rs` — FOUND (contains `accept: None`)

### Commits exist
- `29926b33` — FOUND
- `5662f318` — FOUND

### Tests
- `cargo test -p ferro-json-ui` — 531 passed, 0 failed

## Self-Check: PASSED
