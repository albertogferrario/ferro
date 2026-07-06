# Phase 88: Intent Layer — Core Types — Research

**Researched:** 2026-02-28 (updated)
**Domain:** Structural intent derivation — automatically determining service purpose from ServiceDef structure
**Confidence:** HIGH

<research_summary>
## Summary

Researched two complementary domains: (1) abstract UI description systems (Cameleon, MARIA, XForms, IFML, A2UI, FDC3) for abstraction level positioning and medium-agnostic design principles, and (2) automatic visualization/UI recommendation systems (Metabase semantic types, CompassQL/Voyager, SAGE, Tableau Show Me, Power BI Insights, Snowflake semantic model) for structural derivation heuristics.

Key finding: **No existing system derives service-level purpose from structural analysis of a complete service definition.** Individual building blocks are production-proven — field semantic classification (Metabase), measure/dimension/fact classification (Snowflake/dbt), expressiveness/effectiveness ranking (APT/CompassQL), entity categorization (MDM). The composition into service-level intent is Ferro's novel contribution.

The v9.0 architecture distinguishes two layers that prior research conflated:
- **Service Intent** (Phase 88): "What IS this service?" — derived from structural signals → Browse, Focus, Collect, Process, Summarize, Analyze, Track
- **Interaction Operations** (Phase 89-90): "What can a user DO?" — CRUD + navigation, universal across all services

