---
phase: 122-deploy-scaffold-core-rewrite
verified: 2026-04-07T00:00:00Z
status: human_needed
score: 5/5 automated must-haves verified (1 human check deferred)
human_verification:
  - test: "Regenerate Dockerfile, .dockerignore, .do/app.yaml on real gestiscilo-it/app and gestiscilo-it/mkmenu repos with zero hand edits; build to completion"
    expected: "gestiscilo image contains both gestiscilo + screenshot-worker binaries plus chromium runtime; mkmenu image produces working frontend bundle matching currently-deployed shape; deploy:check fails loudly when ferro ref not pushed"
    why_human: "Reference apps (../../gestiscilo-it/app, ../../gestiscilo-it/mkmenu) not present on this machine; end-to-end Docker build + DO App Platform deploy cannot be automated in verifier; golden tests use hand-authored fixtures, not real apps"
---

# Phase 122: Deploy Scaffold Core Rewrite — Verification Report

**Phase Goal:** Rewrite ferro-cli `docker:init` / `do:init` commands and templates so generated Dockerfile, .dockerignore, .do/app.yaml work for real Ferro apps without hand-patching.

**Verified:** 2026-04-07
**Status:** human_needed (all automated checks pass; end-to-end regeneration on real reference apps deferred to human)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from SCOPE.md Verification bullets)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ferro docker:init` generates Dockerfile with conditional frontend stage, multi-bin, runtime-deps hook, detect-and-copy dirs, GITHUB_TOKEN arg, rust-toolchain detection, workspace-aware chef recipe, ferro-deps rewrite script | VERIFIED | `docker.rs` (14 hits of Context/renderer), `Dockerfile.tpl` (6 placeholder hits incl. `{frontend_stage}`, `{runtime_apt_block}`, `{workspace_copy_*}`, `ARG GITHUB_TOKEN`, `insteadOf`); 5 docker_init unit tests + 4 template scenarios pass |
| 2 | `ferro do:init` generates app.yaml with `--region`, envs from `.env.example` (SECRET classification), databases block on DATABASE_URL, workers per non-server bin, `--repo` validation | VERIFIED | `app.yaml.tpl` has all 4 expected placeholders; `do_init.rs` (19 hits of AppYamlContext/render_app_yaml/is_valid_repo/parse_env_example/is_secret/workers_block); 4 templates::do_spec + 2 commands::do_init tests pass |
| 3 | `ferro deploy:check` pre-flight verifies ferro ref reachable via `git ls-remote`, fails loudly with remediation | VERIFIED | `deploy_check.rs` (13 hits incl. `ls-remote`, `check_ref`, `FERRO_REPO`); 3 local-bare-repo tests pass (`reachable_ref_returns_true`, `unreachable_ref_returns_false`, `invalid_repo_returns_err`) |
| 4 | `.dockerignore` template includes D-20 entries | VERIFIED | `dockerignore.tpl` L46-54: `database.db`, `*.sqlite*`, `.planning/`, `storage/`, `data/` all present verbatim |
| 5 | Shared `project::package_name()` helper replaces duplicated `get_package_name()`; commands walk up to locate Cargo.toml; `--force` overwrite; `--ferro-ref` default `main` | VERIFIED | `project.rs` exposes all 6 helpers (34 grep hits); `docker_init.rs` (23 hits: force/ferro_ref/runtime_deps/render_rewrite_script/render_dockerfile/find_project_root); `get_package_name` absent from command files |

**Score:** 5/5 automated truths verified.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-cli/src/project.rs` | 6 introspection helpers + BinEntry/ProjectDirs | VERIFIED | Exists; 13 project:: tests pass |
| `ferro-cli/src/deploy/{mod,env_example,classify,ferro_deps}.rs` | Pure primitives | VERIFIED | All 4 files exist; 18 deploy:: tests pass |
| `ferro-cli/src/templates/docker.rs` | DockerfileContext + render_dockerfile | VERIFIED | 14 grep hits; 4 scenario tests pass; legacy shim removed |
| `ferro-cli/src/templates/do.rs` | AppYamlContext + render_app_yaml | VERIFIED | Exists; registered as `do_spec` via `#[path]`; 4 tests pass |
| `ferro-cli/src/templates/files/docker/Dockerfile.tpl` | Parameterized skeleton with 11+ placeholders | VERIFIED | 6 placeholder hits for the spot-checked subset |
| `ferro-cli/src/templates/files/docker/dockerignore.tpl` | D-20 entries appended | VERIFIED | All 5 entries grep-confirmed |
| `ferro-cli/src/templates/files/do/app.yaml.tpl` | 6 placeholders (app_name, region, github_repo, envs/databases/workers blocks) | VERIFIED | 4 block placeholders grep-confirmed |
| `ferro-cli/src/commands/docker_init.rs` | run(force, ferro_ref, runtime_deps) orchestrator | VERIFIED | 23 grep hits; 5 unit tests pass |
| `ferro-cli/src/commands/do_init.rs` | run(repo, region, force, ferro_ref) | VERIFIED | 19 grep hits; 2 unit tests pass |
| `ferro-cli/src/commands/deploy_check.rs` | check_ref via git ls-remote | VERIFIED | 13 grep hits; 3 tests pass |
| `ferro-cli/tests/golden.rs` | Snapshot tests for gestiscilo + mkmenu | VERIFIED | Test `golden_gestiscilo_and_mkmenu` passes |
| `ferro-cli/tests/fixtures/{gestiscilo,mkmenu}/` | Hand-authored fixtures + expected outputs | VERIFIED | Both dirs present |
| `ferro-cli/src/lib.rs` | Lib/bin split enabling golden tests | VERIFIED | Exists |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `docker_init::run` | `project::*` helpers | direct calls | WIRED | `find_project_root`, `package_name`, `read_bins`, `read_workspace_members`, `resolve_rust_base_image`, `detect_dirs` all referenced |
| `docker_init::run` | `render_dockerfile` + `render_rewrite_script` | direct calls | WIRED | Both referenced; generates Dockerfile + scripts/rewrite-ferro-deps.sh |
| `do_init::run` | `templates::do_spec::render_app_yaml` | direct call | WIRED | AppYamlContext composed from project+deploy helpers |
| `do_init::run` | `docker_init::generate` | chained call | WIRED | Ensures Dockerfile exists alongside app.yaml |
| `main.rs` | `Commands::DockerInit/DoInit/DeployCheck` | clap dispatch | WIRED | All three variants registered with `--force`, `--ferro-ref`, `--region`, `--runtime-deps`, `--repo` flags |
| Golden test | renderers | library API | WIRED | Lib split enables `use ferro_cli::templates::...` from tests/golden.rs |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All unit tests pass | `cargo test -p ferro-cli` | 343 passed, 0 failed | PASS |
| Golden integration test passes | `cargo test -p ferro-cli --test golden` | 1 passed | PASS |
| Workspace-wide tests pass | (implied by 122-08 SUMMARY `cargo test --all-features` clean) | reported clean | PASS |
| deploy:check against local bare repo | `reachable_ref_returns_true` + `unreachable_ref_returns_false` | both pass | PASS |

