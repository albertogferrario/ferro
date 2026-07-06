# Phase 147: DetailForm component for inline edit — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-23
**Phase:** 147-detailform-component-for-inline-edit-ferro-json-ui
**Mode:** `--auto` — recommended defaults selected; no interactive Q&A
**Source of intent:** gestiscilo Phase 111 design session (2026-04-22), captured verbatim in pre-existing CONTEXT.md (preserved under Specific Ideas in the updated CONTEXT.md)

**Areas auto-resolved:**
1. EditMode enum location and API
2. DetailField / DetailFormProps shape
3. View-mode structural rendering
4. Edit-mode structural rendering (form wrapper)
5. Action / submit wiring
6. Button labels and default locale
7. `edit_url` / `cancel_url` — raw string vs resolvable Action
8. Data pre-fill ownership (caller vs component)
9. Runtime JS requirement
10. Resolver integration

---

## EditMode enum location and API

| Option | Description | Selected |
|--------|-------------|----------|
| EditMode in `ferro-json-ui` | Same crate as the DetailForm component; `from_query(Option<&str>) -> Self` | ✓ |
| EditMode in `framework` (http/) | Co-locate with `Request::query` — owned by the request layer | |
| EditMode in a new crate | New dedicated crate for view/edit modes — premature abstraction | |

**Auto-selection:** `ferro-json-ui`. Matches the spec ("EditMode lives in ferro") interpreted at the crate that owns the DetailForm component. Keeps the concept next to its consumer; no request-layer leakage.

---

## DetailField / DetailFormProps shape

| Option | Description | Selected |
|--------|-------------|----------|
| `DetailField { label, value, input: ComponentNode }` | Input is any component (Input/Select/Textarea/Switch/KeyValueEditor/etc.) | ✓ |
| `DetailField { label, value, input: InputProps }` | Only plain text `Input` component allowed as edit widget | |
| Separate `ViewField` + `EditField` types | Mode-specific types; caller picks one list per mode | |

**Auto-selection:** `ComponentNode`. Matches the spec (`input: Component::Input(...)`) and keeps the edit-mode vocabulary open-ended (any current or future form component, including Phase 146's KeyValueEditor, can be an edit widget).

---

## View-mode structural rendering

| Option | Description | Selected |
|--------|-------------|----------|
| `<dl>/<dt>/<dd>` mirror of DescriptionList | Reuse existing read-only structure; design-token consistency | ✓ |
| Custom grid layout | Hand-rolled 2-column grid specific to DetailForm | |
| Nested Card + inline rows | Wraps in `Card` for elevation — heavier, more opinionated | |

**Auto-selection:** `<dl>/<dt>/<dd>`. Matches `render_description_list` semantics and tokens, which delivers the cross-screen consistency principle (detail reads render the same way everywhere).

---

## Edit-mode structural rendering

| Option | Description | Selected |
|--------|-------------|----------|
| Same `<dl>` + `<form>` wrapper; swap `<dd>` leaves for inputs | Preserves outer scaffold; only leaves change | ✓ |
| Replace `<dl>` with a Form component in edit mode | View and Edit render different trees (what Phase 111 tried) | |
| Stack labels above inputs in flex column | Conventional form layout; different visual shape from View | |

**Auto-selection:** Same `<dl>` + `<form>` wrapper. This is the structural coherence guarantee that defines the phase — the thing Phase 111 could not deliver when branching at the controller level.

---

## Action / submit wiring

| Option | Description | Selected |
|--------|-------------|----------|
| `action: Action` required; resolver populates URL | Mirrors `FormProps.action`; participates in existing resolver pass | ✓ |
| `action: Option<Action>` | Edit mode without an action is a pre-filled readonly form — unclear use case | |
| Derive action from `edit_url` | Collapses two concepts; spec separates view-link from submit-target | |

**Auto-selection:** `action: Action` required. Parallels `FormProps` exactly.

---

## Button labels

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable props with Italian defaults | `edit_label: Option<String>` etc.; `None` → `"Modifica"` / `"Salva"` / `"Annulla"` | ✓ |
| Hardcoded Italian strings, no override | Matches gestiscilo origin; no per-caller customization | |
| English defaults | Ferro is international; Italian is the gestiscilo use case only | |
| `ferro-lang` bound keys | Wire to translation system for locale-aware labels | |

**Auto-selection:** Configurable with Italian defaults. Callers can override for English/other locales; gestiscilo uses defaults directly. `ferro-lang` integration is deferred (see Deferred Ideas).

---

## `edit_url` / `cancel_url` — raw string vs resolvable Action

| Option | Description | Selected |
|--------|-------------|----------|
| Raw `String` emitted verbatim | Matches spec (`"/prodotti/1?mode=edit"`); caller builds query params | ✓ |
| `Action::get` with resolver | Route-name-based; resolver populates URL | |
| Composite: `base_url: Action::get, query_params: BTreeMap` | Resolver + query-param merge | |

**Auto-selection:** Raw `String`. Matches spec and the fact that `?mode=edit` is caller-specific context. Over-abstracting links loses the spec's clarity for no real benefit.

---

## Data pre-fill ownership

| Option | Description | Selected |
|--------|-------------|----------|
| Each input's `default_value` / `data_path` carries its own pre-fill | Orthogonal to mode; same rules as standalone form fields | ✓ |
| DetailForm auto-populates each input's `default_value` from `DetailField.value` | Convenience, but couples `value` (a display string) with input state | |

**Auto-selection:** Caller-owned per-input pre-fill. Consistent with the project's "Form Field Rules" (every form field has a proper `default_value`, with `req.old().or_else()` restoration pattern). `DetailField.value` stays a pure display string for View mode.

---

## Runtime JS

| Option | Description | Selected |
|--------|-------------|----------|
| No JS — mode is a server-side query param | Pure HTML/CSS rendering; no client state | ✓ |
| Client-side toggle button | Replace `?mode=edit` with JS show/hide | |

**Auto-selection:** No JS. Matches spec ("No JS required for the toggle") and the general JSON-UI server-authoritative model.

---

## Resolver integration

| Option | Description | Selected |
|--------|-------------|----------|
| DetailForm participates in the resolver like `Component::Form` | Three arms in resolve.rs; resolves `props.action.url` and walks into `fields[i].input` | ✓ |
| Skip resolver integration | Caller hand-resolves action URLs before passing to render | |

**Auto-selection:** Participate in resolver. Required for `Action::new(handler).url` to be populated at render time — matches how every other action-bearing component works.

---

## Claude's Discretion

The following are not locked decisions — planner/executor picks during implementation:

- Exact Tailwind class lists on buttons and action bar (reuse idioms from `render_form` / existing button rendering)
- Whether the outer wrapper is `<section>` or `<div>` (pick whichever makes tests clearer)
- Whether "Modifica" sits above or below the `<dl>` in View mode (default below, right-aligned — matches Edit-mode action bar placement)
- Whether `DetailField::new(label, value, input)` convenience constructor is added
- Test coverage beyond the minimum (mode × substring assertions, serde round-trip, EditMode::from_query parsing)

---

## Deferred Ideas

Captured in CONTEXT.md under `<deferred>`. Summary:
- `ferro-lang` binding for default button labels
- Handler-based resolution for `edit_url` / `cancel_url`
- Per-field mode override (always-read-only fields)
- Conditional mode toggle visibility (`can_edit: bool`)
- Nested sections / groups within one DetailForm
- Form guards on DetailForm
- Gestiscilo Phase 111 migration itself (downstream work in the gestiscilo repo)
