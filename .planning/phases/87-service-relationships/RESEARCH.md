# Phase 87: Service Relationships - Research

**Researched:** 2026-02-28
**Domain:** Schema-level service relationship definitions for UI generation
**Confidence:** HIGH

<research_summary>
## Summary

Researched how frameworks and specifications model entity relationships at the schema/definition level, with focus on what the Intent Layer (Phase 88-89) needs from relationship metadata.

The problem is well-understood across multiple domains: ER modeling, ORM schemas (Prisma, SeaORM), API specifications (OpenAPI Links, JSON:API, HATEOAS), and GraphQL. The key finding is that **relationships need two dimensions**: structural (cardinality) and navigational (how to present the connection in UI). Most frameworks handle only the structural dimension; Ferro needs both because relationships feed into IntentGraph edge generation.

The existing codebase already has relationship infrastructure at the database level (`relation_map` MCP tool, `FieldMeaning::ForeignKey`, `EdgeRelationship` enum in dependency graph). Phase 87 lifts this to the service definition level as serializable schema.

**Primary recommendation:** Simple `RelationshipDef` struct with `Cardinality` enum and `NavigationHint` enum, added to `ServiceDef` via flat builder methods. No sub-builders, no complex relationship objects. Keep it schema-only.
</research_summary>

<standard_stack>
## Standard Stack

No external libraries needed. This is a pure internal data model within ferro-projections.

### Core
| Component | Source | Purpose | Why |
|-----------|--------|---------|-----|
| `RelationshipDef` | New struct | Declares a service-to-service relationship | Core schema type |
| `Cardinality` | New enum | OneToOne, OneToMany, ManyToOne, ManyToMany | Standard ER cardinality |
| `NavigationHint` | New enum | How renderer should present the relationship | Unique to Ferro's intent architecture |

### Existing Infrastructure (Already Built)
| Component | Location | Relationship to Phase 87 |
|-----------|----------|--------------------------|
| `FieldMeaning::ForeignKey` | `ferro-projections/src/field.rs` | FK fields hint at relationship existence |
| `infer_meaning("_id")` | `ferro-projections/src/field.rs` | Auto-detects FK columns |
| `Relation` struct | `ferro-mcp/src/tools/relation_map.rs` | DB-level relationship introspection |
| `EdgeRelationship` enum | `ferro-mcp/src/tools/dependency_graph.rs` | `BelongsTo`, `HasMany` at graph level |
| `BatchLoad` trait | `framework/src/database/eager_loading.rs` | N+1 prevention for related models |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Flat `RelationshipDef` | Relationship sub-builder | Added complexity for minimal gain; Phase 84 uses flat methods |
| `NavigationHint` enum | Free-form string hints | Enum is exhaustive and validates at compile time |
| Separate relationship module | Inline in `service.rs` | Separate module matches crate CLAUDE.md structure and keeps files focused |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Module Structure (from crate CLAUDE.md)
```
ferro-projections/src/
├── lib.rs              — re-exports (add Relationship types)
├── service.rs          — ServiceDef (add relationships Vec + builder methods)
├── field.rs            — FieldDef, FieldMeaning, DataType (unchanged)
├── relationship.rs     — NEW: RelationshipDef, Cardinality, NavigationHint
└── error.rs            — Error enum (unchanged)
```

### Pattern 1: Two-Dimensional Relationship Schema
**What:** Every relationship has structural (cardinality) and presentational (navigation) dimensions.
**Why:** The Intent Layer needs both to generate correct graph edges. Cardinality determines WHAT to render (single entity vs collection). NavigationHint determines HOW to render (inline embed, link, tab, etc.).
**Prior art:** HATEOAS affordances (structural + action), A2UI surface navigation, ABP Suite navigation properties.

