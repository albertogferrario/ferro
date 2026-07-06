# Phase 93: Field Test & Polish - Research

**Researched:** 2026-03-01
**Domain:** Projection system validation — full-stack field testing of ServiceDef → IntentGraph → Renderer pipeline
**Confidence:** HIGH

<research_summary>
## Summary

Researched the current state of the ferro-projections system to determine what Phase 93 field testing should exercise and what gaps exist between unit-test validation and real-world usage.

The Phase 89-03 validation suite already achieves 100% primary intent accuracy across 12 synthetic ServiceDef fixtures. Phase 90's render tests cover all 7 intents with 52 unit tests and 12 integration tests exercising the full derive→render pipeline. However, all existing tests use inline ServiceDef builders — no tests exercise the actual CLI/MCP toolchain with real projection files in a project directory.

Phase 93's value is bridging the gap between unit-test accuracy and real-world usage: generating projections from actual models, validating them through the CLI/MCP tools, and confirming the full toolchain works end-to-end. The sample app has 3 models (User, Todo, ApiKey) but needs richer domain services to properly exercise all 7 intents.

**Primary recommendation:** Build 7+ representative projection files in the sample app (one per intent), exercise the full MCP toolchain, and fix any issues found in the end-to-end flow. Supplement with hand-crafted complex projections that test multi-analyzer signal interaction.
</research_summary>

<standard_stack>
## Standard Stack

No external libraries needed — Phase 93 uses only existing ferro-projections infrastructure.

### Existing Tools to Exercise
| Tool | Type | Purpose | Status |
|------|------|---------|--------|
| `ferro make:projection` | CLI | Scaffold blank projection module | Built (Phase 91-02) |
| `ferro make:projection --from-model` | CLI | Generate populated ServiceDef from SeaORM model | Built (Phase 92-01) |
| `ferro projection:check` | CLI | Validate projections via ServiceDef::validate() | Built (Phase 92-02) |
| `list_projections` | MCP | Discover projection files via regex scanning | Built (Phase 91-03) |
| `inspect_projection` | MCP | Parse and display single projection details | Built (Phase 91-03) |
| `render_projection` | MCP | Full pipeline: reconstruct → derive → render JSON-UI | Built (Phase 91-03) |
| `validate_projection` | MCP | Single or all-projections validation | Built (Phase 92-02) |
| `projection_coverage` | MCP | Cross-reference models vs projections | Built (Phase 92-03) |

### Sample App Models Available
| Model | Fields | Complexity | Projection Potential |
|-------|--------|------------|---------------------|
| User | id, name, email, password, remember_token, created_at, updated_at | Low | Focus or Collect (depending on writable config) |
| Todo | id, title, description, created_at, updated_at | Low | Browse (simple entity list) |
| ApiKey | id, name, prefix, hashed_key, scopes, last_used_at, expires_at, revoked_at, created_at | Medium | Track (temporal lifecycle with revocation) |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Projection File Convention
```
app/src/projections/
├── mod.rs                    — pub mod declarations
├── user.rs                   — User projection (Focus/Collect)
├── todo.rs                   — Todo projection (Browse)
├── api_key.rs                — ApiKey projection (Track)
├── order.rs                  — Hand-crafted order service (Process)
├── product.rs                — Hand-crafted product catalog (Browse with relationships)
├── report.rs                 — Hand-crafted revenue report (Summarize)
└── analytics.rs              — Hand-crafted sales data (Analyze)
```

### Projection File Structure (from make:projection template)
```rust
use ferro::{
    ServiceDef, FieldDef, FieldMeaning, DataType,
    // + state machine, action, relationship types as needed
};

pub fn service_def() -> ServiceDef {
    ServiceDef::new("name")
        .display_name("Display Name")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        // ... fields, relationships, state machine, actions
}
```

