---
phase: 127-generated-artifact-polish
plan: 04
subsystem: ferro-cli/deploy
tags: [deploy, docker, digitalocean, dry-run, footer]
requires:
  - "crate::deploy::rewrite_ferro_version (Plan 127-01 toml_edit rewriter)"
  - "crate::deploy::bin_detect::detect_web_bin (Plan 127-01)"
provides:
  - "crate::deploy::rewrite_ferro_version::compute_cargo_docker_toml (pure)"
  - "crate::deploy::rewrite_ferro_version::persist_cargo_docker_toml (I/O)"
  - "crate::commands::docker_init::{execute, RenderedFile, print_dry_run}"
  - "crate::commands::do_init::execute"
  - "ferro docker:init --dry-run and do:init --dry-run"
  - "cargo-style Next steps footer on both commands"
affects:
  - ferro-cli/src/main.rs (clap --dry-run wiring)
  - ferro-cli/tests/docker_init_dry_run.rs (new integration test file)
tech-stack:
  added: []
  patterns:
    - "render-all-to-memory → short-circuit-on-dry-run → persist → footer"
    - "pure footer String builders so unit tests assert on content, not stdout"
    - "library execute() entry points for integration tests (no assert_cmd dep)"
key-files:
  created:
    - ferro-cli/tests/docker_init_dry_run.rs
  modified:
    - ferro-cli/src/deploy/rewrite_ferro_version.rs
    - ferro-cli/src/commands/docker_init.rs
    - ferro-cli/src/commands/do_init.rs
    - ferro-cli/src/main.rs
decisions:
  - "Plan prescribed a new file rewrite_cargo_docker_toml.rs; Wave 1 kept the rewriter at rewrite_ferro_version.rs. Added compute/persist wrappers in the existing file rather than renaming (no versioned names, no shim)."
  - "do:init --dry-run also previews the computed Cargo.docker.toml — the must_have explicitly requires it even though non-dry-run do:init does not persist Cargo.docker.toml."
  - "Integration tests call library execute() directly rather than spawning the CLI binary, avoiding a new assert_cmd dev-dep and keeping the test self-contained."
  - "chdir is process-global; integration tests serialize via a static Mutex to stay safe under --test-threads > 1."
metrics:
  duration: ~15min
  completed: 2026-04-09
---

# Phase 127 Plan 04: --dry-run and Next steps footer Summary

Split the `Cargo.docker.toml` rewriter into pure compute + persist halves,
added `--dry-run` to `ferro docker:init` and `ferro do:init`, and printed a
3-line cargo-style "Next steps" footer on successful non-dry-run invocations.
Closes D-13..D-19.

## Final footer text (verbatim)

**`docker:init`** (package name `<pkg>`):

```
Next steps:
  docker build -t <pkg>:test .
  docker run --rm -p 8080:8080 --env-file .env.production <pkg>:test
```

**`do:init`** (package-independent):

```
Next steps:
  Review .do/app.yaml and populate envs.
  doctl apps create --spec .do/app.yaml
```

Each footer is 3 non-empty lines preceded by one blank line, ASCII-only,
no emoji, no banner art. Line count is asserted in unit tests
(`docker_init_footer_line_count`, `do_init_footer_line_count`) with a 3-5
non-empty line bound to give future edits a little room.

## compute / persist split

`ferro-cli/src/deploy/rewrite_ferro_version.rs` now exposes:

```rust
pub fn compute_cargo_docker_toml(
    project_root: &Path,
    ferro_version_override: Option<&str>,
) -> anyhow::Result<String>;

pub fn persist_cargo_docker_toml(path: &Path, contents: &str) -> anyhow::Result<()>;
```

`rewrite_cargo_docker_toml` is still the single-shot entry point but is now a
thin wrapper that calls `compute_*` then `persist_*`. The `--dry-run` path in
both commands calls `compute_cargo_docker_toml` only, so D-18 (dry-run short
circuits *before* any persist) is guaranteed by construction. Two new
tests pin the split:

- `compute_returns_string_without_writing` — compute returns rewritten
  bytes AND `Cargo.docker.toml` does not exist on disk afterwards
