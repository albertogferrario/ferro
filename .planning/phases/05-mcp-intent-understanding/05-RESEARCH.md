# Phase 5: MCP Intent Understanding - Research

**Researched:** 2026-01-15
**Domain:** MCP Protocol Enhancement for Semantic/Intent Communication
**Confidence:** HIGH

<research_summary>
## Summary

Researched how to enhance cancer-mcp to communicate application **intent** and **purpose**, not just structure. The goal is enabling agents to understand "why" an app is built the way it is, not just "what" endpoints exist.

Current cancer-mcp provides excellent structural introspection (34 tools, routes, models, services) but lacks semantic context. Agents see `GET /rifugio/{id}` but don't understand it's "fetch shelter details for visitors" with specific SLA requirements.

The MCP specification (2025-11-25) provides the primitives: tool annotations (readOnlyHint, destructiveHint, etc.), rich descriptions, resources for persistent context, and prompts for guided workflows. The gap is **applying these to expose domain knowledge**.

**Primary recommendation:** Enhance tool descriptions with structured semantic context, add a domain glossary resource, and implement "explain" tools that return purpose alongside structure.
</research_summary>

<standard_stack>
## Standard Stack

### Core MCP Protocol (Already Using)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rmcp | 0.12 | MCP server implementation | Rust MCP SDK, already in use |
| serde_json | - | JSON serialization | Schema definitions |
| schemars | - | JSON Schema generation | Tool inputSchema |

### New Dependencies for Intent Layer
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none required) | - | Intent is documentation/metadata layer | No new deps needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom intent format | OpenAPI x-extensions | OpenAPI is for REST, MCP has its own primitives |
| Separate manifest file | Inline doc comments | Inline keeps intent close to code |
| Database domain model | Code annotations | Code annotations are version-controlled |

**Key Insight:** MCP already has the primitives (tool annotations, descriptions, resources). The work is applying them systematically, not adding new dependencies.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Enhancement Structure
```
cancer-mcp/src/
├── tools/
│   └── (existing tools - enhance descriptions)
├── domain/
│   ├── mod.rs           # Domain registry
│   ├── glossary.rs      # Business term definitions
│   └── intent.rs        # Intent extraction helpers
├── resources/
│   └── domain_context.rs # MCP resource: domain knowledge
└── service.rs           # Add domain-aware tool descriptions
```

### Pattern 1: Rich Tool Descriptions with Structured Intent

**What:** Enhance tool descriptions beyond "what" to include "when", "why", and "how"

**Current State (Minimal):**
```rust
#[tool(name = "list_routes", description = "List all routes defined in the application")]
```

**Enhanced State (Rich Intent):**
```rust
#[tool(
    name = "list_routes",
    description = "List all routes defined in the application. \
        Use this to understand the API surface and find endpoints to test or modify. \
        Returns method, path, handler, and middleware chain. \
        For route purpose/business logic, combine with describe_endpoint tool.",
    annotations = {
        readOnlyHint: true,
        title: "Routes: List All Endpoints"
    }
)]
```

### Pattern 2: Domain Glossary Resource

**What:** Expose business domain terms as an MCP resource that agents can query

**Example Implementation:**
```rust
// Expose as MCP Resource with URI: cancer://domain/glossary
{
    "terms": {
        "rifugio": {
            "definition": "A shelter or refuge, typically a mountain cabin for hikers",
            "related_models": ["Rifugio", "Prenotazione"],
            "business_context": "Core entity users search, view, and book"
        },
        "prenotazione": {
            "definition": "Reservation/booking at a shelter",
            "related_models": ["Prenotazione", "Utente", "Rifugio"],
            "business_context": "Transaction representing user intent to stay"
        }
    }
}
```

**When to Use:** When agents need to understand what domain terms mean, not just their database structure.

### Pattern 3: Purpose-Annotated Routes

**What:** Enhance route introspection to include semantic purpose

**Current State:**
```json
{
    "method": "GET",
    "path": "/rifugio/{id}",
    "handler": "controllers::rifugio::show",
    "middleware": ["web"]
}
```

