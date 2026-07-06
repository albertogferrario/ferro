---
phase: 253-mcp-surface-docs-publish
plan: "04"
subsystem: ferro-json-ui / ferro-cli
tags: [design-lint, cleanup, pre-publish]
dependency_graph:
  requires: [253-03]
  provides: [DS-08]
  affects: [ferro-json-ui/src/design/rules.rs, ferro-cli/src/commands/design_lint.rs]
tech_stack:
  added: []
  patterns: [static-const-trim, counter-gated-branch]
key_files:
  modified:
    - ferro-json-ui/src/design/rules.rs
    - ferro-cli/src/commands/design_lint.rs
decisions:
  - "IN-01: Textarea removed from FIELD_TYPES; RichTextEditor (plugin component) retained"
  - "IN-02: files_linted counter in run() gates the zero-files vs all-clean message split"
metrics:
  duration_seconds: 185
  completed_date: "2026-07-04"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 253 Plan 04: Pre-Publish Cleanup (IN-01 / IN-02) Summary

**One-liner:** Remove dead `"Textarea"` from `FIELD_TYPES` and distinguish zero-files-found from all-clean in `ferro design:lint` output, resolving both Phase 252 info-level deferred findings before the v16.5 publish.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | IN-01 — remove dead Textarea from FIELD_TYPES | `7c9786bf` | ferro-json-ui/src/design/rules.rs |
| 2 | IN-02 — distinguish no-files-found from all-clean | `caa8cb65` | ferro-cli/src/commands/design_lint.rs |

## Verification

- `cargo test -p ferro-json-ui design` — 47 passed, 0 failed (Task 1 gate)
- `cargo test -p ferro-cli design_lint` — 8 passed, 0 failed (Task 2 gate)

## Changes Made

### IN-01: `ferro-json-ui/src/design/rules.rs` line 298

`FIELD_TYPES` trimmed from `["Input", "Select", "Textarea", "RichTextEditor"]` to `["Input", "Select", "RichTextEditor"]`. `Textarea` has no registered builtin component — catalog validation would reject a spec with a `Textarea` element before the lint engine runs, so the entry was unreachable dead code. `RichTextEditor` (a plugin component) is retained so the `form-default-values` rule still applies to it.

The two call sites at lines 303 and 317 (`FIELD_TYPES.contains(&e.type_name.as_str())`) are unchanged — they consume the slice generically.

### IN-02: `ferro-cli/src/commands/design_lint.rs`

Added `let mut files_linted: usize = 0;` before the walker loop. Inside the loop, after reading file content and before calling `lint_content`, increments when `content.contains(SCHEMA_VERSION)` — counting only files that carried the ferro-json-ui/v2 schema marker and were actually linted.

Changed the `else { print_human(&all) }` branch to:
```rust
} else if all.is_empty() && files_linted == 0 {
    println!("{}", style("No JSON-UI spec files found.").yellow());
} else {
    print_human(&all);
}
```

`print_human`'s existing `"No findings — all specs are clean."` message is preserved for the `all.is_empty() && files_linted > 0` path (files were checked and are clean). The `--json` output path and `--deny` exit logic are unchanged.

## Deviations from Plan

None — plan executed exactly as written. The ENOSPC linker failure during Task 2 verification was a disk-full transient (project-known issue); resolved by removing stale test binaries from `target/debug/deps/` before retrying.

## Known Stubs

None. Both changes are complete fixes with no placeholder paths.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. Both changes are internal to static constants and CLI output branching.

## Self-Check

- [x] `ferro-json-ui/src/design/rules.rs` modified — verified by grep
- [x] `ferro-cli/src/commands/design_lint.rs` modified — verified by grep
- [x] Commit `7c9786bf` exists in git log
- [x] Commit `caa8cb65` exists in git log
- [x] Task 1: `grep -c Textarea rules.rs` = 0; `grep -c RichTextEditor rules.rs` = 1
- [x] Task 2: `files_linted` appears at lines 90, 122, 132; both messages present

## Self-Check: PASSED
