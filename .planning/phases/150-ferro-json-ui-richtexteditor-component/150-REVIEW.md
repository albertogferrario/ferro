---
phase: 150-ferro-json-ui-richtexteditor-component
reviewed: 2026-05-01T00:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - docs/src/json-ui/components.md
  - ferro-json-ui/src/assets/mod.rs
  - ferro-json-ui/src/assets/quill.rs
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/plugin.rs
  - ferro-json-ui/src/plugins/mod.rs
  - ferro-json-ui/src/plugins/rich_text_editor.rs
  - ferro-json-ui/src/render.rs
  - ferro-json-ui/src/resolve.rs
  - ferro-json-ui/src/runtime/mod.rs
  - ferro-json-ui/src/runtime/rich_text_editor.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 150: Code Review Report

**Reviewed:** 2026-05-01T00:00:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Phase 150 adds a first-class `RichTextEditor` component backed by Quill 2.0.3.
The overall design is sound: the asset-adapter pattern (routing Quill's CDN assets
through the existing plugin pipeline) is clean and avoids a parallel asset path.
The server-side renderer correctly HTML-escapes all dynamic values. The client-side
sanitizer uses `DOMParser` over innerHTML-assignment, which is the correct
approach.

The issues found are all correctness or robustness concerns at the boundary between
server-rendered HTML and client-side JavaScript:

- The initial HTML content path (non-Delta `value`) is placed in the host element
  as escaped text, but the IIFE never calls `clipboard.dangerouslyPasteHTML` to
  convert that text back to Quill-formatted content — the host's text content is
  only treated as a Delta candidate, so HTML initial values are silently ignored.
- The CSS `id` on the editor host (`id="{name}"`) is not namespaced, meaning two
  `RichTextEditor` instances with the same `name` on one page produce duplicate
  DOM `id` values, which breaks the `for=` label association and the error-element
  selector.
- The `<a>` target attribute is passed through without restriction in
  `stripDisallowedAttributes`, allowing `target="_blank"` without `rel="noopener"`.
- The `cssEscapeId` function's regex misses the `!` and `/` characters, which are
  valid in CSS identifiers but require escaping in selector context.

---

## Warnings

### WR-01: HTML initial value silently ignored — only Delta path is rehydrated

**File:** `ferro-json-ui/src/runtime/rich_text_editor.rs:67-96`

**Issue:** `initRichTextEditor` reads `host.textContent`, attempts to parse it as
JSON Delta, and sets contents via `quill.setContents(initialDelta)` if the parse
succeeds. If parsing fails or the string is not a Delta, the function falls through
and `host.textContent` is left in the DOM as raw text — Quill renders it as a
plain text string, not as HTML. The server-side renderer writes the initial value
as HTML-escaped text in the host (for both Delta and HTML cases), so an initial
value like `<p>Hello <b>world</b></p>` becomes the literal string
`&lt;p&gt;Hello &lt;b&gt;world&lt;/b&gt;&lt;/p&gt;` in the rendered output. When
Quill initialises with that non-Delta content, it displays the raw escaped string
as text rather than as formatted HTML. The `dangerouslyPasteHTML` branch described
in the struct doc and the component catalog is not present in the IIFE.

**Fix:** After the Delta detection block, if `initialDelta === null` and
`hostText.length > 0`, load the content as HTML through Quill's clipboard module:

```javascript
if (initialDelta !== null) {
    quill.setContents(initialDelta);
} else if (hostText.length > 0) {
    // hostText is the HTML-escaped initial value from the server.
    // Decode HTML entities back to their characters before pasting.
    var tmp = document.createElement('div');
    tmp.textContent = hostText;
    var decoded = tmp.innerHTML;
    quill.clipboard.dangerouslyPasteHTML(decoded);
}
```

The decoded HTML is then filtered by Quill's clipboard pipeline (which respects the
`formats` allowlist), so no additional sanitization step is needed at load time.
Clear `host.textContent` beforehand (as is already done for the Delta path) so
Quill does not double-render.

---

### WR-02: Duplicate DOM `id` when two editors share the same `name`

**File:** `ferro-json-ui/src/render.rs:2269-2271`

**Issue:** The editor host div emits `id="{name_escaped}"`. The `<label for=...>`
also uses `{name_escaped}`. If a page contains two `RichTextEditor` components
with the same `name` value (e.g. both named `"body"` in two separate forms on one
page), the DOM has two elements with `id="body"`. This breaks:
1. `document.getElementById('body')` returns the first one only.
2. The `for="body"` association on the second label points to the wrong element.
3. `showOrCreateRteError` uses `wrapper.querySelector('#' + cssEscapeId(errId))`
   which queries within the wrapper scope, but `errId` is `"err-body"` — if both
   wrappers generate the same error-element id, the guard `if (existing)` may find
   the element in the wrong wrapper.

`name` values are under caller control and are expected to be unique per form, but
two separate forms on one page (e.g. a create form and an edit form in a Tabs
component) can legitimately reuse the same field name.

**Fix:** Namespace the host id and the error element id with a per-instance suffix.
The simplest approach is to include the wrapper's index or a random suffix at render
time, or to scope the `for=` association within the wrapper and drop the global `id`:

```rust
// In render_rich_text_editor, use a namespaced id so duplicates cannot occur.
// The label `for` attribute still works because both elements share the same
// scoped id within the same wrapper — browsers match by document-level id,
// but a data attribute approach avoids the collision entirely.
//
// Option A: use data-for on the label and remove the global id contract.
// Option B: append the key from ComponentNode (requires passing it in).
// Option C: document that `name` must be unique per page (not just per form).
//
// The minimal fix for v1 is to add a doc-level note in RichTextEditorProps
// that `name` must be unique within the page, not just within the form, and
// to add a clippy/lint note. A structural fix can be a follow-up phase.
```

