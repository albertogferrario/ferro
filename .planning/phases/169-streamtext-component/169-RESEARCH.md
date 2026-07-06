# Phase 169: StreamText Component - Research

**Researched:** 2026-06-08
**Domain:** ferro-json-ui built-in component extension, inline EventSource JS
**Confidence:** HIGH

## Summary

This phase adds a single leaf built-in component to `ferro-json-ui`. The architecture is a straight extension of the five-point pattern used by every existing built-in (props struct, BUILTIN_TYPES registry, dispatch match arm, render function, catalog entry). All five touch-points are verified against live source.

The one non-obvious complication is the init-script delivery mechanism (D-01). `render_spec_to_html_with_plugins` currently short-circuits and emits no `scripts` output when the spec contains no plugin types. Built-in components have no equivalent path to inject an init script. When a spec contains a `StreamText` element but no plugins, the EventSource wiring script would be silently dropped unless `render_spec_to_html_with_plugins` is extended to detect `StreamText` presence and inject the script independently of the plugin pipeline.

**Primary recommendation:** Extend `render_spec_to_html_with_plugins` to collect builtin init scripts alongside plugin assets. The natural approach: a `collect_builtin_init_scripts(spec) -> Vec<String>` function that detects whether any `StreamText` element is present in the spec (single `contains` check against `BUILTIN_TYPES`-aware logic) and returns the EventSource init script exactly once when true. This function is called unconditionally in `render_spec_to_html_with_plugins` alongside `collect_plugin_assets`, and its output is merged into the `init_scripts` slice passed to `render_js_tags`. The early-return `if plugin_types.is_empty()` must be removed or placed after the builtin-script check.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Deliver EventSource wiring as one shared inline init `<script>`, not per-element. Render function emits only the `<div data-ferro-stream-url="...">` container; a single page-level init script runs `document.querySelectorAll('[data-ferro-stream-url]')`, opens one `EventSource` per element, appends tokens. Script emitted exactly once even when multiple StreamText elements exist, and only when at least one StreamText is present.
- **D-02:** Streamed tokens appended as text nodes (`el.append(document.createTextNode(e.data))` or `el.textContent += e.data`), NEVER `innerHTML`. Non-negotiable security default.
- **D-03:** Token frames arrive as default/`message` SSE events. Client appends `e.data`. Terminal `done` named event calls `EventSource.close()` and clears loading indicator. `onerror` closes source and removes loading indicator, leaving partial output in place.
- **D-04:** Two distinct optional props — `placeholder` (text shown inside content area before first token, cleared on first token) and `loading_text` (status indicator shown while stream is open, removed on `done`).
- **D-05:** `sse_url` written into `data-ferro-stream-url` attribute through existing `html_escape` helper at `render/mod.rs:256`.
- **D-06:** Register `StreamText` in `catalog.rs` with `schema_for!(StreamTextProps)` and per-prop descriptions for AI generation. ferro-mcp picks this up automatically via `global_catalog()`. No ferro-mcp code change.
- **D-07:** Add `### StreamText` section to `docs/src/json-ui/components.md`. Document three props, `data-ferro-stream-url` contract, and server-side requirement for `event: done`. Frame as current version — no "v2"/"legacy" language.

### Claude's Discretion
- Exact init-script registration channel (D-01) — precise registration path for built-in init script (whether it reuses the `init_scripts` Vec or a parallel built-in channel), within the constraint: emitted exactly once, only when StreamText is present.
- Whether the loading indicator (`loading_text`) lives in a sibling element or a child marker div inside the container.
- Serde fixture shape for SC#1 round-trip test (follow existing `*Props` fixture pattern in `atoms.rs`).
- Whether `render_streamtext` lives in `atoms.rs` (recommended) or a new file.
- Exact inline JS source — within D-01/D-02/D-03 contract.

