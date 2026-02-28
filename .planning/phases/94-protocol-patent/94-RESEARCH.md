# Phase 94: Protocol & Patent — Research

**Researched:** 2026-02-28
**Domain:** Protocol specification for data/intent-oriented web services; patent landscape for declarative UI generation
**Confidence:** HIGH

<research_summary>
## Summary

Researched the full landscape for defining a standardized protocol around Ferro's ServiceDef→IntentGraph→Renderer architecture, and the patent/prior art landscape for declarative UI generation from service definitions.

The prior art is **densely populated** — 30+ years of academic work (MB-UIDEs, CAMELEON Reference Framework), multiple granted patents, W3C standards, and commercial systems (SAP Fiori, Oracle ADF). No single existing standard solves the full problem of "describe a service such that a UI can be generated," but the individual layers (semantic types, state machines, abstract UI, multi-renderer projection) all have precedent. The closest existing systems are Hydra + API Platform Admin (production-proven CRUD generation), Siren (task-based interfaces with form fields), and Google A2UI (agent-driven declarative UI).

For protocol design, JSON is the right wire format (already serde-native, human-readable, aligns with MCP/OpenAPI/A2UI ecosystems). `schemars` generates JSON Schema from Rust types, keeping Rust as source of truth. SemVer with maturity labels for versioning. Two-tier extension mechanism (simple `x-*` fields + formal URI-namespaced extensions). mdBook for specification document.

**Primary recommendation:** Focus the protocol spec on what Ferro uniquely combines — the full pipeline from schema-only service definitions (with semantic field meanings and state machines) through state-dependent intent graph generation to pluggable renderers. Do not pursue broad patents; instead consider open-source strategy for ecosystem adoption. If filing, narrow claims to the specific combination of schema-only constraints + state-dependent graph generation + the particular transformation pipeline.
</research_summary>

<standard_stack>
## Standard Stack

### Core (Protocol Infrastructure)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | 1.x | JSON serialization | Already used; all Ferro types derive Serialize/Deserialize |
| schemars | 1.1.0 | Generate JSON Schema from Rust types | `#[derive(JsonSchema)]` with full serde compatibility |
| jsonschema | latest | Validate JSON against JSON Schema | High-perf, supports draft 2020-12 |
| semver | latest | Protocol version parsing | SemVer negotiation for protocol compatibility |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | 2.x | Error type derivation | Already used in ferro-projections |
| mdBook | latest | Specification document | Publishing protocol spec as static site |
| rmp-serde | 1.3.0 | MessagePack serialization | Optional binary format optimization (same serde derives) |
| typeshare | latest | Rust→TypeScript type generation | If TypeScript bindings for protocol types needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| JSON | Protocol Buffers (prost) | 25x faster but requires .proto files, not human-readable, not self-describing |
| JSON | CBOR (ciborium) | Binary but self-describing; 3.5% serialize speed vs 3.9% for JSON |
| schemars | typify (reverse direction) | typify goes JSON Schema→Rust; we want Rust→Schema since Rust types are source of truth |
| mdBook | Bikeshed (W3C) | Bikeshed has auto-linking and bibliographies but requires Python; mdBook is native Rust |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Specification Structure
Based on analysis of MCP, JSON:API, OpenAPI, A2UI, and IETF RFC guidelines:

```
specification/
├── src/
│   ├── SUMMARY.md           # mdBook table of contents
│   ├── introduction.md      # Motivation, scope, non-goals
│   ├── terminology.md       # Domain-specific terms
│   ├── architecture.md      # Roles, layers, data flow
│   ├── data-model/
│   │   ├── service-def.md   # ServiceDef type
│   │   ├── field-meaning.md # FieldMeaning enum
│   │   ├── state-machine.md # StateMachine, Transition
│   │   ├── actions.md       # ActionDef, Precondition
│   │   ├── relationships.md # Service relationships
│   │   └── intent-graph.md  # IntentGraph, IntentNode
│   ├── rendering.md         # Renderer trait, projection rules
│   ├── extensions.md        # Extension mechanism
│   ├── errors.md            # Error codes and structure
│   ├── security.md          # Security considerations
│   ├── conformance.md       # Conformance levels
│   └── appendix/
│       ├── examples.md      # Full worked examples
│       ├── json-schema.md   # Published JSON Schema
│       └── changelog.md     # Revision history
└── book.toml
```

