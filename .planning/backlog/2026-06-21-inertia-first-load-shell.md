# Feedback: `ferro-inertia` has no server-rendered first-load HTML shell (+ dev same-origin convention)

**Source:** Downstream Inertia + WebSocket streaming consumer app (private, AI-native chat product `u`), field assessment 2026-06-21 (paired with the `ferro-mcp-transport` and broadcast backlog items from the same consumer).
**Severity:** Capability gap — blocks any real first-load/browser render of a downstream Inertia app served by Ferro; forces downstream apps to defer their production page shell.
**Ferro version inspected:** local path-dep 0.2.65 (u tree), `ferro-inertia` + `ferro-assets` as of 2026-06-21.

## Planning Note

This document is a sketch from a downstream-app perspective, not an inside-Ferro design. When promoted from backlog to a phase, the Ferro planning agent should reconcile against `.planning/VISION.md`, the existing `ferro-inertia` config surface, and `ferro-assets` (which already mentions SSR manifests) before drafting `PLAN.md`.

---

## Problem statement

Inertia renders a screen in two parts:

1. **First load** — on a fresh URL open, the *backend* must return a complete HTML document with the page object embedded (`<div id="app" data-page="{…json…}">`) plus the asset tags for the JS/CSS bundle (dev: Vite dev-server module tags; prod: hashed assets from the Vite manifest). The client reads `data-page` and renders.
2. **Subsequent visits** — XHR with `X-Inertia` headers returning JSON; no new document.

`ferro-inertia` today gives downstream apps a clean path for **part 2** (the `X-Inertia` JSON contract — `Inertia::render`/`render_ctx`), but the downstream `u` app found **no built-in path for part 1**: there is no `Inertia::render` variant that emits the initial HTML document with the embedded page object and resolved asset tags. `ferro-inertia/src/config.rs` already carries a `vite_dev_server` URL field, which implies the shell was intended — but the shell renderer + the prod manifest resolution appear to be missing or undocumented.

Concretely, `u` had to **defer its entire first-load page shell** (its `show_chat` handler returns only the Inertia JSON contract; there is no backend `/` HTML route). The visible consequence: the app cannot be opened in a browser and hydrated to a real logged-in page from the backend — only the post-load JSON flow works. This blocked downstream visual UAT (e.g. screenshotting an authenticated settings sheet) and left the killer-demo "open the web app" gate unrunnable end-to-end.

A second, smaller gap surfaced alongside it: there is **no documented same-origin / dev-proxy convention**. With Vite serving the SPA on one port and the Ferro backend on another, an Inertia/`fetch` call is cross-origin and the session cookie does not flow. Downstream apps need either a documented Vite `server.proxy` recipe pointing at the Ferro backend, or guidance that the backend serves the shell same-origin (which part 1 above enables).

## Why this is a framework concern, not a downstream concern

The HTML shell, the `data-page` embedding, and the dev-vs-manifest asset resolution are **modality-agnostic Inertia transport plumbing** — the same shape every Ferro+Inertia app needs, identical to how Ferro already owns the JSON contract. A downstream app hand-rolling its own root-HTML template + Vite manifest parsing per app is exactly the duplication Ferro exists to remove. The per-app part is only the page props; the shell is framework infrastructure. `ferro-assets` already speaks of SSR manifests, so much of the substrate may exist — this is likely a wiring + surfacing task, not a from-scratch build.

## Proposed shape (for the planning agent to reconcile)

- A first-load render entry point (e.g. `Inertia::render_document(...)` or making `Inertia::render` content-negotiate: emit the full HTML document when the request is **not** `X-Inertia`, emit JSON when it is).
- Asset resolution with two modes off the existing `vite_dev_server` config: **dev** → emit Vite client + entry module tags against the configured dev-server URL; **prod** → read the Vite `manifest.json` (via `ferro-assets`?) and emit hashed `<script>`/`<link>` tags.
- A configurable root-template (title, `<head>` extras, the `#app` mount node) with a sane default.
- Docs: the same-origin story + a Vite `server.proxy` recipe for the split-port dev flow, and how the session cookie flows.

## u reference (for the promoted phase)

- `u/src/controllers/profile.rs::show_chat` — returns ONLY the Inertia JSON contract; no first-load HTML document.
- `u/frontend/index.html` (`<script type="module" src="/src/main.tsx">`) + `u/frontend/src/main.tsx` (`createInertiaApp`) — the Vite-served SPA shell that today substitutes for the missing backend shell in dev.
- `u/frontend/vite.config.ts` — no `server.proxy` block (the missing same-origin convention).
- u Phase 5 deferral of record: `.planning/phases/05-chat-ui/05-03-PLAN.md` "OQ-4" (first-load HTML shell explicitly DESCOPED for Phase 5, recorded as a post-Phase-5 / production-hardening deferral).
- Impact: blocked u Phase 6 settings-sheet visual UAT (the `ProfileSheet` MCP-token reveal could not be screenshotted end-to-end); u Phase 5's own visual gates (`05-HUMAN-UAT.md`) remain pending for the same reason.