### Deferred Ideas (OUT OF SCOPE)
- Server-side streaming handler or example route.
- Markdown/rich rendering of streamed tokens.
- Retry/backoff or manual reconnect UI.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AISSE-02 | `ferro-json-ui` provides a `StreamText` component that connects to an SSE endpoint URL and renders a token stream in place. The component is a JSON-UI element produced by a `Renderer` — consistent with the projection rendering pipeline. | All five extension points verified. Init-script gap identified and resolution documented below. |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Props definition + serde | ferro-json-ui (library) | — | Props struct lives in `component.rs` alongside all other `*Props` types |
| HTML rendering | ferro-json-ui (library) | — | `render_streamtext` in `atoms.rs`, dispatched from `render/mod.rs` |
| Init script delivery | ferro-json-ui (library) | — | `render_spec_to_html_with_plugins` extended to detect StreamText and inject shared EventSource script |
| Browser SSE client | Client (inline script) | — | Dependency-free EventSource wiring, inline, emitted by the render pipeline |
| Catalog + MCP surface | ferro-json-ui catalog | ferro-mcp (auto-derived) | `catalog.rs` registration; MCP consumes `global_catalog()` — no MCP code change |
| Documentation | docs/src/ | — | `docs/src/json-ui/components.md` |

## Standard Stack

### Core (all already present — no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `schemars` | workspace | `JsonSchema` derive for `StreamTextProps` | [VERIFIED: existing pattern in component.rs] |
| `serde` | workspace | `Serialize`/`Deserialize` for props round-trip | [VERIFIED: existing pattern in component.rs] |

No new `Cargo.toml` dependencies. The component is pure Rust + inline JS.

## Architecture Patterns

### System Architecture Diagram

```
Spec JSON
    │
    ▼
render_spec_to_html_with_plugins(spec, data)
    │
    ├── render_spec_to_html(spec, data)
    │       │
    │       └── render_element("root", ...) → dispatch match
    │               │
    │               └── "StreamText" arm → atoms::render_streamtext(el, ...)
    │                       │
    │                       └── emits: <div data-ferro-stream-url="{escaped_url}">
    │                                      [placeholder text if set]
    │                                      [loading indicator if set]
    │                                  </div>
    │
    ├── collect_plugin_types(spec)  → plugin names (may be empty)
    ├── collect_plugin_assets(...)  → plugin CSS/JS/init_scripts
    │
    └── collect_builtin_init_scripts(spec)  ← NEW
            │
            └── if any element.type_name == "StreamText" present:
                    return [FERRO_STREAM_TEXT_INIT_JS]  (emitted once)
                else:
                    return []
            │
            ▼
    render_js_tags(
        &[plugin_js_assets...],
        &[plugin_init_scripts..., builtin_init_scripts...]
    )
    → <script>…EventSource wiring…</script>  (in page <body>)

Browser
    │
    └── init script runs:
            document.querySelectorAll('[data-ferro-stream-url]')
            for each el:
                open EventSource(el.dataset.ferroStreamUrl)
                on message → el.append(createTextNode(e.data))
                             clear placeholder on first token
                on done    → source.close(); remove loading indicator
                on error   → source.close(); remove loading indicator
```

### Recommended File Locations
```
ferro-json-ui/src/
├── component.rs          # Add StreamTextProps struct (near RawHtmlProps, line ~665)
├── render/
│   ├── mod.rs            # (1) Add "StreamText" to BUILTIN_TYPES array (line 43)
│   │                     # (2) Add dispatch arm (line ~189)
│   │                     # (3) Add collect_builtin_init_scripts() function
│   │                     # (4) Fix render_spec_to_html_with_plugins() early-return
│   └── atoms.rs          # Add render_streamtext() function (after render_raw_html)
├── catalog.rs            # Add StreamTextProps import + BUILTIN_SPECS entry
docs/src/json-ui/
└── components.md         # Add ### StreamText section (after ### RawHtml)
```

### Pattern 1: Props Struct (verified at component.rs:633-665)

```rust
// Source: ferro-json-ui/src/component.rs:633 (SkeletonProps pattern)
// and component.rs:660 (RawHtmlProps pattern — all-optional variant)

/// Props for the `StreamText` component — SSE token stream renderer.
///
/// Connects to `sse_url` via `EventSource` and appends arriving tokens as
/// plain text. The SSE endpoint MUST emit `event: done` on completion to
/// prevent `EventSource` auto-reconnect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StreamTextProps {
    /// URL of the server-sent-events endpoint that streams tokens.
    /// Must emit `event: done` on completion.
    pub sse_url: String,
    /// Text shown inside the content area before the first token arrives.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Status text shown while the stream is open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loading_text: Option<String>,
}
```

Note: `#[serde(rename_all = "snake_case")]` is on enum variants only (see `ColumnFormat` at component.rs:131, `ToastVariant` at component.rs:671). Struct field names are already snake_case so no `rename_all` needed on the struct itself. Confirmed by inspecting `RawHtmlProps` and `SkeletonProps` — neither has `rename_all`.

