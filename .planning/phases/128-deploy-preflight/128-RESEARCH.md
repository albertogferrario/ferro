# Phase 128: Deploy Preflight — Research

**Researched:** 2026-04-09
**Domain:** ferro-cli doctor extension, deploy-specific checks, interactive scaffolder
**Confidence:** HIGH — all findings sourced directly from the codebase

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** New checks register into the existing `default_checks()` list in `ferro-cli/src/doctor/registry.rs` — same `DoctorCheck` trait, same ordering convention. No parallel "preflight" registry.
- **D-02:** `ferro doctor` surfaces all checks. A preflight subset is identified via a per-check `category()` or tag (e.g., `CheckCategory::Deploy`) so `ferro doctor --deploy` and the MCP `deploy_check` tool can filter without duplicating registration.
- **D-03:** MCP `deploy_check` tool calls into the same registry with the deploy filter. One implementation, two surfaces (honors Phase 126 D-07).
- **D-04:** `copy_dirs_dockerignore_collision` — parses `[package.metadata.ferro.deploy].copy_dirs` and `.dockerignore`, FAILs when any `copy_dirs` entry is excluded by `.dockerignore`. Message points at the offending ignore rule.
- **D-05:** `ferro_version_skew` — compares the resolved version of ferro crates used locally (path or crates.io) against what `Cargo.docker.toml` rewrites to. WARN if skew is benign (patch), FAIL if major/minor diverge. Covers items 4 and 13.
- **D-06:** `cargo_docker_toml_staleness` already exists — extend it (not duplicate) to detect the new drift modes from items 4/13/17. Reuse, don't add a parallel check.
- **D-07:** New `ferro-cli/src/commands/deploy_init.rs` command, wired in `commands/mod.rs` and the CLI dispatcher. Mirrors existing scaffolder shape (`docker_init`, `ci_init`, `do_init`).
- **D-08:** Interactive prompts with sensible defaults: binary (auto-detected via `deploy::bin_detect`), worker binary (optional, default none), `copy_dirs` (default `["migrations", "static"]` only if present), runtime env var names (pulled from `.env.example` if present).
- **D-09:** Writes the `[package.metadata.ferro.deploy]` table to the project's root `Cargo.toml` in-place. If the table already exists, prompt: overwrite, merge, or abort. Default = abort.
- **D-10:** Ships a `--dry-run` flag (consistency with Phase 127 docker:init / do:init convention).
- **D-11:** Non-interactive mode: `ferro deploy:init --yes` accepts all defaults, errors if a required value cannot be inferred.

### Claude's Discretion

- Exact error/warning message wording for each check (follow existing check message style).
- Whether `cargo_docker_toml_staleness` should be renamed or simply extended — planner decides based on blast radius.
- Data structures for the filter mechanism (`CheckCategory` enum vs trait method vs tag set).
- Test fixture layout for the new checks.
- Whether deploy:init prompts use `dialoguer` or the existing prompting helper (check what other scaffolders use).

### Deferred Ideas (OUT OF SCOPE)

- Phase 129: publish-workflow gating for docs-only commits (items 8, 14).
- A full `ferro deploy:doctor` alias that runs only the deploy-filtered subset — nice-to-have, planner may include if the filter mechanism makes it trivial.
- Auto-fix mode for `copy_dirs_dockerignore_collision` (edit `.dockerignore` for the user).
</user_constraints>

---

## Summary

Phase 128 extends `ferro doctor` with three deploy-specific checks and adds the `ferro deploy:init` interactive scaffolder. The extension points are clear and self-consistent: a single `default_checks()` registry, a `DoctorCheck` trait, and a mature scaffolder pattern in `commands/`.

The highest-value check is `ferro_version_skew` — it catches the exact silent-mismatch class of bugs that surfaced twice in the gestiscilo session (items 4 and 13). The `deploy:init` scaffolder is the primary user-visible improvement: replacing hand-written TOML with a guided prompt.

