---
phase: 166
slug: structured-outputs-tool-calling-servicedef-aware-schema-normalizer
status: draft
nyquist_compliant: false
wave_0_complete: false
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

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 166-01-xx | 01 | 0 | AISDK-02 | — | schemars 1.x untagged-enum shape probe (resolves A1/A2) | unit | `cargo test -p ferro-ai schema_probe` | ❌ W0 | ⬜ pending |
| 166-02-xx | 02 | 1 | AISDK-02 | — | `for_structured_output` resolves `$ref`/`$defs`, adds `additionalProperties:false`, strips Anthropic-rejected keywords (SC#2) | unit | `cargo test -p ferro-ai schema::` | ❌ W0 | ⬜ pending |
| 166-03-xx | 03 | 2 | AISDK-02 | — | ServiceDef-aware path closes `FieldMeaning`/`Intent` enums; invalid value FAILS `jsonschema` validation (SC#3) | unit | `cargo test -p ferro-ai servicedef_schema` | ❌ W0 | ⬜ pending |
| 166-04-xx | 04 | 2 | AISDK-02 | — | `complete::<T>()` typed round-trip; caller never calls schemars/serde_json (SC#1) | unit | `cargo test -p ferro-ai complete` | ❌ W0 | ⬜ pending |
| 166-05-xx | 05 | 3 | AISDK-03 | T-166-01 | `ToolRegistry` `max_iterations` required, no unbounded path; warn@5, error@cap (SC#4/SC#5) | unit | `cargo test -p ferro-ai tool` | ❌ W0 | ⬜ pending |
| 166-06-xx | 05 | 3 | AISDK-03 | T-166-02 | `ToolError { message }` model-legible; no raw stack traces / DB strings leak (SC#6) | unit | `cargo test -p ferro-ai tool_error` | ❌ W0 | ⬜ pending |
| 166-07-xx | all | final | AISDK-02, AISDK-03 | — | `cargo test --all-features` green; existing `Classifier<T>` tests pass (SC#7) | suite | `cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-ai/src/schema/mod.rs` (or `schema.rs`) — unit-test module with a structural probe asserting the exact schemars 1.2.0 `anyOf` shape emitted for `FieldMeaning`/`Intent` untagged `Custom(String)` variants (resolves research assumptions A1/A2 before the closing algorithm is written)
- [ ] `ferro-ai/Cargo.toml` — add `schemars = "1"`, `ferro-projections`, `futures`; add `jsonschema` (0.46.x, already in Cargo.lock) as dev-dependency for the SC#3 validation test
- [ ] `ferro-ai/tests/` — integration test scaffold for `complete::<T>()` if a mock/stub client is used (no live network in CI)

*Existing infrastructure (`cargo test`, `serde_json`, `jsonschema` 0.46.5) covers the rest.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Anthropic structured-output acceptance of the normalized schema | AISDK-02 | Requires a real API key / network; CI runs offline | With `FERRO_AI_API_KEY` set, run a `complete::<ServiceDef>()` against a real prompt and confirm the provider accepts the normalized schema (no 400 on schema). The SC#2 unit test asserts conformance against documented constraints offline as the gate. |
| Live multi-turn tool dispatch loop against a real provider | AISDK-03 | Requires network + provider tool-use | With a key set, register a trivial tool and confirm `dispatch` completes a tool round-trip; offline the loop is unit-tested with a stub client returning a scripted `tool_use` then a final answer. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (schemars shape probe; jsonschema dev-dep)
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
