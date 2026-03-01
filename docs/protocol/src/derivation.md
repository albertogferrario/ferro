# Intent Derivation

This section specifies the rules for deriving user intents from a `ServiceDef`. Intent derivation is the core transformation that answers: "Given this service definition, what meaningful interactions exist?"

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this section are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

## Overview

The derivation function takes a `ServiceDef` as input and produces a ranked list of `IntentScore` values:

```
derive_intents(service: ServiceDef) -> Vec<IntentScore>
```

**Guarantees:**

- The result MUST contain at least one `IntentScore`.
- If no structural signals are detected, implementations MUST return `Focus` with confidence `0.5` as the default fallback.
- Results MUST be sorted in descending order by confidence.

## Analyzer Pipeline

Implementations MUST run five analyzers in sequence, each contributing zero or more signals. A signal is a tuple of `(Intent, weight, description)` where the weight is implementation-defined.

The signal types described below (what each analyzer examines and which intents those signals map to) are **normative**. The exact numeric weights assigned to each signal are **informative** and implementation-specific. Different implementations MAY use different weighting strategies while remaining conformant, provided the signal-to-intent mappings are preserved.

### Field Meaning Analyzer

This analyzer examines `FieldMeaning` variants on non-system fields to derive intent signals.

**System field exclusion:** Fields with meanings `Identifier`, `CreatedAt`, or `UpdatedAt` MUST be excluded from analysis. These are infrastructure fields that do not contribute to domain intent signals.

The following signal mappings MUST be implemented:

| FieldMeaning Variant(s) | Target Intent | Signal Category |
|--------------------------|---------------|-----------------|
| `Money`, `Percentage`, `Quantity` | Summarize | Numeric aggregation fields indicate summarization |
| `FreeText`, `ImageUrl`, `Url` | Focus | Content-rich fields indicate detail viewing |
| `EntityName` | Browse | Named entities indicate list browsing |
| `DateTime` + any numeric field co-occurrence | Analyze | Temporal + numeric combination indicates analytical views |
| `Status` | Track | Status fields indicate progress tracking |
| `Category` | Browse | Categorical fields indicate filtered browsing |

Implementations SHOULD use proportional signals (scaling with the count of matching fields) rather than binary presence/absence detection.

### Writability Analyzer

This analyzer examines the ratio of readable to writable fields across non-system fields.

The following signal mappings MUST be implemented:

| Condition | Target Intent | Rationale |
|-----------|---------------|-----------|
| Majority of fields are writable (>50%) | Collect | High writable ratio indicates data collection |
| Write-only fields present (writable but not readable) | Collect | Write-only fields are strong data input signals |
| Majority of fields are non-writable (>70%) | Summarize | Read-heavy services indicate summarization |
| More readable fields than writable fields | Focus | Read-dominant services indicate detail viewing |

### State Machine Analyzer

This analyzer MUST be invoked only when the `ServiceDef` contains a `state_machine`. It examines the shape of the state machine to discriminate between `Process` (complex, branching workflows) and `Track` (linear, temporal progression).

**Process signals** (branching and guards):

| Signal | Condition | Rationale |
|--------|-----------|-----------|
| Guard density | Guarded transitions exist | Conditional transitions indicate decision-driven workflows |
| Branching states | States with more than one outgoing transition | Branching indicates non-trivial process flow |
| Transition triggers | Actions have `transition_trigger` values | Explicit triggers indicate workflow-driven actions |
| Workflow states | More than 2 non-final states | Multiple intermediate states indicate complex processes |

**Track signals** (linear and temporal):

| Signal | Condition | Rationale |
|--------|-----------|-----------|
| Linear progression | More than 2 non-final states AND no branching states | Linear state chains indicate status tracking |
| Final states | At least one state is marked `is_final` | Terminal states indicate trackable completion |
| Unguarded progression | No guards on any transitions | Unconditional progression indicates simple tracking |

### Relationship Analyzer

This analyzer MUST be invoked only when the `ServiceDef` contains relationships. It examines relationship cardinalities and navigation hints.

The following signal mappings MUST be implemented:

| Condition | Target Intent | Rationale |
|-----------|---------------|-----------|
| `OneToMany` or `ManyToMany` cardinality | Browse | Collection relationships indicate list views |
| `OneToOne` with `Inline` navigation | Focus | Inline one-to-one indicates embedded detail |
| `ManyToOne` cardinality | Focus | Parent references indicate detail context |
| More than 3 total relationships | Browse | Rich relationship graphs indicate navigational browsing |

### Action Analyzer

This analyzer MUST be invoked only when the `ServiceDef` contains actions. It examines action patterns to discriminate between workflow-driven intents and data-oriented intents.

The following signal mappings MUST be implemented:

| Condition | Target Intent | Rationale |
|-----------|---------------|-----------|
| Actions with `transition_trigger` | Process | State-transitioning actions indicate workflow |
| Actions with more than 2 inputs | Collect | Complex input actions indicate data collection |
| Actions with preconditions | Process | Guarded actions indicate decision-driven process |
| Actions present but no triggers and no preconditions | Browse | Simple CRUD actions indicate basic browsing |

## Signal Aggregation

After all analyzers have contributed their signals, implementations MUST aggregate signals per intent. The aggregation strategy (sum, weighted average, max, or other) is implementation-defined.

The reference implementation uses additive scoring: all signal weights for the same intent are summed.

## Baseline Signals

`Browse` and `Focus` SHOULD receive small baseline scores to ensure they appear in results when no strong signals override them. This reflects the reality that most services support basic browsing and detail viewing regardless of other structural characteristics.

## Normalization

After aggregation, implementations MUST normalize confidence scores to the `[0.0, 1.0]` range. The normalization strategy is implementation-defined.

The reference implementation divides each intent's raw score by the maximum raw score, so the highest-scoring intent always has confidence `1.0`.

## Tie-Breaking

When two or more intents have equal confidence after normalization, implementations MUST use stable ordering based on the following priority (lower number = higher priority):

| Priority | Intent |
|----------|--------|
| 0 | `Process` |
| 1 | `Track` |
| 2 | `Collect` |
| 3 | `Browse` |
| 4 | `Focus` |
| 5 | `Summarize` |
| 6 | `Analyze` |
| 7 | `Custom(String)` |

This ordering reflects the principle that more specific intents (workflow-driven) take precedence over more general intents (browsing, viewing) when structural evidence is equally balanced.

## IntentHint Override

`ServiceDef` MAY include `IntentHint` values that override the structural derivation results.

**`IntentHint::Primary(intent)`:** The specified intent MUST be boosted to the top of the results with maximum confidence. Implementations MUST place this intent at position 0 in the output with confidence `1.0`.

**`IntentHint::Exclude(intent)`:** The specified intent MUST be removed from the results entirely. If the excluded intent would have been the only result, the default fallback (Focus at 0.5) applies.

IntentHint overrides are applied after aggregation and normalization but before the empty-result fallback check.
