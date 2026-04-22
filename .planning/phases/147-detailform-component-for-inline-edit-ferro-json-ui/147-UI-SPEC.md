---
phase: 147
title: DetailForm component for inline edit — ferro-json-ui
status: draft
mode: auto
generated: 2026-04-23
precedent: 146-UI-SPEC.md
---

# 147-UI-SPEC.md — DetailForm

> Note (auto-mode): produced by `gsd-ui-researcher` under `--auto`.
> No interactive questions were asked. Every decision below is either
> (a) inherited from `147-CONTEXT.md` (D-01..D-20), (b) inherited from
> `147-RESEARCH.md`, (c) inherited from the `146-UI-SPEC.md` precedent
> (KeyValueEditor), or (d) auto-selected with rationale.
> Auto-selected items are tagged `[auto]`.

## 0. Scope of this contract

`DetailForm` is a **server-side Rust HTML emitter** in `ferro-json-ui`.
It does not ship JavaScript, does not own state, and does not introduce
a new design system. It composes three already-rendered surfaces:

- `render_description_list` (`<dl>/<dt>/<dd>` block)
- `render_form` (`<form>` wrapper + action bar)
- `render_input` (per-field `<label>` + control)

The "UI" of this phase is therefore (1) the **outer scaffold contract**
that View and Edit modes must share, (2) the **action-bar contract**
the component owns, (3) the **token vocabulary** carried over from
existing renderers, and (4) one **explicit author-facing rule** about
label authoring (Option A from `147-RESEARCH.md`).

Anything that would expand the design system, introduce client JS, or
deviate from existing token vocabulary is out of scope and belongs in
a separate phase.

## 1. Design system

| Item                | Value                                                                  | Source                  |
|---------------------|------------------------------------------------------------------------|-------------------------|
| Tool                | ferro-json-ui Tailwind v4 semantic tokens                              | precedent (146)         |
| Shadcn              | not applicable (server-side Rust HTML; no JS component lib)            | precedent (146)         |
| Icon library        | inline SVG via `currentColor` if/when icons are needed                 | precedent (146) `[auto]`|
| Font                | inherited from page/theme — DetailForm declares no font tokens         | precedent (146)         |
| Theme bridge        | semantic Tailwind classes only — no hex, no raw palette references     | ferro-theme convention  |

**No new tokens are introduced by phase 147.** Every class emitted by
`DetailForm` already appears in `render_form`, `render_description_list`,
or `render_input` today.

## 2. Spacing scale

Inherits the existing `render_description_list` and `render_form` scale.
DetailForm authors no new spacing.

| Token         | Where                                  |
|---------------|----------------------------------------|
| `gap-4`       | between `<dl>` rows (View and Edit)    |
| `mt-1`        | `<dd>` to its `<dt>`                   |
| `mt-6`        | action bar to last `<dd>` row `[auto]` |
| `gap-2`       | between action buttons in action bar   |

`mt-6` is `[auto]`-selected to match the trailing-area separation
already used by `render_form`'s submit row. If the implementer finds
`render_form` uses a different value at the insertion point, the
implementation MUST adopt that value verbatim — DetailForm follows
`render_form`, never overrides it.

## 3. Typography

Inherits both renderers verbatim. No new sizes or weights.

| Element             | Class                                      | Source                       |
|---------------------|--------------------------------------------|------------------------------|
| `<dt>` (label)      | `text-sm font-medium text-text-muted`      | `render_description_list`    |
| `<dd>` text (View)  | `text-sm text-text`                        | `render_description_list`    |
| `<input>` (Edit)    | as emitted by `render_input`               | `render_input` (unchanged)   |
| Action button label | as emitted by existing button styling      | `render_form` action area    |

DetailForm declares **no** font-size, font-weight, or line-height
overrides. Any deviation from the description-list/form typography
would break the structural-coherence contract in §5.

## 4. Color contract

Inherits the ferro-json-ui semantic-token contract. DetailForm uses
only the tokens listed below; it MUST NOT introduce new ones.

