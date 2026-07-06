# Phase 89: Intent Graph Generation - Research

**Researched:** 2026-03-01
**Domain:** Structural analysis heuristics — pattern matching on ServiceDef signals to ranked intents with confidence scores
**Confidence:** MEDIUM (novel approach, validated architecture pattern, needs empirical validation in Phase 93)

<research_summary>
## Summary

Researched how to build the structural analysis engine that derives intent from ServiceDef schema shape. This is Ferro's core innovation — no existing system computes UI intent from structural analysis of a pre-existing data schema without manual annotation or LLM reasoning.

The closest operational precedents are **Metabase's column classifier pipeline** (sequential classifiers examining field name + fingerprint data → semantic type) and **Refine.dev's Inferencer** (API response analysis → CRUD component selection with priority-based type inference). Both validate that structural classification from metadata works in production, but neither attempts domain-intent derivation beyond CRUD.

The recommended architecture is a **signal extractor pipeline with weighted score aggregation**: independent analyzer functions each examine one structural aspect of ServiceDef, contribute weighted scores per intent, then scores are aggregated and normalized. This follows the established rules engine design pattern (composable condition-action units) adapted for multi-class scoring.

**Primary recommendation:** Implement 5 composable signal analyzers (field meanings, writability, state machine, relationships, actions), a weighted score aggregator with normalization to [0.0, 1.0], and IntentHint override application. Validate against 10+ representative ServiceDefs with >70% primary intent accuracy target.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ferro-projections (existing) | workspace | All types already defined | Phases 84-88 built the complete type surface |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none needed) | - | - | Phase 89 is pure Rust logic over existing types |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled heuristics | zen-engine (GoRules) | zen-engine is for externalized business rules with JSON decision models — overkill for 5 analyzers with compile-time logic |
| Hand-rolled heuristics | intent-classifier crate | NLP-focused (text classification) — wrong domain; Ferro classifies schema structure, not text |
| Weighted sum scoring | Bayesian inference | Bayesian requires prior probability distributions we don't have; weighted sum is simpler and sufficient for 7 intents |
| Custom analyzers | ML classifier | Requires training data we don't have; heuristics are interpretable and debuggable |

**No new dependencies required.** Phase 89 is pure algorithmic logic over existing ferro-projections types.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Architecture: Signal Extractor Pipeline

```
ServiceDef → [Signal Extractors] → Raw Scores → [Aggregator] → [IntentHint Override] → Vec<IntentScore>
```

Each stage is independent and testable:

```
┌─────────────┐
│  ServiceDef │
└──────┬──────┘
       │
       ├──→ FieldMeaningAnalyzer  → {Browse: 0.3, Summarize: 0.4, ...}
       ├──→ WritabilityAnalyzer   → {Collect: 0.5, Focus: 0.2, ...}
       ├──→ StateMachineAnalyzer  → {Process: 0.8, Track: 0.6, ...}
       ├──→ RelationshipAnalyzer  → {Browse: 0.4, Focus: 0.3, ...}
       └──→ ActionAnalyzer        → {Process: 0.3, Collect: 0.2, ...}
                    │
                    ▼
           ┌───────────────┐
           │  Aggregator   │  Sum → Normalize → Sort
           └───────┬───────┘
                   │
                   ▼
           ┌───────────────┐
           │ IntentHint    │  Primary forces top, Exclude removes
           │ Override      │
           └───────┬───────┘
                   │
                   ▼
           Vec<IntentScore>  (ranked by confidence, 0.0-1.0)
```

### Pattern 1: Composable Signal Analyzers

**What:** Each analyzer is a pure function `fn(&ServiceDef) -> HashMap<Intent, f64>` examining one structural dimension. Analyzers are independent — adding or removing one doesn't affect others.

**Why this pattern:**
- Metabase uses this exact approach: 4 independent classifiers (name, category, text-fingerprint, no-preview-display), each examining one signal
- Refine.dev uses priority-based inference where multiple matchers run independently
- Rules engine pattern: composable condition-action units with single responsibility
- Testable in isolation — each analyzer has its own test suite

