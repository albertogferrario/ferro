---
phase: 181-json-ui-input-error-prop-inline-render
verified: 2026-05-31T23:30:00Z
status: human_needed
score: 11/12
overrides_applied: 0
human_verification:
  - test: "Walk 5 gestiscilo forms with inline error trigger"
    expected: "Inline <p id=\"err-{field}\" class=\"text-sm text-destructive\">{msg}</p> renders below each offending field; aria-invalid present; form values pre-fill via req.old(); optional toast banner appears when handler adds toast_validation to root_children"
    why_human: "Requires repointing gestiscilo's ferro dep to local path, starting the gestiscilo dev server, and confirming rendered DOM in a browser. Cannot be exercised from automated grep/file checks."
---

# Phase 181: json-ui-input-error-prop-inline-render Verification Report

**Phase Goal:** Fix the JSON-UI resolution pipeline so form-control error messages bound via `{"$data": "/<field>_error"}` or via `JsonUi::render_validation_error` actually reach `props.error` and render as the locked DOM shape `<p id="err-{field}" class="text-sm text-destructive">{error}</p>` below the offending field.
**Verified:** 2026-05-31T23:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Step 0 — Previous Verification

No previous VERIFICATION.md found. Initial mode.

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `JsonUi::render` merges runtime data into spec.data clone before resolve so `$data` bindings resolve against handler-supplied data (Fix A — D-02 root cause 1) | VERIFIED | `spec.clone().merge_data(data.clone())` at `framework/src/json_ui/mod.rs:84` (render_with_config) and `:268` (render_with_errors_config). Two matches exactly as required. |
| 2 | `attach_errors` writes singular `error: String` (first message wins) matching `InputProps.error: Option<String>` shape (Fix B — D-02 root cause 2) | VERIFIED | `props_obj.insert("error".to_string(), Value::String(first.clone()))` at `ferro-json-ui/src/resolve.rs:192`. Exactly one match for the per-field branch. `msgs.first()` confirmed at line 191. The `else if all` branch still writes `errors` (plural) unchanged. |
| 3 | Both authoring paths — manual `$data` binding AND `JsonUi::render_validation_error` — produce `<p id="err-{field}" class="text-sm text-destructive">{msg}</p>` in the rendered HTML (D-04 parity) | VERIFIED | Four pipeline-level integration tests in `framework/src/json_ui/mod.rs` (lines 811, 884, 915, 945) assert the locked DOM shape via `html_body`. All four transitioned RED→GREEN after Plans 01-02. 2812 tests passed, 0 failed in phase gate. |
| 4 | `Checkbox` error state emits `border-destructive`, `focus-visible:ring-destructive`, `aria-invalid="true"`, `aria-describedby="err-{field}"` on the `<input>` and `id="err-{field}"` on the error `<p>` (D-06) | VERIFIED | `render_checkbox` in `ferro-json-ui/src/render/form.rs:459-501` contains `has_error`, `border_class`, `focus_ring_class` locals and the ARIA block before `html.push('>')`. Error `<p>` at line 517-522 includes `id=\"err-{}\"`. Test `checkbox_error_renders_destructive_class_and_aria` at line 920 passes. |
| 5 | `CheckboxList` error state emits ARIA on `<fieldset>` (not per-input) and `border-destructive` on each option `<input>` and `id="err-{field}"` on the error `<p>` (D-06) | VERIFIED | `render_checkbox_list` at `ferro-json-ui/src/render/form.rs:577-635` emits conditional `aria-invalid`/`aria-describedby` on `<fieldset>` open tag, uses `checkbox_border` interpolated into per-option `<input>` class string, and error `<p>` at line 629-634 includes `id=\"err-{}\"`. Test `checkbox_list_error_renders_fieldset_aria` at line 969 passes and verifies aria_count==1 (fieldset only). |
| 6 | `Switch` error state emits `peer-focus:ring-destructive/30` on the pill `<div>`, `aria-invalid="true"` on the hidden `<input>`, and `id="err-{field}"` on the error `<p>` (D-06) | VERIFIED | `render_switch` at `ferro-json-ui/src/render/form.rs:659-774` declares `peer_ring_class` conditional, adds ARIA block to hidden `<input>` at line 755-759, emits `{peer_ring_class}` into the pill `<div>` class at line 763, and error `<p>` at line 768-773 includes `id=\"err-{}\"`. Test `switch_error_renders_destructive_ring_and_aria` at line 1044 asserts both presence (destructive) and absence (primary) of the ring class. |
| 7 | `Input(file)` error state emits `ring-1 ring-destructive` in the class string and `aria-invalid="true"` on the `<input type="file">` (D-06) | VERIFIED | `InputType::File` arm in `render_input` at `ferro-json-ui/src/render/form.rs:221-248` declares `file_ring_class` conditional and appends ARIA block before `html.push('>')`. Test `input_file_error_renders_destructive_ring_and_aria` at line 1098 passes with Pitfall-3 isolation guard. |
| 8 | D-08 cross-repo audit confirms no gestiscilo consumer reads the pre-fix `errors: Vec<String>` plural shape from manually constructed specs | VERIFIED | `181-07-AUDIT.md` Bucket B is explicitly empty. `rg '\.errors' gestiscilo-it/app/src/` returned zero hits. All three `"errors"` literal hits in Bucket A are runtime data objects (not props reads). D-08 closed cleanly with no gestiscilo PR required. |
| 9 | The full pre-commit gate (`cargo fmt + cargo clippy --all-targets -D warnings + cargo test --all-features`) passes on the ferro workspace | VERIFIED | `181-07-AUDIT.md` § Phase Gate confirms all three steps exit 0. Test summary: 2812 passed, 0 failed, 437 ignored. Two formatting/lint issues discovered during the gate (Wave 2 fmt omissions + uninlined_format_args in render_switch) were fixed at root before the gate was declared clean. |
| 10 | `docs/src/json-ui/forms.md` exists and covers all four CONTEXT D-09 authoring patterns (blessed path, escape hatch, flash round-trip, cross-field summary) | VERIFIED | File exists at 146 lines, 4 H2 sections. `grep -c '^## '` returns 4. Pattern coverage confirmed: `render_validation_error` appears 15 times, `$data` appears 15 times, `with_old_input` appears 15 times, `has_validation_errors` appears 15 times. No `v1`/`v2`/`legacy` labels (count 0). |
| 11 | `docs/src/SUMMARY.md` links the new forms.md page between "Data Binding & Visibility" and "Layouts" | VERIFIED | `grep -n 'json-ui/forms.md' docs/src/SUMMARY.md` returns line 58. Surrounding lines confirm position between `data-binding.md` and `layouts.md`. |
| 12 | Inline error `<p>` renders correctly in the gestiscilo browser UI for the ~15 escape-hatch bindings in cassa/products and the settings escape-hatch pattern | HUMAN NEEDED | Requires gestiscilo path-dep repoint + dev server + browser devtools inspection. The 181-07-AUDIT.md documents this as "DEFERRED-TO-OPERATOR / release-time gate". Automated checks confirm the pipeline is correct but browser rendering cannot be verified programmatically. |

