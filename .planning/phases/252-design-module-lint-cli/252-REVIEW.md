---
phase: 252-design-module-lint-cli
reviewed: 2026-07-03T00:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - app/Cargo.toml
  - app/src/tests/design_lint.rs
  - app/src/tests/mod.rs
  - app/src/views/login.json
  - app/src/views/login_confirm.json
  - app/src/views/pagamenti.json
  - ferro-cli/src/commands/design_lint.rs
  - ferro-cli/src/commands/mod.rs
  - ferro-cli/src/main.rs
  - ferro-json-ui/src/action.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/design/infer.rs
  - ferro-json-ui/src/design/mod.rs
  - ferro-json-ui/src/design/rules.rs
  - ferro-json-ui/src/design/types.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/spec.rs
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 252: Code Review Report

**Reviewed:** 2026-07-03
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

The design-lint module is well-structured. The rule engine is pure (no I/O, no panics on bad input), the type contracts are clean, the CLI wiring is correct, the app views all carry explicit `design.intent` declarations, and the D-17 gate test is sound. The `ConfirmDialog.unknown_fields` Stage 2b mechanism is an elegant approach to capturing retired prop names through serde.

Two rule implementations share the same class of logic error: they accept only string literals or arrays for props that are also bindable via `$data` expressions, producing false-positive warnings when those props carry a data-binding object. A third issue is that file I/O errors during the walker are silently discarded rather than surfaced as findings, creating a potential CI false-negative. None of these affect the current app views (which use string literals throughout), but they are correctness issues the rule engine should handle.

## Warnings

### WR-01: `check_list_empty_state` produces false positive for `$data`-bound `empty_message`

**File:** `ferro-json-ui/src/design/rules.rs:144-150`

**Issue:** The rule checks whether a `DataTable` or `MediaCardGrid` has an empty state configured by testing `el.props.get("empty_message").and_then(|v| v.as_str()).is_some()`. The `.and_then(|v| v.as_str())` call returns `None` when `empty_message` is a `$data` binding object (e.g., `{"$data": "/i18n/no_items"}`), making `has_empty_message = false`. If no `EmptyState` element is present, a spurious `list-empty-state` Warning fires and `ferro design:lint --deny` fails CI even though the empty state is properly configured.

**Fix:**

```rust
// Before (accepts string literals only):
let has_empty_message = el
    .props
    .get("empty_message")
    .and_then(|v| v.as_str())
    .is_some();

// After (accepts any non-null value, including $data bindings):
let has_empty_message = el
    .props
    .get("empty_message")
    .map(|v| !v.is_null())
    .unwrap_or(false);
```

Note: `check_page_header` already uses the permissive `map(|v| v.is_null()).unwrap_or(true)` pattern for its `title` check. Apply the same idiom here for consistency.

---

### WR-02: `check_breadcrumb_on_subpages` produces false positive for `$data`-bound `breadcrumb` prop

**File:** `ferro-json-ui/src/design/rules.rs:204-210`

**Issue:** Same class of error as WR-01. The conformance check for a PageHeader with a breadcrumb uses `.and_then(|v| v.as_array()).map(|a| !a.is_empty()).unwrap_or(false)`. A `$data`-bound breadcrumb (e.g., `{"$data": "/breadcrumb_items"}`) is an object, not an array. `as_array()` returns `None`, so the check evaluates to `false`, and the rule fires on a page that has breadcrumb navigation configured via data binding.

**Fix:**

```rust
// Before (accepts non-empty array only):
let has_breadcrumb_in_header = spec.elements.values().any(|e| {
    e.type_name == "PageHeader"
        && e.props
            .get("breadcrumb")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false)
});

// After (accepts any non-null value including $data bindings):
let has_breadcrumb_in_header = spec.elements.values().any(|e| {
    e.type_name == "PageHeader"
        && e.props
            .get("breadcrumb")
            .map(|v| !v.is_null())
            .unwrap_or(false)
});
```

---

### WR-03: File I/O errors during the walker are silently swallowed, producing a CI false-negative

**File:** `ferro-cli/src/commands/design_lint.rs:103-106`

**Issue:** When `std::fs::read_to_string` fails (e.g., a permission error), the walker silently skips the file with `Err(_) => continue`. Under `--deny`, the lint command reports clean even though a spec file was not linted. In a CI environment where files might be inaccessible due to mount or permission issues, this converts an infrastructure problem into a silent false-negative.

