# Phase 124: Doctor, Introspection, CI Scaffold - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning
**Mode:** Auto (decisions sourced from SCOPE.md)

<domain>
## Phase Boundary

Consolidate scattered project-health checks into `ferro doctor`, make `ferro routes` agent-consumable via `--json`, ship a CI workflow scaffold from `do:init` (and a standalone `ferro ci:init`), and unify `.dockerignore`/`.gitignore` from a single source-of-truth template with a `ferro ignore:sync` reconciler.

Source of truth: `.planning/phases/124-doctor-introspection-and-ci-scaffold/SCOPE.md`.
</domain>

<decisions>
## Implementation Decisions

### ferro doctor
- **D-01:** Single command runs all checks in this order: toolchain → db connection → migrations → env completeness → path deps → workspace cargo-chef target dirs → generated artifacts (Dockerfile/.dockerignore/.do/app.yaml).
- **D-02:** Toolchain check: rustc/cargo version vs `rust-toolchain.toml`.
- **D-03:** DB check: open `DATABASE_URL`, run `SELECT 1`.
- **D-04:** Migrations check: pending vs applied.
- **D-05:** Env completeness: every key in `.env.example` is set in `.env`.
- **D-06:** Path deps: any `ferro*` path dep — warn for prod, ok for dev.
- **D-07:** Workspace check: cargo-chef recipe target dirs exist.
- **D-08:** Generated artifacts: warn (not error) when `Dockerfile` / `.dockerignore` / `.do/app.yaml` are missing.
- **D-09:** Output: human-readable default, `--json` for agent consumption. Exit code non-zero on any blocker (errors block, warnings don't).

### ferro routes --json
- **D-10:** Extend existing `generate_routes.rs` to emit JSON when `--json` is passed. Default output stays human pretty-printed.
- **D-11:** JSON schema (stable):
  ```json
  { "routes": [ { "method": "GET", "path": "/users/:id", "handler": "users::show", "name": "users.show", "middleware": ["auth"] } ] }
  ```
- **D-12:** Schema must be consumable by ferro-mcp (publish stable shape).

### CI workflow scaffold
- **D-13:** `do:init` drops `.github/workflows/ci.yml`. Also new `ferro ci:init` standalone command for projects not on DO.
- **D-14:** Triggers: `pull_request`, `push` to main.
- **D-15:** Jobs: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `ferro api:check`, `ferro validate:contracts`.
- **D-16:** Cached cargo registry + target dir (use standard rust cache action).
- **D-17:** Idempotent — skip if file exists unless `--force`.

### dockerignore / gitignore sync
- **D-18:** Single source-of-truth template: `templates/files/ignore_patterns.toml`, patterns grouped by category (rust, sqlite, planning, storage, secrets, ide).
- **D-19:** Both `.dockerignore` and `.gitignore` templates render from this source.
- **D-20:** New `ferro ignore:sync` command reconciles drift in existing projects.

### Cross-cutting
- **D-21:** Provider scope: GitHub Actions only. GitLab/others out of scope.
- **D-22:** `ferro doctor` is complementary to existing `ferro:info` MCP tool — does not replace it.

### Claude's Discretion
- Internal layout of doctor checks (one struct per check, registry pattern, planner decides).
- Exact `--json` schema field naming (must satisfy stability requirement; planner picks consistent shape with other ferro JSON outputs).
- Whether `ignore:sync` is interactive or fully automatic (recommend automatic with `--dry-run` flag).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/phases/124-doctor-introspection-and-ci-scaffold/SCOPE.md` — authoritative scope.

### Existing code to extend / consume
- `ferro-cli/src/commands/generate_routes.rs` — extend with `--json` flag (D-10/11/12).
- `ferro-cli/src/commands/do_init.rs` — extend to drop `ci.yml` (D-13).
- `ferro-cli/src/templates/files/docker/dockerignore.tpl` — to be regenerated from ignore_patterns.toml (D-18/19).
- `.gitignore` template under ferro-cli/src/templates/ — same.
- `ferro-cli/src/project.rs` — toolchain reader from Phase 122 (reuse for D-02).
- `ferro-cli/src/deploy/env_example.rs` — env parser from Phase 122 (reuse for D-05).
- `ferro-cli/src/deploy/ferro_deps.rs` — path-dep finder from Phase 122 (reuse for D-06).
- Migration check: locate existing migration runner in framework or ferro-cli.

### ferro-mcp integration target (D-12)
- `ferro-mcp/src/tools/` — for routes JSON schema consumption (no direct work in this phase, just ensure stable shape).

### Cross-phase
- Phase 122 deferred: `.dockerignore` D-21 noted "drift sync deferred to Phase 124" — that promise lives here.
- Phase 123 architecture note: ferro-mcp now spawns as standalone binary (`ferro mcp` → subprocess). Doctor command should NOT depend on ferro-mcp.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro_cli::project::resolve_rust_base_image` reads `rust-toolchain.toml` (D-02 reuse).
- `ferro_cli::deploy::env_example::parse_env_example` (D-05 reuse).
- `ferro_cli::deploy::ferro_deps::find_ferro_path_deps` (D-06 reuse — public from Phase 123).
- `ferro_cli::project::find_project_root` (walk-up Cargo.toml).
- ferro-cli has `lib.rs` (Phase 122 plan 08) — clean entry for shared consumption.

### Established Patterns
- ferro-cli command pattern: `commands/<name>.rs` + clap variant in `main.rs`.
- `--force` flag convention from Phase 122 (`docker:init`, `do:init`).
- Templates: `ferro-cli/src/templates/files/*` rendered via `templates/<name>.rs` context structs.

### Integration Points
- `do:init` (Phase 122) gets a new step: write `.github/workflows/ci.yml` unless exists.
- `ferro routes` command currently lives in `ferro-cli/src/commands/generate_routes.rs` (or similar — planner verifies path).
- Doctor command opens DB via existing framework DB layer — locate correct entry point.

</code_context>

<specifics>
## Specific Ideas

- Doctor JSON output is the contract for agent consumption — keep field names stable, document them.
- `ignore_patterns.toml` is the unifier — both Phase 122's dockerignore.tpl edits and any prior gitignore template collapse into this one file.
- CI workflow uses `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2` (standard idiomatic GHA Rust pattern).
- `cargo fmt --check && cargo clippy -- -D warnings && cargo test && ferro api:check && ferro validate:contracts` is the canonical Ferro project lint gate — encode it once in the CI template.

</specifics>

<deferred>
## Deferred Ideas

- GitLab CI / CircleCI / other providers → out of scope.
- Replacing `ferro:info` MCP tool → out of scope (complementary).
- Auto-fixing doctor findings → out of scope (diagnostic only).

</deferred>

---

*Phase: 124-doctor-introspection-and-ci-scaffold*
*Context gathered: 2026-04-07 (auto mode)*
