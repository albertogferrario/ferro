# Phase 90: Renderer Trait & JSON-UI Renderer - Research

**Researched:** 2026-03-01
**Domain:** Structural intent → JSON-UI component mapping, Renderer trait design
**Confidence:** HIGH

<research_summary>
## Summary

Researched the architecture for mapping derived intents to JSON-UI components. The core question was whether Ferro's 7 intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track) naturally map to the existing 20 JSON-UI components — or whether the mapping feels forced (which would signal the intent taxonomy needs revision).

**Result: 6 of 7 intents map naturally.** Analyze is limited by the absence of chart components but can still produce a useful sortable Table view. The mapping follows the same structural logic as SAP Fiori floorplans (List Report→Browse, Object Page→Focus, Wizard→Collect, Worklist→Process, Overview Page→Summarize) and Google A2UI's catalog-constrained composition, but with Ferro's novel twist: the intent is DERIVED from ServiceDef structure, not manually specified.

The Renderer trait should live in `ferro-projections` with `serde_json::Value` output (keeping it framework-independent). The `JsonUiRenderer` implementation generates JSON conforming to the JsonUiView schema. Phase 91 wires it into the framework with typed helpers.

**Primary recommendation:** Implement as three layers: (1) `Renderer` trait with `RenderContext` in ferro-projections, (2) intent→layout strategy mapping (which components and how they're arranged), (3) `FieldMeaning`→component mapping (which input/display component for each field semantic). Keep it synchronous — this is pure structural transformation, no I/O.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde_json | 1 | RenderOutput as Value | Already in use, framework-independent output type |
| ferro-projections | (workspace) | ServiceDef, Intent, IntentScore types | Input types for rendering |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| None | - | - | No new dependencies needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `serde_json::Value` output | Associated type `type Output` | Associated types prevent `dyn Renderer` trait objects. Value is universal and JsonUiView deserializes from it. |
| Sync trait | Async trait | No I/O in rendering — pure structural mapping. Async adds unnecessary complexity. |
| Renderer in ferro-projections | Renderer in framework | Renderer trait is part of the projection pipeline. Implementation CAN live in framework (Phase 91), but the trait belongs with the types it consumes. |
| Single `render()` method | Per-intent methods (`render_browse()`, `render_focus()`) | Single method keeps the trait simple. Internal dispatch via match on Intent. |

**Installation:**
```toml
# No new dependencies — uses existing workspace crates
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Module Structure
```
ferro-projections/src/
├── render/
│   ├── mod.rs           # Renderer trait, RenderContext, RenderOutput
│   ├── json_ui.rs       # JsonUiRenderer implementation
│   ├── intent_layout.rs # Intent → layout strategy mapping
│   └── field_map.rs     # FieldMeaning → component mapping
├── derive.rs            # Existing: derive_intents()
├── service.rs           # Existing: ServiceDef
└── ...                  # Existing modules
```

### Pattern 1: Renderer Trait (framework-independent)
**What:** A sync trait in ferro-projections that takes ServiceDef + IntentScores and produces a `serde_json::Value` conforming to the target format.
**When to use:** Any intent-to-UI mapping.
**Design:**
```rust
/// Context for rendering decisions.
pub struct RenderContext {
    /// Which intent index to render (0 = primary).
    pub intent_index: usize,
    /// Current state machine state (for Process/Track intents).
    pub current_state: Option<String>,
    /// Rendering mode.
    pub mode: RenderMode,
}

pub enum RenderMode {
    /// Display mode — read-only view of data.
    Display,
    /// Input mode — form for data entry/editing.
    Input,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Display,
        }
    }
}