**Score:** 11/12 truths verified (1 requires human testing)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `framework/src/json_ui/mod.rs` | Fix A (merge_data at 2 sites) + 4 pipeline tests | VERIFIED | Lines 84, 268 contain `spec.clone().merge_data(data.clone())`. Tests at lines 811, 884, 915, 945 all use `html_body` and assert on `<p id="err-` shape. |
| `ferro-json-ui/src/resolve.rs` | Fix B (singular `error: String` per-field) + 2 updated tests | VERIFIED | Line 192: `props_obj.insert("error".to_string(), ...)`. Line 191: `msgs.first()`. `else if all` branch unchanged at line 197 (`"errors"` plural). |
| `ferro-json-ui/src/render/form.rs` | D-06 parity for Checkbox, CheckboxList, Switch, Input(file) + 4 new tests | VERIFIED | All four renderers confirmed with has_error gates, class swaps, ARIA blocks, and `id` on error `<p>`. Tests at lines 920, 969, 1044, 1098. |
| `.planning/phases/181-json-ui-input-error-prop-inline-render/181-07-AUDIT.md` | D-08 audit results + UAT evidence + phase gate | VERIFIED | File exists with Bucket A/B/C sections, Manual UAT section (DEFERRED-TO-OPERATOR status documented), and Phase Gate section showing 2812/0. |
| `docs/src/json-ui/forms.md` | Form validation docs covering 4 D-09 patterns | VERIFIED | 146 lines, 4 H2 sections, all four patterns covered, no version labels. |
| `docs/src/SUMMARY.md` | Navigation entry for forms.md | VERIFIED | Line 58, between data-binding.md and layouts.md. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `render_with_config` | `Spec::merge_data` | `spec.clone().merge_data(data.clone())` before `Self::resolve` | WIRED | Confirmed at mod.rs:84 |
| `render_with_errors_config` | `Spec::merge_data` | `spec.clone().merge_data(data.clone())` before `Self::resolve_with_errors` | WIRED | Confirmed at mod.rs:268 |
| `attach_errors` per-field branch | `InputProps.error: Option<String>` | `props_obj.insert("error", Value::String(first.clone()))` | WIRED | Confirmed at resolve.rs:192; shape matches serde field exactly |
| `render_checkbox` | `props.error.is_some()` | `has_error` gate | WIRED | Confirmed at form.rs:459 |
| `render_checkbox_list` | `props.error.is_some()` | `has_error` gate + fieldset ARIA | WIRED | Confirmed at form.rs:577 |
| `render_switch` | `props.error.is_some()` | `peer_ring_class` + hidden-input ARIA | WIRED | Confirmed at form.rs:659 |
| `InputType::File` branch | `has_error` (function-scope) | `file_ring_class` + ARIA block | WIRED | Confirmed at form.rs:222 |
| `docs/src/SUMMARY.md` | `docs/src/json-ui/forms.md` | mdbook navigation entry | WIRED | Line 58; mdbook build exits 0 per 181-08-SUMMARY.md |