### Pattern 2: BUILTIN_TYPES + Dispatch (verified at render/mod.rs:43,189)

```rust
// render/mod.rs BUILTIN_TYPES — add after "RawHtml":
"StreamText",

// render/mod.rs dispatch match — add after RawHtml arm (line ~189):
"RawHtml"     => atoms::render_raw_html(el, spec, data, depth),
"StreamText"  => atoms::render_streamtext(el, spec, data, depth),
```

The count test at `render/mod.rs:572` asserts `BUILTIN_TYPES.len() == 44`. After adding `StreamText` it must be updated to `45`. [VERIFIED: render/mod.rs:568-572]

### Pattern 3: Render Function (modeled on render_raw_html at atoms.rs:1374)

```rust
// ferro-json-ui/src/render/atoms.rs — after render_raw_html

pub(crate) fn render_streamtext(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: StreamTextProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("StreamText", e),
    };
    let escaped_url = html_escape(&props.sse_url);
    let placeholder_html = props.placeholder
        .as_deref()
        .map(|t| format!(
            "<span data-ferro-stream-placeholder>{}</span>",
            html_escape(t)
        ))
        .unwrap_or_default();
    let loading_html = props.loading_text
        .as_deref()
        .map(|t| format!(
            "<span data-ferro-stream-loading>{}</span>",
            html_escape(t)
        ))
        .unwrap_or_default();
    format!(
        "<div data-ferro-stream-url=\"{escaped_url}\">{placeholder_html}{loading_html}</div>"
    )
}
```

### Pattern 4: collect_builtin_init_scripts (NEW function in render/mod.rs)

```rust
// ferro-json-ui/src/render/mod.rs

const FERRO_STREAM_TEXT_INIT: &str = r#"(function(){
  document.querySelectorAll('[data-ferro-stream-url]').forEach(function(el){
    var src = new EventSource(el.dataset.ferroStreamUrl);
    var placeholder = el.querySelector('[data-ferro-stream-placeholder]');
    var loading = el.querySelector('[data-ferro-stream-loading]');
    var firstToken = true;
    src.onmessage = function(e){
      if(firstToken){ firstToken=false; if(placeholder) placeholder.remove(); }
      el.appendChild(document.createTextNode(e.data));
    };
    src.addEventListener('done', function(){
      src.close();
      if(loading) loading.remove();
    });
    src.onerror = function(){
      src.close();
      if(loading) loading.remove();
    };
  });
})();"#;

/// Returns the StreamText EventSource init script if the spec contains
/// at least one StreamText element; otherwise returns an empty Vec.
/// Called by `render_spec_to_html_with_plugins` for unconditional
/// built-in script injection.
fn collect_builtin_init_scripts(spec: &Spec) -> Vec<String> {
    let has_stream_text = spec
        .elements
        .values()
        .any(|el| el.type_name == "StreamText");
    if has_stream_text {
        vec![FERRO_STREAM_TEXT_INIT.to_string()]
    } else {
        vec![]
    }
}
```

### Pattern 5: Fix render_spec_to_html_with_plugins (critical — early-return gap)

The current implementation short-circuits when `plugin_types.is_empty()`:

```rust
// CURRENT (broken for StreamText with no plugins):
pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult {
    let html = render_spec_to_html(spec, data);
    let plugin_types = collect_plugin_types(spec);
    if plugin_types.is_empty() {
        return RenderResult { html, css_head: String::new(), scripts: String::new() };
        //                                                           ^^^^^^^^^^^^^^^^
        //                                           DROPS StreamText init script
    }
    ...
}

// FIXED:
pub fn render_spec_to_html_with_plugins(spec: &Spec, data: &Value) -> RenderResult {
    let html = render_spec_to_html(spec, data);
    let builtin_scripts = collect_builtin_init_scripts(spec);
    let plugin_types = collect_plugin_types(spec);
    if plugin_types.is_empty() && builtin_scripts.is_empty() {
        return RenderResult { html, css_head: String::new(), scripts: String::new() };
    }
    let type_names: Vec<String> = plugin_types.into_iter().collect();
    let assets = collect_plugin_assets(&type_names);
    let all_init_scripts: Vec<String> = assets.init_scripts
        .iter()
        .chain(builtin_scripts.iter())
        .cloned()
        .collect();
    RenderResult {
        html,
        css_head: render_css_tags(&assets.css),
        scripts: render_js_tags(&assets.js, &all_init_scripts),
    }
}
```