| Token                  | Use                                                       |
|------------------------|-----------------------------------------------------------|
| `bg-background`        | page surface (DetailForm does not set this; inherits)     |
| `bg-card` / `bg-surface` | container surfaces (inherits from caller)               |
| `text-text`            | primary value text (View `<dd>`)                          |
| `text-text-muted`      | label text (`<dt>`) + secondary actions ("Modifica", "Annulla") |
| `border-border`        | input borders (via `render_input`)                        |
| `border-destructive`   | input error state (via `render_input`)                    |
| `bg-primary`           | "Salva" submit button background                          |
| `text-primary-foreground` | "Salva" submit button label                            |

**Accent reservation list (the 10%):** in the DetailForm surface, the
primary accent is reserved exclusively for the **"Salva" submit
button** in Edit mode. Nothing else in the component uses
`bg-primary` / `text-primary-foreground`. "Modifica" (View mode entry)
and "Annulla" (Edit mode exit) render as outline/link-styled controls
using `text-text-muted` + `border-border`, matching the secondary-action
treatment already used by `render_form`.

**Destructive token:** Phase 147 emits no destructive UI. Delete /
discard / reset actions are explicitly out of scope (see CONTEXT D-?,
RESEARCH out-of-scope list). If a future phase adds a "Reset" or
"Delete" action inside DetailForm, it inherits `border-destructive` /
`bg-destructive` from `render_input` / `render_form` — no new token
work required.

## 5. Structural coherence contract (the killer constraint)

This is the single load-bearing visual rule of phase 147 and the
reason the component exists.

> **The outer scaffold of DetailForm is byte-for-byte the same in
> View mode and Edit mode, except for: (a) an outer `<form>` wrapper
> in Edit mode, (b) the contents of each `<dd>`, and (c) the action
> bar's button set.**

Concretely:

- The `<dl>` element, its classes, and its row order are identical.
- Every `<dt>` is identical in both modes (same text, same classes).
- Each `<dd>` retains its outer tag and classes; only its **inner
  content** swaps:
  - View: plain text value (or formatted scalar)
  - Edit: a rendered input `ComponentNode` (textbox, select, textarea, etc.)
- The action bar is positioned identically in both modes; only its
  button set differs.

The checker MUST be able to diff the rendered HTML of View and Edit
mode for the same record and observe that the only structural
differences are:

1. Presence of `<form …>` / `</form>` wrapping the `<dl>` and action bar.
2. `<dd>` inner content (text vs `<label></label><input …>` etc.).
3. Action bar contents (see §7).

Any other structural drift is a contract violation.

## 6. Mode model

| Mode | Trigger                              | Outer wrapper                  | `<dd>` content    | Action bar     |
|------|--------------------------------------|--------------------------------|-------------------|----------------|
| View | default (no `?mode=edit`)            | none — bare `<dl>`             | text              | "Modifica"     |
| Edit | URL has `?mode=edit`                 | `<form method="post" action=…>`| input ComponentNode| "Salva" + "Annulla" |

- Mode is **URL-driven**; no client JS, no toggle state, no
  localStorage. Refreshing the page reproduces the same mode.
- "Modifica" is a plain `<a>` link to `?mode=edit` (same path).
- "Annulla" is a plain `<a>` link back to the canonical (no-query)
  view URL.
- "Salva" is the form's submit control.
- The form `action` and `method` are properties of the surrounding
  caller, not of DetailForm itself; DetailForm receives them as props.

## 7. Action-bar contract

The action bar sits **below** the `<dl>`, separated by `mt-6`
(see §2 caveat), inside the `<form>` in Edit mode and outside the
`<dl>` in View mode.

| Mode | Buttons (left → right)                            | Alignment              |
|------|---------------------------------------------------|------------------------|
| View | "Modifica"                                        | right-aligned `[auto]` |
| Edit | "Annulla"  ·  "Salva"                             | right-aligned `[auto]` |

Right-alignment matches the existing `render_form` submit-row
convention (`flex justify-end gap-2`) and is `[auto]`-selected on that
basis. Implementer MUST adopt whatever alignment `render_form` already
emits at the chosen insertion point.

