# Phase 175: JSON-UI v2 Runtime Patches — Staff-Domain Field Test (F1–F6) — Research

**Researched:** 2026-05-20
**Domain:** ferro-json-ui runtime — spec validation, component catalog, tab rendering, DataTable interpolation, file upload
**Confidence:** HIGH (all findings verified against live source files)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-F1-depth** — `MAX_NESTING_DEPTH = 16`. Consumer evidence requires at least 8; 16 provides headroom. Reject any plan proposing a value below 12 without new consumer evidence.
- **D-F1-diagnostic** — Split into two distinct diagnostics: `depth limit exceeded at depth N (max=M)` for depth-limit trip; `cycle detected: <path>` for real cycles. Remove the current "cycle guard tripped" comment from the render walker.
- **D-F2-CheckboxGroup** — Option (c) both: register `CheckboxGroup` as a first-class v2 component with the same semantics as `CheckboxList` AND document the v2-native `Form` + repeated `Checkbox[]` substitution path.
- **D-F3-tabs** — Client-side IIFE. Extend the existing `setupTabs()` function in `ferro-json-ui/src/runtime/tabs.rs` to apply initial tab state from `?tab=` URL param at DOMContentLoaded. Server-side conditional render rejected.
- **D-F4-Switch** — Option (c) both: register native `Switch` as a first-class v2 component AND document `Checkbox` with `variant: "switch"` as the substitution path. Note: `Switch` IS already registered and rendered (see Critical Finding F4 below). The plan for F4 is: confirm Switch renders after F1, add `variant` prop to `CheckboxProps` for documentation anchor, and write the substitution docs.
- **D-F5** — File input + `Form.enctype` propagation ship together in a single plan.
- **D-F6** — Extend the existing `template_actions` / `template_url` interpolation pass in `render/data.rs` to also resolve `{row.X}` prefix placeholders (in addition to existing bare `{X}` substitution). Additive, non-breaking.

### Claude's Discretion

- Exact wording of the new depth-limit and cycle diagnostics (must mention limit value and offending depth)
- Internal implementation choice for the tabs IIFE (extend `setupTabs` vs separate `initTabFromUrl` fn)
- Self-check fixture for F5 (in-process multipart submit vs spec round-trip)
- ARIA semantics for `Switch` substitution docs (`role="switch"` is the obvious pick — confirm at plan time)

### Deferred Ideas (OUT OF SCOPE)

- Consumer-side urlencoded fallback in the staff create controller (stays in consumer repo)
- v1 catalog re-imports beyond F2 (CheckboxGroup) and F4 (Switch)
- Plugin model parity
- HXML / non-WebView protocol direction
</user_constraints>

---

## Summary

Phase 175 is a six-finding runtime patch batch against ferro-json-ui, all rooted in the v12.0 JSON-UI v2 runtime as exercised by a gestiscilo-it staff-domain CRUD surface (per-day weekly hours editor with copy-source shortcut, two-month calendar overlay, multipart avatar upload). Every finding has a verified source location; none requires an architectural change.

**Critical pre-planning finding:** F4 ("Switch does not render") is a misdiagnosis in the CONTEXT. `Switch` is already a registered, dispatched, fully-rendered built-in in v2 (`render/mod.rs:BUILTIN_TYPES`, `render/form.rs:render_switch`). The consumer's symptom was Switch being absent from the DOM at depth 8, which is caused entirely by F1 (the render walker's depth tripwire at `MAX_NESTING_DEPTH + 1 = 6` strips nodes before they reach the dispatch). The F4 plan should: (1) confirm Switch renders once F1 raises the limit, (2) add a `variant` field to `CheckboxProps` as the documentation anchor for the "switch-style" substitution pattern, and (3) write the docs. It does NOT need to "register" Switch (already done).

**Critical pre-planning finding (F6):** The existing `template_actions` already handles `{delete_url}` (bare column-key pattern). The consumer used `{row.delete_url}` (prefixed pattern). The fix is to add `{row.X}` → `row[X]` substitution as an alias in `template_actions` and `template_url`. This is three lines of Rust.

