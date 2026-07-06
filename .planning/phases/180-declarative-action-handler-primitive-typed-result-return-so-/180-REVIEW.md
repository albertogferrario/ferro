---
phase: 180-declarative-action-handler-primitive-typed-result-return-so-
reviewed: 2026-05-30T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - framework/src/http/action.rs
  - framework/src/http/request.rs
  - framework/src/http/mod.rs
  - framework/src/lib.rs
  - framework/tests/action_handler.rs
  - framework/Cargo.toml
  - ferro-macros/src/action.rs
  - ferro-macros/src/handler.rs
  - ferro-macros/src/utils.rs
  - ferro-macros/src/lib.rs
  - ferro-macros/Cargo.toml
  - ferro-macros/tests/action_macro.rs
  - ferro-macros/tests/ui/action/pass/minimal.rs
  - ferro-macros/tests/ui/action/pass/question_mark_on_string.rs
  - ferro-macros/tests/ui/action/pass/question_mark_on_framework_error.rs
  - ferro-macros/tests/ui/action/pass/question_mark_on_db_err.rs
  - ferro-macros/tests/ui/action/pass/success_override.rs
  - ferro-macros/tests/ui/action/pass/error_override.rs
  - ferro-macros/tests/ui/action/fail/missing_redirect_to.rs
  - ferro-macros/tests/ui/action/fail/unknown_attr_key.rs
  - ferro-macros/tests/ui/action/fail/non_action_result_return.rs
  - ferro-mcp/src/tools/code_templates.rs
  - docs/src/SUMMARY.md
  - docs/src/the-basics/action-handlers.md
  - docs/src/the-basics/controllers.md
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 180: Code Review Report

**Reviewed:** 2026-05-30
**Depth:** standard
**Files Reviewed:** 18 (plus 6 trybuild fixtures and .stderr snapshots)
**Status:** issues_found

## Summary

Phase 180 delivers the `#[action]` macro and its runtime (`handle_action_result`) cleanly. All three threat-model claims (T-180-01, T-180-02, T-180-03) are implemented and verified. The D-02 revised contract (`ActionResult = Result<(), ActionError>`, no `ActionOk`, success-side overrides via setters) is correctly reflected in the public surface, the macro, the docs, and the MCP template. D-08 is fully satisfied: no `/accedi` literal in any `ferro-*` source, and `ActionError::unauthorized()` defaults `redirect_override` to `None`. The `handle_action_result` visibility is `pub #[doc(hidden)]` as designed; no `__test_handle_action_result` shim exists or is needed. Trybuild `.stderr` snapshots reference fixture-file line numbers and are stable across macro implementation changes.

Two warnings and two info items follow.

## Warnings

### WR-01: Back-compat query string appends `?` unconditionally — malformed URL when target already contains `?`

**File:** `framework/src/http/action.rs:302` (success path) and `:344-349` (error path)

**Issue:** Both code paths build the redirect URL by formatting `"{target}{suffix}"` where the suffix always begins with `?`. If `target` already contains a query string — which can happen when `req.redirect_to("/dashboard?tab=active")` or `ActionError::msg("x").redirect_to("/list?page=2")` is used — the resulting URL will have two `?` characters, e.g. `/dashboard?tab=active?success=1`. RFC 3986 allows only one `?` delimiter; browsers and servers parse the double-`?` inconsistently.

The `is_same_origin` check only validates that the URL starts with `/` and is not scheme-relative; it does not strip or reject existing query strings.

**Fix:** Use `&` instead of `?` when the target already contains a `?`:

```rust
// success path — replace line 302
let sep = if target.contains('?') { "&" } else { "?" };
let location = format!("{target}{sep}{}", &suffix[1..]); // strip leading '?'
```

Or more cleanly:

```rust
let qs_prefix = if target.contains('?') { "&" } else { "?" };
let suffix_body = match overrides.flash.as_deref() {
    Some(k) if !k.is_empty() => format!("success={k}"),
    _ => "success=1".to_string(),
};
let location = format!("{target}{qs_prefix}{suffix_body}");
```

Apply the same pattern to the error path (`format!("{target}?error=...")`).

---

### WR-02: Flash key inserted into back-compat query string without URL encoding

**File:** `framework/src/http/action.rs:299`

**Issue:** The success flash key is interpolated directly into the query string:

```rust
Some(k) if !k.is_empty() => format!("?success={k}"),
```

`k` is the value passed by the user to `req.flash(key)`. If `k` contains characters that are significant in query strings (`&`, `=`, `+`, `%`, space, etc.), the resulting URL will be malformed or will be parsed incorrectly by the consumer template. For example, `req.flash("order & product created")` produces `?success=order & product created`, which splits into two query parameters at the `&`.

In practice, flash keys are short identifiers (`"created"`, `"updated"`), so this is unlikely to trigger in real code. But it is a latent correctness bug and creates an inconsistency: the error-path message IS percent-encoded (via `byte_serialize`) while the success-path key is not.

**Fix:** Encode the flash key in the same way as the error message:

```rust
let encoded_key: String = byte_serialize(k.as_bytes()).collect();
format!("?success={encoded_key}")
```

## Info

### IN-01: Stale `#[allow(dead_code)]` on `action_overrides()`

**File:** `framework/src/http/request.rs:573`

**Issue:** The `action_overrides()` method carries `#[allow(dead_code)]` with a now-outdated comment (`"Called from handle_action_result in action.rs; no external call site exists until Plan 03"`). Plan 03 has shipped: `handle_action_result` in `action.rs` calls `req.action_overrides()` at line 273. The suppressor is no longer needed and its stale comment misstates the current state of the code.

**Fix:** Remove the `#[allow(dead_code)]` attribute and update or drop the comment:

```rust
/// Internal — read by the `#[action]` macro runtime to apply recorded overrides.
pub(crate) fn action_overrides(&self) -> &crate::http::action::ActionOverrides {
    &self.action_overrides
}
```

---

### IN-02: `handle_action_result` is reachable at `ferro::http::action::handle_action_result` but not re-exported at crate root

**File:** `framework/src/lib.rs` (absence of re-export)

**Issue:** `handle_action_result` is `pub #[doc(hidden)]` in `ferro::http::action`. The integration test imports it as `ferro::http::action::handle_action_result` (working correctly because `http` is `pub mod` and `action` is `pub mod`). The macro-generated code calls it as `::ferro::http::action::handle_action_result`. This is correct and intentional. However, the crate-root `pub use http::action::{...}` block in `http/mod.rs` (line 13-15) exports the user-facing types but deliberately omits `handle_action_result`.

No action required — the current approach is correct. This note documents the deliberate omission for future reviewers: `handle_action_result` must remain accessible via the module path but must not appear in the flat `ferro::*` re-export surface to avoid polluting the public API.

---

_Reviewed: 2026-05-30_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