```rust
pub struct RelationshipDef {
    pub name: String,              // Relationship name, e.g., "customer"
    pub target: String,            // Target service name, e.g., "customer"
    pub cardinality: Cardinality,  // Structural dimension
    pub navigation: NavigationHint, // Presentational dimension
    pub foreign_key: Option<String>, // FK field name on owning side
    pub inverse: Option<String>,   // Name of inverse relationship on target
    pub description: Option<String>,
}
```

### Pattern 2: Flat Builder Methods on ServiceDef
**What:** Add relationships via `ServiceDef` builder methods, matching the existing field-adding pattern.
**Why:** Consistency with Phase 84's `.field()`, `.optional_field()`, `.list_field()` API.

```rust
// Full form
ServiceDef::new("order")
    .relationship("customer", "customer", Cardinality::ManyToOne)
    .relationship("line_items", "order_line_item", Cardinality::OneToMany)

// Convenience shorthands
ServiceDef::new("order")
    .belongs_to("customer", "customer")       // Cardinality::ManyToOne
    .has_many("line_items", "order_line_item") // Cardinality::OneToMany
```

### Pattern 3: Cardinality as Standard Four-Way Enum
**What:** OneToOne, OneToMany, ManyToOne, ManyToMany — the complete ER set.
**Why:** Every schema system uses these four (Prisma, SeaORM, JSON:API to-one/to-many, GraphQL implicit). No need to invent new terminology.
**Prior art:** Prisma (`@relation`), SeaORM (`Related` trait), JSON:API (to-one / to-many).

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}
```

### Pattern 4: NavigationHint for Intent Layer
**What:** Enum telling the renderer how to present a relationship in UI.
**Why:** This is Ferro-specific — bridges the gap between structural relationships and UI presentation. Feeds directly into Phase 88 Intent vocabulary and Phase 89 IntentGraph edge generation.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NavigationHint {
    Inline,   // Embed related data in current view (e.g., customer name on order card)
    Link,     // Show as navigable link to related entity
    Tab,      // Show as separate tab in detail view
    Nested,   // Show as nested list/table within current view
    Hidden,   // Relationship exists but not shown in default navigation
}
```

### Anti-Patterns to Avoid
- **Bidirectional relationship objects:** Don't store both sides as a single object. Each service declares its own relationships independently. The `inverse` field is a string hint, not a hard reference. Matches XState's string-reference philosophy.
- **Closures or runtime logic:** Guards, filters, or visibility conditions on relationships must be strings, not closures. This preserves serialization.
- **Relationship inheritance from DB:** Don't auto-import SeaORM relations. ServiceDef relationships are business-level declarations, not database FK mirrors. A DB FK might exist without a meaningful service relationship (and vice versa).
- **Complex relationship metadata:** Don't add cascade rules, join tables, or through-models. That's database concern. ServiceDef only needs what the Intent Layer consumes.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cardinality modeling | Custom relationship graph | Standard 4-way enum | ER modeling is solved since 1976 |
| Relationship validation | Runtime type checking | Compile-time enum exhaustiveness | Rust's type system handles this |
| Serde for new types | Manual JSON parsing | `#[derive(Serialize, Deserialize)]` | Same pattern as Phase 84 types |
| Inverse relationship resolution | Graph traversal at definition time | String references resolved at IntentGraph build | Schema is declaration, not execution |

**Key insight:** This phase is pure data modeling. The complexity lives in Phase 89 (IntentGraph generation) where relationships become graph edges. Phase 87 just needs clean, serializable schema types.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Overloading Relationships with Database Concerns
**What goes wrong:** Adding cascade rules, join table names, or column mappings to RelationshipDef.
**Why it happens:** Tempting to mirror SeaORM's `Related` trait.
**How to avoid:** Remember: ServiceDef is business-level schema for UI generation, not database schema. The `foreign_key` field is the only DB-adjacent field and it's optional.
**Warning signs:** If you're adding `on_delete`, `through`, or `join_table` fields.

