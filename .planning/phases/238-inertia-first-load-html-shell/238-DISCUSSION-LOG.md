# Phase 238: Inertia first-load HTML shell - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-21
**Phase:** 238-inertia-first-load-html-shell
**Mode:** `--auto` (recommended option auto-selected per area)
**Areas discussed:** API shape, Config plumbing, Root-template configurability, Asset resolution & crate deps, Same-origin/proxy docs

---

## API Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Content-negotiate existing `render` | One handler: HTML doc when not X-Inertia, JSON when it is. Already implemented in `render_internal`. | ✓ |
| Add separate `render_document()` | A distinct entry point for the first-load document. | |

**User's choice (auto):** Content-negotiate existing `render`.
**Notes:** Success Criterion 2 mandates a single handler; the behavior already exists — phase preserves it. (D-01)

---

## Config plumbing

| Option | Description | Selected |
|--------|-------------|----------|
| Global config at bootstrap | Implement documented `App::set_inertia_config` + `InertiaConfig::from_env()`; render path reads it instead of `default()`. | ✓ |
| Env-driven default only | Keep `InertiaConfig::default()` reading env; no global setter. | |
| Per-call `render_with_config` | Require apps to pass config on every handler call. | |

**User's choice (auto):** Global config at bootstrap.
**Notes:** This is the real fix for the downstream `u` gap — `render`/`render_ctx` currently hardcode `default()`, and `set_inertia_config`/`from_env` are documented but missing. Falls back to env when unset for zero-change compatibility. (D-02, D-03, D-04)

---

## Root-template configurability

| Option | Description | Selected |
|--------|-------------|----------|
| Structured fields + escape hatch | Add title / `head_extras` / mount-id fields the default template honors; keep `html_template` for full override. | ✓ |
| Raw `html_template` only | String-replace `{page}`/`{csrf}` only — no structured fields. | |
| Full templating engine | Embed a template engine for arbitrary root templates. | |

**User's choice (auto):** Structured fields + escape hatch.
**Notes:** Satisfies Success Criterion 4 (title, head extras, mount node) without over-engineering. (D-05, D-06, D-07)

---

## Asset resolution & crate deps

| Option | Description | Selected |
|--------|-------------|----------|
| Keep resolution in ferro-inertia | Use the existing `manifest.rs` resolver; no new dependency. | ✓ |
| Pull a shared ferro-assets resolver | Depend on ferro-assets for manifest parsing. | |

**User's choice (auto):** Keep resolution in ferro-inertia.
**Notes:** ferro-inertia is a leaf crate with zero ferro deps; ferro-assets only mentions SSR manifests in a doc comment (no real resolver). Keep it framework-agnostic. (D-08, D-09)

---

## Same-origin / proxy docs

| Option | Description | Selected |
|--------|-------------|----------|
| Document both | Backend-serves-shell same-origin story + Vite `server.proxy` recipe; fix doc drift. | ✓ |
| Same-origin only | Only document the backend-serves-shell convention. | |
| Proxy only | Only document the Vite proxy recipe. | |

**User's choice (auto):** Document both.
**Notes:** Success Criterion 5. Also fixes stale struct-literal example and the missing-API references in `docs/src/features/inertia.md`. (D-10, D-11, D-12)

## Claude's Discretion

- Exact `InertiaConfig` field naming/shape (title vs app_name collapse, `head_extras` type).
- Whether the global config store keys the manifest `OnceLock` cache.
- Whether `from_env()` reads `APP_URL` vs `VITE_DEV_SERVER` (or both).

## Deferred Ideas

- True server-side rendering (executing the JS bundle on the server) — separate larger effort.
- A shared `ferro-assets` Vite-manifest resolver — only if a second consumer needs it.
