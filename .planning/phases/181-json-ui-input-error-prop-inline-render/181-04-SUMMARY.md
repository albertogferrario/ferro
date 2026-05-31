---
phase: 181
plan: 04
status: complete
wave: 4
subsystem: ferro-json-ui
tags: [json-ui, form-rendering, accessibility, error-state, d06-parity, checkbox-list]
commits:
  - 0df9efe5
  - 84b1fdd2
files_modified:
  - ferro-json-ui/src/render/form.rs
key-decisions:
  - "ARIA placed on <fieldset> (group element), not on each individual <input> — correct ARIA pattern for grouped controls per UI-SPEC.md"
  - "checkbox_border local declared before per-option loop; fieldset open tag constructed conditionally before label/description emission"
  - "Test isolates fieldset tag by byte-range (find '<fieldset' + find '>') to assert ARIA lives on the group element, not any option input"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-31T14:25:45Z"
  tasks: 2
  files: 1
---

# Phase 181 Plan 04: CheckboxList D-06 Fieldset ARIA + Per-Option Border + id Summary

CheckboxList error-state parity: ARIA on `<fieldset>` (group-level association), `border-destructive` swap on each option `<input type="checkbox">`, and `id="err-{field}"` on the error `<p>` — bringing `render_checkbox_list` to D-06 parity.

## What Was Built

### Task 1: Apply D-06 error-state parity to `render_checkbox_list` (commit `0df9efe5`)

Modified `ferro-json-ui/src/render/form.rs` — `pub(crate) fn render_checkbox_list`:

**Step 1 — `has_error` gate** (inserted after selected-values resolution):
```rust
let has_error = props.error.is_some();
```

**Step 2 — Conditional fieldset open tag** (replaces the static string):
```rust
let mut html = if has_error {
    format!(
        "<fieldset class=\"space-y-2\" aria-invalid=\"true\" aria-describedby=\"err-{}\">",
        html_escape(&props.field)
    )
} else {
    String::from("<fieldset class=\"space-y-2\">")
};
```

**Step 3 — `checkbox_border` local + per-option `<input>` class swap** (declared before the option loop):
```rust
let checkbox_border = if has_error { "border-destructive" } else { "border-border" };
// ...inside loop:
html.push_str(&format!(
    "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" \
     class=\"h-4 w-4 rounded-sm {} text-primary\"",
    html_escape(&checkbox_id),
    html_escape(&props.field),
    html_escape(&option.value),
    checkbox_border,
));
```

**Step 4 — error `<p>` with `id` attribute** (was missing `id`; without it `aria-describedby` has no target):
```rust
if let Some(ref err) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive mt-1\">{}</p>",
        html_escape(&props.field),
        html_escape(err)
    ));
}
```

Representative rendered output for `field="topics"`, options `["a","b"]`, `error="pick at least one"`:
```html
<fieldset class="space-y-2" aria-invalid="true" aria-describedby="err-topics">
  <legend class="text-sm font-medium text-text">Topics</legend>
  <div class="flex items-center gap-2">
    <input type="checkbox" id="topics_a" name="topics" value="a"
      class="h-4 w-4 rounded-sm border-destructive text-primary">
    <label class="text-sm font-medium text-text" for="topics_a">A</label>
  </div>
  <div class="flex items-center gap-2">
    <input type="checkbox" id="topics_b" name="topics" value="b"
      class="h-4 w-4 rounded-sm border-destructive text-primary">
    <label class="text-sm font-medium text-text" for="topics_b">B</label>
  </div>
  <p id="err-topics" class="text-sm text-destructive mt-1">pick at least one</p>
</fieldset>
```

### Task 2: Add unit test `checkbox_list_error_renders_fieldset_aria` (commit `84b1fdd2`)

New test added in `mod tests` block adjacent to `checkbox_error_renders_destructive_class_and_aria`. Exercises three distinct invariants:

1. **ARIA on `<fieldset>` only** — isolated by byte-range (`find("<fieldset")` + `find('>')`) then asserting `aria-invalid="true"` and `aria-describedby="err-topics"` in that range.
2. **Both option inputs carry `border-destructive`** — splits on `<input type="checkbox"`, skips 1, counts chunks where text before `>` contains `border-destructive` → must equal 2.
3. **`aria-invalid` count == 1** — `html.matches("aria-invalid=\"true\"").count()` must equal 1, confirming no per-input ARIA leakage.
4. **Locked error `<p>` DOM shape** — exact string match `<p id="err-topics" class="text-sm text-destructive mt-1">pick at least one</p>`.

## Verification

```
test render::form::tests::checkbox_list_error_renders_fieldset_aria ... ok
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 529 filtered out
```

All 32 pre-existing form tests pass. The new test passes. No regression.

Acceptance criteria checks:
- `fn render_checkbox_list` → exactly 1 match
- `let has_error = props.error.is_some()` within `render_checkbox_list` → 1 match
- `aria-invalid` within `render_checkbox_list` → 1 match (on `<fieldset>` tag)
- `checkbox_border` within `render_checkbox_list` → 2 matches (declaration + usage)
- `<p id=\"err-` within `render_checkbox_list` → 1 match (error `<p>`)

## Deviations from Plan

None. Plan executed exactly as written.

## Known Stubs

None. The change is fully wired: `props.error` flows from `CheckboxListProps` through the `has_error` gate into the fieldset open tag, `checkbox_border` local, and the `id`-tagged error `<p>`.

## Threat Flags

No new threat surface introduced. Both `html_escape(&props.field)` and `html_escape(err)` are applied at every interpolation point, mirroring the canonical pattern in `render_input`. The threat register mitigations T-181-W2-CL1 and T-181-W2-CL2 are implemented.

## Self-Check: PASSED

- ferro-json-ui/src/render/form.rs: FOUND (modified)
- 181-04-SUMMARY.md: FOUND (this file)
- commit 0df9efe5 (Task 1): FOUND
- commit 84b1fdd2 (Task 2): FOUND