**Fix:**

```rust
let content = match std::fs::read_to_string(file_path) {
    Ok(c) => c,
    Err(e) => {
        all.push(FileFinding {
            file: label.clone(),
            finding: Finding {
                rule: "file-read",
                element_id: None,
                severity: Severity::Warning,
                message: format!("Could not read file: {e}"),
                suggestion: "Check file permissions.".into(),
            },
        });
        continue;
    }
};
```

## Info

### IN-01: `FIELD_TYPES` includes `"Textarea"` which has no registered builtin component

**File:** `ferro-json-ui/src/design/rules.rs:299`

**Issue:** `const FIELD_TYPES: &[&str] = &["Input", "Select", "Textarea", "RichTextEditor"]`. Scanning `catalog.rs`'s `BUILTIN_SPECS` and `lib.rs`'s re-exports shows no `TextareaProps` and no `"Textarea"` entry in the builtin table. `RichTextEditor` is registered as a plugin, so that entry is valid. `"Textarea"` is unreachable in practice: catalog validation rejects specs with unknown element types before design lint runs. The entry is dead code in the constant. Either register `Textarea` as a component or remove it.

**Fix:** Remove `"Textarea"` from `FIELD_TYPES` until the component exists, or add a comment indicating it is planned.

---

### IN-02: `print_human` outputs "No findings — all specs are clean" even when no files were linted

**File:** `ferro-cli/src/commands/design_lint.rs:136-142`

**Issue:** `files_seen` is populated from findings, not from files visited. When the target path contains no `.json` files (or none matching the `ferro-json-ui/v2` marker), `all` is empty, `files_seen` is empty, and the human output reads "No findings — all specs are clean." This is misleading: it implies specs were reviewed and found clean, when in fact nothing was linted (e.g., the user pointed the command at the wrong directory).

**Fix:** Track a `files_visited: u32` counter in the walker loop and emit a distinct message when zero files were processed:

```rust
if files_visited == 0 {
    println!("{}", style("No JSON-UI spec files found.").yellow());
    return;
}
if files_seen.is_empty() {
    println!("{}", style("No findings — all specs are clean.").green().bold());
    return;
}
```

---

### IN-03: `allow: ["allow"]` self-suppresses its own unknown-id warning

**File:** `ferro-json-ui/src/design/mod.rs:108-134`

**Issue:** Unknown allow ids emit a `Finding` with `rule: "allow"`. Step 4 suppresses findings by rule id using the `allow` list. If a spec declares `allow: ["allow"]`, the finding for the unrecognised id `"allow"` is produced in Step 2, then removed in Step 4 because `"allow"` is in the allow list. The net effect is that `allow: ["typo-in-rule-id", "allow"]` silently swallows the warning about `"typo-in-rule-id"` by pairing it with `"allow"`. The workaround requires knowing the internal rule id `"allow"`, so the practical risk is low, but the behaviour is unintuitive.

**Fix:** Apply allow-suppression only to findings emitted by the rule registry, not to engine-level findings (`"allow"` and `"declare-intent"`):

```rust
// Step 4: suppress allow-listed findings, but never suppress engine findings.
findings.retain(|f| {
    let is_engine = f.rule == "allow" || f.rule == "declare-intent";
    is_engine || !allow.iter().any(|a| a == f.rule)
});
```

---

### IN-04: `SpecBuilder::build` hardcodes `design: None` with no setter

**File:** `ferro-json-ui/src/spec.rs:447-458`

**Issue:** `SpecBuilder` has no `.design()` method, so `Spec` values constructed programmatically (e.g., by `projection::JsonUiRenderer`) always have `design: None`. The D-17 lint gate test and all app views are JSON-authored, so this has no current impact. When Phase 253 extends the projection renderer to emit design metadata, the builder will need a `.design(meta: DesignMeta) -> Self` method to keep the API usable from Rust code.

**Fix (deferred to Phase 253):** Add a builder method:

```rust
pub fn design(mut self, meta: DesignMeta) -> Self {
    // SpecBuilder needs a `design: Option<DesignMeta>` field
    self.design = Some(meta);
    self
}
```

---

_Reviewed: 2026-07-03_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
