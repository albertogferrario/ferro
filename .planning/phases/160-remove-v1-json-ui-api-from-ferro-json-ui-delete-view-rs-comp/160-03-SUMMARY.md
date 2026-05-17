---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
plan: 03
subsystem: mcp
tags: [ferro-mcp, application_info, json-ui-v2, scanner-rewrite, tdd]

# Dependency graph
requires:
  - phase: 115-json-ui-architecture-formalization
    provides: "Pre-sanction for view_count semantic flip (v1 .rs -> v2 .json) — MCP output type change accepted at framework level"
  - phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
    provides: "Plan 01-02 already cleared v1 doc/template framing in ferro-mcp"
provides:
  - "ferro-mcp `application_info` MCP tool reports correct view count for v2 JSON spec projects"
  - "`scan_json_ui_specs` counts `*.json` under `src/views/` (was: `.rs` files, excluding mod.rs)"
  - "Four unit tests covering happy path, missing dir, empty dir, non-json filter"
  - "Neutral doc comment on `scan_json_ui_specs` — no `legacy`, no `v1`, no `TODO(Phase 120)`"
affects: [160-04, 160-05, 160-06, 160-07, 160-08, 160-09, 160-10, 161]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Scanner semantic-flip with field-shape preservation: change what `view_count` counts without touching the `JsonUiSpecsStatus` wire contract (`available`, `view_count`, `views_dir`, `hint` field names + types unchanged)"
    - "Project-relative path literal for MCP output (V14 security control): `views_dir_display = \"src/views/\"` stays a literal — never `views_dir.display().to_string()` (would leak absolute paths including `/Users/<name>`)"
    - "TDD on scanner rewrite: RED (4 tests, 1 fails against v1 impl) -> GREEN (replace function body, 4/4 pass) -> no REFACTOR needed (paste-ready body already minimal)"

key-files:
  created: []
  modified:
    - ferro-mcp/src/tools/application_info.rs

key-decisions:
  - "Used `Path::extension().is_some_and(|ext| ext == \"json\")` (clippy-clean idiom) instead of `.map(...).unwrap_or(false)` — matches RESEARCH Pattern 2 paste-ready body verbatim"
  - "Kept `JsonUiSpecsStatus` struct definition unchanged (lines 57-62): MCP wire contract preserved; only the meaning of `view_count` changes (pre-sanctioned per Phase 115-04 SUMMARY)"
  - "Updated both hint strings: missing-dir hint now references `JsonUi::render_file(\"views/{name}.json\", data)` and `json_ui_generate` MCP tool; empty-dir hint reads `Views directory exists but no JSON spec files found.` — both neutral, no legacy framing"

patterns-established:
  - "MCP scanner rewrite under Phase 160 surface-reduction: rewrite function body + adjust hints + add unit tests in one atomic GREEN commit, preceded by a RED commit that captures the contract via failing tests"

requirements-completed: [D-05, Pattern-2]

# Metrics
duration: 4min
completed: 2026-05-17
---

# Phase 160 Plan 03: Rewrite scan_json_ui_specs to count v2 JSON spec files Summary

**`ferro-mcp::application_info::scan_json_ui_specs` now counts `*.json` spec files under `src/views/` (the v2 surface) instead of `.rs` files; `JsonUiSpecsStatus` wire contract preserved; four `scan_json_ui_specs_*` unit tests pass.**

## Performance

- **Duration:** ~4 min
- **Tasks:** 1 (TDD: RED + GREEN commits)
- **Files modified:** 1

## Accomplishments
- Rewrote `scan_json_ui_specs` body (lines 244-289 of the pre-edit file) to scan `*.json` files via `Path::extension().is_some_and(|ext| ext == "json")` — paste-ready body from RESEARCH Pattern 2 used verbatim.
- Deleted the `Scans for legacy v1 patterns. TODO(Phase 120):` doc-comment header; replaced with a neutral 4-line description anchored on `JsonUi::render_file("views/{name}.json", ..)`.
- Updated both hint strings to neutral v2 language (no `legacy`, no `v1`, no `TODO(Phase 120)`).
- Added a `#[cfg(test)] mod tests { ... }` block with four `scan_json_ui_specs_*` unit tests:
  - `scan_json_ui_specs_counts_json_files` — 2 `.json` files -> `view_count == 2`, `available == true`, `hint.is_none()`
  - `scan_json_ui_specs_no_views_dir` — missing `src/views/` -> `available == false`, `view_count == 0`, `hint.is_some()`
  - `scan_json_ui_specs_empty_views_dir` — `src/views/` exists but empty -> `available == true`, `view_count == 0`, `hint.is_some()`
  - `scan_json_ui_specs_ignores_non_json_files` — `mod.rs` + `legacy.rs` + `real.json` -> `view_count == 1`
