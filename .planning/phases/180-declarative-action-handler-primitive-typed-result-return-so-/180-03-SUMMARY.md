---
phase: 180
plan: 03
subsystem: ferro-macros
tags: [proc-macro, action-handler, attribute-macro, redirect, flash]

requires:
  - phase: 180-01
    provides: ActionResult, ActionError, ActionOk, handle_action_result runtime helper in framework/src/http/action.rs
  - phase: 180-02
    provides: pub(crate) param-extraction helpers in ferro-macros/src/utils.rs

provides:
  - ferro-macros/src/action.rs — action_impl + parse_action_attrs + ActionAttrs
  - ferro-macros: #[proc_macro_attribute] pub fn action(attr, input) registered in lib.rs
  - framework crate root: pub use ferro_macros::action re-export (consumers write use ferro::action)

affects:
  - phase 180-04 (trybuild integration tests — consumes #[action] surface)
  - phase 180-05 (docs — references the #[action] attribute shape)
  - phase 180-06 (consumer sweep — mechanical sed of POST handlers to #[action])

tech-stack:
  added: []
  patterns:
    - parse_action_attrs uses proc_macro2::TokenStream + syn::parse::Parser::parse2 for unit-testability outside proc-macro context
    - ActionAttrs carries #[allow(dead_code)] on method field — part of public attribute surface but not yet consumed at runtime (D-05)
    - #[derive(Debug)] on ActionAttrs required for unwrap_err() in unit tests
    - action_impl converts proc_macro::TokenStream to proc_macro2::TokenStream at entry point boundary
    - Handler name built with concat!(module_path!(), "::", stringify!(fn_name)) — &'static str, zero runtime allocation (D-07)

key-files:
  created:
    - ferro-macros/src/action.rs
  modified:
    - ferro-macros/src/lib.rs (mod action; added at line 13; pub fn action proc_macro_attribute at lines 231-266)
    - framework/src/lib.rs (pub use ferro_macros::action at line 311)
    - Cargo.lock (version bump propagation)

key-decisions:
  - "parse_action_attrs takes proc_macro2::TokenStream (not proc_macro::TokenStream) — required to make unit tests work outside proc-macro compilation context; action_impl converts at the entry point boundary"
  - "ActionAttrs derives Debug — required because unwrap_err() bounds on the Ok type; #[allow(dead_code)] on method field kept as planned (D-05 surface reservation)"
  - "handle_action_result reference in module doc comment removed — plan criterion requires grep -c returns 1; one reference in generated quote! block is the correct count"
  - "Proc-macro registration placed before handler in lib.rs to maintain alphabetical order of proc_macro_attribute entries"

requirements-completed:
  - D-01
  - D-04
  - D-05
  - D-07

duration: 8 min
completed: 2026-05-30
---

# Phase 180 Plan 03: `#[action]` proc-macro implementation + framework re-export Summary

**`#[action(redirect_to = "...", method = "POST")]` proc-macro that wraps ActionResult-returning handlers in the Plan 01 runtime dispatcher, using Plan 02 param-extraction helpers, registered in ferro-macros and re-exported from the ferro crate root.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-05-30T00:14:29Z
- **Completed:** 2026-05-30T00:22:17Z
- **Tasks:** 2
- **Files modified:** 4 (ferro-macros/src/action.rs created, ferro-macros/src/lib.rs, framework/src/lib.rs, Cargo.lock)

## Accomplishments

- `ferro-macros/src/action.rs`: `parse_action_attrs`, `ActionAttrs`, `expect_str_lit`, `action_impl` — attribute parser with compile errors for missing `redirect_to`, unknown keys, and self receivers; 6 unit tests all passing
- `ferro-macros/src/lib.rs`: `mod action;` registered alphabetically first; `pub fn action` proc_macro_attribute with full rustdoc including security note (T-180-01 cross-reference)
- `framework/src/lib.rs`: `pub use ferro_macros::action;` added to existing proc-macro re-export block — consumers write `use ferro::action;`
- Full workspace `cargo build --all-features`, `cargo clippy --all --all-targets -D warnings`, and `cargo test --all-features --all-targets` all exit 0
- Zero `/accedi` literals in modified files (CLAUDE.md project-agnostic rule)

## Task Commits

1. **Task 1: Create ferro-macros/src/action.rs** — `e78f78d4` (feat)
2. **Task 2: Register #[action] in lib.rs + re-export from framework** — `722e6c69` (feat)

## Files Created/Modified

- `ferro-macros/src/action.rs` — New: attribute parser (`parse_action_attrs`, `ActionAttrs`), `action_impl`, 6 unit tests
- `ferro-macros/src/lib.rs` — Modified: `mod action;` at line 13; `pub fn action` proc_macro_attribute at lines 231-266
- `framework/src/lib.rs` — Modified: `pub use ferro_macros::action;` at line 311 (in existing proc-macro re-export block, before `domain_error`)
- `Cargo.lock` — Modified: version bump propagation

## proc-macro re-export block in framework/src/lib.rs (final form)

```rust
// Re-export the proc-macros for compile-time component validation and type safety
pub use ferro_macros::action;
pub use ferro_macros::domain_error;
pub use ferro_macros::ferro_test;
pub use ferro_macros::handler;
pub use ferro_macros::inertia_response;
pub use ferro_macros::injectable;
pub use ferro_macros::redirect;
pub use ferro_macros::request;
pub use ferro_macros::service;
pub use ferro_macros::ApiResource;
pub use ferro_macros::FerroModel;
pub use ferro_macros::FormRequest as FormRequestDerive;
pub use ferro_macros::InertiaProps;
pub use ferro_macros::ValidateRules;
```

## Decisions Made

- `parse_action_attrs` was written to accept `proc_macro2::TokenStream` instead of `proc_macro::TokenStream` so unit tests can run outside the proc-macro compilation environment. `action_impl` converts at the entry point with `TokenStream2::from(attr)`.
- `ActionAttrs` needed `#[derive(Debug)]` because `unwrap_err()` requires `Debug` on the `Ok` variant; this was added as a Rule 1 fix.
- The module doc comment was revised to not include `handle_action_result` by name, so the acceptance criterion `grep -c returns 1` is satisfied exactly (one occurrence in the generated `quote!` block).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `ActionAttrs` missing `#[derive(Debug)]` for unit test compatibility**
- **Found during:** Task 1 (running unit tests)
- **Issue:** `unwrap_err()` requires `Debug` on the `Ok` type (`ActionAttrs`); test compilation failed with E0277
- **Fix:** Added `#[derive(Debug)]` to `ActionAttrs`
- **Files modified:** ferro-macros/src/action.rs
- **Verification:** All 6 unit tests compile and pass
- **Committed in:** e78f78d4

**2. [Rule 1 - Bug] `parse_action_attrs` used `proc_macro::TokenStream` causing panic in unit tests**
- **Found during:** Task 1 (running `cargo test -p ferro-macros --lib action`)
- **Issue:** `proc_macro::TokenStream::parse()` panics with "procedural macro API is used outside of a procedural macro" in unit test context
- **Fix:** Changed `parse_action_attrs` to accept `proc_macro2::TokenStream` and use `syn::parse::Parser::parse2`; `action_impl` converts at the entry point with `TokenStream2::from(attr)`
- **Files modified:** ferro-macros/src/action.rs
- **Verification:** All 6 unit tests pass; full build and clippy clean
- **Committed in:** e78f78d4

---

**Total deviations:** 2 auto-fixed (2 bugs caught by test execution)
**Impact on plan:** Both fixes required for correct test execution. The proc_macro2::TokenStream pattern is the standard approach for testable proc-macro parsers. No scope creep.

## Issues Encountered

None beyond the two auto-fixed deviations above.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Wave 2 complete. `#[action]` is defined, registered, and re-exported.
- Wave 3 (Plans 04, 05, 06) can proceed: trybuild integration tests (04), docs (05), consumer sweep (06).
- The macro surface matches the D-04/D-05/D-07 contracts exactly. Plan 04's trybuild fixtures can exercise it directly.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. The macro itself emits a call to Plan 01's `handle_action_result` which carries the security mitigations (T-180-02 open redirect gate, T-180-03 log sanitization). The rustdoc on `pub fn action` in `ferro-macros/src/lib.rs` includes a `# Security` paragraph cross-referencing T-180-01 (flash XSS obligation on consumer templates).

---
*Phase: 180-declarative-action-handler-primitive-typed-result-return-so-*
*Completed: 2026-05-30*

## Self-Check: PASSED

- `ferro-macros/src/action.rs`: FOUND
- Commit `e78f78d4`: FOUND
- Commit `722e6c69`: FOUND
- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --all --all-targets -- -D warnings`: exit 0
- `cargo build --all-features`: exit 0
- `cargo test --all-features --all-targets`: all pass, zero failures
