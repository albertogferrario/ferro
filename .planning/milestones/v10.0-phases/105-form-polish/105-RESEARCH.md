# Phase 105: Form Polish - Research

**Researched:** 2026-03-25
**Domain:** Tailwind CSS form utilities, Rust HTML string construction in ferro-json-ui render pipeline
**Confidence:** HIGH

## Summary

Phase 105 applies six visual polish changes to form elements in `ferro-json-ui/src/render.rs`: custom SVG chevron arrow for select, error-state focus rings using `ring-destructive`, transition animations with reduced-motion support, disabled state styling, and correcting the form field DOM ordering to label → input → description → error.

All changes are class string modifications and wrapper HTML restructuring in existing Rust source functions. No new dependencies, no CSS file changes, no new Rust crates required. The token `--color-destructive` already exists in `ferro-theme/assets/default.css`, so `ring-destructive`, `focus-visible:ring-destructive` are valid semantic classes.

The SVG chevron approach must use an inline SVG element inside a wrapper div with Tailwind's `relative` + `absolute` positioning — NOT an SVG data URI as a Tailwind arbitrary background value. CDN mode does not reliably handle SVG data URIs as arbitrary class values (documented in the REQUIREMENTS.md "Out of Scope" table as "unverified in CDN mode"). The wrapper-div-with-inline-SVG pattern is CSS-only, no JavaScript.

**Primary recommendation:** Update `render_input`, `render_select`, and the Textarea branch inside `render_input` in `render.rs`. For each: fix focus ring to use `focus-visible:ring-2 focus-visible:ring-primary` normally and `focus-visible:ring-destructive` when `has_error`; add `transition-colors duration-150 motion-reduce:transition-none`; add `disabled:opacity-50 disabled:cursor-not-allowed`; wrap select with a `relative` div + inline SVG chevron. Fix DOM order to label → input → description → error for `render_input` and `render_select`.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FRM-01 | Select element displays a custom SVG dropdown arrow (CSS-only, no JS) | Wrap `<select>` in `<div class="relative">`, emit inline SVG chevron with `aria-hidden="true" class="pointer-events-none absolute inset-y-0 right-3 flex items-center"` after `</select>`. `appearance-none` already on select (line 834). No JS needed. |
| FRM-02 | Input in error state shows `focus-visible:ring-destructive` | In `render_input` (line 753 and 723 Textarea branch): replace `focus:border-primary focus:ring-1 focus:ring-primary` with conditional focus ring — `focus-visible:ring-2 focus-visible:ring-primary` for normal, `focus-visible:ring-2 focus-visible:ring-destructive` for error state. |
| FRM-03 | All form elements have `transition-colors duration-150 motion-reduce:transition-none` | Add to class strings in `render_input` (lines 723 and 753), `render_select` (line 834). Checkbox (line 905) and Switch (line 1033) are interactive but lower priority — requirements scope is Input, Select, Textarea. |
| FRM-04 | All form elements have `disabled:opacity-50 disabled:cursor-not-allowed` | Add to class strings in `render_input` (lines 723 and 753) and `render_select` (line 834). Removes the need to conditionally add ` opacity-50 cursor-not-allowed` classes elsewhere — Tailwind's `disabled:` variant applies automatically when the HTML `disabled` attribute is set. |
| FRM-05 | Select in error state shows `focus-visible:ring-destructive` | Same pattern as FRM-02 but in `render_select` (line 834): conditional focus ring based on `has_error`. |
| FRM-06 | Textarea in error state shows `focus-visible:ring-destructive` | In `render_input`, Textarea branch (line 723): same conditional focus ring. Textarea is inside `render_input` as `InputType::Textarea`. |
| FRM-07 | Form field order is label → input → description → error message | Current order in `render_input` and `render_select`: label → description → input → error (WRONG). Move description `<p>` push after the input/select/textarea push, before error. Checkbox order is already correct (input+label → description → error). |
</phase_requirements>

## Standard Stack

### Core (no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS v4 CDN | `@4` (jsdelivr) | Form utilities | `focus-visible:ring-*`, `disabled:opacity-50`, `disabled:cursor-not-allowed`, `transition-colors`, `motion-reduce:transition-none`, `pointer-events-none`, `inset-y-0`, `right-3` — all standard Tailwind v4 utilities auto-generated from CDN |
| ferro-theme/assets/default.css | project-local | Token definitions | `--color-destructive` already defined; `ring-destructive` is a valid semantic class. No CSS file changes needed. |

