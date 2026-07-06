# Phase 127: Generated artifact polish — Context

**Gathered:** 2026-04-08
**Status:** Ready for planning
**Mode:** `--auto` (Claude selected recommended defaults; review and override if needed)

<domain>
## Phase Boundary

Make the artifacts emitted by `ferro docker:init` and `ferro do:init` runnable
end-to-end without manual editing. Absorbs REPORT items **5, 6, 7, 9, 10, 16, 18**
from `.planning/phases/126-deploy-experience-feedback/REPORT.md`.

In scope:
- Dockerfile `ENTRYPOINT` / `CMD` emission with bin auto-detection (item 18 — deploy blocker)
- DigitalOcean `web` service entrypoint behavior (item 18 corollary)
- `do:init` real `envs:` entries with secret/non-secret typing (item 16)
- Drop redundant per-bin `cargo build --release` invocations (item 6)
- Preserve dependency-table order when rewriting `Cargo.docker.toml` (item 5)
- "Next steps" footer printed by both `docker:init` and `do:init` (item 7)
- `--dry-run` flag on both `docker:init` and `do:init` (item 9)
- Stop generating cargo `readme = "README.md"` warnings on dockerignored READMEs (item 10)

Out of scope (belongs to Phase 128 or 129):
- Preflight checks (`ferro doctor` extensions, `cargo metadata` resolve, copy_dirs vs dockerignore validation, version skew detection) — items 3, 4, 13, 17 → **Phase 128**
- `ferro deploy:init` interactive metadata scaffolder — item 15 → **Phase 128**
- Publish workflow gating and per-crate version overrides — items 8, 14 → **Phase 129**

</domain>

<decisions>
## Implementation Decisions

### Dockerfile entrypoint (item 18)

- **D-01:** Generated `Dockerfile.tpl` MUST emit both `ENTRYPOINT` and `CMD` lines so the image is runnable with no extra arguments.
- **D-02:** Bin selection reuses the **same `web_bin` detection** that `do:init` already uses (`templates/do.rs`), so the Dockerfile ENTRYPOINT and the DO web service stay in sync by construction. Detection order:
  1. `[package.metadata.ferro.deploy].web_bin` if explicitly set
  2. The bin matching the `package.name` (single-bin or matching multi-bin)
  3. The first declared `[[bin]]`
  4. Fall back to package name if no `[[bin]]` is declared
- **D-03:** Emit `ENTRYPOINT ["/usr/local/bin/<bin>"]` and `CMD ["serve"]`. The `serve` subcommand is the canonical Ferro app default; users with a different default can override CMD by editing the generated Dockerfile (regenerated only on `--force`, per Phase 122.2 §2).
- **D-04:** Add a new template token (e.g. `{{ENTRYPOINT}}`) wired through `templates/docker.rs` so the renderer composes the ENTRYPOINT/CMD lines from the resolved bin name. Keep the template change minimal — token block at the bottom of the runtime stage.

### DO `web` service (item 18 corollary)

- **D-05:** `do:init` does NOT add a `run_command:` to the `web` service. The Dockerfile ENTRYPOINT becomes the single source of truth for "what runs". This avoids duplicating the bin name in two files and matches the existing worker model where `run_command` is only needed when overriding the default. Document this decision inline in the generated `.do/app.yaml` as a one-line comment.

### `do:init` env entries (item 16)

- **D-06:** Replace the comment-only `envs:` block with real entries derived from `.env.example`. Each key emits a YAML entry; the `value:` is left empty (`""`) so users still set the value, but the structure is `doctl apps update`-ready.
- **D-07:** Secret-shaped keys are emitted with `type: SECRET` and `scope: RUN_AND_BUILD_TIME`; non-secret keys get `scope: RUN_TIME` and no explicit type (defaults to `GENERAL`).
- **D-08:** Secret heuristic — case-insensitive substring match on the key name against `{secret, password, passwd, token, key, api_key, dsn, private, credential}`. Keys ending in `_URL` are non-secret unless they also match the heuristic (e.g. `DATABASE_URL` is non-secret; `STRIPE_SECRET_KEY` is secret).
- **D-09:** Source list order matches `.env.example` order (preserves human grouping). Skip blank lines and comment lines, but keep a blank-line separator in the output where the source had one.

### Build dedupe (item 6)

