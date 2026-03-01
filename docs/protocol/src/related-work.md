# Related Work

This section acknowledges prior art and positions the Ferro Projections Protocol within the broader ecosystem of service-to-UI transformation systems and agent protocols.

## Prior Art

### W3C CAMELEON Reference Framework (2003)

The CAMELEON Reference Framework defines a four-level abstraction chain for multi-target user interfaces:

1. **Task & Domain Model** -- application semantics and user tasks
2. **Abstract UI (AUI)** -- modality-independent interaction objects
3. **Concrete UI (CUI)** -- modality-specific widgets
4. **Final UI (FUI)** -- rendered platform code

Ferro's three-layer pipeline maps directly to CAMELEON's levels 1-3:

| CAMELEON Level | Ferro Equivalent |
|----------------|------------------|
| Task & Domain Model | `ServiceDef` + `StateMachine` |
| Abstract UI | `IntentScore[]` (derived intents) |
| Concrete UI | `Renderer` output (JSON-UI components) |

**Key difference:** CAMELEON's AUI is tree-based and statically defined. Ferro's intent layer is dynamically derived from structural analysis with confidence scores, supporting ambiguous or multi-intent services.

*Reference: Calvary et al., "A Unifying Reference Framework for Multi-Target User Interfaces," Interacting with Computers, 2003.*

### SAP Fiori Elements + OData Annotations

SAP Fiori Elements generates production UIs from OData service metadata enriched with semantic annotations. This is the largest-scale deployment of the annotation-driven UI generation pattern, serving enterprise applications with hundreds of millions of users.

OData annotations enrich service metadata with UI-relevant semantics (e.g., `@UI.LineItem`, `@UI.FieldGroup`, `@UI.Chart`). This is functionally identical to `FieldMeaning` driving renderer decisions in Ferro: both systems annotate data fields with semantic meaning that determines how they are rendered.

**Key difference:** OData annotations are manually authored. Ferro infers structural intent automatically while allowing manual override via `IntentHint`.

*Reference: [SAP Fiori Elements Documentation](https://cap.cloud.sap/docs/advanced/fiori)*

### MECANO (Puerta & Eriksson, 1996)

MECANO pioneered the idea of semantically enriching data models for automatic user interface generation. The system attached presentation-relevant metadata to domain objects, enabling a generation engine to produce appropriate UI representations.

Ferro's `FieldMeaning` is a direct descendant of this concept: semantic annotations on data fields that drive rendering decisions. The contribution is not the idea of semantic enrichment (which dates to 1996) but its realization within a modern server-side framework with confidence-scored intent derivation.

*Reference: Puerta & Eriksson, "Beyond Data Models for Automated User Interface Generation," Semantic Scholar, 1996.*

### Siren (2012)

Siren is a hypermedia type for representing entities with embedded sub-entities, actions, and links. Its action model -- actions with named fields, HTTP method, and content type -- influenced `ActionDef` design in Ferro.

**Key difference:** Siren is a hypermedia format for API responses. Ferro's `ActionDef` is a schema-level declaration that participates in intent derivation (e.g., actions with `transition_trigger` signal the Process intent).

*Reference: [Siren Specification](https://github.com/kevinswiber/siren)*

## Complementary Protocols (2025-2026)

The following protocols occupy adjacent layers in the emerging agent protocol stack. Ferro fills the gap between service definition and UI specification -- none of these protocols cover intent derivation from service structure.

### A2UI (Google, 2025-2026)

A2UI (Agent-to-UI) is a declarative UI specification for AI agents. It defines a component catalog and rendering instructions for agent-generated interfaces.

A2UI covers "what UI to render" but not "what UI does this service need." It is complementary: an `A2UIRenderer` could be an alternative `Renderer` implementation that produces A2UI component trees instead of JSON-UI output.

**Status:** v0.8 stable, v0.9 draft. Production use at Google (Opal, Gemini Enterprise).

*Reference: [a2ui.org](https://a2ui.org/)*

### AG-UI (CopilotKit, 2025-2026)

AG-UI (Agent-User Interaction) is an event-based protocol for agent-to-frontend communication over SSE-over-HTTP. It defines 5 event categories: lifecycle, text, tool calls, state, and special.

AG-UI covers "how to deliver UI" but not "what to derive." It is a transport layer. If Ferro-rendered UIs need to stream to frontends, AG-UI events could carry `Renderer` output.

**Status:** v0.1.11. Adopted by Microsoft Agent Framework, Oracle Open Agent Spec, LangGraph, CrewAI, Google ADK.

*Reference: [docs.ag-ui.com](https://docs.ag-ui.com/)*

### MCP (Anthropic / Linux Foundation, 2024-2026)

The Model Context Protocol provides tool and context access for AI agents. `ferro-mcp` uses MCP as the discovery and introspection transport for projections, with tools for listing, inspecting, rendering, and validating service projections.

MCP is the natural transport for Ferro protocol data in agent-driven workflows. The protocol specification itself is transport-agnostic.

**Status:** Spec version 2025-11-25. Governed by Linux Foundation.

*Reference: [modelcontextprotocol.io](https://modelcontextprotocol.io/)*

### Open-JSON-UI (OpenAI, 2025)

Open-JSON-UI is a token-efficient flat JSON format optimized for LLM-generated UI via structured output. It minimizes token count by using a flat component structure.

Open-JSON-UI optimizes for generation efficiency; Ferro optimizes for derivation from service structure. The two address different stages of the pipeline.

*Reference: [Open-JSON-UI Spec](https://docs.copilotkit.ai/generative-ui/specs/open-json-ui)*

### json-render (Vercel, 2025)

json-render is a generative UI framework where AI generates JSON constrained to a developer-defined component catalog, rendered as React components.

json-render validates the catalog-constrained generation approach that Ferro's JSON-UI component system also follows. It is a single-application framework, not a protocol.

*Reference: [json-render.dev](https://json-render.dev/)*

## Novelty Assessment

Generating UI from data models is not novel (MECANO, 1996). Multi-level abstraction is not novel (CAMELEON, 2003). Semantic annotations driving rendering is not novel (SAP Fiori).

The Ferro Projections Protocol's contribution is the specific combination of:

1. **Schema-only constraints** -- no closures, no runtime logic, fully serializable definitions
2. **Confidence-scored intent derivation** -- 5 structural analyzers producing scored, ranked intents from service structure (no precedent found for this approach)
3. **Pluggable rendering** -- the same `ServiceDef` can produce different UI representations through different `Renderer` implementations
4. **Server-side framework integration** -- intent derivation and rendering operate within the application framework, not as external tooling

No existing protocol formalizes the transformation from service definitions to user intents. A2UI describes what UI to show. AG-UI describes how to transport it. MCP provides tool access. But deriving what meaningful interactions exist from a service's structural properties -- field semantics, writability patterns, state machine topology, relationship cardinality, action signatures -- is an open space that this protocol addresses.