### Pattern 6: Catalog Entry (verified at catalog.rs:264-269)

```rust
// catalog.rs import line ~35 — add StreamTextProps to the use list
use crate::component::{
    ..., RawHtmlProps, StreamTextProps, ...
};

// BUILTIN_SPECS array — add after RawHtml entry (line ~264):
(
    "StreamText",
    "Connects to a server-sent-events endpoint and renders token-by-token output as plain text. The SSE endpoint must emit `event: done` on completion to prevent auto-reconnect.",
    || to_value(schema_for!(StreamTextProps)).unwrap(),
    &[],
),
```

### Pattern 7: Serde Round-Trip Test (modeled on atoms.rs:2168-2177)

```rust
// ferro-json-ui/src/render/atoms.rs — in #[cfg(test)] mod tests

#[test]
fn stream_text_props_serde_roundtrip() {
    use crate::component::StreamTextProps;
    let p = StreamTextProps {
        sse_url: "/ai/stream".to_string(),
        placeholder: Some("Response will appear here…".to_string()),
        loading_text: Some("Generating…".to_string()),
    };
    let j = serde_json::to_value(&p).unwrap();
    let back: StreamTextProps = serde_json::from_value(j).unwrap();
    assert_eq!(p, back);
}

#[test]
fn stream_text_props_minimal_serde_roundtrip() {
    use crate::component::StreamTextProps;
    let p = StreamTextProps {
        sse_url: "/stream".to_string(),
        placeholder: None,
        loading_text: None,
    };
    let j = serde_json::to_value(&p).unwrap();
    // Option::None fields must be absent in JSON (skip_serializing_if)
    assert!(j.get("placeholder").is_none(), "placeholder must be absent when None");
    assert!(j.get("loading_text").is_none(), "loading_text must be absent when None");
    let back: StreamTextProps = serde_json::from_value(j).unwrap();
    assert_eq!(p, back);
}

#[test]
fn render_streamtext_emits_data_attribute() {
    let spec = spec_with_root(Element::new("StreamText").prop("sse_url", "/api/stream"));
    let el = spec.elements.get("root").unwrap();
    let html = render_streamtext(el, &spec, &json!(null), 1);
    assert!(html.contains("data-ferro-stream-url=\"/api/stream\""), "got: {html}");
}

#[test]
fn render_streamtext_escapes_url() {
    let spec = spec_with_root(
        Element::new("StreamText").prop("sse_url", "/stream?q=a&b=<x>")
    );
    let el = spec.elements.get("root").unwrap();
    let html = render_streamtext(el, &spec, &json!(null), 1);
    assert!(!html.contains('&'), "raw & must not appear; got: {html}");
    assert!(!html.contains('<'), "raw < must not appear; got: {html}");
}
```

### Pattern 8: Init Script Injection Test

```rust
// ferro-json-ui/src/render/mod.rs — in #[cfg(test)] mod tests

#[test]
fn render_spec_with_stream_text_emits_init_script() {
    use crate::spec::{Element, Spec};
    let spec = Spec::builder()
        .element("root", Element::new("StreamText").prop("sse_url", "/stream"))
        .build()
        .expect("spec builds");
    let result = render_spec_to_html_with_plugins(&spec, &json!({}));
    assert!(
        result.scripts.contains("EventSource"),
        "init script must be present; got: {}",
        result.scripts
    );
}

#[test]
fn render_spec_without_stream_text_emits_no_init_script() {
    use crate::spec::{Element, Spec};
    let spec = Spec::builder()
        .element("root", Element::new("Text").prop("content", "Hello"))
        .build()
        .expect("spec builds");
    let result = render_spec_to_html_with_plugins(&spec, &json!({}));
    assert!(
        result.scripts.is_empty(),
        "no init script when no StreamText; got: {}",
        result.scripts
    );
}
```