**Example:**
```rust
/// Contribution scores from a single analyzer.
/// Keys are intents, values are raw (unnormalized) confidence contributions.
type SignalScores = HashMap<Intent, f64>;

/// Analyzes field meanings to derive intent signals.
fn analyze_field_meanings(service: &ServiceDef) -> SignalScores {
    let mut scores = HashMap::new();

    let meanings: Vec<&FieldMeaning> = service.fields.iter()
        .map(|f| &f.meaning)
        .collect();

    // Money/Percentage/Quantity → Summarize
    let measure_count = meanings.iter().filter(|m| matches!(m,
        FieldMeaning::Money | FieldMeaning::Percentage | FieldMeaning::Quantity
    )).count();
    if measure_count > 0 {
        scores.insert(Intent::Summarize, 0.3 * measure_count as f64);
    }

    // FreeText/ImageUrl/Url → Focus (rich content)
    let content_count = meanings.iter().filter(|m| matches!(m,
        FieldMeaning::FreeText | FieldMeaning::ImageUrl | FieldMeaning::Url
    )).count();
    if content_count > 0 {
        scores.insert(Intent::Focus, 0.25 * content_count as f64);
    }

    // EntityName → Browse (list-worthy)
    if meanings.iter().any(|m| matches!(m, FieldMeaning::EntityName)) {
        *scores.entry(Intent::Browse).or_default() += 0.2;
    }

    // DateTime + numeric → Analyze (time-series)
    let has_datetime = meanings.iter().any(|m| matches!(m,
        FieldMeaning::DateTime | FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt
    ));
    let has_numeric = meanings.iter().any(|m| matches!(m,
        FieldMeaning::Money | FieldMeaning::Percentage | FieldMeaning::Quantity
    ));
    if has_datetime && has_numeric {
        scores.insert(Intent::Analyze, 0.3);
    }

    // Status → Track
    if meanings.iter().any(|m| matches!(m, FieldMeaning::Status)) {
        scores.insert(Intent::Track, 0.2);
    }

    scores
}
```

### Pattern 2: Weighted Score Aggregation with Normalization

**What:** Sum raw scores from all analyzers per intent, then normalize the highest score to 1.0 and scale others proportionally.

**Why this pattern:**
- Simple, interpretable, debuggable
- No need for calibrated probabilities — relative ranking matters more than absolute values
- Avoids Bayesian complexity (no prior distributions needed)
- Used in weighted scoring models across product/engineering

**Normalization approach:** Divide all scores by the maximum score. This ensures the top intent always has confidence 1.0, and others are relative to it. Cap minimum at 0.0.

```rust
fn normalize_scores(raw: HashMap<Intent, f64>) -> Vec<IntentScore> {
    let max_score = raw.values().cloned().fold(0.0_f64, f64::max);
    if max_score <= 0.0 {
        return vec![IntentScore {
            intent: Intent::Focus,  // safest default
            confidence: 0.5,
            matching_signals: vec!["no_structural_signals".into()],
        }];
    }

    let mut scores: Vec<IntentScore> = raw.into_iter()
        .map(|(intent, score)| IntentScore {
            intent,
            confidence: (score / max_score).min(1.0),
            matching_signals: vec![],  // populated by analyzers
        })
        .collect();

    scores.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    scores
}
```

### Pattern 3: IntentHint Override as Post-Processing

**What:** After scoring and ranking, apply IntentHint overrides:
- `Primary(intent)` → force that intent to position 0 with confidence 1.0
- `Exclude(intent)` → remove from results entirely

**Why post-processing:** Keeps hint logic separate from derivation logic. The derivation engine runs the same way regardless of hints. Hints are a transparency layer — you can always see what the engine WOULD have derived without hints.

### Pattern 4: Signal Tracking for Explainability

**What:** Each analyzer returns not just scores but the names of signals that fired. These populate `IntentScore.matching_signals`.

