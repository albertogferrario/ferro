# Phase 84: Service Identity & Field Semantics — Research

**Researched:** 2026-02-28
**Domain:** Rust crate design — schema-only service definitions with semantic field types
**Confidence:** HIGH

<research_summary>
## Summary

Phase 84 creates the `ferro-projections` crate with two core types: `ServiceDef` (service identity and field declarations) and `FieldMeaning` (semantic type annotations for fields). This is an internal patterns phase — the research focuses on implementation patterns rather than ecosystem discovery.

The v9.0 milestone research already validated the architecture against Google A2UI, XState, Metabase semantic types, and MAUI. This phase-level research focuses on: (1) how to design the `DataType` enum that doesn't yet exist, (2) the serde strategy for `FieldMeaning` with a `Custom(String)` fallback, (3) the builder API style matching workspace conventions, and (4) workspace crate setup patterns.

**Primary recommendation:** Follow the existing workspace builder pattern (`with_*` methods returning `Self`), use `#[serde(rename_all = "snake_case")]` with per-variant `#[serde(untagged)]` on `Custom(String)`, and keep DataType to ~10 variants covering database column type categories rather than Rust types.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1 | Serialization/deserialization | All schema types must be serializable |
| serde_json | 1 | JSON format | IntentGraph, MCP introspection will consume JSON |
| thiserror | 1.0 | Error types | Workspace error pattern |

### Supporting (already in workspace)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4 | Date/time types | If DataType needs temporal awareness |

### New Dependencies: None
`ferro-projections` needs only `serde` + `serde_json` + `thiserror`. No new dependencies required. This is purely a type/schema crate.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom builder | derive_builder | derive_builder adds macro dependency; hand-rolled builder is trivial for this use case |
| Custom DataType | sea-orm ColumnType | ColumnType is database-specific; we need abstract semantic types |
| schemars for JSON Schema | Manual serde | overkill — we don't need JSON Schema generation for internal schemas |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Crate Module Structure
```
ferro-projections/
├── Cargo.toml
├── CLAUDE.md          # Already prepared in .planning/phases/84-*/
├── src/
│   ├── lib.rs         # Re-exports, crate docs
│   ├── service.rs     # ServiceDef, ServiceDefBuilder
│   ├── field.rs       # FieldDef, FieldMeaning, DataType
│   └── error.rs       # Error enum
```

Phase 84 creates only the first three modules. Remaining modules (`state.rs`, `action.rs`, `relationship.rs`, `intent.rs`, `graph.rs`, `renderer.rs`) are added in subsequent phases.

### Pattern 1: Builder with `with_*` Methods (Workspace Convention)
**What:** Builders take `mut self` and return `Self` for chaining.
**When to use:** All configuration/definition types.
**Example:**
```rust
// Follows ferro-cache CacheConfig pattern
impl ServiceDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            fields: Vec::new(),
        }
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn field(mut self, name: impl Into<String>, data_type: DataType, meaning: FieldMeaning) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            ..Default::default()
        });
        self
    }
}
```

**Note:** The CLAUDE.md for this phase says builders return `&mut Self`. The workspace convention is `mut self` → `Self` (owned, consuming). Use the workspace convention for consistency. The `&mut Self` pattern would only be needed if ServiceDef needs to be built incrementally across multiple call sites — which it doesn't.