**Primary recommendation:** Land all six plans in wave order per CONTEXT.md — F1 first (unblocks F4, reduces consumer blast radius), then F3/F6 (high traffic), then F2/F4 together (component additions), then F5 (file upload surface).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| F1: Spec parse-time depth limit | ferro-json-ui spec validator (`spec.rs`) | render walker defense guard (`render/mod.rs`) | Two locations, both must be updated together |
| F1: Diagnostic message split | `spec.rs` `SpecError` variants | `render/mod.rs` walker comment | Error variants live in spec; walker comment is independent |
| F2: CheckboxGroup registration | `render/mod.rs` BUILTIN_TYPES + dispatch | `catalog.rs` BUILTIN_SPECS + `component.rs` Props | Registration requires all three; docs in `docs/src/json-ui/components.md` |
| F3: Tab initial state from URL | `runtime/tabs.rs` `setupTabs` IIFE | None | Client-side JavaScript; no server changes |
| F4: Switch confirm + docs | `render/mod.rs` test (verify) + `component.rs` CheckboxProps.variant | `docs/src/json-ui/components.md` | Mostly a docs task after F1 lands |
| F5: Input[type=file] rendering | `component.rs` `InputType` enum | `render/form.rs` `render_input` match arm | Two changes, one per sub-finding |
| F5: Form.enctype propagation | `component.rs` `FormProps.enctype` | `render/form.rs` `render_form` | Two changes, one per sub-finding |
| F6: {row.X} interpolation | `render/data.rs` `template_actions` + `template_url` | None | Pure render-layer addition |

---

## Finding-by-Finding Research

### F1 — Spec Depth Limit

**Files to modify:** [VERIFIED: source read]

| File | Change Shape |
|------|-------------|
| `ferro-json-ui/src/spec.rs` | Change constant `MAX_NESTING_DEPTH: usize = 5` → `16`; update docstring |
| `ferro-json-ui/src/spec.rs` | Update `DepthExceeded` error display string (already has good format: `"nesting depth exceeds maximum of {max}: found depth {found}"`) |
| `ferro-json-ui/src/render/mod.rs` | Change walker tripwire diagnostic comment: `"<!-- ferro-json-ui: cycle guard tripped at depth {depth} — spec should have been rejected at parse time -->"` → `"<!-- ferro-json-ui: depth limit exceeded at depth {depth} (max={MAX_NESTING_DEPTH}) — spec should have been rejected at parse time -->"` |
| `ferro-json-ui/src/spec.rs` tests | Update `from_json_rejects_six_level_nesting` (currently at depth 6 = MAX+1 = 6; this test must be rewritten to use depth 17) and `nested_builder_rejects_depth_six` (same); update boundary tests for "accepts five" → "accepts sixteen" |

**Existing surface:** [VERIFIED: source read]

```rust
// ferro-json-ui/src/spec.rs:37
pub const MAX_NESTING_DEPTH: usize = 5;

// render/mod.rs:137-140 (walker tripwire — wrong diagnostic)
if depth > MAX_NESTING_DEPTH + 1 {
    return format!(
        "<!-- ferro-json-ui: cycle guard tripped at depth {depth} — spec should have been rejected at parse time -->"
    );
}
```

Note that the spec validator uses `depth > MAX_NESTING_DEPTH` at depth 1-based counting, while the render walker uses `depth > MAX_NESTING_DEPTH + 1` (extra +1 because render starts at depth 1 not 0). After changing `MAX_NESTING_DEPTH = 16`, the walker fires at depth 18+ and parse-time at depth 17+. The consumer's depth-8 spec will pass cleanly.