### Anti-Patterns to Avoid
- **innerHTML token append:** Tokens must be appended as text nodes, never via `innerHTML`. LLM output is untrusted.
- **Per-element script duplication:** Each `render_streamtext` call must NOT inline the full EventSource script. The single shared script is emitted once by `render_spec_to_html_with_plugins`.
- **Hardcoding in ferro-mcp:** SC#3 is satisfied by `catalog.rs` registration alone. Do not add a parallel entry in ferro-mcp.
- **Skipping the BUILTIN_TYPES count test update:** `render/mod.rs:572` asserts exactly 44 built-ins. After adding `StreamText` this must become 45 or the test fails.
- **Forgetting the early-return guard:** `render_spec_to_html_with_plugins` returns early when no plugins are present, silently dropping the StreamText init script. The condition must include `builtin_scripts.is_empty()`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML attribute escaping | Custom escaper | `html_escape()` at `render/mod.rs:256` | Already present, tested, handles all five chars |
| Props deserialization | Manual JSON parsing | `decode_props::<StreamTextProps>(&el.props)` at `atoms.rs:47` | Existing helper with diagnostic error return |
| Catalog schema | Hand-written JSON Schema | `schema_for!(StreamTextProps)` via schemars | Derives from `#[derive(JsonSchema)]` automatically |
| MCP component listing | Manual MCP registration | `global_catalog()` in ferro-mcp | Already derives from catalog.rs registration |

## Common Pitfalls

### Pitfall 1: Early-return drops the init script
**What goes wrong:** `render_spec_to_html_with_plugins` returns `scripts: String::new()` when no plugin types are present. A page with only `StreamText` (no plugins) gets no EventSource script and the component silently does nothing.
**Why it happens:** The early-return was designed to skip asset collection when no plugins are used — a valid optimization that predates built-in init scripts.
**How to avoid:** Check `builtin_scripts.is_empty()` alongside `plugin_types.is_empty()` in the early-return condition. See Pattern 5 above.
**Warning signs:** `render_spec_to_html_with_plugins` test for StreamText-only spec shows empty `scripts` field.

### Pitfall 2: BUILTIN_TYPES count test failure
**What goes wrong:** `builtin_types_count_matches_dispatch` test at `render/mod.rs:568` asserts exactly 44 entries. After adding `StreamText`, clippy/tests fail with assertion mismatch.
**Why it happens:** The count is hardcoded as a defense-in-depth invariant.
**How to avoid:** Update the assertion to 45 in the same task that adds `StreamText` to the array.
**Warning signs:** `cargo test` reports `assertion failed: BUILTIN_TYPES.len() == 44`.

### Pitfall 3: EventSource auto-reconnect on stream close
**What goes wrong:** If the SSE server closes the connection without emitting `event: done`, the browser's `EventSource` automatically reconnects and the component re-fetches the same endpoint in a loop.
**Why it happens:** This is the specified behavior of the `EventSource` API per the WHATWG SSE spec.
**How to avoid:** The init script must call `src.close()` on `event: done`. The docs must state the server contract explicitly. Phase 168's `SseEvent::event("done")` builder is the server-side mechanism.
**Warning signs:** Browser devtools shows repeated SSE requests to the same endpoint after the stream completes.

### Pitfall 4: `serde(rename_all)` confusion
**What goes wrong:** Adding `#[serde(rename_all = "snake_case")]` to `StreamTextProps` struct (following enum pattern) causes JSON field names to be double-transformed (they are already snake_case).
**Why it happens:** The attribute is used on enum variants (`ColumnFormat`, `ToastVariant`) but NOT on props structs. `RawHtmlProps` and `SkeletonProps` (the direct patterns) do not have it.
**How to avoid:** Do not add `rename_all` to the struct. Fields `sse_url`, `placeholder`, `loading_text` serialize as their field names.

### Pitfall 5: XSS via placeholder/loading_text
**What goes wrong:** If `placeholder` or `loading_text` are emitted without `html_escape`, an attacker who controls spec values can inject script.
**Why it happens:** Both are `Option<String>` props that come from user-supplied spec JSON.
**How to avoid:** Pass both through `html_escape()` before emitting into the `<span>` content. The D-02 XSS bar applies to all rendered content, not just token payloads.

## Code Examples

### Complete render_streamtext
```rust
// Source: verified pattern from render_raw_html at atoms.rs:1374-1382
pub(crate) fn render_streamtext(
    el: &Element,
    _spec: &Spec,
    _data: &Value,
    _depth: usize,
) -> String {
    let props: StreamTextProps = match decode_props(&el.props) {
        Ok(p) => p,
        Err(e) => return decode_diagnostic("StreamText", e),
    };
    let escaped_url = html_escape(&props.sse_url);
    let placeholder_html = props.placeholder.as_deref()
        .map(|t| format!("<span data-ferro-stream-placeholder>{}</span>", html_escape(t)))
        .unwrap_or_default();
    let loading_html = props.loading_text.as_deref()
        .map(|t| format!("<span data-ferro-stream-loading>{}</span>", html_escape(t)))
        .unwrap_or_default();
    format!(r#"<div data-ferro-stream-url="{escaped_url}">{placeholder_html}{loading_html}</div>"#)
}
```

