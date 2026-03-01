# Phase 94: Protocol Documentation — Research

**Researched:** 2026-03-01 (updated from 2026-02-28)
**Domain:** Protocol specification for data/intent-oriented web services with auto-generated UIs
**Confidence:** HIGH

<research_summary>
## Summary

Updated research incorporating the complete v9.0 implementation (Phases 84-93 shipped) and the rapidly evolving agentic UI protocol ecosystem (A2UI v0.9, AG-UI 0.1.11, MCP Apps, Open-JSON-UI).

The emerging consensus protocol stack is: **A2A** (agent coordination) → **MCP** (tools) → **AG-UI** (event transport) → **A2UI** (declarative UI rendering). Ferro's ServiceDef→IntentGraph pipeline fills a **confirmed gap** in this stack: no existing protocol covers the transformation from service definitions to user intents. A2UI describes *what* UI to show, AG-UI describes *how* to transport it, but nothing formalizes *deriving what users need* from a service schema.

The ferro-projections crate now has 22 public types, 7 structurally-derivable intents, a 5-analyzer derivation engine with 100% primary intent accuracy, and a pluggable renderer producing JSON-UI output. The protocol spec formalizes this complete, validated pipeline.

For specification tooling: **mdBook** (Rust-native, used by Wasm Component Model and Rust language spec), **schemars 1.x** (JSON Schema 2020-12 from Rust types), **date-based versioning** (following MCP pattern), and **RFC 2119** conformance language.

**Primary recommendation:** Publish a transport-agnostic protocol spec via mdBook that documents the ServiceDef→IntentGraph→Renderer chain. Generate JSON Schema from Rust types (never hand-write). Position as the missing "service definition → intent derivation" layer in the emerging agent protocol stack. Start at `0.1.0-draft`, host schemas at a stable URL, submit to SchemaStore.
</research_summary>

<standard_stack>
## Standard Stack

### Core (Protocol Infrastructure)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | 1.x | JSON serialization | Already used; all ferro-projections types derive Serialize/Deserialize |
| schemars | 1.1.0 | JSON Schema 2020-12 from Rust types | `#[derive(JsonSchema)]` already on all 22 public types |
| jsonschema | latest | Validate JSON against JSON Schema | High-perf, supports draft 2020-12 |
| mdBook | latest | Specification document publishing | Rust-native, used by Wasm Component Model + Rust language spec |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | 2.x | Error type derivation | Already used in ferro-projections |
| semver | latest | Protocol version parsing | Version negotiation if needed |
| typeshare | latest | Rust→TypeScript type generation | If TypeScript SDK for protocol types needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| mdBook | Bikeshed (W3C) | Bikeshed has auto-linking/bibliographies but requires Python; mdBook is native Rust |
| mdBook | spec-md (GraphQL) | spec-md is Node.js-based; better for algorithm-heavy specs like GraphQL |
| mdBook | Mintlify (MCP) | Mintlify is SaaS; mdBook is self-hosted and open source |
| JSON | Protocol Buffers | 25x faster but requires .proto files, not human-readable, not self-describing |
| schemars | hand-written JSON Schema | Schema drift is guaranteed; schemars keeps Rust as source of truth |
| typify (reverse) | schemars | typify goes JSON Schema→Rust; we need Rust→Schema since Rust types are source of truth |

**Installation:**
```bash
cargo install mdbook
# schemars and jsonschema already in ferro-projections Cargo.toml
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Specification Structure
Based on analysis of MCP (Mintlify), GraphQL (spec-md), Wasm Component Model (mdBook), and JSON:API:

```
docs/protocol/
├── book.toml               # mdBook configuration
├── src/
│   ├── SUMMARY.md           # Table of contents
│   ├── introduction.md      # Motivation, scope, non-goals
│   ├── terminology.md       # Domain-specific terms
│   ├── architecture.md      # Three-layer pipeline, roles, data flow
│   ├── data-model/
│   │   ├── service-def.md   # ServiceDef type + builder API
│   │   ├── field-def.md     # FieldDef, DataType, FieldMeaning
│   │   ├── state-machine.md # StateMachine, StateDef, Transition
│   │   ├── actions.md       # ActionDef, InputDef, GuardDef
│   │   ├── relationships.md # RelationshipDef, Cardinality, NavigationHint
│   │   └── intent.md        # Intent, IntentScore, IntentHint
│   ├── derivation.md        # Intent derivation rules (5 analyzers)
│   ├── rendering.md         # Renderer trait, RenderContext, RenderMode
│   ├── validation.md        # ServiceDef::validate() rules, Warning/Error types
│   ├── extensions.md        # Extension mechanism (x-* + URI-namespaced)
│   ├── conformance.md       # Conformance levels, RFC 2119 keywords
│   ├── security.md          # Security considerations
│   ├── related-work.md      # CAMELEON, SAP Fiori, A2UI, prior art
│   └── appendix/
│       ├── examples.md      # Full worked examples (all 7 intents)
│       ├── json-schema/     # Generated schemas (one per type)
│       └── changelog.md     # Revision history
└── schemas/
    ├── service-def.json     # Generated by schemars
    ├── intent-score.json    # Generated by schemars
    └── ...                  # One schema per public type