**Why:** Debugging and transparency. When intent derivation is wrong, matching_signals tells you WHY the engine chose what it did. Essential for the Phase 93 validation: if the engine derives Browse for an analytics dashboard, the signals list shows exactly which structural features misled it.

### Recommended Module Structure

```
ferro-projections/src/
├── derive.rs           # Public API: derive_intents(service) -> Vec<IntentScore>
├── derive/
│   ├── mod.rs          # Aggregator + IntentHint override + normalize
│   ├── fields.rs       # FieldMeaning signal analyzer
│   ├── writability.rs  # Readable/writable ratio analyzer
│   ├── state.rs        # State machine shape analyzer
│   ├── relationships.rs # Relationship cardinality analyzer
│   └── actions.rs      # Action pattern analyzer
```

Or simpler (given small graph count of 5 analyzers):
```
ferro-projections/src/
├── derive.rs           # Everything: analyzers + aggregator + normalize + override
```

**Recommendation:** Start with single file. Split only if it exceeds ~400 lines.

### Anti-Patterns to Avoid
- **Trait-based analyzer abstraction:** 5 analyzers don't justify a trait. Plain functions are simpler and the compiler can inline them.
- **Dynamic rule loading:** Rules are compile-time Rust logic, not runtime configuration. This is intentional — the heuristics ARE the product.
- **Threshold constants scattered across analyzers:** Collect all weight/threshold constants at the top of the module for easy tuning.
- **Hard-coding signal names as string literals:** Use constants or an enum for signal names to prevent typos in matching_signals.
</architecture_patterns>

<structural_signals>
## Structural Signal → Intent Mapping

The complete mapping from ServiceDef structural signals to intent classifications:

### Browse — Collection Navigation
| Signal | Source | Weight | Rationale |
|--------|--------|--------|-----------|
| has_many/many_to_many relationships | `relationships` | 0.35 | Collection implies list/browse UI |
| EntityName field present | `fields` | 0.20 | Named entities suggest browseable list |
| Many readable fields | `fields` | 0.10 | Read-heavy service = browse candidate |
| No state machine or simple one | `state_machine` | 0.05 | Workflow-free entities are often browsed |

**Canonical example:** Product catalog — many products, EntityName, has_many categories.

### Focus — Single-Entity Detail
| Signal | Source | Weight | Rationale |
|--------|--------|--------|-----------|
| FreeText fields | `fields` | 0.25 | Long-form content = detail view |
| ImageUrl/Url fields | `fields` | 0.20 | Media content = detail view |
| one_to_one relationship inward | `relationships` | 0.15 | Singular relationship = embedded detail |
| More readable than writable fields | `fields` | 0.10 | Read-emphasis = display, not capture |

**Canonical example:** Blog post — title, body (FreeText), image (ImageUrl), author (one_to_one).

### Collect — Data Capture
| Signal | Source | Weight | Rationale |
|--------|--------|--------|-----------|
| High writable field ratio (>50% writable non-system fields) | `fields` | 0.35 | Many inputs = form/wizard |
| Write-only fields present (readable=false) | `fields` | 0.20 | Password-like fields = data capture |
| Actions with multiple inputs | `actions` | 0.15 | Complex action inputs = form |
| No strong read pattern | `fields` | 0.05 | Not primarily a display service |

**Canonical example:** User registration — name, email, password (write-only), address fields.

### Process — Workflow with State Progression
| Signal | Source | Weight | Rationale |
|--------|--------|--------|-----------|
| State machine with guarded transitions | `state_machine` | 0.40 | Guards = approval/review gates |
| Actions with transition_trigger | `actions` | 0.25 | Actions that advance workflow |
| Multiple non-trivial states (>2 non-final) | `state_machine` | 0.10 | Complex lifecycle = workflow |
| Guards defined in service | `guards` | 0.05 | Preconditions = process gates |

**Canonical example:** Order processing — draft→submitted→approved→shipped→delivered with guards.

