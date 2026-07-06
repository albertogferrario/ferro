# Rendering

This section specifies the rendering abstraction that translates a `ServiceDef` and its derived intents into a framework-independent JSON output.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this section are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

## Renderer Trait

The `Renderer` trait is the protocol's output contract. All rendering implementations MUST conform to this interface:

```
render(service: ServiceDef, intents: [IntentScore], ctx: RenderContext) -> Result<Value, Error>
```

**Requirements:**

- The output MUST be a JSON value (`serde_json::Value` or equivalent).
- The output format is renderer-defined. The protocol does not prescribe a specific component vocabulary.
- The renderer MUST NOT modify the input `ServiceDef` or `IntentScore` values.
- Rendering failures MUST produce an `Error::Render` variant.

## RenderContext

`RenderContext` controls which intent is rendered and how:

| Field | Type | Description |
|-------|------|-------------|
| `intent_index` | `usize` | Index into the `intents` slice (0 = primary intent) |
| `current_state` | `Option<String>` | Current workflow state name, for state-machine-aware rendering |
| `mode` | `RenderMode` | Display or Input mode |

**Defaults:**

- `intent_index`: `0`
- `current_state`: `None`
- `mode`: `Display`

## RenderMode

`RenderMode` determines whether the output is read-only or editable:

- **`Display`** -- Read-only presentation. Fields render as text, badges, progress bars, or other display components.
- **`Input`** -- Editable form view. Fields render as typed input controls (text inputs, selects, switches).

Every intent SHOULD support both modes. When an intent has no meaningful Input mode, implementations MAY fall back to a generic form layout.

## Intent-to-Layout Mapping (Informative)

> **Note:** This section is INFORMATIVE. Different renderer implementations MAY map intents to different UI patterns. The mappings below describe the reference implementation (`JsonUiRenderer`).

| Intent | Display Layout | Input Layout |
|--------|----------------|--------------|
| Browse | Table/List with pagination; system fields excluded from columns; columns are sortable | Form with typed inputs for writable fields |
| Focus | Card with DescriptionList; relationship sections rendered per NavigationHint | Form with typed inputs for writable fields |
| Collect | Form with typed inputs per FieldMeaning; system auto-generated fields skipped; Submit button | Same as Display (Collect is inherently an input layout) |
| Process | Card with current state Badge; guard Alert; transition action Buttons. Falls back to Focus layout without a state machine | Form with transition buttons for editing while progressing state |
| Summarize | Card per metric field (Money/Quantity as Text, Percentage as Progress); Status as Badge; DescriptionList for remaining fields | Form with typed inputs for writable fields |
| Analyze | Summary Card with stat placeholders for numeric fields; sortable Table with all readable fields including DateTime columns | Form with typed inputs for writable fields |
| Track | Table with DateTime system fields visible; Status columns; sorted descending; with Pagination | Form with typed inputs for writable fields |
| Custom(String) | Falls back to Focus layout | Falls back to Collect layout |

## Field Rendering (Informative)

> **Note:** This section is INFORMATIVE. Different renderers MAY use different component vocabularies.

`FieldMeaning` influences which UI component is selected for each field. The reference implementation maps meanings as follows:

### Display Mode

| FieldMeaning | Component | Notes |
|--------------|-----------|-------|
| Identifier | Text | Displayed as plain text |
| ForeignKey | (hidden) | Not shown in display mode |
| EntityName | Text | Primary name display |
| Email, Phone, Url | Text | Displayed as formatted text; renderers MAY add mailto/tel links |
| Money, Quantity, FreeText | Text | Displayed as formatted text |
| ImageUrl | Avatar | Image thumbnail |
| Percentage | Progress | Visual progress indicator |
| Status | Badge (default variant) | Status indicator |
| Category | Badge (secondary variant) | Category tag |
| Boolean | Badge (outline variant) | Boolean indicator |
| DateTime, CreatedAt, UpdatedAt | Text | Formatted timestamp |
| Sensitive | (hidden) | Never shown in display mode |
| Custom(String) | Text | Generic text fallback |

### Input Mode

| FieldMeaning | Component | Input Type |
|--------------|-----------|------------|
| Identifier | Input | `hidden` |
| ForeignKey | Select | Dropdown with options |
| EntityName | Input | `text` (always required) |
| Email | Input | `email` |
| Phone | Input | `tel` |
| Url, ImageUrl | Input | `url` |
| Money | Input | `number` (step 0.01) |
| Percentage | Input | `number` (min 0, max 100) |
| Quantity | Input | `number` |
| Status, Category | Select | Dropdown with options |
| Boolean | Switch | Toggle |
| FreeText | Input | `textarea` |
| CreatedAt, UpdatedAt | Input | `text` (disabled) |
| DateTime | Input | `text` |
| Sensitive | Input | `password` (no data_path) |
| Custom(String) | Input | `text` |

### Column Mode (Tables)

Fields rendered as table columns MAY include a `format` hint:

| FieldMeaning | Format Hint |
|--------------|-------------|
| Money | `currency` |
| DateTime, CreatedAt, UpdatedAt | `datetime` |
| Boolean | `boolean` |
| All others | (none) |

## Relationship Rendering (Informative)

> **Note:** This section is INFORMATIVE. Different renderers MAY use different patterns.

`NavigationHint` controls how relationships are presented in the UI:

| NavigationHint | Rendering Pattern | Description |
|----------------|-------------------|-------------|
| Inline | Embedded Text/Card | Related entity data displayed inline within the parent view |
| Link | Button (link variant) | Navigation link to the related entity |
| Tab | Tabbed Section | Related entity rendered in a separate tab |
| Nested | Embedded Table | Related collection rendered as an inline table |
| Hidden | (not rendered) | Relationship exists in data but is not displayed |

## Output Format

The protocol does not prescribe a specific JSON output schema for renderers. Each renderer implementation defines its own component vocabulary and envelope structure.

The reference `JsonUiRenderer` produces output conforming to the `ferro-json-ui/v2` schema: a top-level `Spec` with a `$schema` tag, a `root` element ID, a flat `elements` map keyed by ID, and optional `title`, `layout`, and `data` fields. Children inside `elements` are referenced by ID rather than by nesting. Other renderers (e.g., A2UI, HTML) MAY produce entirely different output structures while remaining conformant, provided they implement the `Renderer` trait.