- **D-10:** Remove the per-bin `cargo build --release --bin <name>` lines from the generated Dockerfile. The plain `cargo build --release` already builds every `[[bin]]` declared in the workspace. Keep only the single build invocation. Update the corresponding template renderer in `templates/docker.rs` so `{{BIN_BUILDS}}` is emptied (or the token removed entirely).

### Dep table ordering (item 5)

- **D-11:** Switch `ferro-cli/src/deploy/rewrite_ferro_version.rs` from the `toml` crate to `toml_edit` so dependency-table order is preserved on re-serialization. The rewriter only mutates the `version` (and preserved metadata fields per ferro 0.2.1 fix) — it must NOT reorder sibling tables or sibling keys within a table.
- **D-12:** Existing regression tests (`preserves_package_rename_and_features`, etc.) MUST continue to pass; add a new regression test `preserves_dep_table_order` that asserts a multi-dep `Cargo.toml` round-trips with the original key order.

### "Next steps" footer (item 7)

- **D-13:** Both `docker:init` and `do:init` print a 3-5 line footer after success, to stdout. Tone: cargo-style, concise, no emoji, no banner art.
- **D-14:** `docker:init` footer suggests the next user action: `docker build -t <name>:test .` and `docker run --rm -p 8080:8080 --env-file .env.production <name>:test`.
- **D-15:** `do:init` footer suggests: review `.do/app.yaml`, populate envs (either via dashboard or `doctl apps update <id> --spec .do/app.yaml`), and the `doctl apps create --spec .do/app.yaml` first-deploy command.
- **D-16:** Footer is suppressed in `--dry-run` mode (since nothing was written).

### `--dry-run` flag (item 9)

- **D-17:** Both `docker:init` and `do:init` accept `--dry-run`. Behavior: render every file to its in-memory string, print a per-file header (`--- <relative/path> ---`) followed by the rendered content to stdout, and exit 0 without touching the filesystem.
- **D-18:** `--dry-run` short-circuits BEFORE any `Cargo.docker.toml` rewrite is persisted, but the rewrite is still computed in memory and printed.
- **D-19:** Exit code 0 on successful render; exit code non-zero only if rendering itself fails (e.g. missing `[package.metadata.ferro.deploy]` table). `--dry-run` does NOT promote rendering errors to soft warnings.

### `.dockerignore` README warning (item 10)

- **D-20:** Whitelist `README.md` in the generated `.dockerignore` (i.e. add `!README.md` after the `*.md` exclusion) so cargo's `readme = "README.md"` declaration resolves at build time without printing per-crate warnings. This is the smallest behavior change — keep the broad `*.md` exclusion in place for everything else.
- **D-21:** Document this in the generated `.dockerignore` with a one-line comment explaining why `README.md` is included.

### Claude's Discretion

- The exact wording of the "Next steps" footer (within the cargo-style, no-emoji constraint).
- Whether the new template token for ENTRYPOINT is named `{{ENTRYPOINT}}`, `{{ENTRYPOINT_BLOCK}}`, or split into two tokens — pick whatever keeps `templates/docker.rs` cleanest.
- Whether to extract the secret-shaped-key heuristic into a small helper module under `ferro-cli/src/deploy/` for reuse by Phase 128 preflight, or inline it in `templates/do.rs` for now. Recommended: extract — Phase 128 will want it.
- Test layout — extend existing integration tests in `templates/docker.rs` and `templates/do.rs`, or add a new `tests/` integration test file. Recommended: extend existing tests for consistency.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source report
- `.planning/phases/126-deploy-experience-feedback/REPORT.md` — primary source. Items 5, 6, 7, 9, 10, 16, 18 are in scope. Item 18 is the deploy blocker and should be sequenced first within the phase.
- `.planning/phases/126-deploy-experience-feedback/PROPOSAL.md` — triage output that produced this phase (if present).

### Adjacent phases
- `.planning/phases/122-deploy-scaffold-core-rewrite/122-CONTEXT.md` — original deploy scaffold contract (Dockerfile/DO templating model).
- `.planning/phases/122.2-deploy-simplification/122.2-CONTEXT.md` — current generation invariants (`§2` regenerate-only-on-`--force`, `§8` `.dockerignore` rules).
- `.planning/phases/123-deploy-mcp-tools/123-CONTEXT.md` — defines `deploy_check` MCP surface that Phase 128 will reuse; do NOT duplicate preflight logic here.
- `.planning/phases/124-doctor-introspection-and-ci-scaffold/124-CONTEXT.md` — `ferro doctor` surface that Phase 128 extends.