### Pattern 1: Schema-First with Rust as Source of Truth
**What:** Rust types are the canonical protocol definitions. JSON Schema is derived, not hand-written.
**When to use:** Always — this is the core design decision.
**How it works:**
```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub fields: Vec<FieldDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_machine: Option<StateMachine>,
}

// Generate JSON Schema at build time or via CLI:
// let schema = schemars::schema_for!(ServiceDef);
// let json = serde_json::to_string_pretty(&schema).unwrap();
```

### Pattern 2: Two-Tier Extension Mechanism
**What:** Simple vendor extensions via `x-*` prefix + formal protocol extensions via URI-namespaced keys with criticality flags.
**When to use:** For extensibility without breaking interoperability.
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

### Pattern 3: SemVer with Maturity Labels
**What:** Semantic versioning combined with Draft/RC/Stable maturity markers.
**Model:** OpenAPI + MCP hybrid approach.
```
0.1.0-draft    → Initial protocol draft
0.2.0-draft    → Breaking changes during draft
1.0.0-rc.1     → Release candidate
1.0.0          → Stable release
1.1.0          → Backward-compatible additions
2.0.0          → Breaking changes
```

### Pattern 4: RFC 2119 Conformance Language
**What:** Use MUST/SHOULD/MAY keywords per RFC 2119/BCP 14 for requirement levels.
**Why:** Every modern protocol spec (MCP, JSON:API, OpenAPI, A2UI) uses this convention.

### Anti-Patterns to Avoid
- **Inventing a custom wire format:** Use JSON. Every alternative adds friction without solving the actual problem.
- **Schema-last approach:** Don't write JSON Schema by hand and generate Rust types from it. Rust types are the source of truth.
- **Overly broad protocol scope:** The protocol describes service projections (ServiceDef→IntentGraph→Renderer output). It does NOT need to describe transport, authentication, or session management — those are handled by MCP or HTTP.
- **Breaking changes without major version bump:** Once stable (1.0.0), all breaking changes require a major version increment.
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

**Key insight:** The protocol specification is a documentation and formalization task, not an engineering task. The Rust types in `ferro-projections` already define the protocol — Phase 94 wraps them in a formal specification with JSON Schema, versioning, extension mechanism, and publication. Don't over-engineer the tooling.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Broad Patent Claims in a Dense Prior Art Space
**What goes wrong:** Filing patents with broad claims that are easily invalidated by 30 years of prior art.
**Why it happens:** The concept of "generate UI from service definitions" appears novel when viewed from the web framework perspective, but the academic MB-UIDE community has explored this since the 1990s.
**How to avoid:**
- Do NOT claim "generating UI from service definitions" broadly
- Do NOT claim "semantic types driving widget selection" (SAP Fiori OData annotations, Metabase, MECANO all predate)
- Do NOT claim "multi-level abstraction from domain to concrete UI" (CAMELEON Reference Framework, W3C, 2003)
- IF filing, claim the **specific combination**: schema-only definitions (no closures) + state-dependent intent graph generation + the particular transformation pipeline within a server-side framework
- Consider open-source strategy instead — Google open-sourced A2UI, CopilotKit open-sourced AG-UI
**Warning signs:** A patent attorney saying "this is very novel" without having searched the MB-UIDE literature