- Preserved `JsonUiSpecsStatus` struct (lines 57-62) — same four field names, same types, MCP wire contract intact.
- Preserved `views_dir_display = "src/views/"` project-relative literal — never emit absolute path (T-160-05 mitigation).

## TDD Gate Compliance

- **RED:** `4971010d test(160-03): add failing test for scan_json_ui_specs v2 json counting` — `scan_json_ui_specs_counts_json_files` fails against the v1 `.rs` scanner with `assertion left == right failed: left: 0, right: 2`. The other 3 tests coincidentally pass against the v1 impl (no `.rs` in fixtures or, in the non-json-ignore case, only one `.rs` file ≠ `mod.rs` matches the v1 count of 1).
- **GREEN:** `7768e8d4 feat(160-03): rewrite scan_json_ui_specs to count v2 JSON spec files` — all 4 tests pass.
- **REFACTOR:** not needed — paste-ready body already minimal; no cleanup pass produced.

## Task Commits

Each TDD phase committed atomically:

1. **RED — Test before implementation** — `4971010d` (test)
2. **GREEN — Rewrite scanner body** — `7768e8d4` (feat)

## Files Created/Modified
- `ferro-mcp/src/tools/application_info.rs` — function body replaced (-10/+10 net) + test module appended (+69)

## Decisions Made
- **Verbatim paste-ready body from RESEARCH Pattern 2** — no deviation from planned implementation; the body uses `Path::extension().is_some_and(...)` (Rust 1.70+ idiom, clippy-clean) rather than the older `.map(...).unwrap_or(false)` pattern that appeared in the v1 scanner.
- **Field-shape preservation enforced manually** — `JsonUiSpecsStatus` struct definition (lines 57-62) was not touched; only the producer function changes.
- **`views_dir` stays project-relative** — `views_dir_display = "src/views/"` literal kept per T-160-05 mitigation in the plan's `<threat_model>`. No `.display()` call introduced anywhere in the new body.
- **Hint copy refreshed to v2 vocabulary** — both hints now reference `JsonUi::render_file` and the `json_ui_generate` MCP tool; this is observable in the MCP wire output but the field shape is unchanged.

## Deviations from Plan

None - plan executed exactly as written. RESEARCH Pattern 2 paste-ready body used verbatim; test bodies match plan's example shape.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Threat Surface Scan

No new threat-relevant surface introduced. The function reads the project-local filesystem read-only via `Path::join`; same trust boundary as the v1 scanner (T-160-06 accept). T-160-05 (Information Disclosure of absolute path via `views_dir`) explicitly mitigated by preserving the `"src/views/"` literal.

## Next Phase Readiness
- Plan 04 (next site in the ferro-mcp/json-ui v1 surface-reduction sweep) can proceed; no dependency on this plan's output beyond the SUMMARY/STATE updates.
- The `scan_json_ui_specs` site is now off the Phase 160 hit-list (D-05 closed); CONTEXT.md `<key_decisions>` D-05 fully satisfied.
- gestiscilo (running `ferro = { path = "../ferro" }`) will see the updated MCP output on the next `application_info` call; field shape unchanged, only `view_count` semantics change — no consumer-side migration required.

## Self-Check: PASSED

- File exists: `ferro-mcp/src/tools/application_info.rs` — FOUND
- Commit exists: `4971010d` (RED) — FOUND in `git log`
- Commit exists: `7768e8d4` (GREEN) — FOUND in `git log`
- Acceptance gate: `grep -c 'Scans for legacy v1 patterns' ferro-mcp/src/tools/application_info.rs` returns 0 — PASS
- Acceptance gate: `grep -c 'TODO(Phase 120)' ferro-mcp/src/tools/application_info.rs` returns 0 — PASS
- Acceptance gate: `grep -c 'legacy v1 patterns' ferro-mcp/src/tools/application_info.rs` returns 0 — PASS
- Acceptance gate: `grep -q 'Counts JSON-UI spec files' ferro-mcp/src/tools/application_info.rs` succeeds — PASS
- `cargo fmt --all -- --check` exits 0 — PASS
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings` exits 0 — PASS
- `cargo test -p ferro-mcp --all-features --lib application_info` exits 0 — PASS (4 `scan_json_ui_specs_*` tests pass, 0 fail)

---
*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Completed: 2026-05-17*
