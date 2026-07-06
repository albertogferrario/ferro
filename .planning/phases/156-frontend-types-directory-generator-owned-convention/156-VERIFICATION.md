---
phase: 156-frontend-types-directory-generator-owned-convention
verified: 2026-05-14T02:30:00Z
status: human_needed
score: 13/14 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Confirm that the ferro-rs 0.2.34 version is visible on crates.io at https://crates.io/crates/ferro-rs"
    expected: "Version 0.2.34 appears as the latest release"
    why_human: "Cannot verify external crates.io publish state programmatically without network access; origin/master has 0.2.34 committed and the CI workflow is auto-publish on push"
---

# Phase 156: frontend/src/types/ Generator-Owned Convention Cleanup — Verification Report

**Phase Goal:** Reconcile the contradiction between the scaffold gitignore template (which marks `frontend/src/types/` as generator-owned) and Ferro's reference app (which tracks generated files). Untrack generated files in the reference app, add a `ferro doctor` check for hand-written files in `frontend/src/types/`, update the Dockerfile renderer to add a `types-gen` stage so Docker builds work without committed generated files, fix the generator header comment, and document the convention.
**Verified:** 2026-05-14T02:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `git ls-files` reports no entries under `app/frontend/src/types/` | VERIFIED | `git ls-files app/frontend/src/types/` returns 0 lines; commit `63f6e8bc` performed `git rm --cached` |
| 2 | `gitignore.tpl` carries an explicit load-bearing comment naming the convention | VERIFIED | Line present: `# generated_types — load-bearing: frontend/src/types/ is owned by ferro generate-types.` and `# Removing this rule breaks the generator-owned convention (see docs/src/cli/frontend-types.md).` |
| 3 | `generate_types.rs` header directs custom types to `frontend/src/lib/types/` (not `frontend/src/types/`) | VERIFIED | Line 711: `output.push_str("// frontend/src/lib/types/\n");` — old path removed; companion test updated at line 1995 |
| 4 | `ferro doctor` reports an additional check named `frontend_types_convention` | VERIFIED | `ferro-cli/src/doctor/checks/frontend_types_convention.rs` exists (147 lines); registered at position 11 in `default_checks()` |
| 5 | The check returns OK when `frontend/src/types/` is absent or contains only `inertia-props.ts`/`routes.ts` | VERIFIED | 3 unit tests cover absent dir, only-generated, only-routes-ts cases — all pass |
| 6 | The check returns WARN listing hand-written filenames with redirect to `frontend/src/lib/types/` | VERIFIED | `hand_written_file_warns` and `mixed_generated_and_hand_written_warns_on_hand_written_only` tests confirm this; details contain filename and path |
| 7 | The check is capped at WARN, never ERROR, and does not block doctor exit code | VERIFIED | No `CheckResult::error` call in `frontend_types_convention.rs`; IO errors return `Ok("unreadable (skipped)")` |
| 8 | `default_checks()` returns 11 entries with `frontend_types_convention` as the last entry | VERIFIED | Registry test `default_checks_returns_eleven_in_declared_order` asserts `checks.len() == 11`; `FrontendTypesConventionCheck` is at index 10 |
| 9 | `DockerContext` exposes a `ferro_version: String` field; `render_dockerfile` substitutes `{{FERRO_VERSION}}`; `types-gen` stage emitted when `has_frontend == true` | VERIFIED | `pub ferro_version: String` in struct; `.replace("{{FERRO_VERSION}}", &ctx.ferro_version)` in render chain; `TYPES_GEN_STAGE_BODY` const present; 9 renderer tests pass |
| 10 | `resolve_ferro_version` parses `Cargo.lock` for `ferro-rs` and falls back to `env!("CARGO_PKG_VERSION")` | VERIFIED | `pub fn resolve_ferro_version(project_root: &Path) -> String` at line 188; matches `name == Some("ferro-rs")`; 3 resolver unit tests pass |
| 11 | Both `docker_init.rs` and `docker_template_drift.rs` call sites use `resolve_ferro_version` (no `env!` placeholders remain) | VERIFIED | `docker_init.rs` line 72: `unwrap_or_else(\|\| resolve_ferro_version(&root))`; `docker_template_drift.rs` line 73: `ferro_version: resolve_ferro_version(root),`; zero "Plan 04 will replace" comments in workspace |
| 12 | `docs/src/cli/frontend-types.md` exists and covers the convention end-to-end | VERIFIED | File exists (116 lines); contains all required sections: generator output table, gitignore rationale, hand-written types location (`frontend/src/lib/types/`), fresh-clone bootstrap (`cargo run`), Docker types-gen stage, `ferro docker:init --force` upgrade path, TS2307 error, related commands |
| 13 | Workspace version bumped to 0.2.34; CHANGELOG entry covers Phase 156 deliverables | VERIFIED | `Cargo.toml`: `version = "0.2.34"`; `Cargo.lock` ferro-rs entry: `version = "0.2.34"`; CHANGELOG contains `[0.2.34] — 2026-05-14`, `Phase 156`, `frontend_types_convention`, `types-gen`, `ferro docker:init --force`, `resolve_ferro_version` |
| 14 | New version published to crates.io | NEEDS HUMAN | `origin/master` is at `0.2.34` (push confirmed — all Phase 156 commits on remote); CI auto-publish should have fired; cannot verify crates.io state programmatically |

