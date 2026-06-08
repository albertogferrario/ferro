---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
plan: "03"
subsystem: ferro-ai
tags: [schema, normalizer, structured-output, servicedef-aware, projection-enum-closing, complete, tdd, wave-3, sc1, sc3]
dependency_graph:
  requires: [166-01, 166-02]
  provides: [close_projection_enum, complete-typed-entry, sc1, sc3, servicedef-aware-normalizer]
  affects: [ferro-ai]
tech_stack:
  added: []
  patterns: [projection-enum-closing, ref-sibling-merge, jsonschema-draft202012, typed-complete-wrapper]
key_files:
  created: [ferro-ai/src/complete.rs, ferro-ai/tests/projection_schema.rs]
  modified: [ferro-ai/src/schema/mod.rs, ferro-ai/src/lib.rs]
decisions:
  - "close_projection_enum handles two schemars shapes: enum-array (FieldMeaning, no per-variant docs) and const-per-branch (Intent, per-variant docs) — vocabulary from schema, never hardcoded (D-08)"
  - "resolve_refs extended to handle $ref with sibling keys (e.g. description) — schemars emits {$ref, description} on property annotations; previous obj.len()==1 guard left these unresolved"
  - "complete::<T>() struct literal lists exactly 5 current CompletionRequest fields; Plan 04 must update when tools/tool_choice are added (explicit dependency note in module doc)"
  - "complete_into::<T>() (D-02) deferred — no plan requirement mandated it and the caller can set schema directly if needed"
metrics:
  duration: "~525 seconds (~9 minutes)"
  completed: "2026-06-08T05:30:00Z"
  tasks_completed: 3
  tasks_total: 3
  files_created: 2
  files_modified: 2
---

# Phase 166 Plan 03: ServiceDef-aware Enum Closing + complete::<T>() + SC#3 Summary

The CRUX plan: ServiceDef-aware `FieldMeaning`/`Intent` enum closing (D-06), typed `complete::<T>()` entry point (D-01), and SC#3 structural-guarantee test with `jsonschema::draft202012`. The LLM is now locked to valid projection vocabulary by a structural schema constraint, verified by a jsonschema validation test that rejects invalid values and accepts all 18 FieldMeaning and 7 Intent variants.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | ServiceDef-aware projection enum closing (D-06/D-07/D-08) | 5553c2b3 | ferro-ai/src/schema/mod.rs |
| 2 | SC#3 structural-guarantee test + $ref sibling-key fix | 1b350ced | ferro-ai/src/schema/mod.rs, ferro-ai/tests/projection_schema.rs |
| 3 | complete::<T>() typed entry point + SC#1 test + re-exports | 3efbc858 | ferro-ai/src/complete.rs, ferro-ai/src/lib.rs |

## Decisions Made

**Two schemars anyOf shapes handled:** `close_projection_enum` uses `.find(|b| b.get("enum").is_some())` for Shape A (FieldMeaning — enum-array branch) and collects `.get("const")` values for Shape B (Intent — const-per-variant). This was documented in Plan 01's probe but required explicit branching in the implementation. Both cases derive vocabulary from the schema itself (D-08).

**$ref with sibling keys fix:** `resolve_refs` previously only handled pure `{"$ref": "..."}` objects (len==1). schemars 1.x emits `{"$ref": "#/$defs/Cardinality", "description": "..."}` on property-level field annotations. The fix: any object containing `$ref` is treated as a ref node; sibling keys (like `description`) are merged into the resolved result. This was a Rule 1 bug discovered when `jsonschema::draft202012::new` failed with `PointerToNowhere` on surviving `$ref`s in the normalized `ServiceDef` schema.

**complete::<T>() CompletionRequest literal:** The struct literal lists exactly the five fields from Phase 165 (system, messages, max_tokens, model_override, schema). Plan 04 adds `tools: Option<Vec<ToolRequest>>` and `tool_choice: Option<ToolChoice>` — it is responsible for updating this literal with `tools: None, tool_choice: None`. This is documented in the `complete.rs` module doc comment.

**complete_into::<T>() deferred:** D-02 was Claude's discretion. Not implemented — the primary `complete::<T>(client, prompt)` surface satisfies all SC#1 requirements. Callers needing custom system/max_tokens can construct the request directly via `CompletionRequest` and call `client.complete()`.

## Verification Results

