# Phase 181: JSON-UI Input — render `error` prop inline below the field — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-31
**Phase:** 181-json-ui-input-error-prop-inline-render
**Mode:** `/gsd-discuss-phase 181 --auto` (auto-mode, single pass)
**Areas discussed:** Diagnosis premise, Pipeline suspects, Fix scope, API consolidation, Cross-field toast, Error-state styling parity, Test coverage, Backward compatibility, Docs scope

---

## Pre-discussion codebase scout (informed all decisions)

Before writing CONTEXT.md, read:
- `ferro-json-ui/src/render/form.rs` (full file) — confirmed renderer already emits `<p>` and toggles class chain when `props.error` is `Some(string)`.
- `ferro-json-ui/src/expression.rs` (full file) — confirmed `resolve_expressions` walks `spec.data` only, not runtime data arg.
- `ferro-json-ui/src/resolve.rs` §160-201 — confirmed `attach_errors` writes `errors: Vec<String>` plural, field-name mismatch with `error: Option<String>` on prop structs.
- `framework/src/json_ui/mod.rs` §1-310 — confirmed `JsonUi::render(spec, data)` does NOT merge runtime `data` into `spec.data`; `JsonUi::render_file` DOES via `merge_data`.
- `framework/src/http/request.rs` §240-295 — confirmed both `validation_error()` and `has_validation_errors()` read the same flash key.
- `framework/src/session/store.rs` §87-124 — confirmed `_flash.new.*` → `_flash.old.*` aging semantics.
- `.planning/ROADMAP.md` §1916-1924 — confirmed Phase 181 goal and discovery note.
- `.planning/PROJECT.md` Key Decisions table — confirmed "pre-1.0, breaking changes acceptable" and "hard cap on expression language."

This scout revised the CONTEXT.md premise from "renderer drops error on the floor" to "renderer is correct; pipeline is the fault."

---

## Diagnosis premise

| Option | Description | Selected |
|--------|-------------|----------|
| Trust original CONTEXT.md framing (renderer is the fault) | Plan a renderer patch as the discovery notes propose | |
| Re-verify by reproduction | Researcher must reproduce the failure end-to-end against current ferro tree before scoping the fix | ✓ |

**Auto-mode choice:** Re-verify by reproduction.
**Rationale:** Reading `ferro-json-ui/src/render/form.rs:309-315` contradicts the discovery premise. The renderer DOES emit `<p id="err-{field}" class="text-sm text-destructive">{error}</p>` when `props.error` is `Some(string)`. Building on the unverified premise would ship a fix at the wrong layer.

---

## Pipeline suspects

| Option | Description | Selected |
|--------|-------------|----------|
| Investigate only `resolve_expressions` scoping | Suspect (1) — runtime `data` arg invisible to expression resolution | |
| Investigate only `attach_errors` field-name mismatch | Suspect (2) — writes `errors: Vec<String>` plural where renderer reads `error: Option<String>` singular | |
| Investigate only session flash lifecycle | Suspect (3) — `has_validation_errors` vs `validation_error` divergence | |
| Investigate all three suspects | Pin down the actual root cause through code-level diagnosis | ✓ |

**Auto-mode choice:** Investigate all three.
**Rationale:** Each suspect is independently sufficient to break the discovery scenario, and (1) + (2) are likely both present. Splitting investigation across phases doubles cost.

---

## Fix scope

| Option | Description | Selected |
|--------|-------------|----------|
| Surface (renderer-level patch) | Teach `render_input` / `render_select` / etc. to ALSO read `errors: Vec<String>` and pick `errors[0]` | |
| Pipeline (fix where the data flow breaks) | Fix `resolve_expressions` scope, `attach_errors` field name, or flash lifecycle — whichever is the actual fault | ✓ |
| Both — surface fallback as defense in depth | Renderer reads both shapes; pipeline also fixed | |

**Auto-mode choice:** Pipeline.
**Rationale:** Renderer is already correct (D-01). Surface patches calcify two parallel error pathways and lock in the field-name mismatch as a supported contract. Per memory `feedback_audit_report_fix_discrepancies.md`: fix architecture, do not work around it.

---

## API path consolidation

| Option | Description | Selected |
|--------|-------------|----------|
| Single path only — keep `render_validation_error`, deprecate manual `$data` | Force all consumers onto the blessed path | |
| Single path only — keep manual `$data`, deprecate `render_validation_error` | Reject framework-side error plumbing as too magical | |
| Both paths supported, blessed path is the documented default | `render_validation_error` for 95% case; `$data` binding for custom shapes | ✓ |

