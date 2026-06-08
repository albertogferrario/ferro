# Phase 169: StreamText Component - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; rationale logged per decision)

<domain>
## Phase Boundary

Ship the `StreamText` ferro-json-ui component: a built-in JSON-UI component that connects
to a server SSE endpoint URL and renders a token stream in place, token-by-token, with no
external JS framework. It is the browser-side consumer of the SSE transport delivered in
Phase 168 (`SseEvent` / `SseStream` / `HttpResponse::sse()`), and the rendering surface that
will later display `ferro-ai`'s streaming `TokenStream` (Phase 165) end-to-end.

Scope is exactly the five ROADMAP success criteria:
1. `StreamTextProps` with `sse_url: String`, `placeholder: Option<String>`, `loading_text: Option<String>`; serde round-trip.
2. Renderer emits `<div data-ferro-stream-url="{escaped_url}">` + loading state + inline `EventSource` JS that appends tokens.
3. Catalog + ferro-mcp `CatalogComponent` include `StreamText` with accurate prop descriptions.
4. Documented under `### StreamText` in `docs/src/json-ui/components.md`.
5. `cargo clippy --all --all-targets -- -D warnings` + `cargo test --all-features` green.

**This phase adds NO server-side route, NO ferro-ai dependency, NO streaming handler.** It
renders a URL the application author supplies. Producing that SSE endpoint is application
code (or a later phase), not this component.
</domain>

<scope_premise_note>
## Architecture verified (read before planning)

The JSON-UI component model is **not a single `Component` enum** — it is a string-keyed
`Element { type_name, props }` model. Adding a built-in component touches five extension
points, all confirmed by inspection 2026-06-08:

1. **Props struct** — `ferro-json-ui/src/component.rs` (`#[derive(Serialize, Deserialize, JsonSchema)]`, `#[serde(rename_all = "snake_case")]`). Pattern: `RawHtmlProps` (`component.rs:661`).
2. **Builtin type registry** — `BUILTIN_TYPES` array in `ferro-json-ui/src/render/mod.rs:43`. Comment at that site: *"Adding a new built-in requires updating BOTH this list AND the dispatch arm."*
3. **Dispatch arm** — `match el.type_name.as_str()` in `render/mod.rs:164`. Pattern: `"RawHtml" => atoms::render_raw_html(...)` (`render/mod.rs:189`).
4. **Render function** — in `render/atoms.rs` (StreamText is a leaf). Pattern: `render_raw_html` (`atoms.rs:1381`) emits `<div data-ferro-raw-html>...</div>`.
5. **Catalog entry** — `ferro-json-ui/src/catalog.rs` (register name + `schema_for!(StreamTextProps)` + per-prop descriptions). Pattern: `RawHtml` registration (`catalog.rs:265`).

**SC#3 is satisfied by catalog registration alone.** ferro-mcp's `json_ui_catalog`
(`ferro-mcp/src/tools/json_ui_catalog.rs:4,71`) derives every `CatalogComponent` from
`ferro_json_ui::global_catalog()` — there is NO hand-maintained component list in ferro-mcp.
Registering `StreamText` in `catalog.rs` automatically surfaces it through MCP. The planner
must NOT add a parallel hardcoded entry in ferro-mcp.
</scope_premise_note>

<decisions>
## Implementation Decisions

