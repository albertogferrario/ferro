# Phase 92: MCP Introspection & CLI - Research

**Researched:** 2026-03-01
**Domain:** Internal tooling — model-to-projection bridge, validation, service discovery
**Confidence:** HIGH

<research_summary>
## Summary

Phase 92's original scope ("MCP tools for service discovery, CLI scaffolding") was partially delivered by Phase 91, which expanded beyond its original scope to include `ferro make:projection` (CLI scaffolding) and 3 MCP projection tools (list_projections, inspect_projection, render_projection).

The remaining high-value work is the **model-to-projection bridge**: auto-generating ServiceDef definitions from existing SeaORM model fields. The infrastructure is already in place — `make:api` parses models via syn, `infer_meaning()` maps field names to FieldMeaning, and `explain_model` infers field semantics. Combining these creates a scaffolding command that generates a fully populated ServiceDef instead of an empty template.

Secondary gaps: projection validation (ServiceDef::validate() exists but isn't exposed through CLI/MCP), and project-level service coverage reporting (which models have projections, which don't).

**Primary recommendation:** Rescope Phase 92 to three deliverables: (1) model-aware projection scaffolding via `--from-model`, (2) projection validation CLI + MCP tool, (3) service coverage/discovery MCP tools.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already built — no new dependencies)
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| ferro-projections | workspace | ServiceDef, validate(), derive_intents(), JsonUiRenderer | Phase 84-90 |
| ferro-mcp | workspace | MCP projection tools (list/inspect/render) | Phase 91-03 |
| ferro-cli | workspace | make:projection command | Phase 91-02 |
| syn + quote | workspace | Rust source parsing for model field extraction | Used by make:api, list_models |
| walkdir | workspace | Directory traversal for scanning models/projections | Used by list_models |
| regex | workspace | Source parsing in projection tools | Used by Phase 91-03 |

### Reusable Infrastructure (already exists in codebase)
| Component | Location | What It Provides |
|-----------|----------|------------------|
| Model field parser | `ferro-cli/src/commands/make_api.rs` | syn-based SeaORM model field extraction |
| Model lister | `ferro-mcp/src/tools/list_models.rs` | ModelDetails with fields, types, FK detection |
| Model explainer | `ferro-mcp/src/tools/explain_model.rs` | Semantic field meaning inference |
| Field meaning inferrer | `ferro-projections/src/field.rs:infer_meaning()` | field_name → FieldMeaning mapping |
| ServiceDef validator | `ferro-projections/src/service.rs:validate()` | Warning/Error validation results |
| Projection scanner | `ferro-mcp/src/tools/list_projections.rs` | Discover ServiceDef functions in src/projections/ |
| ServiceDef reconstructor | `ferro-mcp/src/tools/render_projection.rs` | Parse source → build ServiceDef programmatically |

### No External Dependencies Needed
Phase 92 requires zero new external crates. All work reuses existing framework primitives.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: Model-Aware Scaffolding (established by make:api)

`make:api` already demonstrates the full pattern: parse model via syn → extract fields → generate code. Phase 92 follows the same approach for projection scaffolding.

**What:** `ferro make:projection user --from-model` reads the User model, maps fields to ServiceDef builder calls
**Precedent:** `ferro-cli/src/commands/make_api.rs` — ModelInfo struct, field extraction, template generation
**How it works:**

```
Model Field                          ServiceDef Builder Call
───────────────────────────────────────────────────────────
id: i32 (primary_key)            →   .field("id", DataType::Integer, FieldMeaning::Identifier)
name: String                     →   .field("name", DataType::String, FieldMeaning::EntityName)
email: String                    →   .field("email", DataType::String, FieldMeaning::Email)
status: String                   →   .field("status", DataType::String, FieldMeaning::Status)
price: f64                       →   .field("price", DataType::Float, FieldMeaning::Money)
user_id: i32                     →   .field("user_id", DataType::Integer, FieldMeaning::ForeignKey)
is_active: bool                  →   .field("is_active", DataType::Boolean, FieldMeaning::Boolean)
created_at: DateTime             →   .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
updated_at: DateTime             →   .read_only_field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
password_hash: String            →   (excluded — sensitive field)
notes: Option<String>            →   .optional_field("notes", DataType::String, FieldMeaning::FreeText)
```

**Type mapping (Rust type → DataType):**
```
String           → DataType::String
i32/i64/u32/u64  → DataType::Integer
f32/f64          → DataType::Float
bool             → DataType::Boolean
DateTime/NaiveDateTime → DataType::DateTime
NaiveDate        → DataType::Date
Uuid             → DataType::Uuid
Vec<u8>          → DataType::Binary
serde_json::Value → DataType::Json
```

**Field meaning mapping uses existing `infer_meaning()` from ferro-projections** — already handles id, email, created_at, updated_at, *_id, *_at, is_*, has_* patterns.

### Pattern 2: Sensitive Field Exclusion (established by make:api)