```

### Pattern 1: Rust Types as Source of Truth
**What:** Rust types are the canonical protocol definitions. JSON Schema is derived, never hand-written.
**Why:** ferro-projections already has `#[derive(JsonSchema)]` on all 22 public types.
**Pipeline:**
```
Rust types (ferro-projections/src/)
  → schemars::schema_for!()
  → JSON Schema files (docs/protocol/schemas/)
  → Published alongside mdBook spec
```

CI should regenerate schemas and fail if they drift from published versions.

### Pattern 2: Date-Based Schema Versioning
**What:** Schema `$id` URLs use date-based paths following MCP's pattern.
**Why:** MCP uses `schema/YYYY-MM-DD/schema.ts`; date-based versioning signals revision point without implying breaking-change semantics.
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://ferro-rs.dev/protocol/2026-03-01/service-def.json",
  "title": "ServiceDef",
  ...
}
```

### Pattern 3: Two-Tier Extension Mechanism
**What:** Simple vendor extensions via `x-*` prefix + formal protocol extensions via URI-namespaced keys.
**Model:** JSON:API's extension/profile system.
```json
{
  "name": "order",
  "fields": [...],
  "x-vendor-custom": "ignored by standard tooling",
  "extensions": {
    "https://example.com/ext/audit": {
      "critical": false,
      "data": { "track_changes": true }
    }
  }
}
```

### Pattern 4: RFC 2119 Conformance Language
**What:** Use MUST/SHOULD/MAY keywords per RFC 2119/BCP 14.
**Why:** Every modern protocol spec (MCP, JSON:API, OpenAPI, A2UI, GraphQL) uses this convention.

### Pattern 5: Transport-Agnostic Design
**What:** The protocol describes ONLY the data model and transformation rules. Transport is out of scope.
**Why:** A2UI deliberately separates UI description from transport. MCP handles tool access. AG-UI handles event streaming. The protocol should be consumable via any of these.
**Scope boundary:**
- IN: ServiceDef schema, IntentGraph derivation rules, Renderer output format, validation rules
- OUT: Transport (HTTP/MCP/WebSocket), authentication, session management, deployment

### Anti-Patterns to Avoid
- **Inventing a custom wire format:** Use JSON. Every alternative adds friction.
- **Schema-last approach:** Never write JSON Schema by hand and generate Rust from it.
- **Over-scoping:** The protocol is NOT a transport spec, NOT an auth spec, NOT a deployment spec.
- **Coupling to A2UI or any specific rendering target:** The Renderer trait is pluggable. JSON-UI is one implementation. A2UI could be another.
- **Premature 1.0:** Start at `0.1.0-draft`. Promote only after ecosystem feedback.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema generation | Manual schema files | `schemars` `#[derive(JsonSchema)]` | Schema stays in sync with Rust types automatically |
| JSON Schema validation | Custom validation logic | `jsonschema` crate | Supports draft 2020-12, high performance, well-tested |
| Version parsing | Custom string parsing | `semver` crate | Handles pre-release labels, build metadata, comparison |
| Specification rendering | Custom HTML generator | mdBook | Rust-native, GitHub Pages compatible, search built-in |
| TypeScript type generation | Manual .d.ts files | `typeshare` or `json-schema-to-typescript` | Automatic sync with Rust types |
| Conformance language | Ad-hoc wording | RFC 2119 keywords (MUST/SHOULD/MAY) | Industry standard, removes ambiguity |
| Schema publishing | Custom hosting | SchemaStore submission + stable URL hosting | Maximum discoverability, editor integration |

