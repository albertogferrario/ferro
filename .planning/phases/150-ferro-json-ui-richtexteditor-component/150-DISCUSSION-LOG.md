# Phase 150: ferro-json-ui RichTextEditor component — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 150-ferro-json-ui-richtexteditor-component
**Mode:** `--auto` (single pass, recommended defaults)
**Areas discussed:** Architectural placement, Props structure, CDN asset injection, Hidden-field naming, Initial value handling, Formats whitelist enforcement, Sanitization mechanism, Theme support, Default formats, Toolbar configuration, Runtime module, Submit interception, MCP catalog, Public surface, Validation & a11y

---

## Architectural placement

| Option | Description | Selected |
|--------|-------------|----------|
| First-class `Component::RichTextEditor` variant | Add to the `Component` enum like KeyValueEditor (Phase 146); reuse plugin asset infrastructure for CDN | ✓ |
| Plugin via `JsonUiPlugin` trait | Register at startup like `MapPlugin`; gets free CDN/SRI asset injection but is not in the typed `Component` enum | |
| Hybrid: synthetic plugin under the hood, exposed as `Component::RichTextEditor` | Best of both, but introduces an internal-only abstraction | |

**Selected reasoning:** Locked by SC-1 ("exists in the ferro-json-ui component catalog ... compile-enforced via the existing component derive"). The cross-cutting refactor is reusing the plugin Asset/dedup pipeline for first-class components — recorded as D-02.

---

## CDN asset injection mechanism for first-class components

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse `Asset` struct + plugin asset-emission pipeline | Add an idempotent emission hook in `render_rich_text_editor` (or via a synthetic registered-at-startup plugin); deduplicate by URL | ✓ |
| Introduce a parallel CDN-emission code path for first-class components | New mechanism alongside the plugin one — fast, but creates two code paths | |
| Inline the Quill `<script>`/`<link>` tags into the renderer with no dedup | Simplest, but emits Quill once per editor instance — bad for pages with multiple editors | |

**Selected reasoning:** Conceptual coherence (PROJECT.md §"Continuous conceptual coherence") — the surface evolves to absorb both plugins and first-class components rather than growing parallel paths. Mechanism choice (synthetic plugin vs explicit hook) is left to Claude's discretion in D-02.

---

## Hidden-field naming on submit

| Option | Description | Selected |
|--------|-------------|----------|
| `{name}_delta` + `{name}_html` (two hidden inputs, underscore-separated) | Locked by SC-3 | ✓ |
| Single hidden input with combined JSON envelope | Cleaner, but diverges from SC | |
| Bracket notation `{name}[delta]` / `{name}[html]` | Rails-style nested params; Ferro doesn't currently parse this shape | |

**Selected reasoning:** SC-3 is explicit. Recorded as D-05 / D-06.

---

## Initial value handling

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-detect Delta vs HTML at runtime | `JSON.parse` succeeds and result has `ops` → Delta; else HTML | ✓ |
| Require explicit `value_format: enum { Delta, Html }` prop | More explicit; expands the prop surface | |
| Always Delta (require caller conversion if they have HTML) | Simpler runtime; harsher caller ergonomics | |

**Selected reasoning:** Quill 2.x's built-in clipboard module already filters HTML input by formats, so accepting either is safe. Auto-detect collapses the decision to zero caller burden. Recorded as D-12.

---

## Formats whitelist enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| Enforce at BOTH init (Quill `formats` option) AND submit (HTML allowlist post-process) | Locked by SC-4 ("enforced both ... consumer cannot bypass by mutating the DOM") | ✓ |
| Init-only | Trusts Quill's runtime filter; vulnerable to DOM-mutation attacks | |
| Submit-only | Editor lets users type disallowed formats then strips on save — confusing UX | |

**Selected reasoning:** SC-4 is explicit. Recorded as D-15.

---

## HTML sanitization mechanism (submit-time)

| Option | Description | Selected |
|--------|-------------|----------|
| Hand-rolled allowlist tree-walker in the IIFE (DOMParser + walk) | Small, scoped, no extra dep | ✓ |
| External JS sanitizer (DOMPurify) loaded as another CDN asset | Battle-tested, but adds another CDN dep + SRI | |
| Server-side Rust sanitizer (e.g. `ammonia`) | Defense-in-depth, but pulls heavy dep into ferro-json-ui for one component | |

**Selected reasoning:** Allowlist is small (six default formats, max ~12 if extended); handwritten walker is well-understood and doesn't add SRI surface area or Rust deps. Server-side defense-in-depth is captured as a deferred idea. Recorded as D-17.

---

## Theme support

| Option | Description | Selected |
|--------|-------------|----------|
| Snow only (other values fall back to Snow silently) | SC-1 default; Bubble has different DOM semantics that would diverge the IIFE | ✓ |
| Snow + Bubble at v1 | Doubles the IIFE complexity for marginal value | |
| Custom theme via CSS token override | Out of scope; needs a separate token-mapping design | |

**Selected reasoning:** Recorded as D-03 / deferred ideas.

---

## Default formats list