**Button styling:**

- "Salva" — primary submit button styling already used by
  `render_form` (`bg-primary text-primary-foreground` + existing
  padding/radius tokens).
- "Modifica" and "Annulla" — secondary/link styling
  (`text-text-muted`, optional `border border-border`) already used by
  secondary actions in `render_form`. **No accent color, no
  destructive color.**

Authors MUST NOT pass arbitrary action buttons through DetailForm in
v1 — the action bar is owned by the component to preserve the
mode-flip contract. Custom auxiliary actions are deferred to a later
phase.

## 8. Copywriting contract

Default labels (Italian, matching ferro/app conventions established in
the ferro sample app and phase 146 precedent):

| Element                       | Label       | Notes                                  |
|-------------------------------|-------------|----------------------------------------|
| View entry button             | `Modifica`  | verb only — already implies the record |
| Edit submit button            | `Salva`     | primary CTA                            |
| Edit cancel button            | `Annulla`   | secondary, exits edit mode             |
| Empty state (no fields)       | n/a         | DetailForm renders nothing — caller's responsibility `[auto]` |
| Error state (validation fail) | inherited from `render_input` per-field error display | no DetailForm-level error banner `[auto]` |
| Destructive actions           | none in v1  | out of scope (see §4 destructive note) |

Labels MUST be overridable by the caller (props), but defaults
above are what `DetailForm::default()` emits. Locale routing through
`ferro-lang` is out of scope for this phase; labels ship as Italian
literals to match the existing ferro-json-ui surface.

`[auto]` rationale on empty/error:
- **Empty:** A DetailForm with zero fields is a programming error,
  not a runtime user state — the caller chose to render the
  component. We do not add an empty-state UI because doing so would
  legitimize an empty contract. `render_description_list` already
  handles the zero-row case identically.
- **Error:** Validation errors are owned by `render_input` per-field
  (red border + message). DetailForm does not add a top-level error
  banner because that would create a second source of truth for
  validation feedback and break the structural-coherence contract
  with View mode.

## 9. The label-duplication contract (Option A from 147-RESEARCH.md)

This is the single author-facing rule that phase 147 introduces.

**Problem.** In Edit mode, each `<dd>` contains a `ComponentNode`
rendered by `render_input`, which **always emits a `<label>` tag**.
The `<dt>` already provides the visible field label. Without
intervention, every Edit-mode field would display two labels.

**Resolution (Option A).** When DetailForm constructs / accepts an
input `ComponentNode` for a field, it MUST set the input's `label`
prop to the empty string `""`. `render_input` then emits
`<label></label>` (a zero-content label tag) which is semantically
inert and visually invisible. The `<dt>` is the sole visible label.

**Authoring rule (documented contract).**

> When a caller passes a manually-constructed `DetailField` whose
> `input` is a `ComponentNode`, the caller MUST set
> `input.props.label = ""`. Authors who pass a non-empty label will
> see a duplicated label in Edit mode. This is intentional: DetailForm
> does not silently mutate caller-supplied props.

The component MUST document this rule in its rustdoc and the
generated `json_ui_catalog` description, so that `ferro-mcp`-driven
agents discover it via introspection.

**Why not Option B (suppress label tag in `render_input`).**
Suppressing the `<label>` element would change `render_input`'s
contract for every other caller and break accessibility outside the
DetailForm context. Option A is local, opt-in, and auditable.

## 10. Component inventory

Phase 147 adds **one** component to `ferro-json-ui`.

| Component   | Variant set | Composes                                     |
|-------------|-------------|----------------------------------------------|
| DetailForm  | View, Edit  | `render_description_list` + `render_form` + `render_input` |

It ships:

- `DetailFormProps` (struct) with at minimum: `mode: Mode`, `fields: Vec<DetailField>`, `action: String`, `method: HttpMethod`, optional `labels: ActionLabels`.
- `DetailField` (struct) with: `label: String` (the `<dt>` text), `view_value: String` (the `<dd>` text in View mode), `input: Option<ComponentNode>` (the `<dd>` content in Edit mode).
- `Mode` (enum): `View`, `Edit`.
- A `Renderer` impl emitting the HTML described in §5–§7.

