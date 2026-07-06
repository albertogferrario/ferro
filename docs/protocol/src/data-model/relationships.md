# Relationships

This page specifies three types: `RelationshipDef` (service-to-service connection), `Cardinality` (structural multiplicity), and `NavigationHint` (presentational rendering guidance).

Relationships carry two orthogonal dimensions: **structural** (cardinality) and **presentational** (navigation hint). This separation allows the same structural relationship to be rendered differently based on context.

## RelationshipDef

A service-to-service relationship declaration. Each service declares its own relationships independently.

> **JSON Schema:** [`relationship-def.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | Relationship name (e.g., `"customer"`, `"line_items"`) |
| `target` | `string` | Yes | -- | Target service name (e.g., `"customer"`, `"order_line_item"`) |
| `cardinality` | [`Cardinality`](#cardinality) | Yes | -- | Structural multiplicity |
| `navigation` | [`NavigationHint`](#navigationhint) | Yes | Derived from cardinality | Presentational rendering guidance |
| `foreign_key` | `string` | No | -- | Foreign key field name on the owning side |
| `inverse` | `string` | No | -- | Name of the inverse relationship on the target service |
| `description` | `string` | No | -- | Human-readable description of the relationship |

### Normative Rules

1. The `name` field MUST be a non-empty string.
2. The `target` field MUST be a non-empty string referencing another service name.
3. The `navigation` field has a default value derived from `cardinality` (see [`Cardinality.default_navigation()`](#default-navigation)). Consumers SHOULD use this default when constructing relationships without an explicit navigation hint.
4. Multiple relationships with the same `name` within a single `ServiceDef` SHOULD produce a duplicate-relationship warning.
5. A `many_to_many` relationship with `foreign_key` set SHOULD produce a warning -- join tables do not have a single owning foreign key.
6. `foreign_key`, `inverse`, and `description` are omitted from JSON output when not set.

### JSON Example

An order with items (one-to-many, nested display) and a customer (many-to-one, link display):

```json
[
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
    "foreign_key": "customer_id",
    "inverse": "orders",
    "description": "Customer who placed this order"
  }
]
```

---

## Cardinality

Structural multiplicity of a service-to-service relationship. Standard entity-relationship cardinality covering the four relationship types.

> **JSON Schema:** [`cardinality.json`](../appendix/json-schema.md)

### Variants

| Variant | JSON Value | Description |
|---------|-----------|-------------|
| `OneToOne` | `"one_to_one"` | Exactly one related entity (e.g., user has one profile) |
| `OneToMany` | `"one_to_many"` | Multiple related entities owned by one parent (e.g., order has many items) |
| `ManyToOne` | `"many_to_one"` | This entity belongs to one parent (e.g., item belongs to order) |
| `ManyToMany` | `"many_to_many"` | Multiple entities on both sides (e.g., products have many tags) |

### Normative Rules

1. Consumers MUST support all 4 variants.
2. `Cardinality` values MUST be serialized as `snake_case` strings.
3. Unknown `Cardinality` values MUST be rejected -- there is no fallback variant.

### Default Navigation

Each cardinality maps to a default [`NavigationHint`](#navigationhint):

| Cardinality | Default NavigationHint | Rationale |
|-------------|----------------------|-----------|
| `OneToOne` | `Inline` | Embed related data in current view (e.g., profile on user card) |
| `ManyToOne` | `Link` | Navigable link to parent entity (e.g., link to customer) |
| `OneToMany` | `Nested` | Nested list within current view (e.g., items table in order) |
| `ManyToMany` | `Nested` | Nested list within current view (e.g., tags on product) |

> **Note:** The `default_navigation()` method in the reference implementation encodes this mapping. It is informative -- implementations MAY choose different defaults.

---

## NavigationHint

Presentational hint for how a relationship should be rendered in UI. Bridges the gap between structural relationships and UI presentation. Defaults are derived from [`Cardinality`](#default-navigation) and can be overridden per relationship.

> **JSON Schema:** [`navigation-hint.json`](../appendix/json-schema.md)

### Variants

| Variant | JSON Value | Description | Typical Rendering |
|---------|-----------|-------------|-------------------|
| `Inline` | `"inline"` | Embed related data in current view | Card or inline section within parent |
| `Link` | `"link"` | Show as navigable link to related entity | Clickable link or reference badge |
| `Tab` | `"tab"` | Show as separate tab in detail view | Tab panel in entity detail |
| `Nested` | `"nested"` | Show as nested list/table within current view | Table or list embedded in parent view |
| `Hidden` | `"hidden"` | Relationship exists but not shown in default navigation | Programmatic access only |

### Normative Rules

1. Consumers MUST support all 5 variants.
2. `NavigationHint` values MUST be serialized as `snake_case` strings.
3. Unknown `NavigationHint` values MUST be rejected -- there is no fallback variant.
4. `Hidden` relationships SHOULD still be accessible programmatically. The hint only affects default UI rendering.

---

## Relationships as Intent Signals

The cardinality and count of relationships contribute to intent derivation:

- **`OneToMany` and `ManyToMany` relationships** signal the **Browse** intent -- services with collection-type relationships are navigational hubs for exploring related entities.
- **`OneToOne` with `Inline` navigation** signals the **Focus** intent -- embedding related data suggests a detail-oriented view.
- **`ManyToOne` relationships** provide weak **Focus** signals -- they indicate the entity belongs to a parent, suggesting it is viewed in detail within a broader context.
- **Services with many relationships** (more than 3) provide a **Browse** signal -- high relationship density indicates a navigational entity.
