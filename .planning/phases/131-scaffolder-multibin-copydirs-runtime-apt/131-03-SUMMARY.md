---
phase: 131-scaffolder-multibin-copydirs-runtime-apt
plan: "03"
subsystem: ferro-cli deploy scaffolders
tags: [deploy, read_bins, refactor, continuous-coherence, wave-3]
dependency_graph:
  requires:
    - ferro-cli/src/project.rs (canonical read_bins, from 131-01)
    - ferro-cli/src/templates/docker.rs (DockerContext, from 131-01)
    - ferro-cli/src/doctor/checks/docker_template_drift.rs (from 131-02)
    - ferro-cli/tests/gestiscilo_fixture.rs (byte-identical tests, from 131-01/02)
  provides:
    - Single canonical read_bins in project.rs returning Vec<BinEntry>
    - Zero references to templates::docker::read_bins anywhere in the crate
  affects:
    - ferro-cli/src/commands/docker_init.rs
    - ferro-cli/src/commands/do_init.rs
    - ferro-cli/src/deploy/bin_detect.rs
    - ferro-cli/src/doctor/checks/docker_template_drift.rs
    - ferro-cli/tests/gestiscilo_fixture.rs
tech_stack:
  added: []
  patterns:
    - Conversion at call site: project::read_bins() -> Vec<BinEntry> mapped to Vec<String> with .into_iter().map(|b| b.name).collect()
    - Pure render boundary: DockerContext.bins stays Vec<String> — no project-module types leak into the template module
key_files:
  created: []
  modified:
    - ferro-cli/src/templates/docker.rs (read_bins deleted; DockerContext.bins doc comment updated)
    - ferro-cli/src/commands/docker_init.rs (import updated; Vec<BinEntry>->Vec<String> at call site)
    - ferro-cli/src/commands/do_init.rs (import updated; Vec<BinEntry>->Vec<String> at call site)
    - ferro-cli/src/deploy/bin_detect.rs (import updated; .name field access in comparisons)
    - ferro-cli/src/doctor/checks/docker_template_drift.rs (import updated; Vec<BinEntry>->Vec<String>)
    - ferro-cli/tests/gestiscilo_fixture.rs (import updated; Vec<BinEntry>->Vec<String> at all call sites)
decisions:
  - "project::read_bins wins — richer type (Vec<BinEntry>), lives in the pure-reader module, already referenced by deploy::bin_detect"
  - "DockerContext.bins stays Vec<String> — renderer only needs names; this preserves the pure-render boundary (no project types in template module)"
  - "Conversion at call site (.into_iter().map(|b| b.name).collect()) — explicit, no adapter layer, easy to read"
metrics:
  duration: ~8min
  completed: "2026-04-09"
  tasks: 1
  files: 6
requirements_satisfied:
  - REQ-131-01 (Dockerfile bin set and DO app.yaml bin set now use the same reader by construction)
  - REQ-131-02 (same reader guarantees agreement between docker:init and do:init)
---

# Phase 131 Plan 03: read_bins Deduplication Summary

Collapses the latent `read_bins` duplication between `project.rs` and `templates/docker.rs` into a single canonical function, ensuring Dockerfile and `.do/app.yaml` cannot disagree about which bins exist.

## One-liner

Single canonical `project::read_bins` returning `Vec<BinEntry>`; `templates::docker::read_bins` (returning `Vec<String>`) deleted; all five call sites updated with name-conversion at the boundary.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Pick canonical read_bins and delete the duplicate | c4e96864 | templates/docker.rs, commands/docker_init.rs, commands/do_init.rs, deploy/bin_detect.rs, doctor/checks/docker_template_drift.rs, tests/gestiscilo_fixture.rs |

## Key Implementation Details

### Selected canonical function

`project::read_bins(root: &Path) -> Vec<BinEntry>` is the winner:
- Richer type (`BinEntry` carries both `name` and `path`; future callers may need `path`)
- Lives in the "pure reader" module (`project.rs`), not the "template" module
- Was already referenced by `deploy::bin_detect::detect_web_bin` (indirectly via `templates::docker::read_bins` before; now directly)
- Infallible return (returns empty `Vec` rather than `Err` on missing Cargo.toml) — appropriate for a reader that may run in contexts where Cargo.toml absence is expected

### DockerContext.bins stays Vec<String>

The renderer (`render_dockerfile`) only needs bin names for the `COPY` lines and entrypoint. Keeping `DockerContext.bins: Vec<String>` preserves the pure-render boundary: no types from the `project` module leak into the `templates` module. The conversion happens at each call site with `.into_iter().map(|b| b.name).collect()`.

### Call sites updated

| File | Before | After |
|------|--------|-------|
| `commands/docker_init.rs` | `use templates::docker::read_bins; read_bins(&root)?` | `use project::read_bins; read_bins(&root).into_iter().map(|b| b.name).collect()` |
| `commands/do_init.rs` | `use templates::docker::read_bins; read_bins(&root)?` | `use project::read_bins; read_bins(&root).into_iter().map(|b| b.name).collect()` |
| `deploy/bin_detect.rs` | `use templates::docker::read_bins; bins.iter().any(|b| b == &pkg)` | `use project::read_bins; bins.iter().any(|b| b.name == pkg)` |
| `doctor/checks/docker_template_drift.rs` | `use templates::docker::{read_bins, ...}; read_bins(root)?` | `use project::read_bins; read_bins(root).into_iter().map(...)` |
| `tests/gestiscilo_fixture.rs` | `use ferro_cli::templates::docker::read_bins; read_bins(&root).expect(...)` | `use ferro_cli::project::read_bins; read_bins(&root).into_iter().map(|b| b.name)` |

### Gestiscilo byte-identical tests

All 131-01 and 131-02 tests remain green after the refactor, including:
- `dockerfile_matches_gestiscilo_6f6d397`
- `app_yaml_matches_gestiscilo_6f6d397`
- `dockerfile_covers_every_bin`
- `app_yaml_workers_from_non_web_bins`

The binary output is identical because the name-conversion path produces the same strings the old `Vec<String>` reader produced.

## Deviations from Plan

None. The plan executed exactly as written.

- Selection rule confirmed: `project::read_bins` wins.
- `DockerContext.bins` stays `Vec<String>` as the plan recommended (conversion approach).
- All five call sites updated (plan listed four; `tests/gestiscilo_fixture.rs` was a fifth, discovered by running clippy — Rule 3 auto-fix).

## Known Stubs

None.

## Self-Check: PASSED
