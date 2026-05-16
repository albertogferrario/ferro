# Frontend types: generator-owned convention

`ferro generate-types` is the single source of truth for the TypeScript types
the Inertia frontend imports. The directory `frontend/src/types/` is reserved
for its output. Hand-written types live elsewhere.

## What the generator produces

`ferro generate-types` writes exactly two files into `frontend/src/types/`:

| File | Contents |
| ---- | -------- |
| `inertia-props.ts` | Per-component Inertia prop interfaces derived from `#[handler]` return types and `Inertia::render` call sites |
| `routes.ts` | Typed route map keyed by route name, derived from the route registry |

Both files are regenerated from current Rust source on every `ferro serve`
restart (and on every `cargo run` that boots the app). The generator is
deterministic for a given Rust source tree but is not guaranteed stable
across unrelated changes — that is intentional, because the output is not
tracked.

## Why `frontend/src/types/` is gitignored

The Ferro scaffold's `.gitignore` template marks `frontend/src/types/` as
generator-owned. The rationale:

- **No drift.** Every server start regenerates from current Rust source.
  Drift between Rust and TypeScript types is impossible by construction.
- **No git noise.** Tracking the generated output would produce a "modified"
  entry against the two files on every server restart, polluting `git status`.
- **No review burden.** Derived files in pull requests duplicate the
  information already present in the Rust diff.

This is the same pattern as `target/`, `dist/`, or OpenAPI-derived clients:
ignore the output, regenerate from the source.

## Where hand-written types belong

Project-specific hand-written TypeScript types (domain types not produced by
the generator) live under `frontend/src/lib/types/`. Any path outside
`frontend/src/types/` works; `frontend/src/lib/types/` is the recommended
convention.

The `ferro doctor` check `frontend_types_convention` flags any file in
`frontend/src/types/` whose name is not on the generator's allowlist
(`inertia-props.ts`, `routes.ts`). The check is advisory (warning, not error)
and never blocks the doctor exit code.

## Fresh-clone bootstrap

On a fresh clone, `frontend/src/types/` does not exist yet — it is
gitignored, so it was never committed. Run the backend once to populate it:

```bash
cargo run
```

The first start emits `frontend/src/types/inertia-props.ts` and
`frontend/src/types/routes.ts`. After that, `npm run dev` and `npm run build`
can resolve their imports.

If the frontend build fails with errors like:

```
TS2307: Cannot find module './types/inertia-props' or its corresponding type declarations.
```

it means types have not been generated yet — run `cargo run` once.

## Docker production builds

Because `frontend/src/types/` is gitignored, it is also absent from the
Docker build context. The scaffolder addresses this by emitting a dedicated
Rust-toolchain stage (`types-gen`) in the rendered `Dockerfile`:

```dockerfile
FROM rust:<tag> AS types-gen
WORKDIR /app
RUN cargo install ferro-cli --version <pinned> --locked
COPY . .
RUN ferro generate-types

FROM node:20-bookworm-slim AS frontend-builder
WORKDIR /frontend
# ...
COPY --from=types-gen /app/frontend/src/types ./src/types
RUN npm run build
```

The `types-gen` stage regenerates the types from Rust source, and the
frontend stage copies them in before `npm run build` runs.

The pinned `ferro-cli` version is read from your project's `Cargo.lock` (the
`ferro-rs` package's version). You can override it explicitly:

```bash
ferro docker:init --ferro-version 0.2.33 --force
```

### Upgrading an existing project

If your project was scaffolded against an older Ferro that did not emit the
`types-gen` stage, re-run the scaffolder to regenerate the Dockerfile:

```bash
ferro docker:init --force
```

This overwrites the existing `Dockerfile` with the current renderer output.
Inspect the diff before committing.

## Related commands

- [`ferro doctor`](doctor.md) — runs the `frontend_types_convention` check
- [`ferro docker:init`](do-init.md) — regenerates the Dockerfile with the
  current `types-gen` stage
