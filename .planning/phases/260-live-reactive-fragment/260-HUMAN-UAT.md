---
status: partial
phase: 260-live-reactive-fragment
source: [260-VERIFICATION.md]
started: 2026-07-26T00:00:00Z
updated: 2026-07-26T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Live-browser fragment DOM swap over /_ferro/ws

expected: With a booted app that registers a `ProjectionRuntime` (with
`with_fragment_renderer(...)` wired to render the fragment child template and
broadcast a `fragment` event) and serves a page containing a `LiveFragment`
(`projection`/`key`/child), dispatching a domain event causes the fragment
container's `innerHTML` to update in place — no page reload, no client-side
render. The automated proof already covers every layer up to and including the
`fragment` HTML frame arriving on `projection.{name}.{key}`
(`ferro-projection::runtime::tests::live_fragment_hook`); this item confirms only
the final client-side `innerHTML` swap in a real browser (Chrome MCP usable once
the app is booted).

result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