**Existing tests that will break (must update):** [VERIFIED: source read]
- `spec.rs:1111` — `from_json_rejects_six_level_nesting` — hardcodes depth 6 (== old MAX+1). Rewrite to depth 17.
- `spec.rs:1813` — `nested_builder_rejects_depth_six` — same. Rewrite to depth 17.
- `spec.rs:1793` — `nested_builder_accepts_depth_five` — should become `nested_builder_accepts_depth_sixteen` (boundary test at the new limit)
- `render/mod.rs:402-408` — `walker_cycle_tripwire_fires_at_depth_4` — passes `MAX_NESTING_DEPTH + 2` directly so it auto-adapts; verify it still passes.

**Self-check:** `cargo test -p ferro-json-ui spec` — all depth tests pass. Construct a 17-element chain spec and confirm it rejects with `DepthExceeded { found: 17, max: 16 }`. Construct an 8-deep spec (matching consumer evidence) and confirm it serializes and renders without stripping.

**Pattern analog:** Phase 164 changed MAX_NESTING_DEPTH from 3 to 5 using the identical constant-rename pattern. See `.planning/phases/162-json-ui-improvements-batch-1-components-expressions-and-spec/162-01-PLAN.md` for the prior precedent.

---

### F2 — CheckboxGroup Component

**Key research finding:** `CheckboxGroup` does NOT exist anywhere in the ferro-json-ui codebase (verified by grep across the entire workspace). It was never shipped in v2. The v2 equivalent is `CheckboxList`, which has the same semantics (`field`, `options`/`options_path`, `selected_path`, `label`, `description`, `disabled`, `error`). [VERIFIED: source read]

The plan must CREATE `CheckboxGroup` as a new component — it is effectively an alias for `CheckboxList` with the type name consumers expect from v1. Two implementation strategies:

**Strategy A (alias):** In `render/form.rs`, dispatch "CheckboxGroup" to the same `render_checkbox_list` function. Add a `CheckboxGroupProps` struct that is identical to `CheckboxListProps` (or reuse `CheckboxListProps` directly with a type alias). Register in catalog, BUILTIN_TYPES, and dispatch.

**Strategy B (type alias only):** In `render/mod.rs`, add `"CheckboxGroup" => form::render_checkbox_list(el, spec, data, depth)` to the dispatch. Use `CheckboxListProps` as-is for deserialization (identical field shape). Register in `catalog.rs` with the same description and same props schema as CheckboxList. BUILTIN_TYPES count goes from 42 to 43.

Strategy B is simpler and sufficient. The decision to document CheckboxGroup as an alias for CheckboxList is appropriate.

**Files to modify:** [VERIFIED: source read]

| File | Change Shape |
|------|-------------|
| `ferro-json-ui/src/render/mod.rs` | Add `"CheckboxGroup"` to `BUILTIN_TYPES` array (43 entries); add dispatch arm `"CheckboxGroup" => form::render_checkbox_list(el, spec, data, depth)` |
| `ferro-json-ui/src/render/mod.rs` | Update `builtin_types_count_matches_dispatch` test: `assert_eq!(BUILTIN_TYPES.len(), 43)` |
| `ferro-json-ui/src/catalog.rs` | Add `("CheckboxGroup", "Multi-select checkbox group (alias for CheckboxList). Each checked option submits as field=value.", ...)` to `BUILTIN_SPECS`; import `CheckboxListProps` for its schema |
| `docs/src/json-ui/components.md` | Add CheckboxGroup section documenting alias relationship; add substitution example using `Form` + repeated `Checkbox[]` |

**Self-check:** Construct a spec with `"type": "CheckboxGroup"` and `options: [...]`, render it, assert HTML contains `<fieldset` and `<input type="checkbox"`. Verify `global_catalog().lookup("CheckboxGroup").is_some()`.

**Pattern analog:** `CheckboxList` registration in `catalog.rs:361-367` and dispatch in `render/mod.rs:202`. CheckboxGroup mirrors both exactly.

---

### F3 — Tabbed Pages Render Every Panel Concurrently