### No new Rust dependencies required
All changes are class string modifications and HTML structure changes in existing Rust source files.

**Installation:** None required.

## Architecture Patterns

### Recommended File Change Map
```
ferro-json-ui/
├── src/render.rs   # All 7 FRM requirements
│   ├── render_input() line 679:
│   │   ├── FRM-02/FRM-06: conditional focus ring (error vs normal)
│   │   ├── FRM-03: add transition-colors duration-150 motion-reduce:transition-none
│   │   ├── FRM-04: add disabled:opacity-50 disabled:cursor-not-allowed
│   │   └── FRM-07: move description push to after input/textarea push
│   ├── render_select() line 802:
│   │   ├── FRM-01: wrap <select> in relative div, add inline SVG chevron
│   │   ├── FRM-03: add transition-colors duration-150 motion-reduce:transition-none
│   │   ├── FRM-04: add disabled:opacity-50 disabled:cursor-not-allowed
│   │   ├── FRM-05: conditional focus ring (error vs normal)
│   │   └── FRM-07: move description push to after </select> push
│   └── Tests: update cosmetic full-string assertions and add new tests
```

### Pattern 1: Conditional Focus Ring (FRM-02, FRM-05, FRM-06)
**What:** When `has_error` is true, the focus ring uses `ring-destructive` instead of `ring-primary`. When false, uses `ring-primary`. Also changes `focus:` to `focus-visible:` per modern a11y practice.
**When to use:** Every form field that has an error state.

```rust
// Source: ferro-json-ui/src/render.rs — pattern to apply in render_input and render_select

let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
};

// Input (line 753):
// BEFORE:
"... focus:border-primary focus:ring-1 focus:ring-primary\""
// AFTER:
"... transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {focus_ring_class}\""

// Same pattern for Textarea (line 723) and Select (line 834).
```

Note: `focus:border-primary` is removed — focus feedback is provided entirely via the ring, consistent with Phase 106 button pattern.

### Pattern 2: Select SVG Chevron Wrapper (FRM-01)
**What:** Wrap `<select>` in a `<div class="relative">` block. After `</select>`, inject an inline SVG chevron with pointer-events-none absolute positioning. The `appearance-none` class (already present on select at line 834) suppresses the native browser arrow.
**When to use:** Only in `render_select`. This is CSS-only — no JS.

```rust
// Source: ferro-json-ui/src/render.rs — render_select()

// Replace:
//   html.push_str(&format!("<select ... class=\"...appearance-none...\"",...));
//   ...
//   html.push_str("</select>");
// With:
html.push_str("<div class=\"relative\">");
html.push_str(&format!(
    "<select id=\"{}\" name=\"{}\" class=\"block w-full appearance-none bg-background rounded-md border {} px-3 py-2 pr-10 text-sm shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
    html_escape(&props.field),
    html_escape(&props.field),
    border_class,
    focus_ring_class
));
// ... attributes ...
html.push_str("</select>");
html.push_str(
    "<span class=\"pointer-events-none absolute inset-y-0 right-3 flex items-center\" aria-hidden=\"true\">\
    <svg class=\"h-4 w-4 text-text-muted\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">\
    <path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>\
    </svg>\
    </span>"
);
html.push_str("</div>");
```

Key detail: add `pr-10` to the select's class to prevent text from overlapping the chevron. The SVG uses `currentColor` so it inherits `text-text-muted` color automatically.

### Pattern 3: Form Field DOM Order (FRM-07)
**What:** Move the description `<p>` push to AFTER the input/select/textarea push, but BEFORE the error `<p>` push.
**When to use:** `render_input` and `render_select`. Checkbox order is already correct.

```rust
// render_input — BEFORE (wrong order: label → desc → input → error):
html.push_str(&format!("<label .../>"));
if let Some(ref desc) = props.description { html.push_str(...) }  // description BEFORE input
match props.input_type { ... }  // input
if let Some(ref error) = props.error { html.push_str(...) }

// render_input — AFTER (correct order: label → input → desc → error):
html.push_str(&format!("<label .../>"));
match props.input_type { ... }  // input FIRST
if let Some(ref desc) = props.description { html.push_str(...) }  // description AFTER input
if let Some(ref error) = props.error { html.push_str(...) }
```