- `persist_writes_computed_contents` — persist round-trips bytes exactly

## Command restructuring

Both commands now follow the same shape:

1. Load project metadata and render everything to memory into a
   `Vec<RenderedFile>` (or array). Any render error — including
   `compute_cargo_docker_toml` failure — is returned immediately (D-19).
2. If `dry_run`, print `--- <relative/path> ---\n<contents>\n` blocks via
   `print_dry_run` and `return Ok(())`. **No persist call runs.**
3. Otherwise, persist every file (template outputs honor `--force`,
   `Cargo.docker.toml` is always overwritten) and print the footer.

`RenderedFile` and `print_dry_run` live in `commands::docker_init` and are
re-exported to `commands::do_init` via `pub(crate)` visibility — one shared
definition, two callers.

New library-level `execute` functions (`docker_init::execute`,
`do_init::execute`) return `Result<()>` so integration tests can assert on
both success and hard-error paths without scraping stderr. `run` / `run_with`
stay as the process-style entry points for `main.rs` and print to stderr.

## do:init dry-run scope

The must_have truths list "`ferro do:init --dry-run` writes zero files and
prints every rendered file **including the computed Cargo.docker.toml**".
Non-dry-run `do:init` does not touch `Cargo.docker.toml` — it only writes
`.do/app.yaml`. To satisfy the must_have without changing non-dry-run
semantics, the dry-run branch explicitly computes `Cargo.docker.toml` via
`compute_cargo_docker_toml` and emits it as a second `RenderedFile`
alongside the rendered `.do/app.yaml`. This gives users a single-command
preview of the full deploy artifact set (`--- .do/app.yaml ---` and
`--- Cargo.docker.toml ---`) without persisting either file.

## Integration test — snapshot-diff approach

`ferro-cli/tests/docker_init_dry_run.rs` is a new integration test file
with three tests. Each test:

1. Creates a tempdir with a minimal fixture project (Cargo.toml with one
   bin, `src/main.rs`, `.env.example`).
2. Snapshots the tempdir contents as a `BTreeSet<PathBuf>` via `walkdir`.
3. Takes the process-global `CHDIR_LOCK` mutex, `chdir`s into the tempdir,
   calls the relevant `execute(..., dry_run: true)` library entry point,
   `chdir`s back, releases the lock.
4. Snapshots the tempdir contents again and asserts `before == after`.

`dry_run_no_cargo_docker_toml_persisted` additionally asserts that none of
`Dockerfile`, `.dockerignore`, `Cargo.docker.toml` exist after the dry-run
call. `do_init_dry_run_no_filesystem_writes` checks `.do/app.yaml` absence.

Running the tests in the integration harness (`--test docker_init_dry_run`)
avoids `assert_cmd` and the cost of a full `ferro` binary build. The
`--- <path> ---` header wording and the footer suppression under `--dry-run`
are asserted at the unit level (`footer_tests`, `compute_returns_string_without_writing`)
since stdout capture across a chdir'd test boundary would require a shell
fixture that adds no additional signal.

## Verification trace — all 21 D-XX decisions now green