### Pattern 2: Serde for Schema Types
**What:** All public types derive `Serialize, Deserialize, Debug, Clone` with `#[serde(rename_all = "snake_case")]` on enums.
**When to use:** Every type in ferro-projections (it's schema-only, everything must serialize).
**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldMeaning {
    Money,
    Percentage,
    Email,
    // ...known variants...
    #[serde(untagged)]
    Custom(String),
}
```

**Verified:** The `#[serde(untagged)]` per-variant pattern was tested with a standalone Rust project. All 8 tests pass:
- `Money` → `"money"`, `ForeignKey` → `"foreign_key"` (rename_all applies)
- `Custom("tax_rate")` → `"tax_rate"` (untagged emits raw string)
- `"money"` → `Money`, `"unknown_thing"` → `Custom("unknown_thing")` (known variants match first, fallback catches rest)
- Full round-trip works for all variants including Custom.

Requires serde >= 1.0.171 (June 2023). Workspace uses serde 1.x — confirmed available.

### Pattern 3: Feature-Gated Framework Integration
**What:** Framework depends on ferro-projections behind a cargo feature.
**When to use:** Phase 91 (framework integration), but the Cargo.toml pattern is established now.
**Example:**
```toml
# framework/Cargo.toml (Phase 91, not Phase 84)
[features]
projections = ["dep:ferro-projections"]

[dependencies]
ferro-projections = { path = "../ferro-projections", version = "0.1", optional = true }
```

### Anti-Patterns to Avoid
- **No closures in definitions:** Guards are `"has_content"` strings, not `|o| o.content.is_some()`. Breaks serialization.
- **No runtime logic in ServiceDef:** It's a schema, not an engine. Validation logic (e.g., "field X requires meaning Y") belongs in a validation pass, not in the builder.
- **No Default impl that hides required fields:** `ServiceDef::default()` should NOT exist. A name is always required.
- **No trait objects:** ServiceDef is concrete. No `dyn ServiceProvider` abstractions at this layer.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Enum serialization | Custom Serialize/Deserialize impls for FieldMeaning | `#[serde(rename_all = "snake_case")]` + `#[serde(untagged)]` on Custom variant | Serde handles this natively since 1.0.171 |
| Error types | Manual Display + Error impls | `thiserror` derive | Workspace convention, less boilerplate |
| Builder validation | Runtime panics in builder methods | Separate `validate()` method returning Result | Don't panic in builder chains |

**Key insight:** This crate is small enough that there's nothing complex to hand-roll. The risk is over-engineering, not under-engineering. The `ServiceDef` builder, `DataType` enum, and `FieldMeaning` enum are each under 50 lines of code.

| FieldMeaning inference | Custom inference from scratch | Reuse existing patterns from codebase | 7 inference rules already exist in ferro-cli and ferro-mcp (see Existing Inference Patterns section) |
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: DataType Explosion
**What goes wrong:** Creating a DataType variant for every possible database column type (Varchar(255), Text, BigInt, SmallInt, Decimal(10,2), etc.)
**Why it happens:** Trying to mirror SeaORM ColumnType or SQL types exactly.
**How to avoid:** DataType represents *abstract data categories*, not database types. Use ~10 variants: String, Integer, Float, Boolean, DateTime, Date, Json, Binary, Uuid, Enum. The mapping from ColumnType → DataType is a one-way projection done at introspection time, not stored.
**Warning signs:** If DataType has more than 15 variants, it's too granular.

### Pitfall 2: FieldMeaning Overload
**What goes wrong:** Adding variants that duplicate DataType (e.g., both `DataType::Boolean` and `FieldMeaning::Boolean`).
**Why it happens:** Confusion between structural type (what the data IS) and semantic meaning (what the data MEANS).
**How to avoid:** FieldMeaning adds information DataType can't express. `FieldMeaning::Money` on a `DataType::Float` says "format as currency." `FieldMeaning::Status` on a `DataType::String` says "render as badge." If a FieldMeaning variant doesn't change rendering behavior, it shouldn't exist.
**Warning signs:** A FieldMeaning variant that maps 1:1 to a DataType and adds no rendering hint.

### Pitfall 3: Builder Returns &mut Self vs Self
**What goes wrong:** Using `&mut self` → `&mut Self` builder pattern when workspace convention is `mut self` → `Self`.
**Why it happens:** The phase CLAUDE.md says `&mut Self` but workspace crates use owned builders.
**How to avoid:** Use `mut self` → `Self` (consuming builder). This matches ferro-cache, ferro-broadcast, ferro-storage patterns. If later phases need incremental building, add a separate `ServiceDefBuilder` that uses `&mut self`.
**Warning signs:** Needing `.clone()` or lifetimes when constructing a ServiceDef.

### Pitfall 4: Serde Untagged Ordering
**What goes wrong:** `Custom(String)` variant placed before named variants in FieldMeaning enum.
**Why it happens:** Serde untagged deserialization tries variants in order. If Custom(String) comes first, it matches everything.
**How to avoid:** `Custom(String)` must ALWAYS be the last variant in the enum.
**Warning signs:** All deserialized values end up in Custom regardless of input.
</common_pitfalls>

<code_examples>
## Code Examples

### ServiceDef Construction
```rust
// Complete ServiceDef for an order management service
let order_service = ServiceDef::new("order")
    .display_name("Order")
    .description("Manages customer orders and fulfillment")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
    .field("total", DataType::Float, FieldMeaning::Money)
    .field("status", DataType::String, FieldMeaning::Status)
    .field("email", DataType::String, FieldMeaning::Email)
    .field("notes", DataType::String, FieldMeaning::FreeText)
    .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
    .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);
```

### FieldMeaning Serde Round-Trip
```rust
// Known variant serialization
let meaning = FieldMeaning::Money;
let json = serde_json::to_string(&meaning).unwrap();
assert_eq!(json, r#""money""#);

// Custom variant serialization
let custom = FieldMeaning::Custom("tax_rate".to_string());
let json = serde_json::to_string(&custom).unwrap();
assert_eq!(json, r#""tax_rate""#);

// Deserialization — known string maps to known variant
let parsed: FieldMeaning = serde_json::from_str(r#""money""#).unwrap();
assert_eq!(parsed, FieldMeaning::Money);

// Deserialization — unknown string maps to Custom
let parsed: FieldMeaning = serde_json::from_str(r#""tax_rate""#).unwrap();
assert_eq!(parsed, FieldMeaning::Custom("tax_rate".to_string()));
```

### DataType Enum (Recommended Design)
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Date,
    Json,
    Binary,
    Uuid,
    Enum,
}
```

**Design rationale:**
- `Copy` — small enum, no heap data, enables cheap cloning
- No `Custom(String)` — DataType is structural, not semantic. If the type doesn't fit, it's `String` or `Json`.
- No database-specific variants — no `Text` vs `Varchar`, no `BigInt` vs `SmallInt`. Those are storage concerns, not schema concerns.
- `Enum` covers database enum types without specifying valid values (that's the field's metadata).

### Error Type
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Service definition error: {0}")]
    Definition(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

### Test Pattern: Serde Round-Trip
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_def_serde_round_trip() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .field("total", DataType::Float, FieldMeaning::Money);

        let json = serde_json::to_string(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();

        assert_eq!(service.name, parsed.name);
        assert_eq!(service.fields.len(), parsed.fields.len());
    }

    #[test]
    fn field_meaning_custom_fallback() {
        // Known variants round-trip correctly
        for meaning in [FieldMeaning::Money, FieldMeaning::Email, FieldMeaning::Status] {
            let json = serde_json::to_string(&meaning).unwrap();
            let parsed: FieldMeaning = serde_json::from_str(&json).unwrap();
            assert_eq!(meaning, parsed);
        }

        // Unknown strings become Custom
        let parsed: FieldMeaning = serde_json::from_str(r#""unknown_type""#).unwrap();
        assert!(matches!(parsed, FieldMeaning::Custom(s) if s == "unknown_type"));
    }

    #[test]
    fn data_type_is_copy() {
        let dt = DataType::Float;
        let dt2 = dt; // Copy, not move
        assert_eq!(dt, dt2);
    }
}
```
</code_examples>

