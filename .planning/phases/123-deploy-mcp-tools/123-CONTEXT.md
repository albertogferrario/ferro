# Phase 123: Deploy MCP Tools - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning
**Mode:** Auto (decisions sourced from SCOPE.md)

<domain>
## Phase Boundary

Add three read-only diagnostic MCP tools to ferro-mcp that expose deploy lifecycle helpers: `deploy_check`, `deploy_diff_env`, `runtime_requirements`. No file mutation, no deploy triggering. Reuses Phase 122 deploy primitives where present, but is independently shippable.

Source of truth: `.planning/phases/123-deploy-mcp-tools/SCOPE.md`.
</domain>

<decisions>
## Implementation Decisions

### deploy_check
- **D-01:** Pre-flight validation tool returning a structured report with severity per finding.
- **D-02:** Findings to detect:
  - Missing env vars (keys in `.env.example` but not in `.do/app.yaml` envs, or vice versa).
  - Path deps still present in `Cargo.toml` for `ferro*` crates (block deploy).
  - `DATABASE_URL` pointing at sqlite (block prod deploy).
  - Missing `Dockerfile` or `.do/app.yaml`.
  - Dirty git tree or unpushed commits on the deploy branch.
  - ferro git ref (from generated rewrite script) not reachable on remote.
- **D-03:** Output is structured (severity per finding) so agents can act on it programmatically.

### deploy_diff_env
- **D-04:** Compare local `.env` against `.do/app.yaml` envs block.
- **D-05:** Surface keys present in one but not the other; flag scope/type classification mismatches (e.g. should be SECRET).
- **D-06:** Output a 3-column table: key, local, remote.

### runtime_requirements
- **D-07:** Scan source for known runtime crate dependencies and emit needed apt packages.
- **D-08:** Initial registry mappings:
  - `chromiumoxide` / `headless_chrome` → `chromium`, `fonts-liberation`
  - `ffmpeg-next` → `ffmpeg`
  - `pdfium` → `libpdfium`
- **D-09:** Registry lives in `ferro-cli/src/deploy/runtime_deps.rs` and is extensible (single source of truth shared with Phase 122 docker:init runtime extras).
- **D-10:** Cross-check against the runtime extras block in the current `Dockerfile` and flag missing entries.

### Cross-cutting
- **D-11:** All three tools are **read-only** — never write files, never trigger deploys.
- **D-12:** Tools live in ferro-mcp crate, reusing Phase 122 deploy primitives (`ferro-cli/src/deploy/{env_example,classify,...}`) where overlap exists. If 122 helpers are not yet on disk, fall back to inline parsing — phase ships independently.
- **D-13:** Phase ships independently of Phase 122. No hard ordering between 122 and 123.

### Claude's Discretion
- MCP tool registration plumbing (follow existing ferro-mcp tool patterns).
- Exact JSON shape of the structured report (keep aligned with existing ferro-mcp tool conventions).
- How to detect "deploy branch" for unpushed commits check (planner decides — likely current HEAD branch).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/phases/123-deploy-mcp-tools/SCOPE.md` — authoritative scope.

### Cross-phase shared code (read-only)
- `ferro-cli/src/deploy/env_example.rs` — env parser (Phase 122).
- `ferro-cli/src/deploy/classify.rs` — SECRET classifier (Phase 122).
- `ferro-cli/src/deploy/ferro_deps.rs` — ferro path-dep detection helpers (Phase 122).
- `ferro-cli/src/project.rs` — project root walk-up + bin enumeration (Phase 122).
- `ferro-cli/src/commands/deploy_check.rs` — git ls-remote pre-flight (Phase 122) — should share logic with the MCP tool, not duplicate.

### ferro-mcp existing patterns
- `ferro-mcp/src/tools/` — existing tool implementations to mirror for registration, error handling, JSON shapes.

### Out of scope (cross-reference only)
- digitalocean-apps MCP — used externally to trigger actual deploys.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (Phase 122)
- `ferro-cli::deploy::env_example::parse_env_example` — parses `.env.example` → `Vec<(key, value)>`.
- `ferro-cli::deploy::classify::classify_secret` — `*_KEY|*_SECRET|*PASSWORD|*TOKEN|DATABASE_URL` heuristic.
- `ferro-cli::deploy::ferro_deps` — finds `ferro*` path deps in Cargo.toml.
- `ferro-cli::project::find_project_root` — walk-up Cargo.toml discovery.
- `ferro-cli::commands::deploy_check::*` — git ls-remote pre-flight (CLI side); MCP tool should call into the shared library function, not the binary.

### Established Patterns
- ferro-mcp tools live under `ferro-mcp/src/tools/` and register via the tool registry (planner: locate the registration entry point).
- ferro-cli now exposes a `lib.rs` (added in Phase 122 plan 08) — good cross-crate consumption point. Add public re-exports for the helpers ferro-mcp needs.

### Integration Points
- Cross-crate dep: `ferro-mcp` will need to depend on `ferro-cli` (or a shared `ferro-deploy` crate, if planner prefers extraction). Default: depend on `ferro-cli` directly to avoid premature crate split.

</code_context>

<specifics>
## Specific Ideas

- Same registry of crate→apt mappings drives both `runtime_requirements` (this phase) and the `--runtime-deps` flag on `docker:init` (Phase 122). Single source: `ferro-cli/src/deploy/runtime_deps.rs`.
- Verification target: gestiscilo (multi-bin, chromium, postgres) and mkmenu (deployed, single bin) — same fixtures Phase 122 uses.

</specifics>

<deferred>
## Deferred Ideas

- DO App Platform deploy triggering → use existing `digitalocean-apps` MCP, not in scope here.
- File mutation/auto-fix of detected issues → out of scope (read-only diagnostic only).
- `ferro doctor` umbrella command → Phase 124.

</deferred>

---

*Phase: 123-deploy-mcp-tools*
*Context gathered: 2026-04-07 (auto mode)*