**Enhanced State:**
```json
{
    "method": "GET",
    "path": "/rifugio/{id}",
    "handler": "controllers::rifugio::show",
    "middleware": ["web"],
    "intent": {
        "purpose": "Display shelter details for public viewing",
        "audience": "anonymous_users",
        "sla": "< 200ms p99",
        "related_events": ["ShelterViewed"],
        "guards": []
    }
}
```

### Pattern 4: Explain Tools (Meta-Introspection)

**What:** Tools that explain the "why" behind structure

**Example:**
```rust
#[tool(name = "explain_route", description = "Explain the purpose and context of a specific route")]
async fn explain_route(&self, params: ExplainRouteParams) -> String {
    // Returns: purpose, business logic, related routes, common issues, examples
}

#[tool(name = "explain_model", description = "Explain what a model represents in the business domain")]
async fn explain_model(&self, params: ExplainModelParams) -> String {
    // Returns: domain meaning, relationships, common queries, validation rules
}

#[tool(name = "describe_feature", description = "Describe a business feature across routes, models, events")]
async fn describe_feature(&self, params: DescribeFeatureParams) -> String {
    // Returns: cross-cutting view of a feature like "booking" or "user profile"
}
```

### Anti-Patterns to Avoid
- **Separate intent files disconnected from code:** Keep intent close to the code it describes
- **Over-engineering domain ontologies:** Start simple (glossary + enhanced descriptions)
- **Relying solely on AI inference:** Be explicit about intent; don't assume agents will guess correctly
- **Ignoring MCP primitives:** Use annotations, resources, descriptions—don't reinvent
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tool metadata format | Custom annotation system | MCP tool annotations (readOnlyHint, etc.) | Already in spec, clients understand them |
| Schema definitions | Manual JSON | schemars + JsonSchema derive | Type-safe, auto-generates |
| Resource URIs | Arbitrary strings | MCP resource URI patterns | cancer://domain/{type} is consistent |
| Intent documentation | Separate markdown | Doc comments + enhanced descriptions | Stays with code, version-controlled |
| Domain glossary format | Custom ontology | Simple JSON with term → definition | Start minimal, expand if needed |

**Key Insight:** The MCP spec already solved the metadata format problem. Tool annotations, resource patterns, and description fields exist. The work is using them well, not inventing alternatives.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Intent Drift from Implementation
**What goes wrong:** Domain glossary or route purposes become stale as code changes
**Why it happens:** Intent documentation lives separately from code
**How to avoid:**
- Keep intent in doc comments extracted at runtime
- Validate intent claims (e.g., if "requires auth" claim, check middleware)
- Flag discrepancies in introspection output
**Warning signs:** Agents get confused by contradictory information

### Pitfall 2: Over-Indexing on Formal Ontologies
**What goes wrong:** Spending weeks designing domain models before delivering value
**Why it happens:** DDD enthusiasm leads to elaborate bounded context diagrams
**How to avoid:**
- Start with simple glossary (term → one-sentence definition)
- Add structure (relationships, constraints) only when agents fail without it
- Measure: do agents make better decisions with this context?
**Warning signs:** Long design phases without measurable agent comprehension improvement

### Pitfall 3: Trusting Agent Inference Over Explicit Context
**What goes wrong:** Assuming agents will "figure out" that Rifugio is a shelter
**Why it happens:** Humans infer from Italian easily; agents may not
**How to avoid:**
- Be explicit about domain terms, especially non-English
- Include example values in schemas
- Test with agents to verify comprehension
**Warning signs:** Agents asking clarifying questions that docs should answer

### Pitfall 4: Neglecting Tool Composition Guidance
**What goes wrong:** Agents don't know which tools to combine for complex tasks
**Why it happens:** Each tool is documented in isolation
**How to avoid:**
- In server instructions, document tool workflows
- Add "see also" references in tool descriptions
- Create MCP prompts that orchestrate common workflows
**Warning signs:** Agents calling tools in wrong order or missing prerequisites