### Summarize — Overview Dashboard
| Signal | Source | Weight | Rationale |
|--------|--------|--------|-----------|
| Money/Percentage/Quantity fields (measures) | `fields` | 0.35 | Numeric measures = KPI display |
| Mostly read-only (>70% non-writable) | `fields` | 0.20 | Read-heavy = display/dashboard |
| No state machine or minimal one | `state_machine` | 0.10 | Simple lifecycle = summary focus |
| Category field for grouping | `fields` | 0.05 | Groupable data = aggregation |

**Canonical example:** Revenue report — total (Money), margin (Percentage), units_sold (Quantity).

### Analyze — Time-Series Exploration
| Signal | Source | Weight | Rationale |
|--------|--------|--------|-----------|
| DateTime fields + numeric measures | `fields` | 0.35 | Temporal + numeric = time-series |
| Multiple DateTime fields | `fields` | 0.15 | Rich temporal dimension |
| Percentage/Quantity/Money co-occurring | `fields` | 0.15 | Measurable quantities over time |
| Category for segmentation | `fields` | 0.05 | Segmented analysis |

**Canonical example:** Sales analytics — order_date (DateTime), revenue (Money), region (Category).

### Track — Timeline/Audit Trail
| Signal | Source | Weight | Rationale |
|--------|--------|--------|-----------|
| Status field present | `fields` | 0.25 | Status tracking = timeline |
| CreatedAt/UpdatedAt temporal fields | `fields` | 0.20 | Temporal ordering = timeline |
| Linear state machine (sequential, few branches) | `state_machine` | 0.20 | Linear progression = tracking |
| No heavy guard presence | `state_machine` | 0.05 | Simple progression, not approval workflow |

**Canonical example:** Shipment tracking — status (Status), shipped_at, delivered_at, carrier.

### Signal Weight Principles
1. **Weights are tuning parameters, not sacred values.** Start with these, adjust based on Phase 93 validation.
2. **No single signal should dominate.** Max weight per signal: 0.40. This prevents one feature from overwhelming others.
3. **Baseline Browse+Focus.** Every ServiceDef gets a small baseline score for Browse (0.1) and Focus (0.1) since these are always minimally applicable.
4. **Absence signals matter.** "No state machine" is a signal for Browse/Summarize. "No writable fields" is a signal against Collect.
</structural_signals>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Graph library for IntentGraph | petgraph or custom graph DS | `Vec<IntentScore>` (ranked list) | Phase 89 output is a ranked list per ServiceDef, not a graph. The "graph" comes later when connecting multiple ServiceDefs. |
| ML classifier | Training pipeline + inference | Weighted heuristics | No training data. Heuristics are interpretable and tunable. |
| Complex scoring framework | Generic scoring library | Simple HashMap<Intent, f64> | 7 intents × 5 analyzers = 35 rules max. No framework needed. |
| Calibrated probability estimation | Statistical calibration | Relative normalization (max=1.0) | We need ranking order, not calibrated probabilities. |

**What SHOULD be hand-rolled (this IS the product):**

| Problem | Why Hand-Roll |
|---------|--------------|
| Signal extraction functions | Domain-specific heuristics — no library exists for "which FieldMeaning patterns suggest Browse" |
| Score aggregation | Simple weighted sum — 20 lines of code |
| IntentHint override | Trivial post-processing — filter + reorder |
| Validation test suite | Custom ServiceDef fixtures with known expected intents |

**Key insight:** Phase 89 is ~300-500 lines of pure Rust logic. There are no external dependencies because the problem is novel — no library solves "derive UI intent from schema structure." The value is in the heuristic rules themselves, not in infrastructure.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Overfitting to Example ServiceDefs
**What goes wrong:** Heuristics tuned to the 3 test ServiceDefs fail on the 4th.
**Why it happens:** Small validation set. Weight constants adjusted to pass specific tests rather than generalize.
**How to avoid:** Build 10+ diverse ServiceDefs BEFORE writing analyzers. Include edge cases: minimal service (1 field), maximal service (all features), ambiguous service (triggers multiple intents equally).
**Warning signs:** Adding special-case logic for specific test fixtures.