### MCP Tool Discovery Pattern
The MCP tools discover projections by regex-scanning `src/projections/*.rs` files for `pub fn service_def()` functions. This means:
- Function must be named exactly `service_def`
- Function must be `pub fn`
- File must be in `src/projections/` directory
- `render_projection` reconstructs ServiceDef by parsing source code via regex (not compilation)

### Full Pipeline Flow
```
1. Model → make:projection --from-model → projection file
2. projection file → list_projections → discovered
3. projection file → inspect_projection → parsed fields/relationships
4. projection file → render_projection → derive_intents() → JsonUiRenderer → JSON-UI output
5. projection file → validate_projection → ServiceDef::validate() → warnings/errors
6. all models + projections → projection_coverage → coverage % + suggestions
```

### Anti-Patterns to Avoid
- **Testing only generated projections:** `--from-model` generates basic projections without state machines, actions, or relationships beyond FK detection. Hand-crafted projections are needed to exercise the full system.
- **Testing only happy paths:** Need to test edge cases — ambiguous services, empty projections, services that need IntentHint overrides.
- **Ignoring render output validation:** Confirming intent derivation is correct isn't enough — the rendered JSON-UI must also be structurally valid and useful.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Projection generation | Manual ServiceDef typing | `ferro make:projection --from-model` first, then enrich | Ensures field types and FK relationships match actual model |
| Validation | Manual JSON inspection | `ferro projection:check` + `validate_projection` MCP | Catches structural warnings programmatically |
| Coverage tracking | Mental checklist | `projection_coverage` MCP tool | Automated model-to-projection cross-reference |
| Intent verification | Manual derive_intents() calls | `render_projection` MCP tool | Full pipeline including JSON-UI output |

**Key insight:** The toolchain already exists. Phase 93 is about exercising it, not building more tools. If the toolchain doesn't work, that's a bug to fix — not new infrastructure to create.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Regex Parser Limitations
**What goes wrong:** MCP tools fail to parse complex projection files because `reconstruct_service_def` uses regex, not AST parsing
**Why it happens:** Regex can't handle multi-line builder chains with nested expressions, comments in the middle, or conditional logic
**How to avoid:** Keep projection files simple and declarative — straight builder chains without conditional logic or variables
**Warning signs:** `inspect_projection` shows fewer fields than expected, or `render_projection` fails to reconstruct

### Pitfall 2: Field Meaning Mismatch Between Model and Projection
**What goes wrong:** `--from-model` infers wrong FieldMeaning from column names (e.g., "description" → FreeText when it should be EntityName)
**Why it happens:** `infer_meaning()` uses naming heuristics (7 rules) that don't always match domain intent
**How to avoid:** Always review and adjust generated projections — `--from-model` is a starting point, not final output
**Warning signs:** Primary intent is wrong because field meanings skew the analyzer signals

### Pitfall 3: Competing Analyzer Signals
**What goes wrong:** A service designed to be "Process" gets classified as "Track" or vice versa
**Why it happens:** Multiple analyzers contribute overlapping signals — e.g., state machine with linear progression + Status field creates both Process and Track signals
**How to avoid:** Understand the signal weights. Process wins when guards/branching/transition_triggers are present; Track wins with linear progression + Status without guards
**Warning signs:** Primary intent confidence is low (< 0.5), multiple intents have similar scores

### Pitfall 4: System Field Exclusion
**What goes wrong:** Fields with CreatedAt/UpdatedAt/Identifier meanings are excluded from domain analysis, reducing signal strength
**Why it happens:** `is_system_field()` explicitly excludes these from field_meaning analysis
**How to avoid:** Don't use CreatedAt/UpdatedAt for domain-relevant temporal fields — use DateTime instead
**Warning signs:** Services with many system fields produce weak signals, defaulting to Focus fallback