Phase 88 defines the service intent layer. Every intent must be derivable from ServiceDef structure without manual annotation (Architecture Principle #3). Multiple intents can apply with confidence scores (Principle #4).

**Primary recommendation:** 7 structurally-derivable intents + Custom escape hatch. Each intent maps to a distinct structural signal pattern in ServiceDef. Use multi-signal weighted scoring (proven by CompassQL/APT) for intent derivation in Phase 89.
</research_summary>

<standard_stack>
## Standard Stack

No external libraries needed. Phase 88 produces pure Rust types within `ferro-projections`. The "stack" is the conceptual framework informing type design.

### Conceptual Foundations
| Framework | Year | What It Provides | How It Applies |
|-----------|------|-----------------|----------------|
| Cameleon Reference Framework | 2003+ | 4-level UI abstraction (Task→AUI→CUI→FUI) | Positions Intent at AUI level |
| APT (Mackinlay 1986) | 1986 | Expressiveness + effectiveness ranking | Multi-signal scoring model |
| CompassQL/Voyager | 2016 | Field-type → visualization recommendation | Proven structural → intent derivation |
| SAGE Data Characterization | 1994 | Multi-dimensional data classification | Signal taxonomy for fields |
| Metabase Semantic Types | 2015+ | ~30 semantic types → auto-visualization | Validates FieldMeaning → rendering |
| Snowflake Semantic Model | 2024 | Measure/Dimension/Fact classification | Field role classification |
| MDM Entity Categories | Industry | Master/Transactional/Reference/Freeform | Service-level entity classification |
| Google A2UI | 2025 | Agent-driven UI protocol | Validates medium-agnostic design |
| FDC3 (FINOS) | 2024 | 8 intent prefixes, typed vocabulary | Validates small intent set |

### Architecture Principles (from v9.0 decisions)
| Principle | Source | Application |
|-----------|--------|-------------|
| Structural intent derivation | v9.0 Principle #1 | Intent derived from ServiceDef, not manually specified |
| Schema as protocol contract | v9.0 Principle #2 | JSON Schema from Rust types IS the protocol |
| Every intent structurally derivable | v9.0 Principle #3 | No intent that requires manual annotation |
| Confidence scores over hard selection | v9.0 Principle #4 | Ranked list, consumer picks |
| Specify semantics, not appearance | XForms/Cameleon | `Browse` not `Table`, intent not widget |
| Small vocabulary | FDC3 (8 prefixes) | Under 12 Intent variants |
| Medium-agnostic | A2UI/Cameleon | Every intent must work in HTML, voice, spatial |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Two-Layer Intent Architecture

The prior art research (v1 of this document) revealed that academic systems (MARIA, XForms, IFML) describe **user interaction patterns** — what a user does with any UI. The v9.0 architecture asks a different question: **what is this service's structural nature?**

These are two distinct layers:

```
ServiceDef structure
    ↓ (Phase 89: structural analysis)
Service Intent: "This is a Process service"     ← Phase 88 types
    ↓ (Phase 89-90: graph + rendering)
Interaction Ops: Browse list, Inspect item, Execute action  ← Universal CRUD
    ↓ (Phase 90: renderer)
Concrete UI: Kanban board with action buttons   ← Platform-specific
```

The service intent determines HOW the universal operations are presented:
- A **Process** service shows Browse as a Kanban board, Inspect shows state progression
- An **Analyze** service shows Browse as charts/graphs, Inspect shows drill-down
- A **Collect** service shows Browse minimally, emphasizes the Create form
- A **Browse** service shows Browse as a rich table/grid, Inspect as detail card

### Abstraction Level: Cameleon AUI (Service Purpose Layer)

```
Task & Domain  →  "Manage orders with approval workflow"    (ServiceDef)
Abstract UI    →  "This is a Process service"               (Intent) ← WE ARE HERE
Concrete UI    →  "Kanban board + approval actions"         (Renderer)
Final UI       →  "HTML/JSON-UI/Voice output"               (RenderOutput)
```

### Pattern 1: Intent Enum — 7 Structural Intents + Custom

Each intent answers: **"What structural signal pattern in ServiceDef triggers this?"**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// Collection navigation and master list management.
    /// Signals: has_many relationships, EntityName fields, acts as FK target.
    Browse,

    /// Single-entity deep view with rich content.
    /// Signals: FreeText/ImageUrl/Url fields, content-oriented structure.
    Focus,

    /// Data capture — form-heavy entry point.
    /// Signals: many writable fields, write_only fields, input-heavy actions.
    Collect,

    /// Workflow with state progression and gated transitions.
    /// Signals: state machine with ≥3 states, guards, approval/rejection actions.
    Process,

    /// Overview dashboard with key metrics.
    /// Signals: read-only numeric fields (Money, Percentage, Quantity), computed values.
    Summarize,

    /// Time-series and analytical exploration.
    /// Signals: DateTime/CreatedAt fields + numeric measures, high-volume transactional.
    Analyze,

    /// Timeline/audit trail — event sequence over time.
    /// Signals: Status field + temporal fields, ordered state transitions, history.
    Track,

    /// Escape hatch for intents not structurally derivable.
    Custom(String),
}
```

### Pattern 2: Structural Signal → Intent Mapping

This is the core derivation logic (implemented in Phase 89, types defined in Phase 88):

| Intent | Primary Structural Signals | MDM Category | Example Service |
|--------|---------------------------|--------------|-----------------|
| Browse | has_many rels ≥ 2, EntityName fields, FK target for others | Master data (parent) | Customer, Department, Category |
| Focus | FreeText + ImageUrl/Url fields, content lifecycle states | Master data (detail) | Article, Product, UserProfile |
| Collect | writable fields > 60%, write_only present, few rels | Transactional (entry) | Registration, Survey, Import |
| Process | state_machine with ≥3 states, guarded transitions, actions with preconditions | Workflow | Order, Application, Ticket |
| Summarize | read-only Money/Percentage/Quantity fields, few writable fields | Aggregation | Dashboard, Report, Overview |
| Analyze | CreatedAt/DateTime + Money/Quantity measures, high FK cardinality | Analytical | SalesRecord, EventLog, MetricSeries |
| Track | Status + temporal ordering, transition history, audit events | Audit/timeline | ActivityLog, StatusHistory, Changelog |

### Pattern 3: Multi-Signal Weighted Scoring

Inspired by CompassQL/APT expressiveness+effectiveness ranking:

```rust
pub struct IntentScore {
    pub intent: Intent,
    pub confidence: f64,            // 0.0 to 1.0
    pub matching_signals: Vec<String>,  // Which signals contributed
}
```

A ServiceDef can match multiple intents. The derivation engine (Phase 89) returns a ranked list:

```
OrderService: [Process(0.85), Track(0.60), Browse(0.40)]
CustomerService: [Browse(0.90), Focus(0.55)]
SalesRecord: [Analyze(0.80), Track(0.65), Summarize(0.50)]
```

The primary intent drives the default rendering. Secondary intents inform available views/tabs.

### Pattern 4: IntentHint — Manual Override

For cases where structural derivation is wrong (expected <30% per Phase 93 target):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntentHint {
    /// Override primary intent
    Primary(Intent),
    /// Exclude an intent from consideration
    Exclude(Intent),
}
```