### Pitfall 2: Binary Presence vs. Proportional Signals
**What goes wrong:** "Has Money field" treated as all-or-nothing, ignoring how MANY Money fields exist.
**Why it happens:** Using `any()` when `count()` would give better signal.
**How to avoid:** Prefer proportional signals over binary. "3 Money fields out of 8" is a stronger Summarize signal than "1 Money field out of 20."
**Warning signs:** Identical confidence scores for services with very different field distributions.

### Pitfall 3: State Machine Shape Ambiguity (Process vs Track)
**What goes wrong:** Every state machine triggers both Process and Track with similar confidence.
**Why it happens:** Both intents use state machine presence as a signal.
**How to avoid:** Differentiate by shape:
- Process = branching (multiple transitions from same state, guards, approval patterns)
- Track = linear (each state has exactly one outgoing transition to next state)
The discriminator is branching factor and guard density, not just presence.
**Warning signs:** Process and Track always within 0.1 confidence of each other.

### Pitfall 4: System Fields Polluting Signals
**What goes wrong:** `id`, `created_at`, `updated_at` fields (present on every ServiceDef) skew signals.
**Why it happens:** Not filtering system/infrastructure fields from analysis.
**How to avoid:** Exclude fields with meanings `Identifier`, `CreatedAt`, `UpdatedAt` from most signal calculations. These are infrastructure, not domain signals. Exception: Track analyzer SHOULD consider temporal fields.
**Warning signs:** Every service scores identically on field-based signals.

### Pitfall 5: Missing "No Signal" Default
**What goes wrong:** An empty or minimal ServiceDef produces no scores, causing a panic or empty result.
**Why it happens:** All analyzers return empty maps for trivial inputs.
**How to avoid:** Always return at least one IntentScore. Default to Focus (safest single-entity view) with low confidence and "no_structural_signals" in matching_signals.
**Warning signs:** `derive_intents()` returning empty Vec on edge cases.

### Pitfall 6: Threshold Sensitivity
**What goes wrong:** Changing one weight by 0.05 flips 30% of classifications.
**Why it happens:** Scores are clustered too close together. Small weight changes have outsized impact.
**How to avoid:** Design signal weights with clear separation between primary and secondary signals. Primary signals (state machine → Process, Money fields → Summarize) should contribute 2-3x more than secondary signals. Test sensitivity: ±10% weight variation should not change primary intent for >80% of test cases.
**Warning signs:** Multiple intents within 0.05 confidence of each other on most test cases.
</common_pitfalls>

<code_examples>
## Code Examples

### Public API: derive_intents()
```rust
use std::collections::HashMap;
use crate::{Intent, IntentHint, IntentScore, ServiceDef};

/// Derives ranked intents from a ServiceDef's structural signals.
///
/// Runs 5 independent signal analyzers, aggregates scores,
/// applies IntentHint overrides, and returns ranked IntentScores.
///
/// Always returns at least one IntentScore. Default is Focus
/// with low confidence if no structural signals are detected.
pub fn derive_intents(service: &ServiceDef) -> Vec<IntentScore> {
    // 1. Run signal analyzers
    let mut scores: HashMap<Intent, (f64, Vec<String>)> = HashMap::new();

    for (intent, weight, signal) in analyze_field_meanings(service) {
        let entry = scores.entry(intent).or_insert((0.0, Vec::new()));
        entry.0 += weight;
        entry.1.push(signal);
    }
    // ... same for other analyzers

    // 2. Normalize scores
    let mut result = normalize(scores);

    // 3. Apply IntentHint overrides
    apply_hints(&mut result, &service.intent_hints);

    // 4. Ensure non-empty
    if result.is_empty() {
        result.push(IntentScore {
            intent: Intent::Focus,
            confidence: 0.5,
            matching_signals: vec!["no_structural_signals".into()],
        });
    }

    result
}
```

