---
phase: 181
plan: 06
status: complete
wave: 6
subsystem: ferro-json-ui
tags: [json-ui, form-rendering, accessibility, error-state, d06-parity, file-input]
commits:
  - 1bd34b06
  - 3faad2f5
files_modified:
  - ferro-json-ui/src/render/form.rs
key-decisions:
  - "ring-1 ring-destructive appended to class string (not border-swap) because file inputs have no CSS border to swap cleanly"
  - "ARIA block placed after accept/required/disabled attributes, before closing >, mirroring the canonical pattern at form.rs:277-282"
  - "Shared error <p> at lines 309-315 left untouched — already has id=err-{field} from prior plans"
  - "file_ring_class uses leading space inside destructive variant to separate from hover:file:bg-surface/80"
metrics:
  duration: "~8 minutes"
  completed: "2026-05-31T20:32:24Z"
  tasks: 2
  files: 1
---

# Phase 181 Plan 06: Input(file) D-06 Destructive Ring + ARIA Summary

File input error-state parity: `ring-1 ring-destructive` appended to the class string and `aria-invalid`/`aria-describedby` added to `<input type="file">` when `has_error` — completing Wave 2 D-06 parity across all four form-control variants (Checkbox / CheckboxList / Switch / Input-file).

## What Was Built

### Task 1: Apply D-06 error-state parity to `render_input` `InputType::File` branch (commit `1bd34b06`)

Modified `ferro-json-ui/src/render/form.rs` — `InputType::File =>` arm inside `pub(crate) fn render_input`:

**Step 1 — `file_ring_class` local** (reuses `has_error` declared at line 174):
```rust
let file_ring_class = if has_error { " ring-1 ring-destructive" } else { "" };
```

**Step 2 — Class string extended with `file_ring_class`** (leading space inside the destructive variant separates from `hover:file:bg-surface/80`):
```rust
html.push_str(&format!(
    "<input type=\"file\" id=\"{}\" name=\"{}\" class=\"block w-full text-sm text-text file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-surface file:text-text hover:file:bg-surface/80{}\"",
    html_escape(&props.field),
    html_escape(&props.field),
    file_ring_class,
));
```

**Step 3 — ARIA block after existing attribute emissions** (mirrors canonical form.rs:277-282):
```rust
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
html.push('>');
```

Shared error `<p>` at form.rs:309-315 is untouched — it already carries `id="err-{field}"` from Plan 03 work.

Representative rendered output for `field="avatar"`, `label="Avatar"`, `input_type="file"`, `error="must be PNG"`:
```html
<div class="space-y-1">
  <label class="block text-sm font-medium text-text" for="avatar">Avatar</label>
  <input type="file" id="avatar" name="avatar"
    class="block w-full text-sm text-text file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-surface file:text-text hover:file:bg-surface/80 ring-1 ring-destructive"
    aria-invalid="true" aria-describedby="err-avatar">
  <p id="err-avatar" class="text-sm text-destructive">must be PNG</p>
</div>
```

### Task 2: Add unit test `input_file_error_renders_destructive_ring_and_aria` (commit `3faad2f5`)

New `#[test]` added inside `mod tests`, adjacent to `switch_error_renders_destructive_ring_and_aria`. Exercises three invariants with Pitfall 3-resistant isolation:

1. **Isolate `<input type="file">` tag bytes** — finds the tag by byte-range before any assertion, preventing false positives from the error `<p>` text appearing elsewhere in the HTML.
2. **Ring and ARIA on the isolated tag** — asserts `ring-1 ring-destructive`, `aria-invalid="true"`, and `aria-describedby="err-avatar"` are present in the isolated tag bytes.
3. **Shared error `<p>` DOM shape** — exact string match `<p id="err-avatar" class="text-sm text-destructive">must be PNG</p>` verifying the cross-variant block at form.rs:309-315 was not accidentally broken.

## Wave 2 D-06 Parity — Complete

All four Wave 2 components are now at D-06 parity:

| Plan | Component | Ring/Border | ARIA on | Error `<p>` id |
|------|-----------|-------------|---------|----------------|
| 03 | Checkbox | `border-destructive` + `ring-destructive` focus | `<input type="checkbox">` | `ml-6 text-sm text-destructive` |
| 04 | CheckboxList | `border-destructive` per-option + `aria-invalid` on `<fieldset>` | `<fieldset>` | `text-sm text-destructive mt-1` |
| 05 | Switch | `peer-focus:ring-destructive/30` on pill | hidden `<input class="sr-only peer">` | `text-sm text-destructive` |
| 06 | Input(file) | `ring-1 ring-destructive` appended | `<input type="file">` | shared `<p>` at line 309-315 |

## Verification

```
test render::form::tests::input_file_error_renders_destructive_ring_and_aria ... ok
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 529 filtered out
```

All 34 pre-existing form tests pass. New test passes.

Acceptance criteria checks:
- `InputType::File =>` → 1 main match (line 221; line 257 is `unreachable!()` arm)
- `file_ring_class` within `InputType::File` arm → 2 matches (declaration + interpolation)
- `ring-1 ring-destructive` within `InputType::File` arm → 1 match (in `if has_error` branch)
- `aria-invalid` within `InputType::File` arm → 1 match (in `if has_error` block)
- `fn input_file_error_renders_destructive_ring_and_aria` → exactly 1 match (line 1074)

## Deviations from Plan

None. Plan executed exactly as written.

## Known Stubs

None. The change is fully wired: `props.error` flows from `InputProps` through the `has_error` gate (line 174) into `file_ring_class` and the ARIA block, and the shared error `<p>` at lines 309-315 already emits the `id` attribute.

## Threat Flags

No new threat surface introduced. `html_escape(&props.field)` is applied at every interpolation point — mitigation T-181-W2-F1 from the plan's threat register is implemented.

## Self-Check: PASSED

- ferro-json-ui/src/render/form.rs: FOUND (modified)
- 181-06-SUMMARY.md: FOUND (this file)
- commit 1bd34b06 (Task 1): verified via git log
- commit 3faad2f5 (Task 2): verified via git log