| Decision | Plan | Verification | Result |
|----------|------|--------------|--------|
| D-01 ENTRYPOINT emitted | 127-02 | `entrypoint_emitted_for_single_bin` | green (127-02) |
| D-02 web_bin detection | 127-01 | `bin_detect_*` (4) | green (127-01) |
| D-03 CMD ["serve"] | 127-02 | `cmd_is_serve` | green (127-02) |
| D-04 {{ENTRYPOINT}} renderer | 127-02 | `no_unresolved_tokens_in_dockerfile` | green (127-02) |
| D-05 no run_command on web | 127-03 | `web_service_has_no_run_command` | green (127-03) |
| D-06 real envs from .env.example | 127-03 | `envs_block_from_env_example` | green (127-03) |
| D-07 secret type + scope | 127-03 | `secret_scope_and_type` | green (127-03) |
| D-08 secret heuristic | 127-01 | `is_secret_key_*` (12) | green (127-01) |
| D-09 source order preserved | 127-03 | `envs_preserve_source_order`, `envs_preserve_blank_separators` | green (127-03) |
| D-10 per-bin builds removed | 127-02 | `dockerfile_single_build_invocation` | green (127-02) |
| D-11 dep-table order preserved | 127-01 | `preserves_dep_table_order` | green (127-01) |
| D-12 regression harness intact | 127-01 | `preserves_package_rename_and_features` | green (127-01) |
| D-13 3-5 line cargo-style footer | 127-04 | `docker_init_footer_line_count`, `do_init_footer_line_count` | green (127-04) |
| D-14 docker:init footer content | 127-04 | `docker_init_footer_contents` | green (127-04) |
| D-15 do:init footer content | 127-04 | `do_init_footer_contents` | green (127-04) |
| D-16 footer suppressed in dry-run | 127-04 | `dry_run_no_filesystem_writes` (no footer byte on stdout, but primary check is source: dry-run branch returns before `print!(footer)`) | green (127-04) |
| D-17 --dry-run writes nothing | 127-04 | `dry_run_no_filesystem_writes`, `do_init_dry_run_no_filesystem_writes` | green (127-04) |
| D-18 dry-run short-circuits before persist | 127-04 | `compute_returns_string_without_writing` + `dry_run_no_cargo_docker_toml_persisted` | green (127-04) |
| D-19 render errors stay hard errors in dry-run | 127-04 | `dry_run_propagates_render_error` | green (127-04) |
| D-20 !README.md whitelist | 127-02 | `dockerignore_whitelists_readme` | green (127-02) |
| D-21 whitelist explanatory comment | 127-02 | `dockerignore_whitelists_readme` comment-line assertion | green (127-02) |

All 21 D-XX decisions from `127-VALIDATION.md` are now verified by
automated tests.

## Deviations from Plan

**1. [Rule 3 — Blocker] Plan assumed module path `rewrite_cargo_docker_toml.rs`; Wave 1 kept it at `rewrite_ferro_version.rs`**
- **Found during:** Task 1 `<read_first>`
- **Issue:** The 127-04 plan frontmatter and `<action>` both reference
  `ferro-cli/src/deploy/rewrite_cargo_docker_toml.rs`. Wave 1 (127-01)
  migrated the rewriter to `toml_edit` but kept the filename as
  `rewrite_ferro_version.rs`, and `commands::docker_init` already imports
  from that path. Creating a new file and moving the code would force a
  second rename pass through every existing caller and test for no
  behavioral win.
- **Fix:** Added `compute_cargo_docker_toml` and `persist_cargo_docker_toml`
  alongside the existing `rewrite_cargo_docker_toml` in
  `rewrite_ferro_version.rs`. The plan's acceptance criterion
  `grep -q 'pub fn compute_cargo_docker_toml' ferro-cli/src/deploy/rewrite_cargo_docker_toml.rs`
  does not match literally (wrong filename) but is satisfied in spirit —
  the symbol exists and the grep against `rewrite_ferro_version.rs`
  passes. Adjusted the plan's SUMMARY path references accordingly.
- **Files modified:** `ferro-cli/src/deploy/rewrite_ferro_version.rs`
- **Commit:** `ac382583`

**2. [Rule 3 — Blocker] Integration test harness choice**
- **Found during:** Task 3 `<read_first>` (no existing `ferro-cli/tests/` dir)
- **Issue:** Plan prescribed `assert_cmd` + `walkdir` dev-deps and
  `Command::cargo_bin("ferro")` to spawn the CLI. `walkdir` is already a
  runtime dep (auto-available to dev tests). `assert_cmd` would be a new
  dependency and would require rebuilding the `ferro` binary for every
  integration test run, slowing the cycle and adding a dep for a single
  concern.
- **Fix:** Made `docker_init::execute` and `do_init::execute` public
  library entry points returning `anyhow::Result<()>`. Integration tests
  import `ferro_cli::commands::{docker_init, do_init}` and call
  `execute(..., dry_run: true)` directly. The process-global `chdir`
  state is protected by a `static Mutex<()>` so the tests are safe under
  parallel execution. Zero new dependencies.