**Key research finding:** The `render_tabs` function in `render/containers.rs:273-283` ALREADY emits `class="... hidden"` on non-default tab panels at server render time. The bug is that when a consumer's spec `props.default_tab` does not match the URL's `?tab=` parameter, the server has no way to honor the URL-requested tab. On page load, the JavaScript `setupTabs()` in `runtime/tabs.rs` only wires click handlers — it does NOT initialize from the current URL. Result: the server renders tab "A" as active (from `default_tab`), but the URL says tab "B" should be active. All of tab B's panels are in the DOM (rendered as `hidden`) but the tab strip doesn't activate B. [VERIFIED: source read]

**The IIFE fix shape:** Extend `setupTabs()` in `runtime/tabs.rs` to, on `DOMContentLoaded`, read `?tab=` from `window.location.search` and programmatically activate that tab if found. The existing click handler logic in `makeTabHandler` already knows how to toggle panels — reuse it.

**Files to modify:** [VERIFIED: source read]

| File | Change Shape |
|------|-------------|
| `ferro-json-ui/src/runtime/tabs.rs` | Add `initTabFromUrl(container)` function; call from `initTabContainer(container)` after wiring click handlers |
| `ferro-json-ui/src/runtime/mod.rs` | Add assertion test that `FERRO_RUNTIME_JS.contains("initTabFromUrl")` |

**Proposed addition to `tabs.rs`:**
```javascript
function initTabFromUrl(container) {
    var params = new URLSearchParams(window.location.search);
    var value = params.get('tab');
    if (!value) return;
    var triggers = container.querySelectorAll('[data-tab]');
    var panels = container.querySelectorAll('[data-tab-panel]');
    makeTabHandler(triggers, panels)({ currentTarget: { getAttribute: function() { return value; } } });
}
```
Called from `initTabContainer` after handler wiring.

**Self-check:** Construct a `Tabs` spec with two tabs, render it, assert the non-default tab has `hidden` class in HTML. Separately, a runtime/browser test (or integration test) would be ideal but inline Rust tests cannot drive the browser; a minimal self-check is: assert the runtime JS string contains `initTabFromUrl` and `URLSearchParams` (same pattern as the existing toast URL test in `runtime/mod.rs:116-121`).

**Pattern analog:** `runtime/toasts.rs` already uses `new URLSearchParams(window.location.search)` + `history.replaceState` for URL-based toast init on DOMContentLoaded. F3's URL-init pattern is identical.

---

### F4 — Switch Component Does Not Render

**Critical finding:** `Switch` IS registered, dispatched, and rendered in v2. [VERIFIED: source read]
- `render/mod.rs:85` — `"Switch"` in `BUILTIN_TYPES`
- `render/mod.rs:201` — `"Switch" => form::render_switch(el, spec, data, depth)`
- `render/form.rs:577-697` — `render_switch` full implementation with tests
- `catalog.rs:355-360` — Switch in `BUILTIN_SPECS`

The consumer's symptom (Switch absent from DOM) is caused entirely by F1: at depth 8, the render walker fires its tripwire at `MAX_NESTING_DEPTH + 1 = 6` and emits the diagnostic comment instead of dispatching. After F1 raises the limit to 16, Switch will render at depth 8 without any additional code changes.

**What the F4 plan actually needs to do (per D-F4 decision):**
1. **Verify** Switch renders correctly at depths up to 16 once F1 is applied (unit test: build an 8-deep spec where the deepest element is a Switch, confirm it renders `role="switch"`).
2. **Document** the `Checkbox variant: "switch"` substitution path. NOTE: `CheckboxProps` has NO `variant` field. The plan must decide whether to add a `variant` field to `CheckboxProps` (with `Switch` value that changes rendering) or document the substitution purely as guidance ("use Switch for toggle semantics; if you prefer composing from Checkbox primitives, apply these CSS classes"). The simpler path is docs-only: the docs page describes the `Switch` component and notes that consumers who want Checkbox with switch appearance can style it manually.

**Files to modify:** [VERIFIED: source read]

| File | Change Shape |
|------|-------------|
| `docs/src/json-ui/components.md` | Add `Checkbox variant=switch` substitution note to the Switch section; clarify semantic distinction (state-flip vs binary-choice) |
| `ferro-json-ui/src/render/form.rs` | Add a test: depth-8 Switch renders role="switch" after F1 lands (can reference the spec round-trip helper) |

