---
status: partial
phase: 238-inertia-first-load-html-shell
source: [238-VERIFICATION.md]
started: 2026-06-21
updated: 2026-06-21
---

## Current Test

[awaiting human testing]

## Tests

### 1. End-to-end first-load browser smoke test
expected: Start a real Ferro + Inertia app (dev mode, Vite dev-server running). Open the app root cold in a browser (no prior X-Inertia XHR). The backend returns a full HTML document; the `data-page` payload hydrates the React app; the session cookie flows and an authenticated page renders. Then build for production (Vite `manifest.json` present), set `APP_ENV=production`, and confirm the document emits hashed `<script>`/`<link>` tags (no `@vite/client` / `@react-refresh`).
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
