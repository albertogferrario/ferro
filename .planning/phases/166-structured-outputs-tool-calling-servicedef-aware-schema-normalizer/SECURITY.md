# SECURITY.md — Phase 166: Structured Outputs, Tool Calling, ServiceDef-Aware Schema Normalizer

**Phase:** 166 — structured-outputs-tool-calling-servicedef-aware-schema-normalizer
**ASVS Level:** 1
**Audit Date:** 2026-06-08
**Auditor:** gsd-security-auditor (claude-sonnet-4-6)

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-166-DEP-01 | Tampering/DoS | mitigate | CLOSED | ferro-ai/Cargo.toml:24 depends on ferro-projections; ferro-projections/Cargo.toml has no ferro-ai dep (confirmed by grep). No cycle. |
| T-166-DEP-02 | Info disclosure | accept | CLOSED | error.rs:42-58: SchemaError(String), ToolIterationLimit(u32), ToolNotFound(String) carry structural metadata only (counts, names). No secrets, no provider response bodies. Accepted-risk rationale holds. |
| T-166-SCHEMA-01 | DoS (infinite recursion) | mitigate | CLOSED | schema/mod.rs:229-230: `visited: HashSet<String>` passed to resolve_refs; schema/mod.rs:253-256: cycle guard returns `{"type":"object"}` placeholder on re-entry. |
| T-166-SCHEMA-02 | Tampering (constraint bypass) | mitigate | CLOSED | schema/mod.rs:157-168: STRIP_KEYWORDS does NOT contain "enum"; schema/mod.rs:543: `schema_normalizer_preserves_enum` test in mod tests. |
| T-166-PI-01 | Tampering/prompt-injection | mitigate | CLOSED | schema/mod.rs:104-151: `close_projection_enum` drops Custom(String) open branch from LLM-facing schema; tests/projection_schema.rs:26-58: SC#3 test (`servicedef_schema_rejects_invalid_field_meaning`) proves structural enforcement. |
| T-166-PI-02 | Spoofing (invalid value passes) | mitigate | CLOSED | tests/projection_schema.rs:26-58, 83-109: `servicedef_schema_rejects_invalid_field_meaning` (asserts `is_err()` for "totally_bogus") and `servicedef_schema_rejects_invalid_intent` use `jsonschema::draft202012::new`. Non-vacuous — valid values also tested and must pass. |
| T-166-PI-03 | Info disclosure | accept | CLOSED | complete.rs:80: `serde_json::from_str::<T>(&text).map_err(|e| Error::Deserialization(e.to_string()))` — serde parse message only (offset/expected token), not provider secrets. Accepted-risk rationale holds. |
| T-166-01 | DoS (runaway loop/cost) | mitigate | CLOSED | tools/mod.rs:135-145: `ToolRegistry::new(max_iterations)` is only constructor; no Default impl confirmed (grep returned nothing); tools/mod.rs:225-239: `Error::ToolIterationLimit` returned at hard cap; tools/mod.rs:228-232: `warn!` at iteration 5; tools/mod.rs:402-415: `tool_registry_enforces_max_iterations` test asserts `matches!(result, Err(Error::ToolIterationLimit(3)))`. |
| T-166-02 | Info disclosure (error leak to model) | mitigate | CLOSED | tools/mod.rs:37-47: `ToolError { message: String }` sole boundary; tools/mod.rs:190-199: `result_to_message` surfaces only `te.message`; tools/mod.rs:444-490: `dispatch_surfaces_tool_error` asserts content contains "order not found" and does NOT contain "panicked at"; tools/mod.rs:326-335: `tool_error_is_model_legible` asserts `format!("{err}") == "domain message"`. |
| T-166-03 | Tampering (untrusted LLM input) | accept | CLOSED | tools/mod.rs:62-63: ToolDef.parameters_schema rustdoc states "handler implementations are responsible for validating their own inputs before privileged actions (T-166-03)". SDK passes raw `serde_json::Value`, executes no privileged action. Accepted-risk rationale holds. |
| T-166-04 | Info disclosure (API key in error) | mitigate | CLOSED | client/anthropic.rs:196-202, 300-306: `Error::Provider { status: Some(status), message: text }` where `text` = provider HTTP response body only; API key is in `x-api-key` header, never in the `message` field. Same pattern in client/openai.rs. error.rs:16: Provider.message doc comment states "Must not contain the API key or auth header." |
| T-166-PUB-01 | DoS (broken release pipeline) | mitigate | CLOSED | .github/workflows/publish.yml:246: `WAVE1B_CRATES="ferro-projections ferro-ai ferro-stripe..."` — ferro-projections precedes ferro-ai in the publish loop order. |
| T-166-GATE-01 | Tampering (regression) | mitigate | CLOSED | 166-05-SUMMARY.md confirms `cargo test --all-features` exits 0; `cargo clippy --all --all-targets -- -D warnings` clean; `cargo test -p ferro-ai classifier` 8/8 green (SC#7). All Phase 166 tests present and green. |

---

## Accepted Risks Log

| Threat ID | Rationale |
|-----------|-----------|
| T-166-DEP-02 | Error variants SchemaError, ToolIterationLimit, ToolNotFound carry only structural metadata (counts, names). No user data, no provider secrets, no API keys. The variants are used for control flow only and are not serialized to any external surface. |
| T-166-PI-03 | `Error::Deserialization(e.to_string())` carries serde's parse message (byte offset, expected token type). No provider response content, no API key, no user PII. Consistent with the pre-existing classifier error path. |
| T-166-03 | The SDK is a library — it passes raw LLM-generated `serde_json::Value` to registered handler closures and executes no privileged action itself. Handler implementations are documented as responsible for their own input validation. This is the correct boundary: a generic SDK cannot know what "valid" means for application-specific tool inputs. |

---

## Unregistered Threat Flags

None. SUMMARY.md `## Threat Flags` sections for Plans 01-05 all report "No threat flags" or no new trust boundary surfaces.

---

## Notes

- **T-166-DEP-01 cycle guard:** The build-time guard (cargo dependency cycle) is structural and would produce a compile error. Verified by confirming ferro-projections/Cargo.toml has no ferro-ai dependency entry.
- **T-166-SCHEMA-02 allowlist vs. denylist:** The STRIP_KEYWORDS constant is an allowlist of keywords to remove (explicit list). The `enum` keyword is intentionally absent. The `schema_normalizer_preserves_enum` test is the regression guard for this property.
- **T-166-01 no unbounded path:** `ToolRegistry` has no `Default` impl and no zero-arg `new()`. The `with_default_iterations()` convenience constructor delegates to `new(10)`. There is no way to construct a `ToolRegistry` without specifying a cap.
- **T-166-04 API key isolation:** In both Anthropic and OpenAI clients, the API key is placed in HTTP headers (`x-api-key` / `Authorization: Bearer`). Error paths read the response body text into `message` — the API key string never appears in `message`. The error.rs Provider variant doc comment reinforces this as a documented invariant.