### Pitfall 2: Forgetting That Relationships Are Asymmetric
**What goes wrong:** Treating Order→Customer and Customer→Orders as the same relationship.
**Why it happens:** They refer to the same DB foreign key.
**How to avoid:** Each service declares its own perspective. Order has `belongs_to("customer")`. Customer has `has_many("orders")`. The `inverse` field is a documentation hint, not enforcement.
**Warning signs:** Trying to build a single bidirectional relationship struct.

### Pitfall 3: Making NavigationHint Mandatory Before Phase 88 Exists
**What goes wrong:** Requiring navigation hints when the Intent Layer doesn't exist yet to consume them.
**Why it happens:** Designing ahead of the consumer.
**How to avoid:** Give NavigationHint a sensible default. `Link` for to-one, `Nested` for to-many. Let it be overridden but not required.
**Warning signs:** Builder API requires 4+ parameters per relationship.

### Pitfall 4: ManyToMany Without Clear Ownership
**What goes wrong:** ManyToMany relationships don't have a natural "owning side" — no FK field.
**Why it happens:** Join tables are a DB concept; at service level there's no FK.
**How to avoid:** For ManyToMany, `foreign_key` is None. Both services declare the relationship. IntentGraph handles it as bidirectional edges.
**Warning signs:** Trying to force a `foreign_key` value on ManyToMany relationships.
</common_pitfalls>

<code_examples>
## Code Examples

### Basic Relationship Declaration
```rust
// Source: ferro-projections pattern from Phase 84
let order = ServiceDef::new("order")
    .display_name("Order")
    .description("Manages customer orders")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
    .field("total", DataType::Float, FieldMeaning::Money)
    .field("status", DataType::String, FieldMeaning::Status)
    // Relationships
    .belongs_to("customer", "customer")
    .has_many("line_items", "order_line_item");
```

### Full Relationship Builder
```rust
// Explicit form with all options
ServiceDef::new("order")
    .relationship(
        RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
            .foreign_key("customer_id")
            .inverse("orders")
            .navigation(NavigationHint::Link)
            .description("Customer who placed this order"),
    )
    .relationship(
        RelationshipDef::new("line_items", "order_line_item", Cardinality::OneToMany)
            .inverse("order")
            .navigation(NavigationHint::Nested)
            .description("Items in this order"),
    );
```

### Cardinality with Default Navigation
```rust
impl Cardinality {
    /// Default navigation hint for this cardinality.
    pub fn default_navigation(&self) -> NavigationHint {
        match self {
            Cardinality::OneToOne => NavigationHint::Inline,
            Cardinality::ManyToOne => NavigationHint::Link,
            Cardinality::OneToMany => NavigationHint::Nested,
            Cardinality::ManyToMany => NavigationHint::Nested,
        }
    }
}
```