### Pitfall 2: Over-Scoping the Protocol
**What goes wrong:** Trying to define transport, authentication, session management, AND service projections in one protocol.
**Why it happens:** Desire for a complete, self-contained standard.
**How to avoid:** The protocol should describe ONLY the data model (ServiceDef, IntentGraph, RendererOutput) and the rules for transforming between levels. Transport is MCP or HTTP. Authentication is out of scope. Session management is out of scope.
**Warning signs:** Protocol spec growing beyond ~50 pages; sections about TCP connections or TLS

### Pitfall 3: Schema Drift Between Spec and Implementation
**What goes wrong:** The published JSON Schema gets out of sync with the actual Rust types.
**Why it happens:** Manual maintenance of separate artifacts.
**How to avoid:** Generate JSON Schema from Rust types via `schemars`. Never hand-edit the schema. CI should regenerate and compare.
**Warning signs:** Manual edits to .json schema files; "update schema" as a separate task from code changes

### Pitfall 4: Premature Standardization
**What goes wrong:** Locking down protocol details before the implementation is validated through Phase 93 field test.
**Why it happens:** Phase 94 is at the end of the roadmap but may pull forward before all implementation phases complete.
**How to avoid:** Start at `0.1.0-draft`. Use the field test (Phase 93) to validate the protocol. Only promote to `1.0.0` after real-world usage confirms the design.
**Warning signs:** Marking spec as "stable" or "1.0" before Phase 93 is complete

### Pitfall 5: Ignoring the CAMELEON Prior Art
**What goes wrong:** Describing Ferro's architecture as fully novel when the W3C CAMELEON Reference Framework defines the same four-level chain (Domain → Abstract UI → Concrete UI → Final UI).
**Why it happens:** Not searching academic literature.
**How to avoid:** Acknowledge CAMELEON in the spec's "Related Work" section. Position Ferro's contribution as a specific instantiation optimized for server-side web frameworks with schema-only constraints, not as the invention of the multi-level abstraction.
**Warning signs:** Claims of "first ever" or "novel approach" without CAMELEON citation
</common_pitfalls>

<prior_art>
## Prior Art Analysis

### Critical Prior Art (HIGH relevance to Ferro)

#### W3C CAMELEON Reference Framework (2003)
Four-level abstraction chain almost identical to Ferro's architecture:
1. **Task & Domain Model** → Ferro's `ServiceDef` + `StateMachine`
2. **Abstract UI (AUI)** → Ferro's `IntentGraph`
3. **Concrete UI (CUI)** → Ferro's `Renderer` output
4. **Final UI (FUI)** → Rendered HTML/JSON/native code

