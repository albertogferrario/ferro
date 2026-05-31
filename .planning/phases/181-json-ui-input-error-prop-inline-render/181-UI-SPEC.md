---
phase: 181
slug: json-ui-input-error-prop-inline-render
status: draft
shadcn_initialized: false
preset: none
created: 2026-05-31
---

# Phase 181 — UI Design Contract

> Visual and interaction contract for server-side HTML emission from `ferro-json-ui`.
> This phase emits no JavaScript and uses no client-side design system.
> All "UI" decisions are class strings produced by Rust code in `ferro-json-ui/src/render/form.rs`.

---

## Scope Note

This phase patches the JSON-UI resolution pipeline and applies error-state class parity to
`Checkbox`, `CheckboxList`, `Switch`, and `Input (file)`. The renderer for `Input (text/textarea/number/…)`
and `Select` already emits the correct visual treatment; those components are not changed in this phase.

The design system is `ferro-theme` — a set of semantic CSS custom properties defined in
`ferro-theme/assets/default.css`. No shadcn, no component library, no JavaScript.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (ferro-theme semantic tokens) |
| Preset | not applicable |
| Component library | none |
| Icon library | none |
| Font | Inter (--font-sans in ferro-theme/assets/default.css) |

Token file: `ferro-theme/assets/default.css`
Renderer: `ferro-json-ui/src/render/form.rs`

---

## Spacing Scale

The existing `space-y-1` wrapper is the canonical vertical rhythm for all form-control elements. This
phase does not introduce new spacing. The table below documents the tokens already in use by the
form renderer; no new values are added.

| Token | Value | Usage in form renderer |
|-------|-------|------------------------|
| space-y-1 | 4px gap | Outer `<div>` wrapper for every form control |
| gap-2 | 8px | Horizontal gap between checkbox `<input>` and its `<label>` |
| ml-6 | 24px | Description / error `<p>` indent inside Checkbox (matches label offset) |
| mt-1 | 4px | Error `<p>` top margin inside CheckboxList (existing) |
| px-3 py-2 | 12px / 8px | Input / Select control padding |

Exceptions: none introduced by this phase.

---

## Typography

All text sizes are sourced from the existing renderer. This phase does not add new typographic roles.

| Role | Tailwind class | Weight class | Notes |
|------|---------------|--------------|-------|
| Form label | `text-sm` (14px) | `font-medium` (500) | `block text-sm font-medium text-text` |
| Description | `text-sm` (14px) | normal (400) | `text-sm text-text-muted` |
| Error message | `text-sm` (14px) | normal (400) | `text-sm text-destructive` — locked DOM shape |
| Input / Select value | `text-base` (16px) | normal (400) | control body text |

Line heights: browser default for all roles (no explicit line-height override in existing renderer).

---

## Color

Semantic tokens from `ferro-theme/assets/default.css`. No new tokens are introduced.

| Role | CSS variable | Light value | Dark value | Usage |
|------|-------------|-------------|------------|-------|
| Surface | `--color-background` | oklch(100% 0 0) | oklch(12% 0 0) | Page background |
| Secondary | `--color-surface` | oklch(97% 0 0) | oklch(17% 0 0) | Card / form backgrounds |
| Accent (primary) | `--color-primary` | oklch(55% 0.2 250) | oklch(56% 0.2 250) | Focus rings, checked state |
| Border | `--color-border` | oklch(90% 0 0) | oklch(30% 0 0) | Default control border |
| Destructive | `--color-destructive` | oklch(55% 0.22 25) | oklch(59% 0.22 25) | Error borders, error text, error rings |
| Text muted | `--color-text-muted` | oklch(50% 0 0) | oklch(60% 0 0) | Description copy |

`--color-destructive` is reserved for: error borders on form controls, destructive focus rings,
error `<p>` text. No other usage in this phase.

Contrast note: `--color-destructive` (oklch 55% light / 59% dark) against `--color-background`
(oklch 100% / oklch 12%) satisfies WCAG AA (4.5:1) for normal text at 14px. Planner must verify
the dark-mode pair if the exact oklch values are adjusted.

---

## Error State Class-Chain Contract

This is the load-bearing section for this phase. The planner and executor reference these exact
strings when extending error-state styling to `Checkbox`, `CheckboxList`, `Switch`, and `Input (file)`.

### Canonical baseline (Input text/textarea and Select — already shipped, do not change)

**Border swap:**
```
border-border       →  border-destructive        (when has_error)
```

**Focus ring swap:**
```
focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2
  →
focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2
                                                                                (when has_error)
```

