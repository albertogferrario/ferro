# Handoff — Nearly app + Ferro DX fixes

State snapshot for a fresh context. Everything below is **committed and pushed
to `master`** (working tree clean). Session role: CTO/PM building the `nearly`
reference app on Ferro, then fixing framework frictions found while doing so.

**Milestone "Ship-ready Nearly" — ✅ done:** CSRF (Inertia `X-CSRF-TOKEN` via a
`<meta>` + axios default; verified 419/302), friendly Inertia 404 pages
(`fallback!` + missing-entity) + login validation, a CI `frontend` job (`npm ci`
+ `npm run build` for both frontends), and a multi-stage `nearly/Dockerfile` +
deploy docs (`SERVER_HOST=0.0.0.0`). Docker daemon was unavailable in-sandbox, so
the image build wasn't executed — steps verified locally.

## What exists now

### `nearly/` — a reference app (fully Inertia.js + React)
Location-based, deliberately chat-less social app (map + "trillo" ping). It is
**100% Inertia** — no JSON-UI (Cargo feature `["projections"]` only; projections
are backend truth, not UI). Docs: `nearly/PRODUCT.md`, `nearly/ARCHITECTURE.md`,
`nearly/README.md`.

- Backend: `nearly/src/` — controllers return `Inertia::render`; models/migrations/
  projections; `ShareInertiaData` shares auth+CSRF; demo seed (Milan).
- Frontend: `nearly/frontend/` — React 18 + Inertia + Vite + `react-leaflet`.
  Pages in `src/pages/`, app-shell `Layout.tsx`, design system `styles.css`.
- Tests (`nearly/src/tests/mod.rs`): projection intents, presence freshness,
  no-chat-surface guard, Authenticatable-derive.

### Ferro framework fixes landed (all dogfooded + verified)
See `FERRO-BACKLOG.md` (status + remaining) and `nearly/FERRO-FEEDBACK.md`
(original evidence). Done:
- `JsonUi::render_file` actionable error (path + CWD).
- **Leaflet self-hosted** — vendored in `ferro-json-ui/assets/leaflet/`, served at
  `/_ferro/leaflet/*`; Map plugin defaults to it (no CDN/SRI). `FERRO_LEAFLET_CDN=1`
  opts back into unpkg; `FERRO_LEAFLET_BASE` overrides.
- `ferro-inertia` warns + **`Inertia::preflight()`** fail-fast boot check when the
  prod Vite manifest is missing (opt-in; `nearly` calls it).
- **`#[derive(Authenticatable)]`** (ferro-macros) + **`ModelUserProvider<E>`**
  (framework) → `Auth::user_as::<T>()` with zero hand-written impl/provider.
- Sample `app/frontend` builds again (`vite build`, correct `outDir`).

### Remaining backlog (P2 DX — all optional polish)
1. Typed Inertia redirect helper + the 303-for-PUT/PATCH/DELETE note.
2. `req.data()` that content-negotiates form vs JSON (today Inertia posts JSON →
   must use `req.input()`, not `req.form()`).
3. Clarify JSON-UI-vs-Inertia docs; fix the sample `app/frontend` `tsconfig` so
   `npm run typecheck` passes.
4. (P3) geo `FieldMeaning`, theme tokens as CSS for any frontend, design-lint
   discoverability. See `FERRO-BACKLOG.md`.

## How to run / verify

```bash
# Rust checks (per crate — the full workspace build is heavy):
cargo fmt -p <crate> -- --check
cargo clippy -p <crate> --all-targets -- -D warnings
cargo test -p <crate>

# Run Nearly (from the nearly/ dir!):
cd nearly/frontend && npm install && npm run build   # → ../public/{.vite/manifest.json,assets/*}
cd .. && APP_ENV=production cargo run -p nearly       # http://localhost:8080  (demo: alex@nearly.app / password123)
# dev with HMR: `npm run dev` in frontend/ + `cargo run -p nearly` (APP_ENV=local)
```

## Gotchas for the next context
- **Run apps from their crate dir** — JSON-UI views & the Vite manifest resolve
  relative to CWD.
- **`master` has a version-bump bot** that pushes `chore: bump version` commits.
  Before pushing: `git fetch origin master`; if it moved, `git rebase origin/master`.
  Verify the push landed by checking `HEAD == origin/master` (do **not** trust
  `if git push | tail` — the pipe hides the push exit code).
- **`nearly/public/` and `app/public/`** are gitignored build output (regenerated
  by `npm run build`); `node_modules/` too.
- Committing directly to `master`, no PRs (per instruction).