---

### Data-Flow Trace (Level 4)

The core data flow is handler-supplied `serde_json::Value` → `merge_data` → `resolve_expressions` → renderer prop deserialization → HTML emission.

| Stage | Artifact | Data Variable | Source | Produces Real Data | Status |
|-------|----------|---------------|--------|--------------------|--------|
| Fix A merge | `render_with_config` | `spec_with_data` | `spec.clone().merge_data(data.clone())` | Yes — handler data wins on collision | FLOWING |
| Fix B attach | `attach_errors::per-field` | `props_obj["error"]` | `msgs.first()` from `errors: &HashMap<String, Vec<String>>` | Yes — real validation messages | FLOWING |
| D-06 class | `render_checkbox` | `border_class`, `focus_ring_class` | `props.error.is_some()` | Yes — conditional on real prop | FLOWING |
| D-06 error `<p>` | All four renderers | `props.error` | `Option<String>` deserialized from props_obj | Yes — set by Fix B or direct prop | FLOWING |

---

### Behavioral Spot-Checks

Cannot re-run cargo test per `feedback_one_cpu_op_at_a_time.md`. The phase gate evidence from 181-07-AUDIT.md is the canonical test run.

| Behavior | Evidence | Status |
|----------|----------|--------|
| `pipeline_data_binding_error_prop_renders_p_tag` (D-07a) passes | 181-02-SUMMARY.md GREEN evidence; phase gate 2812/0 | PASS (by evidence) |
| `pipeline_render_validation_error_renders_p_tag` (D-07b) passes | 181-02-SUMMARY.md GREEN evidence; phase gate 2812/0 | PASS (by evidence) |
| `checkbox_error_renders_destructive_class_and_aria` passes | 181-03-SUMMARY.md; phase gate 2812/0 | PASS (by evidence) |
| `checkbox_list_error_renders_fieldset_aria` passes | Phase gate 2812/0 | PASS (by evidence) |
| `switch_error_renders_destructive_ring_and_aria` passes | Phase gate 2812/0 | PASS (by evidence) |
| `input_file_error_renders_destructive_ring_and_aria` passes | Phase gate 2812/0 | PASS (by evidence) |
| Full pre-commit gate | 181-07-AUDIT.md § Phase Gate: `cargo fmt` + `cargo clippy -D warnings` + `cargo test --all-features` all exit 0 | PASS |

---

### Requirements Coverage

D-01..D-09 from CONTEXT.md:

| Decision | Description | Status | Evidence |
|----------|-------------|--------|----------|
| D-01 | Diagnosis premise re-verified before any code change | SATISFIED | 181-RESEARCH.md + 181-01-SUMMARY.md RED-state evidence confirms pipeline (not renderer) was the fault layer |
| D-02 | Both root causes investigated and fixed (Fix A + Fix B) | SATISFIED | Fix A at mod.rs:84+268, Fix B at resolve.rs:192; both confirmed in source |
| D-03 | Fix at the pipeline layer, not at the renderer (no surface shim) | SATISFIED | No renderer-side compatibility branch for `errors` plural was added; 181-02-SUMMARY.md explicitly notes "No backward-compat shim added" |
| D-04 | Both authoring paths work end-to-end | SATISFIED | D-07a (manual $data) and D-07b (render_validation_error) both pass; parity confirmed |
| D-05 | `has_validation_errors`/`toast_validation` cross-field symptom verified | PARTIALLY SATISFIED | RESEARCH §Suspect 3 analysis concluded no ferro bug exists (both readers hit same key); browser UAT for the cross-field toast banner is part of the operator verification gate |
| D-06 | Error-state class+ARIA parity for Checkbox, CheckboxList, Switch, Input(file) | SATISFIED | All four renderers confirmed in source + 4 unit tests passing |
| D-07 | Integration tests at JsonUi pipeline level (not just renderer isolation) | SATISFIED | Two new pipeline tests (D-07a, D-07b) + two upgraded existing tests; all use `html_body` and assert locked DOM shape |
| D-08 | Clean break on `errors`→`error` reconciliation; cross-repo audit | SATISFIED | Bucket B empty; no gestiscilo consumer reads plural shape; 181-07-AUDIT.md verdict "closed cleanly" |
| D-09 | Docs page covers all four authoring patterns | SATISFIED | `docs/src/json-ui/forms.md` at 146 lines, 4 H2 sections, all patterns covered, no version labels |