### Requirements Coverage (SCOPE.md D-01..D-21)

All 21 decisions declared across plans 01-08 frontmatter (`requirements:` fields cover D-01..D-21, and D-20 via 122-06 which lacks the field but is grep-confirmed in `dockerignore.tpl`).

| Req | Description | Status | Evidence |
|-----|-------------|--------|----------|
| D-01 | Conditional frontend stage | SATISFIED | `Dockerfile.tpl` `{frontend_stage}` placeholder; `has_frontend` probe in `project.rs` |
| D-02 | Multi-binary support | SATISFIED | `BinEntry`, `read_bins`, `cargo_build_bins` helper, runtime_bin_copies; test `multi_bin_chromium_workspace` |
| D-03 | `--runtime-deps` flag + marked apt block | SATISFIED | `# >>> ferro:runtime-deps` markers; `runtime_deps` clap flag; test `runtime_deps_appear_in_dockerfile` |
| D-04 | Detect-and-copy themes/lang/public/migrations | SATISFIED | `detect_dirs` → `ProjectDirs`; conditional copy lines |
| D-05 | ARG GITHUB_TOKEN + insteadOf | SATISFIED | Both grep-confirmed in `Dockerfile.tpl` |
| D-06 | rust-toolchain.toml detection | SATISFIED | `resolve_rust_base_image` helper; test `custom_rust_toolchain` |
| D-07 | Workspace-aware chef recipe | SATISFIED | `read_workspace_members`; `workspace_copy_block` |
| D-08 | `scripts/rewrite-ferro-deps.sh` generator | SATISFIED | `deploy::ferro_deps::render_rewrite_script`; 5 tests |
| D-09 | Dockerfile invokes script in planner+builder | SATISFIED | `{ferro_rewrite_planner/builder}` placeholders, unconditional per 122-03 decision |
| D-10 | `--ferro-ref` flag persisted | SATISFIED | Clap flag on both commands; test `writes_ferro_ref_into_script_header` |
| D-11 | deploy:check pre-flight | SATISFIED | `deploy_check.rs`; 3 tests |
| D-12 | `--region` default fra1 | SATISFIED | Clap default_value = "fra1"; in `app.yaml.tpl` |
| D-13 | envs block + SECRET classification | SATISFIED | `classify::is_secret` + `parse_env_example`; wired into do_init |
| D-14 | databases block on DATABASE_URL | SATISFIED | Conditional databases block in `do.rs` |
| D-15 | workers per non-server bin | SATISFIED | workers_block emits one per bin ≠ package_name; gestiscilo golden fixture exercises this |
| D-16 | `--force` flag | SATISFIED | Clap on both commands; test `refuses_to_overwrite_without_force` + `overwrites_with_force` |
| D-17 | Walk up to Cargo.toml | SATISFIED | `find_project_root` used by both commands |
| D-18 | `--repo` validation | SATISFIED | `is_valid_repo`; test `repo_validation_matrix` (8 cases) |
| D-19 | Lifted `package_name` helper | SATISFIED | `project::package_name`; `get_package_name` absent from commands |
| D-20 | dockerignore additions | SATISFIED | All 5 entries grep-confirmed in `dockerignore.tpl` |
| D-21 | Drift audit note (deferred to P124) | SATISFIED | Note present in `dockerignore.tpl` |

