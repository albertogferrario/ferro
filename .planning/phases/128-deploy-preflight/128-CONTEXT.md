# Phase 128: Deploy preflight - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning
**Mode:** --auto (recommended defaults selected)

<domain>
## Phase Boundary

Extend `ferro doctor` with deploy-specific preflight checks that catch failures before a 1–10 minute Docker round-trip, and ship an interactive `ferro deploy:init` scaffolder for the `[package.metadata.ferro.deploy]` block. Absorbs REPORT items 3, 4, 13, 15, 17. One check registry, two surfaces: CLI (`ferro doctor`) and MCP (`deploy_check`).

**In scope:**
- New doctor checks: `copy_dirs` vs `.dockerignore` collision (item 3); local path-dep vs `Cargo.docker.toml` version skew (items 4, 13); `Cargo.docker.toml` staleness vs `Cargo.toml` (item 17 — note existing `cargo_docker_toml_staleness` may overlap; extend/rename as needed).
- `ferro deploy:init` interactive command that writes `[package.metadata.ferro.deploy]` with prompted + sensible defaults.
- MCP `deploy_check` tool exposes the same registry (no duplicate implementation).

**Out of scope:**
- Phase 129 publish-workflow gating (items 8, 14).
- Any new capability not in items 3/4/13/15/17.
</domain>

<decisions>
## Implementation Decisions

### Check registry & surface
- **D-01:** New checks register into the existing `default_checks()` list in `ferro-cli/src/doctor/registry.rs` — same `DoctorCheck` trait, same ordering convention. No parallel "preflight" registry.
- **D-02:** `ferro doctor` surfaces all checks (existing behavior). A preflight subset is identified via a per-check `category()` or tag (e.g., `CheckCategory::Deploy`) so `ferro doctor --deploy` and the MCP `deploy_check` tool can filter without duplicating registration.
- **D-03:** MCP `deploy_check` tool calls into the same registry with the deploy filter. One implementation, two surfaces (honors Phase 126 D-07).

### Checks to add
- **D-04:** `copy_dirs_dockerignore_collision` — parses `[package.metadata.ferro.deploy].copy_dirs` and `.dockerignore`, FAILs when any `copy_dirs` entry is excluded by `.dockerignore`. Message points at the offending ignore rule.
- **D-05:** `ferro_version_skew` — compares the resolved version of ferro crates used locally (path or crates.io) against what `Cargo.docker.toml` rewrites to. WARN if skew is benign (patch), FAIL if major/minor diverge. Covers items 4 and 13.
- **D-06:** `cargo_docker_toml_staleness` already exists — extend it (not duplicate) to detect the new drift modes from items 4/13/17. Reuse, don't add a parallel check.

### `ferro deploy:init` scaffolder
- **D-07:** New `ferro-cli/src/commands/deploy_init.rs` command, wired in `commands/mod.rs` and the CLI dispatcher. Mirrors existing scaffolder shape (`docker_init`, `ci_init`, `do_init`).
- **D-08:** Interactive prompts with sensible defaults: binary (auto-detected via `deploy::bin_detect`), worker binary (optional, default none), `copy_dirs` (default `["migrations", "static"]` only if present), runtime env var names (pulled from `.env.example` if present).
- **D-09:** Writes the `[package.metadata.ferro.deploy]` table to the project's root `Cargo.toml` in-place. If the table already exists, prompt: overwrite, merge, or abort. Default = abort.
- **D-10:** Ships a `--dry-run` flag (consistency with Phase 127 docker:init / do:init convention — see recent commits) that prints the diff without writing.
- **D-11:** Non-interactive mode: `ferro deploy:init --yes` accepts all defaults, errors if a required value cannot be inferred.

### Claude's Discretion
- Exact error/warning message wording for each check (follow existing check message style).
- Whether `cargo_docker_toml_staleness` should be renamed or simply extended — planner decides based on blast radius.
- Data structures for the filter mechanism (`CheckCategory` enum vs trait method vs tag set).
- Test fixture layout for the new checks.
- Whether deploy:init prompts use `dialoguer` or the existing prompting helper (check what other scaffolders use).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 128 scope
- `.planning/ROADMAP.md` §"Phase 128" — goal statement and absorbed REPORT items.
- `.planning/phases/126-deploy-experience-feedback/REPORT.md` — field notes, items 3, 4, 13, 15, 17 with reproduction context.
- `.planning/phases/126-deploy-experience-feedback/PROPOSAL.md` §"D-07 resolution" — where deploy_check lives (folds into `ferro doctor`).

