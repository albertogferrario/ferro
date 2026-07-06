---
phase: 181
plan: 05
status: complete
wave: 5
subsystem: ferro-json-ui
tags: [json-ui, form-rendering, accessibility, error-state, d06-parity, switch]
commits:
  - e304252a
  - df0d66ca
files_modified:
  - ferro-json-ui/src/render/form.rs
key-decisions:
  - "ARIA placed on hidden <input> (sr-only peer), not on the pill <div> — form control is the correct ARIA target for role=switch"
  - "peer_ring_class carries both ring-2 and ring-destructive/30 together to prevent partial-swap where ring-2 is dropped"
  - "error <p> gains id=err-{field} so aria-describedby on the hidden <input> resolves to a real DOM target"
metrics:
  duration: "~8 minutes"
  completed: "2026-05-31T14:30:13Z"
  tasks: 2
  files: 1
---

# Phase 181 Plan 05: Switch D-06 Peer-Focus Destructive Ring + ARIA + id Summary

Switch error-state parity: `peer-focus:ring-destructive/30` on the pill `<div>` (via `peer_ring_class`), `aria-invalid`/`aria-describedby` on the hidden `<input class="sr-only peer">`, and `id="err-{field}"` on the error `<p>` — bringing `render_switch` to D-06 parity.

## What Was Built

### Task 1: Apply D-06 error-state parity to `render_switch` (commit `e304252a`)

Modified `ferro-json-ui/src/render/form.rs` — `pub(crate) fn render_switch`:

**Step 1 — `has_error` gate + `peer_ring_class`** (inserted after `is_checked` resolution):
```rust
let has_error = props.error.is_some();
let peer_ring_class = if has_error {
    "peer-focus:ring-2 peer-focus:ring-destructive/30"
} else {
    "peer-focus:ring-2 peer-focus:ring-primary/30"
};
```

**Step 2 — ARIA on hidden `<input>`** (before closing `>` of the sr-only checkbox):
```rust
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
```

**Step 3 — Pill `<div>` emission uses `peer_ring_class`** (replaces hardcoded `peer-focus:ring-2 peer-focus:ring-primary/30`):
```rust
html.push_str(&format!(
    "<div class=\"w-11 h-6 bg-border rounded-full peer peer-checked:bg-primary {} after:content-[''] ...\"></div>",
    peer_ring_class
));
```

**Step 4 — error `<p>` with `id` attribute** (was missing `id`; without it `aria-describedby` dangled):
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
```

Representative rendered output for `field="notify"`, `label="Notify me"`, `error="required"`:
```html
<div class="space-y-1">
  <div class="flex items-center justify-between">
    <div>
      <label class="text-sm font-medium text-text" for="notify">Notify me</label>
    </div>
    <label class="relative inline-flex items-center cursor-pointer">
      <input type="checkbox" id="notify" name="notify" value="1" role="switch"
        aria-checked="false" class="sr-only peer"
        aria-invalid="true" aria-describedby="err-notify">
      <div class="w-11 h-6 bg-border rounded-full peer peer-checked:bg-primary
        peer-focus:ring-2 peer-focus:ring-destructive/30
        after:content-[''] after:absolute after:top-0.5 after:left-[2px]
        after:bg-background after:rounded-full after:h-5 after:w-5 after:transition-all
        peer-checked:after:translate-x-full"></div>
    </label>
  </div>
  <p id="err-notify" class="text-sm text-destructive">required</p>
</div>
```

### Task 2: Add unit test `switch_error_renders_destructive_ring_and_aria` (commit `df0d66ca`)

New test added in `mod tests` block adjacent to `checkbox_list_error_renders_fieldset_aria`. Exercises four distinct invariants:

1. **Hidden `<input>` is the sr-only peer** — isolates the first `<input>` by byte-range and asserts `class="sr-only peer"` is present, confirming we found the correct element.
2. **ARIA on hidden `<input>`** — asserts `aria-invalid="true"` and `aria-describedby="err-notify"` are in the isolated input tag.
3. **Pill ring swap — both directions** — asserts `peer-focus:ring-destructive/30` is present AND `peer-focus:ring-primary/30` is absent, so a partial swap (only adding destructive without removing primary) fails.
4. **Locked error `<p>` DOM shape** — exact string match `<p id="err-notify" class="text-sm text-destructive">required</p>`.

## Verification

```
test render::form::tests::switch_error_renders_destructive_ring_and_aria ... ok
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 529 filtered out
```

All 33 pre-existing form tests pass. The new test passes. No regression.

Acceptance criteria checks:
- `fn render_switch` → exactly 1 match
- `peer_ring_class` within `render_switch` → 2 matches (declaration + emission)
- `aria-invalid` within `render_switch` → 1 match (on hidden `<input>`)
- `<p id=\"err-` within `render_switch` → 1 match (error `<p>`)
- `peer-focus:ring-primary/30` within `render_switch` → 1 match (non-error branch of `peer_ring_class`)

## Deviations from Plan

None. Plan executed exactly as written.

## Known Stubs

None. The change is fully wired: `props.error` flows from `SwitchProps` through the `has_error` gate into `peer_ring_class`, the ARIA block on the hidden `<input>`, and the `id`-tagged error `<p>`.

## Threat Flags

No new threat surface introduced. Both `html_escape(&props.field)` and `html_escape(error)` are applied at every interpolation point — mitigations T-181-W2-S1 and T-181-W2-S2 from the plan's threat register are implemented.

## Self-Check: PASSED

- ferro-json-ui/src/render/form.rs: FOUND (modified)
- 181-05-SUMMARY.md: FOUND (this file)
- commit e304252a (Task 1): FOUND
- commit df0d66ca (Task 2): FOUND