**Self-check:** After F1 lands, `cargo test -p ferro-json-ui render::form::switch` passes. Build a spec at depth 8 with Switch at leaf; render; assert DOM contains `role="switch"`.

**No catalog, BUILTIN_TYPES, or render changes needed for F4.**

---

### F5 — Input[type=file] and Form.enctype

**Two sub-findings, one plan per CONTEXT decision.**

#### F5a — `Input[type=file]` not rendered

**Root cause:** `InputType` enum in `component.rs:68-81` does NOT include a `File` variant. The `render_input` match arm has no `InputType::File` case; if a consumer JSON-decodes `input_type: "file"`, serde will return a decode error (the field is `#[serde(rename_all = "snake_case")]` so it would look for literal `"file"`). Actually since `InputType::Text` is the default, a missing variant means serde falls back to the default `Text` — no decode error, but the wrong input type. Either way, `<input type="file">` is never emitted. [VERIFIED: source read]

**Files to modify:** [VERIFIED: source read]

| File | Change Shape |
|------|-------------|
| `ferro-json-ui/src/component.rs` | Add `File` to `InputType` enum (after `Search` at line 80). Add `accept: Option<String>` field to `InputProps` for the `accept="image/jpeg,..."` attribute |
| `ferro-json-ui/src/render/form.rs` | Add `InputType::File` arm to the `match props.input_type` block in `render_input`. Emit `<input type="file" id=... name=... [accept=...]>`. No `value=""` attribute (file inputs cannot be pre-filled). No label wrapper difference needed (use existing `space-y-1` wrapper with label) |
| `ferro-json-ui/src/render/form.rs` | Update the `InputType::Textarea | InputType::Hidden => unreachable!()` catch-arm to also exclude `InputType::File` |

**Proposed `InputType::File` render:**
```rust
InputType::File => {
    html.push_str(&format!(
        "<input type=\"file\" id=\"{}\" name=\"{}\" class=\"block w-full text-sm text-text file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-surface file:text-text hover:file:bg-surface/80\"",
        html_escape(&props.field),
        html_escape(&props.field),
    ));
    if let Some(ref accept) = props.accept {
        html.push_str(&format!(" accept=\"{}\"", html_escape(accept)));
    }
    if props.required == Some(true) {
        html.push_str(" required");
    }
    if props.disabled == Some(true) {
        html.push_str(" disabled");
    }
    html.push('>');
}
```

#### F5b — `Form.enctype` not propagated

**Root cause:** `FormProps` in `component.rs:209-226` has no `enctype` field. `render_form` in `form.rs:37-131` emits `<form ... action=... method=...>` with no `enctype` attribute. [VERIFIED: source read]

**Files to modify:** [VERIFIED: source read]

| File | Change Shape |
|------|-------------|
| `ferro-json-ui/src/component.rs` | Add `enctype: Option<String>` field to `FormProps` (`#[serde(default, skip_serializing_if = "Option::is_none")]`) |
| `ferro-json-ui/src/render/form.rs` | In `render_form`, after building the `<form ...>` opening tag string, append ` enctype="..."` when `props.enctype.is_some()`. Must pass through `html_escape`. |

**Self-check for F5:** Construct a spec with `Form { enctype: "multipart/form-data" }` containing an `Input { input_type: "file", accept: "image/jpeg" }`. Render to HTML. Assert `<form ... enctype="multipart/form-data">` is present. Assert `<input type="file" accept="image/jpeg"` is present. Both in the same unit test (single self-check satisfies the "shipping both unblocks" criterion).

**Pattern analog:** `Form.guard` field in `FormProps` (added in a prior phase) follows the same `Option<String>` + emit-when-Some pattern as `enctype`. `render_form:88-101` shows the guard attribute pattern to mirror for enctype.

---

### F6 — DataTable {row.X} Placeholders Not Interpolated