<datatype_validation>
## DataType Validation Against Real Models

Tested the 10-variant DataType enum against the sample app's actual model fields.

### Sample App Coverage

| Model | Fields | Types Used |
|-------|--------|------------|
| User | id, name, email, password, remember_token, created_at, updated_at | i32, String, Option\<String\> |
| Todo | id, title, description, created_at, updated_at | i32, String, Option\<String\> |
| ApiKey | id, name, prefix, hashed_key, scopes, last_used_at, expires_at, revoked_at, created_at | i64, String, Option\<String\>, DateTimeUtc, Option\<DateTimeUtc\> |

### DataType Variants Actually Used: 3 of 10
- **String** — name, email, password, title, description, etc.
- **Integer** — id (i32, i64)
- **DateTime** — last_used_at, expires_at, created_at (DateTimeUtc)

### Variants Not Used in Sample (but valid for real apps)
Float, Boolean, Date, Json, Binary, Uuid, Enum — the sample app is minimal. These are all common in production apps (prices, flags, dates, JSONB columns, file blobs, UUIDs, status enums).

### Cross-Reference: CLI FieldType Enum
The CLI scaffold (`ferro-cli/src/commands/make_scaffold.rs`) already has a 9-variant `FieldType` enum:
```
String, Text, Integer, BigInteger, Float, Boolean, DateTime, Date, Uuid
```

