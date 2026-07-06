---
phase: 166
slug: structured-outputs-tool-calling-servicedef-aware-schema-normalizer
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-08
---

# Phase 166 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| build-time deps | New crate deps enter the workspace dependency graph; a cycle would break the build | crate dependency edges |
| schema producer → LLM provider | The normalized schema is sent to an external provider; a malformed schema causes a 400 but no data leak | JSON Schema |
| LLM output → ServiceDef | Untrusted model output is constrained by the closed schema before it becomes a ServiceDef | LLM-generated JSON |
| LLM → tool handler | LLM-generated `serde_json::Value` is passed to handler closures | untrusted tool-call args |
| tool handler → LLM | Handler results and errors are sent back to the model | handler output / `ToolError` |
| dispatch loop → provider | Repeated provider calls; an unbounded loop is a cost/availability risk | completion requests |
| CI publish pipeline | Crate publish order must satisfy the dependency DAG or first-publish fails | crate publish order |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-166-DEP-01 | Tampering/DoS | ferro-ai → ferro-projections dependency | mitigate | Leaf-direction dependency; ferro-projections has no ferro-ai dep, so no cycle. `ferro-ai/Cargo.toml:24` + ferro-projections/Cargo.toml (no reverse edge). | closed |
| T-166-DEP-02 | Information disclosure | new Error variants | accept | SchemaError/ToolIterationLimit/ToolNotFound carry structural metadata only. `error.rs:42-58`. | closed |
| T-166-SCHEMA-01 | DoS (infinite recursion) | resolve_refs on cyclic schema | mitigate | `visited: HashSet<String>` cycle guard returns `{"type":"object"}` on re-entry. `schema/mod.rs:229,253-256`. | closed |
| T-166-SCHEMA-02 | Tampering (constraint bypass) | strip pass removing `enum` | mitigate | STRIP_KEYWORDS allowlist excludes `enum`; `schema_normalizer_preserves_enum` regression test. `schema/mod.rs:157-168,543`. | closed |
| T-166-PI-01 | Tampering / prompt-injection | SD-aware closed enum | mitigate | `close_projection_enum` drops the `Custom(String)` escape hatch from the LLM-facing schema. `schema/mod.rs:104-151`; SC#3 test `tests/projection_schema.rs:26-58`. | closed |
| T-166-PI-02 | Spoofing (invalid value passes) | jsonschema enum enforcement | mitigate | `servicedef_schema_rejects_invalid_field_meaning` / `_intent` assert invalid values fail `jsonschema::draft202012`; non-vacuous. `tests/projection_schema.rs:26-58,83-109`. | closed |
| T-166-PI-03 | Information disclosure | complete::<T>() deserialization error | accept | `Error::Deserialization(e.to_string())` carries serde parse message only. `complete.rs:80`. | closed |
| T-166-01 | DoS (runaway loop / cost) | ToolRegistry::dispatch | mitigate | `max_iterations` REQUIRED at construction (no Default/zero-arg ctor); `Error::ToolIterationLimit` at cap; warn@5. `tools/mod.rs:135,228-239`; test `:402-415`. | closed |
| T-166-02 | Information disclosure (error leak to model) | tool error surfacing | mitigate | `ToolError { message }` is the only thing surfaced; handler errors mapped, never raw panics/stack traces/DB strings. `tools/mod.rs:190-199`; tests `:444-490,326-335`. | closed |
| T-166-03 | Tampering (untrusted LLM input) | handler input validation | accept | SDK passes raw `serde_json::Value` to handlers and executes no privileged action; handlers validate own inputs. Documented in ToolDef rustdoc `tools/mod.rs:62-63`. | closed |
| T-166-04 | Information disclosure (API key in error) | complete_with_tools provider errors | mitigate | Reuses `Error::Provider { status, message }` whose message carries HTTP response body only; API key lives in the auth header, never in `message`. `client/anthropic.rs:196-202`, `client/openai.rs`, `error.rs:16`. | closed |
| T-166-PUB-01 | DoS (broken release pipeline) | WAVE1B publish order | mitigate | `ferro-projections` precedes `ferro-ai` in WAVE1B; DAG satisfied. `.github/workflows/publish.yml:246`. | closed |
| T-166-GATE-01 | Tampering (regression) | full suite gate | mitigate | `cargo test --all-features` + clippy `-D warnings` phase gate; existing Classifier<T> tests green (SC#7). 166-05-SUMMARY.md. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-166-01 | T-166-DEP-02 | New Error variants (SchemaError/ToolIterationLimit/ToolNotFound) carry only structural metadata (counts, names) — no secrets, no provider response bodies. | Alberto Ferrario | 2026-06-08 |
| AR-166-02 | T-166-PI-03 | `Error::Deserialization` surfaces serde's parse message (offset/expected token), not provider secrets — consistent with the existing classifier path. | Alberto Ferrario | 2026-06-08 |
| AR-166-03 | T-166-03 | The SDK passes raw LLM `serde_json::Value` to handlers and executes no privileged action itself; handler implementations validate their own inputs. Documented in ToolDef rustdoc. | Alberto Ferrario | 2026-06-08 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-08 | 13 | 13 | 0 | gsd-security-auditor (sonnet) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-08