**Error paragraph (emitted after description `<p>` if any):**
```html
<p id="err-{field}" class="text-sm text-destructive">{error}</p>
```
Note: `id="err-{field}"` is required for the ARIA `aria-describedby` pairing. All strings
interpolated into HTML pass through `html_escape`.

**ARIA attributes on the control element (when has_error):**
```
aria-invalid="true" aria-describedby="err-{field}"
```

### D-06 parity targets — class-chain specification for each component

#### Checkbox (`render_checkbox`)

Current class on `<input type="checkbox">`:
```
h-4 w-4 rounded-sm border-border text-primary transition-colors duration-150
motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed
focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2
```

Required change when `has_error`:
- Swap `border-border` → `border-destructive`
- Swap `focus-visible:ring-primary` → `focus-visible:ring-destructive`
- Add `aria-invalid="true" aria-describedby="err-{field}"` on the `<input>`

Error paragraph location: inside the outer `<div class="space-y-1">`, after the inner flex
`<div>` (after any description `<p>`), with `ml-6` indent to align with label text. Current
emission at form.rs:485-490 already uses `ml-6` but lacks the `id` attribute needed for
`aria-describedby`. Add `id="err-{field}"` to the error `<p>`.

Full post-fix error `<p>`:
```html
<p id="err-{field}" class="ml-6 text-sm text-destructive">{error}</p>
```

#### CheckboxList (`render_checkbox_list`)

Current: no border-destructive swap, no focus-ring swap, no ARIA attributes on individual
checkboxes, error `<p>` at form.rs:582-587 uses `class="text-sm text-destructive mt-1"` but
lacks `id`.

Required change when `has_error`:
- Add `id="err-{field}"` to the error `<p>` (already outside the `<fieldset>`).
- Add `aria-describedby="err-{field}"` to the `<fieldset>` element (use `aria-describedby`
  on the `<fieldset>`, not on each individual `<input>`, since this is a group).
- Add `aria-invalid="true"` to the `<fieldset>`.
- Border swap on individual checkboxes within the group: swap `border-border` →
  `border-destructive` on each `<input type="checkbox">` inside the list.

Full post-fix fieldset open tag when `has_error`:
```html
<fieldset class="space-y-2" aria-invalid="true" aria-describedby="err-{field}">
```

Full post-fix error `<p>`:
```html
<p id="err-{field}" class="text-sm text-destructive mt-1">{error}</p>
```

#### Switch (`render_switch`)

Current: no ring swap, no ARIA, error `<p>` at form.rs:705-710 uses `class="text-sm text-destructive"` but lacks `id`.

The hidden checkbox has `role="switch"` and `class="sr-only peer"`. The visible toggle pill
gets its ring via `peer-focus:ring-2 peer-focus:ring-primary/30` on the `<div>` pill element.

Required change when `has_error`:
- On the `<div>` pill: swap `peer-focus:ring-primary/30` → `peer-focus:ring-destructive/30`.
- Add `aria-invalid="true"` and `aria-describedby="err-{field}"` on the hidden `<input>`.
- Add `id="err-{field}"` to the error `<p>`.

Current pill class:
```
w-11 h-6 bg-border rounded-full peer peer-checked:bg-primary
peer-focus:ring-2 peer-focus:ring-primary/30
after:content-[''] after:absolute after:top-0.5 after:left-[2px]
after:bg-background after:rounded-full after:h-5 after:w-5 after:transition-all
peer-checked:after:translate-x-full
```

Post-fix pill class when `has_error` (only `peer-focus:ring-primary/30` changes):
```
w-11 h-6 bg-border rounded-full peer peer-checked:bg-primary
peer-focus:ring-2 peer-focus:ring-destructive/30
after:content-[''] after:absolute after:top-0.5 after:left-[2px]
after:bg-background after:rounded-full after:h-5 after:w-5 after:transition-all
peer-checked:after:translate-x-full
```

Full post-fix error `<p>`:
```html
<p id="err-{field}" class="text-sm text-destructive">{error}</p>
```

#### Input (file) (`render_input`, `InputType::File` branch)

Current: no border swap, no focus ring, no ARIA, error `<p>` emitted at form.rs:309-315
(shared with all non-hidden Input variants) but `aria-invalid` / `aria-describedby` not
added in the File branch (form.rs:221-237 does not include the `has_error` ARIA block).

Required change when `has_error`:
- Add a destructive ring to the `<input type="file">` wrapper: insert
  `ring-1 ring-destructive` on the `<input>` element when `has_error`.
  (File inputs have no border that can be swapped cleanly; ring is the correct visual hook.)
- Add `aria-invalid="true" aria-describedby="err-{field}"` to the `<input type="file">`.
- The shared error `<p>` block at form.rs:309-315 already includes `id="err-{field}"`.
  No change needed there.

