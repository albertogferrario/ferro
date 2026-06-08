---
phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands
fixed_at: 2026-06-08T21:36:30Z
review_path: .planning/phases/171-ferro-ai-make-ferro-ai-explain-cli-commands/171-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 171: Code Review Fix Report

**Fixed at:** 2026-06-08T21:36:30Z
**Source review:** `.planning/phases/171-ferro-ai-make-ferro-ai-explain-cli-commands/171-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (WR-01, WR-02, WR-03, IN-01, IN-03; IN-02 skipped per instructions)
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: Function name in generated source uses raw LLM-controlled service name

**Files modified:** `ferro-cli/src/commands/ai_make.rs`
**Commit:** `8a209714`
**Applied fix:** `emit_service_def_source` now derives `fn_name` from `crate::naming::to_snake_case(name)` instead of using `name` directly. Added test `emitter_pascal_case_name_produces_snake_case_function` that verifies `ServiceDef::new("OrderItem")` produces `pub fn order_item_service()` (not `OrderItem_service`). The `ServiceDef::new(...)` call in the builder body still uses the original name (runtime identity), only the Rust function identifier is snake-cased.

### WR-02: Module-local ENV_LOCK instances do not serialize env-var tests across modules

**Files modified:** `ferro-cli/src/commands/mod.rs`, `ferro-cli/src/commands/ai_explain.rs` (also in WR-02 commit: `ai_explain.rs` WR-03 doc fix landed here)
**Commit:** `03024580`
**Applied fix:** Added a single `pub(crate) static ENV_LOCK: std::sync::Mutex<()>` to `ferro-cli/src/commands/mod.rs` following the existing `CWD_TEST_LOCK` pattern. Removed the module-local `static ENV_LOCK` from both `ai_make.rs` tests and `ai_explain.rs` tests. Both test modules now `use crate::commands::ENV_LOCK` to reference the shared instance, serializing all env-var-mutating tests across both modules.

### WR-03: Misleading doc comment in ai_explain::run claims dry-run validates AI config

**Files modified:** `ferro-cli/src/commands/ai_explain.rs` (landed in WR-02 commit — same file)
**Commit:** `03024580`
**Applied fix:** Replaced the false comment "Even in dry-run we validate config to surface missing env vars early, but we skip the actual LLM call" with an accurate description: "In dry-run mode, AI config is NOT checked — the assembled prompt is printed and the function returns without calling the LLM or requiring any env vars to be set." Code behavior unchanged.

### IN-01: Prompt injection — `</description>` in user input closes the tag early

**Files modified:** `ferro-cli/src/commands/ai_make.rs` (landed in WR-01 commit — same file)
**Commit:** `8a209714`
**Applied fix:** Extracted a testable `pub(crate) fn sanitize_description(description: &str) -> String` helper that replaces `</description>` with `[/description]` and `<description>` with `[description]`. The `run()` function now calls `sanitize_description(&description)` before embedding the input in the prompt. Added three unit tests: `sanitize_description_strips_closing_tag`, `sanitize_description_strips_opening_tag`, and `sanitize_description_passthrough_clean_input`.

### IN-03: make_projection.rs FieldMeaning::Custom uses unescaped `{}` formatting

**Files modified:** `ferro-cli/src/commands/make_projection.rs`
**Commit:** `6db83412`
**Applied fix:** Changed `format!("FieldMeaning::Custom(\"{}\".into())", field.name)` to `format!("FieldMeaning::Custom({:?}.into())", field.name)` in `model_aware_template`, aligning with the `{:?}` debug-escaping pattern already used in `ai_make.rs`'s `emit_field_meaning`.

## Skipped Issues

### IN-02: to_snake_case does not handle hyphens

**File:** `ferro-cli/src/naming.rs:23-36`
**Reason:** Skipped per instructions — lowest-value finding, current behavior (rejection via `is_valid_identifier`) is safe, and this fix was explicitly deferred unless it fell out trivially from WR-01 work.
**Original issue:** `to_snake_case` passes hyphens through unchanged; `"order-item"` is correctly rejected by `is_valid_identifier` but the error message may be unexpected to users.

---

## Gate Results

All three validation gates passed after fixes:

1. `cargo fmt --all -- --check` — pass (one fmt drift fixed in new test code)
2. `cargo clippy -p ferro-cli -p ferro-ai --all-targets -- -D warnings` — pass (0 warnings)
3. `cargo test -p ferro-cli -p ferro-ai --lib` — pass (95 ferro-ai + 554 ferro-cli = 649 tests, 0 failed)

---

_Fixed: 2026-06-08T21:36:30Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