- `cargo test -p ferro-ai schema::` — 9/9 green (Plan 01 probe + Plan 02 generic + Plan 03 closing)
- `cargo test -p ferro-ai --test projection_schema` — 6/6 green (SC#3 full coverage):
  - `servicedef_schema_rejects_invalid_field_meaning` (totally_bogus fails, money passes)
  - `servicedef_schema_rejects_invalid_intent` (bogus intent fails, browse passes)
  - `servicedef_schema_accepts_all_known_field_meaning_variants` (18 variants)
  - `servicedef_schema_accepts_all_known_intent_variants` (7 variants)
  - `servicedef_schema_accepts_minimal_servicedef`
  - `normalized_schema_has_no_surviving_refs` (regression guard)
- `cargo test -p ferro-ai complete` — 2/2 green (SC#1):
  - `complete_returns_typed_result`
  - `complete_propagates_deserialization_error`
- `cargo clippy --all --all-targets -- -D warnings` — clean (full workspace)
- `cargo fmt --all -- --check` — clean

## Deviations from Plan

### Auto-fixed Issues

**[Rule 1 - Bug] resolve_refs did not handle $ref with sibling keys**
- **Found during:** Task 2 — `jsonschema::draft202012::new(&normalized_schema)` failed with `PointerToNowhere { pointer: "/$defs/Cardinality" }`
- **Issue:** schemars emits `{"$ref": "#/$defs/Cardinality", "description": "Structural cardinality..."}` for property-level field annotations. The previous `obj.len() == 1` guard in `resolve_refs` only handled pure `$ref` objects, leaving `Cardinality` and `NavigationHint` refs unresolved when they appeared alongside a `description` key.
- **Fix:** Changed the match guard from `obj.len() == 1 && obj.contains_key("$ref")` to `obj.contains_key("$ref")`. When sibling keys exist, they are merged into the resolved object (sibling `description` overrides same-named key in the resolved def).
- **Files modified:** ferro-ai/src/schema/mod.rs
- **Commit:** 1b350ced

### Test shape correction (Task 1 TDD RED)
- The initial test fixtures for `closes_field_meaning_enum` and related tests used `{"$defs": ..., "$ref": ...}` at the top level (two keys), which the resolver treated as a generic object (not a pure `$ref` node). Corrected to use `{"type": "object", "properties": {"field": {"$ref": "..."}}, "$defs": ...}` which is the real schemars output shape.

## Plan 04 Obligation

**Plan 04 must update the `CompletionRequest` struct literal in `ferro-ai/src/complete.rs`** when it adds `tools: Option<Vec<ToolRequest>>` and `tool_choice: Option<ToolChoice>` fields to `CompletionRequest`. The current literal:

```rust
let request = CompletionRequest {
    system: None,
    messages: vec![...],
    max_tokens: 4096,
    model_override: None,
    schema: Some(normalized),
};
```

Must become (after Plan 04):

```rust
let request = CompletionRequest {
    system: None,
    messages: vec![...],
    max_tokens: 4096,
    model_override: None,
    schema: Some(normalized),
    tools: None,
    tool_choice: None,
};
```

## Known Stubs

None — all three success criteria (SC#1, SC#3, D-01, D-06, D-07, D-08) are fully implemented production code, not stubs.

## Threat Surface Scan

No new network endpoints, auth paths, or file access. The threat model mitigations from the plan:

- **T-166-PI-01 (Tampering / prompt-injection):** `close_projection_enum` drops the `Custom(String)` escape hatch from the LLM-facing schema. The model cannot emit a non-vocabulary `FieldMeaning` or `Intent` value — the closed schema is a structural control. Verified by SC#3 tests.
- **T-166-PI-02 (Spoofing — invalid value passes validation):** `servicedef_schema_rejects_invalid_field_meaning` and `servicedef_schema_rejects_invalid_intent` assert that invalid values fail `jsonschema::draft202012` validation. Not a vacuously-passing test (A4 confirmed).
- **T-166-PI-03 (Information disclosure — deserialization error):** `Error::Deserialization(e.to_string())` carries serde's parse message (offset/expected token), not provider secrets — consistent with existing classifier path.

## Self-Check: PASSED

- `ferro-ai/src/schema/mod.rs` contains `fn close_projection_enum`: confirmed
- `ferro-ai/src/schema/mod.rs` contains `PROJECTION_DEF_NAMES` with `"FieldMeaning"` and `"Intent"`: confirmed
- `ferro-ai/tests/projection_schema.rs` exists and contains `servicedef_schema_rejects_invalid_field_meaning`: confirmed
- `ferro-ai/src/complete.rs` contains `pub async fn complete`: confirmed
- `ferro-ai/src/lib.rs` contains `pub mod complete;` and `pub use complete::complete;`: confirmed
- Commit 5553c2b3 exists: confirmed
- Commit 1b350ced exists: confirmed
- Commit 3efbc858 exists: confirmed
- `cargo test -p ferro-ai` — 68 unit tests + 6 integration tests green: confirmed
- `cargo clippy --all --all-targets -- -D warnings` — clean: confirmed