Post-fix addition to the `<input type="file">` class when `has_error`:
```
ring-1 ring-destructive
```
Full class string (has_error = true):
```
block w-full text-sm text-text
file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0
file:text-sm file:font-medium file:bg-surface file:text-text
hover:file:bg-surface/80
ring-1 ring-destructive
```

---

## DOM Shape Summary

Every form control with an error follows this DOM structure:

```html
<div class="space-y-1">
  <label class="block text-sm font-medium text-text" for="{field}">{label}</label>
  <!-- control element with aria-invalid="true" aria-describedby="err-{field}" when has_error -->
  <p class="text-sm text-text-muted">{description}</p>   <!-- if description present -->
  <p id="err-{field}" class="text-sm text-destructive">{error}</p>   <!-- if error present -->
</div>
```

CheckboxList uses `<fieldset>` as outer element instead of `<div>`.
Checkbox error `<p>` includes `ml-6` for indent alignment.
Switch: error `<p>` is inside the `<div class="space-y-1">` after the toggle block.

---

## Copywriting Contract

This phase has no user-facing copy changes. Error messages are authored by the application
developer (passed as strings through the validation pipeline). The framework emits them
verbatim after `html_escape`.

| Element | Contract |
|---------|----------|
| Error message voice | Application-supplied string, no framework-default copy |
| Error paragraph aria label | None (content is the visible text; no separate aria-label needed) |
| Diagnostic HTML comment | `<!-- ferro-json-ui: failed to decode {Component} props: {err} -->` — existing pattern, not changed |

Out of scope: toast copy, empty-state copy, CTA labels. This phase adds no new page-level UI elements.

---

## Accessibility Contract

| Requirement | Implementation |
|-------------|----------------|
| Error indication | `aria-invalid="true"` on the control when `has_error` |
| Error association | `aria-describedby="err-{field}"` on the control, `id="err-{field}"` on the error `<p>` |
| Checkbox group error | `aria-invalid="true"` + `aria-describedby="err-{field}"` on `<fieldset>` |
| Focus visibility | `focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2` on keyboard focus when `has_error` |
| Color contrast | `text-destructive` (oklch 55% light / 59% dark) against page background passes WCAG AA for 14px text |
| Motion | `motion-reduce:transition-none` preserved on all controls (existing, not changed) |

WCAG targets: 2.1 AA minimum. Color is not the sole error indicator — border color change and
`aria-invalid` together satisfy WCAG 1.4.1 (Use of Color).

---

## Out of Scope

| Item | Reason |
|------|--------|
| Client-side validation | Banned by PROJECT.md "Hard cap on expression language" |
| Animation / transition on error appearance | Not in D-06; server-rendered HTML has no lifecycle hooks |
| New semantic color tokens | This phase reuses existing tokens only |
| Multi-error per field | Deferred — see CONTEXT.md Deferred Ideas |
| Toast component rework | Deferred — see CONTEXT.md D-05 note |
| JSON-UI catalog schema changes | Out of scope for Phase 181 |

---

## Registry Safety

Not applicable. This phase uses no third-party component registry. All components are hand-authored
Rust emitting raw HTML strings.

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| none | n/a | not applicable |

---

## Pre-Populated From

| Source | Decisions used |
|--------|---------------|
| CONTEXT.md D-01 | Bug is in pipeline, not renderer — renderer class chains are the canonical baseline |
| CONTEXT.md D-06 | Locked parity targets: Checkbox / CheckboxList / Switch / Input-file |
| form.rs:174-184 | Exact border + focus-ring swap class strings |
| form.rs:309-315 | Locked error `<p>` DOM shape |
| form.rs:213-218, 277-282 | Locked ARIA pairing pattern (aria-invalid + aria-describedby) |
| form.rs:456-460 | Current Checkbox class chain (pre-fix baseline) |
| form.rs:684-701 | Current Switch pill class chain (pre-fix baseline) |
| ferro-theme/assets/default.css | All color token values |
| CONTEXT.md Deferred Ideas | Client-side validation and multi-error out of scope |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS (no application copy in scope; framework diagnostic strings unchanged)
- [ ] Dimension 2 Visuals: PASS (class-chain parity specified for all 4 components)
- [ ] Dimension 3 Color: PASS (destructive token reserved for error states only; existing tokens reused)
- [ ] Dimension 4 Typography: PASS (text-sm / font-medium / text-destructive — existing scale)
- [ ] Dimension 5 Spacing: PASS (space-y-1 / ml-6 / mt-1 — existing scale, no new values)
- [ ] Dimension 6 Registry Safety: PASS (no third-party registry)

**Approval:** pending