### Serde Round-Trip (Test Pattern)
```rust
#[test]
fn relationship_serde_round_trip() {
    let rel = RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
        .foreign_key("customer_id")
        .navigation(NavigationHint::Link);

    let json = serde_json::to_string(&rel).unwrap();
    let parsed: RelationshipDef = serde_json::from_str(&json).unwrap();
    assert_eq!(rel, parsed);
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| DB-only relationships (FK constraints) | Service-level relationship schema | 2024-2025 (low-code platforms) | Enables UI generation from schema |
| Hardcoded navigation | Declarative navigation hints | 2025 (A2UI, MCP Apps) | Agent-driven interfaces need relationship metadata |
| ER diagram cardinality only | Cardinality + navigation affordance | 2025 (HATEOAS revival, AICF) | UI generation needs both dimensions |

**Relevant industry context:**
- **Google A2UI:** Surfaces have navigation between them but don't formalize relationships between data entities. Ferro's approach is more structured.
- **HATEOAS affordances:** Spring HATEOAS attaches "affordances" to links — similar concept to our NavigationHint. Links tell the client what it CAN do, not just what IS connected.
- **NocoBase/ABP Suite:** Low-code platforms generate navigation UI directly from relationship definitions. Validates our approach of NavigationHint driving intent generation.

**Not relevant:**
- **Prisma schema relations:** Too DB-focused (column mappings, cascade rules). Good for cardinality reference but wrong abstraction level.
- **GraphQL relationships:** Implicit through field types. Too implicit for schema-driven UI generation.
</sota_updates>

<open_questions>
## Open Questions

1. **Should RelationshipDef use a sub-builder or flat struct?**
   - What we know: Phase 84 uses flat `.field()` method. But relationships have more optional fields than FieldDef.
   - What's unclear: Whether `.belongs_to("name", "target")` convenience + `.relationship(RelationshipDef::new(...))` full form is the right balance.
   - Recommendation: Start with both — convenience shorthands for common cases, full `RelationshipDef::new()` for complex ones. Validate during planning.

2. **Should NavigationHint default from Cardinality?**
   - What we know: There's a natural mapping (ManyToOne→Link, OneToMany→Nested). Most relationships follow this.
   - What's unclear: Whether defaults should be applied in the builder or left to the Intent Layer.
   - Recommendation: Apply defaults in builder (less boilerplate for users), allow override. Intent Layer uses whatever is set.

3. **How does ManyToMany work without a visible join service?**
   - What we know: At DB level, join tables exist. At service level, the join entity might not be a service.
   - What's unclear: Whether ManyToMany needs special handling or if both sides just declare the relationship.
   - Recommendation: Both services declare the relationship with the other as target. No join service needed. If the join table has extra data (e.g., order_items with quantity), it becomes its own service with two ManyToOne relationships.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- ferro-projections crate source (`ferro-projections/src/`) — existing patterns and conventions
- ferro-projections/CLAUDE.md — planned module structure (`relationship.rs`)
- v9.0-RESEARCH.md — architectural decisions (schema-only, string guards, no petgraph)
- v9.0-STRATEGY.md — execution strategy for Phase 87

### Secondary (MEDIUM confidence)
- [Prisma Relations](https://www.prisma.io/docs/orm/prisma-schema/data-model/relations) — cardinality patterns and syntax
- [JSON:API Specification v1.1](https://jsonapi.org/format/) — to-one / to-many relationship representation
- [HATEOAS](https://restfulapi.net/hateoas/) — hypermedia affordances on relationship links
- [Spring HATEOAS Affordances](https://pradeepl.com/blog/rest/hateoas/) — action hints on links
- [Entity-Relationship Model](https://en.wikipedia.org/wiki/Entity%E2%80%93relationship_model) — cardinality fundamentals

### Tertiary (LOW confidence - needs validation)
- [NocoBase](https://www.nocobase.com/en/blog/how-to-build-efficient-crud-apps) — low-code relationship-to-UI pattern
- [ABP Suite Navigation Properties](https://abp.io/docs/commercial/latest/abp-suite/creating-many-to-many-relationship) — relationship-driven navigation generation
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: ferro-projections internal data modeling
- Ecosystem: ER modeling, ORM schemas, API specifications, low-code platforms
- Patterns: Two-dimensional relationships (structural + navigational), flat builder API
- Pitfalls: DB concern leakage, asymmetric relationships, ManyToMany ownership

**Confidence breakdown:**
- Cardinality design: HIGH — standard 4-way enum, universally agreed across all sources
- NavigationHint design: MEDIUM — Ferro-specific invention, validated by HATEOAS/low-code prior art but not directly replicated
- Builder API: HIGH — follows established Phase 84 patterns
- Integration with Intent Layer: MEDIUM — Phase 88-89 doesn't exist yet, but relationship data clearly feeds into graph edges

**Research date:** 2026-02-28
**Valid until:** 2026-03-28 (30 days — internal patterns, stable domain)
</metadata>

---

*Phase: 87-service-relationships*
*Research completed: 2026-02-28*
*Ready for planning: yes*