---

### Anti-Patterns Found

Scanned files from phase: `framework/src/json_ui/mod.rs`, `ferro-json-ui/src/resolve.rs`, `ferro-json-ui/src/render/form.rs`, `docs/src/json-ui/forms.md`.

| File | Issue | Severity | Impact |
|------|-------|----------|--------|
| `framework/src/json_ui/mod.rs:214-222` and `:277-289` | `render_json` and `render_json_with_errors` do not call `merge_data` before resolving expressions — asymmetry with HTML paths fixed by Fix A. `$data` bindings that reference handler-supplied data will resolve to null on these JSON paths. | Warning | Latent bug; no current gestiscilo consumer uses these paths with `$data` error bindings per D-08 audit. Documented in 181-REVIEW.md as WR-01. |
| `docs/src/json-ui/forms.md:28-33` | GET handler example passes `ValidationError::default()` (always empty) to `render_validation_error`, and prose incorrectly implies the framework reads session flash automatically. The handler must retrieve errors from the session and pass them explicitly. | Info | Reader confusion about API contract; no runtime impact. Documented in 181-REVIEW.md as IN-01. |
| `ferro-json-ui/src/render/form.rs:594` | `CheckboxList` description `<p>` emits `text-muted-foreground` while every other renderer uses `text-text-muted`. Pre-existing before Phase 181. | Info | Visual inconsistency depending on theme token mapping; not introduced by this phase. Documented in 181-REVIEW.md as IN-02. |

No blockers that prevent the phase goal. The WR-01 warning is a known follow-up (explicitly documented in 181-02-SUMMARY.md as deferred "render_json asymmetry").

---

### Human Verification Required

#### 1. Gestiscilo Browser UAT — Inline Error Rendering

**Test:** Repoint gestiscilo-it's ferro dependency to the local ferro tree (`ferro = { path = "..." }` in the workspace Cargo.toml), build gestiscilo, start the dev server, and walk the 5 representative forms documented in `181-07-AUDIT.md § Manual UAT — Representative Sample`:

1. `/dashboard/cassa/prodotti/{id}/modifica` — submit with `overage_threshold=2`, `overage_price` empty. Expected: `<p>` below "Soglia sovrapprezzo" with the error text.
2. Any `calendario/bookings` new/edit form — submit empty. Expected: inline `<p>` below each required field.
3. `settings` general form — submit invalid. Expected: inline `<p>` below invalid field.
4. Staff member create/edit form — submit with duplicate email. Expected: inline `<p>` below email field.
5. Document upload form — submit empty. Expected: inline `<p>` below file input AND destructive ring on `<input type="file">` (Plan 06 verification).

For each form also confirm:
- `aria-invalid="true"` on the offending input (browser devtools).
- Top-of-page validation toast renders when handler adds `toast_validation` to `root_children` (D-05 check).
- Form values pre-fill correctly via `req.old(...)`.

**Expected:** All 5 forms render the inline error `<p>` with the locked DOM shape. `aria-invalid="true"` present. Values pre-fill. D-05 toast either renders (ferro layer correct) or the consumer-side handler omitting `toast_validation` from `root_children` is identified as the culprit.

**Why human:** Requires browser + local gestiscilo dev server + path-dep repoint. Cannot be verified programmatically. Per `feedback_friction_loop_release_cadence.md`, ferro must not publish Phase 181 until this UAT is complete and documented in `181-07-AUDIT.md`.

---

### Gaps Summary

No code-level gaps blocking the phase goal. All 11 programmatically verifiable truths pass.

The one open item (Truth 12) is human verification of browser rendering in gestiscilo — a deliberate release-time gate per `feedback_friction_loop_release_cadence.md`, not an implementation gap. The pipeline is correct; the question is whether the correct HTML materializes in an actual browser session.

The WR-01 warning (`render_json` asymmetry) is a known follow-up deferred from Plan 02, with a concrete fix documented in 181-REVIEW.md. It does not block the phase goal (no current consumer uses `render_json` with `$data`-driven error bindings).

---

*Verified: 2026-05-31T23:30:00Z*
*Verifier: Claude (gsd-verifier)*