### JS delivery mechanism (SC#2)
- **D-01:** Deliver the `EventSource` wiring as **one shared inline init `<script>`**, not a
  per-element script. The render function emits only the `<div data-ferro-stream-url="...">`
  container; a single page-level init script (registered through the existing init-script /
  assets mechanism — `render_js_tags(assets, init_scripts)` at `render/mod.rs:292`) runs
  `document.querySelectorAll('[data-ferro-stream-url]')`, opens one `EventSource` per element,
  and appends tokens. `[auto] recommended: single shared script over N duplicated inline
  scripts — smaller payload, CSP-friendlier, scales to multiple StreamText on one page.`
  - This still satisfies "inline EventSource JS, no external JS framework": the script is
    inline (emitted in the document, no CDN/bundler), it just isn't duplicated per element.
  - **Open for planner (Claude's discretion):** exact registration path — whether the init
    script registers via the same channel plugins use (`init_scripts`) or a built-in
    equivalent. The constraint is: the script must be emitted exactly once even when multiple
    StreamText elements exist, and only when at least one StreamText is present in the spec.

### Token append + XSS safety (SC#2)
- **D-02:** Streamed tokens are appended as **text nodes** (`el.append(document.createTextNode(e.data))`
  or `el.textContent += e.data`), NEVER `innerHTML`. SSE payloads originate from an LLM / server
  stream and are untrusted; they must render as literal text, not parsed HTML. `[auto] non-negotiable
  security default — mirrors the workspace-wide "raw <script> must not appear" render tests
  (atoms.rs:1429, data.rs:787).`

### SSE event protocol + reconnect control (SC#2)
- **D-03:** Token frames arrive as default/`message` SSE events; the client appends `e.data`.
  A terminal **`done`** named event (`event: done`) signals completion — on receipt the client
  **calls `EventSource.close()`** and clears the loading indicator. `[auto]`
  - **Rationale (important):** `EventSource` auto-reconnects when the connection closes. If the
    server simply ends the stream without a `done` signal, the browser reopens the connection in
    a loop. Closing on an explicit `done` event prevents the reconnect storm. The planner should
    document this as the StreamText↔server contract: *the SSE endpoint feeding a StreamText must
    emit `event: done` when the stream is complete.* (Phase 168 `SseEvent` already supports
    `.event("done")` — `framework/src/http/sse.rs:78`.)
  - An `error` SSE event (or `EventSource.onerror` after open) closes the source and removes the
    loading indicator, leaving any tokens already appended in place (partial output preserved).

### Loading state semantics (SC#1, SC#2)
- **D-04:** Two distinct optional props, distinct roles:
  - `placeholder` — text shown **inside the content area** before the first token arrives
    (empty-state hint, e.g. "Response will appear here…"). Replaced/cleared on first token.
  - `loading_text` — a **status indicator** shown while the stream is open (e.g. "Generating…"),
    removed when the `done` event closes the stream.
  When `None`, render no placeholder / no loading indicator respectively (graceful absence,
  consistent with other optional-prop components). `[auto]`

### Attribute escaping (SC#2)
- **D-05:** `sse_url` is written into the `data-ferro-stream-url` attribute through the existing
  `html_escape` helper (`render/mod.rs:256`) — the same helper every other attribute-emitting
  renderer uses. The SC's "{escaped_url}" wording is met by this. `[auto]`

### Catalog + MCP surface (SC#3)
- **D-06:** Register `StreamText` in `catalog.rs` with `schema_for!(StreamTextProps)` and
  per-prop descriptions written for AI generation: `sse_url` ("URL of the server-sent-events
  endpoint that streams tokens; must emit `event: done` on completion"), `placeholder` ("text
  shown before the first token arrives"), `loading_text` ("status text shown while streaming").
  ferro-mcp picks this up automatically via `global_catalog()`. No ferro-mcp code change unless
  a test fixture enumerates components. `[auto]`

### Docs (SC#4)
- **D-07:** Add a `### StreamText` section to `docs/src/json-ui/components.md` documenting the
  three props, the `data-ferro-stream-url` contract, and the **server-side requirement** that
  the SSE endpoint emit `event: done`. Frame as the only/current version — no "v2"/"legacy"
  language (per JSON-UI naming convention). `[auto]`

### Claude's Discretion
- Exact init-script registration channel (D-01) and the precise JS source (token append loop,
  per-element EventSource bookkeeping) — within the D-01/D-02/D-03 contract.
- Whether the loading indicator (`loading_text`) lives in a sibling element or a child marker
  div inside the container — visual structure is unconstrained provided D-04 semantics hold.
- Serde fixture shape for the SC#1 round-trip test (follow existing `*Props` fixture pattern).
- Whether `render_streamtext` lives in `atoms.rs` (recommended — it's a leaf) or a new file.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase dependency (SSE transport this component consumes)
- `.planning/phases/168-framework-sse-primitives/168-CONTEXT.md` — `SseEvent`/`SseStream`/`HttpResponse::sse()` contract, the wire format, and the `event`/`id`/`data` field semantics StreamText's client must parse.
- `framework/src/http/sse.rs` §`SseEvent` (line 48) — `data`/`event`/`id`/`retry` fields + `.event("done")` builder (line 78). This is how a server marks stream completion.

### JSON-UI component extension points (the five touch-points)
- `ferro-json-ui/src/component.rs:661` — `RawHtmlProps`, the closest pattern for a minimal leaf props struct.
- `ferro-json-ui/src/render/mod.rs:43` — `BUILTIN_TYPES` registry (must add `"StreamText"`).
- `ferro-json-ui/src/render/mod.rs:164` — dispatch `match`; `mod.rs:189` shows the `RawHtml` arm.
- `ferro-json-ui/src/render/mod.rs:256` — `html_escape` (attribute escaping for `sse_url`).
- `ferro-json-ui/src/render/mod.rs:292` — `render_js_tags(assets, init_scripts)`, the inline-init-script emission mechanism for D-01.
- `ferro-json-ui/src/render/atoms.rs:1381` — `render_raw_html`, the leaf-renderer pattern (emits a single `data-ferro-*` div).
- `ferro-json-ui/src/catalog.rs:265` — `RawHtml` catalog registration pattern (name + `schema_for!` + descriptions).

### MCP surface (auto-derived — do not duplicate)
- `ferro-mcp/src/tools/json_ui_catalog.rs:4,67,71` — confirms `CatalogComponent` is sourced from `ferro_json_ui::global_catalog()`; registering in `catalog.rs` is sufficient for SC#3.

### Docs target
- `docs/src/json-ui/components.md` — add `### StreamText` (SC#4).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `html_escape` (`render/mod.rs:256`) — reuse verbatim for the `data-ferro-stream-url` attribute.
- `render_js_tags` / `init_scripts` mechanism (`render/mod.rs:292`) — the channel for D-01's shared init script; plugins already use it, so a built-in can too.
- `render_raw_html` (`atoms.rs:1381`) — copy its single-`data-*`-div shape as the StreamText container template.
- `schema_for!` + catalog registration helper (`catalog.rs`) — same call shape as every other component.

### Established Patterns
- `*Props` structs derive `Serialize, Deserialize, JsonSchema` with `#[serde(rename_all = "snake_case")]`; `Option` fields use `#[serde(default, skip_serializing_if = "Option::is_none")]` (see `component.rs:145`).
- Render functions return `String`; XSS tests assert raw `<script>` never survives (`atoms.rs:1429`, `data.rs:787`) — StreamText's textContent-append (D-02) must hold the same bar.
- Adding a builtin requires updating BOTH `BUILTIN_TYPES` AND the dispatch arm (enforced by the comment at `render/mod.rs:42`) — a frequent miss; planner should make these one task.

### Integration Points
- ferro-mcp `json_ui_catalog` auto-derives from `global_catalog()` — zero MCP code change needed for SC#3.
- Phase 168 SSE endpoint is the runtime peer; StreamText only needs the URL — no compile-time coupling to `framework`'s SSE types.
</code_context>

<specifics>
## Specific Ideas

- **StreamText↔server contract** the docs + catalog must state explicitly: the SSE endpoint a
  StreamText points at MUST emit `event: done` to terminate cleanly; otherwise `EventSource`
  auto-reconnects in a loop. This is the one non-obvious correctness requirement of the whole phase.
- Keep the inline JS dependency-free and small — it is the only JS this component ships, and the
  framework's positioning is "no external JS framework required."
</specifics>

<deferred>
## Deferred Ideas

- **Server-side streaming handler / example route** that produces tokens for a StreamText to
  consume — belongs to the ferro-ai CLI/integration phases (170–173) or an app-level example, not
  to this component phase.
- **Markdown / rich rendering of streamed tokens** (vs plain text) — a future enhancement; D-02
  deliberately ships plain text for safety. Would need a sanitizing incremental renderer; out of scope.
- **Retry/backoff or manual reconnect UI** beyond closing on `done` — not in the five success
  criteria; revisit only if a real consumer needs it.

None of the above blocks Phase 169; all stayed out of the component's boundary.
</deferred>

---

*Phase: 169-streamtext-component*
*Context gathered: 2026-06-08*