**Key differences from proposed DataType:**
- CLI splits String/Text and Integer/BigInteger (storage-level detail we intentionally collapse)
- CLI doesn't have Json, Binary, Enum (gaps in scaffolding)
- Our DataType adds Json, Binary, Enum while collapsing storage variants

**Conclusion:** 10 variants is the right number. Covers CLI patterns plus fills gaps.
</datatype_validation>

<existing_inference>
## Existing Field Name Inference Patterns

Seven name-based inference rules already exist across the codebase. These can be unified into a `infer_field_meaning()` function for auto-populating FieldMeaning from field names.

### Rules Found

| Pattern | Rule | Location | Maps To |
|---------|------|----------|---------|
| `*_id` | Foreign key | `ferro-cli/src/commands/make_scaffold.rs:424`, `ferro-cli/src/analyzer.rs:296` | `FieldMeaning::ForeignKey` |
| `*_at` | Timestamp | `ferro-cli/src/commands/make_scaffold.rs:428` | `FieldMeaning::CreatedAt` / `UpdatedAt` / `DateTime` |
| `is_*`, `has_*` | Boolean flag | `ferro-cli/src/commands/make_scaffold.rs:433` | `FieldMeaning::Boolean` |
| `email` | Email field | `ferro-cli/src/commands/make_scaffold.rs:439` | `FieldMeaning::Email` |
| `password*` | Sensitive | `ferro-cli/src/commands/make_api.rs:21` | `FieldMeaning::Sensitive` (new variant?) |
| `*token*`, `*secret*`, `*api_key*` | Sensitive | `ferro-cli/src/commands/make_api.rs:21` | `FieldMeaning::Sensitive` |
| `id` | Primary key | `ferro-mcp/src/tools/list_models.rs:94` | `FieldMeaning::Identifier` |

### Refined Inference Function (for Phase 84 or later)
```rust
pub fn infer_meaning(field_name: &str) -> FieldMeaning {
    // Exact matches first
    match field_name {
        "id" => return FieldMeaning::Identifier,
        "email" => return FieldMeaning::Email,
        "created_at" => return FieldMeaning::CreatedAt,
        "updated_at" => return FieldMeaning::UpdatedAt,
        _ => {}
    }

    // Suffix patterns
    if field_name.ends_with("_id") {
        return FieldMeaning::ForeignKey;
    }
    if field_name.ends_with("_at") {
        return FieldMeaning::DateTime;
    }

    // Prefix patterns
    if field_name.starts_with("is_") || field_name.starts_with("has_") {
        return FieldMeaning::Boolean;
    }

    // Sensitive field patterns
    const SENSITIVE: &[&str] = &["password", "secret", "token", "api_key", "hashed_key"];
    if SENSITIVE.iter().any(|s| field_name.contains(s)) {
        return FieldMeaning::Sensitive;
    }

    FieldMeaning::Custom(field_name.to_string())
}
```

### Design Decision: Sensitive Variant
The existing `SENSITIVE_FIELD_PATTERNS` in make_api.rs (8 patterns) suggests `FieldMeaning::Sensitive` should be a first-class variant. Sensitive fields need special handling: excluded from API resources, masked in logs, hidden from introspection. This is not in the v9.0 research's 16 variants but warrants inclusion.

### When to Apply Inference
- **MCP introspection:** When building ServiceDef from existing models at introspection time
- **CLI scaffolding:** When `make:service` generates a ServiceDef
- **NOT in the builder:** The builder takes explicit FieldMeaning. Inference is a convenience layer on top.
</existing_inference>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Serde `#[serde(other)]` for catch-all | Per-variant `#[serde(untagged)]` | serde 1.0.171 (2023-06) | `#[serde(other)]` only works on unit variants; `#[serde(untagged)]` on a specific variant captures the data |
| derive_builder crate for complex builders | Hand-rolled `with_*` methods | N/A | derive_builder adds proc-macro dependency; manual builders are clearer for small types |
| SeaORM ColumnType as universal type | Abstract DataType enum | Ferro v9.0 | ColumnType is storage-specific; abstract types decouple from database |