### Pitfall 5: Sample App Models Are Too Simple
**What goes wrong:** Generated projections all derive Focus or Browse because the models lack state machines, complex relationships, and action definitions
**Why it happens:** User (5 fields), Todo (3 fields), ApiKey (8 fields) are all flat CRUD models without workflow semantics
**How to avoid:** Supplement with hand-crafted projections that add state machines, actions, and relationships to exercise all 7 intents
**Warning signs:** All `--from-model` projections derive the same intent
</common_pitfalls>

<code_examples>
## Code Examples

### Example 1: Model-Generated Projection (from --from-model)
```rust
// Generated by: ferro make:projection user --from-model
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn service_def() -> ServiceDef {
    ServiceDef::new("user")
        .display_name("User")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("email", DataType::String, FieldMeaning::Email)
        // password excluded (Sensitive)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
}
// Expected primary intent: Focus (EntityName + Email, mostly readable)
```

### Example 2: Enriched Projection with State Machine (Process intent)
```rust
use ferro::{
    ServiceDef, DataType, FieldMeaning,
    StateMachine, StateDef, Transition,
    ActionDef, GuardDef,
};

pub fn service_def() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .belongs_to("customer", "user")
        .has_many("line_items", "line_item")
        .state_machine(
            StateMachine::new("order_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("submitted"))
                .state(StateDef::new("approved"))
                .state(StateDef::new("shipped"))
                .state(StateDef::new("delivered").final_state())
                .state(StateDef::new("cancelled").final_state())
                .transition(Transition::new("draft", "submit", "submitted"))
                .transition(Transition::new("submitted", "approve", "approved").guard("is_manager"))
                .transition(Transition::new("submitted", "reject", "cancelled"))
                .transition(Transition::new("approved", "ship", "shipped"))
                .transition(Transition::new("shipped", "deliver", "delivered"))
                .transition(Transition::new("draft", "cancel", "cancelled")),
        )
        .guard(GuardDef::new("is_manager").display_name("Manager Approval Required"))
        .action(ActionDef::new("submit").transition_trigger("submit"))
        .action(ActionDef::new("approve").transition_trigger("approve").precondition("is_manager"))
        .action(ActionDef::new("ship").transition_trigger("ship"))
}
// Expected primary intent: Process (guarded transitions, branching, transition triggers)
```

### Example 3: Summarize Projection (read-only metrics)
```rust
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn service_def() -> ServiceDef {
    ServiceDef::new("revenue_dashboard")
        .display_name("Revenue Dashboard")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .read_only_field("total_revenue", DataType::Float, FieldMeaning::Money)
        .read_only_field("profit_margin", DataType::Float, FieldMeaning::Percentage)
        .read_only_field("order_count", DataType::Integer, FieldMeaning::Quantity)
        .read_only_field("avg_order_value", DataType::Float, FieldMeaning::Money)
        .read_only_field("return_rate", DataType::Float, FieldMeaning::Percentage)
}
// Expected primary intent: Summarize (all read-only, Money/Percentage/Quantity fields)
```

### Example 4: Browse Projection with Relationships
```rust
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn service_def() -> ServiceDef {
    ServiceDef::new("product")
        .display_name("Product")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("price", DataType::Float, FieldMeaning::Money)
        .field("category", DataType::String, FieldMeaning::Category)
        .field("image", DataType::String, FieldMeaning::ImageUrl)
        .has_many("reviews", "review")
        .has_many("variants", "product_variant")
        .belongs_to("brand", "brand")
}
// Expected primary intent: Browse (EntityName + Category + OneToMany relationships)
```
</code_examples>

<sota_updates>
## State of the Art (Current System State)

### Test Coverage Summary
| Component | Tests | Coverage |
|-----------|-------|----------|
| ferro-projections unit tests | 309 | All types, builders, serde, validation |
| derive_intents() analyzers | 30+ | All 5 analyzers individually + integration |
| Phase 89-03 validation fixtures | 12 | All 7 intents at 100% primary accuracy |
| JsonUiRenderer unit tests | 52 | All 7 intents, Display + Input modes |
| JsonUiRenderer integration | 12 | Full pipeline derive→render for 5 service types |
| Edge case tests | 8 | Empty, minimal, maximal, ambiguous, hints |
| ferro-mcp projection tools | 17 | list/inspect/render/validate/coverage |
| ferro-cli make_projection | 10 | Template generation, --from-model |
| ferro-cli projection:check | 5 | Validation scanning |

