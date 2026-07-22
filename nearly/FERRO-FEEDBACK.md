# Ferro — dogfooding feedback from building Nearly

Written while building the Nearly reference app (first on JSON-UI, then migrated
to Inertia/React). Each item is a friction I actually hit, with evidence and a
concrete suggestion. Ordered by impact. Nothing here is a blocker — the app
shipped and is green — but each would have saved real time.

Legend: **[DX]** developer experience · **[Bug]** likely defect ·
**[Docs]** documentation gap · **[Feat]** feature request.

---

## High impact

### 1. The Inertia asset contract is implicit, and the sample's `vite.config` is wrong for production **[Bug][Docs]**
`InertiaConfig::from_env()` defaults `manifest_path` to `public/.vite/manifest.json`
and `resolve_assets()` returns `/{manifest.file}`, while `try_serve_static_file`
serves `public/…`. For these to line up, Vite must build with
`outDir: '../public'` (assets → `public/assets/*` → `/assets/*`).

But the sample `app/frontend/vite.config.ts` uses `outDir: '../public/assets'`,
which puts the manifest at `public/assets/.vite/manifest.json` (≠ the default
`manifest_path`) and files at `public/assets/assets/*`. In production this
resolves to 404s. I only got it right by reading `manifest.rs` + `static_files.rs`
and deriving the mapping.

**Suggestions:** ship a correct, documented `vite.config.ts` in the template;
validate at boot (if `APP_ENV=production` and the manifest entry for the entry
point is missing, log a loud, actionable error instead of silently serving
`/assets/main.js`); and document the outDir ⇄ manifest_path ⇄ static-root
triangle in the Inertia docs.

### 2. The shipped Map plugin hard-depends on a SRI-pinned CDN **[Bug][Feat]**
`ferro-json-ui`'s `MapPlugin` loads Leaflet from `unpkg.com` with `integrity`
hashes. Behind a TLS-terminating proxy (common in CI/agent sandboxes) or offline,
the integrity check fails and the browser refuses the script → a blank map with
**no diagnostic**. The map is the plugin's whole point, so this silently breaks
the flagship demo in exactly the environments a Rust-first team runs in.

**Suggestions:** offer a bundled/self-hosted Leaflet option (serve the assets
from the framework like `ferro-base.css`), or a config to drop SRI / point at a
local URL. At minimum, document the external dependency and surface a console
warning when the tile/script layer fails.

### 3. `JsonUi::render_file` is CWD-relative and fails opaquely **[DX][Bug]**
`JsonUi::render_file("src/views/x.json", …)` resolves relative to the process CWD.
Running the binary from the workspace root (not the crate dir) yields a 500 with
`Failed to load spec: failed to read spec file: No such file or directory` — no
mention of *which* path/CWD it tried. My first smoke test failed purely on this.

**Suggestions:** resolve relative to `CARGO_MANIFEST_DIR` (or a configurable
views root), and include the attempted absolute path + CWD in the error.

### 4. `Auth::user_as::<T>()` requires `Authenticatable`, which models don't implement and nothing derives **[DX][Docs]**
The natural call to get the logged-in user is `Auth::user_as::<User>()`, but the
generated/hand-written `Model` doesn't implement `Authenticatable`, so it fails to
compile with no pointer to the fix. I fell back to `Auth::id()` +
`Entity::find_by_pk`. The template's `ShareInertiaData` even ships with the
auth-sharing block **commented out as a TODO**, so the batteries aren't included.

**Suggestions:** a `#[derive(Authenticatable)]` (or a blanket impl for models with
an `id`), a working `Auth::user()` that returns a serializable principal, and a
complete `ShareInertiaData` in the template (auth + flash actually wired).

---

## Medium impact

### 5. Two starter paths (JSON-UI vs Inertia) with no stated default, and the Inertia sample doesn't build **[Docs][Bug]**
`ferro new` scaffolds an Inertia/React frontend, while `CLAUDE.md` + the sample
`app` center JSON-UI/projections. It's unclear which is "the way" and when to pick
which. Worse, `app/frontend`'s `npm run build` (`tsc && vite build`) **fails** on
`JSX.IntrinsicElements`/`inertia-props` type errors out of the box — so the
official Inertia example is not currently buildable.