Source: [W3C CAMELEON](https://www.w3.org/community/uad/wiki/Cameleon_Reference_Framework), [W3C Abstract UI Models](https://www.w3.org/TR/abstract-ui/)

#### MECANO (Puerta & Eriksson, 1996)
"Beyond Data Models for Automated User Interface Generation" — pioneered semantic enrichment of data models for UI generation. Ferro's `FieldMeaning` is exactly this concept.

Source: [Semantic Scholar](https://www.semanticscholar.org/paper/Beyond-Data-Models-for-Automated-User-Interface-Puerta-Eriksson/2df4fa562abb4a5426ae6e0c2d351dc5e4851143)

#### SAP Fiori Elements + OData Annotations
Production-proven at massive scale. OData annotations enrich service metadata with semantic UI information — functionally identical to `FieldMeaning` driving renderer decisions. "This field is a currency amount" → render with currency formatting.

Source: [SAP Fiori Elements](https://cap.cloud.sap/docs/advanced/fiori), [OData UI Annotations](https://sapui5.hana.ondemand.com/sdk/docs/topics/83c89ccef12f48ab98f6c3811bd025b3.html)

### Relevant Patents

| Patent | What it Claims | Risk to Ferro |
|--------|---------------|---------------|
| US7941438B2 | Platform-independent "Interaction Units" generating code for Desktop/Web/Mobile/Voice from formal specs | HIGH — same structural pattern as IntentGraph→Renderer |
| US8930831B2 | Business process definition (steps + actions) → auto-generated UI | MODERATE — StateMachine → UI is structurally similar |
| US20070266329A1 | State diagram model → GUI generation | MODERATE — StateMachine→UI claimed, though inverse direction |
| US10810358 | Traversing node list with metadata → rendering widgets by type | MODERATE — FieldMeaning-driven rendering |
| US20100106547A1 | Document definitions → auto-defined workflow states → UI documents | MODERATE — state→auto-generated UI direction |

### Existing Standards Comparison

| Standard | Entities | Operations | Semantic Types | State Machine | UI Generation | Status |
|----------|----------|------------|----------------|---------------|---------------|--------|
| OpenAPI 3.1 | Schemas | Full CRUD | `format` field | No | Via JSON Schema forms | Active |
| GraphQL SDL | Types | Query/Mutation | No | No | Via introspection tools | Active |
| Hydra | Classes + Properties | hydra:Operation | Via Schema.org | No | API Platform Admin (production) | Low activity |
| Siren | Entities | Actions + fields | HTML5 input types | No | Actions are form specs | Stable |
| A2UI | Components | Actions + context | Catalog-defined | No | Full declarative UI | Active (v0.9) |
| JSON-LD/Schema.org | 827 types | Action vocabulary | Rich semantic types | No (but actionStatus) | Indirect | Active |
| **Ferro (planned)** | ServiceDef | Actions + StateMachine | FieldMeaning enum | Yes (schema-only) | IntentGraph→Renderer | In development |

**Gap Ferro fills:** No existing standard combines semantic field types + schema-only state machines + intent graph generation + pluggable renderers in a single coherent pipeline. Each layer exists individually; Ferro's contribution is the integrated pipeline within a server-side framework.

### Novelty Assessment

| Aspect | Novel? | Prior Art |
|--------|--------|-----------|
| Generating UI from data models | No | MECANO (1996), MB-UIDEs (1990s-2000s) |
| Multi-level abstraction (Domain→Abstract→Concrete→Final) | No | CAMELEON (2003), USIXML, IFML |
| Semantic type annotations driving UI rendering | No | SAP OData annotations, Metabase semantic types |
| State machines driving UI generation | No | US20070266329A1, US20100106547A1 |
| Schema-only state machines (no closures, fully serializable) | Partially | XState philosophy, but XState still has runtime |
| State-dependent directed graph as intermediate representation | Partially | CAMELEON AUI is tree-based, not graph-based |
| Full pipeline in server-side framework with schema-only constraint | Moderate | No exact precedent found |
| IntentGraph dynamically generated from current state context | Moderate | Static AUI models don't account for runtime state |
</prior_art>

<code_examples>
## Code Examples

### JSON Schema Generation from Rust Types
```rust
// Source: schemars documentation
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldMeaning {
    Identifier,
    Money,
    Email,
    Status,
    // ... other variants
    #[serde(untagged)]
    Custom(String),
}

// Generate schema:
let schema = schema_for!(ServiceDef);
let json = serde_json::to_string_pretty(&schema).unwrap();
// Publish as part of protocol spec release
```

### Protocol Version Declaration
```json
{
  "protocol": {
    "name": "ferro-projections",
    "version": "0.1.0-draft",
    "schema": "https://ferro-rs.dev/protocol/v0.1/schema.json"
  },
  "service": {
    "name": "order",
    "display_name": "Order",
    "fields": [
      {
        "name": "id",
        "data_type": "integer",
        "meaning": "identifier",
        "required": true,
        "is_list": false
      },
      {
        "name": "total",
        "data_type": "float",
        "meaning": "money",
        "required": true,
        "is_list": false
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
}
```

### Extension Mechanism Example
```json
{
  "name": "order",
  "fields": [...],
  "x-acme-priority": "high",
  "extensions": [
    {
      "uri": "https://acme.com/ext/audit-trail",
      "critical": false,
      "data": {
        "track_field_changes": true,
        "retention_days": 90
      }
    }
  ]
}
```

### Error Object Structure
```json
{
  "code": 1001,
  "message": "Invalid field meaning",
  "data": {
    "detail": "FieldMeaning 'currency' is not recognized. Did you mean 'money'?",
    "source": {
      "pointer": "/fields/2/meaning"
    }
  }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WSDL/SOAP for service description | OpenAPI 3.x + JSON Schema | 2015-2020 | JSON-based, human-readable, universal adoption |
| Custom UI generation tools | A2UI / Open-JSON-UI protocols | 2025-2026 | Agent-driven declarative UI is now an industry direction |
| MCP as tools-only | MCP Apps (SEP-1865) | 2025 | MCP servers can now serve interactive UI surfaces |
| Manual patent searches | AI-assisted prior art analysis | 2024-2025 | Easier to find prior art, harder to hide from it |
| Protocol specs as PDFs | Living docs (mdBook, Mintlify, Bikeshed) | 2020s | Auto-generated, versioned, searchable |

**Emerging protocol stack (2026):**
1. **MCP** — Agent ↔ Tools & Data (Anthropic)
2. **A2A** — Agent ↔ Agent (Google)
3. **AG-UI** — Agent ↔ User transport (CopilotKit)
4. **A2UI / Open-JSON-UI** — Declarative UI specification (Google / OpenAI)

**Ferro's position:** Sits between the service layer and the UI specification layer. ServiceDef→IntentGraph is the "brain" that decides WHAT UI to generate. A2UI/Open-JSON-UI are potential render targets for HOW to express that UI.

**New tools to consider:**
- `schemars` 1.x: Major rewrite with better JSON Schema 2020-12 support
- Google A2UI: Potential rendering target for IntentGraph output
- MCP Apps: Could serve Ferro-generated UIs as MCP app surfaces

**Deprecated/outdated:**
- USDL: Dead since ~2012
- WADL: Superseded by OpenAPI
- XForms: Conceptually relevant but W3C spec has no modern adoption
</sota_updates>

<open_questions>
## Open Questions

1. **Should the protocol define IntentGraph wire format or only ServiceDef?**
   - What we know: ServiceDef is stable and well-defined. IntentGraph is planned but not yet implemented (Phase 88-89).
   - What's unclear: Whether external consumers need to receive IntentGraphs, or only ServiceDefs (with graphs generated server-side).
   - Recommendation: Start with ServiceDef-only protocol (Phase 94 plan 01). Add IntentGraph to protocol after Phase 89 validates the design.

2. **Patent filing: provisional vs utility vs open-source?**
   - What we know: Prior art is dense. Broad claims will fail. Open-source ecosystem adoption may be more valuable.
   - What's unclear: Alberto's IP strategy preferences. Whether defensive patents (preventing others from blocking Ferro) or offensive patents (licensing revenue) are the goal.
   - Recommendation: Discuss with Alberto before planning patent documentation. If filing, use provisional patent (12-month window) after Phase 93 field test validates the design. If not filing, focus on open-source spec publication for ecosystem adoption.

3. **How does the protocol relate to MCP?**
   - What we know: Ferro already has ferro-mcp and ferro-api-mcp. MCP is the natural transport for ServiceDef introspection.
   - What's unclear: Should ServiceDef be exposed as an MCP resource? Should IntentGraph generation be an MCP tool?
   - Recommendation: Phase 92 (MCP Introspection) will address this. The protocol spec should be transport-agnostic; MCP is one transport option.

4. **Should the spec target interoperability with A2UI?**
   - What we know: A2UI's component catalog model maps to JSON-UI components. IntentGraph→A2UI rendering is architecturally feasible.
   - What's unclear: Whether A2UI will gain enough adoption to warrant a dedicated renderer.
   - Recommendation: Design the Renderer trait to be pluggable. A2UI renderer can be a future extension. Don't couple the protocol to A2UI.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)

**Protocol Design:**
- [RFC 2360: Guide for Internet Standards Writers](https://datatracker.ietf.org/doc/html/rfc2360) — spec structure
- [RFC 3117: On the Design of Application Protocols](https://www.rfc-editor.org/rfc/rfc3117.html) — design philosophy
- [RFC 2119: Key words for use in RFCs](https://datatracker.ietf.org/doc/html/rfc2119) — conformance language
- [Patterns in application-layer protocol design](https://www.devever.net/~hl/applayer) — protocol patterns taxonomy

**Standards:**
- [OpenAPI 3.1.1](https://spec.openapis.org/oas/v3.1.1.html) — extension mechanism, versioning
- [JSON:API v1.1](https://jsonapi.org/format/) — extension/profile system, error objects
- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25) — capability negotiation, versioning
- [A2UI Specification v0.8](https://a2ui.org/specification/v0.8-a2ui/) — catalog system, declarative UI
- [JSON Schema 2020-12](https://json-schema.org/specification) — vocabulary system

**Prior Art:**
- [W3C CAMELEON Reference Framework](https://www.w3.org/community/uad/wiki/Cameleon_Reference_Framework) — four-level abstraction chain
- [W3C Abstract UI Models](https://www.w3.org/TR/abstract-ui/) — abstract UI specification
- [SAP Fiori Elements + OData Annotations](https://cap.cloud.sap/docs/advanced/fiori) — production semantic UI generation

**Patents:**
- [US7941438B2](https://patents.google.com/patent/US7941438) — Interaction Units for automatic UI generation
- [US8930831B2](https://patents.google.com/patent/US8930831) — UI from business process definition
- [US20070266329A1](https://patents.google.com/patent/US20070266329) — State machine GUI programming

**Tooling:**
- [Schemars](https://graham.cool/schemars/) — JSON Schema generation from Rust
- [mdBook](https://github.com/rust-lang/mdBook) — Rust specification publishing

### Secondary (MEDIUM confidence)
- [MECANO paper (1996)](https://www.semanticscholar.org/paper/Beyond-Data-Models-for-Automated-User-Interface-Puerta-Eriksson/2df4fa562abb4a5426ae6e0c2d351dc5e4851143) — semantic enrichment concept
- [IFML (OMG Standard)](https://www.ifml.org/) — interaction flow modeling
- [Hydra Core Vocabulary](https://www.hydra-cg.com/spec/latest/core/) — hypermedia-driven APIs
- [Rust Serialization Benchmarks](https://github.com/djkoloski/rust_serialization_benchmark) — format performance
- [Generative and Malleable UIs (2025)](https://arxiv.org/html/2503.04084v1) — server-side gap in AI-driven UI generation

### Tertiary (LOW confidence — needs validation)
- [AG-UI Protocol](https://docs.ag-ui.com/) — agent-user transport (new, ecosystem unclear)
- IETF draft on SemVer for protocol specs — draft status, may not advance
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Protocol specification design, JSON Schema, serialization formats
- Ecosystem: OpenAPI, GraphQL, Hydra, Siren, A2UI, MCP, JSON:API
- Patterns: Extension mechanisms, versioning, capability negotiation, error handling
- Prior art: 30+ years of MB-UIDE research, 5 relevant patents, CAMELEON framework
- Pitfalls: Patent density, over-scoping, schema drift, premature standardization

**Confidence breakdown:**
- Standard stack: HIGH — schemars/jsonschema/serde are established, well-maintained
- Protocol design patterns: HIGH — based on analysis of 5+ production protocol specs
- Prior art: HIGH — comprehensive patent and academic search conducted
- Patent strategy: MEDIUM — patent viability assessment is clear, but depends on Alberto's IP goals
- Code examples: HIGH — based on existing ferro-projections types and schemars docs

**Research date:** 2026-02-28
**Valid until:** 2026-03-30 (30 days — protocol design patterns are stable; A2UI/AG-UI ecosystems may evolve)
</metadata>

---

*Phase: 94-protocol-patent*
*Research completed: 2026-02-28*
*Ready for planning: yes*