---

### WR-03: `<a target=...>` passed through without `rel="noopener noreferrer"`

**File:** `ferro-json-ui/src/runtime/rich_text_editor.rs:301-304`

**Issue:** `stripDisallowedAttributes` keeps `href` and `target` attributes on
`<a>` elements. When `target` is `"_blank"`, the opened page can access
`window.opener` of the current tab. In a rich-text editor context a user or a
lightly-malicious paste source can produce `<a target="_blank">` content that
survives sanitization. If the sanitized HTML is later rendered in the browser
(e.g. displayed via `innerHTML` from `{name}_html`), this is a tabnabbing vector.

**Fix:** Either strip `target` entirely (the simplest fix for v1), or when `target`
is kept, ensure `rel="noopener noreferrer"` is set:

```javascript
if (tag === 'A') {
    if (lower === 'href') {
        // keep
    } else if (lower === 'target') {
        // keep only safe values; strip _blank or force rel
        var targetVal = el.getAttribute(attr) || '';
        if (targetVal === '_blank') {
            el.setAttribute('rel', 'noopener noreferrer');
        }
    } else {
        el.removeAttribute(attr);
    }
    continue;
}
```

---

### WR-04: `cssEscapeId` regex is incomplete — misses `!`, `/`, `,`, `@`, `%`

**File:** `ferro-json-ui/src/runtime/rich_text_editor.rs:147-149`

**Issue:** The `cssEscapeId` function escapes a limited set of characters for CSS
selector context:

```javascript
function cssEscapeId(id) {
    return String(id).replace(/(["\\#.()\[\]:>+~*=^$|])/g, '\\$1');
}
```

The character class omits several characters that are special in CSS selectors:
`!`, `@`, `,`, `/`, `%`, `?`, `;`, and the space character. If `name` contains any
of these (e.g. a dotted name like `article.body`), the selector
`wrapper.querySelector('#' + cssEscapeId(errId))` may throw a `SyntaxError` rather
than finding or not finding the element. The dot `.` is included, which is the most
common hazard.

While `name` values in practice are simple identifiers, the function is presented
as a general-purpose polyfill. If it silently fails or throws on an unexpected
input, the required-validation error is not shown to the user.

**Fix:** Use the standard `CSS.escape` when available (modern browsers), and fall
back to the limited polyfill only for known-safe id patterns, or restrict `name`
validation to alphanumeric-plus-hyphen-underscore at the Rust layer and document
the constraint:

```javascript
function cssEscapeId(id) {
    if (typeof CSS !== 'undefined' && CSS.escape) {
        return CSS.escape(id);
    }
    // Fallback for very old browsers: covers the common CSS selector
    // metacharacters. Callers must ensure id is a simple identifier.
    return String(id).replace(/(["\\#.()\[\]:>+~*=^$|!@,%?; ])/g, '\\$1');
}
```

---

## Info

### IN-01: `#[allow(dead_code)]` on all four Quill constants is premature

**File:** `ferro-json-ui/src/assets/quill.rs:15-33`

**Issue:** All four constants (`QUILL_JS_URL`, `QUILL_CSS_URL`, `QUILL_JS_SRI`,
`QUILL_CSS_SRI`) carry `#[allow(dead_code)]`. The constants are imported and
actively used by `ferro-json-ui/src/plugins/rich_text_editor.rs`. Once that import
exists, the compiler will no longer emit dead-code warnings for them, so the
`allow` attributes serve no purpose and suppress potential future warnings if the
constants become genuinely unused.

**Fix:** Remove the four `#[allow(dead_code)]` attributes. Cargo clippy will tell
you if any become unused again.

---

### IN-02: `register_built_in_plugins` is exported but never called by the framework

**File:** `ferro-json-ui/src/plugins/mod.rs:16-19`

**Issue:** `register_built_in_plugins` is a public function that registers `MapPlugin`
and `RichTextEditorPlugin`. However, `global_plugin_registry()` in `plugin.rs`
(line 153-159) already calls `registry.register(crate::plugins::MapPlugin)` and
`registry.register(crate::plugins::RichTextEditorPlugin)` inline during lazy
initialization — so the built-in plugins are registered via two separate code paths.
`register_built_in_plugins` is dead code unless a caller invokes it explicitly (which
none do in the codebase). If called, it re-registers both plugins, which is harmless
but wasteful and confusing.

**Fix:** Either remove `register_built_in_plugins` (since `global_plugin_registry`
handles initialization), or make `global_plugin_registry` call
`register_built_in_plugins` and remove the inline registrations — pick one path
and delete the other.

---

### IN-03: Docs component count says "26 built-in component types" but catalog now has 42

**File:** `docs/src/json-ui/components.md:3`

**Issue:** The opening sentence reads "JSON-UI includes 26 built-in component types
organized into six groups." The MCP catalog test at
`ferro-mcp/src/tools/json_ui_catalog.rs:1298` asserts `components.len() == 42`.
The count in the documentation is stale.

**Fix:** Update the introductory sentence in `docs/src/json-ui/components.md` to
reflect the current component count, or replace the hard-coded number with a phrase
that does not require manual maintenance (e.g. "a rich set of built-in component
types").

---

_Reviewed: 2026-05-01T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