### Minimal dependency-free EventSource init script (D-01/D-02/D-03 compliant)
```javascript
// Emitted as a single <script> block via render_js_tags init_scripts.
// No external framework. Tokens appended as text nodes (D-02).
// close() on 'done' event prevents auto-reconnect (D-03).
(function(){
  document.querySelectorAll('[data-ferro-stream-url]').forEach(function(el){
    var src = new EventSource(el.dataset.ferroStreamUrl);
    var placeholder = el.querySelector('[data-ferro-stream-placeholder]');
    var loading = el.querySelector('[data-ferro-stream-loading]');
    var firstToken = true;
    src.onmessage = function(e){
      if(firstToken){ firstToken=false; if(placeholder) placeholder.remove(); }
      el.appendChild(document.createTextNode(e.data));
    };
    src.addEventListener('done', function(){
      src.close();
      if(loading) loading.remove();
    });
    src.onerror = function(){
      src.close();
      if(loading) loading.remove();
    };
  });
})();
```

### Docs section shape (modeled on RawHtml at docs/src/json-ui/components.md:1426-1447)
```markdown
### StreamText

Connects to a server-sent-events endpoint and renders token-by-token output as
plain text. Tokens are appended as text nodes — no HTML interpretation.

| Prop | Type | Description |
|------|------|-------------|
| `sse_url` | `string` | URL of the SSE endpoint that streams tokens |
| `placeholder` | `string?` | Text shown inside the content area before the first token arrives |
| `loading_text` | `string?` | Status indicator shown while the stream is open |

\`\`\`json
"response_area": {
  "type": "StreamText",
  "props": {
    "sse_url": "/ai/generate",
    "placeholder": "Response will appear here…",
    "loading_text": "Generating…"
  }
}
\`\`\`

**Server contract.** The SSE endpoint must emit `event: done` when the stream
is complete:

\`\`\`rust
tx.send(SseEvent::new().event("done").data("")).await.ok();
\`\`\`

Without `event: done`, the browser's `EventSource` auto-reconnects after the
connection closes, causing the component to re-fetch in a loop.

**Security.** Tokens are appended as plain text nodes — `innerHTML` is never
called. Streamed content cannot inject HTML or execute scripts regardless of
its content.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No SSE support in framework | `SseEvent`/`SseStream`/`FerroBody::Stream` | Phase 168 (2026-06-08) | StreamText can now point at a real framework SSE endpoint |
| Plugin-only init scripts | Built-in init script via `collect_builtin_init_scripts` | Phase 169 (this phase) | Built-ins can now contribute page-level JS without using the plugin registry |

## Assumptions Log

No `[ASSUMED]` claims — all findings verified against live source.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

All claims in this research were verified by direct file inspection.

## Open Questions

1. **Loading indicator DOM structure**
   - What we know: D-04 says `loading_text` is a "status indicator" shown while the stream is open. D-04 leaves the visual structure (sibling vs child) as Claude's discretion.
   - What's unclear: Whether the indicator should be inside the same container div or a sibling. A child `<span data-ferro-stream-loading>` inside the container is the simplest approach and what the code examples above assume.
   - Recommendation: Use a child `<span data-ferro-stream-loading>` inside the container div. The init script removes it by querying `el.querySelector('[data-ferro-stream-loading]')`. This is self-contained and requires no coordination between the render function and the init script beyond the attribute name.

2. **`render_spec_to_html` callers that bypass `_with_plugins`**
   - What we know: Several callers use `render_spec_to_html` directly (not the `_with_plugins` variant) — notably in tests. These callers will render `StreamText` correctly (the `<div data-ferro-stream-url>` is emitted) but will not receive the init script.
   - What's unclear: Whether this is a problem for production callers.
   - Recommendation: Document in the `StreamText` render function and in docs that `render_spec_to_html_with_plugins` must be used (not the bare `render_spec_to_html`) for the EventSource wiring to be emitted. The production render path in the framework's JSON-UI route handler should already use the plugin-aware variant. Verify this during implementation.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — this phase is pure Rust code + inline JS).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AISSE-02 SC#1 | `StreamTextProps` serde round-trip (all fields, minimal) | unit | `cargo test -p ferro-json-ui stream_text_props_serde` | ❌ Wave 0 |
| AISSE-02 SC#2a | `render_streamtext` emits `data-ferro-stream-url` attribute | unit | `cargo test -p ferro-json-ui render_streamtext_emits` | ❌ Wave 0 |
| AISSE-02 SC#2b | `render_streamtext` escapes `sse_url` via `html_escape` | unit | `cargo test -p ferro-json-ui render_streamtext_escapes` | ❌ Wave 0 |
| AISSE-02 SC#2c | `render_spec_to_html_with_plugins` emits init script when StreamText present | unit | `cargo test -p ferro-json-ui render_spec_with_stream_text_emits_init_script` | ❌ Wave 0 |
| AISSE-02 SC#2d | No init script emitted when no StreamText in spec | unit | `cargo test -p ferro-json-ui render_spec_without_stream_text` | ❌ Wave 0 |
| AISSE-02 SC#3 | `global_catalog()` includes `StreamText` component spec | unit | `cargo test -p ferro-json-ui catalog` | ❌ Wave 0 |
| AISSE-02 SC#5 | BUILTIN_TYPES count is 45 after adding StreamText | unit | `cargo test -p ferro-json-ui builtin_types_count` | ✅ (needs count update) |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `stream_text_props_serde_roundtrip` test in `atoms.rs` `#[cfg(test)]`
- [ ] `render_streamtext_emits_data_attribute` test in `atoms.rs` `#[cfg(test)]`
- [ ] `render_streamtext_escapes_url` test in `atoms.rs` `#[cfg(test)]`
- [ ] `render_spec_with_stream_text_emits_init_script` test in `render/mod.rs` `#[cfg(test)]`
- [ ] `render_spec_without_stream_text_emits_no_init_script` test in `render/mod.rs` `#[cfg(test)]`
- [ ] `global_catalog_includes_stream_text` test in `catalog.rs` or dedicated test

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | `html_escape()` on `sse_url`, `placeholder`, `loading_text`; text-node append for streamed tokens |
| V6 Cryptography | no | — |

