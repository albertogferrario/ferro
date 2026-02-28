# Phase 88: Intent Layer — Core Types — Research

**Researched:** 2026-02-28
**Domain:** Medium-agnostic intent taxonomy for UI generation from service definitions
**Confidence:** HIGH

<research_summary>
## Summary

Researched 20+ years of abstract UI description systems (UsiXML, MARIA, IFML, XForms, CTT, Cameleon Reference Framework) and modern agent-driven UI protocols (Google A2UI v0.9, FDC3, Android Intents, Alexa APL, AG-UI, Open-JSON-UI, AICF) to design Ferro's Intent vocabulary.

All systems converge on four universal categories of UI operations: **Input/Edit** (accept data), **Output/Display** (show data), **Navigation** (transition between views), and **Action/Control** (trigger computation). Every system also treats **selection from a set** as a first-class operation distinct from free-form input, and separates **navigation** (go somewhere) from **activation** (do something).

**Primary recommendation:** Design 9 Intent variants organized into three families (Orientation, Action, Movement) operating at the Cameleon AUI level — below user goals/tasks but above platform-specific widgets. Keep the vocabulary small and medium-agnostic; let renderers handle the mapping to concrete components.
</research_summary>

<standard_stack>
## Standard Stack

No external libraries needed. Phase 88 produces pure Rust types within the existing `ferro-projections` crate. The "stack" here is the conceptual framework.

### Core Frameworks Referenced
| Framework | Version/Year | Purpose | Why Referenced |
|-----------|-------------|---------|----------------|
| Cameleon Reference Framework | 2003+ | Meta-framework for multi-target UI | Positions our Intent at the right abstraction level (AUI) |
| MARIA | 2009 | Abstract UI interactor taxonomy | Most rigorous interactor classification |
| XForms | W3C 1.1 (2009) | Semantic form controls | Purest "intent not appearance" design |
| Google A2UI | v0.9 (Dec 2025) | Agent-driven UI protocol | Validates our architecture direction |
| FDC3 (FINOS) | 2.2 (2024) | Intent naming standard | Most mature intent vocabulary standard |

### Design Principles (from cross-system synthesis)
| Principle | Source | Application |
|-----------|--------|-------------|
| Specify semantics, not appearance | XForms | `Browse` not `Table`, `Confirm` not `Modal` |
| Four operation categories | UsiXML/MARIA/XForms/IFML/CTT | Input, Output, Navigation, Control |
| Selection is first-class | MARIA/XForms/IFML/UsiXML | `Select` is distinct from `Create`/`Edit` |
| Navigator vs Activator | MARIA | `Navigate` (go somewhere) vs `Execute` (do something) |
| Schema-only, serializable | XState/A2UI/Ferro v9.0 decision | String guards, no closures |
| Small vocabulary | FDC3 (8 prefixes), A2UI (18 components) | Under 12 Intent variants |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Abstraction Level: Cameleon AUI

The Cameleon Reference Framework defines four abstraction levels:

```
Task & Domain  →  "User wants to manage orders"     (Phase 84: ServiceDef)
Abstract UI    →  "Browse collection, Inspect item"  (Phase 88: Intent) ← WE ARE HERE
Concrete UI    →  "Table with sort, Card with fields" (Phase 90: Renderer)
Final UI       →  "HTML/JSON-UI/Voice output"        (Phase 90: RenderOutput)
```

The Intent enum operates at the **Abstract UI** level. It describes *what* interaction is happening without prescribing *how* it looks. This is the correct level because:
- It's below ServiceDef (which describes capabilities, not interactions)
- It's above components (which are rendering-specific)
- It maps cleanly across mediums (HTML, voice, spatial)

### Pattern 1: Three Intent Families

The v9.0 research identified three families, validated by this deeper research:

| Family | Question | Academic Basis |
|--------|----------|---------------|
| **Orientation** | "What can I see?" | MARIA OnlyOutput, IFML ViewComponent, UsiXML Output facet |
| **Action** | "What can I do?" | MARIA Edit+Activator, XForms input/trigger, UsiXML Input+Control |
| **Movement** | "Where can I go?" | MARIA Navigator, IFML NavigationFlow, UsiXML Navigation facet |