Applied on ServiceDef:
```rust
ServiceDef::new("unusual_service")
    .intent_hint(IntentHint::Primary(Intent::Focus))
    // ... fields, actions, etc.
```

### Pattern 5: Medium-Agnostic Validation

Every intent must pass the voice test — "This service is primarily for [intent]ing":

| Intent | Voice Description | HTML Treatment | API Treatment |
|--------|------------------|----------------|---------------|
| Browse | "Here are your customers..." | Rich table/grid with filters | Paginated list endpoint |
| Focus | "Let me show you this article..." | Detail page with media | Single-resource endpoint |
| Collect | "I need some information..." | Multi-step form | POST with validation |
| Process | "This order needs your approval..." | Kanban/flow with actions | State-machine endpoint |
| Summarize | "Here's your overview..." | Dashboard cards/KPIs | Aggregation endpoint |
| Analyze | "Looking at trends..." | Charts and drill-downs | Time-series endpoint |
| Track | "Here's what happened..." | Timeline/event log | Chronological endpoint |

### Anti-Patterns to Avoid
- **CRUD as intents:** Don't use Create/Edit/Remove as intents. Every service has CRUD. Intent describes what the service IS, not what users DO.
- **Widget names as intents:** Don't use Table/Form/Chart. Intent is medium-agnostic.
- **Intents that need manual annotation:** If an intent can't be triggered by structural signals alone, it doesn't belong in the enum (Principle #3).
- **Single-signal derivation:** Don't map one field meaning to one intent. Use multi-signal weighted scoring.
- **Ignoring the Custom escape hatch:** Custom(String) exists for exactly the cases structural derivation can't handle. Don't stretch the taxonomy to cover every edge case.
</architecture_patterns>

<structural_signals>
## Available Structural Signals (Phase 87 Inventory)

ServiceDef exposes 7 signal dimensions for intent derivation:

### 1. Field Structure
| Signal | Type | Intent Relevance |
|--------|------|-----------------|
| Field count | usize | High count → Collect or Focus |
| Writable ratio | f64 | >60% writable → Collect |
| Read-only ratio | f64 | >60% read-only → Summarize |
| Write-only present | bool | → Collect (sensitive inputs) |
| is_list fields | count | → Browse (collection-oriented) |

### 2. Field Semantics (FieldMeaning)
| Signal | Intent Weight |
|--------|--------------|
| FreeText + ImageUrl/Url | Focus (content-rich) |
| Money + Percentage + Quantity | Summarize/Analyze (numeric-heavy) |
| Status | Process or Track |
| CreatedAt/DateTime + numeric | Analyze (time-series) |
| EntityName + ForeignKey dominant | Browse (reference entity) |
| Email + Phone + Url | Focus (contact/profile) |
| Sensitive fields present | Collect (security-aware input) |
| Category + Enum | Browse (filterable collection) |

### 3. State Machine Shape
| Signal | Intent Weight |
|--------|--------------|
| No state machine | Neutral (doesn't indicate Process) |
| 2 states (simple toggle) | Weak Process signal |
| ≥3 states with guards | Strong Process signal |
| Linear progression (no branching) | Track (sequential) |
| Branching with guards | Process (decision-based) |
| Final states present | Process (lifecycle completion) |

### 4. Actions & Guards
| Signal | Intent Weight |
|--------|--------------|
| No actions | Neutral |
| Actions with preconditions | Process (gated operations) |
| Actions with transition_trigger | Process (state-changing) |
| Actions without transitions | Browse/Focus (operational) |
| Many guards | Process (complex business rules) |

### 5. Relationships
| Signal | Intent Weight |
|--------|--------------|
| has_many ≥ 2 | Browse (navigation hub) |
| belongs_to dominant | Transactional (child entity) |
| OneToOne relationships | Focus (paired detail) |
| ManyToMany relationships | Browse (association hub) |
| NavigationHint::Nested | Parent with inline children |
| NavigationHint::Tab | Focus (multi-section detail) |
| No relationships | Collect or standalone |

### 6. Composite Patterns
| Pattern | Signals Combined | Derived Intent |
|---------|-----------------|----------------|
| Master entity | EntityName + has_many ≥ 2 + few own fields | Browse |
| Content entity | FreeText + ImageUrl + lifecycle states | Focus |
| Form entity | many writable + write_only + few rels | Collect |
| Workflow entity | state_machine ≥ 3 states + guarded actions | Process |
| Dashboard entity | read-only Money/Percentage + computed | Summarize |
| Analytical entity | DateTime + numeric measures + high FK count | Analyze |
| Audit entity | Status + CreatedAt + belongs_to + ordered | Track |
</structural_signals>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Intent taxonomy from scratch | Ad-hoc categories | Cross-validated against MDM, CompassQL, Cameleon | 40+ years of research on entity and data classification |
| Field semantic classification | Custom heuristics | Existing FieldMeaning enum (18 variants) | Already built in Phase 84, proven by Metabase semantic types |
| Multi-signal scoring algorithm | Custom weighting | APT/CompassQL-inspired expressiveness+effectiveness | Expressiveness first (can this intent express this service?), then effectiveness (is it the best fit?) |
| Full UIDL pipeline | Cameleon 4-level transformation | Direct Intent→Renderer mapping | Modern systems (A2UI) skip CUI level |
| Graph algorithms | Custom BFS/DFS | Simple Vec iteration with ID lookups | 5-15 nodes; complexity irrelevant at this scale |
| CRUD operation taxonomy | Interaction-level intents (Create/Edit/Remove) | Universal CRUD handled by renderer | CRUD applies to ALL services; it's not differentiating |

**Key insight:** The novel contribution is the SERVICE-LEVEL intent derivation, not the interaction taxonomy. MARIA/XForms already solved "what can a user do." The gap in the literature is "what IS this service FOR" — that's what Ferro's 7 intents answer.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Confusing Service Intent with User Operations
**What goes wrong:** Intent enum contains Create/Edit/Remove/Navigate. Every service matches every intent. Intent becomes meaningless noise.
**Why it happens:** Following academic UI description systems (MARIA, XForms) that describe interaction patterns, not service purpose.
**How to avoid:** Intent answers "what IS this service?" not "what can a user DO?" Browse means "this service exists to navigate collections," not "the user can see a list."
**Warning signs:** Every service matches 7+ intents with similar confidence scores. No differentiation between a Customer service and a SalesRecord service.

### Pitfall 2: Taxonomy Too Granular
**What goes wrong:** Intent has 15+ variants, many with overlapping structural signals. Renderers special-case each. New renderers require O(n) per intent.
**Why it happens:** Trying to capture every nuance of service behavior in the intent.
**How to avoid:** Keep under 12 variants. Merge if two intents would render the same way in most contexts. Let IntentHint handle edge cases.
**Warning signs:** Intent variants that differ only in which fields are emphasized, not in service purpose.

### Pitfall 3: Intents Not Derivable from Structure
**What goes wrong:** An intent like "Collaborate" or "Configure" sounds useful but has no structural signal pattern. Requires manual IntentHint every time.
**Why it happens:** Designing intents top-down from use cases instead of bottom-up from available signals.
**How to avoid:** For each proposed intent, write the derivation rule. If you can't express it as "when ServiceDef has [these signals]," drop it.
**Warning signs:** Phase 93 field test shows >30% of services need IntentHint to get correct primary intent.

### Pitfall 4: Single-Signal Derivation
**What goes wrong:** "Has Money field → Summarize." A Product service with a price field gets classified as Summarize instead of Focus.
**Why it happens:** Using one-to-one field→intent mapping instead of multi-signal weighted scoring.
**How to avoid:** Require ≥2 signals for any intent above 0.5 confidence. Weight signals by specificity.
**Warning signs:** Many false positives in intent derivation. Services keep getting classified by their most "interesting" field.

### Pitfall 5: Forgetting Minimal Services
**What goes wrong:** Derivation logic assumes every ServiceDef has state machines, actions, and relationships. Simple services (just fields) produce empty or broken results.
**Why it happens:** Designing for complex cases (Order with full workflow) first.
**How to avoid:** Default intent for a minimal service (just fields, no state machine, no actions, no relationships) should be Browse or Focus depending on field semantics. Test derivation against minimal ServiceDefs.
**Warning signs:** Services without state machines get Custom("unknown") as primary intent.

### Pitfall 6: Ignoring Confidence Overlap
**What goes wrong:** Treating intent derivation as single-select. A service that's 0.7 Process and 0.65 Track loses the Track dimension.
**Why it happens:** Picking the top-1 intent and discarding the rest.
**How to avoid:** Return ranked list per Architecture Principle #4. Secondary intents inform available views/tabs in the rendered UI.
**Warning signs:** Users frequently need IntentHint not because the primary is wrong, but because a secondary intent is missing.
</common_pitfalls>

<code_examples>
## Code Examples

### Intent Enum with Structural Derivation Comments
```rust
// Source: Synthesized from CompassQL (structural recommendation),
// MDM (entity classification), Cameleon (abstraction level)
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    Browse,     // Master entity: has_many rels, EntityName, FK target
    Focus,      // Content entity: FreeText, ImageUrl, rich detail
    Collect,    // Form entity: many writable fields, write_only, few rels
    Process,    // Workflow entity: state machine ≥3, guarded transitions
    Summarize,  // Dashboard entity: read-only Money/Percentage/Quantity
    Analyze,    // Analytical entity: DateTime + numeric measures
    Track,      // Audit entity: Status + temporal ordering

    #[serde(untagged)]
    Custom(String),
}
```

### IntentScore — Confidence-Scored Result
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct IntentScore {
    pub intent: Intent,
    pub confidence: f64,                // 0.0 to 1.0
    pub matching_signals: Vec<String>,  // e.g., ["has_state_machine", "guarded_transitions"]
}
```

### IntentHint — Manual Override
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntentHint {
    Primary(Intent),
    Exclude(Intent),
}
```

### ServiceDef with IntentHint
```rust
// When structural derivation gets it wrong (<30% expected):
let service = ServiceDef::new("unusual_service")
    .description("Looks like Browse but is actually a Process service")
    .intent_hint(IntentHint::Primary(Intent::Process))
    .field("id", DataType::Uuid, FieldMeaning::Identifier)
    // ... fields that structurally look like Browse
    .state_machine(/* ... minimal 2-state machine ... */);
```

### Derivation Result Example
```rust
// Phase 89 will produce these — Phase 88 defines the types:

// Order with state machine, guards, actions
// → Process(0.85), Track(0.60), Browse(0.40)

// Customer with has_many orders, contacts, addresses
// → Browse(0.90), Focus(0.55)

// SalesRecord with DateTime, Money, belongs_to customer
// → Analyze(0.80), Track(0.65), Summarize(0.50)

// Article with FreeText body, ImageUrl, draft→published states
// → Focus(0.85), Process(0.45)

// Registration form with many writable fields, write_only password
// → Collect(0.90), Focus(0.30)

// Dashboard with read-only Money/Percentage KPIs
// → Summarize(0.90), Analyze(0.55)
```

### Medium-Agnostic Test
```rust
// Every intent must pass: "This service is primarily for [verb]ing"
// "Browsing customers"          ✓ Browse — navigating a collection
// "Focusing on this article"    ✓ Focus — examining one entity deeply
// "Collecting registration data" ✓ Collect — capturing input
// "Processing this order"       ✓ Process — moving through workflow
// "Summarizing the dashboard"   ✓ Summarize — overview of metrics
// "Analyzing sales trends"      ✓ Analyze — exploring patterns
// "Tracking activity history"   ✓ Track — following events over time
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual UI specification (IFML, UsiXML) | Structural derivation from metadata | 2024-2025 | No hand-authored UI models needed |
| Field→widget mapping only | Field→semantic role→visualization (Metabase, Snowflake) | Production | Validates multi-level semantic mapping |
| Single best chart (Tableau Show Me) | Ranked recommendations with scores (CompassQL) | 2016+ | Validates confidence-scored approach |
| CRUD-based UI generation (Django admin, Appsmith) | Purpose-aware UI generation | Gap | Ferro's novel contribution |
| Agent generates HTML directly | Agent returns declarative specs (A2UI, AICF) | 2025 | Validates schema-driven approach |
| Full Cameleon 4-level pipeline | Direct AUI→FUI skip (A2UI) | 2025 | Skip CUI level |

**New systems validating our approach:**
- **CompassQL/Voyager** (UW): Multi-signal field analysis → ranked visualization recommendations. Directly validates our IntentScore model.
- **Metabase semantic types**: ~30 semantic types auto-drive visualization selection. Validates FieldMeaning → rendering decisions.
- **Snowflake Semantic Model Generator**: AI-assisted measure/dimension/fact classification. Validates automated field role detection.
- **Google A2UI v0.9**: Medium-agnostic component vocabulary. Validates our separation of intent from rendering.
- **Bridging Gulfs (arXiv 2601.19171, 2025)**: Four-level semantic hierarchy for UI generation with bidirectional mapping. Validates hierarchical intent → rendering.

**The gap Ferro fills:**
No existing system composes field semantics + state machine shape + relationship cardinality + action definitions into a service-level purpose classification. Individual layers are proven. The composition is novel.

**Deprecated/outdated:**
- **CRUD as intent taxonomy**: MARIA/XForms/IFML interaction taxonomies describe what users DO, not what services ARE. Wrong abstraction for structural derivation.
- **XML-based UIDLs**: Concepts sound, but JSON/Rust enums are the representation.
- **Petgraph for intent graphs**: Confirmed unnecessary for our scale (5-15 nodes).
</sota_updates>

<open_questions>
## Open Questions

1. **Should IntentGraph types live in Phase 88 or Phase 89?**
   - What we know: Phase 88 is "Core Types." Phase 89 is "Intent Graph Generation." IntentNode/IntentEdge/IntentGraph are graph structure types.
   - What's unclear: Whether graph types are "core" or "generation-specific."
   - Recommendation: Define Intent, IntentScore, IntentHint in Phase 88. Define IntentNode/IntentEdge/IntentGraph in Phase 89 where they're consumed. Keeps Phase 88 focused on the classification vocabulary.

2. **Should Custom(String) be untagged or tagged in serde?**
   - What we know: FieldMeaning uses `#[serde(untagged)]` for Custom(String), which causes anyOf shadowing in JSON Schema.
   - What's unclear: Whether the same approach is acceptable for Intent or if we should use a different pattern.
   - Recommendation: Use `#[serde(untagged)]` for consistency with FieldMeaning. Document the anyOf limitation in JSON Schema description annotation (same mitigation as Phase 85.1-01).

3. **Should there be a Reference intent for lookup/config data?**
   - What we know: MDM clearly distinguishes reference data (currency codes, country codes). Structural signals: few fields, rarely writable, high cardinality as FK target.
   - What's unclear: Whether Reference is distinct enough from Browse to warrant its own intent.
   - Recommendation: Start without it. If Phase 93 field test shows Browse consistently misclassifies reference entities, add Reference then. Principle: validate, don't speculate.

4. **How should intent_hints be stored on ServiceDef?**
   - What we know: Single IntentHint per service is simple. Vec<IntentHint> allows both Primary and Exclude.
   - What's unclear: Whether a service ever needs both "force Primary" and "exclude certain intents."
   - Recommendation: Use `Vec<IntentHint>` — allows `[Primary(Process), Exclude(Browse)]`. Serialize as empty-skipping vec.

5. **Should IntentScore.matching_signals be typed or strings?**
   - What we know: Typed enum would be safer. Strings are more flexible and easier to add.
   - What's unclear: Whether the signal vocabulary is stable enough for an enum.
   - Recommendation: Strings for now. The signal vocabulary will evolve through Phase 89 development. Type it later if the vocabulary stabilizes. Each string is a named signal like `"has_state_machine"`, `"money_fields_present"`.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [Cameleon Reference Framework (W3C)](https://www.w3.org/community/uad/wiki/Cameleon_Reference_Framework) — 4-level abstraction model, AUI positioning
- [CompassQL/Voyager (UW Interactive Data Lab)](https://github.com/vega/compassql) — Multi-signal field analysis → ranked recommendations, expressiveness+effectiveness
- [Metabase Semantic Types](https://www.metabase.com/docs/latest/data-modeling/semantic-types) — ~30 semantic types → auto-visualization, production-proven
- [APT (Mackinlay 1986)](https://dl.acm.org/doi/10.1145/22949.22950) — Expressiveness + effectiveness ranking principles
- [SAGE Data Characterization (CMU)](http://www.cs.cmu.edu/~./sage/sagedc.html) — Multi-dimensional data classification for visualization
- [Google A2UI v0.9](https://github.com/google/A2UI) — Agent-driven UI protocol, medium-agnostic design validation
- [FDC3 Intents v2.2 (FINOS)](https://fdc3.finos.org/docs/intents/spec) — 8 intent prefixes, typed vocabulary, small-set validation

### Secondary (MEDIUM confidence)
- [Snowflake Semantic Model Generator](https://github.com/Snowflake-Labs/semantic-model-generator) — AI-assisted measure/dimension/fact classification
- [Tableau Show Me](https://help.tableau.com/current/pro/desktop/en-us/buildauto_showme.htm) — Field-based chart type recommendation
- [Power BI Automatic Insights](https://learn.microsoft.com/en-us/power-bi/create-reports/service-insights) — Statistical algorithms for anomaly/trend/correlation detection
- [MARIA (W3C)](https://www.w3.org/wiki/images/3/36/MARIA.pdf) — Full interactor taxonomy (interaction layer reference, not intent layer)
- [XForms 1.1 (W3C)](https://www.w3.org/TR/xforms11/) — Semantic controls (interaction layer reference)
- [IFML (OMG Standard)](https://www.omg.org/spec/IFML/1.0/About-IFML) — ViewComponent subtypes, event taxonomy
- [Bridging Gulfs (arXiv 2601.19171, 2025)](https://arxiv.org/abs/2601.19171) — Four-level semantic hierarchy for UI generation
- [MDM Entity Classification](https://en.wikipedia.org/wiki/Master_data) — Master/Transactional/Reference/Freeform categories

### Tertiary (LOW confidence — needs validation)
- [DB-USE Approach](https://www.researchgate.net/publication/220728656_UI_generation_from_task_domain_and_user_models_The_DB-USE_approach) — Auto-derives CRUD from domain model (closest academic analog)
- [Airbnb Server-Driven UI](https://medium.com/airbnb-engineering/a-deep-dive-into-airbnbs-server-driven-ui-system-842244c5f5) — Enum-based component switching
- [Intent-Aware Visualization Recommendation (Springer 2022)](https://link.springer.com/article/10.1007/s41019-022-00191-7) — Considers user intent alongside data characteristics
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust enum design for structural service intent derivation
- Ecosystem: 5 visualization recommendation systems + 6 academic MBUI systems + 4 modern agent-UI standards + MDM classification
- Patterns: Structural signal analysis, multi-signal weighted scoring, service-level classification
- Pitfalls: CRUD confusion, granularity, non-derivable intents, single-signal derivation

**Confidence breakdown:**
- Intent taxonomy (7 variants): HIGH — grounded in MDM classification, validated by ServiceDef signal inventory, each intent has concrete derivation rules
- Multi-signal scoring model: HIGH — proven by CompassQL, APT, Metabase in production systems
- Structural signal inventory: HIGH — exhaustive inventory from Phase 87 codebase analysis
- IntentHint override mechanism: MEDIUM — straightforward but usage patterns TBD until Phase 93
- IntentScore.matching_signals as strings: MEDIUM — pragmatic, may need typing later
- Derivation rules (signal → intent): MEDIUM — well-reasoned but untested until Phase 89 implementation

**Research date:** 2026-02-28
**Valid until:** 2026-03-30 (30 days — structural signals are stable; derivation rules may evolve in Phase 89)
</metadata>

---

*Phase: 88-intent-core-types*
*Research completed: 2026-02-28 (updated: structural derivation alignment)*
*Ready for planning: yes*
