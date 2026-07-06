---
phase: 181
plan: 03
status: complete
wave: 3
subsystem: ferro-json-ui
tags: [json-ui, form-rendering, accessibility, error-state, d06-parity]
commits:
  - 64285f47
  - f62033af
files_modified:
  - ferro-json-ui/src/render/form.rs
key-decisions:
  - "ARIA block placed before html.push('>') to mirror canonical render_input pattern at form.rs:277-282"
  - "has_error / border_class / focus_ring_class introduced before value_attr to keep logical grouping consistent with render_input"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-31T14:18:06Z"
  tasks: 2
  files: 1
---

# Phase 181 Plan 03: Checkbox D-06 Error-State Class Parity + ARIA + id Summary

Checkbox error-state class swap (`border-border` → `border-destructive`, `ring-primary` → `ring-destructive`), ARIA attributes (`aria-invalid`, `aria-describedby`), and `id="err-{field}"` on the error `<p>` — bringing `render_checkbox` to parity with `render_input` and `render_select`.

## What Was Built

### Task 1: Apply D-06 error-state parity to `render_checkbox` (commit `64285f47`)

Modified `ferro-json-ui/src/render/form.rs` — `pub(crate) fn render_checkbox`:

**Step 1 — `has_error` gate + conditional class strings** (inserted after props decode, before `value_attr`):
```rust
let has_error = props.error.is_some();
let border_class = if has_error { "border-destructive" } else { "border-border" };
let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};
```

**Step 2 — `<input type="checkbox">` class string now uses interpolated variables:**
```rust
html.push_str(&format!(
    "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" class=\"h-4 w-4 rounded-sm {} text-primary transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
    html_escape(&checkbox_id),
    html_escape(&props.field),
    html_escape(value_attr),
    border_class,
    focus_ring_class,
));
```

**Step 3 — ARIA block** (before `html.push('>')`, mirrors form.rs:277-282):
```rust
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
```

**Step 4 — error `<p>` with `id` attribute** (was missing, now added):
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"ml-6 text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
```

Post-fix rendered output for `field="agreed"`, `error="required"`:
```html
<div class="space-y-1">
  <div class="flex items-center gap-2">
    <input type="checkbox" id="agreed" name="agreed" value="1"
      class="h-4 w-4 rounded-sm border-destructive text-primary transition-colors duration-150
             motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed
             focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
      aria-invalid="true" aria-describedby="err-agreed">
    <label class="text-sm font-medium text-text" for="agreed">Agree</label>
  </div>
  <p id="err-agreed" class="ml-6 text-sm text-destructive">required</p>
</div>
```

### Task 2: Add unit test `checkbox_error_renders_destructive_class_and_aria` (commit `f62033af`)

New test added in `mod tests` block adjacent to `input_error_emits_aria_describedby`. Includes the Pitfall 3 guard (isolates `<input>` tag bytes before asserting so the class check is on the input element, not on any sibling `<div>`).

## Verification

```
test render::form::tests::checkbox_error_renders_destructive_class_and_aria ... ok
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 529 filtered out
```

All 31 pre-existing form tests pass. The new test passes. No regression.

Grep checks (acceptance criteria):
- `grep -n 'fn render_checkbox'` → 1 match (no rename)
- `has_error = props.error.is_some()` within `render_checkbox` → 1 match
- `border_class` within `render_checkbox` → 2 matches (definition + usage)
- `aria-invalid` within `render_checkbox` → 1 match
- `id=\\"err-` within `render_checkbox` → 1 match (error `<p>`)
- `cargo build -p ferro-json-ui` → success
- Pitfall 3 guard: `input_tag` referenced 9 times in test (exceeds minimum 4)

## Deviations from Plan

None. Plan executed exactly as written. The prerequisite commits from Plan 02 (Fix A + Fix B) were cherry-picked into the worktree because this worktree was spawned from master rather than from the Plan 02 tip — this was a worktree base mismatch, not a plan deviation. The Plan 03 source changes are isolated to the two task commits above.

## Known Stubs

None. The change is fully wired: `props.error` flows from `CheckboxProps` through the `has_error` gate into class string interpolation, ARIA attributes, and the `id`-tagged error `<p>`.

## Threat Flags

No new threat surface introduced. Both `html_escape(&props.field)` and `html_escape(error)` are applied at every interpolation point, mirroring the canonical ARIA+id pattern already in `render_input` (form.rs:277-282, 309-315). No raw-string interpolation.

## Self-Check: PASSED

- ferro-json-ui/src/render/form.rs: FOUND
- 181-03-SUMMARY.md: FOUND
- commit 64285f47 (Task 1): FOUND
- commit f62033af (Task 2): FOUND