| Option | Description | Selected |
|--------|-------------|----------|
| `["bold", "italic", "underline", "list", "header", "link"]` (six entries, mapped from SC-1) | Quill's native format names; `list` covers ordered+bullet | ✓ |
| Add `strike`, `blockquote`, `code-block` to the default | Slightly richer default; not in SC-1's list | |
| Empty default — caller must specify | Explicit, but every consumer would copy-paste the same list | |

**Selected reasoning:** SC-1 says "defaults to bold/italic/underline/lists/headings/links" — six entries match exactly. Recorded as D-18.

---

## Toolbar configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-derived from `formats` via internal mapping (single source of truth) | One prop drives both toolbar and allowlist; no caller-side toolbar config | ✓ |
| Separate `toolbar: Vec<...>` prop alongside `formats` | More flexible; doubles the API surface and risks divergence | |
| Custom toolbar via raw JSON injection | Most flexible; loses type safety | |

**Selected reasoning:** Single source of truth (D-15 reasoning) extends to toolbar config. Recorded as D-19. Custom-toolbar override is deferred.

---

## Runtime module location

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro-json-ui/src/runtime/rich_text_editor.rs` | Mirrors `runtime/key_value_editor.rs` exactly | ✓ |
| `ferro-json-ui/src/components/rich_text_editor/mod.rs` (new pattern) | Group-by-component instead of group-by-concern; would require restructuring the rest of the runtime | |

**Selected reasoning:** Group-by-concern is the established pattern (form_guards, kanban, dropdowns, modals, sse, tabs all live in `runtime/`). Recorded as D-20.

---

## Submit interception

| Option | Description | Selected |
|--------|-------------|----------|
| IIFE finds enclosing `<form>` via `closest('form')`, attaches submit listener (capture phase) | Works for any form layout; multiple editors per form supported independently | ✓ |
| Caller wires the editor explicitly to a specific form ID | More explicit but pushes work to every consumer | |
| Use `MutationObserver` to track DOM placement | Overkill for this use case | |

**Selected reasoning:** Every other ferro-json-ui form component assumes editor-inside-`<form>` placement; matching that convention. Recorded as D-13.

---

## MCP catalog integration

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-derive from `JsonSchema` if ferro-mcp's catalog supports it; else manual entry | Path of least resistance; planner determines which during discovery | ✓ |
| Manual entry only | Explicit, but duplicates type info already in `JsonSchema` | |

**Selected reasoning:** SC-6 / SC-7 require the entry to surface; mechanism (auto vs manual) is an implementation detail the planner resolves. Recorded as D-23.

---

## `JsonSchema` derive on `RichTextEditorProps`

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — props are flat (String / Option<String> / Vec<String> / Option<bool>); no Action or ComponentNode children | Matches `KeyValueEditorProps`; powers MCP auto-derive | ✓ |
| No — match `FormProps` / `DetailFormProps` precedent | Those skip JsonSchema only because they contain Action / ComponentNode (which have custom serde) — RichTextEditor has neither | |

**Selected reasoning:** Recorded as D-04.

---

## Validation & required-field handling

| Option | Description | Selected |
|--------|-------------|----------|
| `required: Option<bool>` with IIFE-installed empty-check that prevents submit and surfaces the destructive error region | Matches HTML5 form-validation UX; works with full-page POST | ✓ |
| Server-side validation only (no IIFE check) | Round-trip cost on every empty submit; worse UX | |
| HTML5 `required` attribute on a hidden input that mirrors editor state | Works in some browsers but not all (`required` on hidden inputs is implementation-dependent) | |

**Selected reasoning:** Recorded as D-29. Localized error message ("Required") is English literal in v1; i18n is deferred (matches Phase 147 / D-13 precedent).

---

## Claude's Discretion

The following were noted as deferred-to-implementation rather than locked:
- Exact Tailwind class lists for the editor wrapper, label, and error block (follow `render_input` / `render_key_value_editor` semantic-token conventions)
- Whether to scope Quill asset constants in `render.rs` next to `render_rich_text_editor` or in a new `assets/quill.rs` module
- Whether to add a headless-DOM integration test for hidden-input naming on top of the substring-assertion tests
- Exact mechanism for D-02 (synthetic-plugin-under-the-hood vs explicit asset hook in the renderer) — the constraint is "one CDN load per page, deduplicated, identical SRI emission shape as `MapPlugin`"
- Doc comment style on the public types (follow `InputProps` / `KeyValueEditorProps` doc tone)

## Deferred Ideas

(see CONTEXT.md `<deferred>` for the canonical list — preserved here for audit completeness)

- Image / video / file upload toolbar buttons
- Markdown ↔ Delta converters
- Inertia / SPA fetch-based submission
- Multiple Quill themes (Bubble, custom)
- Custom keyboard bindings per editor
- Mention / autocomplete module
- Real-time collaboration
- Localized toolbar tooltips
- Server-side HTML sanitizer pass
- Vendoring Quill into static-served assets
- Toolbar configuration override beyond `formats`
- Per-instance theme override
- Quill version bump
- Accessibility audit beyond Quill defaults
