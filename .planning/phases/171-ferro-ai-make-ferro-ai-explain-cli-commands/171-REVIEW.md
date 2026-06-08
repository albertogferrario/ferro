---
phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - ferro-ai/src/complete.rs
  - ferro-ai/src/lib.rs
  - ferro-cli/src/commands/ai_explain.rs
  - ferro-cli/src/commands/ai_make.rs
  - ferro-cli/src/commands/make_projection.rs
  - ferro-cli/src/commands/mod.rs
  - ferro-cli/src/lib.rs
  - ferro-cli/src/main.rs
  - ferro-cli/src/naming.rs
  - ferro-cli/src/relevance.rs
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 171: Code Review Report

**Reviewed:** 2026-06-08
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 171 ships two new CLI commands (`ferro ai:make` and `ferro ai:explain`) backed by `ferro-ai`'s typed completion primitives. The overall design is sound: path traversal is correctly blocked by `resolve_projection_path` → `to_snake_case` → `is_valid_identifier`, all string values placed into generated Rust source use `{:?}` debug formatting (proper escape), the tokio runtime bridge is handled cleanly, and introspection fallbacks are in place for unavailable DB and HTTP routes.

Three warnings and three info items are noted below. No critical issues were found.

---

## Warnings

### WR-01: Function name in generated source uses raw LLM-controlled service name, not the sanitized snake_case name

**File:** `ferro-cli/src/commands/ai_make.rs:27-29`

**Issue:** `emit_service_def_source` reads `service.name` directly to build the function name and the `ServiceDef::new(...)` call. `render_output` validates and snake_cases the name for the *file path* via `resolve_projection_path`, but the same sanitized value is never passed back into the emitter. If the LLM returns `service.name = "OrderItem"` (PascalCase), the generated file is `src/projections/order_item.rs` but the function inside is named `OrderItem_service`, which violates Rust naming conventions and triggers a `non_snake_case` compiler warning. For inputs already accepted by `is_valid_identifier` (e.g., pure snake, or a single uppercase letter like `"A"`), the generated Rust is syntactically valid but stylistically wrong and `clippy -D warnings` will reject it.

**Fix:** In `render_output`, extract the validated snake name from `resolve_projection_path` and pass it to the emitter, or have `emit_service_def_source` accept an explicit `fn_name_override`:

```rust
// In render_output, after resolve_projection_path succeeds:
let snake_name = crate::naming::to_snake_case(&service.name);
let content = emit_service_def_source_with_name(service, &snake_name);
```

Or simpler: `emit_service_def_source` derives `fn_name` from `to_snake_case(name)` rather than raw `name`.

---

### WR-02: Module-local `ENV_LOCK` instances do not serialize env-var tests across modules

**File:** `ferro-cli/src/commands/ai_make.rs:708` and `ferro-cli/src/commands/ai_explain.rs:391`

**Issue:** Each test module defines its own `static ENV_LOCK: Mutex<()>`. Because each module has a separate `Mutex` instance, parallel test threads from `ai_make`'s tests and `ai_explain`'s tests can race on `FERRO_AI_MAX_TOKENS_PER_COMMAND`. Both modules mutate this env var. The `ferro_ai::lib.rs` crate-level `ENV_LOCK` pattern documents exactly this risk: "Per-module static ENV_LOCK instances are insufficient because each module gets its own mutex instance."

**Fix:** Promote a single `pub(crate) static ENV_LOCK` to `ferro-cli/src/commands/mod.rs` (it already exports `CWD_TEST_LOCK` at that level as a precedent) and replace both module-local instances:

```rust
// In ferro-cli/src/commands/mod.rs
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
```

---

### WR-03: Misleading doc comment in `ai_explain::run` claims dry-run validates AI config; code does not

**File:** `ferro-cli/src/commands/ai_explain.rs:291-295`

**Issue:** The comment reads: _"Even in dry-run we validate config to surface missing env vars early, but we skip the actual LLM call."_ The code directly below only exits on config error when `!dry_run`. In dry-run mode a missing `FERRO_AI_API_KEY` is silently swallowed. The subsequent `client_result.expect("already validated above")` at line 346 is safe (reached only after `!dry_run` is enforced at line 295 and the dry-run early-return at line 335), but any reader who trusts the comment and adds code in the dry-run path after line 335 expecting the client to be valid will get a panic.

**Fix:** Either remove the false claim from the comment, or enforce the check unconditionally:

```rust
// Option A: fix the comment
// 1. Fail-fast: require AI provider configuration (D-06). In dry-run, config is NOT checked.
let client_result = AiConfig::from_env();
if !dry_run {
    if let Err(ref e) = client_result { ... std::process::exit(1); }
}
```

---

## Info

### IN-01: Prompt injection mitigation is delimiters-only; `</description>` inside user input closes the tag early

**File:** `ferro-cli/src/commands/ai_make.rs:619-622`

**Issue:** The prompt wraps `description` in `<description>...</description>` XML tags. This is the standard Anthropic-recommended approach, but it does not prevent a crafted description that contains `</description>` from closing the tag prematurely and appending arbitrary content outside the delimiters. For a CLI tool only invoked by the developer themselves, the practical threat is low. The system prompt's "do not use generic placeholders" instruction provides mild semantic resistance but not structural resistance.

**Suggestion:** Strip or escape `</description>` from the description before embedding, or switch to a delimiter that users are unlikely to type (e.g., `<<<DESCRIPTION>>>`):

```rust
let safe_description = description.replace("</description>", "[/description]");
let user_prompt = format!("...\n<description>\n{safe_description}\n</description>");
```

---

### IN-02: `to_snake_case` in `naming.rs` does not handle hyphens; tests have no hyphen coverage

**File:** `ferro-cli/src/naming.rs:23-36`

**Issue:** `to_snake_case` only converts uppercase letters to `_lower`; hyphens, spaces, and dots are passed through unchanged. A name like `"order-item"` produces `"order-item"` which `is_valid_identifier` correctly rejects. This is safe, but the gap between user expectation ("hyphenated names should work like PascalCase") and behavior may surface as confusing error messages. The test suite has no hyphen coverage.

**Suggestion:** Either extend `to_snake_case` to replace hyphens and spaces with underscores (matching typical CLI name normalization), or add a test that documents the current rejection behavior explicitly.

---

### IN-03: `make_projection.rs` `model_aware_template` uses unescaped `{}` for `FieldMeaning::Custom` value

**File:** `ferro-cli/src/commands/make_projection.rs:335`

**Issue:** `format!("FieldMeaning::Custom(\"{}\".into())", field.name)` embeds `field.name` without `{:?}` debug escaping. In contrast, `ai_make.rs`'s `emit_field_meaning` correctly uses `format!(r#"FieldMeaning::Custom({s:?}.into())"#)`. Field names derived from `syn::Ident` cannot contain double-quotes, so there is no current path to exploit this, but the inconsistency is a maintenance trap: if `field.name` ever comes from a less-constrained source the generated Rust source would be syntactically broken rather than producing a compile error with a clear location.

**Suggestion:** Align with the pattern in `ai_make.rs`:

```rust
// Before:
format!("FieldMeaning::Custom(\"{}\".into())", field.name)
// After:
format!("FieldMeaning::Custom({:?}.into())", field.name)
```

---

_Reviewed: 2026-06-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
