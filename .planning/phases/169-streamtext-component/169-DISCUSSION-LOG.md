# Phase 169: StreamText Component - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 169-streamtext-component
**Mode:** `--auto` (all gray areas auto-selected to recommended default)
**Areas discussed:** JS delivery, Token append/XSS, SSE protocol & reconnect, Loading semantics, Attribute escaping, Catalog/MCP surface, Docs

---

## JS delivery mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Shared inline init script | One page-level `<script>` scans `[data-ferro-stream-url]`, opens one EventSource per element | ✓ |
| Per-element inline script | Each StreamText div emits its own `<script>` | |
| External JS asset/bundle | Ship a CDN/bundled script | |

**Choice:** Shared inline init script. **Notes:** Smaller payload, CSP-friendlier, scales to multiple StreamText on one page; still "inline, no external framework" per SC#2. Registration via existing `render_js_tags` init-script channel (`render/mod.rs:292`).

---

## Token append + XSS safety

| Option | Description | Selected |
|--------|-------------|----------|
| textContent / text node | Append tokens as literal text | ✓ |
| innerHTML | Parse tokens as HTML | |

**Choice:** textContent. **Notes:** SSE payloads (LLM output) are untrusted; must not parse as HTML. Mirrors workspace-wide "raw `<script>` must not survive" render tests.

---

## SSE event protocol + reconnect control

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit `done` event closes EventSource | Append on message; `event: done` → `.close()` | ✓ |
| Rely on server closing connection | No explicit done signal | |

**Choice:** Explicit `done`. **Notes:** EventSource auto-reconnects on close; without a `done` signal the browser loops reconnecting. Becomes the documented StreamText↔server contract. `error` closes source, preserves partial output.

---

## Loading state semantics

| Option | Description | Selected |
|--------|-------------|----------|
| placeholder = pre-first-token hint; loading_text = streaming status | Two distinct roles | ✓ |
| Collapse both into one indicator | Single message | |

**Choice:** Two distinct roles. **Notes:** `placeholder` cleared on first token; `loading_text` removed on `done`. Both optional, absent → nothing rendered.

---

## Attribute escaping

| Option | Description | Selected |
|--------|-------------|----------|
| Existing `html_escape` helper | Reuse `render/mod.rs:256` | ✓ |
| New escaping logic | Custom | |

**Choice:** Existing `html_escape`. **Notes:** Same helper every other attribute renderer uses.

---

## Catalog / MCP surface

| Option | Description | Selected |
|--------|-------------|----------|
| Register in ferro-json-ui catalog only | MCP auto-derives via `global_catalog()` | ✓ |
| Also add hardcoded ferro-mcp entry | Parallel list | |

**Choice:** Catalog-only registration. **Notes:** `ferro-mcp/json_ui_catalog.rs` sources components from `global_catalog()`; a parallel MCP entry would create two sources of truth.

---

## Claude's Discretion

- Exact init-script registration channel and JS source within the D-01/D-02/D-03 contract.
- Loading-indicator DOM structure (sibling vs child marker).
- Serde fixture shape for the round-trip test.
- File placement of the render function (recommended `atoms.rs`).

## Deferred Ideas

- Server-side streaming handler / example route (Phases 170–173 or app-level).
- Markdown/rich rendering of streamed tokens (future; plain text shipped for safety).
- Retry/backoff / manual reconnect UI beyond closing on `done`.