### Pattern 4: Disabled State via Tailwind Variant (FRM-04)
**What:** Add `disabled:opacity-50 disabled:cursor-not-allowed` to class strings. Tailwind's `disabled:` variant applies CSS when the element has the HTML `disabled` attribute, so no conditional Rust logic is needed.
**When to use:** `render_input` (both regular input and textarea) and `render_select`.

The existing conditional Rust logic that adds ` disabled` attribute HTML attribute stays — the `disabled` HTML attribute is needed to disable form submission and user interaction. The Tailwind `disabled:` variant classes are additive styling on top.

### Anti-Patterns to Avoid
- **SVG as background-image arbitrary value:** `bg-[url('data:image/svg+xml,...')]` is unreliable in CDN mode. Use inline SVG with absolute positioning instead.
- **Removing the `disabled` HTML attribute:** Keep the `disabled` HTML attribute — it's required for form submission behavior. The `disabled:` Tailwind variant adds visual styling ON TOP of the attribute.
- **Keeping `focus:border-primary`:** Remove this — the focus border competes visually with the ring. Focus feedback is fully covered by the ring-2 approach.
- **Not adding `pr-10` to select:** Without right padding, the option text overlaps the chevron SVG on narrow selects.
- **`pointer-events-none` missing on SVG span:** Without it, clicking the chevron won't open the select (the click passes through to the `<select>` below it in z-order).
- **Applying FRM-07 ordering to checkbox:** Checkbox order is intentionally different (input + label side-by-side, then description/error below). Don't reorder checkbox.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Disabled visual state | Conditional Rust class strings based on `disabled == Some(true)` | `disabled:opacity-50 disabled:cursor-not-allowed` Tailwind variant | Automatic — CSS applies when HTML `disabled` attribute present. Simpler code. |
| Error focus color | If-else producing separate full class strings for normal/error | Single `focus_ring_class` variable + format interpolation | Keeps class string DRY — only the ring color changes |
| Reduced motion | Custom media query in CSS | `motion-reduce:transition-none` Tailwind variant | Built-in, no CSS file needed, applies globally to `prefers-reduced-motion: reduce` |
| Custom select arrow | JS-based custom dropdown | Inline SVG + `appearance-none` + absolute positioning | Zero JS, works in all browsers, no accessibility regression |

**Key insight:** Tailwind v4 CDN generates variants like `disabled:`, `motion-reduce:`, `focus-visible:` automatically when encountered in HTML. These classes require no custom configuration — they are part of Tailwind's default variant system.

## Common Pitfalls

### Pitfall 1: SVG Data URI in CDN Mode Fails
**What goes wrong:** Developer tries `class="bg-[url('data:image/svg+xml,...')]"` — Tailwind CDN does not generate the utility because the class name has special characters and spaces.
**Why it happens:** CDN mode scans HTML for class names and generates only what it finds. SVG data URIs contain characters that break Tailwind's class name scanner.
**How to avoid:** Use the inline SVG wrapper pattern. The SVG is an HTML element, not a CSS background-image. No Tailwind arbitrary values needed at all.
**Warning signs:** Select shows no chevron in the browser; Chrome DevTools shows no matching CSS rule for the background-image class.

### Pitfall 2: Cosmetic Test Failures from Class String Changes
**What goes wrong:** Tests that assert the full class string of input/select/textarea fail when classes are added or reordered.
**Why it happens:** Several tests use `html.contains("focus:border-primary focus:ring-1 focus:ring-primary")` or similar full-class-string assertions.
**How to avoid:** After changing class strings, run `cargo test -p ferro-json-ui` and update any failing assertions. Look for tests at lines 3152 (`border-border` assertion — not affected), 3177 (`border-destructive` assertion — not affected). The focus ring assertions: currently NO tests assert on focus ring class strings, so FRM-02/03/05/06 changes do not break existing tests.
**Warning signs:** `cargo test -p ferro-json-ui` fails with "assertion failed: html.contains(...)".

