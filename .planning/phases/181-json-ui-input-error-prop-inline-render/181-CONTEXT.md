---
phase: 181
name: JSON-UI Input — render `error` prop inline below the field
status: Ready for planning
gathered: 2026-05-31
discovered-by: gestiscilo Phase 175 UAT (operator product-edit form)
---

# Phase 181: JSON-UI Input — render `error` prop inline below the field — Context

<domain>
## Phase Boundary

Close the visible-display gap on form-control error messages in the JSON-UI rendering pipeline.

Concretely: every form-control element (`Input`, `Select`, `Textarea` via `Input { input_type: textarea }`, `Checkbox`, `Switch`, `CheckboxList`) accepts an `error` prop. The renderer is already wired to emit a destructive-tone `<p>` element when `props.error` is `Some(string)`. The bug is upstream: the resolution pipeline does not reliably populate that field at render time. This phase fixes the pipeline so the documented authoring patterns (`JsonUi::render_validation_error(...)` and manual `error: {"$data": "/<field>_error"}` binding) both surface the error string in the DOM near the field that produced it.

In scope:
- Diagnose which of three suspected pipeline failure modes actually breaks the gestiscilo product-edit repro.
- Fix the pipeline so `props.error` reaches the renderer as `Some(String)` on the documented authoring paths.
- Cross-cut to the paired symptom: `req.has_validation_errors()` returning `false` while `req.validation_error("field")` returns `Some` for the same request — both readers in `framework/src/http/request.rs:260-295` hit the same key, so this cannot be a renderer-side issue and must share a root cause with the per-field bug.
- Apply error-state class parity to `Checkbox` / `Switch` so the visual treatment matches `Input` / `Select` (which already toggle `border-destructive` + destructive focus-ring at form.rs:174-184).
- Pipeline-level integration test that fixes the regression on the `$data` binding path (current unit tests at `ferro-json-ui/src/render/form.rs:835-851` only cover literal-string `error` props, not the bound path the discovery exercises).
- Docs update covering the blessed `render_validation_error` flow vs the manual `$data` binding flow.

Out of scope:
- JSON-UI catalog rework (covered by v12.0 closure phases 117/117.1, already shipped).
- Server-side expression language beyond `$data` / `$template` (PROJECT.md "Hard cap on expression language" — locked).
- Replacing the session-flash storage model (validated since Phase 137).
- Renderer-level fallback paths that read both `error: String` and `errors: Vec<String>` to mask the bug — surface-only patches that calcify dual pathways are explicitly rejected (D-03 below).

</domain>

<decisions>
## Implementation Decisions

### D-01: Diagnosis premise must be re-verified before any code change
The original discovery note says the v1 Input renderer "drops the value on the floor." Reading the current source contradicts that:

- `ferro-json-ui/src/render/form.rs:174-184` toggles `border-destructive` + destructive focus-ring class chain when `has_error` is true.
- `ferro-json-ui/src/render/form.rs:309-315` emits `<p id="err-{field}" class="text-sm text-destructive">{error}</p>` when `props.error` is `Some(string)`.
- The corresponding emission exists in `render_select` (line 418-424), `render_checkbox` (485-490), `render_switch` (705-710), `render_checkbox_list` (582-587).

The bug must live upstream of the renderer in the resolution pipeline. The researcher must reproduce the gestiscilo failure mode locally against the current ferro tree before scoping the fix. Building on the unverified renderer premise would ship a fix for the wrong layer.

### D-02: Three pipeline suspects, all must be investigated
The plumbing from `req.validation_error("X")` → `<p>{error}</p>` has three branch points that can null out the error string. Researcher resolves all three before planning the fix:

