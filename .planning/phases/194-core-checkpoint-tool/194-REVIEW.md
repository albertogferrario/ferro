---
phase: 194-core-checkpoint-tool
reviewed: 2026-06-10T00:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - ferro-mcp/src/tools/checkpoint_projection.rs
  - ferro-mcp/src/service.rs
  - ferro-mcp/src/tools/mod.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 194: Code Review Report

**Reviewed:** 2026-06-10
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

The `checkpoint_projection` tool is well-structured. The path-traversal guard (`validate_name`) is correctly called as the first statement of `run_for` before any path construction or IO, and the accepted charset (`[a-zA-Z0-9_-]`) is genuinely safe. The coverage-honesty invariant (CHK-03) is upheld: `not_checked` seams never coerce to `fail`, and every prerequisite-absent path returns `NotChecked`. The field→column comparison logic is correct; it iterates `service.fields` (column-backed builders only), not relationships. Cache write IO is properly guarded with `create_dir_all` and mapped errors at each fallible step, no `unwrap` on IO paths. The MCP registration in `service.rs` is consistent with sibling tools.

Three warnings and two info items were found.

## Warnings

### WR-01: `regex::Regex::new(...).unwrap()` inside hot path — panic propagates as MCP crash

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:333-336`

**Issue:** `count_column_backed_builders` compiles four regexes inside a `map` closure using `.unwrap()`. The patterns are currently valid, so the panic path is never exercised, but `.unwrap()` in a non-test, non-main library function means a future edit that introduces a malformed pattern (e.g. during a refactor) would panic and crash the MCP tool call entirely rather than returning a structured `SeamResult::NotChecked`. There is no test that covers the panic path. A panic in this function propagates up through `field_to_column_seam` and through `run_for`, bypassing the `Result<Verdict, String>` error-propagation contract.

**Fix:** Use `OnceLock`-cached compiled regexes so compilation happens once at first call and errors are impossible at the call site:

```rust
use std::sync::OnceLock;

static FIELD_PATTERNS: OnceLock<[regex::Regex; 4]> = OnceLock::new();

fn field_patterns() -> &'static [regex::Regex; 4] {
    FIELD_PATTERNS.get_or_init(|| {
        [
            regex::Regex::new(r"\.field\(").expect("static pattern"),
            regex::Regex::new(r"\.optional_field\(").expect("static pattern"),
            regex::Regex::new(r"\.read_only_field\(").expect("static pattern"),
            regex::Regex::new(r"\.write_only_field\(").expect("static pattern"),
        ]
    })
}
```

Using `expect` on truly-static patterns is acceptable; the `OnceLock` ensures the panic window is the first call only and the string is self-documenting. Alternatively, use the `once_cell::sync::Lazy` or `std::sync::LazyLock` pattern if already available in the dependency graph.

---

### WR-02: Block comments not stripped — spurious D-06 `Warn` on `/* .field( */ ` patterns

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:311-323`

**Issue:** `count_column_backed_builders` strips `//` line comments before counting, but does not strip `/* ... */` block comments. A projection source that contains a commented-out builder call in a block comment:

```rust
/*
 * .field("old_col", DataType::Integer, FieldMeaning::Identifier)
 */
```

will have the `.field(` pattern counted, inflating `invocation_count` above `service.fields.len()` and triggering the D-06 completeness `Warn` with `reason: "reconstruction_incomplete"`. The generated finding points the agent toward "unsupported builder patterns" when the real cause is a block comment. This is a false-positive that degrades tool reliability; the agent receives a misleading `Warn` verdict for a structurally correct projection.

**Fix:** Strip block comments before counting. A minimal approach that avoids a full parser:

```rust
fn strip_comments(content: &str) -> String {
    // Strip /* ... */ block comments (non-greedy, handles multi-line).
    let block_re = regex::Regex::new(r"/\*[\s\S]*?\*/").expect("static pattern");
    let no_blocks = block_re.replace_all(content, "");
    // Strip // line comments.
    no_blocks
        .lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

Note: `//` inside a string literal (e.g. a URL default) would still be incorrectly stripped by the current logic too, but that is a pre-existing limitation documented in RESEARCH.md Pitfall 2 and is out of scope here.

---

### WR-03: `CheckpointProjectionParams.name` doc claims to accept service names — lookup only resolves function names

**File:** `ferro-mcp/src/service.rs:328-330`

**Issue:** The parameter doc comment reads:

> Projection function name (e.g. "user_service") or service name (e.g. "User").

However, `execute` → `run_for` calls `inspect_projection::execute(project_root, name)`, which resolves by projection function name. Passing a bare service/model name like `"User"` will result in `inspect_projection` returning `InspectResult::NotFound`, causing `run_for` to return `Err("projection 'User' not found. Available: [...]")`. The MCP tool returns `{"error": "projection 'User' not found ..."}`. Agents following the documented contract will try both forms and get confused by the asymmetry. The "or service name" clause is incorrect.

**Fix:** Correct the doc comment to match actual behavior:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CheckpointProjectionParams {
    /// Projection function name (e.g. "user_service"). Use `list_projections` to discover available names.
    pub name: String,
}
```

---

## Info

### IN-01: All-`NotChecked` aggregate resolves to `Pass` — may mislead callers in Phase 195 transition

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:347-361`

**Issue:** `aggregate_status` returns `SeamStatus::Pass` when all seams are `NotChecked` (the test `aggregate_status_all_not_checked_is_pass` asserts this explicitly). With the current Phase 194 state (4 of 5 seams are `NotChecked` stubs), any projection where seam 2 (`field_to_column`) resolves to `Pass` will produce an overall verdict of `Pass`. An agent reading `status: "pass"` may conclude the projection is fully validated when in fact most seams were not run. The `seams` array contains the `not_checked` entries, so a careful agent can detect this, but the top-level status signal is optimistic relative to actual coverage.

**Note:** This is intentional per the locked CHK-03 spec ("not_checked seams do not raise the overall status to fail"). The concern is that the spec does not address the `all-not-checked = pass` implication. Worth revisiting as Phase 195 fills in the remaining seams to decide if a `not_checked` aggregate status variant is warranted.

**Suggestion:** No code change required now. When Phase 195 lands, consider adding a `partial` or `not_checked` aggregate status variant, or add a `checked_seam_count` field to `Verdict` so callers can detect low-coverage runs without parsing every `SeamResult`.

---

### IN-02: D-06 `Warn` finding uses entity name as `subject`, not the unresolvable field

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:218-231`

**Issue:** When the completeness check fires (invocation count exceeds parsed field count), the single `Finding` has `subject: service_name.to_string()` — the entire entity name. The finding is about reconstruction incompleteness, not about a specific field. This makes the `next_steps` deduplication key `(service_name, fix_string)`, which is correct, but the actionable text points the agent toward "unsupported builder patterns" generically rather than naming the specific field that failed to parse. When combined with WR-02's block-comment false positive, the subject granularity gap makes the finding harder to act on.

**Suggestion:** If reconstruction is incomplete, log which `DataType` variants were unrecognised during `reconstruct_service_def` and include them in the `detail` field. This requires a small change to the `reconstruct_service_def` return type or a second pass counting unrecognised type strings.

---

_Reviewed: 2026-06-10_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