### Signal Analyzer Return Type
```rust
/// A single signal contribution: (intent, raw weight, signal description).
type Signal = (Intent, f64, String);

/// Analyzes state machine shape for Process vs Track discrimination.
fn analyze_state_machine(service: &ServiceDef) -> Vec<Signal> {
    let Some(sm) = &service.state_machine else {
        return vec![];
    };

    let mut signals = Vec::new();

    // Guard density → Process
    let guarded = sm.transitions.iter().filter(|t| t.guard.is_some()).count();
    if guarded > 0 {
        let guard_ratio = guarded as f64 / sm.transitions.len().max(1) as f64;
        signals.push((
            Intent::Process,
            0.4 * guard_ratio,
            format!("{guarded}/{} guarded_transitions", sm.transitions.len()),
        ));
    }

    // Branching factor → Process (multiple transitions from same state)
    let states_with_multiple_out: usize = {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for t in &sm.transitions {
            *counts.entry(t.from.as_str()).or_default() += 1;
        }
        counts.values().filter(|&&c| c > 1).count()
    };
    if states_with_multiple_out > 0 {
        signals.push((
            Intent::Process,
            0.15,
            format!("{states_with_multiple_out}_branching_states"),
        ));
    }

    // Linear progression → Track
    let non_final = sm.states.iter().filter(|s| !s.is_final).count();
    if non_final > 2 && states_with_multiple_out == 0 {
        signals.push((
            Intent::Track,
            0.3,
            format!("{non_final}_linear_states"),
        ));
    }

    // Final states reachable → Track (lifecycle with clear end)
    let has_final = sm.states.iter().any(|s| s.is_final);
    if has_final {
        signals.push((
            Intent::Track,
            0.1,
            "has_final_states".into(),
        ));
    }

    signals
}
```

### IntentHint Override Application
```rust
fn apply_hints(scores: &mut Vec<IntentScore>, hints: &[IntentHint]) {
    // Remove excluded intents
    for hint in hints {
        if let IntentHint::Exclude(intent) = hint {
            scores.retain(|s| &s.intent != intent);
        }
    }

    // Force primary intent to top
    for hint in hints {
        if let IntentHint::Primary(intent) = hint {
            // Remove if already present
            scores.retain(|s| &s.intent != intent);
            // Insert at position 0 with max confidence
            scores.insert(0, IntentScore {
                intent: intent.clone(),
                confidence: 1.0,
                matching_signals: vec!["intent_hint_primary".into()],
            });
        }
    }
}
```

### Validation Test Pattern
```rust
#[cfg(test)]
mod validation {
    use super::*;

    /// Build a representative ServiceDef and verify primary intent derivation.
    fn assert_primary_intent(service: &ServiceDef, expected: Intent) {
        let scores = derive_intents(service);
        assert!(
            !scores.is_empty(),
            "derive_intents returned empty for '{}'", service.name
        );
        assert_eq!(
            scores[0].intent, expected,
            "Expected primary intent {:?} for '{}', got {:?} (confidence: {:.2}). \
             Signals: {:?}",
            expected, service.name, scores[0].intent,
            scores[0].confidence, scores[0].matching_signals
        );
    }

    #[test]
    fn order_management_derives_process() {
        let order = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .state_machine(StateMachine::new("lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("submitted"))
                .state(StateDef::new("approved"))
                .state(StateDef::new("completed").final_state())
                .transition(Transition::new("draft", "submit", "submitted")
                    .guard("has_items"))
                .transition(Transition::new("submitted", "approve", "approved")
                    .guard("is_manager"))
                .transition(Transition::new("approved", "complete", "completed"))
            );
        assert_primary_intent(&order, Intent::Process);
    }

    #[test]
    fn product_catalog_derives_browse() {
        let product = ServiceDef::new("product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
            .field("category", DataType::String, FieldMeaning::Category)
            .has_many("reviews", "review");
        assert_primary_intent(&product, Intent::Browse);
    }

    #[test]
    fn revenue_report_derives_summarize() {
        let report = ServiceDef::new("revenue_report")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total_revenue", DataType::Float, FieldMeaning::Money)
            .field("margin", DataType::Float, FieldMeaning::Percentage)
            .field("units_sold", DataType::Integer, FieldMeaning::Quantity)
            .read_only_field("growth_rate", DataType::Float, FieldMeaning::Percentage);
        assert_primary_intent(&report, Intent::Summarize);
    }
}
```
</code_examples>