### Pattern 2: Intent Enum (9 variants)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    // === Orientation — "What can I see?" ===
    Browse,     // Display collection (IFML: List, MARIA: OnlyOutput+Repeater)
    Inspect,    // Display single item detail (IFML: Details, MARIA: Description)

    // === Action — "What can I do?" ===
    Create,     // Enter new item (MARIA: Edit, XForms: input+submit)
    Edit,       // Modify existing item (MARIA: Edit, XForms: input+setvalue)
    Remove,     // Delete with confirmation (XForms: delete, MARIA: Activator)
    Execute,    // Trigger named action (MARIA: Activator, XForms: trigger)
    Confirm,    // Acknowledge/approve (A2UI: Modal, XForms: message[modal])

    // === Movement — "Where can I go?" ===
    Navigate,   // Go to related resource (MARIA: Navigator, XForms: load)
    Select,     // Pick from options (MARIA: Selection, XForms: select1)
}
```

**Why 9 and not fewer/more:**
- FDC3 uses 8 prefixes for all financial desktop software → 9 is in the right range
- A2UI has 18 components but those are concrete widgets, not intents
- MARIA has ~15 interactor types but includes CUI-level distinctions we don't need
- The v9.0 risk assessment said "under 12 variants" → 9 is safely under

### Pattern 3: Medium-Agnostic Mapping

Every intent must make sense across rendering targets:

| Intent | HTML | Voice | Spatial | API/JSON |
|--------|------|-------|---------|----------|
| Browse | Table/CardList | "You have 5 orders..." | Floating card grid | Array response |
| Inspect | Detail card/page | "Order #123: total $50..." | Expanded panel | Object response |
| Create | Empty form | "What's the name?" | Input panels | POST endpoint |
| Edit | Pre-filled form | "Current name is X. New?" | Edit overlay | PUT endpoint |
| Remove | Confirm dialog | "Delete #123? Say yes." | Gesture prompt | DELETE endpoint |
| Execute | Action button/menu | "Say 'publish' to publish" | Action gesture | POST action |
| Confirm | Modal dialog | "Are you sure?" | Confirm gesture | Confirmation step |
| Navigate | Link/redirect | "Going to customer..." | Teleport/zoom | Link/URI |
| Select | Clickable list | "Choose: 1, 2, or 3" | Selectable objects | Selection endpoint |

### Pattern 4: IntentNode and IntentEdge

The IntentGraph (Phase 89) will be composed of nodes and edges. Phase 88 defines the types:

```rust
pub struct IntentNode {
    pub id: String,
    pub intent: Intent,
    pub label: String,
    pub description: Option<String>,
    pub fields: Vec<String>,  // Which ServiceDef fields are relevant
}

pub struct IntentEdge {
    pub from: String,  // Source node ID
    pub to: String,    // Target node ID
    pub label: String, // Human-readable edge label
    pub guard: Option<String>,  // String guard (schema-only, no closures)
    pub trigger: EdgeTrigger,
}

