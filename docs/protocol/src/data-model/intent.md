# Intents

This page specifies three types: `Intent` (structural classification), `IntentScore` (confidence-scored derivation result), and `IntentHint` (manual override).

Intents answer "what IS this service?" based on its structural shape -- fields, relationships, state machine -- not "what can a user DO?" Each standard intent is derivable from structural signals alone, without manual annotation.

## Intent

Structurally-derivable intent classification for a service. Seven known variants cover the space of common service types, with a `Custom` escape hatch for domain-specific intents.

> **JSON Schema:** [`intent.json`](../appendix/json-schema.md)

### Variants

| Variant | JSON Value | Structural Signals | Description |
|---------|-----------|-------------------|-------------|
| `Browse` | `"browse"` | OneToMany relationships, EntityName fields, Category fields | Exploring collections of entities |
| `Focus` | `"focus"` | FreeText/ImageUrl/Url fields, OneToOne+Inline relationships | Viewing a single entity in detail |
| `Collect` | `"collect"` | High proportion of writable fields, write-only fields | Gathering data through forms |
| `Process` | `"process"` | Guarded transitions, transition triggers, branching states | Advancing entities through workflows |
| `Summarize` | `"summarize"` | Read-only Money/Percentage/Quantity fields | Presenting aggregated/metric views |
| `Analyze` | `"analyze"` | DateTime + numeric measure fields combined | Examining data patterns and trends |
| `Track` | `"track"` | Status fields, linear state progression, final states | Monitoring entity status over time |

### Custom Variant

| Variant | JSON Value | Description |
|---------|-----------|-------------|
| `Custom(String)` | Any unrecognized string | Escape hatch for intents not covered by the standard set |

### Normative Rules

1. Consumers MUST recognize all 7 known variants.
2. Consumers MUST accept any string as a valid `Intent`. Unrecognized strings are `Custom` values.
3. Known variants MUST be serialized as their `snake_case` form. `Custom` values serialize as plain strings (untagged), not as `{"custom": "..."}`.
4. During deserialization, known variant names MUST match before the `Custom` fallback. For example, `"browse"` MUST deserialize as `Browse`, never as `Custom("browse")`.
5. `Custom(String)` MUST remain the last variant in implementations to ensure correct deserialization order.
6. Each standard intent MUST be derivable from structural signals alone. No manual annotation is required for the 7 known intents.
7. `Custom` intents are NOT structurally derivable. They are assigned through [`IntentHint`](#intenthint) overrides or application-specific logic.

### JSON Example

```json
"process"
```

```json
"dashboard"
```

In the second example, `"dashboard"` is a `Custom` intent because it does not match any known variant.

---

## IntentScore

A scored intent with confidence and the structural signals that contributed to the classification. Produced by the structural analysis engine.

> **JSON Schema:** [`intent-score.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `intent` | [`Intent`](#intent) | Yes | -- | The classified intent |
| `confidence` | `number` (f64) | Yes | -- | Confidence score in the range [0.0, 1.0] |
| `matching_signals` | `string[]` | Yes | `[]` | Structural signals that contributed to this classification |

### Normative Rules

1. `confidence` MUST be a number in the range [0.0, 1.0] inclusive.
2. `matching_signals` describes what structural properties contributed to the score. Signal format is implementation-defined.
3. `IntentScore` does NOT implement equality comparison because `confidence` is a floating-point value.
4. Intent derivation MUST return at least one `IntentScore`. When no structural signals produce a result, implementations SHOULD return a default `Focus` intent with 0.5 confidence as a fallback.
5. Results SHOULD be sorted by confidence descending. The first element is the primary intent.
6. When two intents have equal confidence, implementations MUST apply a stable tie-breaking order: `Process` > `Track` > `Collect` > `Browse` > `Focus` > `Summarize` > `Analyze` > `Custom`.

### JSON Example

Intent derivation output for an order service:

```json
[
  {
    "intent": "process",
    "confidence": 0.85,
    "matching_signals": [
      "guard_density_ratio: 0.4",
      "transition_triggers: 0.25",
      "branching_states: 0.15"
    ]
  },
  {
    "intent": "browse",
    "confidence": 0.45,
    "matching_signals": [
      "one_to_many_relationships: 0.35",
      "baseline: 0.1"
    ]
  },
  {
    "intent": "focus",
    "confidence": 0.1,
    "matching_signals": [
      "baseline: 0.1"
    ]
  }
]
```

---

## IntentHint

Manual override for intent derivation when structural analysis is insufficient or incorrect. Hints allow service authors to guide the derivation engine.

> **JSON Schema:** [`intent-hint.json`](../appendix/json-schema.md)

### Variants

| Variant | JSON Structure | Description |
|---------|---------------|-------------|
| `Primary(Intent)` | `{"primary": "browse"}` | Force this intent as the top classification |
| `Exclude(Intent)` | `{"exclude": "process"}` | Remove this intent from consideration entirely |

`IntentHint` uses externally tagged serde -- each variant is a JSON object with a single key.

### Normative Rules

1. `Primary(X)` MUST set intent `X` as the highest-confidence result, regardless of structural signals.
2. `Exclude(X)` MUST remove intent `X` from the derivation output entirely.
3. Specifying both `Primary(X)` and `Exclude(X)` for the same intent `X` SHOULD produce a conflicting-hints warning.
4. Specifying multiple `Primary` hints SHOULD produce a warning. When multiple `Primary` hints exist, the first one takes precedence.
5. `IntentHint` MAY reference `Custom` intents. For example, `{"primary": "dashboard"}` forces a custom intent as primary.

### JSON Example

Configuration and derivation result with hints:

**Input: IntentHint array on ServiceDef**

```json
[
  { "primary": "browse" },
  { "exclude": "process" }
]
```

**Effect:** The derivation engine forces Browse as the primary intent and removes Process from results, regardless of structural signals.

---

## Intent Derivation Summary

The derivation engine uses five analyzers to compute `IntentScore[]` from a `ServiceDef`. Each analyzer examines a different structural dimension:

1. **Field meaning analyzer** -- Examines `FieldMeaning` variants to signal Summarize (Money/Percentage/Quantity), Focus (FreeText/ImageUrl/Url), Browse (EntityName/Category), Analyze (DateTime + numeric), and Track (Status).
2. **Writability analyzer** -- Examines `readable`/`writable` ratios to signal Collect (mostly writable), Summarize (mostly non-writable), and Focus (mostly readable).
3. **State machine analyzer** -- Examines guard density, branching, and progression to signal Process (complex workflows) and Track (linear workflows).
4. **Relationship analyzer** -- Examines cardinality and navigation to signal Browse (collection relationships) and Focus (detail relationships).
5. **Action analyzer** -- Examines transition triggers, input counts, and preconditions to signal Process (workflow actions) and Collect (data-gathering actions).

The full derivation algorithm is specified in the [Intent Derivation](../derivation.md) chapter.