<validation_methodology>
## Validation Methodology

### Test Suite Design (Phase 89 requirement: >70% accuracy)

**10+ representative ServiceDefs covering diverse domains:**

| # | ServiceDef | Expected Intent | Structural Signature |
|---|-----------|----------------|---------------------|
| 1 | Order Management | Process | State machine + guards + transition triggers |
| 2 | Product Catalog | Browse | has_many + EntityName + Category |
| 3 | Blog Post | Focus | FreeText + ImageUrl + Url |
| 4 | User Registration | Collect | Many writable + write-only password |
| 5 | Revenue Report | Summarize | Money + Percentage + Quantity + read-only |
| 6 | Sales Analytics | Analyze | DateTime + Money + Category |
| 7 | Shipment Tracking | Track | Status + linear state machine + temporal |
| 8 | Comment Thread | Browse | has_many + minimal fields |
| 9 | Support Ticket | Process | State machine + guards + actions |
| 10 | User Profile | Focus | ImageUrl + FreeText + one_to_one |
| 11 | Survey Form | Collect | All writable + many inputs |
| 12 | Activity Log | Track | Status + CreatedAt + no writable fields |

### Metrics

**Primary metric:** Primary intent accuracy = (correct primary intent) / (total test cases)
- Target: >70% (per roadmap requirement)
- Stretch: >85%

**Secondary metric:** Top-3 recall = (correct intent in top 3) / (total test cases)
- Target: >90%
- If top-3 recall is high but primary accuracy is low → weight tuning needed, not architectural change

**Sensitivity metric:** ±10% weight variation should not flip primary intent for >80% of test cases

### Evaluation Process
1. Build all test ServiceDefs BEFORE writing analyzers (prevents overfitting)
2. Write analyzers
3. Run against test suite
4. If <70% accuracy: analyze confusion patterns, adjust weights
5. If still <70% after tuning: consider adding new signal dimensions or splitting ambiguous intents
6. Document results in plan execution notes
</validation_methodology>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual intent specification (SAP OData annotations) | Still manual in SAP Fiori 2025 | Unchanged | Ferro's structural derivation is still novel |
| LLM-driven UI selection (A2UI) | A2UI v0.9 — agent selects from catalog | 2025 | Validates catalog approach but still LLM-driven |
| Fixed CRUD inference (Refine.dev) | Refine v5 — still list/show/create/edit | 2025 | Proves field inference works, but taxonomy is fixed |
| Metabase column fingerprinting | Unchanged — 4 classifiers pipeline | Stable | Validates composable classifier architecture |

**The gap remains:** No production system derives domain-level intent (beyond CRUD) from structural schema analysis without manual annotation or LLM reasoning.

**Closest operational precedents:**
- **Metabase classify.clj:** Sequential classifiers (name → no-preview-display → category → text-fingerprint). Each classifier examines one signal dimension. Architecture validated for Ferro's approach.
- **Refine.dev Inferencer:** Priority-based field type inference from API response data. Validates that naming convention + data shape analysis works for component selection.
- **Weighted scoring models:** Standard practice in product prioritization. Applies directly to multi-intent scoring.

**New since Phase 85.1 research:**
- No fundamentally new approaches emerged. The gap Ferro fills remains open.
</sota_updates>

<open_questions>
## Open Questions

1. **Should system fields (id, created_at, updated_at) be excluded from ALL analyzers?**
   - What we know: These fields appear on every ServiceDef and would skew signals identically
   - What's unclear: Whether created_at/updated_at should count for Track intent (temporal ordering)
   - Recommendation: Exclude from field meaning analyzer. Include in Track analyzer only. Create a helper `is_system_field()` predicate.

2. **What happens when two intents tie in confidence?**
   - What we know: With 5 analyzers and weighted scoring, ties are possible
   - What's unclear: Whether tie-breaking should favor a specific intent
   - Recommendation: Tie-break by stable ordering: Process > Track > Collect > Browse > Focus > Summarize > Analyze. This ordering reflects implementation specificity (Process is more specific than Browse).

