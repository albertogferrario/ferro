---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
verified: 2026-06-08T06:30:00Z
status: passed
score: 7/7
overrides_applied: 0
re_verification: false
---

# Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer — Verification Report

**Phase Goal:** Ship `ferro_ai::complete::<T>()` for typed structured outputs, the schema normalizer that makes `schemars` output compatible with provider structured-output APIs, the `ServiceDef`-aware specialization that locks the LLM to valid projection shapes, and `ToolRegistry` with a hard `max_iterations` guard.
**Verified:** 2026-06-08T06:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ferro_ai::complete::<T>(client, prompt)` exists with signature `T: schemars::JsonSchema + serde::DeserializeOwned -> Result<T, Error>`; caller never calls schemars/serde_json directly | VERIFIED | `complete.rs:57` — exact signature confirmed. `lib.rs:60` re-exports. Unit test `complete_returns_typed_result` passes with mock client. |
| 2 | `ferro_ai::schema::for_structured_output` resolves `$ref`/`$defs` inline, adds `additionalProperties:false` to every object, strips Anthropic-rejected keywords, PRESERVES `enum`; unit test verifies | VERIFIED | `schema/mod.rs` — full implementation confirmed. `STRIP_KEYWORDS` array excludes `enum`. `normalize_node` adds `additionalProperties:false` only when `type==object && properties present`. Tests: `schema_normalizer_strips_rejected_keywords`, `schema_normalizer_resolves_refs`, `schema_normalizer_preserves_enum`, `schema_normalizer_skips_additional_properties_on_anyof` — all pass. |
| 3 | ServiceDef-aware path closes `FieldMeaning`/`Intent` enums (drops `Custom(String)` open branch); invalid FieldMeaning/Intent FAILS jsonschema validation; valid values pass; vocabulary sourced from ferro-projections NOT a hardcoded list | VERIFIED | `close_projection_enum` function reads vocabulary from actual schemars output — no hardcoded variant strings in production code. `projection_schema.rs` tests: `servicedef_schema_rejects_invalid_field_meaning` asserts validation failure on `"totally_bogus"`, `servicedef_schema_rejects_invalid_intent` asserts failure on `"totally_bogus_intent"`. All 6 projection_schema tests pass. |
| 4 | `ToolDef` carries name/description/parameters_schema (normalized via `for_structured_output`)/async handler | VERIFIED | `tools/mod.rs:54-74` — all four fields present with correct types. `ToolDef.parameters_schema: serde_json::Value` docs state normalization is required. `tool_def_construction` test passes. |
| 5 | `ToolRegistry::dispatch` runs the loop; `max_iterations: u32` required at construction with NO unbounded/override path (no `Default`, no zero-arg ctor); warn@5 fires when `max_iterations > 5`; error@hard-cap | VERIFIED | `tools/mod.rs:130-140` — `ToolRegistry::new(max_iterations)` is the only full constructor. No `impl Default`. `dispatch` loop at line 227: `if iteration == 5 && self.max_iterations > 5 { warn!(...) }` fires BEFORE cap check (WR-02 fix confirmed). `tool_registry_enforces_max_iterations` test passes. `tool_registry_requires_max_iterations` documents no zero-arg path. |
| 6 | `ToolError { message: String }` model-legible; tool failures surfaced as this message, not raw panics/DB strings | VERIFIED | `tools/mod.rs:37-41` — `ToolError { pub message: String }`. `result_to_message` returns `te.message` on error. Tests: `tool_error_is_model_legible`, `dispatch_surfaces_tool_error`, `dispatch_surfaces_unknown_tool_as_tool_error` — all pass. |
| 7 | `cargo test --all-features` passes; existing `Classifier<T>` tests green | VERIFIED | `cargo test -p ferro-ai` confirms: 80 unit tests + 6 integration tests pass. 0 failures. Classifier tests (part of the 80) confirmed green. |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-ai/src/complete.rs` | `complete::<T>()` entry point | VERIFIED | 144 lines; function + 2 unit tests. Re-exported at `lib.rs:60`. |
| `ferro-ai/src/schema/mod.rs` | Normalizer + Wave 0 probes + closing pass | VERIFIED | 711 lines; `for_structured_output`, `close_projection_enum`, `normalize_node`, `resolve_refs`, full test suite. |
| `ferro-ai/src/tools/mod.rs` | `ToolDef`, `ToolError`, `ToolRegistry`, dispatch loop | VERIFIED | 577 lines; all types + dispatch loop + CR/WR regression tests. |
| `ferro-ai/src/client/mod.rs` | `Message.tool_call_id`, `CompletionResponse::ToolUse{blocks,assistant_content}`, `complete_with_tools` | VERIFIED | `tool_call_id: Option<String>` on Message (CR-01/03 fix). `CompletionResponse::ToolUse` is struct variant with both fields (CR-02 fix). |
| `ferro-ai/src/client/anthropic.rs` | Correct Anthropic wire format for tool results | VERIFIED | `Role::Tool` maps to `{"role":"user","content":[{"type":"tool_result","tool_use_id":...}]}` using `m.tool_call_id`. Regression test `test_build_body_tool_result_wire_format` passes. |
| `ferro-ai/src/client/openai.rs` | Correct OpenAI wire format; `tool_choice` honored | VERIFIED | `Role::Tool` maps to `{"role":"tool","tool_call_id":...,"content":...}` using `m.tool_call_id`. `tool_choice` respects `ToolChoice::None` vs `Auto`. Regression tests pass. |
| `ferro-ai/src/lib.rs` | Re-exports: `complete`, `for_structured_output`, `make_handler`, `ToolDef`, `ToolError`, `ToolRegistry` | VERIFIED | Lines 60, 66, 67 — all items re-exported at crate root. `make_handler` included (IN-01 fix). |
| `ferro-ai/src/error.rs` | `SchemaError`, `ToolIterationLimit`, `ToolNotFound` variants + WR-03 doc | VERIFIED | All three variants present with `thiserror` derive. `ToolNotFound` has detailed doc explaining it is reserved for future use (WR-03 fix). |
| `ferro-ai/tests/projection_schema.rs` | SC#3 structural-guarantee test; asserts failure on invalid values | VERIFIED | 194 lines; 6 tests covering both failure and success paths for FieldMeaning and Intent. `normalized_schema_has_no_surviving_refs` regression guard present. |
| `.github/workflows/publish.yml` | WAVE1B has `ferro-projections` before `ferro-ai` | VERIFIED | Line 246: `WAVE1B_CRATES="ferro-projections ferro-ai ..."`. Dep comment at line 242 documents the new edge. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-ai/src/lib.rs` | `schema::for_structured_output` | `pub use` | WIRED | `lib.rs:66` |
| `ferro-ai/src/lib.rs` | `complete::complete` | `pub use` | WIRED | `lib.rs:60` |
| `ferro-ai/src/lib.rs` | `tools::{make_handler, ToolDef, ToolError, ToolRegistry}` | `pub use` | WIRED | `lib.rs:67` |
| `complete.rs` | `schema::for_structured_output` | call inside `complete::<T>()` | WIRED | `complete.rs:63` |
| `complete.rs` | `client.complete(request)` | `LlmClient::complete` call | WIRED | `complete.rs:79` |
| `tools/mod.rs dispatch` | `client.complete_with_tools` | dispatch loop | WIRED | `tools/mod.rs:243` |
| `tools/mod.rs dispatch` | assistant turn pushed BEFORE tool results | CR-02 ordering requirement | WIRED | `tools/mod.rs:257-268` — `Role::Assistant` message pushed before tool results loop |
| `anthropic.rs Role::Tool` | `tool_use_id` as real field | `m.tool_call_id` | WIRED | `anthropic.rs:69-77` — `tool_use_id: tool_use_id` in content block |
| `openai.rs Role::Tool` | `tool_call_id` as top-level field | `m.tool_call_id` | WIRED | `openai.rs:74-79` — `"tool_call_id": call_id` at message root |
| `openai.rs tool_choice` | `ToolChoice::None` → `"none"` | match on `request.tool_choice` | WIRED | `openai.rs:123-126` — match expression, WR-01 fix confirmed |
| `for_structured_output` | `close_projection_enum` before ref inlining | call order in Step 1 | WIRED | `schema/mod.rs:206-210` — closing runs before `resolve_refs` at line 231 |
| `WAVE1B_CRATES` publish order | `ferro-projections` before `ferro-ai` | list order in for-loop | WIRED | `publish.yml:246` |

### Data-Flow Trace (Level 4)

Phase 166 ships library functions and tests, not UI components rendering dynamic data. Level 4 data-flow trace not applicable — the functions (`complete`, `for_structured_output`, `dispatch`) are the data transformation themselves.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `ferro-ai` unit tests (all 80) | `cargo test -p ferro-ai` | 80 passed, 0 failed | PASS |
| SC#3 projection_schema integration tests (6) | `cargo test -p ferro-ai` | 6 passed, 0 failed | PASS |
| SC#5 iteration cap enforced | `tool_registry_enforces_max_iterations` | `ToolIterationLimit(3)` returned | PASS |
| WR-02 warn@5 guard ordering | loop guard: `iteration == 5 && self.max_iterations > 5` before cap check | correct ordering in code | PASS |
| CR-01 Anthropic tool_result wire format | `test_build_body_tool_result_wire_format` | `tool_use_id` is real field | PASS |
| CR-02 assistant turn before tool results | `dispatch_includes_assistant_turn_before_tool_results` | assistant_pos < tool_result_pos | PASS |
| CR-03 OpenAI tool_call_id as real field | `test_build_body_tool_result_wire_format` (openai) | `tool_call_id` top-level | PASS |
| WR-01 OpenAI tool_choice honored | `test_build_body_tool_choice_none` | emits `"none"` | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| AISDK-02 | 166-01, 166-02, 166-03 | Typed `complete::<T>()` + schema normalizer + ServiceDef-aware path | SATISFIED | `complete.rs`, `schema/mod.rs`, `projection_schema.rs` — all three layers implemented and tested. |
| AISDK-03 | 166-04 | Tool registration + `max_iterations` bounded dispatch | SATISFIED | `tools/mod.rs` — `ToolDef`, `ToolRegistry`, dispatch loop with hard cap. All SC#4/5/6 tests pass. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-ai/src/error.rs` | 57 | `ToolNotFound` variant defined but never constructed | Info | Documented with explicit WR-03 doc comment explaining reservation for future `dispatch_single`. Regression test `dispatch_surfaces_unknown_tool_as_tool_error` confirms dispatch uses ToolError (not ToolNotFound). No impact on SC#6. |
| `ferro-ai/src/schema/mod.rs` | 71-79 | `PROJECTION_DEF_NAMES` trigger list includes 5 names; closing list is only `FieldMeaning`/`Intent` | Info | Documented with IN-02 multi-line comment distinguishing trigger vs closed list. Design is intentional — trigger list is a superset. |

No blockers or warnings found. Both info items are documented and intentional.

### Human Verification Required

None. All 7 success criteria are verifiable programmatically and the test suite is green.

### Gaps Summary

No gaps. All 7 ROADMAP success criteria verified in the actual codebase:

1. `complete::<T>()` — exists, correct signature, re-exported, tested.
2. `for_structured_output` — full implementation: ref inlining, keyword stripping, `additionalProperties:false`, `enum` preserved.
3. ServiceDef-aware path — vocabulary derived from schemars output (not hardcoded), closing fires before inlining, invalid values fail jsonschema validation.
4. `ToolDef` — all four fields present.
5. `ToolRegistry::dispatch` — no `Default`, no zero-arg ctor, warn@5 fires correctly (WR-02 fix landed and confirmed), hard cap enforced.
6. `ToolError` — model-legible, surfaced correctly.
7. Full suite green — 80 unit + 6 integration tests pass.

All code-review fixes (CR-01, CR-02, CR-03, WR-01, WR-02, WR-03, IN-01, IN-02) are present in the codebase with regression tests.

---

_Verified: 2026-06-08T06:30:00Z_
_Verifier: Claude (gsd-verifier)_
