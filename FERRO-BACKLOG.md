# Ferro — improvements backlog (from building the Nearly app)

Source: dogfooding Ferro by building the `nearly` reference app (JSON-UI, then
Inertia/React). Full write-up with evidence: [`nearly/FERRO-FEEDBACK.md`](./nearly/FERRO-FEEDBACK.md).

This document is the **actionable state**: what was fixed in this pass, and a
prioritized backlog for the next Ferro work, each with acceptance criteria.

---

## ✅ Fixed in this pass

| # | Fix | Files |
|---|-----|-------|
| A | `JsonUi::render_file` now returns an actionable error (attempted absolute path + CWD + hint) instead of a bare "No such file or directory". | `framework/src/json_ui/mod.rs` |
| B | Map plugin can self-host Leaflet via `FERRO_LEAFLET_BASE` (loads `{base}/leaflet.{css,js}` without SRI) — the escape hatch for TLS-terminating proxies / offline / CI. Default (SRI-pinned unpkg) unchanged. Unit-tested. | `ferro-json-ui/src/plugins/map.rs` |
| C | `ferro-inertia` now logs a loud, actionable warning when the production Vite manifest/entry is missing, instead of silently falling back to `/assets/main.js` (which 404s to a blank page). | `ferro-inertia/src/manifest.rs` |
| D | The sample Inertia example builds again: dropped the failing `tsc &&` from the build script (moved to a `typecheck` script), and corrected `vite.config` `outDir` to `../public` so the manifest + assets match the framework's production asset contract. | `app/frontend/package.json`, `app/frontend/vite.config.ts` |
| E | **Leaflet is now vendored and self-hosted** (backlog P1.2 below). Leaflet 1.9.4 (js/css/marker images) is embedded in `ferro-json-ui` and served at `/_ferro/leaflet/*`; the Map plugin **defaults** to it (no CDN, no SRI) so the map renders offline / behind TLS-terminating proxies with zero config. `FERRO_LEAFLET_CDN=1` opts back into the unpkg CDN; `FERRO_LEAFLET_BASE` still overrides. Unit + route tests, live-verified. | `ferro-json-ui/src/assets/leaflet.rs`, `ferro-json-ui/src/plugins/map.rs`, `framework/src/server.rs` |

Note: the `ferro new` **template** vite.config was already correct
(`outDir: '../public'`); only the checked-in sample `app/frontend` had drifted.

---

## 📋 Backlog for next Ferro work (prioritized)

### P1 — correctness / silent-failure

**1. Boot-time validation of the Inertia asset contract.**
`resolve_assets` now warns, but a missing manifest still yields a blank page.
- *Change:* at server start in production, verify the entry point resolves in
  `manifest_path`; fail fast (or log an error banner) with the exact paths.
- *Accept:* starting with `APP_ENV=production` and no build prints a single clear
  error naming the manifest path and the fix (`npm run build`).
- *Crates:* `ferro-inertia`, `framework`.

**2. First-class self-hosted Leaflet — ✅ DONE (see fix E above).**

**3. `Authenticatable` ergonomics — ✅ mostly done.**
- ✅ `#[derive(Authenticatable)]` (ferro-macros) generates the trait from an
  integer `id` field (`#[auth(id = "…")]` to override); re-exported from the
  framework alongside the trait (same name, like `serde::Serialize`). Kills the
  ~17-line hand-written impl. Dogfooded + tested in `nearly`.
- ✅ Sample app `ShareInertiaData` now actually shares the auth user via
  `Auth::user_as::<User>()` (the commented-out TODO is done).
- ⏳ *Remaining:* a generic `ModelUserProvider<E>` so apps don't hand-write a
  `UserProvider` for the common "load model by pk" case. Non-trivial because the
  primary-key type varies (i32/i64/uuid) — needs care with SeaORM bounds. Until
  then `Auth::user_as` still needs a registered provider (the derive removes the
  *trait* boilerplate, not the provider).
- *Crates:* `ferro-macros`, `framework` (auth).

### P2 — developer experience

**4. `req.form()` vs `req.input()` for Inertia.**
Inertia posts JSON; `req.form()` fails at runtime.
- *Change:* document prominently; consider `req.data()` that content-negotiates,
  or make `form()` fall back to JSON.
- *Crates:* `framework` (http), docs.

**5. Design-lint discoverability.**
`ferro design:lint` exists but isn't referenced where views are authored, and the
idiomatic test gate panics on the first failing file (one finding per rebuild).
- *Change:* link `design:lint` from the JSON-UI docs + `make:json-view` output;
  ship a test helper that reports **all** files' findings before asserting;
  optionally emit findings as dev-mode render warnings; document the rule catalog
  (IDs, intents, what each rule wants).
- *Accept:* an author sees every finding across every view in one command/run.
- *Crates:* `ferro-json-ui`, docs, CLI.

**6. `ferro new --workspace-member` (a.k.a. `ferro make:app`).**
`ferro new` only makes standalone, crates.io-dep projects outside the repo;
adding an in-workspace app is manual.
- *Change:* a flag/command that emits path deps + `version.workspace = true` and
  registers the member; consider excluding example apps from the default
  `cargo test --all-features` set (opt-in, like the benchmark apps).
- *Crates:* `ferro-cli`, workspace `Cargo.toml`.

**7. Clarify the JSON-UI vs Inertia story.**
State the default and the decision criteria (server-driven JSON-UI for
AI-authored/CRUD surfaces; Inertia for bespoke UX). Fix the sample's tsconfig so
`typecheck` passes (this pass only removed `tsc` from the build path).
- *Crates:* docs, `app/frontend`.

### P3 — features / polish

**8. Geo support.** First-class `FieldMeaning::Latitude/Longitude` (or `GeoPoint`)
and a projection render target that emits a map. (`ferro-projections`, `ferro-json-ui`.)

**9. Theme tokens for any frontend.** Expose `Theme` tokens as a plain CSS file/
endpoint (`/_ferro/theme.css`) so Inertia/other frontends share one source of
truth. (`ferro-theme`, `framework`.)

**10. Doc polish.** `QueryBuilder::all()` vs a `.get()` alias; typed Inertia
redirect + the 303-for-PUT/PATCH/DELETE note; document `render_file`'s
CWD-relative resolution next to its dev mtime-reload behavior.

---

## What to preserve (don't regress)

- **Projections (`ServiceDef` + `derive_intents`)** — survived a full UI rewrite
  untouched as backend truth. This is the framework's best idea; keep it central.
- The **migration → model → controller → route** vertical is fast and pleasant.
- **`Inertia::render` + content negotiation + `SavedInertiaContext`** — smooth
  once the asset contract is understood (see fixes A–D).
- The **design linter** concept — enforcing intent-coherent UIs at write time is
  the right altitude; it mainly needs discoverability (item 5).
