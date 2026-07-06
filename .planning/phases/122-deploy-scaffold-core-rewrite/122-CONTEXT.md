# Phase 122: Deploy Scaffold Core Rewrite - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning
**Mode:** Auto (decisions sourced from SCOPE.md)

<domain>
## Phase Boundary

Rewrite ferro-cli `docker:init` and `do:init` commands and their templates so generated `Dockerfile`, `.dockerignore`, and `.do/app.yaml` work for real Ferro apps (gestiscilo, mkmenu) without any hand-patching.

Source of truth: `.planning/phases/122-deploy-scaffold-core-rewrite/SCOPE.md`.
</domain>

<decisions>
## Implementation Decisions

All decisions are locked by SCOPE.md. Summary for downstream agents:

### Dockerfile.tpl
- **D-01:** Frontend stage conditional on `frontend/package.json` existing.
- **D-02:** Multi-binary support — read `[[bin]]` entries from `Cargo.toml`, emit one `--bin <name>` per entry plus runtime `COPY` per binary.
- **D-03:** `--runtime-deps "pkg1,pkg2"` flag on `docker:init`; emit a clearly marked apt-install block in the runtime stage that survives regeneration.
- **D-04:** Detect-and-copy `themes/`, `lang/`, `public/`, `migrations/` only when present.
- **D-05:** Bake `ARG GITHUB_TOKEN=""` + `git config insteadOf` workaround for private ferro git repos.
- **D-06:** Use `rust-toolchain.toml` toolchain version when present, else fall back to current hardcoded base image.
- **D-07:** Workspace-aware cargo-chef recipe — copy `crates/`, `migration/`, and any sibling workspace members declared in workspace `Cargo.toml`.

### Path → git ferro dep rewrite
- **D-08:** Generate `scripts/rewrite-ferro-deps.sh` at `docker:init` time. Script reads project `Cargo.toml`, finds every `ferro*` path dep, rewrites to git dep using `--ferro-ref`.
- **D-09:** Dockerfile invokes the script in planner and builder stages after `COPY .` (local dev keeps path deps; Docker builds use git deps).
- **D-10:** New `--ferro-ref <branch|tag|sha>` flag on both `docker:init` and `do:init`, default `main`, persisted into generated script header.
- **D-11:** CLI pre-flight (`ferro deploy:check`, also blocks `docker:build`) verifies chosen ferro git ref is reachable on remote via `git ls-remote`. Fail loudly with push instructions.

### app.yaml.tpl
- **D-12:** `--region` flag on `do:init`, default `fra1`.
- **D-13:** Generate `envs:` block from `.env.example` with `SCOPE: RUN_TIME` per key; auto-classify as `type: SECRET` when key matches `*_KEY|*_SECRET|*PASSWORD|*TOKEN|DATABASE_URL`.
- **D-14:** Emit `databases:` block when `DATABASE_URL` is in `.env.example`, referencing `${db.DATABASE_URL}` in envs.
- **D-15:** Emit one `workers:` entry per non-server `[[bin]]` so background workers get their own DO App Platform component.

### docker_init.rs / do_init.rs
- **D-16:** Add `--force` flag to overwrite existing files.
- **D-17:** Walk up from CWD to locate `Cargo.toml` (don't require running from project root).
- **D-18:** Validate `--repo` argument format (`owner/repo`) before writing app.yaml.
- **D-19:** Lift duplicated `get_package_name()` into shared `project::package_name()` helper used by both commands.

### dockerignore.tpl
- **D-20:** Add entries: `database.db`, `*.sqlite*`, `.planning/`, `storage/`, `data/`.
- **D-21:** Drift audit vs `.gitignore` template noted but kept-in-sync work is deferred to Phase 124.

### Claude's Discretion
- Internal module layout for templates and helpers (planner decides).
- Test approach (unit tests for parsing helpers + golden file tests for rendered templates is the natural fit; planner finalizes).
- Exact CLI error messages and progress output style (must follow existing ferro-cli conventions).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/phases/122-deploy-scaffold-core-rewrite/SCOPE.md` — authoritative scope, all 21 decisions sourced from here.

### Reference apps (validation targets — read-only)
- `../../gestiscilo-it/app/Dockerfile` — current hand-patched Dockerfile (multi-bin, chromium, themes/, postgres, 4 ferro path deps).
- `../../gestiscilo-it/app/.do/app.yaml` — current hand-patched DO app spec.
- `../../gestiscilo-it/mkmenu/Dockerfile` — current hand-patched single-bin frontend-build Dockerfile.
- `../../gestiscilo-it/mkmenu/.do/app.yaml` — currently deployed `fra1` spec with managed postgres + Spaces.

### Existing ferro-cli code to rewrite
- `ferro-cli/src/commands/docker_init.rs` (or current path)
- `ferro-cli/src/commands/do_init.rs` (or current path)
- `ferro-cli/templates/Dockerfile.tpl`
- `ferro-cli/templates/dockerignore.tpl`
- `ferro-cli/templates/app.yaml.tpl`
- `ferro-cli/src/commands/` — locate `get_package_name()` duplications.

### Out of scope (cross-reference only)
- Phase 123 SCOPE — MCP deploy tools.
- Phase 124 SCOPE — `.gitignore`/`.dockerignore` drift sync.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- ferro-cli template rendering pipeline (already used by `docker:init`/`do:init`).
- `get_package_name()` exists in both commands — to be lifted into `project::package_name()`.
- `Cargo.toml` parsing already done somewhere in ferro-cli — planner should locate and reuse instead of pulling in a new TOML crate if possible.

### Established Patterns
- ferro-cli commands live under `ferro-cli/src/commands/`, templates under `ferro-cli/templates/`.
- Commands take `--force` flags elsewhere — follow that convention.
- Pre-flight checks pattern: see existing CLI validation flows.

### Integration Points
- `docker:init`, `do:init`, and new `deploy:check` registered in CLI command dispatcher.
- `--ferro-ref` and `--region` flags wired through clap-style argument parsing used by ferro-cli.

</code_context>

<specifics>
## Specific Ideas

- gestiscilo and mkmenu are the **acceptance test targets** — Verification section of SCOPE.md is the contract.
- The whole point: zero hand edits after regeneration. Any fallback to "user must edit X" is a bug.
- Workers must be first-class in app.yaml, not an afterthought (gestiscilo's `screenshot-worker` is the canonical example).

</specifics>

<deferred>
## Deferred Ideas

- New MCP deploy tools → Phase 123.
- `ferro doctor`, `routes --json`, CI workflow scaffold → Phase 124.
- `.gitignore` ↔ `.dockerignore` drift sync automation → Phase 124.
- `make:module`, json-ui runtime split → Phase 125.

</deferred>

---

*Phase: 122-deploy-scaffold-core-rewrite*
*Context gathered: 2026-04-07 (auto mode)*
