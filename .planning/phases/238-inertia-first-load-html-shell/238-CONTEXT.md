# Phase 238: Inertia first-load HTML shell — Context

**Gathered:** 2026-06-21
**Status:** Ready for planning
**Mode:** `--auto` (recommended options selected; review decisions below)

<domain>
## Phase Boundary

`ferro-inertia` (and the `framework::inertia` wrapper) emits a complete **first-load
HTML document** — embedded `data-page` page object plus resolved Vite asset tags —
when a request is **not** `X-Inertia`, while continuing to emit the `X-Inertia` JSON
contract when it is. A single content-negotiated handler serves both. The root
template (title, `<head>` extras, mount node) is configurable with a working default,
and the app-level `InertiaConfig` is actually reachable from the render path. Docs
cover the same-origin convention and a Vite `server.proxy` recipe for the split-port
dev flow.

**This is a wiring + surfacing + docs + hardening task, NOT a from-scratch build.**
The reconciliation below establishes that most substrate already exists.

</domain>

<reconciliation>
## Pre-Planning Reconciliation (ROADMAP "reconcile before planning" — RESOLVED)

The ROADMAP framed this as "ferro-inertia has no server-rendered first-load HTML
document." **That framing is stale.** Scout of the live tree (ferro-inertia +
framework/src/inertia) found the shell substrate already present and wired end-to-end:

**Already implemented (do NOT rebuild):**
- `ferro_inertia::InertiaResponse::to_html_response()` (`ferro-inertia/src/response.rs:374`)
  emits a full `<!DOCTYPE html>` document with `<div id="app" data-page="{…escaped json…}">`.
- Content negotiation: `render_internal` (`ferro-inertia/src/response.rs:293`) emits JSON
  when `is_inertia`, HTML document otherwise — **single handler, already content-negotiated**
  (satisfies Success Criteria 1 & 2 in code).
- Dev mode: emits Vite client + `@react-refresh` preamble + entry module tags against
  `config.vite_dev_server` (`response.rs:402-432`).
- Prod mode: `resolve_assets()` (`ferro-inertia/src/manifest.rs:64`) reads Vite
  `manifest.json` (cached via `OnceLock`) and emits hashed `<script>`/`<link>` tags
  (`response.rs:433-461`) (satisfies Success Criterion 3 in code).
- Custom-template escape hatch via `InertiaConfig::html_template` with `{page}`/`{csrf}`
  placeholders (`config.rs:138`).
- Framework wrapper `framework::Inertia::render` → `ferro_inertia::Inertia::render_with_options`
  → content negotiation, with CSRF injection (`framework/src/inertia/context.rs:125`).

**Confirmed gaps (the actual work of this phase):**
1. **Config is unreachable through the common render path.** `framework::Inertia::render`
   and `render_ctx` hardcode `InertiaConfig::default()` (`context.rs:126`, `:200`). A
   downstream app can only customize via env vars or by calling `render_with_config` on
   every handler. There is no global app-level config.
2. **`App::set_inertia_config()` and `InertiaConfig::from_env()` are documented but DO NOT
   EXIST** (`docs/src/features/inertia.md:43-44`). This is the documented-but-missing
   surface that caused the downstream `u` app to perceive "no built-in path for part 1."
3. **Root-template configurability is incomplete** vs Success Criterion 4: title is only
   settable via `app_name`; `<head>` extras are unsupported (no field); mount node id is
   hardcoded `id="app"`.
