---
status: complete
phase: 238-inertia-first-load-html-shell
source: [238-VERIFICATION.md]
started: 2026-06-21
updated: 2026-06-21
---

## Current Test

[testing complete]

## Tests

### 1. End-to-end first-load browser smoke test
expected: Start a real Ferro + Inertia app (dev mode, Vite dev-server running). Open the app root cold in a browser (no prior X-Inertia XHR). The backend returns a full HTML document; the `data-page` payload hydrates the React app; the session cookie flows and an authenticated page renders. Then build for production (Vite `manifest.json` present), set `APP_ENV=production`, and confirm the document emits hashed `<script>`/`<link>` tags (no `@vite/client` / `@react-refresh`).
result: pass
note: |
  Verified live against the sample `app` (controller GET / → Inertia::render("Home")) in Chrome. See 238-UAT.md for the full per-test breakdown and 238-first-load-hydrated.png.
  DEV: ran `npm install` + this app's Vite on :5174, started the app with VITE_DEV_SERVER=http://localhost:5174, reloaded http://127.0.0.1:8090/ cold. Backend returned a full HTML document; React hydrated from data-page (<h1>Welcome to Ferro!</h1>, User Info + Stats, visits "1,234"); dev tags pointed at the configured dev server; zero console errors.
  PROD: `vite build` (manifest + hashed main-CTIBvuga.js), staged manifest at the default resolver path, restarted with APP_ENV=production. The document emitted the hashed `/assets/main-CTIBvuga.js` resolved from the Vite manifest and contained NO `@vite/client` / `@react-refresh` / dev-server URL (SC-3 prod + T-238-03 confirmed live).

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