**Key insight:** The protocol specification is a documentation and formalization task, not an engineering task. The Rust types in ferro-projections already define the protocol — Phase 94 wraps them in a formal specification with JSON Schema, versioning, extension mechanism, and publication. The 22 public types, 5 analyzers, and Renderer trait are the protocol. Don't over-engineer the tooling.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Over-Scoping the Protocol
**What goes wrong:** Trying to define transport, authentication, session management, AND service projections in one protocol.
**Why it happens:** Desire for a complete, self-contained standard.
**How to avoid:** The protocol describes ONLY the data model (ServiceDef, IntentScore, RendererOutput) and the rules for transforming between levels. Transport is MCP or HTTP. Authentication is out of scope.
**Warning signs:** Protocol spec growing beyond ~50 pages; sections about TCP connections or TLS.

### Pitfall 2: Schema Drift Between Spec and Implementation
**What goes wrong:** The published JSON Schema gets out of sync with the actual Rust types.
**Why it happens:** Manual maintenance of separate artifacts.
**How to avoid:** Generate JSON Schema from Rust types via schemars. Never hand-edit the schema. CI should regenerate and compare.
**Warning signs:** Manual edits to .json schema files; "update schema" as a separate task from code changes.

### Pitfall 3: Premature Standardization
**What goes wrong:** Locking down protocol details before ecosystem feedback.
**Why it happens:** Phase 94 comes after implementation is complete.
**How to avoid:** Start at `0.1.0-draft`. Use the field test results from Phase 93 to validate design. Only promote to `1.0.0` after external adoption confirms the design.
**Warning signs:** Marking spec as "stable" or "1.0" before any external consumer exists.

### Pitfall 4: Ignoring the CAMELEON Prior Art
**What goes wrong:** Describing Ferro's architecture as fully novel when the W3C CAMELEON Reference Framework defines the same four-level chain (Domain → Abstract UI → Concrete UI → Final UI).
**Why it happens:** Not searching academic literature.
**How to avoid:** Acknowledge CAMELEON in the spec's "Related Work" section. Position Ferro's contribution as the specific combination of schema-only constraints + intent graph derivation + pluggable renderers in a server-side framework — not the invention of multi-level abstraction.
**Warning signs:** Claims of "first ever" or "novel approach" without CAMELEON citation.

### Pitfall 5: Documenting Derivation Weights as Protocol
**What goes wrong:** Encoding the exact signal weights (e.g., "0.3 * count for Summarize from Money fields") into the spec as normative requirements.
**Why it happens:** Wanting to ensure consistent behavior across implementations.
**How to avoid:** The spec should define WHAT signals each analyzer considers and the general rules (e.g., "field meaning analyzer MUST consider Money fields as a signal for Summarize intent"). The exact weights are implementation-specific tuning. Document the 5 analyzers and their signal types, not the numeric weights.
**Warning signs:** Weight constants appearing as MUST requirements in the spec.
</common_pitfalls>

<prior_art>
## Prior Art Analysis

### Critical Prior Art (HIGH relevance)

#### W3C CAMELEON Reference Framework (2003)
Four-level abstraction chain almost identical to Ferro's architecture:
1. **Task & Domain Model** → Ferro's `ServiceDef` + `StateMachine`
2. **Abstract UI (AUI)** → Ferro's `IntentScore[]` (derived intents)
3. **Concrete UI (CUI)** → Ferro's `Renderer` output (JSON-UI components)
4. **Final UI (FUI)** → Rendered HTML/native code

**Ferro's differentiation:** CAMELEON AUI is tree-based and static. Ferro's IntentGraph is dynamically derived from structural signals with confidence scores and state-dependent context.