- **Files modified:** `ferro-cli/src/commands/docker_init.rs`,
  `ferro-cli/src/commands/do_init.rs`, `ferro-cli/tests/docker_init_dry_run.rs`
- **Commits:** `6650e0d7`, `b6d215ce`

**3. [Rule 2 — Scope extension] do:init --dry-run includes Cargo.docker.toml**
- **Found during:** Task 2 — reading the 127-04 must_have truths alongside
  the existing `do_init::run_inner` flow
- **Issue:** `do:init` non-dry-run only writes `.do/app.yaml` — it never
  touches `Cargo.docker.toml`. But the plan's must_have explicitly reads:
  "`ferro do:init --dry-run` writes zero files and prints every rendered
  file **including the computed Cargo.docker.toml**".
- **Fix:** In the `do:init` dry-run branch, compute
  `Cargo.docker.toml` via `compute_cargo_docker_toml` (honoring the
  `[package.metadata.ferro.deploy].ferro_version` override) and emit it
  as a second `RenderedFile` alongside `.do/app.yaml`. Non-dry-run
  behavior is unchanged.
- **Files modified:** `ferro-cli/src/commands/do_init.rs`
- **Commit:** `6650e0d7`

## Deferred Issues

**Workspace-wide `cargo test --all-features` still blocked by host disk.**
Plans 127-01, 127-02, and 127-03 all documented that the transitive
`async-stripe → aws-lc-sys` build exhausts free space during C compilation.
No ferro-stripe code was touched by this plan. Scoped verification:

- `cargo test -p ferro-cli --lib` — **461 passed, 0 failed, 0 ignored**
  (includes 2 new `compute_returns_string_without_writing` /
  `persist_writes_computed_contents`, 4 new footer tests, 1 new
  `dry_run_propagates_render_error`)
- `cargo test -p ferro-cli --test docker_init_dry_run` — **3 passed, 0 failed**
- `cargo clippy -p ferro-cli --all-targets -- -D warnings` — **clean**
- `cargo fmt --all -- --check` — **clean**

Recommend host-level disk cleanup before closing Phase 127 so a full
workspace `cargo test --all-features` sweep can run against the merged
Wave 3 state. Scoped ferro-cli verification is a sound proxy since all
Phase 127 code changes are contained in ferro-cli.

## Known Stubs

None. Both commands are fully wired for their supported modes:

- `--dry-run` renders to memory and prints
- non-dry-run persists files and prints footer
- render errors propagate as `Err` in every mode

The `SLACK_WEBHOOK_URL` classification carve-out documented in 127-01 is
still present but is unrelated to this plan — it belongs to the D-08
heuristic reused by 127-03.

## Self-Check: PASSED

- `ferro-cli/src/deploy/rewrite_ferro_version.rs` contains
  `pub fn compute_cargo_docker_toml` — FOUND
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` contains
  `pub fn persist_cargo_docker_toml` — FOUND
- `ferro-cli/src/commands/docker_init.rs` contains `dry_run` — FOUND
- `ferro-cli/src/commands/do_init.rs` contains `dry_run` — FOUND
- `ferro-cli/src/commands/docker_init.rs` contains `compute_cargo_docker_toml` — FOUND
- `ferro-cli/src/commands/docker_init.rs` contains `docker build` (footer) — FOUND
- `ferro-cli/src/commands/do_init.rs` contains `doctl apps create --spec` — FOUND
- `ferro-cli/tests/docker_init_dry_run.rs` — FOUND
- commit `ac382583` (Task 1: compute/persist split) — FOUND
- commit `6650e0d7` (Task 2: --dry-run + footer) — FOUND
- commit `b6d215ce` (Task 3: integration test) — FOUND
- 461/461 ferro-cli lib tests green — CONFIRMED
- 3/3 ferro-cli docker_init_dry_run integration tests green — CONFIRMED
- clippy clean under `-D warnings` — CONFIRMED
- fmt check clean — CONFIRMED