No new crates are required. All building blocks exist: `dialoguer` for prompts, `toml_edit` for in-place TOML writes, `deploy::bin_detect` for binary auto-detection, `deploy::rewrite_ferro_version::read_path_dep_version` for version resolution, and the static `.dockerignore` template string for collision matching.

**Primary recommendation:** Implement in wave order — filter mechanism first (touches `check.rs` and `registry.rs`), then the two new check files, then the staleness extension, then `deploy_init.rs`, then MCP wiring. Keep the ordering assertion test updated as new checks are added.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `toml` | workspace (0.8.x) | Parse `Cargo.toml` / `Cargo.docker.toml` for staleness check | Already used in `cargo_docker_toml_staleness.rs` |
| `toml_edit` | 0.22 | In-place TOML write for `deploy:init` without destroying formatting | Already used in `rewrite_ferro_version.rs` |
| `dialoguer` | 0.11 | Interactive prompts (Input, Confirm, Select) | Already in ferro-cli; used by `new.rs`, `docker_compose.rs`, `make_api.rs` |
| `console` | 0.15 | Styled terminal output (`style("✓").green()`) | Already used by every scaffolder command |
| `anyhow` | 1 | Error propagation throughout CLI commands | Workspace dep, universal in ferro-cli |
| `tempfile` | 3.24.0 | Test fixtures for check unit tests | Already used by all check test modules |
| `serde_json` | workspace | JSON output for `--json` mode | Already used by `doctor.rs` |

### No New Crates Required
`.dockerignore` pattern matching does **not** require the `ignore` crate. The static `.dockerignore` written by `docker:init` is a known fixed-format file owned by ferro. The collision check only needs to match each `copy_dirs` entry against the lines of the `.dockerignore` file using simple glob-style logic. A line-by-line prefix/suffix match is sufficient for the patterns in `dockerignore.tpl` (e.g. `data/`, `storage/`). Full gitignore-spec matching would be over-engineering for this scope.

---

## Architecture Patterns

### Recommended Project Structure (additions)

```
ferro-cli/src/
├── doctor/
│   ├── check.rs                    # ADD: CheckCategory enum + category() to DoctorCheck trait
│   ├── registry.rs                 # MODIFY: default_checks() → add 2 new checks; update ordering test
│   └── checks/
│       ├── mod.rs                  # MODIFY: re-export 2 new checks
│       ├── cargo_docker_toml_staleness.rs   # MODIFY: extend drift modes
│       ├── copy_dirs_dockerignore_collision.rs   # NEW
│       └── ferro_version_skew.rs              # NEW
├── commands/
│   ├── mod.rs                      # MODIFY: add deploy_init module
│   ├── deploy_init.rs              # NEW
│   └── doctor.rs                   # MODIFY: add --deploy filter flag
├── main.rs                         # MODIFY: add DeployInit variant + doctor --deploy arg
└── ...
ferro-mcp/src/
├── service.rs                      # MODIFY: add deploy_check tool method + DeployCheckParams
└── tools/
    └── deploy_check.rs             # NEW MCP tool
```

### Pattern 1: DoctorCheck Trait Extension for Category Filter

The cleanest approach is adding a `category()` method to the `DoctorCheck` trait with a default returning `None` (or `CheckCategory::General`), so all existing checks continue to compile unchanged. New deploy checks return `CheckCategory::Deploy`.

```rust
// Source: ferro-cli/src/doctor/check.rs (existing trait, proposed extension)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckCategory {
    General,
    Deploy,
}

pub trait DoctorCheck {
    fn name(&self) -> &'static str;
    fn run(&self, root: &Path) -> CheckResult;
    /// Category used to filter checks. Defaults to General.
    fn category(&self) -> CheckCategory {
        CheckCategory::General
    }
}
```

The `default_checks()` registry returns all checks. The filter is applied at the call site:

