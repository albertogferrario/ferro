---
phase: 252-design-module-lint-cli
fixed_at: 2026-07-03T00:00:00Z
review_path: .planning/phases/252-design-module-lint-cli/252-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 252: Code Review Fix Report

**Fixed at:** 2026-07-03
**Source review:** .planning/phases/252-design-module-lint-cli/252-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-01, WR-02, WR-03; IN-* findings excluded by fix_scope)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: `check_list_empty_state` produces false positive for `$data`-bound `empty_message`

**Files modified:** `ferro-json-ui/src/design/rules.rs`
**Commit:** 0c7fada3 (batched with WR-02 — same file, same fix pattern)
**Applied fix:** Changed `.and_then(|v| v.as_str()).is_some()` to `.map(|v| !v.is_null()).unwrap_or(false)` in `check_list_empty_state`, matching the permissive pattern already used by `check_page_header`. Added regression test `list_empty_state_conforming_data_bound_empty_message` that passes a `{"$data": "/i18n/no_items"}` binding and asserts zero findings.

---

### WR-02: `check_breadcrumb_on_subpages` produces false positive for `$data`-bound `breadcrumb` prop

**Files modified:** `ferro-json-ui/src/design/rules.rs`
**Commit:** 0c7fada3 (batched with WR-01 — same file, same fix pattern)
**Applied fix:** Changed `.and_then(|v| v.as_array()).map(|a| !a.is_empty()).unwrap_or(false)` to `.map(|v| !v.is_null()).unwrap_or(false)` in `check_breadcrumb_on_subpages`. Added regression test `breadcrumb_on_subpages_conforming_data_bound_breadcrumb_prop` that passes a `{"$data": "/breadcrumb_items"}` binding and asserts zero findings.

Note: WR-01 and WR-02 were committed in a single atomic commit because both fixes are in `ferro-json-ui/src/design/rules.rs` and selective hunk staging is not available in the non-interactive environment. The two fixes are independent (different functions) and have separate regression tests.

---

### WR-03: File I/O errors during the walker are silently swallowed, producing a CI false-negative

**Files modified:** `ferro-cli/src/commands/design_lint.rs`
**Commit:** f3b71901
**Applied fix:** Moved `let label = file_path.display().to_string()` before the `read_to_string` match so it is available in both the error and success paths. Changed `Err(_) => continue` to push a `FileFinding` with `rule: "file-read"`, `severity: Severity::Warning`, and a message containing the OS error. Added regression test `has_warning_true_for_file_read_finding` that constructs a `file-read` FileFinding directly and confirms `has_warning` returns true, verifying `--deny` will trip on I/O errors.

---

_Fixed: 2026-07-03_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