**Score:** 13/14 truths verified (1 needs human confirmation)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-cli/src/templates/files/root/gitignore.tpl` | Load-bearing comment naming convention + doc cross-reference | VERIFIED | Contains `load-bearing: frontend/src/types/ is owned by` and `see docs/src/cli/frontend-types.md` |
| `ferro-cli/src/commands/generate_types.rs` | Corrected header comment pointing to `frontend/src/lib/types/` | VERIFIED | Line 711 corrected; old path absent |
| `ferro-cli/src/doctor/checks/frontend_types_convention.rs` | `FrontendTypesConventionCheck` + `DoctorCheck` impl + 6 unit tests | VERIFIED | 147 lines; struct, allowlist, trait impl, 6 tests all present |
| `ferro-cli/src/doctor/checks/mod.rs` | `pub mod` + `pub use` for the new check, alphabetically placed | VERIFIED | Between `docker_template_drift` and `generated_artifacts` (alphabetical) |
| `ferro-cli/src/doctor/registry.rs` | `default_checks()` returning 11 entries; updated count test | VERIFIED | `Box::new(FrontendTypesConventionCheck)` appended; test asserts 11 |
| `ferro-cli/src/templates/docker.rs` | `DockerContext.ferro_version` + `TYPES_GEN_STAGE_BODY` + `FRONTEND_STAGE_WITH_TYPES_COPY_BODY` + `resolve_ferro_version` + 9 new tests | VERIFIED | All constants, field, helper, and tests present; 28 docker template tests pass total |
| `ferro-cli/src/commands/docker_init.rs` | `resolve_ferro_version(&root)` call replacing Plan 03 placeholder; 2 smoke tests | VERIFIED | Wired; `ferro_version_flag` active (CLI override); 2 new tests present |
| `ferro-cli/src/doctor/checks/docker_template_drift.rs` | `resolve_ferro_version(root)` call replacing Plan 03 placeholder | VERIFIED | Wired in `check_impl`; test fixtures retain `"0.0.0-test"` |
| `docs/src/cli/frontend-types.md` | Canonical convention reference page (min 60 lines) | VERIFIED | 116 lines; all D-08 content items covered |
| `docs/src/SUMMARY.md` | Index entry for frontend-types page | VERIFIED | `  - [frontend-types](cli/frontend-types.md)` present after `[doctor]` |
| `docs/src/cli/doctor.md` | Eleven checks; 11-row table including `docker_template_drift` + `frontend_types_convention`; "eleven names" | VERIFIED | "Runs eleven checks"; 11 numbered rows; both checks present; "eleven names" |
| `docs/src/reference/cli.md` | Doctor row says "eleven checks" | VERIFIED | `Run project health diagnostics (eleven checks)` |
| `ferro-cli/src/templates/files/root/README.md.tpl` | Troubleshooting bullet for missing types on fresh clone | VERIFIED | Bullet with `Cannot find module './types/inertia-props'` and `cargo run` present |
| `Cargo.toml` | Workspace version bumped to 0.2.34 | VERIFIED | `version = "0.2.34"` |
| `CHANGELOG.md` | Phase 156 entry under `## ferro-rs` section | VERIFIED | `### [0.2.34] — 2026-05-14` with all required strings |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `gitignore.tpl` load-bearing comment | `docs/src/cli/frontend-types.md` | Doc cross-reference in comment body | WIRED | Comment contains `see docs/src/cli/frontend-types.md` |
| `generate_types.rs` lines 710-711 | `frontend/src/lib/types/` scaffold convention | `output.push_str("// frontend/src/lib/types/\n")` | WIRED | Line 711 confirmed |
| `ferro-cli/src/doctor/registry.rs default_checks()` | `FrontendTypesConventionCheck` | `Box::new(FrontendTypesConventionCheck)` appended to vec | WIRED | Grep confirms presence |
| `ferro-cli/src/doctor/checks/mod.rs` | `frontend_types_convention::FrontendTypesConventionCheck` | `pub mod` + `pub use` re-export pair | WIRED | Both lines confirmed |
| `FrontendTypesConventionCheck::check_impl` | `frontend/src/types/` directory entries | `std::fs::read_dir` filtered against `GENERATED_ALLOWLIST` | WIRED | `read_dir` call at line ~185 confirmed |
| `DockerContext.ferro_version` | `{{FERRO_VERSION}}` substitution in `DOCKERFILE_TPL` | `.replace("{{FERRO_VERSION}}", &ctx.ferro_version)` | WIRED | Present in render chain |
| `render_dockerfile` when `has_frontend==true` | `TYPES_GEN_STAGE_BODY` + `FRONTEND_STAGE_WITH_TYPES_COPY_BODY` | `format!("{TYPES_GEN_STAGE_BODY}{FRONTEND_STAGE_WITH_TYPES_COPY_BODY}")` | WIRED | Confirmed |
| `resolve_ferro_version(root)` | `Cargo.lock` parsed for `ferro-rs` | `name == Some("ferro-rs")` | WIRED | Line 195 confirmed |
| `docker_init::execute` | `templates::docker::resolve_ferro_version` | Import + `resolve_ferro_version(&root)` call | WIRED | Import at line 21; call at line 72 |
| `docker_template_drift::check_impl` | `templates::docker::resolve_ferro_version` | Import + `resolve_ferro_version(root)` call | WIRED | Import at line 12; call at line 73 |
| `docs/src/SUMMARY.md` | `docs/src/cli/frontend-types.md` | Markdown link in Reference -> CLI section | WIRED | `[frontend-types](cli/frontend-types.md)` confirmed |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `render_dockerfile` | `ferro_version` in rendered Dockerfile | `resolve_ferro_version` parses `Cargo.lock` for `ferro-rs` package version | Yes — Cargo.lock lookup with `env!` fallback; 3 unit tests verify both paths | FLOWING |
| `FrontendTypesConventionCheck::check_impl` | `hand_written` vec from `std::fs::read_dir` | Live directory read filtered against `GENERATED_ALLOWLIST` | Yes — reads real filesystem; IO errors return Ok (safe fallback) | FLOWING |

