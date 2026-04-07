# Phase 122 — Deploy scaffold core rewrite

## Context
Deploying gestiscilo (2026-04-07) made it obvious the current scaffold is a
"happy path for one app" snapshot. mkmenu's deployed Dockerfile is the same
template hand-patched in five places: a `sed` rewrite of the ferro path dep, a
`GITHUB_TOKEN` ARG, a custom binary name, and `lang/` instead of `themes/`.
Every new Ferro app reproduces the same patches by hand, and gestiscilo needs
even more (4 ferro path deps, 2 binaries, chromium runtime, postgres). The
scaffold should encode these decisions so no app starts deployment by editing
templates.

## Goal
Rewrite ferro-cli `docker_init` / `do_init` commands and their templates so that
generated `Dockerfile` and `.do/app.yaml` work for real Ferro apps without any
hand-patching.

## Reference apps (validation targets)
- `../../gestiscilo-it/app` — server-rendered (no frontend stage), multi-bin
  (`gestiscilo` + `screenshot-worker` requiring chromium), uses `themes/`,
  postgres in production, 4 ferro path deps (`ferro`, `ferro-json-ui`,
  `ferro-whatsapp`, `ferro-ai`).
- `../../gestiscilo-it/mkmenu` — frontend build stage, single bin, `lang/`,
  already deployed on DO App Platform `fra1` with managed postgres and Spaces.

## Scope

### 1. `Dockerfile.tpl`
- Make frontend stage **conditional** on `frontend/package.json` existing.
- **Multi-binary** support — read `[[bin]]` entries from `Cargo.toml` and emit
  one `--bin <name>` per entry plus a runtime `COPY` per binary.
- **Runtime extras hook** — `--runtime-deps "chromium,fonts-liberation"` flag
  on `docker:init` plus a clearly marked block in the runtime stage so users can
  add apt packages without losing them on regeneration.
- **Detect and copy** `themes/`, `lang/`, `public/`, `migrations/` only if
  present in the project root.
- Bake in `ARG GITHUB_TOKEN=""` and the `git config insteadOf` workaround for
  private ferro git repos.
- Read `rust-toolchain.toml` if present and use that toolchain version instead
  of the hardcoded `rust:1.88-slim-bookworm`.
- **Workspace-aware cargo-chef recipe** — copy `crates/`, `migration/`, and any
  sibling workspace members declared in the workspace `Cargo.toml`, not just
  `Cargo.toml + src/`.

### 2. Path → git ferro dep rewrite
- Generate `scripts/rewrite-ferro-deps.sh` at `docker:init` time. Script reads
  the project `Cargo.toml`, finds every `ferro*` dep with a `path = "..."`, and
  rewrites it to a `git = "..."` dep using `--ferro-ref` (default `main`).
- Dockerfile invokes the script in the planner and builder stages (after
  `COPY .`) so local development continues to use path deps while Docker builds
  use git deps.
- **CLI pre-flight**: before allowing `docker:build` (or as a standalone
  `ferro deploy:check`) verify that the chosen ferro git ref is pushed and
  reachable on the remote (`git ls-remote`). Fail loudly if not, with a
  message telling the user to push their ferro changes first.
- New flag: `--ferro-ref <branch|tag|sha>` on `docker:init` and `do:init`,
  persisted into the generated script header.

### 3. `app.yaml.tpl`
- `--region` flag on `do:init`, default `fra1`.
- Generate an `envs:` block from `.env.example` with `SCOPE: RUN_TIME` for each
  key. Auto-classify as `type: SECRET` when key matches
  `*_KEY|*_SECRET|*PASSWORD|*TOKEN|DATABASE_URL`.
- Optional `databases:` block when `DATABASE_URL` is detected in `.env.example`,
  referencing `${db.DATABASE_URL}` from the envs block.
- Emit one `workers:` entry per non-server `[[bin]]` so background workers
  (e.g. `screenshot-worker`) get their own DO App Platform component.

### 4. `docker_init.rs` / `do_init.rs`
- Add `--force` flag to overwrite existing files.
- Walk **up** from the current directory to locate `Cargo.toml` instead of
  requiring the user to be in the project root.
- Validate `--repo` argument format (`owner/repo`) before writing app.yaml.
- Lift the duplicated `get_package_name()` into a shared
  `project::package_name()` helper module used by both commands (and any future
  ones).

### 5. `dockerignore.tpl`
- Add: `database.db`, `*.sqlite*`, `.planning/`, `storage/`, `data/`.
- Audit drift against `.gitignore` template (Phase 124 will keep them in sync,
  this phase only adds the missing entries).

## Verification
- Delete `Dockerfile`, `.dockerignore`, `.do/app.yaml` from gestiscilo and
  mkmenu, regenerate via `ferro docker:init` + `ferro do:init`, and confirm:
  - Build succeeds end-to-end with **zero** hand edits.
  - gestiscilo image contains both `gestiscilo` and `screenshot-worker` binaries
    plus chromium runtime.
  - mkmenu image still produces a working frontend bundle and matches the
    currently deployed shape.
  - Pre-flight check correctly fails when ferro local commits are not pushed.
  - `do:init --region nyc --repo owner/foo` writes `region: nyc` and a valid
    envs block from `.env.example`.

## Out of scope (deferred to later phases)
- New MCP deploy tools (Phase 123).
- `ferro doctor`, `routes --json`, CI workflow scaffold (Phase 124).
- `make:module`, json-ui runtime split (Phase 125).