3. **Should the default baseline (Browse: 0.1, Focus: 0.1) exist?**
   - What we know: Every entity is minimally browseable and focusable
   - What's unclear: Whether these baselines help or just add noise
   - Recommendation: Include them. They prevent "no signal" edge cases and reflect reality. But keep weights low (0.1) so they never dominate over real signals.

4. **Single-file or multi-file module structure?**
   - What we know: ~300-500 lines estimated. 5 analyzers + aggregator + tests.
   - What's unclear: Whether tests will push it over manageable single-file size
   - Recommendation: Start single-file (`derive.rs`). Split at plan time if the plan exceeds 400 lines of production code.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Metabase classify.clj ([GitHub](https://github.com/metabase/metabase/blob/master/src/metabase/sync/analyze/classify.clj)) — Sequential classifier pipeline architecture, classifier composition pattern
- Metabase fingerprint.clj ([GitHub](https://github.com/metabase/metabase/blob/master/src/metabase/sync/analyze/fingerprint.clj)) — Fingerprinting → classification two-stage architecture
- [Metabase semantic types documentation](https://www.metabase.com/docs/latest/data-modeling/semantic-types) — Complete semantic type taxonomy, auto-detection scope
- [Refine.dev Inferencer documentation](https://refine.dev/core/docs/packages/inferencer/) — Field type inference, priority-based resolution, relationship detection heuristics
- Phase 85.1 RESEARCH.md (local) — Prior art analysis, structural intent derivation as novel contribution, intent taxonomy grounding

### Secondary (MEDIUM confidence)
- [Rules Engine Design Pattern (Nected)](https://www.nected.ai/blog/rules-engine-design-pattern) — Condition-action model, rule composition, chain of responsibility integration
- [GoRules Zen Engine (GitHub)](https://github.com/gorules/zen) — Rust-native rules engine architecture (evaluated and rejected — too heavy for this use case)
- [Weighted Scoring Model guide (daily.dev)](https://daily.dev/blog/weighted-scoring-model-guide-for-developers) — Weighted sum scoring, criterion weighting, normalization approaches
- [Google ML Classification Metrics](https://developers.google.com/machine-learning/crash-course/classification/accuracy-precision-recall) — Precision/recall evaluation methodology for classifier validation
- [Metabase cljdoc classify API](https://cljdoc.org/d/metabase-core/metabase-core/1.0.0-SNAPSHOT/api/metabase.sync.analyze.classify) — Detailed classifier function signatures and composition

### Tertiary (LOW confidence — needs validation in Phase 93)
- Structural signal → intent weight values — Estimated from domain analysis, not empirically validated
- 10% weight sensitivity target — Engineering heuristic, not backed by prior art
- Tie-breaking order — Arbitrary but deterministic; may need revision after field test
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Composable heuristic analysis engine in pure Rust
- Ecosystem: Metabase classifiers, Refine.dev Inferencer, rules engine patterns, weighted scoring
- Patterns: Signal extractor pipeline, weighted aggregation, normalization, override post-processing
- Pitfalls: Overfitting, system field pollution, Process/Track ambiguity, threshold sensitivity, empty results
- Validation: Precision/accuracy methodology for heuristic classifier evaluation

**Confidence breakdown:**
- Architecture (signal pipeline): HIGH — validated by Metabase and Refine.dev production usage
- Signal-to-intent mapping: MEDIUM — grounded in structural analysis but weight values are estimates
- Validation methodology: HIGH — standard classifier evaluation with domain-specific adaptations
- Code examples: HIGH — uses existing ferro-projections types directly
- Weight values: LOW — initial estimates requiring Phase 93 empirical tuning

**Research date:** 2026-03-01
**Valid until:** 2026-03-31 (30 days — core logic is stable; weights need empirical validation ASAP)
</metadata>

---

*Phase: 89-intent-graph-generation*
*Research completed: 2026-03-01*
*Ready for planning: yes*