```rust
// Source: pattern from commands/doctor.rs (existing), with proposed --deploy filter
let checks = default_checks();
let filtered: Vec<_> = if deploy_only {
    checks.iter().filter(|c| c.category() == CheckCategory::Deploy).collect()
} else {
    checks.iter().collect()
};
```

**MCP `deploy_check` tool** calls `default_checks()` and applies the same `Deploy` filter before running. No separate registry.

### Pattern 2: New Check File Shape

Each check lives in its own file under `checks/`. The canonical shape (from `cargo_docker_toml_staleness.rs`):

```rust
// Source: ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs

pub struct MyCheck;
const NAME: &str = "check_name_snake_case";

impl DoctorCheck for MyCheck {
    fn name(&self) -> &'static str { NAME }
    fn run(&self, root: &Path) -> CheckResult { check_impl(root) }
    fn category(&self) -> CheckCategory { CheckCategory::Deploy }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult { /* ... */ }
```

The `pub(crate) fn check_impl(root)` separation allows tests to call the logic directly without constructing the struct.

### Pattern 3: `deploy_init` Scaffolder Shape

All scaffolders follow the same three-layer shape (observed in `docker_init.rs` and `do_init.rs`):

1. `pub fn run(flags)` — thin wrapper, prints errors to stderr.
2. `pub fn run_with(flags)` — full entry point with all flag variants; calls `execute()`.
3. `pub fn execute(flags) -> anyhow::Result<()>` — library-level entry used by tests, returns `Result`.

`--dry-run` is handled by rendering to `RenderedFile` structs in memory, then calling `print_dry_run(&files)` (shared helper in `docker_init.rs`) instead of writing to disk.

For `deploy:init` the "render" is a TOML string computed in memory; `--dry-run` prints it. The actual write uses `toml_edit` to append/merge/overwrite the `[package.metadata.ferro.deploy]` table in-place.

### Pattern 4: In-Place TOML Write with toml_edit

`rewrite_ferro_version.rs` is the established `toml_edit` pattern:

```rust
// Source: ferro-cli/src/deploy/rewrite_ferro_version.rs
use toml_edit::{DocumentMut, Item, Value};

let content = fs::read_to_string(&cargo_path)?;
let mut doc: DocumentMut = content.parse()?;
// mutate specific keys ...
let result = doc.to_string();
fs::write(&cargo_path, result)?;
```

For `deploy:init`, the write inserts/replaces the `[package.metadata.ferro.deploy]` table. `toml_edit` preserves all other content and key order, which is required since `Cargo.toml` is user-authored.

### Pattern 5: `ferro_version_skew` Version Resolution

`rewrite_ferro_version.rs` already has a `read_path_dep_version(project_root, rel_path) -> Option<String>` private function that reads the version from a path-dep's `Cargo.toml`. The same logic exists in `cargo_docker_toml_staleness.rs`. For `ferro_version_skew`, the check should:

1. Read `Cargo.toml` — find all `ferro*` path deps.
2. For each path dep: call `read_path_dep_version` (can extract to a shared helper in `deploy/mod.rs` to avoid duplication) to get the local workspace version.
3. Read `Cargo.docker.toml` — find the `version =` string for the same key (the version that would be used in the Docker build).
4. Compare: if major or minor differ → `Error`; if patch differs → `Warn`; if equal → `Ok`.

The semver comparison should use string splitting on `.` rather than pulling in the `semver` crate — the version strings in `Cargo.docker.toml` are always resolved (no `^`, `~`, `*` wildcards) so exact comparison with major/minor parsing is sufficient.

### Pattern 6: `.dockerignore` Collision Matching

The static `.dockerignore` template uses directory patterns (`data/`, `storage/`, `target/`) and wildcard patterns (`*.log`, `*.sqlite*`). For the collision check:

1. Read `.dockerignore` (if absent: `Ok`, nothing to collide with).
2. Read `[package.metadata.ferro.deploy].copy_dirs` (if absent: use defaults from `FerroDeployMetadata::default()`).
3. For each `copy_dirs` entry, test if any non-comment, non-negation line in `.dockerignore` matches it.
4. Matching rules sufficient for this scope:
   - Exact match: `data` matches `data` or `data/`.
   - Prefix match: `data/` in `.dockerignore` excludes any entry whose first path component is `data`.
   - Negation lines (`!...`) do not exclude — skip them.
5. Report the specific offending `.dockerignore` line in `details`.

Do **not** implement full gitignore-spec matching (no `**`, no character classes). The ferro-generated `.dockerignore` only uses patterns from the above categories and the check is designed to catch the common user error of listing a globally-excluded directory in `copy_dirs`.

### Anti-Patterns to Avoid

- **Parallel check registry:** A second `deploy_checks()` function would immediately create divergence from the ordering test and the display loop in `doctor.rs`. Use `category()` filter on the single registry.
- **Duplicate `read_path_dep_version`:** Already exists privately in both `cargo_docker_toml_staleness.rs` and `rewrite_ferro_version.rs`. Extract to a `pub(crate)` helper in `deploy/mod.rs` so both the staleness check and the new version-skew check share one implementation.
- **Using `semver` crate for version comparison:** Unnecessary. The version strings in `Cargo.docker.toml` are concrete resolved versions (e.g. `"0.2.0"`). Split on `.` and compare the first two components.
- **Interactive prompts without TTY guard:** `new.rs` guards `dialoguer` with `std::io::IsTerminal::is_terminal(&std::io::stdin())`. Do the same in `deploy_init.rs` — skip prompts (use defaults or error) when stdin is not a TTY.
- **Forgetting to update the ordering assertion:** `default_checks_returns_nine_in_declared_order` in `registry.rs` asserts count and names. Every new check breaks this test if not updated. The count will become 11; names must include the two new deploy checks.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| In-place TOML mutation | Custom regex or string replacement | `toml_edit` (0.22, already in Cargo.toml) | Preserves key order, comments, whitespace; no parse-then-reserialize churn |
| Interactive prompts | Manual stdin reading | `dialoguer` (0.11, already in Cargo.toml) | TTY detection, colors, default values, `--yes` bypass |
| Terminal output colors | ANSI escape codes | `console` crate `style()` | Already used by every scaffolder; consistent look |
| Temporary directories in tests | Manual `fs::create_dir_all` cleanup | `tempfile::TempDir` | Already in all check tests; RAII cleanup |

---

## Key Code Facts

### `DoctorCheck` Trait (current state)

```rust
// Source: ferro-cli/src/doctor/check.rs
pub trait DoctorCheck {
    fn name(&self) -> &'static str;
    fn run(&self, root: &Path) -> CheckResult;
}
```

No `category()` method yet. Adding it with a default impl is backward-compatible — all nine existing checks continue to compile with zero changes.

### Registry Ordering (MUST update test)

```rust
// Source: ferro-cli/src/doctor/registry.rs — test asserts exact 9-name order
assert_eq!(names, vec![
    "toolchain_match", "db_connection", "migrations_pending",
    "local_env_parity", "deploy_env_parity", "cargo_docker_toml_staleness",
    "generated_artifacts", "database_url_sqlite_in_prod", "git_clean_and_pushed",
]);
```

After Phase 128 the count becomes 11 and the test must include `"copy_dirs_dockerignore_collision"` and `"ferro_version_skew"` in the declared positions. The insertion point is after `"cargo_docker_toml_staleness"` (to cluster deploy checks together) or at the end before `"git_clean_and_pushed"`.

### `FerroDeployMetadata` Schema (current state)

```rust
// Source: ferro-cli/src/project.rs
pub struct FerroDeployMetadata {
    pub runtime_apt: Vec<String>,
    pub copy_dirs: Vec<String>,
    pub ferro_version: Option<String>,
    pub web_bin: Option<String>,
}
```

`deploy:init` writes this table. `read_deploy_metadata()` reads it. No schema change needed — `deploy:init` uses these four fields as the prompt surface.