### What's NOT Tested Yet
| Gap | Impact | Phase 93 Action |
|-----|--------|-----------------|
| Real projection files in project directory | MCP regex parser untested on real files | Create actual projection files in sample app |
| Full MCP toolchain end-to-end | Tools tested individually, not as workflow | Exercise list→inspect→render→validate flow |
| --from-model output quality | Generated code not validated against intent expectations | Generate from all 3 models, verify intents |
| Complex multi-analyzer signal interaction | Validation fixtures are designed to cleanly trigger single intents | Build ambiguous services that test signal competition |
| IntentHint effectiveness in practice | Only unit-tested | Add hints to projections where structural derivation is insufficient |
| JSON-UI output usability | Structure verified, not semantic quality | Review rendered output for each intent |
| projection_coverage accuracy | Unit-tested with mocks | Run against real sample app |
</sota_updates>

<open_questions>
## Open Questions

1. **Should field test projections remain in the sample app permanently?**
   - What we know: Sample app is a reference implementation, projections would demonstrate the system
   - What's unclear: Whether 7+ projection files bloat the sample app beyond its purpose
   - Recommendation: Keep them — they serve as documentation and regression test fixtures

2. **Should we add integration tests that exercise the MCP tools against real files?**
   - What we know: MCP tests currently use inline strings, not filesystem scanning
   - What's unclear: Whether MCP integration tests need a test fixture directory or can reuse sample app
   - Recommendation: Use sample app projections as MCP integration test fixtures — avoids duplication

3. **What constitutes "polish" for intent derivation?**
   - What we know: 100% accuracy on 12 synthetic fixtures, no real-world testing yet
   - What's unclear: Whether weight tuning is needed when real projections produce unexpected results
   - Recommendation: Only tune weights if field testing reveals systematic bias (not one-off misclassifications)
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- ferro-projections/src/derive.rs — 12 validation fixtures with signal weight documentation
- ferro-projections/src/render/json_ui.rs — 52 unit tests + 12 integration tests for all 7 intents
- ferro-cli/src/commands/make_projection.rs — --from-model implementation with type/meaning mapping
- ferro-mcp/src/tools/ — 5 projection MCP tools with output format documentation
- .planning/STATE.md — Phase 84-92 accumulated decisions documenting all design choices

### Secondary (MEDIUM confidence)
- app/src/models/ — 3 models (User, Todo, ApiKey) examined for projection potential
- ferro-projections/CLAUDE.md — Crate conventions and anti-patterns

### Tertiary (LOW confidence - needs validation)
- Regex parser robustness — `reconstruct_service_def` in render_projection.rs may have edge cases with complex builder chains (needs testing with real files)
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: ferro-projections full pipeline (ServiceDef → derive_intents → JsonUiRenderer)
- Ecosystem: CLI commands (make:projection, projection:check) + 5 MCP tools
- Patterns: Projection file structure, model-to-projection generation, intent validation
- Pitfalls: Regex parsing limits, signal competition, system field exclusion, simple model limitations

**Confidence breakdown:**
- Standard stack: HIGH — all tools already built and individually tested
- Architecture: HIGH — file conventions and pipeline flow verified against source code
- Pitfalls: HIGH — derived from analyzing signal weights and parser implementation
- Code examples: HIGH — based on existing test fixtures and make:projection template

**Research date:** 2026-03-01
**Valid until:** 2026-03-31 (30 days — internal system, stable)
</metadata>

---

*Phase: 93-field-test-polish*
*Research completed: 2026-03-01*
*Ready for planning: yes*