### Pitfall 5: Inconsistent Semantic Depth
**What goes wrong:** Some tools have rich descriptions, others are bare
**Why it happens:** Incremental enhancement without consistency pass
**How to avoid:**
- Define a template for tool descriptions (what, when, why, related)
- Review all 34 tools for consistent depth
- Automate lint checks for description quality
**Warning signs:** Agent performance varies wildly by tool category
</common_pitfalls>

<code_examples>
## Code Examples

### Enhanced Tool Description Pattern
```rust
// Source: MCP spec tool annotations + best practices
#[tool(
    name = "list_routes",
    description = "List all HTTP routes defined in the application. \n\n\
        **When to use:** Understanding API surface, finding endpoints to test or modify, \
        verifying route registration after changes.\n\n\
        **Returns:** Method, path, handler function, middleware chain.\n\n\
        **Combine with:** `get_handler` to see implementation, `test_route` to exercise.",
    annotations = {
        readOnlyHint: true,
        idempotentHint: true,
        title: "Routes: List All"
    }
)]
pub async fn list_routes(&self) -> String { /* ... */ }
```

### Domain Glossary Resource
```rust
// Source: MCP resource pattern + domain-driven design
use rmcp::resource;

#[resource(
    uri = "cancer://domain/glossary",
    name = "Domain Glossary",
    description = "Business domain terms and their definitions. \
        Query this to understand what models and routes represent.",
    mime_type = "application/json"
)]
pub async fn domain_glossary(&self) -> String {
    serde_json::json!({
        "terms": {
            "rifugio": {
                "definition": "Mountain shelter/cabin for hikers",
                "models": ["Rifugio"],
                "routes": ["/rifugio", "/rifugio/{id}"],
                "intent": "Core entity users discover and book"
            }
        }
    }).to_string()
}
```

### Explain Tool Implementation
```rust
// Source: Pattern derived from MCP best practices
#[tool(
    name = "explain_route",
    description = "Get detailed explanation of a route's purpose, business context, \
        and implementation notes. Use when understanding WHY a route exists."
)]
pub async fn explain_route(&self, params: Parameters<ExplainRouteParams>) -> String {
    let path = &params.inner.path;

    // 1. Get structural info
    let route = self.find_route(path).await;

    // 2. Extract intent from doc comments
    let handler_docs = self.extract_handler_docs(&route.handler).await;

    // 3. Infer from patterns (guarded = auth required, etc.)
    let inferred = self.infer_intent(&route);

    // 4. Combine into explanation
    serde_json::json!({
        "route": path,
        "purpose": handler_docs.summary,
        "business_context": handler_docs.context,
        "guards": route.middleware.iter().filter(|m| m.is_guard()).collect::<Vec<_>>(),
        "related_routes": self.find_related_routes(path),
        "common_issues": self.known_issues.get(path),
        "usage_examples": handler_docs.examples
    }).to_string()
}
```

### Server Instructions with Workflow Guidance
```rust
// Source: cancer-mcp existing pattern, enhanced
const CANCER_MCP_INSTRUCTIONS: &str = r#"
# Cancer Framework MCP Server

## Tool Workflows

### Understanding a Feature
1. `list_routes` - Find relevant endpoints
2. `explain_route` for each - Understand purpose
3. `list_models` - See data structures
4. `explain_model` - Understand domain meaning
5. `relation_map` - See how models connect

### Debugging an Issue
1. `last_error` - Get recent error
2. `get_handler` - Read the failing code
3. `read_logs` - Context around error time
4. `explain_route` - Understand expected behavior

### Adding a Feature
1. `domain_glossary` resource - Learn domain terms
2. `list_routes` - See existing patterns
3. `list_services` - Understand available services
4. `explain_model` - For models you'll interact with
"#;
```
</code_examples>

<sota_updates>
## State of the Art (2024-2025)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Minimal tool descriptions | Structured descriptions with when/why/how | MCP 2025 | Better agent decision-making |
| Implicit domain knowledge | Explicit glossaries and context resources | 2025 best practices | Agents understand business terms |
| Tool-centric design | Workflow-centric documentation | 2025 guidance | Agents compose tools effectively |
| SSE transport | Streamable HTTP | June 2025 spec | Better for networked servers |
| No annotations | readOnlyHint, destructiveHint, etc. | March 2025 | Safety hints for clients |