### `deploy_check` MCP Tool: Does Not Exist Yet

Confirmed: `grep -r "deploy_check" ferro-mcp/src/` returns zero hits. The tool must be created from scratch in `ferro-mcp/src/tools/deploy_check.rs` and registered in `service.rs` using the `#[tool(...)]` macro pattern.

MCP tool shape:

```rust
// Pattern: ferro-mcp/src/service.rs existing tools
#[tool(
    name = "deploy_check",
    description = "Run deploy-specific preflight checks..."
)]
pub async fn deploy_check(&self) -> String {
    // calls default_checks(), filters by CheckCategory::Deploy, runs each
    // returns JSON Report
}
```

The tool takes no parameters (runs all deploy checks) or optionally a `root` path override. For consistency with other zero-param tools (`application_info`, `list_events`), no params struct is needed unless a path override is wanted.

### `dialoguer` Usage in ferro-cli

Two interaction types are already used:
- `Input::with_theme(&ColorfulTheme::default())` — text input with default value (in `new.rs`)
- `dialoguer::Confirm::new()` — yes/no prompt (in `make_api.rs`, `docker_compose.rs`)

For `deploy:init`, a third type is needed for the "table exists: overwrite/merge/abort" choice: `dialoguer::Select`. This is part of `dialoguer 0.11` and requires no new crate version.

### `--dry-run` Convention (from Phase 127)

Phase 127 (`docker:init`, `do:init`) established:
- CLI flag: `--dry-run` (not `--dryrun` or `--preview`).
- Behavior: render everything to memory → print with `--- <path> ---` headers → return `Ok(())` without writing any files.
- Render errors remain hard errors in `--dry-run` mode.
- "Next steps" footer is suppressed in `--dry-run`.
- The split `compute_*` + `persist_*` functions enable this cleanly.

`deploy:init` should mirror this: compute the TOML string to insert, print it with a header like `--- Cargo.toml ([package.metadata.ferro.deploy]) ---`, return without writing.

---

## Common Pitfalls

### Pitfall 1: Ordering Assertion Test Breaks on New Check

**What goes wrong:** Adding a check to `default_checks()` without updating the `default_checks_returns_nine_in_declared_order` test causes a compile-passing test failure.
**Why it happens:** The test asserts both count (9) and exact name order.
**How to avoid:** Update the test in the same commit that adds the check to `default_checks()`. New count = 11.
**Warning signs:** `cargo test` in `ferro-cli` fails with `assertion failed: checks.len() == 9`.

### Pitfall 2: `toml_edit` vs `toml` Crate Confusion

**What goes wrong:** Using the `toml` (value-level) crate to write `deploy:init` output scrambles key order and removes comments from `Cargo.toml`.
**Why it happens:** The `toml` crate re-serializes with alphabetical key ordering.
**How to avoid:** Use `toml_edit` for any write path. Use `toml` only for read-only parsing where ordering doesn't matter (already the pattern in `cargo_docker_toml_staleness.rs`).
**Warning signs:** `Cargo.toml` diff shows unrelated keys reordered or comments removed.

### Pitfall 3: `.dockerignore` May Not Exist

**What goes wrong:** `copy_dirs_dockerignore_collision` panics or returns an error when `.dockerignore` is absent.
**Why it happens:** Calling `fs::read_to_string(".dockerignore")?` when the file is missing.
**How to avoid:** If `.dockerignore` is absent, return `Ok` with a message like "skipped (.dockerignore absent)" — same pattern as `CargoDockerTomlStalenessCheck` skips when `Cargo.docker.toml` is absent.
**Warning signs:** Test on a project without `.dockerignore` returns an error check result.

### Pitfall 4: `deploy:init` Writing to Wrong `Cargo.toml`