/// Maps ServiceDef + derived intents → renderable output.
pub trait Renderer: Send + Sync {
    /// Render a service projection as a JSON value.
    ///
    /// The output JSON conforms to the target format
    /// (e.g., ferro-json-ui/v1 schema for JsonUiRenderer).
    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &RenderContext,
    ) -> Result<serde_json::Value, Error>;
}
```

**Why this design:**
- `Send + Sync` matches existing Ferro trait patterns (UserProvider, Middleware)
- `serde_json::Value` output keeps ferro-projections independent of framework crate
- `RenderContext` is minimal but extensible — new fields can be added without breaking
- Sync (not async) because rendering is pure structural mapping, no I/O
- `&[IntentScore]` passes the full ranked list so renderers can use secondary intents for secondary UI sections

### Pattern 2: Intent → Layout Strategy Mapping
**What:** Each intent maps to a "layout strategy" — a combination of primary and secondary JSON-UI components.
**When to use:** Inside JsonUiRenderer to decide page structure.

| Intent | Primary Component | Secondary Components | Layout |
|--------|-------------------|---------------------|--------|
| Browse | Table | Pagination, Breadcrumb | List view with filterable columns |
| Focus | Card + DescriptionList | Tabs (relationships), Badge (status) | Detail view with sections |
| Collect | Form | Input/Select/Switch per field | Data entry form |
| Process | Card (state display) | Badge (current state), Button (transitions), Alert (guards) | Workflow control panel |
| Summarize | Card grid | Text (KPI values), Progress (percentages), Badge | Dashboard of metrics |
| Analyze | Table (sortable) | DescriptionList (summary stats) | Analytical table view |
| Track | Table | Badge (status column), Pagination | Status timeline/audit list |

**SAP Fiori Floorplan Validation:**

| Ferro Intent | SAP Fiori Analog | Mapping Quality |
|-------------|-----------------|-----------------|
| Browse | List Report | Natural — same purpose, same primary component |
| Focus | Object Page | Natural — detail view with sections and navigation |
| Collect | Wizard / Object Page (edit) | Natural — form-based data capture |
| Process | Worklist | Natural — state machine progression with actions |
| Summarize | Overview Page | Natural — KPI cards and metrics |
| Analyze | Analytical List Page | Limited — no chart components in JSON-UI yet |
| Track | (No direct SAP analog) | Novel — timeline/audit view, renders as status table |

### Pattern 3: FieldMeaning → Display Component Mapping
**What:** Each FieldMeaning maps to a specific display treatment.
**When to use:** When generating read-only views (Focus, Browse, Summarize, Track).

| FieldMeaning | Display Component | Configuration |
|-------------|-------------------|---------------|
| Identifier | Text (muted) | Small/secondary text, often in Card subtitle |
| ForeignKey | (hidden) | Not directly displayed; resolved via relationship |
| EntityName | Text (bold) | Primary identifier — Card title, Table primary column |
| Email | Text | Display as-is (mailto: link deferred to HTML renderer) |
| Phone | Text | Display as-is (tel: link deferred to HTML renderer) |
| Url | Text | Display as-is (clickable link deferred to HTML renderer) |
| ImageUrl | Avatar | Round image thumbnail |
| Money | Text | DescriptionList with ColumnFormat::Currency |
| Percentage | Progress or Text | Progress bar (0-100) or "X%" text |
| Quantity | Text | Numeric display |
| Status | Badge | Badge variant based on value |
| Category | Badge (secondary) | Badge with secondary variant |
| Boolean | Badge | "Yes"/"No" or "Active"/"Inactive" |
| FreeText | Text | Multi-line, potentially truncated in table columns |
| CreatedAt | Text | DescriptionList with ColumnFormat::DateTime |
| UpdatedAt | Text | DescriptionList with ColumnFormat::DateTime |
| DateTime | Text | DescriptionList with ColumnFormat::Date |
| Sensitive | (hidden) | Never displayed in read-only views |
| Custom(_) | Text | Default text rendering |

### Pattern 4: FieldMeaning → Input Component Mapping
**What:** Each FieldMeaning maps to a specific input component.
**When to use:** When generating forms (Collect intent, Focus with RenderMode::Input).

| FieldMeaning | Input Component | Input Type | Notes |
|-------------|----------------|------------|-------|
| Identifier | (hidden Input) | hidden | Auto-generated, not user-editable |
| ForeignKey | Select | - | Options populated from relationship target |
| EntityName | Input | text | Required by default |
| Email | Input | email | Browser-level validation |
| Phone | Input | tel | |
| Url | Input | url | |
| ImageUrl | Input | url | File upload deferred to later phase |
| Money | Input | number | step="0.01" |
| Percentage | Input | number | min="0" max="100" |
| Quantity | Input | number | |
| Status | Select | - | Options from state machine states or known values |
| Category | Select | - | Options TBD (no enum values in FieldDef yet) |
| Boolean | Switch | - | Toggle component |
| FreeText | Input | textarea | Multi-line text |
| DateTime | Input | datetime-local | |
| Sensitive | Input | password | Never pre-filled |
| Custom(_) | Input | text | Default text input |

### Pattern 5: Relationship → Navigation Component Mapping
**What:** NavigationHint drives how related entities appear in the UI.
**When to use:** When rendering Focus views and secondary sections.

| NavigationHint | Component | Behavior |
|---------------|-----------|----------|
| Inline | DescriptionList item | Embed target entity name inline |
| Link | Text (as link) | Navigable link to related entity |
| Tab | Tabs child | Separate tab with related entity table |
| Nested | Table (child) | Nested table within current view |
| Hidden | (omitted) | Not rendered in default view |

### Anti-Patterns to Avoid
- **Hard-coding JSON-UI component structure:** Use the mapping tables above, not literal JSON construction. Changes to JSON-UI schema should only require updating the mapping functions.
- **Rendering data values:** The Renderer produces component STRUCTURE with `data_path` bindings. Data resolution happens at HTML render time. The Renderer never sees actual data.
- **Coupling to framework crate:** The Renderer produces `serde_json::Value` matching the JsonUiView schema. It does NOT import JsonUiView types. Phase 91 adds typed integration.
- **Async rendering:** No I/O happens during rendering. Keep it sync.
- **Over-engineering RenderContext:** Start minimal (intent_index, current_state, mode). Expand in later phases as needs emerge.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON-UI schema conformance | Manual JSON construction | Builder helpers generating `serde_json::json!({})` matching known schema | Reduces drift risk, easier to test |
| Field visibility logic | Custom readable/writable rules | Existing `FieldDef.readable`/`FieldDef.writable` booleans | Already defined in Phase 86 |
| Intent derivation | Re-derive in renderer | `derive_intents()` output from Phase 89 | Renderer consumes intents, doesn't compute them |
| State machine analysis | Custom transition queries | `StateMachine::events_from_state()` existing API | Already implemented in Phase 85 |
| Component unique keys | Manual key generation | `format!("{}-{}", intent, field.name)` pattern | Deterministic, collision-free |

**What SHOULD be hand-rolled:**

| Problem | Why Hand-Roll |
|---------|--------------|
| Intent → layout strategy mapping | Core Phase 90 logic — no library exists for this |
| FieldMeaning → component selection | Domain-specific mapping tables — Ferro's own vocabulary |
| Component tree assembly | Combines multiple mapping decisions into coherent page structure |

**Key insight:** The Renderer is a STRUCTURAL MAPPER, not a visual renderer. It decides WHICH components and HOW to arrange them. The existing JSON-UI HTML renderer handles actual visual rendering. Phase 90 adds the "what to render" logic; the "how to render it" already exists.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Coupling Renderer to Framework Crate
**What goes wrong:** ferro-projections imports JsonUiView from framework, creating a circular or heavyweight dependency.
**Why it happens:** Natural instinct to use typed components instead of raw JSON.
**How to avoid:** Renderer outputs `serde_json::Value`. Builder helpers in ferro-projections construct JSON matching the JsonUiView schema without importing the types. Phase 91 adds typed integration in the framework crate.
**Warning signs:** `framework` appearing in ferro-projections Cargo.toml dependencies.

### Pitfall 2: Rendering Actual Data Instead of Structure
**What goes wrong:** Renderer tries to fill in field values, making it depend on runtime data.
**Why it happens:** Confusing "what to show" with "showing it."
**How to avoid:** Renderer produces component trees with `data_path` bindings (e.g., `"/data/items"` for a Table). Data resolution happens at HTML render time via the existing `resolve_path()` system.
**Warning signs:** Renderer function signatures requiring `data: &serde_json::Value` parameter.

### Pitfall 3: Missing FieldMeaning Coverage
**What goes wrong:** Renderer handles 5 of 18 FieldMeaning variants, falls through to Text for the rest. Output looks generic/unhelpful.
**Why it happens:** Not exhaustively mapping every FieldMeaning variant.
**How to avoid:** Use Rust match exhaustiveness — match on all 18 FieldMeaning variants explicitly. Custom(String) gets default treatment.
**Warning signs:** Large `_ =>` catch-all in FieldMeaning match expressions.

### Pitfall 4: Intent Layout Assumes Single Intent
**What goes wrong:** Renderer uses only the primary intent, producing a one-dimensional view (e.g., pure table for Browse, ignoring that the service also scores high for Summarize).
**Why it happens:** Using only `intents[0]` without considering secondary intents.
**How to avoid:** Primary intent drives layout strategy. Secondary intents can enrich it — e.g., a Browse view with Summarize signals can include a stats Card above the Table. Start simple (primary only) but design the API to support secondary intent enrichment.
**Warning signs:** Renderer ignoring `intents[1..]`.

### Pitfall 5: Forced Analyze Mapping
**What goes wrong:** Trying to generate chart-like views with JSON-UI components that don't support charts. Result looks awkward.
**Why it happens:** Analyze intent expects visualization but JSON-UI only has Table and Progress.
**How to avoid:** Acknowledge the limitation. Analyze renders as a sortable Table with DateTime and numeric columns. Add a comment in PLAN.md that chart component support is a future enhancement. Don't fake charts with Progress bars.
**Warning signs:** Creative abuse of Progress or Badge to simulate chart elements.

### Pitfall 6: Forgetting System Fields in Display
**What goes wrong:** Rendering Identifier, CreatedAt, UpdatedAt prominently in Browse tables, cluttering the view.
**Why it happens:** Not filtering system fields from primary display.
**How to avoid:** Use the existing `is_system_field()` helper from derive.rs (checks for Identifier, CreatedAt, UpdatedAt). System fields are excluded from primary Table columns but shown in Focus detail views.
**Warning signs:** Every Table has "id", "created_at", "updated_at" as visible columns.
</common_pitfalls>

<code_examples>
## Code Examples

### Renderer Trait Definition
```rust
// Source: Designed for ferro-projections/src/render/mod.rs
use crate::{Error, IntentScore, ServiceDef};