### Known Threat Patterns for streaming text injection

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via streamed token | Tampering | `document.createTextNode()` — browser never parses token as HTML |
| XSS via `sse_url` attribute | Tampering | `html_escape()` on the prop before emitting into the `data-` attribute |
| XSS via `placeholder`/`loading_text` | Tampering | `html_escape()` on both props before emitting as element content |
| SSE reconnect loop (DoS-like) | Denial of Service | `src.close()` on `event: done` and `onerror`; documented server contract |

## Sources

### Primary (HIGH confidence)
- `ferro-json-ui/src/component.rs:633-665` — verified `SkeletonProps` and `RawHtmlProps` struct patterns (derive macros, `Option` serde attributes)
- `ferro-json-ui/src/render/mod.rs:43-92` — verified `BUILTIN_TYPES` array (44 entries, enforced by test at line 572)
- `ferro-json-ui/src/render/mod.rs:164-215` — verified dispatch match arm pattern
- `ferro-json-ui/src/render/mod.rs:114-131` — verified `render_spec_to_html_with_plugins` early-return gap
- `ferro-json-ui/src/render/mod.rs:256-262` — verified `html_escape` function
- `ferro-json-ui/src/render/mod.rs:293-317` — verified `render_js_tags(assets, init_scripts)` signature
- `ferro-json-ui/src/render/atoms.rs:1374-1382` — verified `render_raw_html` leaf-renderer pattern
- `ferro-json-ui/src/catalog.rs:29-38,264-269` — verified import pattern and `RawHtml` registration shape
- `ferro-json-ui/src/render/atoms.rs:2168-2177` — verified serde round-trip test pattern
- `ferro-json-ui/src/plugin.rs:195-247` — verified `collect_plugin_assets` and `init_scripts` handling
- `docs/src/json-ui/components.md:1426-1447` — verified `### RawHtml` docs section shape

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all extension points verified by direct file inspection
- Architecture: HIGH — init-script gap identified and resolution verified against actual code
- Pitfalls: HIGH — each derived from a specific verified code path

**Research date:** 2026-06-08
**Valid until:** Stable until `ferro-json-ui/src/render/mod.rs` render pipeline changes (low churn area)