**What goes wrong:** When run inside a workspace, `find_project_root()` returns the workspace root `Cargo.toml`, not the member crate's `Cargo.toml`. The deploy metadata belongs in the member crate.
**Why it happens:** `find_project_root()` walks up to the first `Cargo.toml` it finds, which is the workspace root.
**How to avoid:** This is the same behavior as `docker:init` and `do:init` — they call `read_deploy_metadata()` which also uses `find_project_root()`. The user is expected to run `ferro deploy:init` from inside the crate directory. Document this in the "Next steps" footer and/or the command's help text.

### Pitfall 5: `ferro_version_skew` Check When `Cargo.docker.toml` Is Absent

**What goes wrong:** `ferro_version_skew` errors on a project that has not run `docker:init` yet.
**Why it happens:** The check reads `Cargo.docker.toml` to find the rewritten version.
**How to avoid:** If `Cargo.docker.toml` is absent, return `Ok` with `"skipped (Cargo.docker.toml absent)"` — consistent with `cargo_docker_toml_staleness` behavior.

### Pitfall 6: `dialoguer` Panic When Not Running in a TTY

**What goes wrong:** `deploy:init` panics when called in a non-interactive context (CI, pipe).
**Why it happens:** `dialoguer` panics if called without a TTY.
**How to avoid:** Guard prompts with `std::io::IsTerminal::is_terminal(&std::io::stdin())`. If not a TTY and `--yes` is not passed, print an error and exit. The `new.rs` command has the canonical guard.

---

## MCP Tool Registration Pattern

The `service.rs` uses the `#[tool_router(router = tool_router)]` + `#[tool(name = "...")]` proc-macro pattern from the `rmcp` crate. Every tool is a method on `FerroMcpService`. Tools with no parameters use no `Parameters<T>` argument:

```rust
// Example: ferro-mcp/src/service.rs
pub async fn application_info(&self) -> String {
    match tools::application_info::execute(&self.project_root) { ... }
}
```

For `deploy_check`, the tool needs `ferry-cli` as a workspace dependency of `ferro-mcp` (to call `default_checks()` and the category filter). Confirm this dependency exists — `ferro-mcp` likely already depends on `ferro-cli` since it calls deploy-related introspection functions. If not, add it to `ferro-mcp/Cargo.toml`.

---

## Environment Availability

Step 2.6: SKIPPED — phase is pure code changes within the `ferro-cli` and `ferro-mcp` crates. No external tools, services, or databases are required beyond the standard Rust toolchain already in the project.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tempfile` fixtures |
| Config file | None (standard Cargo test runner) |
| Quick run command | `cargo test -p ferro-cli -- doctor` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| D-01/D-02 | `default_checks()` includes new checks; `category()` filter works | unit | `cargo test -p ferro-cli -- registry` | Update existing `default_checks_returns_nine_in_declared_order` |
| D-04 | `copy_dirs_dockerignore_collision` detects collision | unit | `cargo test -p ferro-cli -- copy_dirs_dockerignore` | Fixture: temp dir with `.dockerignore` excluding `data/`, `copy_dirs = ["data"]` |
| D-04 | No false positive when `copy_dirs` not excluded | unit | same | Fixture: `copy_dirs = ["migrations"]`, not in `.dockerignore` |
| D-04 | Returns Ok/skip when `.dockerignore` absent | unit | same | Fixture: temp dir without `.dockerignore` |
| D-05 | `ferro_version_skew` detects major/minor drift → Error | unit | `cargo test -p ferro-cli -- ferro_version_skew` | Fixture: path dep at 0.2.0, docker toml at 0.1.0 |
| D-05 | `ferro_version_skew` detects patch drift → Warn | unit | same | Fixture: 0.2.0 vs 0.2.1 |
| D-05 | `ferro_version_skew` no drift → Ok | unit | same | Fixture: same version both sides |
| D-05 | Skipped when `Cargo.docker.toml` absent | unit | same | Fixture: no `Cargo.docker.toml` |
| D-06 | Extended staleness check catches new drift modes | unit | `cargo test -p ferro-cli -- cargo_docker_toml_staleness` | Extend existing test suite |
| D-07 | `deploy:init` writes `[package.metadata.ferro.deploy]` | unit | `cargo test -p ferro-cli -- deploy_init` | Use `execute()` entry point |
| D-09 | `deploy:init` aborts (default) when table exists | unit | same | Fixture: `Cargo.toml` with existing deploy table |
| D-10 | `deploy:init --dry-run` writes zero files | unit | same | Assert `Cargo.toml` unchanged after execute with dry_run=true |
| D-11 | `deploy:init --yes` uses defaults non-interactively | unit | same | Call `execute(yes=true)` from test |
| D-03 | MCP `deploy_check` returns filtered results | unit | `cargo test -p ferro-mcp` | Call tool handler directly |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-cli -- doctor` (doctor module only, fast)
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