No new variants beyond View/Edit. No size variants. No density variants.
Opinionated and minimal by design.

## 11. Interaction states

DetailForm has only the states inherited from its substrate. It adds none.

| State          | Behaviour                                                     |
|----------------|---------------------------------------------------------------|
| Loading        | n/a — server-rendered                                         |
| Hover (button) | inherited from existing button styling in `render_form`       |
| Focus (input)  | inherited from `render_input`                                 |
| Validation err | inherited per-field from `render_input`; no component-level banner |
| Disabled       | inherited from `render_input` per field; no component-level disable |

Keyboard:

- Tab order follows DOM order — `<dt>` is non-focusable, `<dd>`'s
  `<input>` is focusable; the action-bar buttons follow the last field.
- `Enter` inside a text input submits the form (default browser
  behaviour, intentionally preserved).
- "Modifica" / "Annulla" are `<a>` elements — activated by `Enter`
  per browser default.

Accessibility:

- View mode is a semantic `<dl>` — already accessible without ARIA.
- Edit mode wraps the same `<dl>` in a `<form>`. Inputs are
  associated to the `<dt>` visually; the empty `<label>` (Option A)
  is intentional but means each input MUST also carry an `aria-label`
  derived from the `<dt>` text. **`DetailForm` MUST set
  `aria-label` on each input ComponentNode** equal to the field's
  `label: String` value when applying the `label = ""` rule from §9.
  This is a hard requirement — without it, screen readers would lose
  the field name.

## 12. Registry

Not applicable (no shadcn registry; no third-party block consumption).
Safety-gate: not applicable.

## 13. Out of scope (explicit fences)

To prevent scope creep during planning/execution:

- Client-side mode toggle (no JS).
- Optimistic updates / partial saves.
- Per-field inline edit (only whole-form Edit mode).
- Custom action buttons beyond Modifica/Salva/Annulla.
- Top-level error banner.
- Empty-state UI.
- Locale routing through `ferro-lang`.
- Destructive actions (Delete / Reset).
- Density / size variants.
- A "diff against original" view.

Each of these is a defensible follow-up phase but would dilute the
structural-coherence contract in §5 if added now.

## 14. Acceptance for the UI checker

A `gsd-ui-checker` pass requires all of:

1. The View-mode HTML and Edit-mode HTML, diffed for the same record,
   differ ONLY in the three ways listed in §5.
2. Every emitted class appears in either `render_form`,
   `render_description_list`, or `render_input` today — no new
   class names introduced.
3. The "Salva" button is the only element using
   `bg-primary` / `text-primary-foreground`.
4. Each Edit-mode `<input>` has both `<label></label>` (empty) and
   a non-empty `aria-label` matching the `<dt>` text.
5. Default labels are `Modifica` / `Salva` / `Annulla`.
6. Action bar is right-aligned and uses the same wrapper classes as
   `render_form`'s submit row.
7. Component rustdoc includes the §9 author-facing rule verbatim.
8. `json_ui_catalog` description (introspectable via `ferro-mcp`)
   restates the §9 rule.

## 15. Pre-population provenance

| Source                  | Decisions used                                           |
|-------------------------|----------------------------------------------------------|
| 147-CONTEXT.md          | mode model, action-bar buttons, scope fences, copy defaults |
| 147-RESEARCH.md         | Option A label-duplication resolution, insertion points, token names |
| 146-UI-SPEC.md          | structural template, voice, token vocabulary, registry posture |
| ferro-json-ui source    | exact class names from `render_form`, `render_description_list`, `render_input` |
| Auto-selected `[auto]`  | icon-library posture, action-bar alignment, `mt-6` separation, empty/error posture |

No user input was solicited in this auto-mode pass. Every `[auto]`
above is reversible by editing this file before the checker runs.
