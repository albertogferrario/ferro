---
phase: 169-streamtext-component
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/runtime/sse.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - docs/src/json-ui/components.md
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 169: Code Review Report

**Reviewed:** 2026-06-08
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Phase 169 adds the `StreamText` component: a `StreamTextProps` struct, a `render_streamtext` atom renderer, a shared inline `FERRO_STREAM_TEXT_INIT` EventSource script, and documentation. The security-critical paths are clean: `sse_url` is HTML-attribute-escaped via `html_escape`, the init script appends tokens via `createTextNode` (never `innerHTML`), `EventSource.close()` is called on both `done` and `onerror`, the script is emitted at most once per page, the early-return guard in `render_spec_to_html_with_plugins` is correct, and there is no hardcoded app identity anywhere. Registry sync (`BUILTIN_TYPES` / `BUILTIN_SPECS` / dispatch arm) is consistent at 45 entries with the drift guard enforced at build time.

Two behavioral edge cases warrant attention before the component is used in production.

## Warnings

### WR-01: Empty `sse_url` resolves to current page URL in the browser

**File:** `ferro-json-ui/src/component.rs:677`

**Issue:** `StreamTextProps.sse_url` carries `#[serde(default)]`, so an element spec with no `sse_url` prop (or an explicit `""`) successfully decodes and renders `<div data-ferro-stream-url="">`. The init script then calls `new EventSource("")`, which per the WHATWG EventSource spec resolves the empty string relative to the current document — i.e., it opens an SSE connection to the current page URL. On pages that do not respond as SSE, the browser will receive a non-200 or non-text/event-stream response, fire `onerror`, and call `close()` — so no infinite loop. However, the side-effect GET request to the current page is unexpected and may pollute access logs, confuse server-side analytics, or trigger unintended auth flows on SSR pages.

**Fix:** Guard the URL at render time and skip the EventSource setup when the attribute value is empty:

In `FERRO_STREAM_TEXT_INIT` (mod.rs ~line 269):
```js
document.querySelectorAll('[data-ferro-stream-url]').forEach(function(el){
  var url = el.dataset.ferroStreamUrl;
  if (!url) return;                   // <-- add this guard
  var src = new EventSource(url);
  // ...
});
```

This is a one-line fix in the constant string that makes the component a no-op rather than a misfiring GET when `sse_url` is absent.

---

### WR-02: `placeholder` element not removed on zero-token `done`

**File:** `ferro-json-ui/src/render/mod.rs:273-279`

**Issue:** The init script only removes the placeholder on the first `onmessage` event (`if(firstToken){ ... placeholder.remove() }`). If the SSE endpoint emits `event: done` immediately without any data frames (empty response), the `done` handler fires, the loading indicator is correctly removed, but the placeholder span stays visible forever. The user sees the placeholder text after the stream ends with no content — a misleading UI state.

**Fix:** Remove the placeholder in the `done` handler as well:

```js
src.addEventListener('done', function(){
  src.close();
  if(loading) loading.remove();
  if(placeholder) placeholder.remove();   // <-- add this line
});
```

The placeholder is already removed on first token, so this addition is idempotent in the normal flow.

## Info

### IN-01: `render_streamtext` has no rustdoc comment

**File:** `ferro-json-ui/src/render/atoms.rs:1387`

**Issue:** Every other atom renderer in this file has at least an inline section comment. `render_streamtext` has only the section marker `// ── StreamText — SSE token stream renderer ───────────────────────────────` above it but no function-level doc comment. The security note about tokens being plain text nodes (present in docs/) is not in rustdoc.

**Fix:** Add a brief doc comment consistent with the crate's existing style, noting the key trust boundary:

```rust
/// Renders the StreamText container div with a `data-ferro-stream-url` attribute.
/// The EventSource init script (emitted by `collect_builtin_init_scripts`) appends
/// tokens via `createTextNode` — streamed content is never parsed as HTML.
pub(crate) fn render_streamtext(
```

---

### IN-02: `render_streamtext_escapes_url` test only asserts absence of raw chars

**File:** `ferro-json-ui/src/render/atoms.rs:2300-2313`

**Issue:** The escaping test checks that `&b=` and `<script>` do not appear in the output, but does not positively assert that the escaped forms (`&amp;b=`, `&lt;script&gt;`) are present. The test would still pass if `html_escape` silently dropped the characters rather than encoding them.

**Fix:** Add positive assertions:

```rust
assert!(html.contains("&amp;b="), "& must be encoded as &amp;; got: {html}");
assert!(
    html.contains("&lt;script&gt;"),
    "<script> must be encoded; got: {html}"
);
```

---

_Reviewed: 2026-06-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