**No orphaned requirements.** All SCOPE.md decisions are implemented and wired.

### Anti-Patterns Found

None blocking. Documented non-issues:

- `#![allow(dead_code)]` in `project.rs` — downstream plan consumers (not real dead code; tests exercise full surface).
- `#![allow(dead_code, unused_imports)]` in `deploy/mod.rs` — same rationale.

Both allowances were removed where consumers arrived (e.g. `do.rs` in 122-05 Task 2).

### Human Verification Required

#### 1. Real reference-app regeneration

**Test:**
1. On a machine with `../../gestiscilo-it/app` and `../../gestiscilo-it/mkmenu` present:
   - Delete `Dockerfile`, `.dockerignore`, `.do/app.yaml` in both repos
   - Run `ferro docker:init --runtime-deps chromium,fonts-liberation` + `ferro do:init --repo gestiscilo-it/app` in gestiscilo
   - Run `ferro docker:init` + `ferro do:init --repo gestiscilo-it/mkmenu` in mkmenu
2. `docker build .` end-to-end for both
3. `ferro deploy:check --ferro-ref <unpushed-local-branch>` against an intentionally unpushed ferro branch

**Expected:**
- Zero hand edits needed for Docker builds to succeed
- gestiscilo image contains both `gestiscilo` and `screenshot-worker` binaries plus `chromium` runtime
- mkmenu image produces working frontend bundle; app.yaml `fra1` region + managed postgres + envs block matches currently-deployed shape
- `deploy:check` exits non-zero with push-instruction message

**Why human:**
- Reference apps not present on this verifier machine
- Full `docker build` + DO App Platform deploy is out of scope for automated verification
- Golden tests in Plan 122-08 use hand-authored fixtures derived from SCOPE.md, not the real repos (acknowledged "Known Gap" in 122-08 SUMMARY)

### Gaps Summary

Zero automated gaps. All 21 SCOPE decisions implemented, all key links wired, all 343 unit tests + golden integration test passing, `cargo test --all-features` workspace clean per 122-08 SUMMARY verification.

The only residual item is the manual end-to-end validation against real `gestiscilo-it/{app,mkmenu}` repos, explicitly flagged as a Known Gap in the Plan 122-08 SUMMARY and deferred to a human on a machine with those repos available. Until that check runs, the phase goal ("work for real Ferro apps without hand-patching") has strong circumstantial evidence but no live empirical confirmation.

---

*Verified: 2026-04-07*
*Verifier: Claude (gsd-verifier)*