None — existing test infrastructure (tempfile, built-in #[test], no external test runner config) covers all phase requirements. The new tests follow the existing pattern in `cargo_docker_toml_staleness.rs` exactly.

---

## Sources

### Primary (HIGH confidence)

All findings sourced directly from the codebase (no external lookups needed):

- `ferro-cli/src/doctor/check.rs` — `DoctorCheck` trait, `CheckResult`, `Report`, `CheckStatus`
- `ferro-cli/src/doctor/registry.rs` — `default_checks()`, ordering assertion test
- `ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs` — existing check to extend; `check_impl` separation pattern; test fixture pattern
- `ferro-cli/src/doctor/checks/mod.rs` — re-export pattern for new checks
- `ferro-cli/src/commands/doctor.rs` — CLI entry point for `ferro doctor`
- `ferro-cli/src/commands/docker_init.rs` — `RenderedFile`, `print_dry_run`, `--dry-run` convention
- `ferro-cli/src/commands/do_init.rs` — full `--dry-run` + `run_with` + `execute` three-layer scaffolder shape
- `ferro-cli/src/commands/ci_init.rs` — simpler scaffolder (no --dry-run) for contrast
- `ferro-cli/src/commands/new.rs` — `dialoguer` TTY guard pattern
- `ferro-cli/src/deploy/bin_detect.rs` — `detect_web_bin()` for `deploy:init` default
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` — `read_path_dep_version`, `toml_edit` write pattern
- `ferro-cli/src/deploy/mod.rs` — `find_ferro_path_deps`, `FerroDeployMetadata`
- `ferro-cli/src/project.rs` — `FerroDeployMetadata` schema, `read_deploy_metadata()`
- `ferro-cli/src/main.rs` — clap command tree, `--dry-run` flag spelling convention
- `ferro-mcp/src/service.rs` — `#[tool(...)]` registration pattern, `FerroMcpService`
- `ferro-mcp/src/tools/mod.rs` — confirms `deploy_check` does NOT exist (zero hits)
- `ferro-cli/src/templates/files/docker/dockerignore.tpl` — exact content of generated `.dockerignore`
- `ferro-cli/Cargo.toml` — confirmed: `dialoguer = "0.11"`, `toml_edit = "0.22"`, `console = "0.15"`, `anyhow = "1"`, `tempfile = "3.24.0"`

---

## Metadata

**Confidence breakdown:**
- DoctorCheck trait extension: HIGH — read the actual trait, confirmed no `category()` method exists
- Check implementations: HIGH — all helper functions confirmed present and callable
- MCP tool: HIGH — confirmed `deploy_check` does not exist in `ferro-mcp/src/tools/`; registration pattern confirmed from `service.rs`
- `deploy:init` scaffolder: HIGH — `dialoguer`, `toml_edit`, `find_project_root`, `FerroDeployMetadata` all confirmed present
- `.dockerignore` matching: HIGH — full `.dockerignore` content read; simple line-based matching confirmed sufficient

**Research date:** 2026-04-09
**Valid until:** This research reflects the codebase as of 2026-04-09. Stable until next doctor/deploy changes.
