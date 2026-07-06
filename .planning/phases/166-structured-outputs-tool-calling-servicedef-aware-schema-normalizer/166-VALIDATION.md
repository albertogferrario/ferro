---
phase: 166
slug: structured-outputs-tool-calling-servicedef-aware-schema-normalizer
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-08
---

# Phase 166 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in; unit tests in-crate, integration tests in `ferro-ai/tests/`) |
| **Config file** | none — Cargo workspace |
| **Quick run command** | `cargo test -p ferro-ai` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30–90 seconds (workspace incremental) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-ai`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green + `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings`
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Covering Tests | Status |
|---------|------|------|-------------|------------|-----------------|-----------|----------------|--------|
| 166-01-xx | 01 | 0 | AISDK-02 | — | schemars 1.x untagged-enum shape probe (resolves A1/A2) | unit | `schema_probe_field_meaning_any_of_shape`, `schema_probe_intent_any_of_shape` | ✅ green |
| 166-02-xx | 02 | 1 | AISDK-02 | T-166-SCHEMA-01/02 | `for_structured_output` resolves `$ref`/`$defs`, adds `additionalProperties:false`, strips Anthropic-rejected keywords (SC#2) | unit | `schema_normalizer_strips_rejected_keywords`, `schema_normalizer_resolves_refs`, `schema_normalizer_preserves_enum`, `schema_normalizer_skips_additional_properties_on_anyof`, `normalized_schema_has_no_surviving_refs` | ✅ green |
| 166-03-xx | 03 | 2 | AISDK-02 | T-166-PI-01/02 | ServiceDef-aware path closes `FieldMeaning`/`Intent` enums; invalid value FAILS `jsonschema` validation (SC#3) | unit | `servicedef_schema_rejects_invalid_field_meaning`, `servicedef_schema_rejects_invalid_intent`, `closes_field_meaning_enum`, `closes_intent_enum_const_branch_style`, `non_projection_schema_not_closed`, `servicedef_schema_accepts_*` | ✅ green |
| 166-04-xx | 04 | 2 | AISDK-02 | T-166-PI-03 | `complete::<T>()` typed round-trip; caller never calls schemars/serde_json (SC#1) | unit | `complete_returns_typed_result`, `complete_propagates_deserialization_error` | ✅ green |
| 166-05-xx | 05 | 3 | AISDK-03 | T-166-01 | `ToolRegistry` `max_iterations` required, no unbounded path; warn@5, error@cap (SC#4/SC#5) | unit | `tool_registry_requires_max_iterations`, `tool_registry_enforces_max_iterations`, `tool_def_construction`, `dispatch_returns_on_text`, `dispatch_includes_assistant_turn_before_tool_results` | ✅ green |
| 166-06-xx | 05 | 3 | AISDK-03 | T-166-02 | `ToolError { message }` model-legible; no raw stack traces / DB strings leak (SC#6) | unit | `tool_error_is_model_legible`, `dispatch_surfaces_tool_error`, `dispatch_surfaces_unknown_tool_as_tool_error` | ✅ green |
| 166-07-xx | all | final | AISDK-02, AISDK-03 | — | `cargo test --all-features` green; existing `Classifier<T>` tests pass (SC#7) | suite | `cargo test -p ferro-ai` → 80 unit + 6 integration passed, 0 failed | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

Audit run `cargo test -p ferro-ai` (2026-06-08): **86 passed, 0 failed, 0 flaky** (80 unit + 6 integration; 8 doc-tests ignored by design).

---

## Wave 0 Requirements

- [x] `ferro-ai/src/schema/mod.rs` — structural probe asserting the schemars `anyOf` shape for `FieldMeaning`/`Intent` untagged `Custom(String)` variants (`schema_probe_field_meaning_any_of_shape`, `schema_probe_intent_any_of_shape`; A1/A2 resolved)
- [x] `ferro-ai/Cargo.toml` — `schemars`, `ferro-projections`, `futures` added; `jsonschema` available as dev-dependency for the SC#3 validation tests
- [x] `ferro-ai/tests/projection_schema.rs` — integration tests for the normalized ServiceDef schema using a stub client (no live network in CI); `complete::<T>()` round-trip unit-tested in `src/complete.rs` with a stub `LlmClient`

*Existing infrastructure (`cargo test`, `serde_json`, `jsonschema`) covers the rest.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Anthropic structured-output acceptance of the normalized schema | AISDK-02 | Requires a real API key / network; CI runs offline | With `FERRO_AI_API_KEY` set, run a `complete::<ServiceDef>()` against a real prompt and confirm the provider accepts the normalized schema (no 400 on schema). The SC#2 unit test asserts conformance against documented constraints offline as the gate. |
| Live multi-turn tool dispatch loop against a real provider | AISDK-03 | Requires network + provider tool-use | With a key set, register a trivial tool and confirm `dispatch` completes a tool round-trip; offline the loop is unit-tested with a stub client returning a scripted `tool_use` then a final answer. |

---

## Validation Audit 2026-06-08

| Metric | Count |
|--------|-------|
| Requirements audited | 7 |
| COVERED (green) | 7 |
| PARTIAL | 0 |
| MISSING | 0 |
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

State A audit: all per-task requirements already mapped to implemented, passing tests. No gaps — no test generation required. Evidence: `cargo test -p ferro-ai` → 86 passed, 0 failed.

---

## Validation Sign-Off

- [x] All tasks have automated verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (schemars shape probe; jsonschema dev-dep)
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** verified 2026-06-08