### Pitfall 3: FRM-07 Ordering Breaks Tests That Assert Description Presence
**What goes wrong:** Moving description after input might break a test that asserts the description appears immediately after the label.
**Why it happens:** If a test asserts on adjacent text like `<label>...Email</label><p class="text-sm text-text-muted">Your work email</p>`, moving the description block changes that sequence.
**How to avoid:** Check existing tests — line 3146 only asserts `html.contains("Your work email")`, which is order-independent. No test will break from the FRM-07 reorder.
**Warning signs:** Test `input_renders_label_and_field` fails after reordering.

### Pitfall 4: Missing `pr-10` on Select Causes Text Overflow
**What goes wrong:** Option text in a narrow select overlaps the SVG chevron.
**Why it happens:** The select's text content flows to the full width. The SVG is positioned absolutely over the right edge.
**How to avoid:** Add `pr-10` (2.5rem = 40px right padding) to the select's class string so text is forced left of the arrow area.
**Warning signs:** Option text visually overlaps the chevron in the rendered output.

### Pitfall 5: `focus-visible:ring-offset-2` Missing
**What goes wrong:** The focus ring appears to overlap the element's border without visual separation.
**Why it happens:** `ring-2` without `ring-offset` draws the ring as a direct box-shadow.
**How to avoid:** Add `focus-visible:ring-offset-2` to provide a 2px gap between element border and focus ring. This matches the button focus ring pattern planned for Phase 106 (`INT-01`: `focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2`).
**Warning signs:** Focus ring looks merged with the element border on light backgrounds.

## Code Examples

Verified patterns from codebase inspection and official Tailwind v4 documentation:

### FRM-01: Select with SVG Chevron Wrapper (render_select)
```rust
// Source: ferro-json-ui/src/render.rs — render_select(), replacing current select push

let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};

html.push_str("<div class=\"relative\">");
html.push_str(&format!(
    "<select id=\"{}\" name=\"{}\" class=\"block w-full appearance-none bg-background rounded-md border {} px-3 py-2 pr-10 text-sm shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
    html_escape(&props.field),
    html_escape(&props.field),
    border_class,
    focus_ring_class
));
if props.required == Some(true) { html.push_str(" required"); }
if props.disabled == Some(true) { html.push_str(" disabled"); }
html.push('>');
// ... options ...
html.push_str("</select>");
html.push_str(concat!(
    "<span class=\"pointer-events-none absolute inset-y-0 right-3 flex items-center\" aria-hidden=\"true\">",
    "<svg class=\"h-4 w-4 text-text-muted\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 20\" fill=\"currentColor\">",
    "<path fill-rule=\"evenodd\" d=\"M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z\" clip-rule=\"evenodd\"/>",
    "</svg></span>"
));
html.push_str("</div>");
```

### FRM-02/FRM-06: Input + Textarea with Error Focus Ring (render_input)
```rust
// Source: ferro-json-ui/src/render.rs — render_input(), regular input branch (line 753)

let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};

// Regular input:
html.push_str(&format!(
    "<input type=\"{}\" id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-sm shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
    input_type,
    html_escape(&props.field),
    html_escape(&props.field),
    border_class,
    focus_ring_class
));

// Textarea branch (line 723) — same pattern:
html.push_str(&format!(
    "<textarea id=\"{}\" name=\"{}\" class=\"block w-full rounded-md border {} px-3 py-2 text-sm shadow-sm transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed {}\"",
    html_escape(&props.field),
    html_escape(&props.field),
    border_class,
    focus_ring_class
));
```

### FRM-07: Corrected Field Order (render_input)
```rust
// Source: ferro-json-ui/src/render.rs — render_input()

let mut html = String::from("<div class=\"space-y-1\">");
// 1. Label
html.push_str(&format!(
    "<label class=\"block text-sm font-medium text-text\" for=\"{}\">{}</label>",
    html_escape(&props.field),
    html_escape(&props.label)
));

// 2. Input/Textarea (BEFORE description)
match props.input_type { ... }

// 3. Description (AFTER input — was before, now moved)
if let Some(ref desc) = props.description {
    html.push_str(&format!(
        "<p class=\"text-sm text-text-muted\">{}</p>",
        html_escape(desc)
    ));
}

// 4. Error (last)
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p class=\"text-sm text-destructive\">{}</p>",
        html_escape(error)
    ));
}
html.push_str("</div>");
```