pub enum EdgeTrigger {
    UserAction(String),   // "click_row", "submit_form", "confirm_delete"
    StateChange(String),  // State machine transition name
    Auto,                 // Automatic (e.g., after Create → Inspect)
}
```

**Why string guards, not closures:** Validated by XState (effects as string references), A2UI (actions as named events), and the v9.0 key decision #1 (schema-only, serializable).

### Pattern 5: IntentGraph Structure

```rust
pub struct IntentGraph {
    pub service: String,      // ServiceDef name
    pub nodes: Vec<IntentNode>,
    pub edges: Vec<IntentEdge>,
    pub entry: String,        // Entry node ID (typically Browse)
}
```

**Why custom graph, not petgraph:** Validated by v9.0 decision #6. Small graphs (5-15 nodes), state-dependent edge filtering, domain-specific logic. petgraph would obscure the intent.

### Anti-Patterns to Avoid
- **Coupling Intent to component type:** Don't use `Table` or `Form` as intents. Intents are `Browse` and `Create` — the renderer picks components.
- **Splitting intents by field type:** Don't create `EditText`, `EditNumber`, `EditDate`. Just `Edit` with field-level semantics from FieldMeaning.
- **Adding layout intents:** Don't add `Row`, `Column`, `Tabs`. Layout is a renderer concern, not an intent.
- **Making Action too generic:** Don't collapse Create/Edit/Remove/Execute into a single `Action(ActionType)`. The distinction matters for rendering and accessibility.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Intent taxonomy from scratch | Custom categorization | Cross-validated taxonomy from MARIA/XForms/IFML | 20+ years of research already solved this |
| Full UIDL transformation pipeline | Cameleon 4-level pipeline | Direct Intent→Renderer mapping | Ferro doesn't need AUI→CUI→FUI; one renderer per target suffices |
| Graph algorithms | Custom BFS/DFS/cycle detection | Simple Vec iteration with ID lookups | 5-15 nodes; algorithmic complexity is irrelevant at this scale |
| Data binding language | Custom expression DSL | FieldMeaning + field names | ServiceDef already carries semantic info; don't re-derive it |
| Navigation state machine | Router abstraction | IntentGraph edge traversal | Navigation emerges from the graph; don't add a separate router |

**Key insight:** The biggest risk is over-engineering. A2UI defines 18 component types, 14 functions, and a full streaming protocol. XForms defines 10 controls and 16 actions. MARIA has 15+ interactor types. Ferro needs **9 Intent variants and 3 supporting structs**. The complexity is in the graph generation (Phase 89) and rendering (Phase 90), not in the types themselves.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Taxonomy Too Granular
**What goes wrong:** Intent enum has 20+ variants, each mapping to a specific rendering pattern. Renderers need special cases for each. Adding a new renderer is O(n) work per intent.
**Why it happens:** Following MARIA too literally (15+ interactor types) or modeling at CUI level instead of AUI.
**How to avoid:** Keep under 12 variants. If two intents would render the same way in most renderers, merge them.
**Warning signs:** Intent variants that differ only in which fields are shown, not in what the user is *doing*.

### Pitfall 2: Taxonomy Too Coarse
**What goes wrong:** Only `View` and `Action` intents. Renderers must inspect node context to decide what to actually render. Intent becomes meaningless.
**Why it happens:** Over-abstracting. Confusing "medium-agnostic" with "information-free."
**How to avoid:** Each intent should prescribe a distinct *interaction pattern* (viewing vs editing vs confirming), even if the concrete component varies.
**Warning signs:** Renderer has `match intent { View => { if fields.len() == 1 { ... } else if has_collection { ... } } }` — branching on data inside the intent match.

### Pitfall 3: Leaking Rendering into Intent
**What goes wrong:** Intent variants named after components (`Table`, `Form`, `Modal`). Breaks when targeting voice or spatial.
**Why it happens:** Designing Intent by looking at JSON-UI components and working backwards.
**How to avoid:** Every intent must pass the "voice test" — does it make sense as a voice interaction? "Browse orders" yes. "Table orders" no.
**Warning signs:** Intent names that are nouns (components) instead of verbs (operations).

### Pitfall 4: Forgetting Stateless Services
**What goes wrong:** IntentGraph generation assumes every service has a state machine. Services without states produce empty or broken graphs.
**Why it happens:** Designing for the complex case first (Order with states) and not testing with simple cases (User profile with just CRUD).
**How to avoid:** Phase 88 types must work with minimal ServiceDefs (just fields, no states, no actions). The default graph for a stateless service is: Browse → Inspect, Browse → Create, Inspect → Edit, Inspect → Remove.
**Warning signs:** Required fields on IntentNode that only make sense with state machines.

### Pitfall 5: Making Edges Too Smart
**What goes wrong:** IntentEdge tries to carry full business logic (preconditions, side effects, data transformations). Edge type becomes as complex as an action definition.
**Why it happens:** Trying to make the graph self-sufficient for rendering without context.
**How to avoid:** Edges carry only: source, target, label, optional guard (string), trigger type. Business logic stays in ActionDef (Phase 86).
**Warning signs:** IntentEdge has more fields than IntentNode.
</common_pitfalls>

<code_examples>
## Code Examples

### Basic Intent Usage
```rust
// Source: Design based on MARIA/XForms/FDC3 cross-validation
use ferro_projections::{Intent, IntentNode, IntentEdge, EdgeTrigger, IntentGraph};

