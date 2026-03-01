# Conformance

This section defines conformance levels for implementations of the Ferro Projections Protocol.

## Notational Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) / [BCP 14](https://datatracker.ietf.org/doc/html/bcp14).

## Conformance Levels

The protocol defines three conformance levels, each building on the previous:

### Level 1 -- Schema Conformant

An implementation is **Schema Conformant** if it correctly serializes and deserializes all protocol types according to the [JSON Schema definitions](appendix/json-schema.md).

**Requirements:**

- Implementations MUST produce JSON output that validates against the published JSON Schema for each type.
- Implementations MUST accept valid JSON conforming to the published schemas.
- Implementations MUST use `snake_case` for enum variant serialization.
- Implementations MUST preserve unknown fields during round-trip serialization (forward compatibility).
- Implementations MUST handle the `Custom(String)` fallback for `FieldMeaning` and `Intent` enums.

**Use case:** Validation-only tools, schema editors, documentation generators.

### Level 2 -- Derivation Conformant

An implementation is **Derivation Conformant** if it is Schema Conformant (Level 1) and implements intent derivation from `ServiceDef` structure.

**Requirements:**

- Implementations MUST implement all 5 analyzer categories:
  1. **Field meaning analysis** -- deriving intent signals from `FieldMeaning` variant distribution
  2. **Writability analysis** -- deriving intent signals from readable/writable field ratios
  3. **State machine analysis** -- deriving intent signals from state machine structure (guards, branching, transitions)
  4. **Relationship analysis** -- deriving intent signals from relationship cardinality and navigation patterns
  5. **Action analysis** -- deriving intent signals from action definitions (transition triggers, inputs, preconditions)

- Implementations MUST consider the normative signal categories documented in [Intent Derivation](derivation.md). The exact numeric weights are implementation-specific.
- Implementations MUST return at least one `IntentScore` from derivation.
- Implementations MUST respect `IntentHint::Primary` and `IntentHint::Exclude` overrides when present.
- Implementations SHOULD produce confidence scores in the range `[0.0, 1.0]`.

**Use case:** Intent derivation engines, service analysis tools, projection planning systems.

### Level 3 -- Rendering Conformant

An implementation is **Rendering Conformant** if it is Derivation Conformant (Level 2) and implements the `Renderer` trait contract.

**Requirements:**

- Implementations MUST implement the `Renderer` trait: `render(service, intents, context) -> Result<Value, Error>`.
- Implementations MUST produce valid JSON output.
- Implementations SHOULD support all 7 standard intents: Browse, Focus, Collect, Process, Summarize, Analyze, Track.
- Implementations SHOULD support the `Custom(String)` intent variant with a reasonable fallback rendering.
- Implementations MUST support both `RenderMode::Display` and `RenderMode::Input`.
- Implementations MUST handle `RenderContext` fields: `intent_index`, `current_state`, and `mode`.

**Use case:** Full projection renderers, UI generation engines, framework integrations.

## Partial Conformance

Implementations MAY be conformant at Level 1 only (e.g., a validation-only tool) or Levels 1+2 (e.g., a derivation engine without rendering). Partial conformance is valid and expected for specialized tools.

An implementation MUST NOT claim Level 2 conformance without Level 1 conformance, nor Level 3 without Level 2.

## Conformance Declaration

Implementations SHOULD declare their conformance level in documentation or metadata. The recommended format is:

```
Ferro Projections Protocol 0.1.0-draft, Level {1|2|3} Conformant
```

## Reference Implementation

The `ferro-projections` crate (Rust) serves as the reference implementation at Level 3 conformance. Its test suite (309+ tests) validates schema serialization, intent derivation accuracy, and renderer output correctness.

Future protocol versions MAY include a formal, language-independent conformance test suite.
