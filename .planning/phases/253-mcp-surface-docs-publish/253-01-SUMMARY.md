---
phase: 253-mcp-surface-docs-publish
plan: "01"
subsystem: ferro-mcp
tags: [design-lint, mcp-tool, ferro-json-ui, ds-07]
dependency_graph:
  requires: [252-design-lint-cli]
  provides: [design_lint-mcp-tool]
  affects: [ferro-mcp, ferro-json-ui]
tech_stack:
  added: []
  patterns: [tool-execute-pattern, xor-input-guard, warning-as-finding]
key_files:
  created:
    - ferro-mcp/src/tools/design_lint.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs
decisions:
  - "XOR-input violations and parse failures returned as Warning-level FileFinding, never as MCP tool errors (D-04)"
  - "FileFinding struct replicates CLI contract verbatim (D-02) — same shape as ferro design:lint --json"
  - "lint_string silently skips non-ferro JSON (no ferro-json-ui/v2 marker) matching CLI WalkDir behaviour"
  - "CLEAN spec in tests uses layout=auth + allow=[page-header] to produce zero findings from the design engine"
metrics:
  duration_seconds: 420
  completed_date: "2026-07-04T01:04:41Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 2
---

# Phase 253 Plan 01: design_lint MCP Tool Summary

**One-liner:** `design_lint` MCP tool wiring `ferro_json_ui::design::lint` inline/path XOR into the same `FileFinding[]` envelope as the CLI `--json` flag.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create the design_lint tool module (TDD) | 43f3c57b | ferro-mcp/src/tools/design_lint.rs, ferro-mcp/src/tools/mod.rs |
| 2 | Register the design_lint MCP tool + fix stale component count | 1524432d | ferro-mcp/src/service.rs |

## What Was Built

### Task 1 — `ferro-mcp/src/tools/design_lint.rs`

New tool module following the `json_ui_validate_spec` analog exactly:

- `FileFinding { file: String, #[serde(flatten)] finding: Finding }` — identical to `ferro_cli::commands::design_lint::FileFinding` (D-02 contract).
- `pub fn execute(spec_json: Option<&str>, path: Option<&str>) -> Vec<FileFinding>` — handles three cases: inline (Some, None), path (None, Some), XOR violation (anything else) — each returns a well-formed `FileFinding[]`, never panics, never returns a tool error (D-04).
- `fn lint_string(label, content)` — skips non-ferro JSON silently; maps `Spec::from_json` errors to a `spec-parse` Warning finding; maps `lint(&spec)` results to `FileFinding` with the label as `file`.
- 5 unit tests: `inline_clean_spec_returns_empty`, `inline_malformed_returns_spec_parse_warning`, `path_mode_reads_and_lints`, `both_none_returns_tool_input_warning`, `both_some_returns_tool_input_warning`.
- Module registered alphabetically in `tools/mod.rs` between `deploy_check` and `diagnose_error`.

### Task 2 — `ferro-mcp/src/service.rs`

- `DesignLintParams { spec_json: Option<String>, path: Option<String> }` with the standard derive set (Debug, Clone, Deserialize, Serialize, JsonSchema).
- `#[tool(name = "design_lint", description = ...)] pub async fn design_lint(...)` placed immediately after `json_ui_validate_spec`, routes to `tools::design_lint::execute(spec_json.as_deref(), path.as_deref())`.
- Fixed stale component count in `json_ui_catalog`: doc comment and description string both updated from "39 built-in" to "47 built-in".

## Verification

- `cargo test -p ferro-mcp design_lint`: 5 passed, 0 failed.
- `cargo test -p ferro-mcp --lib`: 308 passed, 0 failed.
- `cargo build -p ferro-mcp`: exit 0, clean.
- `grep -c "39 built-in" ferro-mcp/src/service.rs`: 0.
- `grep -c "47 built-in" ferro-mcp/src/service.rs`: 2.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The tool is fully wired to the live `ferro_json_ui::design::lint` engine.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries beyond what the threat model in the plan already covers (T-253-01 through T-253-03).

## Self-Check: PASSED

- `ferro-mcp/src/tools/design_lint.rs`: exists (created in Task 1)
- `ferro-mcp/src/tools/mod.rs`: `pub mod design_lint;` present
- `ferro-mcp/src/service.rs`: `DesignLintParams`, `design_lint` tool method, 47 built-in (×2) present
- Commits 43f3c57b and 1524432d: confirmed in git log