**Root cause verified:** `template_actions` in `render/data.rs:357-402` iterates the row object and substitutes `{col_key}` patterns (bare key, no prefix). The consumer used `{row.delete_url}` (prefixed with `row.`). The current code has no path for the `row.` prefix. [VERIFIED: source read]

**Files to modify:** [VERIFIED: source read]

| File | Change Shape |
|------|-------------|
| `ferro-json-ui/src/render/data.rs` | In `template_actions` (line 381-389), after the existing `{col_key}` loop, add a second loop substituting `{row.X}` as an alias for `{X}` — or do it in one pass by checking both `format!("{{{col_key}}}")` and `format!("{{row.{col_key}}}")` in the same loop body |
| `ferro-json-ui/src/render/data.rs` | Same change in `template_url` (line 306-328) for `props.row_href` URL templating consistency |
| `ferro-json-ui/src/render/data.rs` | Add unit test: DataTable with `action: "{row.delete_url}"` renders the resolved URL |

**Proposed addition inside the `col_key` loop:**
```rust
for (col_key, col_val) in obj {
    let val_str = match col_val { ... };
    url = url.replace(&format!("{{{col_key}}}"), &val_str);
    url = url.replace(&format!("{{row.{col_key}}}", ), &val_str);  // row. prefix alias
}
```

**Self-check:** Add a test to `render/data.rs` tests (near `data_table_url_template_replaces_column_key` at line 690): construct a DataTable with `row_actions: [{"action": {"url": "{row.delete_url}"}}]`, render with row data `{"delete_url": "/dashboard/staff/1/assenze/3/elimina"}`, assert rendered HTML contains `/dashboard/staff/1/assenze/3/elimina`.

**Existing analog tests that validate the fix doesn't break:** `data_table_url_template_missing_key_leaves_placeholder` (line 833) — a placeholder with no matching column must remain literal. The fix must preserve this invariant for `{row.X}` as well (if `X` not in row object, leave `{row.X}` as-is).

---

## Dependencies Between Findings

| Finding | Depends On | Notes |
|---------|-----------|-------|
| F1 | None | Prerequisite for F4 diagnosis to close |
| F2 | None | Independent component addition |
| F3 | None | Independent runtime IIFE addition |
| F4 | F1 (must land first) | F4 plan verifies Switch renders at depth 8+; pointless before F1 |
| F5 | None | Independent (file upload surface is shallower than depth 8) |
| F6 | None | Independent data-layer fix |

Wave order from CONTEXT.md: F1 → F3 → F6 → F2 + F4 (parallel) → F5.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner via `cargo test` |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Per-Finding Test Map

| Finding | Behavior | Test Type | Automated Command | Test File |
|---------|----------|-----------|-------------------|-----------|
| F1 | Depth-17 spec rejected with DepthExceeded | unit | `cargo test -p ferro-json-ui from_json_rejects` | `spec.rs` tests module |
| F1 | Depth-8 spec accepted and renders | unit | `cargo test -p ferro-json-ui nested_builder` | `spec.rs` tests module |
| F1 | Walker diagnostic says "depth limit" not "cycle guard" | unit | `cargo test -p ferro-json-ui walker_cycle_tripwire` | `render/mod.rs` tests |
| F2 | CheckboxGroup renders fieldset + checkboxes | unit | `cargo test -p ferro-json-ui render::form` | `render/form.rs` |
| F2 | global_catalog().lookup("CheckboxGroup") is Some | unit | `cargo test -p ferro-json-ui catalog` | `catalog.rs` |
| F3 | runtime JS contains initTabFromUrl + URLSearchParams | unit | `cargo test -p ferro-json-ui runtime` | `runtime/mod.rs` |
| F3 | Non-default tab panel has hidden class in rendered HTML | unit | `cargo test -p ferro-json-ui render::containers` | `render/containers.rs` |
| F4 | Depth-8 spec with Switch renders role="switch" | unit | `cargo test -p ferro-json-ui render::form::switch` | `render/form.rs` |
| F5 | Form with enctype=multipart/form-data emits enctype attr | unit | `cargo test -p ferro-json-ui render::form` | `render/form.rs` |
| F5 | Input[type=file] with accept emits type=file and accept attr | unit | `cargo test -p ferro-json-ui render::form` | `render/form.rs` |
| F6 | {row.delete_url} placeholder resolves in DataTable action URL | unit | `cargo test -p ferro-json-ui render::data` | `render/data.rs` |