### Code surface to modify
- `ferro-cli/src/templates/files/docker/Dockerfile.tpl` — add ENTRYPOINT/CMD token slot (D-01..D-04).
- `ferro-cli/src/templates/files/docker/dockerignore.tpl` — add `!README.md` whitelist (D-20, D-21).
- `ferro-cli/src/templates/files/do/app.yaml.tpl` — replace `{{ENV_COMMENTS}}` with `{{ENVS_BLOCK}}` (D-06..D-09).
- `ferro-cli/src/templates/docker.rs` — bin detection, drop per-bin builds, ENTRYPOINT token wiring (D-02, D-10).
- `ferro-cli/src/templates/do.rs` — envs renderer, secret heuristic, no `run_command` on web (D-05..D-09).
- `ferro-cli/src/commands/docker_init.rs` — `--dry-run` flag, "Next steps" footer (D-13..D-19).
- `ferro-cli/src/commands/do_init.rs` — `--dry-run` flag, "Next steps" footer (D-13..D-19).
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` — switch to `toml_edit` (D-11, D-12).
- `ferro-cli/src/deploy/env_production.rs` — existing `.env.example` parser; reuse for envs:-block generation if shapes align.

### Project conventions
- `CLAUDE.md` (root) — pre-commit lint/test command; vision anchors.
- `~/.claude/CLAUDE.md` — Go/Rust standards, "delete old code" rule (no versioned function names).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets
- **Bin-detection logic** already exists in `templates/do.rs` for picking `web_bin`. Phase 127 should extract this into a small shared helper (e.g. `ferro-cli/src/deploy/bin_detect.rs`) so `templates/docker.rs` can call it without duplicating the heuristic.
- **`.env.example` parser** in `ferro-cli/src/deploy/env_production.rs` (77 LOC) already walks `.env.example` line-by-line. The do:init envs renderer should reuse it rather than re-parsing.
- **`rewrite_ferro_version.rs` regression test harness** is already set up — add the dep-order test alongside existing tests.

### Established patterns
- Templates use `{{TOKEN}}` substitution (no real templating engine). New tokens follow the same convention.
- Generated files carry a `Generated by ferro <command>` header; preserve this on any modified template.
- CLI commands return cargo-style output: concise, no banner, no emoji.
- `--force` is the regenerate gate (Phase 122.2 §2). `--dry-run` is independent of `--force` and never writes.

### Integration points
- `commands/docker_init.rs` and `commands/do_init.rs` are the single entry points; flag parsing lives there. Add `--dry-run` to both clap definitions.
- `templates/docker.rs::render_dockerfile` and `templates/do.rs::render_app_yaml` are the render boundaries — `--dry-run` should call these but skip the `fs::write`.

</code_context>

<specifics>
## Specific Ideas

- The `gestiscilo` smoke test in REPORT item 18 (image starts, exits 0 silently) is the canonical "before" repro. After Phase 127, the same `docker run --rm -p 8080:8080 --env-file .env.production <name>:test` invocation should reach the user's app code (and fail on whatever the user's app fails on — DB connect, missing env, etc.) rather than exit 0 with no logs.
- The generated `.do/app.yaml` after Phase 127 should be `doctl apps create --spec .do/app.yaml`-ready modulo the user filling in env values. No dashboard clicks should be required for the structural shape.

</specifics>

<deferred>
## Deferred Ideas

- **Preflight checks** (REPORT items 3, 4, 12, 13, 17) — Phase 128.
- **Interactive `ferro deploy:init` metadata scaffolder** (item 15) — Phase 128.
- **Publish workflow gating + per-crate version notes** (items 8, 14) — Phase 129.
- **gsd-tools phase-numbering bug** (item 11) — file against `gsd-tools` repo, not Ferro.
- **Per-crate `ferro_version` overrides** (item 14 long form) — defer until a real crate desync forces it; do not implement speculatively.

</deferred>

---

*Phase: 127-generated-artifact-polish*
*Context gathered: 2026-04-08 (--auto mode)*
