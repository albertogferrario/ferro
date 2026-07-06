---
phase: 172-mcp-tool-wrappers
reviewed: 2026-06-08T00:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - ferro-mcp/src/tools/relevance.rs
  - ferro-mcp/src/tools/ai_scaffold.rs
  - ferro-mcp/src/tools/ai_explain_core.rs
  - ferro-mcp/src/tools/mod.rs
  - ferro-mcp/src/lib.rs
  - ferro-mcp/src/service.rs
  - ferro-cli/src/commands/ai_make.rs
  - ferro-cli/src/commands/ai_explain.rs
  - ferro-cli/src/commands/mod.rs
  - ferro-cli/src/lib.rs
  - Cargo.toml
  - docs/src/features/ai.md
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 172: Code Review Report

**Reviewed:** 2026-06-08
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Phase 172 relocated `scaffold_core` and `explain_core` from ferro-cli into ferro-mcp and registered two MCP tools (`ai_scaffold`, `ai_explain`). The CLI is now a thin wrapper. The high-priority invariants from the phase brief all hold:

- `scaffold_core` performs no disk writes (D-02 confirmed).
- Neither core calls `process::exit`, `eprintln!`, or panics — they return `Result<_, String>` throughout.
- No `block_on` / runtime bridge inside the cores; every async call uses `.await` directly.
- The ferro-mcp → ferro-cli dependency direction is correct: ferro-cli depends on ferro-mcp, not the reverse (confirmed via Cargo.toml).
- `sanitize_description` is applied before the description is embedded in the LLM prompt (T-172-PI).

Three warnings were found — one is a logic gap (sanitization not applied on the `ai_explain` target in the prose branches), one is a type-safety issue in the CLI dry-run flow, and one is a residual `process::exit(0)` for a non-error case. Three info items cover minor style and documentation issues.

---

## Warnings

### WR-01: `ai_explain` target string reaches LLM prompt unescaped in prose branches

**File:** `ferro-mcp/src/tools/ai_explain_core.rs:220-265`
**Issue:** The module's threat model doc says the `target` argument is "used as a lookup key only" and "the prose prompt is built from introspected artifact facts, not from the raw target string." This is accurate for the route/model *fact fields* (`r.purpose`, `m.domain_meaning`, etc.), but `r.route`, `r.method`, `r.handler`, `m.model`, and `m.table` are sourced directly from the project's introspection layer and are generally project-owned data, so the residual risk is low as documented.

However, one gap exists: `build_route_prompt` embeds `r.guards.join(", ")` and `r.related_routes.join(", ")` (line 228), and `build_model_prompt` embeds `m.relationships.join(", ")` and `m.related_routes.join(", ")` (line 255). These fields are built from static-analysis results that scan Rust source — they could include content from code comments or string literals in source files that an attacker who can write to the project source could control. The risk is confined to the project owner's own source tree, but the threat model in the doc comment implies "zero injection surface" for these paths, which slightly overstates the protection.

More concretely: the `call_llm_prose` path in `explain_core` (line 288-305) embeds all of these fields directly into the user prompt with no sanitization, while `scaffold_core` applies `sanitize_description` before embedding the user-supplied string.

**Fix:** Either update the module's doc comment to accurately state the residual risk (the route/model facts come from project source, not from the `target` parameter), or apply a narrow sanitization (`replace("<", "[").replace(">", "]")`) to `r.related_routes`, `r.guards`, `m.relationships`, and `m.related_routes` before embedding them in the prompt. The doc-comment update is the lower-effort fix and is accurate given the actual threat boundary.

---

### WR-02: CLI `ai_explain` `--dry-run` uses `resolve_target(Path::new("."))` but `ai_make` uses `current_dir()`

**File:** `ferro-cli/src/commands/ai_explain.rs:61`
**Issue:** `resolve_target` is called with `Path::new(".")` as the project root:

```rust
let resolved = rt.block_on(resolve_target(
    std::path::Path::new("."),
    &target,
    type_override.as_deref(),
));
```

`ai_make` uses `std::env::current_dir()` for the same purpose (line 465 of `ai_make.rs`). Both approaches resolve to the same directory at runtime, but `Path::new(".")` is a relative path. If any callee internally converts the path to absolute (e.g., via `canonicalize`) and the CWD is not the project root, they diverge. `scaffold_core` gets `&cwd` (an absolute `PathBuf`) while `explain_core` gets a `&Path` relative ref. The inconsistency is a latent bug — if the MCP server or a future test sets CWD to something other than the project root, `ai_explain` silently looks in the wrong place.

**Fix:**
```rust
let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
let resolved = rt.block_on(resolve_target(
    &cwd,
    &target,
    type_override.as_deref(),
));
```

