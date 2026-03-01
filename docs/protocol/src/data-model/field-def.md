# Fields & Types

This page specifies three types: `FieldDef` (a field within a service), `DataType` (structural data categories), and `FieldMeaning` (semantic annotations that drive rendering).

## FieldDef

A field definition within a service projection. Fields describe the data shape of a domain entity.

> **JSON Schema:** [`field-def.json`](../appendix/json-schema.md)

### Type Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | -- | Machine-readable field name (e.g., `"total_amount"`) |
| `data_type` | [`DataType`](#datatype) | Yes | -- | Structural data category |
| `meaning` | [`FieldMeaning`](#fieldmeaning) | Yes | -- | Semantic meaning driving rendering behavior |
| `required` | `boolean` | No | `true` | Whether the field is required for valid entities |
| `is_list` | `boolean` | No | `false` | Whether the field holds a collection of values |
| `readable` | `boolean` | No | `true` | Whether the field is visible in read/display contexts |
| `writable` | `boolean` | No | `true` | Whether the field is editable in write/input contexts |

### Normative Rules

1. The `name` field MUST be a non-empty string.
2. The `name` field SHOULD use `snake_case` naming.
3. When `required` is omitted from input, consumers MUST default it to `true`.
4. When `is_list` is omitted from input, consumers MUST default it to `false`.
5. When `readable` is omitted from input, consumers MUST default it to `true`.
6. When `writable` is omitted from input, consumers MUST default it to `true`.
7. A field with `readable: false` and `writable: true` is a write-only field (e.g., password inputs). Renderers SHOULD exclude it from display views and include it in input forms.
8. A field with `readable: true` and `writable: false` is a read-only field (e.g., identifiers, timestamps). Renderers SHOULD display it but exclude it from editable forms.
9. The `readable` and `writable` flags are structural signals for intent derivation. A high proportion of writable fields signals the Collect intent.

### Serialization Rules

- `required` and `readable` default to `true` when omitted during deserialization.
- `is_list` and `writable` default to their documented defaults when omitted.
- All fields are always serialized in the JSON output (no skip-on-default for booleans).

### JSON Example

```json
[
  {
    "name": "id",
    "data_type": "integer",
    "meaning": "identifier",
    "required": true,
    "is_list": false,
    "readable": true,
    "writable": false
  },
  {
    "name": "email",
    "data_type": "string",
    "meaning": "email",
    "required": true,
    "is_list": false,
    "readable": true,
    "writable": true
  },
  {
    "name": "total",
    "data_type": "float",
    "meaning": "money",
    "required": true,
    "is_list": false,
    "readable": true,
    "writable": true
  },
  {
    "name": "tags",
    "data_type": "string",
    "meaning": "category",
    "required": false,
    "is_list": true,
    "readable": true,
    "writable": true
  },
  {
    "name": "password_hash",
    "data_type": "string",
    "meaning": "sensitive",
    "required": true,
    "is_list": false,
    "readable": false,
    "writable": false
  }
]
```

---

## DataType

Abstract data type categories for service fields. Represents structural types independent of database storage details.

> **JSON Schema:** [`data-type.json`](../appendix/json-schema.md)

### Variants

| Variant | JSON Value | Description |
|---------|-----------|-------------|
| `String` | `"string"` | Text data of any length |
| `Integer` | `"integer"` | Whole number values |
| `Float` | `"float"` | Decimal/floating-point values |
| `Boolean` | `"boolean"` | True/false values |
| `DateTime` | `"date_time"` | Date and time combined (e.g., ISO 8601) |
| `Date` | `"date"` | Date without time component |
| `Json` | `"json"` | Arbitrary JSON structure |
| `Binary` | `"binary"` | Binary/blob data |
| `Uuid` | `"uuid"` | Universally unique identifier |
| `Enum` | `"enum"` | Enumerated value from a fixed set |

### Normative Rules

1. Consumers MUST support all 10 variants.
2. `DataType` values MUST be serialized as `snake_case` strings.
3. `DataType` represents abstract categories, not database column types. Implementations map from storage types to `DataType` at introspection time.
4. Unknown `DataType` values MUST be rejected -- there is no fallback variant.

---

## FieldMeaning

Semantic meaning of a field, driving rendering and behavior decisions. Each variant maps to a specific UI treatment.

> **JSON Schema:** [`field-meaning.json`](../appendix/json-schema.md)

### Known Variants

| Variant | JSON Value | Semantic Meaning | Rendering Guidance |
|---------|-----------|-----------------|-------------------|
| `Identifier` | `"identifier"` | Primary key or unique identifier | Display as read-only label; exclude from forms |
| `ForeignKey` | `"foreign_key"` | Reference to another entity | Display as link to related entity |
| `CreatedAt` | `"created_at"` | Record creation timestamp | Display as relative or formatted date; system field |
| `UpdatedAt` | `"updated_at"` | Record last-modified timestamp | Display as relative or formatted date; system field |
| `EntityName` | `"entity_name"` | Human-readable name of the entity | Display prominently; use as list item label |
| `Email` | `"email"` | Email address | Render as `mailto:` link; validate as email |
| `Phone` | `"phone"` | Phone number | Render as `tel:` link; validate as phone |
| `Url` | `"url"` | Web URL | Render as clickable hyperlink |
| `ImageUrl` | `"image_url"` | URL pointing to an image | Render as image element |
| `Money` | `"money"` | Monetary value | Format with currency symbol and decimal places |
| `Percentage` | `"percentage"` | Percentage value | Format with `%` suffix or progress indicator |
| `Quantity` | `"quantity"` | Numeric count or amount | Format with locale-appropriate number formatting |
| `Status` | `"status"` | Entity status or state label | Render as badge or colored indicator |
| `Category` | `"category"` | Classification or grouping label | Render as tag or filter chip |
| `Boolean` | `"boolean"` | Semantic true/false value | Render as toggle, checkbox, or yes/no label |
| `FreeText` | `"free_text"` | Unstructured text content | Render as textarea in forms; multi-line display |
| `DateTime` | `"date_time"` | Timestamp (not a creation/update system field) | Format as localized date-time |
| `Sensitive` | `"sensitive"` | Sensitive data (passwords, tokens, keys) | Exclude from display; mask in forms |

### Custom Variant

Any string value that does not match a known variant deserializes as `Custom(String)`.

| Variant | JSON Value | Description |
|---------|-----------|-------------|
| `Custom(String)` | Any unrecognized string | Domain-specific meaning not covered by known variants |

### Normative Rules

1. Consumers MUST recognize all 18 known variants.
2. Consumers MUST accept any string as a valid `FieldMeaning`. Unrecognized strings are `Custom` values.
3. Known variants MUST be serialized as their `snake_case` form. `Custom` values serialize as plain strings.
4. During deserialization, known variant names MUST match before the `Custom` fallback. For example, `"money"` MUST deserialize as `Money`, never as `Custom("money")`.
5. `Custom(String)` MUST remain the last variant in implementations to ensure correct deserialization order.
6. `Identifier`, `CreatedAt`, and `UpdatedAt` are **system fields**. Intent derivation analyzers SHOULD exclude them from proportional signal calculations (they appear in most services and carry no discriminating intent signal).
7. The `Sensitive` meaning indicates data that MUST NOT be displayed in read views. Renderers SHOULD mask sensitive fields in input forms.

### JSON Example

Fields with different meanings on an order service:

```json
[
  { "name": "id", "data_type": "integer", "meaning": "identifier" },
  { "name": "customer_email", "data_type": "string", "meaning": "email" },
  { "name": "total", "data_type": "float", "meaning": "money" },
  { "name": "discount", "data_type": "float", "meaning": "percentage" },
  { "name": "status", "data_type": "string", "meaning": "status" },
  { "name": "notes", "data_type": "string", "meaning": "free_text" },
  { "name": "priority", "data_type": "string", "meaning": "priority_level" }
]
```

In this example, `"priority_level"` is a `Custom` meaning because it does not match any known variant.

### Meaning Inference (Informative)

> **Note:** The `infer_meaning()` function in the reference implementation provides one possible algorithm for auto-detecting meaning from field names. This algorithm is informative, not normative. Other implementations MAY use different heuristics.

The reference implementation applies seven inference rules:

1. **Exact match:** `"id"` maps to `Identifier`, `"email"` to `Email`, `"created_at"` to `CreatedAt`, `"updated_at"` to `UpdatedAt`
2. **Suffix `_id`:** maps to `ForeignKey`
3. **Suffix `_at`:** maps to `DateTime`
4. **Prefix `is_` or `has_`:** maps to `Boolean`
5. **Contains `password`, `secret`, `token`, `api_key`, `hashed_key`:** maps to `Sensitive`
6. **Fallback:** `Custom(field_name)`