// A typical CRUD service generates this default graph:
let graph = IntentGraph {
    service: "order".into(),
    nodes: vec![
        IntentNode {
            id: "browse".into(),
            intent: Intent::Browse,
            label: "Orders".into(),
            description: Some("List all orders".into()),
            fields: vec!["id".into(), "status".into(), "total".into(), "created_at".into()],
        },
        IntentNode {
            id: "inspect".into(),
            intent: Intent::Inspect,
            label: "Order Details".into(),
            description: None,
            fields: vec![], // Empty = all fields
        },
        IntentNode {
            id: "create".into(),
            intent: Intent::Create,
            label: "New Order".into(),
            description: None,
            fields: vec!["customer_id".into(), "total".into(), "notes".into()],
        },
        IntentNode {
            id: "edit".into(),
            intent: Intent::Edit,
            label: "Edit Order".into(),
            description: None,
            fields: vec!["total".into(), "notes".into(), "status".into()],
        },
        IntentNode {
            id: "remove".into(),
            intent: Intent::Remove,
            label: "Delete Order".into(),
            description: None,
            fields: vec![],
        },
    ],
    edges: vec![
        IntentEdge {
            from: "browse".into(),
            to: "inspect".into(),
            label: "View".into(),
            guard: None,
            trigger: EdgeTrigger::UserAction("click_row".into()),
        },
        IntentEdge {
            from: "browse".into(),
            to: "create".into(),
            label: "New".into(),
            guard: None,
            trigger: EdgeTrigger::UserAction("click_create".into()),
        },
        IntentEdge {
            from: "inspect".into(),
            to: "edit".into(),
            label: "Edit".into(),
            guard: None,
            trigger: EdgeTrigger::UserAction("click_edit".into()),
        },
        IntentEdge {
            from: "inspect".into(),
            to: "remove".into(),
            label: "Delete".into(),
            guard: None,
            trigger: EdgeTrigger::UserAction("click_delete".into()),
        },
        IntentEdge {
            from: "create".into(),
            to: "inspect".into(),
            label: "Created".into(),
            guard: None,
            trigger: EdgeTrigger::Auto,
        },
        IntentEdge {
            from: "edit".into(),
            to: "inspect".into(),
            label: "Saved".into(),
            guard: None,
            trigger: EdgeTrigger::Auto,
        },
    ],
    entry: "browse".into(),
};
```

### State-Dependent Edges (with guards from Phase 85/86)
```rust
// When a service has a state machine, edges gain guards:
IntentEdge {
    from: "inspect".into(),
    to: "execute_ship".into(),
    label: "Ship".into(),
    guard: Some("status == confirmed".into()),  // String guard, not closure
    trigger: EdgeTrigger::UserAction("click_ship".into()),
}
```

### Intent Medium-Agnostic Test
```rust
// Every intent must pass this: does it make sense as a sentence?
// "[User] [intent] [service]"
// "User browses orders"        ✓ Browse
// "User inspects order #123"   ✓ Inspect
// "User creates order"         ✓ Create
// "User edits order #123"      ✓ Edit
// "User removes order #123"    ✓ Remove
// "User executes ship on #123" ✓ Execute
// "User confirms deletion"     ✓ Confirm
// "User navigates to customer" ✓ Navigate
// "User selects order #123"    ✓ Select
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom UI description languages per framework | Converging on A2UI/Open-JSON-UI standards | 2025 | Validates abstract component catalogs |
| Intent = Android-style action strings | Intent = typed enums with graph edges (FDC3 model) | 2024 | Structured navigation over flat dispatch |
| Full UIDL transformation pipelines (4 levels) | Direct intent→renderer mapping | 2025 | Skip CUI level; go AUI→FUI directly |
| Hardcoded component catalogs | Catalog negotiation (A2UI) | Dec 2025 | Renderer declares what it supports |
| Agent generates HTML directly | Agent returns declarative specs (AICF/CopilotKit) | 2025 | Declarative tier is the sweet spot |

**New patterns to consider:**
- **A2UI Surface model:** Multiple independent UI regions in one stream. Could map to Ferro's multi-service dashboard scenarios.
- **FDC3 prefix taxonomy:** `View___`, `Create___`, `Start___` pattern is elegant for naming. Consider adopting for IntentNode labels.
- **Airbnb SectionComponentType:** Enum-based switching between renderings of the same data. Maps to our Intent→Renderer dispatch.

**Deprecated/outdated:**
- **Full Cameleon 4-level pipeline:** Modern systems skip the CUI level, going directly from abstract to rendered.
- **XML-based UIDLs (MARIA, UsiXML):** The concepts are sound but JSON/Rust enums are the modern representation.
- **Petgraph for intent graphs:** Confirmed unnecessary for our scale (5-15 nodes).
</sota_updates>

<open_questions>
## Open Questions

1. **Should `Execute` carry the action name inline?**
   - What we know: MARIA's Activator is generic; FDC3 uses prefixes. Our current design has `Execute` as a bare variant with context from the node.
   - What's unclear: Whether `Execute` should be `Execute { action: String }` or whether the action name lives only on IntentNode.
   - Recommendation: Keep `Execute` bare for now. The action name is on the node label/description. If Phase 89 graph generation needs it, add it then.