### Behavioral Spot-Checks

Step 7b skipped for documentation artifacts (Plans 01, 05, 06). Code artifacts verified via unit test evidence from summaries:

| Behavior | Test | Result | Status |
|----------|------|--------|--------|
| `FrontendTypesConventionCheck` warns on hand-written file | `hand_written_file_warns` unit test | 6 tests pass (verified in SUMMARY-02) | PASS |
| `resolve_ferro_version` reads Cargo.lock | `resolve_ferro_version_reads_cargo_lock` unit test | 9 resolver+renderer tests pass (verified in SUMMARY-03) | PASS |
| Rendered Dockerfile pins to Cargo.lock version | `dockerfile_pins_to_cargo_lock_ferro_version` smoke test | 2 smoke tests pass (verified in SUMMARY-04) | PASS |
| `types-gen` stage present when `has_frontend==true` | `types_gen_stage_present_when_has_frontend` unit test | 28 docker template tests pass (verified in SUMMARY-03) | PASS |
| `COPY --from=types-gen` precedes `RUN npm run build` | `copy_from_types_gen_before_npm_build` unit test | Pass (verified in SUMMARY-03) | PASS |

### Requirements Coverage

Requirements D-05, D-06, D-08, D-09, D-10, D-11, D-13, D-14, D-15, D-16, D-17, D-18, D-20 were specified across the 6 plans. REQUIREMENTS.md does not exist in this project (requirements are tracked as D-IDs in CONTEXT.md). Coverage mapped to plan must_haves:

