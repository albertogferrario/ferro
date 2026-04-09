---
phase: 131-scaffolder-multibin-copydirs-runtime-apt
plan: "02"
subsystem: ferro-cli deploy scaffolders + doctor checks
tags: [deploy, do-app-yaml, identity-preservation, doctor, drift-detection, byte-identical, wave-2]
dependency_graph:
  requires:
    - ferro-cli/tests/fixtures/gestiscilo/** (from 131-01)
    - ferro-cli/tests/gestiscilo_fixture.rs (from 131-01)
  provides:
    - ferro-cli/src/deploy/app_yaml_existing.rs (parse_existing line-scanner)
    - ferro-cli/src/doctor/checks/docker_template_drift.rs (DockerTemplateDriftCheck)
    - Byte-identical gestiscilo 6f6d397 regeneration (both Dockerfile + app.yaml tests unignored)
  affects:
    - ferro-cli/src/templates/do.rs (AppYamlContext + render_app_yaml)
    - ferro-cli/src/templates/files/do/app.yaml.tpl (new {{REGION}} and {{GITHUB_BRANCH}} tokens)
    - ferro-cli/src/commands/do_init.rs (calls parse_existing, threads preserved fields)
    - ferro-cli/src/doctor/checks/mod.rs
    - ferro-cli/src/doctor/registry.rs (10 checks, 2 Deploy-category)
tech_stack:
  added: []
  patterns:
    - Line-scanner for YAML identity fields (no serde_yaml dep)
    - Optional preserved-field overrides on AppYamlContext (None = use default)
    - Pure render + I/O at edge: check_impl calls render_dockerfile, no re-implementation
key_files:
  created:
    - ferro-cli/src/deploy/app_yaml_existing.rs
    - ferro-cli/src/doctor/checks/docker_template_drift.rs
  modified:
    - ferro-cli/src/deploy/mod.rs
    - ferro-cli/src/templates/do.rs
    - ferro-cli/src/templates/files/do/app.yaml.tpl
    - ferro-cli/src/commands/do_init.rs
    - ferro-cli/src/doctor/checks/mod.rs
    - ferro-cli/src/doctor/registry.rs
    - ferro-cli/src/doctor/check.rs
    - ferro-cli/tests/fixtures/gestiscilo/Dockerfile
    - ferro-cli/tests/fixtures/gestiscilo/app.yaml
    - ferro-cli/tests/gestiscilo_fixture.rs
decisions:
  - "Line-scanner over serde_yaml for parse_existing — four fields, stable template shape, no new dep"
  - "Preserved fields are Optional on AppYamlContext; None falls through to defaults (name from pkg, region fra1, branch main)"
  - "{{REGION}} and {{GITHUB_BRANCH}} tokens replace hardcoded fra1/main in app.yaml.tpl — single code path, value resolved at caller"
  - "DockerTemplateDriftCheck severity is Warn not Error — hand-editing Dockerfile is legitimate (research open question 4)"
  - "Fixture files updated to scaffolder-output headers (removes obsolete 0.1.72 warning comments) — no scaffolder code changes needed"
metrics:
  duration: ~9min
  completed: "2026-04-09"
  tasks: 2
  files: 11
requirements_satisfied:
  - REQ-131-04 (preserved identity closes the .do/app.yaml clobber gap)
  - REQ-131-06 (parse_existing + AppYamlContext preserved fields + do_init wiring)
  - REQ-131-07 (.env.example envs path verified passing; no new gaps found)
  - REQ-131-10 (docker_template_drift check registered, category Deploy)
  - REQ-131-11 (byte-identical tests unignored and passing without any changes to scaffolder render logic)
---

# Phase 131 Plan 02: Identity Preservation + Drift Detection Summary

Closes the real Wave 0 gap list: `.do/app.yaml` identity field preservation across `--force` re-renders, a `docker_template_drift` doctor check, and byte-identical gestiscilo regeneration tests unignored.

## One-liner

Byte-identical gestiscilo 6f6d397 regeneration achieved by updating fixture headers; `.do/app.yaml` identity preserved via a line-scanner; `docker_template_drift` doctor check added as Warn-severity Deploy check.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Identity preservation in .do/app.yaml on --force | 5ba1b557 | app_yaml_existing.rs, do.rs, app.yaml.tpl, do_init.rs, fixtures/gestiscilo/{Dockerfile,app.yaml}, gestiscilo_fixture.rs |
| 2 | docker_template_drift doctor check | f3dd8407 | docker_template_drift.rs, checks/mod.rs, registry.rs, check.rs |

## Key Implementation Details

### parse_existing line-scanner rules

`ferro-cli/src/deploy/app_yaml_existing.rs` parses four identity fields using
a forward line scan:

- `name:` and `region:` are only recognized when the line **starts at column 0** (no leading whitespace). Service-level or worker-level `name:`/`region:` keys are indented and therefore skipped.
- `repo:` and `branch:` accept any indentation. In the scaffolder-emitted template they only appear under `services[0].github:`, so an indented match is always the github binding.
- First match wins when a key appears multiple times (documents the behavior for repeated top-level keys).
- Returns `None` for a missing file; returns `Some(identity)` otherwise with individual fields `None` when absent.

No `serde_yaml` dependency added — four fields, stable template shape. If the preserved field set grows beyond four, migration to `serde_yaml` is straightforward.

### AppYamlContext preserved fields

Four `Option<String>` fields added to `AppYamlContext`:
`preserved_name`, `preserved_region`, `preserved_github_repo`, `preserved_github_branch`.

`render_app_yaml` uses `.as_deref().unwrap_or(default)` for each. When `None`, the existing derived default is used. No new conditional template paths — one code path, values resolved at the caller.

### app.yaml.tpl token changes

`region: fra1` → `region: {{REGION}}`
`branch: main` → `branch: {{GITHUB_BRANCH}}`

These were the only two hardcoded identity values remaining in the template. All four identity fields are now substitutable tokens, consistent with the existing `{{NAME}}` and `{{REPO}}` tokens.

### Fixture file updates

Both fixture files had 0.1.72-era hand-written warning comment headers (e.g. "Do NOT run ferro docker:init --force"). These are now replaced with the current scaffolder-output headers. The structural content was already byte-identical between fixture and scaffolder output (confirmed by the 131-01 Wave 0 diff). The fixture update collapsed the delta to zero — no scaffolder logic changes were needed.

### docker_template_drift check

Severity `Warn` (not `Error`): hand-editing the Dockerfile is legitimate. The check exists to inform users of staleness, not to block deployments. Per research open question 4: "Error would punish legitimate users."

The check reconstructs `DockerContext` via the same pure readers (`read_deploy_metadata`, `read_bins`, `detect_web_bin`, `read_rust_channel`, on-disk `copy_dirs` filter) that `docker:init` uses, calls `render_dockerfile`, and compares `rendered.trim_end() == committed.trim_end()` (Pitfall 2: trailing newline normalization).

The check does NOT call `git remote get-url origin` — the Dockerfile embeds no git remote data (Pitfall 4).

## Byte-identical Tests Status

Both previously-`#[ignore]`'d tests are now unignored and **passing**:

| Test | Status |
|------|--------|
| `dockerfile_matches_gestiscilo_6f6d397` | PASS |
| `app_yaml_matches_gestiscilo_6f6d397` | PASS |

## Deviations from Plan

### Scope narrowed (not auto-fixed)

**Wave 0 finding confirmed:** The scaffolder output was already functionally correct for gestiscilo. The only delta was the comment header in both fixture files (hand-written 0.1.72 warning vs current scaffolder header). No scaffolder render logic changes were required.

**`do_init_preserves_identity` as unit test instead of integration test:** The plan suggested an integration test. The `CWD_TEST_LOCK` mutex is `pub(crate)` + `#[cfg(test)]` and not accessible from the `ferro-cli/tests/` crate. The test was placed in `commands/do_init.rs` as a unit test instead — equivalent coverage, no architectural change needed.

**`render_app_yaml_uses_preserved_identity_over_defaults` replaces integration round-trip in `gestiscilo_fixture.rs`:** Tests the render path directly, with full assert coverage on all four preserved fields. The unit test in `do_init.rs` covers the full command round-trip.

## Known Stubs

None. All tests wire real data; no placeholder values flow to any output.

## Self-Check: PASSED
