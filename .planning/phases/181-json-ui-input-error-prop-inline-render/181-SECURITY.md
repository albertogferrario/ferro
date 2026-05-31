---
status: SECURED
phase: 181
plan_range: "01–08"
asvs_level: 1
threats_total: 13
threats_closed: 13
threats_open: 0
audit_date: 2026-06-01
auditor: gsd-secure-phase
---

# Phase 181 — Security Audit Report

Phase: JSON-UI input error-prop inline rendering (Plans 01–08, 8 plans across 7 waves)

## Threat Verification

| Threat ID | Category | Disposition | Evidence |
|-----------|----------|-------------|----------|
| T-181-W0-01 | Information disclosure | accept | Test fixtures use synthetic strings only: `"email_error"`, `"must be valid"`, `"email"`. No PII, no secrets. Confirmed at `framework/src/json_ui/mod.rs:927,957`. |
| T-181-W1-01 | Tampering | mitigate | `spec.clone().merge_data(data.clone())` at two call sites. Original `spec` not mutated. `framework/src/json_ui/mod.rs:84,268`. |
| T-181-W1-02 | Information disclosure | mitigate | `html_escape(error)` at error `<p>` emission for all Input variants: `ferro-json-ui/src/render/form.rs:325,434,521,633,772`. Fix B writes raw string into `props_obj["error"]`; escape occurs at the renderer (consumer), not at the producer. No double-escape, no XSS surface. |
| T-181-W1-03 | Spoofing | accept | `resolve_expressions` walks JSON paths supplied by the handler. `merge_data` does not grant access to data the handler did not pass. No new surface vs. the pre-existing `render_file_with_config` merge pattern (same dispatch path). |
| T-181-W2-C1 | Information disclosure | mitigate | `html_escape(error)` in `render_checkbox` error `<p>` at `ferro-json-ui/src/render/form.rs:521`. Unchanged from pre-Plan-03 emission; parity work only added `id` attribute. |
| T-181-W2-C2 | Information disclosure | mitigate | `html_escape(&props.field)` in Checkbox `aria-describedby` and `id="err-{field}"` at `ferro-json-ui/src/render/form.rs:499,520`. Mirrors canonical Input pattern at lines 292, 324. |
| T-181-W2-CL1 | Information disclosure | mitigate | `html_escape(err)` in `render_checkbox_list` error `<p>` at `ferro-json-ui/src/render/form.rs:633`. |
| T-181-W2-CL2 | Information disclosure | mitigate | `html_escape(&props.field)` in CheckboxList fieldset `aria-describedby` and `id` at `ferro-json-ui/src/render/form.rs:581,632`. |
| T-181-W2-S1 | Information disclosure | mitigate | `html_escape(error)` in `render_switch` error `<p>` at `ferro-json-ui/src/render/form.rs:772`. |
| T-181-W2-S2 | Information disclosure | mitigate | `html_escape(&props.field)` in Switch hidden-input `aria-describedby` and `id` at `ferro-json-ui/src/render/form.rs:758,771`. |
| T-181-W2-F1 | Information disclosure | mitigate | `html_escape(&props.field)` in File input `aria-describedby` at `ferro-json-ui/src/render/form.rs:245` (ARIA block in `InputType::File` arm). `file_ring_class` contains only static class tokens; no field interpolation into the ring class string itself. |
| T-181-W3-A1 | (n/a) | accept | Plan 07 is verification-only (D-08 grep audit + manual UAT + pre-commit gate). No new runtime code paths. No attack surface introduced. |
| T-181-W3-D1 | Information disclosure | mitigate | `docs/src/json-ui/forms.md` uses synthetic field names (`email`, `overage_threshold`). No real consumer code, no secrets, no PII. Confirmed: 0 matches for `v1|v2|legacy`, 4 H2 sections, 15 API-pattern matches. |

## Accepted Risks Log

| Threat ID | Risk Summary | Accepted By | Rationale |
|-----------|-------------|-------------|-----------|
| T-181-W0-01 | Synthetic test data could theoretically reveal field naming conventions | Plan 01 threat model | Field names (`email`, `email_error`) are generic; no system-specific or sensitive naming. |
| T-181-W1-03 | `resolve_expressions` walks handler-controlled JSON paths | Plan 02 threat model | Handler already supplies the `data` argument; `merge_data` expands the resolution scope to data the handler explicitly passed. Identical trust level to the pre-existing `render_file_with_config` pattern shipped before Phase 181. |
| T-181-W3-A1 | No mitigation applicable | Plan 07 threat model | Verification-only plan; no runtime code emitted. |

## Unregistered Threat Flags

No unregistered threat flags. All SUMMARY.md `## Threat Flags` sections across Plans 01–08 reported either "None" or explicitly documented that the flags were covered by the registered mitigations.

## Phase Gate Evidence

Full pre-commit gate result from Plan 07 Task 3 (commit `d362223c`):

| Step | Outcome |
|------|---------|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all --all-targets -- -D warnings` | PASS |
| `cargo test --all-features` | PASS — 2812 passed, 0 failed, 437 ignored |

## Audit Trail

- 2026-06-01 — Security audit by gsd-secure-phase against PLAN.md threat models for Plans 01–08
- All 13 threats verified closed by direct grep evidence in implementation files
- Implementation files read-only; no changes made during this audit