### FRM-07: Corrected Field Order (render_select)
```rust
// Source: ferro-json-ui/src/render.rs — render_select()

// 1. Label
// 2. SVG wrapper div + select + SVG chevron (BEFORE description)
// 3. Description (AFTER select close wrapper)
if let Some(ref desc) = props.description {
    html.push_str(&format!(
        "<p class=\"text-sm text-text-muted\">{}</p>",
        html_escape(desc)
    ));
}
// 4. Error (last)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `focus:ring-1 focus:ring-primary` (always) | `focus-visible:ring-2 focus-visible:ring-destructive` in error state | Phase 105 | Error state is visually distinct; `focus-visible` avoids ring on mouse click |
| No transition on form elements | `transition-colors duration-150 motion-reduce:transition-none` | Phase 105 | Smooth hover/focus color changes; motion-safe |
| No disabled Tailwind variant; conditional Rust class strings | `disabled:opacity-50 disabled:cursor-not-allowed` via Tailwind variant | Phase 105 | Simpler Rust code — no conditional logic needed for visual disabled state |
| Native browser select arrow (removed by `appearance-none`) | Inline SVG chevron in wrapper div | Phase 105 | Consistent arrow across all browsers; no JS |
| label → description → input → error | label → input → description → error | Phase 105 | Matches UX convention: description acts as help text below the field |

**Retained patterns (no change):**
- `border-destructive` for error border (already present) — stays
- `appearance-none` on select (already present) — stays
- `bg-background` on select (already present) — stays
- Checkbox order (input+label → description → error) — already correct, no change

## Open Questions

1. **Should Checkbox and Switch also get FRM-03/FRM-04 treatment?**
   - What we know: Requirements FRM-03 and FRM-04 say "All form elements." Checkbox and Switch are form elements.
   - What's unclear: The success criteria mention "Input, Select, and Textarea" explicitly for error focus rings (FRM-02/05/06). FRM-03/04 say "All form elements" without naming specific components.
   - Recommendation: Apply `transition-colors duration-150 motion-reduce:transition-none` and `disabled:opacity-50 disabled:cursor-not-allowed` to Checkbox (line 905) and Switch too, for completeness. This is conservative — all form element classes should be consistent.

2. **`focus-visible:ring-offset-2` background color**
   - What we know: `ring-offset` uses the ring-offset-color, which defaults to white. On dark backgrounds, the white offset looks wrong.
   - What's unclear: The phase doesn't define `ring-offset-color`. In the button's Phase 106 pattern (INT-01), `ring-offset-2` is specified without color qualification.
   - Recommendation: Use `focus-visible:ring-offset-2` without explicit color. Token-aware ring-offset is a Phase 106+ concern. For form elements, `ring-offset-2` provides adequate visual separation on light backgrounds, which is the primary design scenario.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none (workspace-level Cargo.toml) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FRM-01 | Select wrapper renders inline SVG chevron | unit | `cargo test -p ferro-json-ui test_render_select_appearance_none` (extend) | existing test at line 5069 — add SVG assertions |
| FRM-01 | Select wrapper has `relative` class div | unit | new test `select_renders_chevron_wrapper` | Wave 0 gap |
| FRM-02 | Input in error state has `ring-destructive` in class | unit | `cargo test -p ferro-json-ui input_renders_error_with_red_border` (extend) | existing at line 3156 — add `ring-destructive` assertion |
| FRM-03 | Input has `transition-colors duration-150` class | unit | new test `input_renders_transition_classes` | Wave 0 gap |
| FRM-03 | Input has `motion-reduce:transition-none` class | unit | `cargo test -p ferro-json-ui` (via `input_renders_transition_classes`) | Wave 0 gap |
| FRM-04 | Input disabled renders with `disabled:opacity-50` class | unit | `cargo test -p ferro-json-ui` (verify via new test or extend existing) | Wave 0 gap |
| FRM-05 | Select in error state has `ring-destructive` | unit | `cargo test -p ferro-json-ui select_renders_error` (extend at line 3433) | existing — add `ring-destructive` assertion |
| FRM-06 | Textarea in error state has `ring-destructive` | unit | new test `textarea_renders_error_focus_ring` | Wave 0 gap |
| FRM-07 | Input: description comes after input in DOM | unit | new test `input_description_order` | Wave 0 gap |
| FRM-07 | Select: description comes after select in DOM | unit | new test `select_description_order` | Wave 0 gap |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
New test functions needed (add to `render.rs` test module, following existing patterns):

- [ ] `select_renders_chevron_wrapper` — asserts `html.contains("class=\"relative\"")` and `html.contains("aria-hidden=\"true\"")` and `html.contains("<svg")` — covers FRM-01
- [ ] `input_renders_transition_classes` — asserts `transition-colors`, `duration-150`, `motion-reduce:transition-none` — covers FRM-03
- [ ] `input_disabled_renders_disabled_classes` — asserts `disabled:opacity-50`, `disabled:cursor-not-allowed` — covers FRM-04
- [ ] `textarea_renders_error_focus_ring` — asserts `ring-destructive` when error set on `InputType::Textarea` — covers FRM-06
- [ ] `input_description_order` — asserts description `<p>` text appears AFTER the `<input` tag in HTML string — covers FRM-07 for Input
- [ ] `select_description_order` — asserts description `<p>` text appears AFTER `</select>` in HTML string — covers FRM-07 for Select

For order tests: use `html.find("<input")` vs `html.find("Your work email")` and assert index ordering.

Existing tests that need assertion updates (not new tests):
- `test_render_select_appearance_none` (line 5069): add assertions for SVG presence
- `input_renders_error_with_red_border` (line 3156): add `ring-destructive` assertion
- `select_renders_error` (line 3433): add `ring-destructive` assertion

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection: `ferro-json-ui/src/render.rs`
  - `render_input` at line 679: current class strings at lines 723 (textarea) and 753 (regular input)
  - `render_select` at line 802: current class string at line 834
  - `render_checkbox` at line 877: current class string at line 905
  - Current DOM ordering: label → description → input — confirmed at lines 696-710 (render_input) and 819-833 (render_select)
  - Existing tests at lines 3156, 3433, 5069 — confirmed no focus-ring-string assertions exist
  - Structural tests at lines 5336-5413: annotated "Phase 105 adds transitions, disabled states" and "Phase 105 adds custom arrow styling"
- Direct codebase inspection: `ferro-theme/assets/default.css`
  - `--color-destructive` token confirmed at line 18 — `ring-destructive` and `focus-visible:ring-destructive` are valid
  - No custom CSS needed for Tailwind variant utilities
- `.planning/REQUIREMENTS.md` "Out of Scope" table: confirms arbitrary Tailwind values are "Unverified in CDN mode" — SVG data URI arbitrary value is excluded

### Secondary (MEDIUM confidence)
- Tailwind CSS v4 documentation (tailwindcss.com/docs/background-image): confirms `bg-[url('data:image/svg+xml,...')]` requires special handling; arbitrary values with spaces/special chars unreliable in CDN scanner mode
- Tailwind CSS v4 CDN discussion (github.com/tailwindlabs/tailwindcss/discussions/15918): SVG backgrounds not working in Tailwind v4 CDN mode — corroborates inline SVG approach
- Modern CSS Solutions (moderncss.dev/custom-select-styles-with-pure-css/): wrapper div + inline SVG + appearance-none pattern for CSS-only custom select arrows

### Tertiary (LOW confidence)
- Chrome 135 `appearance: base-select` — new in 2026, allows full select styling without wrapper div. Explicitly out of scope for this phase (requires broader browser support investigation).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, all utilities are standard Tailwind v4 CDN
- Architecture (what to change): HIGH — all file locations and line numbers identified from direct code inspection
- SVG chevron approach: HIGH — inline SVG pattern confirmed as the only reliable CDN-mode approach
- Focus ring pattern: HIGH — `focus-visible:ring-2 ring-destructive` is standard Tailwind pattern, token exists
- FRM-07 DOM ordering impact: HIGH — confirmed no tests assert on ordering, only on content presence
- Wave 0 test gaps: HIGH — identified from test inventory; new test functions follow established patterns

**Research date:** 2026-03-25
**Valid until:** 2026-06-25 (render.rs structure is stable across this milestone)
