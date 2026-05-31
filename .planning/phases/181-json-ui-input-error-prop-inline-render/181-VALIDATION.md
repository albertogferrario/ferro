---
phase: 181
slug: json-ui-input-error-prop-inline-render
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-31
---

# Phase 181 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 2021 edition, workspace tests) |
| **Config file** | `Cargo.toml` (workspace root) + `framework/Cargo.toml` + `ferro-json-ui/Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-json-ui --lib && cargo test -p framework --lib json_ui::tests` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | Quick: ~30–60s · Full: ~3–5 min on cold cache, ~60–90s warm |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --lib && cargo test -p framework --lib json_ui::tests`
- **After every plan wave:** Run `cargo test -p ferro-json-ui && cargo test -p framework` (lib + integration)
- **Before `/gsd-verify-work`:** Full suite must be green — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Max feedback latency:** 90 seconds (warm cache)

Per memory `feedback_one_cpu_op_at_a_time.md`: serialize CPU-intensive operations. Do NOT chain or parallelize cargo/clippy/test runs across tasks. Reuse prior step's test evidence rather than re-running the full suite per task.

---

## Per-Task Verification Map

Task IDs follow the wave structure from `181-RESEARCH.md §Recommended Task Structure`. REQ-IDs are not assigned (phase has no formal REQ-ID mapping per ROADMAP.md — Requirements field is "TBD"). Threat refs are not applicable (no new security surface per RESEARCH §Security Domain).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 181-W0-01 | 01 | 0 | — | — | N/A | unit | `cargo test -p framework --lib json_ui::tests::render_with_errors_populates_form_fields -- --exact` (must FAIL before fix; PASS after) | ✅ existing test, upgrade assertion | ⬜ pending |
| 181-W0-02 | 01 | 0 | — | — | N/A | unit | `cargo test -p framework --lib json_ui::tests::render_validation_error_accepts_framework_type -- --exact` (must FAIL before fix; PASS after) | ✅ existing test, upgrade assertion | ⬜ pending |
| 181-W0-03 | 01 | 0 | — | — | N/A | integration | `cargo test -p framework --lib json_ui::tests::pipeline_data_binding_error_prop_renders_p_tag -- --exact` (D-07 test 1 — new) | ❌ Wave 0 creates | ⬜ pending |
| 181-W0-04 | 01 | 0 | — | — | N/A | integration | `cargo test -p framework --lib json_ui::tests::pipeline_render_validation_error_renders_p_tag -- --exact` (D-07 test 2 — new) | ❌ Wave 0 creates | ⬜ pending |
| 181-01-01 | 02 | 1 | — | — | N/A | unit | `cargo test -p framework --lib json_ui::tests::pipeline_data_binding_error_prop_renders_p_tag -- --exact` (passes after Fix A) | ✅ from W0-03 | ⬜ pending |
| 181-01-02 | 02 | 1 | — | — | N/A | unit | `cargo test -p ferro-json-ui --lib resolve::tests` (3 existing tests at resolve.rs:785-833 updated to assert on `"error"` singular) | ✅ existing tests, update assertions | ⬜ pending |
| 181-01-03 | 02 | 1 | — | — | N/A | unit | `cargo test -p framework --lib json_ui::tests::pipeline_render_validation_error_renders_p_tag -- --exact` (passes after Fix B) | ✅ from W0-04 | ⬜ pending |
| 181-02-01 | 03 | 2 | — | — | A11y: aria-invalid + aria-describedby on checkbox; border-destructive swap | unit | `cargo test -p ferro-json-ui --lib render::form::tests::checkbox_error_renders_destructive_class_and_aria` (NEW) | ❌ new test required | ⬜ pending |
| 181-02-02 | 04 | 2 | — | — | A11y: aria-invalid + aria-describedby on fieldset; per-checkbox border swap | unit | `cargo test -p ferro-json-ui --lib render::form::tests::checkbox_list_error_renders_fieldset_aria` (NEW) | ❌ new test required | ⬜ pending |
| 181-02-03 | 05 | 2 | — | — | A11y: aria-invalid on switch; peer-focus ring swap | unit | `cargo test -p ferro-json-ui --lib render::form::tests::switch_error_renders_destructive_ring_and_aria` (NEW) | ❌ new test required | ⬜ pending |
| 181-02-04 | 06 | 2 | — | — | A11y: aria-invalid on file input; ring-destructive added | unit | `cargo test -p ferro-json-ui --lib render::form::tests::input_file_error_renders_destructive_ring_and_aria` (NEW) | ❌ new test required | ⬜ pending |
| 181-03-01 | 07 | 3 | — | — | N/A | manual | gestiscilo cross-repo audit: `rg '\.prop\("error"' ../gestiscilo-it/app/src/` returns ~30 sites; smoke-test one representative form per controller area (cassa/products, calendario/bookings, settings, staff, documenti) shows inline error message after a forced ValidationError redirect-back. | ❌ manual UAT (D-08) | ⬜ pending |
| 181-03-02 | 08 | 3 | — | — | N/A | doc | `ls docs/src/json-ui/forms.md` exists AND `grep -E 'render_validation_error\|\\$data\|with_old_input\|has_validation_errors' docs/src/json-ui/forms.md` finds all four authoring patterns documented (D-09) | ❌ Wave 3 creates | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Critical assertion convention (Pitfall 1 from RESEARCH §475-485):** Pipeline-level integration tests for error rendering MUST use the `html_body(result)` helper (which calls `response.body().to_string()`), NOT the `response_body(result)` helper (which returns Debug-formatted bytes that include the `data-view` attribute payload). Assertions must target `<p id="err-{field}"` tag presence in the rendered HTML, NOT substring presence in the JSON envelope. Tests using `response_body` are false-positive against this bug class.

---

## Wave 0 Requirements

- [ ] `framework/src/json_ui/mod.rs` — upgrade existing `render_with_errors_populates_form_fields` and `render_validation_error_accepts_framework_type` tests to assert via `html_body` on `<p id="err-` tag presence; document expected pre-fix RED state inline.
- [ ] `framework/src/json_ui/mod.rs` — add new test `pipeline_data_binding_error_prop_renders_p_tag` (D-07 test 1): build a Spec with `.prop("error", json!({"$data": "/email_error"}))`, call `JsonUi::render(&spec, &json!({"email_error": "must be valid"}))`, assert `html_body` contains `<p id="err-email"` AND `must be valid`.
- [ ] `framework/src/json_ui/mod.rs` — add new test `pipeline_render_validation_error_renders_p_tag` (D-07 test 2): build a Spec with just `.prop("field", "email").prop("label", "Email")`, call `JsonUi::render_validation_error(&spec, &json!({}), &ValidationError::new().add("email", "must be valid"))`, assert `html_body` contains `<p id="err-email"` AND `must be valid`.
- [ ] Confirm all four Wave 0 tests FAIL before any pipeline fix is applied (RED state is the load-bearing verification — failing tests prove the bug exists). Capture failing test output in plan SUMMARY.md or VERIFICATION.md.
- [ ] No new framework install — Rust workspace ships full toolchain via Cargo.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cross-repo gestiscilo regression (D-08 audit) | — | gestiscilo is a separate workspace; ferro tests cannot exercise its handlers. | After ferro fix lands, in `../gestiscilo-it/`: run `cargo build`, then in a dev session open one representative form per area (cassa/products edit, calendario/booking new, settings staff edit, documenti upload), submit invalid data, confirm inline error `<p>` appears below the offending field AND the top-of-page validation toast renders. |
| `JsonUi::render_with_errors` migration (D-04 escape hatch → blessed path) | — | Behavioral parity check on real consumer code rather than a synthetic test. | In gestiscilo, pick one handler currently using the manual `$data` binding pattern (e.g. `cassa/products::dettaglio`). Switch it to `JsonUi::render_validation_error(...)`. Repeat the form-error UAT. Both paths must produce identical rendered HTML (modulo data-view JSON ordering). |
| Docs page renders (D-09 verification) | — | `mdbook build` validates structure but not prose quality. | After `docs/src/json-ui/forms.md` is written, run `cd docs && mdbook build`, open the rendered page locally, verify all four authoring patterns (blessed, `$data` binding, flash round-trip with `req.old()` + `req.validation_error()`, cross-field summary with `has_validation_errors()`) are described with complete code examples. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags (cargo test runs once and exits — compliant)
- [ ] Feedback latency < 90s warm cache
- [ ] `nyquist_compliant: true` set in frontmatter (after Wave 0 lands and 181-W0-{01..04} are confirmed RED before fix, then GREEN after)

**Approval:** pending