| Requirement | Plan | Coverage | Status |
|-------------|------|----------|--------|
| D-05 (untrack reference app types) | 01 | `git ls-files app/frontend/src/types/` returns 0 | SATISFIED |
| D-06 (gitignore.tpl load-bearing comment) | 01 | Comment with convention name + doc cross-reference | SATISFIED |
| D-08 (canonical docs page) | 05 | `docs/src/cli/frontend-types.md` exists with all D-08 content | SATISFIED |
| D-09 (doctor check, general category) | 02 | `FrontendTypesConventionCheck` in `CheckCategory::General` | SATISFIED |
| D-10 (check severity WARN not ERROR) | 02 | Hard-capped at Warn; IO errors return Ok | SATISFIED |
| D-11 (README.md.tpl troubleshooting) | 05 | Troubleshooting bullet with `cargo run` | SATISFIED |
| D-13 (no new crate introduced) | 06 | Only existing `ferro-cli` modified | SATISFIED |
| D-14 (CHANGELOG entry) | 06 | `### [0.2.34] — 2026-05-14` with all deliverables | SATISFIED |
| D-15 (Dockerfile types-gen stage) | 03 | `TYPES_GEN_STAGE_BODY` emitted when `has_frontend==true` | SATISFIED |
| D-16 (DockerContext.ferro_version + resolve_ferro_version) | 03 | Field and helper present; Cargo.lock parsing | SATISFIED |
| D-17 (COPY --from=types-gen before npm run build) | 03 | Ordering verified by `copy_from_types_gen_before_npm_build` test | SATISFIED |
| D-18 (generate_types.rs header path fix) | 01 | Line 711 corrected; old path absent | SATISFIED |
| D-20 (check count test updated to 11) | 02 | `default_checks_returns_eleven_in_declared_order` asserts `len() == 11` | SATISFIED |
| D-21 (resolve_ferro_version wired to call sites) | 04 | Both `docker_init.rs` and `docker_template_drift.rs` wired | SATISFIED |

### Anti-Patterns Found

No blockers, warnings, or notable anti-patterns found:
- No TODO/FIXME/HACK/PLACEHOLDER comments in modified Rust source files
- No "Plan 04 will replace" placeholder comments remain in workspace (0 matches)
- No stale "nine checks"/"nine names" references remain in doctor documentation
- `FRONTEND_STAGE_BODY` dead constant was deleted per no-dead-code convention (confirmed in SUMMARY-03)
- `#[allow(dead_code)]` on `resolve_ferro_version` was a legitimate temporary measure in Plan 03 — removed in Plan 04 when call sites were wired (confirmed in SUMMARY-04)

### Human Verification Required

### 1. crates.io Publish Confirmation

**Test:** Visit https://crates.io/crates/ferro-rs and confirm version 0.2.34 is listed as the latest version. Alternatively, run `cargo search ferro-rs` to see the latest published version.
**Expected:** Version 0.2.34 appears as the latest release on crates.io.
**Why human:** Cannot verify external crates.io publication state programmatically. `origin/master` confirms the version bump commit (`652b8ae6`) is on the remote, and the `.github/workflows/publish.yml` CI workflow auto-publishes on master push. The STATE.md checkpoint note confirms this is the outstanding human-action item.

### Gaps Summary

No gaps. All 13 programmatically verifiable must-haves pass. The single outstanding item is external state verification (crates.io publish) which is a human checkpoint by design (Plan 06 Task 3 is `checkpoint:human-action`).

---

_Verified: 2026-05-14T02:30:00Z_
_Verifier: Claude (gsd-verifier)_
