---
phase: 120-cli-and-mcp-updates
fixed_at: 2026-04-21T17:10:00Z
review_path: .planning/phases/120-cli-and-mcp-updates/120-REVIEW.md
iteration: 1
fix_scope: critical_warning
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 120: Code Review Fix Report

**Fixed at:** 2026-04-21
**Source review:** .planning/phases/120-cli-and-mcp-updates/120-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: Char index used as byte index in struct-body slicing

**Files modified:** `ferro-cli/src/ai.rs`, `ferro-mcp/src/tools/json_ui_generate.rs`
**Commit:** b90f25f3
**Applied fix:** Changed `rest.chars().enumerate()` to `rest.char_indices()` and renamed the
index variable from `i` to `byte_idx` in both `scan_models` implementations. This ensures the
index used to slice `&rest[..struct_end]` is always a byte offset, preventing a panic on
source files containing multi-byte Unicode characters (e.g. accented letters in doc-comments).

---

### WR-02: Dead `_schema` parameter in `build_json_view_pass2`

**Files modified:** `ferro-cli/src/ai.rs`, `ferro-cli/src/commands/make_json_view.rs`
**Commit:** d7fb8699
**Applied fix:** Removed the `_schema: &serde_json::Value` parameter from
`build_json_view_pass2`. Updated the call site in `make_json_view.rs` to call
`build_json_view_pass2(&pass1_result)` without arguments, and moved the
`global_catalog().json_schema().clone()` call to just before `call_anthropic_structured`
where the schema value is actually used. Eliminates the misleading API and the redundant clone.

---

### WR-03: `inspect_component` classifies unknown components as `is_plugin: true`

**Files modified:** `ferro-mcp/src/tools/json_ui_inspect.rs`
**Commit:** e901f5e2
**Applied fix:** Restructured the plugin-registry fallback branch to check whether a plugin
entry was actually found before returning `is_plugin: true`. When neither a built-in nor a
plugin entry exists, the function now returns `is_plugin: false` (was incorrectly `true`).
Updated `test_inspect_unknown_component` to assert `!info.is_plugin` instead of
`info.is_plugin`.

---

_Fixed: 2026-04-21_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