`make:api` excludes password, token, secret, api_key fields from API resources. The same pattern applies to projection scaffolding — sensitive fields should be excluded or marked write_only.

**Precedent:** `SENSITIVE_FIELD_PATTERNS` in `make_api.rs`

### Pattern 3: Foreign Key to Relationship Inference

`explain_model` already infers belongs_to relationships from `*_id` fields. The scaffolding can generate `.belongs_to()` calls.

**What:** `user_id: i32` → `.belongs_to("user", "user")`
**Precedent:** `explain_model.rs:infer_relationships()`

### Pattern 4: Validation Exposure (CLI + MCP)

ServiceDef::validate() returns `Result<Vec<Warning>, Error>`. The CLI and MCP tools need to reconstruct a ServiceDef (reusing render_projection's reconstruction logic) and expose the validation results.

**Precedent:** `validate_contracts` CLI command + MCP tool already validates Inertia type contracts

### Anti-Patterns to Avoid
- **Don't auto-generate projections at compile time** — ServiceDef is intentionally separate from models; scaffolding is a starting point, not a derived artifact
- **Don't couple projection validation to database state** — validate() checks structural correctness, not data
- **Don't duplicate model parsing logic** — reuse or extract from make_api's existing ModelInfo parser
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Model field extraction | New syn parser | make_api's ModelInfo/FieldInfo parser | Already handles SeaORM derives, table_name, field types |
| Field name → FieldMeaning | New mapping function | `ferro_projections::infer_meaning()` | Already implements 7+ inference rules |
| Projection source scanning | New file scanner | list_projections::execute() | Already scans src/projections/*.rs |
| ServiceDef reconstruction | New parser | render_projection::reconstruct_service_def() | Already handles all builder calls |
| Sensitive field detection | New list | make_api::SENSITIVE_FIELD_PATTERNS | Already lists 8 sensitive patterns |
| Relationship inference | New FK analyzer | explain_model::infer_relationships() | Already detects *_id → belongs_to |

**Key insight:** Every component needed for Phase 92 already exists in isolated form across ferro-cli and ferro-mcp. The work is combining and exposing these capabilities, not building new analysis engines.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Model Parser Code Duplication
**What goes wrong:** Creating a third copy of model field extraction logic (make_api has one, list_models has another)
**Why it happens:** Each tool evolved independently with slightly different field representations
**How to avoid:** Extract shared model parsing into a common module in ferro-cli (or a shared analyzer crate). At minimum, reuse make_api's parser directly for Phase 92.
**Warning signs:** Writing `syn::visit::Visit` implementation for the third time

### Pitfall 2: Incomplete Type Mapping
**What goes wrong:** Model has a Rust type that doesn't map to any DataType
**Why it happens:** SeaORM supports types that DataType doesn't cover (e.g., Decimal, TimeZone-aware DateTime)
**How to avoid:** Default unmapped types to DataType::String with a comment noting the mapping. Log a warning rather than failing.
**Warning signs:** Panic or skip on unknown field types

### Pitfall 3: Scaffolding vs Runtime Confusion
**What goes wrong:** Users expect `--from-model` to keep projection in sync with model changes
**Why it happens:** Natural expectation from code generation tools
**How to avoid:** Make it clear in output that scaffolding is a one-time generation — projection is then user-maintained. The `projection_coverage` tool can detect drift.
**Warning signs:** Adding file watchers or auto-regeneration logic

### Pitfall 4: Over-Confident Field Meaning Inference
**What goes wrong:** `infer_meaning()` assigns wrong FieldMeaning (e.g., field named "description" maps to FreeText but user wants it as EntityName)
**Why it happens:** Heuristic inference isn't perfect
**How to avoid:** Generate with comments showing the inference reason, so users can easily correct. Mark inferred meanings with `// inferred from field name` comments.
**Warning signs:** No way for users to see WHY a particular FieldMeaning was chosen
</common_pitfalls>

<code_examples>
## Code Examples

### Model-Aware Projection Template (generated output)
```rust
// Source: Derived from make_api model parsing + infer_meaning() patterns
use ferro::{
    DataType, FieldMeaning, ServiceDef,
};

/// Build the Order service projection.
///
/// Generated from Order model (src/models/order.rs).
/// Review and adjust FieldMeaning assignments as needed.
pub fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .optional_field("notes", DataType::String, FieldMeaning::FreeText)
        .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .read_only_field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
        // Relationships (inferred from foreign keys):
        .belongs_to("customer", "customer") // from customer_id
}
```

### Rust Type → DataType Mapping Function
```rust
// Source: Derived from make_api field type handling
fn rust_type_to_data_type(rust_type: &str) -> DataType {
    let cleaned = rust_type
        .replace("Option<", "").replace(">", "")
        .replace(" ", "");
    match cleaned.as_str() {
        "String" | "&str" => DataType::String,
        "i32" | "i64" | "u32" | "u64" | "i16" | "u16" => DataType::Integer,
        "f32" | "f64" => DataType::Float,
        "bool" => DataType::Boolean,
        "DateTime" | "NaiveDateTime" | "DateTimeUtc" | "DateTimeWithTimeZone"
            => DataType::DateTime,
        "NaiveDate" | "Date" => DataType::Date,
        "Uuid" => DataType::Uuid,
        "Vec<u8>" => DataType::Binary,
        "Json" | "serde_json::Value" => DataType::Json,
        _ => DataType::String, // safe fallback
    }
}
```

### Validation CLI Output Pattern
```rust
// Source: Derived from validate_contracts CLI pattern
// ferro projection:check
//
// Expected output:
// Checking projections...
//   ✓ order_service (src/projections/order.rs) — 0 warnings
//   ⚠ user_service (src/projections/user.rs) — 2 warnings
//     - UnreachableState: state "archived" is not reachable from initial state
//     - UnusedGuard: guard "is_admin" is defined but never referenced
//   ✓ product_service (src/projections/product.rs) — 0 warnings
//
// 3 projections checked, 2 warnings in 1 projection
```

### Coverage MCP Tool Output
```json
{
  "models": [
    {"name": "User", "has_projection": true, "projection": "user_service"},
    {"name": "Order", "has_projection": true, "projection": "order_service"},
    {"name": "Product", "has_projection": false, "suggestion": "ferro make:projection product --from-model"}
  ],
  "coverage": {
    "total_models": 3,
    "with_projections": 2,
    "without_projections": 1,
    "percentage": 66.7
  }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

No external ecosystem changes relevant. Phase 92 is internal tooling using established patterns.

| Decision | Made In | Still Current |
|----------|---------|---------------|
| syn for model parsing | make_api (Phase 76) | Yes — standard for Rust source analysis |
| regex for projection parsing | Phase 91-03 | Yes — lightweight, no syn needed for ServiceDef calls |
| infer_meaning() heuristics | Phase 84 | Yes — 7 rules covering common patterns |
| ServiceDef::validate() | Phase 86-02 | Yes — subsumes state machine validation |

**No deprecated/outdated patterns to worry about.**
</sota_updates>

<open_questions>
## Open Questions

1. **Should model parsing be extracted into a shared module?**
   - What we know: make_api and list_models have overlapping but different parsers (make_api is more complete)
   - What's unclear: Whether the extraction is worth the refactoring effort for Phase 92
   - Recommendation: Reuse make_api's parser directly for now; extract later if a fourth consumer appears

2. **Should `--from-model` auto-detect relationships beyond belongs_to?**
   - What we know: FK fields give belongs_to. has_many requires knowing the other side.
   - What's unclear: Whether scanning all models for reverse FKs is worth the complexity
   - Recommendation: Generate belongs_to from FKs only. Note in comments where has_many might apply. Keep it simple.

3. **Should coverage reporting include intent derivation results?**
   - What we know: render_projection already derives intents
   - What's unclear: Whether showing "Order → Browse (0.85)" adds value to coverage report
   - Recommendation: Include derived primary intent in coverage output — it validates that projections are working correctly
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- `ferro-cli/src/commands/make_api.rs` — model parsing, field extraction, sensitive field exclusion
- `ferro-cli/src/commands/make_projection.rs` — current projection scaffolding template
- `ferro-mcp/src/tools/list_models.rs` — ModelDetails, FieldInfo structs
- `ferro-mcp/src/tools/explain_model.rs` — field meaning inference, relationship detection
- `ferro-mcp/src/tools/list_projections.rs` — projection discovery via regex
- `ferro-mcp/src/tools/render_projection.rs` — ServiceDef reconstruction from source
- `ferro-projections/src/field.rs:infer_meaning()` — FieldMeaning inference rules
- `ferro-projections/src/service.rs:validate()` — ServiceDef validation API

### Secondary (MEDIUM confidence)
- None — all findings from direct source code analysis

### Tertiary (LOW confidence)
- None — internal codebase research, no external sources needed
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Internal ferro-cli + ferro-mcp tooling
- Ecosystem: SeaORM model parsing, ferro-projections validation API
- Patterns: Model-aware scaffolding, validation exposure, coverage reporting
- Pitfalls: Code duplication, type mapping gaps, scaffolding vs runtime confusion

**Confidence breakdown:**
- Standard stack: HIGH — all components already exist in codebase
- Architecture: HIGH — follows make_api pattern exactly
- Pitfalls: HIGH — derived from observed code duplication patterns
- Code examples: HIGH — constructed from verified source patterns

**Phase 91 overlap assessment:**
- Phase 91-02 delivered: `ferro make:projection` (basic scaffolding) — Phase 92 scope
- Phase 91-03 delivered: 3 MCP projection tools — Phase 92 scope
- Phase 92 remaining: model-aware scaffolding, validation, coverage — new work

**Research date:** 2026-03-01
**Valid until:** 2026-04-01 (30 days — internal patterns are stable)
</metadata>

---

*Phase: 92-mcp-introspection-cli*
*Research completed: 2026-03-01*
*Ready for planning: yes*