1. **`resolve_expressions` scoping (`ferro-json-ui/src/expression.rs:35-40`).** `pub fn resolve_expressions(spec: &mut Spec) { let data = spec.data.clone(); ... }`. The pass walks `spec.data` only — NOT the runtime `data` argument that the handler passes as the second parameter to `JsonUi::render(spec, data)`. If the gestiscilo handler emits `obj.insert("overage_threshold_error", ...)` into the runtime arg rather than merging into `spec.data` (via `Spec::builder().data(...)` or `merge_data(...)`), then `{"$data": "/overage_threshold_error"}` resolves to `Value::Null` and the renderer correctly skips emission. The contrast: `render_file` at `framework/src/json_ui/mod.rs:194-206` explicitly merges handler data into `spec.data` before resolution; `render(spec, data)` at line 74-86 does not.

2. **`attach_errors` field-name mismatch (`ferro-json-ui/src/resolve.rs:178-201`).** `resolve_errors` is the auto-population path used by `JsonUi::render_validation_error` and `render_with_errors`. It calls `attach_errors`, which inserts `errors: Vec<String>` (plural, array) into the props bag. The form-control prop structs at `ferro-json-ui/src/component.rs:283-490` declare `error: Option<String>` (singular, string). Field-name and shape do not match — the blessed error path silently no-ops.

3. **Flash round-trip / session middleware (`framework/src/http/request.rs:260-295` + `framework/src/session/store.rs:87-124`).** `validation_error()` and `has_validation_errors()` both read `_flash.old._validation_errors`. They cannot disagree within one request unless the session middleware advances `_flash.new` → `_flash.old` between the two reads, or one reader is being intercepted by a consumer-side wrapper. The discovery report claims they DO disagree in gestiscilo Phase 175 UAT — researcher must verify whether this is a ferro bug, a consumer-side helper bug, or a middleware ordering edge case.

The fix lands on whichever of (1), (2), (3) is the actual root cause. (1) and (2) are both plausible — both may need fixing.

### D-03: Fix at the pipeline layer, not at the renderer
The renderer surface is correct. Surface-level patches (e.g., "teach `render_input` to also read `errors: Vec<String>` and pick `errors[0]`") would calcify two parallel error pathways and lock in the field-name mismatch from D-02 suspect (2) as the supported contract. Rejected.

The fix lands wherever the pipeline actually fails: in `resolve_expressions` if the runtime-data scoping is the gap, in `attach_errors` if the field name is the gap, in session middleware ordering if the flash round-trip is the gap.

### D-04: Single blessed path, with documented escape hatch
Two authoring patterns exist today and both must work end-to-end after this phase:

1. **Blessed: `JsonUi::render_validation_error(&spec, &data, &validation_error)`** (`framework/src/json_ui/mod.rs:293-299`) — handler hands the framework a `ValidationError`, framework plumbs error messages onto matching fields automatically. This is the path consumers should use 95% of the time. Today it does not work because of D-02 suspect (2).

2. **Escape hatch: manual `obj.insert("<field>_error", req.validation_error("<field>"))` + `.prop("error", json!({"$data": "/<field>_error"}))`** — gives consumers full control over which error binds where, useful for cross-field error display, custom keys, or per-form-shape variation. Today it does not work because of D-02 suspect (1) (or works only when handlers happen to merge data into `spec.data`).

Both must be functional after Phase 181. The docs update (D-09) makes the blessed path the documented default and the escape hatch the documented alternative for advanced cases.

### D-05: Cross-field validation summary (`toast_validation`) is in scope
The discovery context flags a paired symptom in the gestiscilo product-edit handler:

```rust
if req.has_validation_errors() {
    root_children.push("toast_validation".to_string());
}
```

This branch is not taken even when per-field `req.validation_error("X")` returns `Some` for the same request. Per D-02 suspect (3), both readers in `framework/src/http/request.rs:286-295` hit the same key (`_flash.old._validation_errors`) — they cannot disagree unless the session lifecycle interferes between the two reads, or the consumer's helper intercepts one path.

Folded into this phase because the diagnosis investigation will surface the same root cause; splitting into two phases doubles the investigation cost.

### D-06: Error-state visual parity across all form-control variants
Today's class chain coverage in `ferro-json-ui/src/render/form.rs`:

| Component       | Border on error | Focus ring on error | Error `<p>` |
|-----------------|-----------------|---------------------|-------------|
| Input (text)    | ✅ (174-184)     | ✅                  | ✅          |
| Input (textarea)| ✅ (196-219)     | ✅                  | ✅          |
| Input (file)    | ❌               | ❌                  | ✅          |
| Select          | ✅ (343-353)     | ✅                  | ✅          |
| Checkbox        | ❌ (456-460)     | ❌                  | ✅          |
| CheckboxList    | ❌               | ❌                  | ✅          |
| Switch          | ❌ (684-701)     | ❌                  | ✅          |

Bring Checkbox/CheckboxList/Switch/Input-file to parity. Visual semantics:
- **Checkbox / CheckboxList**: when `has_error`, swap `border-border` → `border-destructive` on the checkbox `<input>`.
- **Switch**: when `has_error`, swap the focus ring color of the hidden checkbox from `peer-focus:ring-primary/30` → `peer-focus:ring-destructive/30`; optionally add a destructive outline on the visible toggle pill.
- **Input file**: when `has_error`, swap `file:bg-surface` → `file:bg-destructive/10` or apply a destructive ring to the wrapper. Defer the exact treatment to the planner; the principle is parity.

Claude's discretion on the exact class-chain composition. The principle is the locked decision.

### D-07: Integration test at the JsonUi pipeline level
Existing tests at `ferro-json-ui/src/render/form.rs:835-851` cover literal-string `error: "required"` — they do not exercise the `$data`-binding path the discovery hits, nor the `render_validation_error` path D-04 blesses.

Add at least two new integration tests landing in `framework/src/json_ui/mod.rs` test module (renders the full pipeline including `resolve_expressions` and `resolve_errors`):

1. **Manual `$data` binding path** — spec with `.prop("error", json!({"$data": "/email_error"}))` + handler data containing `"email_error": "must be valid"`; assert rendered HTML contains `<p id="err-email" class="text-sm text-destructive">must be valid</p>`.
2. **Blessed `render_validation_error` path** — spec with `.prop("field", "email").prop("label", "Email")`; render via `JsonUi::render_validation_error(&spec, &data, &ValidationError::new().add("email", "must be valid"))`; assert same DOM emission.

Both tests must fail today (before the fix) and pass after the fix.

### D-08: Clean break on the `errors: Vec<String>` → `error: Option<String>` reconciliation
If fixing D-02 suspect (2) changes the field name from `errors` to `error` (or unifies on a single shape that both `attach_errors` and the form-control props use), this is a behavior change for any consumer that's somehow worked around the bug. PROJECT.md Status: "Pre-1.0. Breaking changes acceptable across all 0.x." Per memory `feedback_breaking_changes_v12_ai.md`: rework freely.

Cross-repo audit step: search gestiscilo for any consumer code reading `errors` (plural array) off props it constructed — if any, sync the fix to that repo in the same release loop.