/// Rendering mode — display (read-only) or input (forms).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderMode {
    Display,
    Input,
}

/// Context for rendering decisions.
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub intent_index: usize,
    pub current_state: Option<String>,
    pub mode: RenderMode,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Display,
        }
    }
}

/// Maps ServiceDef + derived intents → renderable output as JSON.
pub trait Renderer: Send + Sync {
    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &RenderContext,
    ) -> Result<serde_json::Value, Error>;
}
```

### JsonUiRenderer Intent Dispatch (Conceptual)
```rust
// Source: Conceptual design for ferro-projections/src/render/json_ui.rs
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &RenderContext,
    ) -> Result<serde_json::Value, Error> {
        let intent = intents.get(ctx.intent_index)
            .ok_or(Error::Definition("No intent at given index".into()))?;

        let components = match &intent.intent {
            Intent::Browse => self.render_browse(service, ctx),
            Intent::Focus => self.render_focus(service, ctx),
            Intent::Collect => self.render_collect(service, ctx),
            Intent::Process => self.render_process(service, ctx),
            Intent::Summarize => self.render_summarize(service, ctx),
            Intent::Analyze => self.render_analyze(service, ctx),
            Intent::Track => self.render_track(service, ctx),
            Intent::Custom(_) => self.render_focus(service, ctx), // Fallback
        };

        Ok(serde_json::json!({
            "$schema": "ferro-json-ui/v1",
            "title": service.display_name.as_deref()
                .unwrap_or(&service.name),
            "components": components,
        }))
    }
}
```

### Browse Intent → Table Component (Conceptual)
```rust
// Source: Conceptual FieldMeaning → Table column mapping
fn render_browse(&self, service: &ServiceDef, ctx: &RenderContext) -> Vec<Value> {
    // Filter to readable, non-system fields for columns
    let columns: Vec<Value> = service.fields.iter()
        .filter(|f| f.readable && !is_system_field(&f.meaning))
        .map(|f| {
            let mut col = serde_json::json!({
                "key": &f.name,
                "label": field_display_name(&f.name),
            });
            // Add format hints based on FieldMeaning
            match &f.meaning {
                FieldMeaning::Money => { col["format"] = "currency".into(); }
                FieldMeaning::DateTime | FieldMeaning::CreatedAt => {
                    col["format"] = "datetime".into();
                }
                _ => {}
            }
            col
        })
        .collect();

    vec![
        serde_json::json!({
            "type": "Table",
            "key": format!("{}-table", service.name),
            "columns": columns,
            "data_path": "/data/items",
        }),
        serde_json::json!({
            "type": "Pagination",
            "key": format!("{}-pagination", service.name),
            "current_page": 1,
            "per_page": 25,
            "total": 0,
            "base_url": format!("/{}", service.name),
        }),
    ]
}
```

### FieldMeaning → Input Component (Conceptual)
```rust
// Source: Conceptual mapping for Collect intent forms
fn field_to_input(field: &FieldDef) -> Value {
    match &field.meaning {
        FieldMeaning::Email => serde_json::json!({
            "type": "Input",
            "key": format!("input-{}", field.name),
            "name": &field.name,
            "label": field_display_name(&field.name),
            "input_type": "email",
            "required": field.required,
            "data_path": format!("/data/{}", field.name),
        }),
        FieldMeaning::Boolean => serde_json::json!({
            "type": "Switch",
            "key": format!("switch-{}", field.name),
            "name": &field.name,
            "label": field_display_name(&field.name),
            "data_path": format!("/data/{}", field.name),
        }),
        FieldMeaning::FreeText => serde_json::json!({
            "type": "Input",
            "key": format!("input-{}", field.name),
            "name": &field.name,
            "label": field_display_name(&field.name),
            "input_type": "textarea",
            "required": field.required,
            "data_path": format!("/data/{}", field.name),
        }),
        FieldMeaning::Status => serde_json::json!({
            "type": "Select",
            "key": format!("select-{}", field.name),
            "name": &field.name,
            "label": field_display_name(&field.name),
            "required": field.required,
            "data_path": format!("/data/{}", field.name),
            // Options populated from state machine states when available
        }),
        FieldMeaning::Sensitive => serde_json::json!({
            "type": "Input",
            "key": format!("input-{}", field.name),
            "name": &field.name,
            "label": field_display_name(&field.name),
            "input_type": "password",
            "required": field.required,
            // No data_path — never pre-filled
        }),
        // Money, Percentage, Quantity → number inputs
        FieldMeaning::Money => serde_json::json!({
            "type": "Input",
            "key": format!("input-{}", field.name),
            "name": &field.name,
            "label": field_display_name(&field.name),
            "input_type": "number",
            "step": "0.01",
            "required": field.required,
            "data_path": format!("/data/{}", field.name),
        }),
        // Default: text input
        _ => serde_json::json!({
            "type": "Input",
            "key": format!("input-{}", field.name),
            "name": &field.name,
            "label": field_display_name(&field.name),
            "input_type": "text",
            "required": field.required,
            "data_path": format!("/data/{}", field.name),
        }),
    }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual floorplan selection (SAP Fiori) | LLM-driven UI generation (A2UI v0.9) | 2025 | Agents compose UI from catalogs, but selection is still LLM-driven |
| Hardcoded CRUD pages (Refine.dev) | Structural field inference → CRUD | 2023+ | Closest precedent to structural derivation, but limited to 4 operations |
| Screen-by-screen mobile UI | Server-driven UI platforms (Airbnb Ghost, Shopify SDUI) | 2021+ | Backend decides what to show, client renders component trees |
| Design-time abstract UI (W3C MBUI) | Runtime intent derivation (Ferro, planned) | 2026 | Novel — no existing system derives intent from schema structure |

**New tools/patterns to consider:**
- **A2UI v0.9 (Google):** Declarative JSON format for agent-driven UIs. Uses flat adjacency list model with ID references. Catalog-constrained component composition. Transport-agnostic. v0.9 adds validation functions and data model binding. Ferro's JSON-UI is already a similar format — A2UI validates the approach.
- **AG-UI Static Generative UI (CopilotKit):** Agent picks from predefined component library. Similar to Ferro's Renderer consuming an intent + component catalog.
- **Airbnb Ghost Platform:** Section components map data models to UI via SectionComponentType enum dispatch. Same pattern as Ferro's Intent → Component mapping.

**Deprecated/outdated:**
- **W3C MBUI (2012-2014):** 4-level model (Task → AUI → CUI → FUI) is architecturally sound but research stalled. Ferro's pipeline maps to it (ServiceDef→IntentGraph→Renderer→Output) with the key divergence of runtime generation.
- **Manual OData annotations for UI:** SAP's approach requires `@UI.LineItem`, `@UI.Identification` etc. Ferro derives this from structure.

**Ferro's position:** Every existing SDUI system either requires manual specification (SAP), defers to LLM reasoning (A2UI), or hardcodes CRUD (Refine). Ferro's structural derivation → rendering pipeline occupies genuinely unexplored territory.
</sota_updates>

<open_questions>
## Open Questions

1. **Should the Renderer produce data_path bindings or resolved values?**
   - What we know: JSON-UI's existing data binding uses `data_path` strings resolved at render time. The Renderer is a structural mapper, not a data resolver.
   - What's unclear: Whether all intents can work with data_path alone, or if some need pre-computed values.
   - Recommendation: Start with data_path only. If Process intent needs computed "available transitions" that can't be expressed as a data_path, add a `computed_data` field to RenderOutput.

2. **How should Select options be populated for Status/Category fields?**
   - What we know: Select component needs `options: [{value, label}]`. Status fields may have known states from the state machine. Category fields have no enumerated values in FieldDef.
   - What's unclear: Where option values come from at rendering time.
   - Recommendation: For Status fields with a state machine, derive options from `StateMachine.states`. For others, leave options empty — they must be populated at framework integration time (Phase 91). Add a `TODO` marker in the component JSON.

3. **Should secondary intents influence the primary layout?**
   - What we know: A service may score highly for both Browse and Summarize. The primary intent drives layout, but secondary intents could enrich it.
   - What's unclear: Whether cross-intent enrichment is valuable in v1 or over-engineering.
   - Recommendation: Implement primary-only rendering first. Design the API so secondary enrichment can be added without breaking changes. Test with Phase 93 field test scenarios.

4. **Where do action handler URLs come from?**
   - What we know: JSON-UI Actions use `handler: "controller.method"` strings resolved to URLs via framework `resolve_actions()`. The Renderer produces structural components, not resolved URLs.
   - What's unclear: Whether the Renderer should emit placeholder action handlers or leave actions unresolved.
   - Recommendation: Emit action handlers as `"service_name.action_name"` convention. Framework integration (Phase 91) provides the resolver that maps these to actual route URLs.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- ferro-projections crate source (workspace) — All types: ServiceDef, Intent, IntentScore, FieldDef, FieldMeaning, etc.
- framework/src/json_ui/ source (workspace) — All 20 JSON-UI components, builder API, plugin system, HTML renderer
- .planning/phases/85.1-architecture-refinement/85.1-RESEARCH.md — Prior art analysis, structural derivation as core innovation
- .planning/milestones/v9.0-ROADMAP.md — Architecture principles, pipeline design, phase dependencies

### Secondary (MEDIUM confidence)
- [A2UI v0.9 Specification](https://a2ui.org/specification/v0.9-a2ui/) — Catalog-constrained component composition, adjacency list model, data binding
- [A2UI Catalogs](https://a2ui.org/catalogs/) — Catalog schema structure, custom component definition
- [SAP Fiori Floorplan Overview](https://experience.sap.com/fiori-design-web/floorplan-overview/) — List Report, Object Page, Overview Page, Worklist, Wizard, Analytical List Page
- [Airbnb Server-Driven UI (Ghost Platform)](https://medium.com/airbnb-engineering/a-deep-dive-into-airbnbs-server-driven-ui-system-842244c5f5) — Section components, SectionComponentType dispatch, data model reusability
- [Rust Type Registry Patterns](https://willcrichton.net/rust-api-type-patterns/registries.html) — Type-safe registries, trait-based dispatch
- [Google A2UI Announcement](https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/) — Agent-driven interfaces, catalog of trusted components

### Tertiary (LOW confidence — needs validation during implementation)
- Intent→Layout mapping completeness — Based on analysis of existing components, needs Phase 93 validation
- FieldMeaning→Input mapping for Category/Status — Option population strategy is unresolved
- Secondary intent enrichment value — Theoretical benefit, untested
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Renderer trait design, intent→JSON-UI component mapping
- Ecosystem: A2UI, SAP Fiori floorplans, Airbnb Ghost Platform, Refine.dev
- Patterns: Intent→layout strategy, FieldMeaning→display/input mapping, NavigationHint→component
- Pitfalls: Framework coupling, data vs structure, missing FieldMeaning coverage, forced Analyze mapping

**Confidence breakdown:**
- Renderer trait design: HIGH — follows existing Ferro patterns (UserProvider, Middleware), sync, Send+Sync
- Intent→layout mapping: HIGH — 6/7 natural mappings validated against SAP Fiori analogs
- FieldMeaning→component mapping: HIGH — exhaustive coverage of all 18 meanings, two modes (display/input)
- Module structure: HIGH — follows existing ferro-projections organization
- Analyze intent limitation: MEDIUM — acknowledged, serviceable with Table, charts deferred
- Secondary intent enrichment: LOW — theoretical, needs validation

**Research date:** 2026-03-01
**Valid until:** 2026-03-31 (30 days — internal architecture, not fast-moving ecosystem)
</metadata>

---

*Phase: 90-renderer-json-ui*
*Research completed: 2026-03-01*
*Ready for planning: yes*