**Auto-mode choice:** Both paths, blessed is default.
**Rationale:** Per Phase 137 (Validator & Old Input) and `framework/src/json_ui/mod.rs:293-310` the framework already ships `render_validation_error`. Stripping it would break the documented surface. The escape hatch covers legitimate cases (multi-error display, custom keys, cross-field errors) that the blessed path can't express ergonomically.

---

## Cross-field toast (`toast_validation` symptom)

| Option | Description | Selected |
|--------|-------------|----------|
| Defer to separate phase | Treat `has_validation_errors()` mismatch as independent bug | |
| Fold into Phase 181 | Both symptoms share the validation-error flash round-trip; one investigation covers both | ✓ |

**Auto-mode choice:** Fold in.
**Rationale:** Both readers (`framework/src/http/request.rs:273-295`) hit the same key (`_flash.old._validation_errors`). They cannot disagree within one request without a session-middleware or consumer-helper interaction — diagnosing one diagnoses both.

---

## Error-state styling parity

| Option | Description | Selected |
|--------|-------------|----------|
| Skip — error `<p>` alone is sufficient | Don't expand the visual error treatment to Checkbox / Switch / Input-file | |
| Bring Checkbox / Switch / Input-file / CheckboxList to parity with Input / Select | Same `border-destructive` + destructive ring composition where visually meaningful | ✓ |
| Full visual redesign of error state | Replace the current treatment with something more prominent | |

**Auto-mode choice:** Parity.
**Rationale:** Today Input/Select toggle `border-destructive` + destructive focus-ring at `form.rs:174-184` and `:343-353`. Checkbox/Switch/CheckboxList only emit the `<p>` — visual inconsistency hits the operator when both control types appear on one form. Class composition is Claude's discretion (D-06); the principle is locked.

---

## Test coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Unit tests at renderer level only | Add cases to `ferro-json-ui/src/render/form.rs` tests | |
| Integration tests at JsonUi pipeline level | Exercise full pipeline including `resolve_expressions` and `resolve_errors`; assert on rendered HTML | ✓ |
| Both — unit + integration | Add tests at both layers | |

**Auto-mode choice:** Integration at pipeline level.
**Rationale:** Existing unit tests at `ferro-json-ui/src/render/form.rs:835-851` already cover literal-string `error` props. They do NOT catch the discovery regression because they bypass `resolve_expressions` and `attach_errors`. Two new integration tests in `framework/src/json_ui/mod.rs` test module — one per blessed-vs-escape-hatch path — fail today and pass after the fix.

---

## Backward compatibility

| Option | Description | Selected |
|--------|-------------|----------|
| Backward-compat shim | If `attach_errors` field name changes, add a fallback that accepts both shapes | |
| Clean break | Rework the field name and surface, audit gestiscilo for any workaround code, sync the fix cross-repo | ✓ |

**Auto-mode choice:** Clean break.
**Rationale:** PROJECT.md Status: "Pre-1.0. Breaking changes acceptable across all 0.x." Memory `feedback_breaking_changes_v12_ai.md`: rework freely. Memory `feedback_audit_report_fix_discrepancies.md`: do not work around — fix and migrate consumers in the same release loop.

---

## Docs scope

| Option | Description | Selected |
|--------|-------------|----------|
| No docs update — code-only fix | Skip the docs update | |
| Brief mention in existing form docs page | Single paragraph noting the error-binding pattern | |
| Full validation-flow page covering blessed + escape-hatch + flash round-trip + cross-field summary | All four authoring patterns documented end-to-end | ✓ |

**Auto-mode choice:** Full validation-flow docs page.
**Rationale:** CLAUDE.md user-instruction: "Always update docs when framework changes — `docs/src/` must reflect current features." With four interlocking patterns (blessed render, manual `$data`, flash round-trip on POST→GET, cross-field summary) consumers need a single page that shows how they compose.

---

## Claude's Discretion

- Exact class-chain composition for the error-state styling on Checkbox / CheckboxList / Switch / Input-file (D-06).
- Test placement and exact assertion text for the integration tests (D-07).
- Page filename and section ordering for the docs update (D-09).
- Whether `attach_errors` becomes `error: String` (first message wins) or stays a multi-message shape under a unified field name (D-08). Planner decides after diagnosis pins the actual fault.

## Deferred Ideas

- Multi-error per field display — not exercised by the discovery.
- Live (client-side) validation feedback — banned by PROJECT.md ("hard cap on expression language").
- Toast component structural rework — out of scope; only its data flow is fixed.
- `ferro-projection`-level error projection — v13.0 (Road to v1.0 / Compressive) direction.

## Auto-mode pass cap

Per `discuss-phase.md` "CRITICAL — Auto-mode pass cap": single pass only. CONTEXT.md written once; no re-read for "gap discovery" loop. Proceeding directly to git commit, state update, and auto-advance to `/gsd-plan-phase 181 --auto`.