4. **Docs gap** (Success Criterion 5): no same-origin convention, no Vite `server.proxy`
   recipe, no session-cookie-flow explanation. Plus doc drift — the struct-literal example
   (`inertia.md:53-59`) omits the now-required `app_name` and `manifest_path` fields and
   references the two nonexistent APIs above (won't compile).
5. **No end-to-end test** proving content negotiation (same handler → HTML vs JSON) and
   both asset modes.

**ferro-assets note:** `ferro-assets/src/lib.rs:15` mentions "SSR manifests" in a doc
comment only — no shared Vite-manifest resolver exists there. ferro-inertia has its own
`manifest.rs` and **zero ferro dependencies** (leaf crate). Decision D-08 below keeps it
that way.

</reconciliation>

<decisions>
## Implementation Decisions

### API Shape (first-load entry point)
- **D-01:** Keep a **single content-negotiated `render`** — emit the full HTML document
  when the request is not `X-Inertia`, emit JSON when it is. Do NOT add a separate
  `render_document()` method. This is already how `render_internal` behaves; Success
  Criterion 2 mandates one handler. The phase preserves this shape, it does not change it.

### Config Plumbing (the core gap)
- **D-02:** Introduce app-level Inertia config set once at bootstrap and read by the
  common render path. Implement the **already-documented** API: `App::set_inertia_config(config)`
  + `InertiaConfig::from_env()`. Back it with a process-global (`OnceLock`-style) so
  `framework::Inertia::render` / `render_ctx` resolve the configured value instead of
  hardcoding `InertiaConfig::default()`.
- **D-03:** `InertiaConfig::from_env()` is an explicit constructor reading `APP_NAME`,
  `APP_URL`/`VITE_DEV_SERVER`, `VITE_ENTRY_POINT`, `INERTIA_VERSION`, `APP_ENV` — mirroring
  the framework `from_env()` convention (CLAUDE.md "project-agnostic crates" rule). The
  current env-reading logic lives in `Default::default()`; move/share it so `from_env()`
  and `default()` agree.
- **D-04:** When no config is set via `set_inertia_config`, the render path falls back to
  `from_env()`/`default()` — so existing apps keep working with zero changes.

### Root Template Configurability (Success Criterion 4)
- **D-05:** Extend `InertiaConfig` with structured fields the default template honors:
  a title (override distinct from `app_name` if both warranted — planner may collapse to
  one), `head_extras` (raw HTML injected into `<head>`, e.g. meta/favicon/font tags), and
  a configurable mount node id (default `"app"`).
- **D-06:** Keep the existing `html_template` string-replace escape hatch for full override.
  Do NOT introduce a templating engine — structured fields + escape hatch cover the need
  (avoids over-engineering).
- **D-07:** Preserve the current `data-page` HTML-attribute escaping; verify it remains
  correct for the double-quoted attribute (`&`,`<`,`>`,`"` at minimum).

### Asset Resolution & Crate Boundaries
- **D-08:** Keep Vite-manifest resolution **inside `ferro-inertia`** (`manifest.rs`). Do
  NOT add a `ferro-assets` dependency — `ferro-inertia` is a leaf crate with zero ferro
  deps and must stay framework-agnostic/portable. The two-mode (dev tags vs manifest
  hashed tags) resolution already keys off the existing `development` flag + `manifest_path`.
- **D-09:** Confirm the manifest `OnceLock` cache does not break tests/multi-config use
  (it currently caches the first `manifest_path` seen globally). Planner should assess
  whether this global cache is acceptable or needs to key on path.

### Docs (Success Criterion 5)
- **D-10:** Document **both** stories: (a) backend-serves-the-shell same-origin (the
  primary first-load story this phase enables — no cross-origin cookie problem), and
  (b) a Vite `server.proxy` recipe for the split-port dev flow, explicitly showing the
  session cookie flowing across the proxy.
- **D-11:** Fix existing doc drift in `docs/src/features/inertia.md`: the struct-literal
  example must include all current fields; the `from_env()` / `set_inertia_config()`
  references become accurate once D-02/D-03 land. Update ferro-mcp `code_templates` /
  generation context if it surfaces the Inertia bootstrap.

### Testing
- **D-12:** Add an end-to-end test proving: same handler returns a full HTML document
  (with `data-page`) on a non-`X-Inertia` GET and the unchanged JSON contract on an
  `X-Inertia` GET; plus dev-mode tag emission and prod-mode manifest-resolved tags.

### Claude's Discretion
- Exact field naming/shape on `InertiaConfig` (title vs app_name collapse, `head_extras`
  type) — planner/executor decide.
- Whether the global config store keys the manifest cache (D-09 assessment).
- Whether `from_env()` reads `APP_URL` vs `VITE_DEV_SERVER` (or both) for the dev-server URL.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase source & vision
- `.planning/backlog/2026-06-21-inertia-first-load-shell.md` — the promoted backlog item;
  full downstream-`u` field report, proposed shape, and `u` references.
- `.planning/ROADMAP.md` §"v16.2 ferro-inertia First-Load HTML Shell (Phase 238)" (≈line 3319)
  — goal, depends-on, reconcile note, 5 success criteria.
- `.planning/VISION.md` — projection/intent core; multimodal-as-v2 (Inertia render is the
  visual modality of the JSON contract).

### Existing implementation (reconcile against — mostly present)
- `ferro-inertia/src/response.rs` — `to_html_response()` (`:374`), content negotiation in
  `render_internal` (`:293`), dev/prod template branches (`:402`/`:433`).
- `ferro-inertia/src/config.rs` — `InertiaConfig` fields + builders; env-reading `Default`.
- `ferro-inertia/src/manifest.rs` — `resolve_assets()` + `ViteManifest` (`OnceLock` cache).
- `ferro-inertia/src/lib.rs` — public exports.
- `framework/src/inertia/context.rs` — `framework::Inertia::render` (`:125`, hardcodes
  `default()`), `render_ctx` (`:187`), `convert_response` (`:207`).
- `framework/src/inertia/mod.rs`, `framework/src/inertia/config.rs` — re-export surface.
- `framework/src/lib.rs:121-122` — public re-exports of Inertia types (where new
  `set_inertia_config` would surface).

### Docs to fix / extend
- `docs/src/features/inertia.md` — has stale struct literal (`:53-59`) and references the
  missing `from_env()` / `set_inertia_config()` (`:43-44`); needs same-origin + proxy
  sections. (`docs/book/...` is generated — regenerate, do not hand-edit.)

### Conventions
- `CLAUDE.md` "Project-agnostic crates" — `ferro-*` crates read `APP_NAME`/`APP_URL` via
  their own `from_env()`; no hardcoded tenant identity (governs D-03/D-08).
- `ferro-assets/src/lib.rs:15` — "SSR manifests" mention is doc-only; no shared resolver
  to reuse (governs D-08: keep resolution in ferro-inertia).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `to_html_response()` + dev/prod template branches — the document renderer exists; extend
  it for head_extras / mount-id rather than writing a new one.
- `manifest.rs::resolve_assets()` — prod manifest resolution already done and tested.
- `framework::Inertia::render` / `render_ctx` — the wrapper exists; the change is to make it
  read app-level config instead of `default()`.
- `InertiaShared` + CSRF injection path — already feeds `to_html_response(csrf)`.

### Established Patterns
- `with_*(mut self) -> Self` consuming builders on `InertiaConfig` — extend in that style.
- Env-driven config via `APP_NAME`/`APP_ENV`/`VITE_DEV_SERVER` — already used in `Default`.
- Framework wraps the leaf crate and converts `InertiaHttpResponse` → `HttpResponse`.

### Integration Points
- `framework/src/lib.rs` re-exports (where `App::set_inertia_config` surfaces).
- Global config store (new) read by `context.rs` render path.
- `docs/src/features/inertia.md` (doc), ferro-mcp generation context (if it templates Inertia bootstrap).

</code_context>

<specifics>
## Specific Ideas

- The downstream `u` app deferred its **entire first-load page shell** (Phase 5 OQ-4)
  because it found no built-in path for part 1. The success test for this phase is that a
  consumer needs to supply **only page props** — no hand-rolled root HTML, no per-app Vite
  manifest parsing. The killer outcome: "open the web app cold in a browser and land on a
  hydrated, logged-in page" works end-to-end against framework infrastructure alone.
- `u` references for the proxy/same-origin doc recipe: `u/frontend/vite.config.ts` has no
  `server.proxy` block today — the doc recipe should make that block the documented default.

</specifics>

<deferred>
## Deferred Ideas

- True server-side rendering (executing the JS bundle on the server for SSR'd markup
  inside `#app`) — out of scope; this phase embeds `data-page` for client hydration only,
  matching standard Inertia first-load. SSR is a separate, larger effort.
- A shared `ferro-assets` Vite-manifest resolver consumed by multiple crates — only if a
  second consumer needs manifest resolution (D-08 keeps it local for now).
- None of the above came from scope-creep during discussion; discussion stayed within phase scope.

</deferred>

---

*Phase: 238-inertia-first-load-html-shell*
*Context gathered: 2026-06-21*