Source: [W3C CAMELEON](https://www.w3.org/community/uad/wiki/Cameleon_Reference_Framework)

#### SAP Fiori Elements + OData Annotations
Production-proven at massive scale. OData annotations enrich service metadata with semantic UI information — functionally identical to `FieldMeaning` driving renderer decisions.

Source: [SAP Fiori Elements](https://cap.cloud.sap/docs/advanced/fiori)

#### MECANO (Puerta & Eriksson, 1996)
Pioneered semantic enrichment of data models for UI generation. Ferro's `FieldMeaning` is this concept.

Source: [Semantic Scholar](https://www.semanticscholar.org/paper/Beyond-Data-Models-for-Automated-User-Interface-Puerta-Eriksson/2df4fa562abb4a5426ae6e0c2d351dc5e4851143)

### The 2026 Agent Protocol Stack

The ecosystem has converged on a layered stack. Ferro occupies the gap between service definition and UI specification:

```
┌─────────────────────────────────────────────────────────────┐
│  A2A (Google/LF)          Agent ↔ Agent coordination        │
├─────────────────────────────────────────────────────────────┤
│  MCP (Anthropic/LF)       Agent ↔ Tools & Context           │
├─────────────────────────────────────────────────────────────┤
│  ══════════════════════════════════════════════════════════  │
│  Ferro Protocol     ServiceDef → IntentGraph → Renderer     │
│  (THE GAP)          "What UI does this service need?"       │
│  ══════════════════════════════════════════════════════════  │
├─────────────────────────────────────────────────────────────┤
│  AG-UI (CopilotKit)       Agent ↔ Frontend event transport  │
├─────────────────────────────────────────────────────────────┤
│  A2UI / Open-JSON-UI      Declarative UI component specs    │
│  (Google / OpenAI)                                          │
└─────────────────────────────────────────────────────────────┘
```

No existing protocol covers steps 1→2 of:
1. **Service definition** (fields, state machines, actions, relationships)
2. **Intent derivation** (what meaningful interactions exist?)
3. **UI generation** (declarative component tree from intents)

A2UI covers step 3. AG-UI transports step 3. MCP provides tools. But formalizing "given this service definition, derive these user intents" is an open space.

### Existing Standards Comparison (Updated March 2026)

| Standard | Entities | Operations | Semantic Types | State Machine | Intent Derivation | UI Output | Status |
|----------|----------|------------|----------------|---------------|-------------------|-----------|--------|
| OpenAPI 3.1 | Schemas | Full CRUD | `format` field | No | No | Via forms | Active |
| GraphQL SDL | Types | Query/Mutation | No | No | No | Via introspection | Active |
| Hydra | Classes | hydra:Operation | Schema.org | No | No | API Platform Admin | Low activity |
| Siren | Entities | Actions+fields | HTML5 inputs | No | No | Actions as forms | Stable |
| A2UI v0.9 | Components | Actions+context | Catalog-defined | No | No | Full declarative | Pre-1.0 |
| Open-JSON-UI | Components | Actions | Minimal | No | No | Token-efficient | Active |
| MCP Apps | Tools | Tool calls | No | No | No | Sandboxed iframe | Extension |
| **Ferro** | ServiceDef | Actions+StateMachine | FieldMeaning | Yes (schema-only) | 5-analyzer engine | Pluggable Renderer | Implementing |

### Novelty Assessment

| Aspect | Novel? | Prior Art |
|--------|--------|-----------|
| Generating UI from data models | No | MECANO (1996), MB-UIDEs (1990s-2000s) |
| Multi-level abstraction (Domain→Abstract→Concrete→Final) | No | CAMELEON (2003), USIXML, IFML |
| Semantic type annotations driving UI rendering | No | SAP OData annotations, Metabase |
| State machines driving UI generation | No | US20070266329A1, US20100106547A1 |
| Schema-only state machines (no closures, fully serializable) | Partial | XState philosophy, but XState has runtime |
| Confidence-scored intent derivation from structural signals | Yes | No precedent for scored, multi-signal derivation |
| 5-analyzer pipeline (fields, writability, state machine, relationships, actions) | Yes | No precedent for this specific combination |
| Full pipeline in server-side framework with schema-only constraint | Yes | No exact precedent found |
</prior_art>

<ecosystem_update>
## Ecosystem Update (March 2026)

### A2UI (Google)
- **Current:** v0.8 stable, v0.9 draft (Nov 2025)
- **Key change in v0.9:** Shifted from "Structured Output First" to "Prompt First" — schema embedded in prompt rather than relying on structured output modes
- **Production adoption:** Google Opal (hundreds of thousands of users), Gemini Enterprise, Flutter GenUI SDK
- **Roadmap to 1.0:** Spec stabilization, official React/Compose/SwiftUI renderers, REST transport
- **Relevance to Ferro:** A2UI is a potential rendering target. Ferro's JsonUiRenderer could be complemented by an A2UIRenderer that outputs A2UI component trees.
- Source: [a2ui.org](https://a2ui.org/), [A2UI v0.9 Spec](https://a2ui.org/specification/v0.9-a2ui/)

### AG-UI (CopilotKit)
- **Current:** v0.1.11 (Feb 2026)
- **Adoption:** Microsoft Agent Framework, Oracle Open Agent Spec, LangGraph, CrewAI, Google ADK, AG2, Pydantic AI
- **Architecture:** Event-based SSE-over-HTTP with 5 event categories (lifecycle, text, tool calls, state, special)
- **Relevance to Ferro:** AG-UI is a transport layer. If Ferro-generated UIs need to stream to frontends, AG-UI events could carry Renderer output.
- Source: [docs.ag-ui.com](https://docs.ag-ui.com/)

### MCP (Anthropic/Linux Foundation)
- **Current spec:** 2025-11-25
- **Key additions:** Tasks primitive (async long-running ops), enhanced authorization, extensions framework
- **MCP Apps:** Extension (`ext-apps` v1.1.2) enabling tools to return sandboxed iframe UIs. Supported by ChatGPT, Claude, Goose, VS Code.
- **Governance:** Linux Foundation alongside A2A
- **Relevance to Ferro:** ferro-mcp already has projection introspection tools. MCP Apps could serve Ferro-rendered UIs as interactive surfaces. MCP is the natural discovery/introspection transport.
- Source: [modelcontextprotocol.io](https://modelcontextprotocol.io/specification/2025-11-25)

### Open-JSON-UI (OpenAI)
- **Status:** Active, used alongside A2UI
- **Key trait:** Token-efficient flat JSON optimized for LLM structured output generation
- **Relationship to A2UI:** Complementary. Open-JSON-UI → A2UI renderer translates between the two. A2UI is explicit/renderable; Open-JSON-UI is minimal/LLM-friendly.
- **Relevance to Ferro:** Ferro's ServiceDef is more structured than Open-JSON-UI (semantic types, state machines). But Open-JSON-UI's token-efficiency lessons apply to how we serialize IntentScore output.
- Source: [docs.copilotkit.ai/generative-ui/specs/open-json-ui](https://docs.copilotkit.ai/generative-ui/specs/open-json-ui)

### json-render (Vercel)
- **What:** Monolithic generative UI framework — AI generates JSON constrained to developer-defined component catalog, rendered as React.
- **Relevance to Ferro:** Validates the "catalog-constrained generation" approach. Ferro's JSON-UI component catalog serves the same purpose. Not a protocol competitor — it's a single-app framework.
- Source: [json-render.dev](https://json-render.dev/)
</ecosystem_update>

<implementation_summary>
## Implementation Summary (Phases 84-93 Complete)

The protocol spec documents the following validated, shipped implementation:

### Public API Surface (22 types)

**Core types:**
- `ServiceDef` — Complete service projection schema (fields, actions, guards, relationships, state machine, intent hints)
- `FieldDef` — Field with name, DataType, FieldMeaning, required, is_list, readable, writable
- `DataType` — 10 abstract categories: String, Integer, Float, Boolean, DateTime, Date, Json, Binary, Uuid, Enum
- `FieldMeaning` — 18 semantic variants + Custom(String): Identifier, ForeignKey, CreatedAt, UpdatedAt, EntityName, Email, Phone, Url, ImageUrl, Money, Percentage, Quantity, Status, Category, Boolean, FreeText, DateTime, Sensitive

**State machine types:**
- `StateMachine` — Lifecycle definition with states, transitions, initial state
- `StateDef` — State with display_name, is_final, on_enter/on_exit effects, metadata
- `Transition` — From/event/to with optional guard and actions

**Action types:**
- `ActionDef` — Business operation with inputs, preconditions, effects, transition_trigger
- `InputDef` — Action parameter reusing DataType/FieldMeaning
- `GuardDef` — Named boolean condition

**Relationship types:**
- `RelationshipDef` — Service-to-service with cardinality, navigation hint, foreign key
- `Cardinality` — OneToOne, OneToMany, ManyToOne, ManyToMany
- `NavigationHint` — Inline, Link, Tab, Nested, Hidden

**Intent types:**
- `Intent` — 7 variants + Custom: Browse, Focus, Collect, Process, Summarize, Analyze, Track
- `IntentScore` — Scored intent with confidence (f64) and matching_signals
- `IntentHint` — Primary(Intent) or Exclude(Intent) manual override

**Rendering types:**
- `Renderer` trait — `render(&self, service, intents, ctx) -> Result<Value, Error>`
- `RenderContext` — intent_index, current_state, mode
- `RenderMode` — Display or Input
- `JsonUiRenderer` — Concrete implementation producing ferro-json-ui/v1 output

**Validation:**
- `Warning` — 8 structural warning variants
- `Error` — 4 error variants (Definition, Validation, Render, Serialization)

### Derivation Engine (5 Analyzers)
1. **Field meaning analyzer** — proportional count-weighted signals from FieldMeaning variants
2. **Writability analyzer** — readable/writable ratios signal Collect, Summarize, Focus
3. **State machine analyzer** — guard density, branching, transitions signal Process/Track
4. **Relationship analyzer** — cardinality and navigation patterns signal Browse/Focus
5. **Action analyzer** — transition triggers, inputs, preconditions signal Process/Collect

**Validation result:** 100% primary intent accuracy across 12 representative ServiceDefs covering all 7 intents.

### Serialization Guarantees
All public types derive `Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema`:
- Enums use `#[serde(rename_all = "snake_case")]`
- Custom variants use `#[serde(untagged)]` for fallback
- Optional fields skip serialization when None
- Empty vecs skip serialization
- All types are JSON-round-trip safe
</implementation_summary>

<code_examples>
## Code Examples

### JSON Schema Generation Pipeline
```rust
// Source: schemars 1.x documentation + existing ferro-projections derives
use schemars::schema_for;
use ferro_projections::ServiceDef;

// Generate schema for the core type:
let schema = schema_for!(ServiceDef);
let json = serde_json::to_string_pretty(&schema).unwrap();
// Write to docs/protocol/schemas/service-def.json
```

### Complete ServiceDef JSON (Protocol Wire Format)
```json
{
  "name": "order",
  "display_name": "Order",
  "fields": [
    {
      "name": "id",
      "data_type": "integer",
      "meaning": "identifier",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": false
    },
    {
      "name": "total",
      "data_type": "float",
      "meaning": "money",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": true
    },
    {
      "name": "status",
      "data_type": "string",
      "meaning": "status",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": false
    }
  ],
  "actions": [
    {
      "name": "submit",
      "display_name": "Submit Order",
      "inputs": [
        {
          "name": "notes",
          "data_type": "string",
          "meaning": "free_text",
          "required": false
        }
      ],
      "preconditions": ["has_items"],
      "transition_trigger": "submit"
    }
  ],
  "guards": [
    {
      "name": "has_items",
      "display_name": "Has Items",
      "description": "Order must contain at least one item"
    }
  ],
  "relationships": [
    {
      "name": "items",
      "target": "order_item",
      "cardinality": "one_to_many",
      "navigation": "nested"
    },
    {
      "name": "customer",
      "target": "customer",
      "cardinality": "many_to_one",
      "navigation": "link",
      "foreign_key": "customer_id"
    }
  ],
  "state_machine": {
    "name": "order_lifecycle",
    "initial_state": "draft",
    "states": [
      { "name": "draft", "display_name": "Draft" },
      { "name": "submitted", "display_name": "Submitted" },
      { "name": "completed", "display_name": "Completed", "is_final": true }
    ],
    "transitions": [
      {
        "from": "draft",
        "event": "submit",
        "to": "submitted",
        "guard": "has_items"
      },
      {
        "from": "submitted",
        "event": "complete",
        "to": "completed"
      }
    ]
  }
}
```

### Protocol Envelope (Versioned)
```json
{
  "protocol": "ferro-projections",
  "version": "0.1.0-draft",
  "schema_url": "https://ferro-rs.dev/protocol/2026-03-01/service-def.json",
  "service": { ... }
}
```

### Intent Derivation Output
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
  }
]
```

### Extension Mechanism
```json
{
  "name": "order",
  "fields": [...],
  "x-acme-priority": "high",
  "extensions": [
    {
      "uri": "https://acme.com/ext/audit-trail",
      "critical": false,
      "data": { "track_field_changes": true, "retention_days": 90 }
    }
  ]
}
```
</code_examples>

<sota_updates>
## State of the Art (March 2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| A2UI v0.8 (structured output first) | A2UI v0.9 (prompt first) | Nov 2025 | Richer schema, requires post-gen validation |
| MCP core only | MCP + MCP Apps extension | Jan 2026 | Tools can return interactive sandboxed UIs |
| AG-UI early adoption | AG-UI broad adoption (Microsoft, Oracle, Google) | Q1 2026 | De facto agent-frontend transport standard |
| Separate protocol efforts | Linux Foundation governance (MCP + A2A) | 2025-2026 | Consolidation toward consensus stack |
| Custom UI generation | json-render (Vercel) + Thesys (C1 API) | 2025-2026 | Generative UI becoming mainstream |
| Protocol PDFs | Living docs (mdBook, Mintlify, Bikeshed) | 2020s | Auto-generated, versioned, searchable |
| SemVer for protocol versions | Date-based versioning (MCP pattern) | 2025 | Signals revision point, not breaking semantics |

**Emerging consensus stack (March 2026):**
1. **A2A** (Google/LF) — Agent ↔ Agent coordination
2. **MCP** (Anthropic/LF) — Agent ↔ Tools & Context
3. **AG-UI** (CopilotKit) — Agent ↔ Frontend event transport
4. **A2UI / Open-JSON-UI** (Google / OpenAI) — Declarative UI specification

**Ferro's position:** Between MCP (tools/context) and A2UI (declarative UI). The "brain" that decides WHAT UI to generate from a service definition. No other protocol occupies this layer.

**New tools/patterns:**
- MCP Apps: Sandboxed iframe UI from MCP tools — potential distribution channel for Ferro-rendered UIs
- json-render (Vercel): Validates catalog-constrained generation approach
- Thesys/Crayon: Commercial generative UI API — validates market demand
- Oracle Open Agent Spec: Declarative agent definitions with AG-UI integration

**Deprecated/outdated from previous research:**
- Patent strategy: Descoped in Phase 85.1-02. Open-source publication preferred.
- USDL, WADL, XForms: Still dead/irrelevant.
</sota_updates>

<open_questions>
## Open Questions

### Resolved (from previous research)

1. **~~Should the protocol define IntentGraph wire format or only ServiceDef?~~**
   **RESOLVED:** Both. derive_intents() is implemented and validated. The protocol documents ServiceDef (input), IntentScore[] (intermediate), and Renderer output (final). All three layers are stable.

2. **~~Patent filing: provisional vs utility vs open-source?~~**
   **RESOLVED:** Patent descoped in Phase 85.1-02. Open-source specification publication for ecosystem adoption.

3. **~~How does the protocol relate to MCP?~~**
   **RESOLVED:** ferro-mcp has 3 projection introspection tools (list_projections, inspect_projection, render_projection) + validation + coverage tools. MCP is the discovery/introspection transport. The protocol spec is transport-agnostic.

4. **~~Should the spec target interoperability with A2UI?~~**
   **RESOLVED:** The Renderer trait is pluggable by design. JsonUiRenderer is the current implementation. An A2UIRenderer could be added as a future extension without protocol changes.

### New Open Questions

1. **Should the spec include derivation rules as normative or informative?**
   - What we know: The 5-analyzer pipeline works. Exact weights are tuning parameters.
   - Recommendation: Derivation signal types (WHAT each analyzer considers) should be normative. Exact weights should be informative/implementation-specific.

2. **Should the protocol envelope include derived intents, or only ServiceDef?**
   - What we know: Some consumers may want to receive pre-derived intents (e.g., MCP tool returning render results). Others may want raw ServiceDef and derive locally.
   - Recommendation: Define both: a ServiceDef-only format and a "resolved" format that includes IntentScore[] alongside ServiceDef. Let consumers choose.

3. **Where to host the spec and schemas?**
   - Options: GitHub Pages (simple, free), ferro-rs.dev subdomain (branded), SchemaStore (discoverability)
   - Recommendation: All three. Spec on GitHub Pages, schemas at stable ferro-rs.dev URL, submit to SchemaStore for editor integration.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)

**Protocol Design:**
- [RFC 2119: Key words for use in RFCs](https://datatracker.ietf.org/doc/html/rfc2119) — conformance language
- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25) — capability negotiation, versioning, extensions
- [MCP Versioning](https://modelcontextprotocol.io/specification/versioning) — date-based versioning pattern
- [MCP Apps Extension](https://github.com/modelcontextprotocol/ext-apps) — sandboxed UI from MCP tools
- [JSON:API v1.1](https://jsonapi.org/format/) — extension/profile system, error objects
- [OpenAPI 3.1.1](https://spec.openapis.org/oas/v3.1.1.html) — extension mechanism, deprecation
- [A2UI v0.8 Spec](https://a2ui.org/specification/v0.8-a2ui/) — catalog system, declarative UI
- [A2UI v0.9 Spec (Draft)](https://a2ui.org/specification/v0.9-a2ui/) — prompt-first shift
- [JSON Schema 2020-12](https://json-schema.org/specification) — vocabulary system

**Ecosystem:**
- [AG-UI Protocol](https://docs.ag-ui.com/) — event-based agent-frontend transport
- [AG-UI and A2UI Explained](https://www.copilotkit.ai/blog/ag-ui-and-a2ui-explained-how-the-emerging-agentic-stack-fits-together) — stack relationship
- [Open-JSON-UI Spec](https://docs.copilotkit.ai/generative-ui/specs/open-json-ui) — token-efficient UI format
- [A2A Protocol](https://a2aprotocol.ai/) — agent-to-agent coordination
- [json-render (Vercel)](https://json-render.dev/) — catalog-constrained generative UI

**Tooling:**
- [Schemars 1.x](https://graham.cool/schemars/) — JSON Schema generation from Rust
- [mdBook](https://github.com/rust-lang/mdBook) — Rust specification publishing
- [Wasm Component Model (mdBook)](https://component-model.bytecodealliance.org/) — Rust protocol spec example
- [SchemaStore](https://www.schemastore.org/) — JSON Schema registry
- [Typify (Oxide)](https://github.com/oxidecomputer/typify) — JSON Schema → Rust (reverse direction)

**Prior Art:**
- [W3C CAMELEON Reference Framework](https://www.w3.org/community/uad/wiki/Cameleon_Reference_Framework) — four-level abstraction chain
- [SAP Fiori Elements + OData Annotations](https://cap.cloud.sap/docs/advanced/fiori) — production semantic UI generation
- [MECANO (1996)](https://www.semanticscholar.org/paper/Beyond-Data-Models-for-Automated-User-Interface-Puerta-Eriksson/2df4fa562abb4a5426ae6e0c2d351dc5e4851143) — semantic enrichment concept

### Secondary (MEDIUM confidence)
- [Microsoft AG-UI Integration](https://learn.microsoft.com/en-us/agent-framework/integrations/ag-ui/) — AG-UI adoption evidence
- [Oracle AG-UI Integration](https://blogs.oracle.com/ai-and-datascience/announcing-ag-ui-integration-for-agent-spec) — AG-UI adoption evidence
- [Google ADK + AG-UI](https://developers.googleblog.com/delight-users-by-combining-adk-agents-with-fancy-frontends-using-ag-ui/) — AG-UI adoption evidence
- [The Agent Protocol Stack (Blog)](https://subhadipmitra.com/blog/2026/agent-protocol-stack/) — consensus stack analysis
- [Spec-Driven Development (arXiv)](https://arxiv.org/abs/2602.00180) — spec-as-source-code pattern
- [Tor mdBook Proposal](https://spec.torproject.org/proposals/345-specs-in-mdbook.html) — mdBook for protocol specs
- [RFC 8594: Sunset Header](https://www.rfc-editor.org/rfc/rfc8594) — deprecation mechanism

### Tertiary (LOW confidence — needs validation)
- [JSON Schema 2025 Stability Promise](https://json-schema.org/blog/posts/the-last-breaking-change) — "last breaking change" commitment
- [MCP SemVer Proposal (SEP-1400)](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1400) — potential MCP versioning change
- Thesys/Crayon — commercial generative UI, longevity unclear
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Protocol specification design, JSON Schema, mdBook publishing
- Ecosystem: A2UI v0.9, AG-UI, MCP Apps, Open-JSON-UI, A2A, json-render — full 2026 agent protocol stack
- Patterns: Extension mechanisms, versioning (date-based), capability negotiation, conformance levels
- Prior art: CAMELEON, SAP Fiori, MECANO, 2026 protocol convergence
- Implementation: 22 public types, 5 analyzers, Renderer trait — all complete and validated
- Pitfalls: Over-scoping, schema drift, premature standardization, weight-as-spec

**Confidence breakdown:**
- Standard stack: HIGH — schemars/jsonschema/mdBook are established, well-maintained
- Protocol design patterns: HIGH — based on analysis of 6+ production protocol specs
- Ecosystem positioning: HIGH — confirmed gap in consensus protocol stack
- Prior art: HIGH — comprehensive search, updated with 2026 landscape
- Implementation knowledge: HIGH — all Phases 84-93 complete with 309+ tests
- Code examples: HIGH — based on actual ferro-projections types and schemars docs

**Research date:** 2026-03-01
**Valid until:** 2026-03-31 (30 days — protocol design patterns stable; A2UI/AG-UI ecosystems evolving fast toward 1.0 releases)
</metadata>

---

*Phase: 94-protocol-documentation*
*Research completed: 2026-03-01 (updated)*
*Ready for planning: yes*