**New tools/patterns to consider:**
- **MCP Elicitation (June 2025):** Server can ask user for clarification—useful for ambiguous intent queries
- **MCP Tasks (Nov 2025):** Track long-running operations—relevant if "explain" tools do heavy analysis
- **Context Engineering (2025 trend):** Multi-agent systems coordinating context—cancer-mcp as context source

**Deprecated/outdated:**
- **SSE transport:** Replaced by Streamable HTTP in June 2025 spec
- **Single-tool thinking:** Best practice is tool composition with prompts/workflows
</sota_updates>

<open_questions>
## Open Questions

1. **Where should domain knowledge live?**
   - What we know: Can be doc comments, manifest file, or database
   - What's unclear: Best practice for Rust frameworks
   - Recommendation: Start with doc comments + dedicated domain/ module; expand if needed

2. **How to keep intent synchronized with code?**
   - What we know: Drift is a common problem
   - What's unclear: Automated validation approaches for Rust
   - Recommendation: Runtime extraction from doc comments; validation tool that flags mismatches

3. **Depth vs. breadth of enhancement?**
   - What we know: 34 tools exist; varying description quality
   - What's unclear: Whether to enhance all tools or add new "explain" tools first
   - Recommendation: Add explain tools first (high value); then standardize existing descriptions

4. **Client support for annotations?**
   - What we know: Annotations are "advisory hints", client may ignore
   - What's unclear: Which clients (Claude, ChatGPT, etc.) actually use them
   - Recommendation: Add annotations anyway; they're low-effort and future-proof
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25) - Protocol primitives, tools, resources, annotations
- [MCP Blog: One Year of MCP](http://blog.modelcontextprotocol.io/posts/2025-11-25-first-mcp-anniversary/) - Nov 2025 release notes
- [WorkOS MCP Features Guide](https://workos.com/blog/mcp-features-guide) - Tools, Resources, Prompts best practices
- [MCP Tool Annotations Introduction](https://blog.marcnuri.com/mcp-tool-annotations-introduction) - readOnlyHint, destructiveHint, etc.

### Secondary (MEDIUM confidence)
- [15 Best Practices for MCP Servers](https://thenewstack.io/15-best-practices-for-building-mcp-servers-in-production/) - Architecture patterns verified against spec
- [7 MCP Server Best Practices 2025](https://www.marktechpost.com/2025/07/23/7-mcp-server-best-practices-for-scalable-ai-integrations-in-2025/) - Tool design guidance
- [API Design for LLMs](https://www.gravitee.io/blog/designing-apis-for-llm-apps) - Semantic documentation principles
- [AI Code Assistants 2025](https://www.augmentcode.com/tools/13-best-ai-coding-tools-for-complex-codebases) - How tools handle domain context

### Tertiary (LOW confidence - needs validation)
- General DDD patterns - Not MCP-specific, but relevant for domain modeling
- OpenAPI semantic enrichment approaches - May inform MCP resource design
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: MCP Protocol (2025-11-25 spec)
- Ecosystem: rmcp, MCP server patterns, agent comprehension
- Patterns: Intent representation, domain glossaries, explain tools
- Pitfalls: Intent drift, over-engineering, inconsistent depth

**Confidence breakdown:**
- Standard stack: HIGH - rmcp already in use, no new deps needed
- Architecture patterns: HIGH - Based on MCP spec and best practices
- Pitfalls: HIGH - Common issues in documentation systems
- Code examples: MEDIUM - Conceptual patterns, implementation details TBD

**Research date:** 2026-01-15
**Valid until:** 2026-02-15 (30 days - MCP ecosystem evolving but stable)
</metadata>

---

*Phase: 05-mcp-intent-understanding*
*Research completed: 2026-01-15*
*Ready for planning: yes*