### Phase Gate

All six findings green = all of the above pass + `cargo test --all-features` passes with zero warnings.

### Wave 0 Gaps

The existing test infrastructure is sufficient. Tests to ADD (not pre-existing):
- [ ] `spec.rs` — rewrite `from_json_rejects_six_level_nesting` → `from_json_rejects_depth_17`
- [ ] `spec.rs` — add `from_json_accepts_depth_8` (consumer evidence fixture)
- [ ] `render/form.rs` — `checkbox_group_renders_fieldset` (new)
- [ ] `render/form.rs` — `input_file_renders_file_type_and_accept` (new)
- [ ] `render/form.rs` — `form_enctype_emitted_when_set` (new)
- [ ] `render/form.rs` — `switch_at_depth_8_renders_role_switch` (new, requires unchecked spec builder at depth 8)
- [ ] `render/data.rs` — `data_table_row_prefix_placeholder_resolved` (new)
- [ ] `runtime/mod.rs` — assert `FERRO_RUNTIME_JS.contains("initTabFromUrl")`

---

## Pattern Map Preview

For the pattern-mapper agent, the closest analogs in the codebase:

| Finding | Closest Analog | Location |
|---------|---------------|----------|
| F1 constant change | `MAX_NESTING_DEPTH` prior raise (Phase 164) | `ferro-json-ui/src/spec.rs:37` |
| F1 diagnostic rename | `render_plugin_or_unknown` diagnostic comment pattern | `render/mod.rs:211-217` |
| F2 registration | `CheckboxList` registration | `catalog.rs:362-367`, `render/mod.rs:202`, `render/form.rs:477` |
| F3 URL-init IIFE | `initToastFromUrl` in toasts | `runtime/toasts.rs` (URLSearchParams pattern) |
| F4 test-via-depth | `walker_cycle_tripwire_fires_at_depth_4` pattern | `render/mod.rs:401-409` |
| F5a InputType variant | `InputType::Hidden` arm in render_input | `render/form.rs:160-168` |
| F5b Form attribute emission | `FormProps.guard` → `data-form-guard="..."` emission | `render/form.rs:88-101` |
| F6 interpolation | `template_actions` `{col_key}` loop | `render/data.rs:380-391` |

---

## Common Pitfalls

### Pitfall 1: BUILTIN_TYPES count assertion
**What goes wrong:** `render/mod.rs:543` asserts `BUILTIN_TYPES.len() == 42`. Adding CheckboxGroup bumps it to 43; the test will fail if not updated.
**How to avoid:** Update the count assertion alongside the BUILTIN_TYPES array and dispatch arm.

### Pitfall 2: Depth test cascade
**What goes wrong:** Several spec.rs tests hardcode depth 6 as the rejection boundary. After raising MAX to 16, these tests become success cases, not rejection cases.
**How to avoid:** Identify all hardcoded depth values in spec.rs tests (lines 1112, 1770, 1793, 1813, 1814, 1830) and update to the new boundary (depth 17).

### Pitfall 3: File input `value=""` emission
**What goes wrong:** The general `render_input` arm emits `value="..."` from `resolved_value`. For file inputs, value is security-restricted by browsers; emitting it would be ignored at best and cause an error at worst.
**How to avoid:** The `InputType::File` arm must not emit a `value` attribute. No `default_value` support for file fields.

### Pitfall 4: F6 {row.X} vs {X} for missing keys
**What goes wrong:** The existing test `data_table_url_template_missing_key_leaves_placeholder` asserts that `{nonexistent}` stays literal. The new `{row.nonexistent}` must behave identically (stay literal). A naive implementation might silently strip it.
**How to avoid:** The substitution loop only fires when the key exists in the row object; if `X` is absent, neither `{X}` nor `{row.X}` is replaced. Verify with a test.

