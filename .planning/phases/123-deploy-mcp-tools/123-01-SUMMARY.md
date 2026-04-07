---
phase: 123-deploy-mcp-tools
plan: 01
subsystem: ferro-cli/deploy
tags: [deploy, runtime-deps, registry, mcp]
requires: []
provides:
  - ferro_cli::deploy::runtime_deps::RUNTIME_DEP_REGISTRY
  - ferro_cli::deploy::runtime_deps::scan_runtime_deps
  - ferro_cli::deploy::runtime_deps::scan_runtime_deps_str
  - ferro_cli::deploy::runtime_deps::scan_runtime_dep_matches
affects: []
tech_added: []
patterns: [toml::Value parsing, &'static registry, flat_map dedup sort]
files_created:
  - ferro-cli/src/deploy/runtime_deps.rs
files_modified:
  - ferro-cli/src/deploy/mod.rs
decisions:
  - Registry duplicates DEP_TABLES constant locally (ferro_deps.rs keeps its copy private)
  - Malformed TOML returns empty Vec (mirrors ferro_deps.rs tolerance)
  - Exact Cargo.toml key spelling required (no dash/underscore normalization)
metrics:
  duration: ~10min
  tasks: 1
  tests_added: 9
  completed: 2026-04-07
---

# Phase 123 Plan 01: Runtime Deps Registry Summary

Registry + Cargo.toml scanner mapping ferro-ecosystem crates to Debian apt runtime packages, acting as the single source of truth shared between the upcoming `runtime_requirements` MCP tool (Phase 123) and `docker:init --runtime-deps` (Phase 122).

## What Was Built

`ferro-cli/src/deploy/runtime_deps.rs`:

- `RuntimeDep { crate_name, apt_packages }` struct with `&'static` fields.
- `RUNTIME_DEP_REGISTRY` const slice with 4 entries: chromiumoxide, headless_chrome, ffmpeg-next, pdfium.
- `scan_runtime_deps_str(content) -> Vec<String>` — parses TOML, flat_maps apt packages, sorts+dedups.
- `scan_runtime_dep_matches(content) -> Vec<&'static RuntimeDep>` — returns static refs for richer MCP reports.
- `scan_runtime_deps(path) -> io::Result<Vec<String>>` — reads file, delegates to `_str` variant.
- Module re-exported via `ferro-cli/src/deploy/mod.rs` for cross-crate (ferro-mcp) consumption.

## Test Coverage (9 tests)

- chromiumoxide → `[chromium, fonts-liberation]`
- chromiumoxide + headless_chrome dedup
- ffmpeg-next table form
- pdfium both string and table forms
- Unknown crates (serde, tokio) return empty
- Malformed TOML returns empty (tolerant parse)
- Scans across `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`
- `scan_runtime_dep_matches` exposes `&'static` refs
- `scan_runtime_deps` Path variant reads real tempfile

All pass; `cargo clippy -p ferro-cli --all-targets --no-deps -- -D warnings` clean.

## Deviations from Plan

None — plan executed exactly as written.

## Deferred Issues (Out of Scope)

- `ferro-json-ui/src/render.rs:391` has a pre-existing `clippy::uninlined_format_args` warning unrelated to this plan. Logged to `deferred-items.md`. ferro-cli itself is lint-clean.

## Commits

- `da54686a` feat(123-01): add runtime_deps registry and Cargo.toml scanner

## Self-Check: PASSED

- ferro-cli/src/deploy/runtime_deps.rs: FOUND
- ferro-cli/src/deploy/mod.rs: FOUND (updated)
- Commit da54686a: FOUND
- 9/9 runtime_deps tests pass
- ferro-cli clippy clean