**Industry validation (from v9.0 research):**
- Google A2UI validates intent→renderer separation
- Metabase semantic types validate FieldMeaning design (40+ types, start with 16)
- XState validates schema-only, string-referenced guards

**No new tools/patterns needed:** This is internal pattern design, not an ecosystem integration.
</sota_updates>

<open_questions>
## Open Questions

1. **~~Should FieldMeaning::Boolean exist?~~** RESOLVED: Yes.
   - The `is_*` / `has_*` name inference pattern (already in make_scaffold.rs) maps to `FieldMeaning::Boolean`. Removing it would break the inference chain.
   - The renderer uses it to distinguish "toggle switch" from "yes/no text".

2. **~~FieldDef required vs optional — default value?~~** RESOLVED: Default `required: true`.
   - The builder uses `.field()` for required and `.optional_field()` for nullable. Matches the sample app where most fields are required (only description, scopes, and timestamps are Optional).

3. **~~Should DataType include Array/List?~~** RESOLVED: Use `is_list: bool` on FieldDef.
   - Keeps DataType `Copy`-able and flat. No `Vec<String>` in the sample app models (all are scalar fields).

4. **Should FieldMeaning::Sensitive be a first-class variant?**
   - What we know: 8 sensitive patterns already exist in `make_api.rs`. Sensitive fields need hiding from API resources, masking in logs, exclusion from introspection.
   - Recommendation: YES — add `Sensitive` to FieldMeaning. It drives real rendering/behavior differences (hidden, masked, excluded). The inference function can auto-detect it from field names.

5. **Should inference live in Phase 84 or later?**
   - What we know: The inference logic is straightforward (~20 lines) and reuses existing patterns.
   - Recommendation: Include a basic `infer_meaning()` function in Phase 84 as a utility in `field.rs`. It's small, tested, and immediately useful for MCP introspection in Phase 92.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [v9.0-RESEARCH.md](../v9.0-RESEARCH.md) — architecture validation, FieldMeaning variants, state machine patterns
- [Serde enum representations](https://serde.rs/enum-representations.html) — externally/internally/adjacently tagged, untagged
- [Serde variant attributes](https://serde.rs/variant-attrs.html) — `#[serde(other)]` limitations, `#[serde(untagged)]` per-variant
- [serde-rs/json #1044](https://github.com/serde-rs/json/issues/1044) — pattern for unit variants + Custom(String) catch-all
- [Metabase Semantic Types](https://www.metabase.com/docs/latest/data-modeling/semantic-types) — ~40 semantic types, design principles

### Secondary (MEDIUM confidence)
- Workspace crate analysis (ferro-cache, ferro-events, ferro-storage, ferro-broadcast) — builder patterns, module structure, test organization
- Phase 84 CLAUDE.md — module structure, naming conventions, anti-patterns
- Sample app model analysis (app/src/models/entities/) — DataType coverage validation
- CLI inference patterns (ferro-cli/src/commands/make_scaffold.rs, make_api.rs, analyzer.rs) — field name inference rules

### Tertiary (LOW confidence — needs validation)
- None. All findings verified — serde pattern tested with standalone project, DataType validated against real models, inference patterns traced to source files.
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust crate design (serde, thiserror, builder patterns)
- Ecosystem: No external dependencies needed beyond workspace
- Patterns: Builder API, serde enum serialization, feature-gated integration
- Pitfalls: DataType granularity, FieldMeaning overload, serde ordering

**Confidence breakdown:**
- Standard stack: HIGH — uses only existing workspace dependencies
- Architecture: HIGH — follows established workspace crate patterns
- Pitfalls: HIGH — identified from codebase analysis and serde documentation
- Code examples: HIGH — verified against serde docs and workspace conventions

**Research date:** 2026-02-28
**Valid until:** 2026-03-28 (30 days — internal patterns, stable)
</metadata>

---

*Phase: 84-service-identity-field-semantics*
*Research completed: 2026-02-28*
*Ready for planning: yes*