### Pitfall 5: F3 tabs IIFE fires before DOM is ready
**What goes wrong:** If `initTabFromUrl` is called synchronously (not via DOMContentLoaded), elements may not be in the DOM.
**How to avoid:** The IIFE already wraps everything in `document.addEventListener('DOMContentLoaded', ferroRuntime)`. `initTabFromUrl` is called from `initTabContainer`, which is called from `setupTabs()`, which is called from `ferroRuntime()`. No extra guard needed — the call chain is already DOMContentLoaded-gated.

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes — `accept` prop for file inputs | `html_escape` on all prop values before emission (existing pattern) |
| V2 Authentication | no | |
| V3 Session Management | no | |
| V4 Access Control | no | |
| V6 Cryptography | no | |

**File upload security note:** The `accept` attribute is a UI hint only — browsers do not enforce it on actual file content. Server-side MIME validation is the consumer's responsibility (not in scope for F5; consistent with the existing CLAUDE.md security posture of validate-all-inputs in form handlers).

---

## Runtime State Inventory

This is a greenfield-additive code patch batch, not a rename/refactor. No runtime state changes:
- Stored data: None — no DB schema changes.
- Live service config: None.
- OS-registered state: None.
- Secrets/env vars: None.
- Build artifacts: None beyond the compiled crate.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — all changes are Rust source code and in-process JavaScript string constants; `cargo` is the only tool needed, already available).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `CheckboxGroup` in v1 was semantically equivalent to `CheckboxList` (multi-checkbox with shared name) | F2 | Low — the CONTEXT explicitly describes the consumer's usage as a multi-select picker matching CheckboxList's API exactly |
| A2 | Adding `{row.X}` alias in `template_actions` does not conflict with any existing row field named `row` | F6 | Very low — a row field literally named "row" would only cause issues if its value were also used as a url segment, which is structurally unlikely |

---

## Sources

### Primary (HIGH confidence — verified by source read in this session)
- `ferro-json-ui/src/spec.rs` — MAX_NESTING_DEPTH constant (line 37), DepthExceeded variant (line 195), detect_cycle and check_depth functions (lines 894, 925)
- `ferro-json-ui/src/render/mod.rs` — BUILTIN_TYPES (lines 43-90), walker tripwire (lines 137-140), dispatch arms (lines 159-208)
- `ferro-json-ui/src/render/form.rs` — render_switch (line 577), render_checkbox_list (line 477), render_form (line 37), render_input (line 139)
- `ferro-json-ui/src/render/containers.rs` — render_tabs with hidden class emission (lines 273-283)
- `ferro-json-ui/src/render/data.rs` — template_actions (line 357), template_url (line 306)
- `ferro-json-ui/src/runtime/tabs.rs` — setupTabs IIFE (click-only, no URL init)
- `ferro-json-ui/src/runtime/mod.rs` — FERRO_RUNTIME_JS assembly and tests
- `ferro-json-ui/src/component.rs` — InputType enum (line 68), FormProps (line 209), CheckboxProps (line 382), SwitchProps (line 437)
- `ferro-json-ui/src/catalog.rs` — BUILTIN_SPECS registration pattern (lines 300-380)
- `ferro-json-ui/src/plugin.rs` — plugin registry pattern (reference for F2 registration analog)

### Tertiary (LOW confidence — inferred from symptom descriptions in CONTEXT.md, not directly observed)
- F4: Consumer observation that Switch "produces no DOM element" — our research shows Switch IS registered; the root cause is F1 depth stripping. This ASSUMED interpretation shapes the F4 plan significantly.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — this is pure in-crate Rust work; no external library changes
- Architecture: HIGH — all files verified by source read
- Pitfalls: HIGH — derived from direct code analysis; not speculation

**Research date:** 2026-05-20
**Valid until:** Stable (no external dependencies; valid until the codebase changes)