**Suggestions:** state the default and the decision criteria (server-driven
JSON-UI for AI-authored/CRUD surfaces; Inertia for bespoke UX); fix the sample's
tsconfig/build (or drop `tsc` from the build script); keep one example of each
that actually builds in CI.

### 6. Design-lint findings surface one-at-a-time in the natural loop **[DX][Docs]**
`ferro design:lint` exists and is great (`--json`, `--deny`) — but I didn't
discover it until after the fact, because the JSON-UI docs don't mention it and
the idiomatic gate is a **test that panics on the first failing file**
(`app/src/tests/design_lint.rs`). So authoring felt like whack-a-mole:
`breadcrumb-on-subpages` → fix → `register-grid-fill` → fix →
`destructive-confirmation` → fix, one rebuild each.

**Suggestions:** reference `ferro design:lint` from the JSON-UI feature docs and
the `make:json-view` output; ship the design-lint test helper so it reports **all**
files' findings before asserting; consider emitting findings as dev-mode render
warnings. Also: document the rule catalog (IDs, intents, what each wants) — I had
to read `design/rules.rs` to understand `register-grid-fill`.

### 7. `req.form()` vs `req.input()` is a silent Inertia trap **[DX][Docs]**
Inertia posts JSON, so controllers using `req.form()` (urlencoded) fail to
deserialize. The fix is `req.input()`. This isn't obvious and produces a runtime
error, not a compile error.

**Suggestions:** document prominently for Inertia; consider a `req.data()` that
content-negotiates form vs JSON transparently, or make `form()` fall back to JSON.

### 8. No geo `FieldMeaning`, and the projection→UI story has no map target **[Feat]**
For a location app I modeled lat/lng as `FieldMeaning::Custom("latitude")`. Geo
coordinates are common enough (and Ferro even ships a Map plugin) to warrant
first-class `Latitude`/`Longitude`/`GeoPoint` meanings and a projection render
target that emits a map.

---

## Lower impact / polish

### 9. Adding an app to the workspace is a manual, heavy operation **[DX]**
`ferro new` builds a standalone project with crates.io deps *outside* the repo;
there's no supported way to scaffold a **workspace member** (path deps, inherited
version). I hand-assembled the crate from the template `.tpl` files. Also, every
new member joins the ~30-crate `cargo test --all-features` build, which `CLAUDE.md`
itself notes strains CI disk.

**Suggestions:** a `ferro new --workspace-member` (or `ferro make:app`) that emits
path deps + `version.workspace = true`; consider excluding example apps from the
default workspace test set (opt-in like the benchmark apps already are).

### 10. Minor API discoverability **[Docs]**
- `QueryBuilder` fetch-all is `.all()`; I first reached for `.get()`. A one-liner
  in docs (or a `.get()` alias) helps.
- Inertia redirects: `Redirect::to("/x").into()` works for POST (302), but there's
  no typed Inertia redirect and no note that PUT/PATCH/DELETE want 303 for Inertia.
- `render_file` path caching in dev reloads on mtime — good — but this isn't
  documented next to the CWD behavior in (3).

### 11. Theme tokens are JSON-UI-only **[Feat]**
`ThemeMiddleware`/`Theme` inject CSS vars for JSON-UI pages. When I moved to
Inertia I lost them and hand-rolled a palette. Exposing the theme tokens as a
plain CSS file/endpoint (`/_ferro/theme.css`) any frontend can import would keep
one source of truth across rendering modes.

---

## What worked well (worth preserving)

- **Projections (`ServiceDef` + `derive_intents`)** are a genuinely nice modeling
  primitive — clear, and they survived the JSON-UI→Inertia switch untouched as
  backend truth.
- **Migrations + SeaORM model helpers** (`Model`/`ModelMut`, `QueryBuilder`) are
  ergonomic; the vertical (migration → model → controller → route) is fast.
- **`Inertia::render` + content negotiation + `SavedInertiaContext`** made the
  React switch smooth once the asset contract was understood.
- **The design linter itself** is a great idea — enforcing intent-coherent UIs at
  write time is exactly the right altitude. It mostly needs discoverability.