### Existing surfaces to extend
- `ferro-cli/src/doctor/registry.rs` — check registration point.
- `ferro-cli/src/doctor/check.rs` — `DoctorCheck` trait and `CheckStatus` / `Report` types.
- `ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs` — existing check to extend (D-06).
- `ferro-cli/src/commands/doctor.rs` — CLI entry point.
- `ferro-cli/src/deploy/bin_detect.rs` — reusable binary auto-detection for `deploy:init` defaults.
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` — how `Cargo.docker.toml` rewrites ferro versions (context for D-05).
- `ferro-cli/src/commands/docker_init.rs`, `ferro-cli/src/commands/do_init.rs`, `ferro-cli/src/commands/ci_init.rs` — scaffolder shape to mirror for `deploy:init`.

### Dependency phases
- Phase 122.2 `.planning/phases/122.2-deploy-simplification/` — the decision to fold deploy checks into doctor.
- Phase 123 — existing MCP `deploy_check` tool (if present; planner must verify current state — no `deploy_check` hits found in `ferro-mcp/src/` during scout).
- Phase 124 — `ferro doctor` surface this phase extends.
- Phase 127 — recent `--dry-run` convention on docker:init / do:init to mirror.

### User-facing docs to update
- `docs/src/` — whichever page documents `[package.metadata.ferro.deploy]` and `ferro doctor`. Planner confirms exact file.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `DoctorCheck` trait + `Report` / `CheckStatus` types — all new checks plug in via existing trait.
- `default_checks()` registry — single extension point for D-01.
- `deploy::bin_detect` — for `deploy:init` binary default.
- `deploy::rewrite_ferro_version` — authoritative source of what ends up in `Cargo.docker.toml`; D-05 should call it (or a shared helper) rather than re-parsing.
- Existing `cargo_docker_toml_staleness` check — extend rather than duplicate (D-06).
- Scaffolder pattern in `docker_init.rs`, `do_init.rs`, `ci_init.rs` — template for `deploy_init.rs` including `--dry-run` (Phase 127 convention).

### Established Patterns
- Registry ordering is explicit and asserted in `default_checks_returns_nine_in_declared_order` test — new checks must update both the registry and the ordering assertion.
- Each check lives in its own file under `ferro-cli/src/doctor/checks/` and is re-exported via `checks/mod.rs`.
- Scaffolders live in `ferro-cli/src/commands/` and are dispatched from the CLI entry point.

### Integration Points
- `ferro-cli/src/commands/mod.rs` — wire `deploy_init` module.
- CLI clap/command tree — add `deploy:init` subcommand (match existing `docker:init` / `do:init` naming with colon).
- `ferro-mcp` — `deploy_check` tool (if it exists) must be updated to call filtered registry. If it does not yet exist, planner must confirm and scope accordingly. Codebase scout found no `deploy_check` references in `ferro-mcp/src/`.
</code_context>

<specifics>
## Specific Ideas

- Keep the filter mechanism minimal — a single `category()` method returning an enum is enough; no tag sets unless a check belongs to multiple categories (none do today).
- `ferro_version_skew` is the highest-value check in this phase — it catches the silent-mismatch class of bugs that the field report flagged twice (items 4 and 13). Plan it first.
- The `deploy:init` scaffolder is the one user-visible "killer" improvement — replacing hand-typed TOML with a guided prompt is disproportionately impactful per effort. Polish the prompt copy.
</specifics>

<deferred>
## Deferred Ideas

- Phase 129: publish-workflow gating for docs-only commits (items 8, 14). Already its own phase.
- A full `ferro deploy:doctor` alias that runs only the deploy-filtered subset — nice-to-have, planner may include if the filter mechanism makes it trivial.
- Auto-fix mode for `copy_dirs_dockerignore_collision` (edit `.dockerignore` for the user) — deferred; check should only diagnose in this phase.
</deferred>

---

*Phase: 128-deploy-preflight*
*Context gathered: 2026-04-09*