### D-09: Docs page covers both paths and the flash round-trip
Update or create `docs/src/json-ui/forms.md` (existing form-rendering page or a new one — planner's call) to cover:

1. The blessed path — `JsonUi::render_validation_error(&spec, &data, &validation_error)` end-to-end example.
2. The escape hatch — `obj.insert("<field>_error", req.validation_error("<field>"))` + `.prop("error", json!({"$data": "/..._error"}))`.
3. The flash round-trip pattern — `errors.with_old_input(&data).redirect_back(...)` on POST, then `req.old("...")` for `default_value` and `req.validation_error("...")` for the error message on the GET re-render.
4. The cross-field summary pattern — `if req.has_validation_errors() { /* render a top-of-page banner */ }`.

CLAUDE.md user-instruction: "Always update docs when framework changes — `docs/src/` must reflect current features." Non-negotiable for this phase.

### Claude's Discretion
- Exact class-chain composition for the new error-state styling on Checkbox / CheckboxList / Switch / Input-file (D-06).
- Test placement and exact assertion text for D-07.
- Page filename and section ordering for the docs update (D-09).
- Whether `attach_errors` becomes `error: String` (first message wins) or stays a multi-message shape under a unified field name. Planner decides after the diagnosis pins down the actual fault.

### Folded Todos
None — no pending todos surfaced by `gsd-tools todo match-phase 181` would alter this phase's scope.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source — renderer & resolution pipeline (ferro-json-ui)
- `ferro-json-ui/src/render/form.rs` §137-318 (`render_input`) — emits `<p id="err-{field}" class="text-sm text-destructive">{error}</p>` when `props.error: Option<String>` is `Some`; toggles `border-destructive` + destructive focus-ring class chain when `has_error`.
- `ferro-json-ui/src/render/form.rs` §320-427 (`render_select`) — same emission pattern as `render_input`.
- `ferro-json-ui/src/render/form.rs` §429-493 (`render_checkbox`) — emits error `<p>` with `ml-6` indent; does not toggle border styling on the checkbox input itself.
- `ferro-json-ui/src/render/form.rs` §495-590 (`render_checkbox_list`) — emits error `<p>` outside the fieldset wrapper.
- `ferro-json-ui/src/render/form.rs` §592-718 (`render_switch`) — emits error `<p>` after the toggle block; does not toggle ring color.
- `ferro-json-ui/src/render/form.rs` §835-851 (existing test `input_error_emits_aria_describedby`) — only literal-string `error: "required"`; does not cover the `$data` binding path.
- `ferro-json-ui/src/component.rs` §283-490 — `InputProps`, `SelectProps`, `CheckboxProps`, `CheckboxListProps`, `SwitchProps` all declare `pub error: Option<String>`.
- `ferro-json-ui/src/expression.rs` §35-66 — `resolve_expressions` reads `spec.data.clone()` only; runtime `data` argument to `JsonUi::render(spec, data)` is NOT visible to expression resolution.
- `ferro-json-ui/src/expression.rs` §17-22 — pipeline ordering comment: "must run after `resolve_actions` and before `Catalog::validate`."
- `ferro-json-ui/src/resolve.rs` §162-201 — `resolve_errors` + `attach_errors` writes `props_obj.insert("errors", Value::Array(...))` (plural, array). Field-name mismatch with `InputProps.error`.

### Source — framework integration layer
- `framework/src/json_ui/mod.rs` §37-86 — `JsonUi::render(spec, data)` calls `resolve()` then `build_response(&resolved, data, config)`. Pipeline: `expand_directives` → `resolve_actions` → `resolve_expressions`.
- `framework/src/json_ui/mod.rs` §181-206 — `JsonUi::render_file` merges `handler_data` into `spec.data` before resolution (contrast with `render(spec, data)` which does not).
- `framework/src/json_ui/mod.rs` §223-310 — `resolve_with_errors`, `render_with_errors`, `render_validation_error`, `render_json_validation_error` — the blessed error-flow APIs.
- `framework/src/http/request.rs` §247-295 — `req.old()`, `req.validation_error()`, `req.has_validation_errors()`. Both error-flag readers hit `_flash.old._validation_errors`.

### Source — flash lifecycle
- `framework/src/session/store.rs` §87, §118-124 — `_flash.new.*` → `_flash.old.*` aging on session save.
- `framework/src/validation/error.rs` §147 — `_flash.new.*` namespace isolation note.

### Discovery context
- `.planning/ROADMAP.md` §1916-1924 — Phase 181 entry with goal statement and discovery note.
- `.planning/phases/175-structured-product-pricing-model-five-typed-columns-on-produ/175-SUMMARY.md` — gestiscilo Phase 175 UAT context (the discovery surface).
- `.planning/PROJECT.md` Key Decisions §415 "Hard cap on expression language" — `$data` + `$template` only; no `$if`/`$for`/`$state`.

### Prior-phase context (already-decided constraints to honor)
- `.planning/phases/137-validator-old-input/137-CONTEXT.md` (if present) — `Validator` + old-input flash round-trip semantics (`with_old_input`, `redirect_back`, `_flash.old._old_input.<field>`, `_flash.old._validation_errors`).
- `.planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/` — v1 JSON-UI API deletion. Per memory `feedback_json_ui_naming.md`: drop the "v1" / "v2" language in public docs; there is one JSON-UI.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Class-chain pattern for error-state** — `ferro-json-ui/src/render/form.rs:174-184` shows the locked formula: `border-destructive` swap + destructive focus-ring composition. Reuse verbatim for D-06 parity work.
- **Error `<p>` emission template** — `<p id="err-{field}" class="text-sm text-destructive">{error}</p>` is the locked DOM shape for error messages. `aria-invalid="true"` + `aria-describedby="err-{field}"` on the form control is the accessibility pairing.
- **`html_escape` helper** — `ferro-json-ui/src/render/mod.rs::html_escape` (re-exported in `framework/src/json_ui/mod.rs:319-325`) is the standard escape for any string interpolated into HTML attribute or text context.
- **Pipeline composition pattern** — `framework/src/json_ui/mod.rs:48-66` (`JsonUi::resolve`) and §227-246 (`JsonUi::resolve_with_errors`) show the locked ordering: `expand_directives` → `global_catalog().validate` → `resolve_actions` → `resolve_expressions` → (optional) `resolve_errors`. Any new pipeline step lands at a known position in this chain.
- **`merge_data` API** — `Spec::merge_data(handler_data)` (used by `render_file` at §202) is the structural answer to D-02 suspect (1): if the fix is to lift handler data into `spec.data` automatically, this is the existing API to call.
- **`ValidationError::all()`** — `crate::validation::ValidationError::all()` returns `&HashMap<String, Vec<String>>` — the shape `render_validation_error` already speaks.

### Established Patterns
- **`<p>` description and `<p>` error live side-by-side inside a `space-y-1` wrapper** — when both `description` and `error` are present, the wrapper's `space-y-1` gives the right vertical rhythm without extra margin classes.
- **`aria-invalid` + `aria-describedby` pairing** — `render_input` at §213-218 and §277-282 already does this when `has_error`. Replicate for Checkbox / Switch when D-06 extends error-state styling.
- **Decode-failure diagnostics** — `ferro-json-ui` renderers emit `<!-- ferro-json-ui: failed to decode {Component} props: {err} -->` HTML comments on shape decode failure rather than panicking. The post-fix integration tests (D-07) MUST assert no such comment leaks in the happy path.

### Integration Points
- **Handler-side consumers** — gestiscilo's `gestiscilo/src/controllers/cassa/products.rs::dettaglio` (referenced in the discovery) is the exemplar of the manual `$data` binding pattern. After the fix, that handler should be able to switch to `JsonUi::render_validation_error(...)` without re-architecting (D-04). Cross-repo audit (D-08) drives any required gestiscilo migration in the same release loop.
- **Docs site** — `docs/src/json-ui/` contains the public surface documentation. Forms-related authoring guidance currently lives across multiple pages (`forms.md` if it exists, plus per-component pages). D-09 consolidates the validation-error story.
- **MCP introspection** — `ferro-mcp` exposes `json_ui_catalog`. The form-control prop schemas surfaced via MCP should reflect the post-fix shape; verify schema accuracy after the pipeline fix lands.

</code_context>

<specifics>
## Specific Ideas

- Test ergonomics inspiration: existing `input_xss_in_value_is_escaped` at `ferro-json-ui/src/render/form.rs:819-832` builds a minimal spec, calls the renderer directly, asserts on the produced HTML string. The pipeline-level tests under D-07 follow the same shape but invoke `JsonUi::render_*` from `framework/src/json_ui/mod.rs` instead of the renderer in isolation — so the full resolve pipeline is exercised.
- Discovery quote that anchors scope: from the original notes, "the data block carries `overage_threshold_error: 'Per il sovrapprezzo...'`" with "the form value is restored to `value=\"2\"` via the data_path binding" — both observations confirm the runtime data IS reaching the spec, which makes D-02 suspect (1) the most likely actual fault (data_path resolution at form.rs:158 hits the runtime `data` arg directly via `resolve_path_string(data, dp)`, but `resolve_expressions` hits `spec.data` only).
- Audit hint: `rg '\.prop\("error"' gestiscilo/app/src/` (per the original notes' last line) returns ~30 bindings across `cassa/products`, `calendario/bookings`, `settings`, `staff`, `documenti`. All of those become live error displays the moment the pipeline fix ships — sanity-check a representative sample after the cross-repo loop completes.

</specifics>

<deferred>
## Deferred Ideas

- **Multi-error per field display** — Today's `error: Option<String>` shape carries only the first message. Consumers wanting to display the full error array per field need either a new prop or a `$data` binding into a JSON array. Out of scope for Phase 181 (the discovery exercises single-message display); revisit if a real consumer surfaces the need.
- **Live (client-side) validation feedback** — `$state`/`$bindState` and client-side reactive evaluation are explicitly banned by PROJECT.md ("Hard cap on expression language"). Server-roundtrip validation is the supported model.
- **Toast component reformulation** — The discovery flags `toast_validation` as a related symptom (D-05 folds it into scope), but a structural rework of the toast component itself (e.g., promoting it to a first-class component with field-error iteration) is a separate consideration; this phase only fixes the data flow that feeds it.
- **`ferro-projection`-level error projection** — Mapping `ValidationError` through the projection/intent system (rather than threading it into a spec by hand) is an architectural direction for v13.0 (Road to v1.0 / Compressive). Not Phase 181.

### Reviewed Todos (not folded)
None — `gsd-tools todo match-phase 181` produced no matches at discuss time.

</deferred>

---

## Discovery Transcript (preserved from prior CONTEXT.md)

The original discovery notes captured during gestiscilo Phase 175 UAT (2026-05-31) framed this phase as "the v1 Input renderer drops the value on the floor." Code reading during `/gsd-discuss-phase 181 --auto` (2026-05-31) revised that framing: the renderer is correct; the bug is upstream in the resolution pipeline. The full original repro is preserved here for historical context and end-to-end reproduction.

### Concrete repro (gestiscilo product-edit form)

`gestiscilo/src/controllers/cassa/products.rs::dettaglio` — operator product-edit page at `/dashboard/cassa/prodotti/{id}/modifica`.

```rust
builder = builder.element(
    "field_overage_threshold",
    Element::new("Input")
        .prop("field", "overage_threshold")
        .prop("label", "Soglia sovrapprezzo")
        .prop("input_type", "number")
        .prop("data_path", /* ... */)
        .prop("error", json!({ "$data": "/overage_threshold_error" })),
);
```

Data plumbing (same handler):

```rust
obj.insert("overage_threshold_error".to_string(),
           json!(req.validation_error("overage_threshold")));
```

After a `ValidationError::new().add("overage_threshold", "Per il sovrapprezzo, ...").with_old_input(&data).redirect_to(...)` round-trip:

- ✅ The data block carries `overage_threshold_error: "Per il sovrapprezzo, compila sia la soglia che il prezzo"`
- ✅ `req.validation_error("overage_threshold")` returns Some(msg)
- ✅ The form value is restored to `value="2"` via the data_path binding
- ❌ No `<p>` / `<span>` / error element is rendered below the input
- ❌ The fallback `?error=generic&msg=Per+il+sovrapprezzo...` query-string flash is the operator's only visible signal.

Observed DOM:

```html
<div class="space-y-1">
  <label for="overage_threshold">Soglia sovrapprezzo</label>
  <input type="number" id="overage_threshold" value="2">
  <p class="text-sm text-text-muted">Numero di persone oltre cui scatta…</p>
</div>
```

### Affected gestiscilo fields (audit)

```bash
rg '\.prop\("error"' ../gestiscilo-it/app/src/
```

Last count before this phase: ~30 bindings across cassa/products, calendario/bookings, settings, staff, documenti. All become live error displays the moment the pipeline fix ships.

---

*Phase: 181-json-ui-input-error-prop-inline-render*
*Context gathered: 2026-05-31*