---

### WR-03: `ai_make::run` exits with code 0 for the `AlreadyExists` path — misleading for scripting callers

**File:** `ferro-cli/src/commands/ai_make.rs:491-495`
**Issue:** When a projection file already exists, the CLI prints a message to `stderr` (using `eprintln!`) and calls `std::process::exit(0)`. Printing to stderr while exiting 0 is a contradictory signal: stderr is conventionally used for errors or warnings, but exit 0 signals success to the shell. A calling script that checks exit code to detect "nothing was written" will treat this as success and proceed, potentially overwriting or skipping follow-up steps that depend on the file being freshly created.

```rust
Ok(OutputResult::AlreadyExists(path)) => {
    eprintln!(
        "{} Projection already exists at {}. Delete it first or use a different name.",
        style("Info:").yellow().bold(),
        path.display()
    );
    std::process::exit(0);  // <-- exit 0 despite printing to stderr
}
```

**Fix:** Either use `println!` (stdout) to match exit 0, or exit with a distinct non-zero code (e.g., 2) so callers can detect the "already exists" case. Given this is an informational skip (not an error), `println!` + exit 0 is the more user-friendly choice:

```rust
Ok(OutputResult::AlreadyExists(path)) => {
    println!(
        "{} Projection already exists at {}. Delete it first or use a different name.",
        style("Info:").yellow().bold(),
        path.display()
    );
    // No process::exit — normal return, exit code 0
}
```

---

## Info

### IN-01: `sanitize_description` only strips `<description>` and `</description>` — other XML-significant sequences pass through

**File:** `ferro-mcp/src/tools/ai_scaffold.rs:35-39`
**Issue:** The sanitization correctly addresses the T-172-PI threat (closing the `<description>` tag early). It does not strip other XML delimiters (`<system>`, `</system>`, `<user>`, `</user>`, etc.). Depending on the LLM provider's internal prompt structure, other tag names might have significance. This is low-risk for the current prompt structure (no other XML-tagged blocks in the assembled prompt), but if the prompt template is ever extended with additional XML sections, the mitigation scope will not automatically extend.

**Fix:** Document the narrow scope explicitly in the `sanitize_description` docstring, or generalize to strip all `<…>` sequences: `description.replace('<', "[").replace('>', "]")`. The broader replacement is unambiguous and removes the need to enumerate tag names.

---

### IN-02: `resolve_kind_priority` accepts unknown `type_override` values silently with a "not_found" return, not an error

**File:** `ferro-mcp/src/tools/ai_explain_core.rs:56-83`
**Issue:** When `type_override` is `Some("bogus_value")`, the function returns `"not_found"`. The caller `resolve_target` propagates this into `ResolvedTarget::NotFound(format!("Unknown --type value '{other}'. Use 'service', 'route', or 'model'."))` — so the user-visible error message is correct. However, `resolve_kind_priority` itself returning `"not_found"` for an unknown override conflates "no match" with "bad input." This makes the function contract ambiguous for any future caller that directly uses `resolve_kind_priority` without going through `resolve_target`.

**Fix:** Return `Option<&'static str>` (or a dedicated enum) from `resolve_kind_priority` where `None` means "unknown override" vs. `Some("not_found")` meaning "no artifact matched." Alternatively, add a doc comment stating that the function returns `"not_found"` for unknown override values and that `resolve_target` is the correct call site for user-facing error handling.

---

### IN-03: `docs/src/features/ai.md` references an outdated env var for the Anthropic provider

**File:** `docs/src/features/ai.md:17`
**Issue:** Line 17 reads `Set ANTHROPIC_API_KEY in your .env or environment before using the AnthropicProvider.` The MCP tools (`ai_scaffold`, `ai_explain`) and `scaffold_core`/`explain_core` require `FERRO_AI_API_KEY` (along with `FERRO_AI_PROVIDER` and `FERRO_AI_MODEL`). The document covers the lower-level `ferro-ai` Classifier API which uses `ANTHROPIC_API_KEY` directly, but the new MCP-tool section added at the bottom (lines 325-371) correctly names `FERRO_AI_PROVIDER`, `FERRO_AI_API_KEY`, and `FERRO_AI_MODEL`. The top-level setup section is accurate for the Classifier API; the inconsistency is that a reader skimming the document may miss the distinction between the two API surfaces.

**Fix:** Add a note near line 17 clarifying that `ANTHROPIC_API_KEY` applies to the direct `AnthropicProvider` / `Classifier` API, while the `ferro ai:make`, `ferro ai:explain`, and MCP tool surface uses the provider-agnostic `FERRO_AI_*` env vars documented in the MCP Tools section.

---

_Reviewed: 2026-06-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