2. **Should `Confirm` be a separate intent or a modifier on edges?**
   - What we know: XForms models confirmation as `message[modal]`. A2UI uses Modal component. MARIA has no explicit confirm.
   - What's unclear: Whether confirmation is always a distinct graph node or sometimes just an edge property.
   - Recommendation: Keep as a node for now. A Remove edge can route through a Confirm node. Simpler graph semantics.

3. **IntentContext type needed in Phase 88?**
   - What we know: Phase 89 needs runtime context (current state, user permissions) for graph traversal.
   - What's unclear: Whether the context type should be defined with the core types (Phase 88) or with graph generation (Phase 89).
   - Recommendation: Define a minimal IntentContext in Phase 88 (current_state, current_item_id) and extend in Phase 89.

4. **How does `Select` differ from `Browse` in practice?**
   - What we know: MARIA and XForms treat selection as fundamentally different from display. Select implies "pick one to act on"; Browse implies "see an overview."
   - What's unclear: Whether they're distinct enough for our small graph model.
   - Recommendation: Keep both. Browse is "I'm looking at a list." Select is "I'm choosing from a list for a purpose." The difference matters for renderers (Select highlights the choice action; Browse may not).
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [A2UI Protocol v0.9 Specification](https://a2ui.org/specification/v0.9-a2ui/) — Complete spec analysis: 18 components, action model, surface model, data binding
- [A2UI GitHub Repository](https://github.com/google/A2UI) — JSON schemas, concept docs
- [XForms 1.1 W3C Recommendation](https://www.w3.org/TR/xforms11/) — 10 semantic controls, 16 actions
- [FDC3 Intents Specification v2.2](https://fdc3.finos.org/docs/intents/spec) — 8 intent prefixes, 17 standard intents
- [MARIA W3C Document](https://www.w3.org/wiki/images/3/36/MARIA.pdf) — Full interactor taxonomy
- [Cameleon Reference Framework (W3C)](https://www.w3.org/community/uad/wiki/Cameleon_Reference_Framework) — 4-level abstraction model

### Secondary (MEDIUM confidence)
- [IFML OMG Standard](https://www.omg.org/spec/IFML/1.0/About-IFML) — ViewComponent subtypes (List, Details, Form), event taxonomy — verified against official site
- [ConcurTaskTrees (W3C)](https://www.w3.org/2012/02/ctt/) — Task categories, temporal operators — verified against academic papers
- [UsiXML W3C Incubator](https://www.w3.org/2005/Incubator/model-based-ui/wiki/images/e/ef/UsiXML-MBUI-W3C2009.pdf) — AIO taxonomy (Input/Output/Navigation/Control facets)
- [Alexa APL Commands](https://developer.amazon.com/en-US/docs/alexa/alexa-presentation-language/apl-standard-commands.html) — Voice-first command vocabulary (25 command types)
- [JSON Forms](https://jsonforms.io/docs/uischema/layouts) — UI schema element types (Control, Layout, Group, Categorization)
- [Android Intent Standard Actions](https://developer.android.com/reference/android/content/Intent) — 58 standard actions

### Tertiary (LOW confidence — needs validation)
- [AICF Architecture](https://ai.plainenglish.io/aicf-rethinking-frontend-architecture-in-a-world-where-ai-builds-the-ui-6adb817c1804) — Conceptual framework, not implementation
- [Airbnb Server-Driven UI](https://medium.com/airbnb-engineering/a-deep-dive-into-airbnbs-server-driven-ui-system-842244c5f5) — Enum-based component mapping pattern
- [Open-JSON-UI](https://github.com/CopilotKit/generative-ui) — Still early/evolving
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust enum design for medium-agnostic UI intent
- Ecosystem: 6 academic MBUI systems + 11 modern agent-UI standards
- Patterns: Intent taxonomy, graph-based navigation, AUI-level abstraction
- Pitfalls: Granularity, coupling, stateless services, edge complexity

**Confidence breakdown:**
- Intent taxonomy (9 variants): HIGH — cross-validated across 6+ systems spanning 20 years
- Three-family organization: HIGH — matches UsiXML 4 facets, MARIA categories, v9.0 design
- IntentNode/IntentEdge design: MEDIUM — reasonable but may need adjustment in Phase 89
- IntentGraph structure: MEDIUM — custom graph validated but implementation details TBD
- Code examples: MEDIUM — illustrative, not tested yet

**Research date:** 2026-02-28
**Valid until:** 2026-03-30 (30 days — core concepts are stable; A2UI may evolve)
</metadata>

---

*Phase: 88-intent-core-types*
*Research completed: 2026-02-28*
*Ready for planning: yes*
