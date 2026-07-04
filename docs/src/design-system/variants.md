# Variant Vocabulary

These canonical values are enforced at catalog validation time and checked by the design lint engine. For the full component rename/migration table from earlier versions, see [Components](../json-ui/components.md#component-specific-enum-values).

Three shared enums cover the weight, status, and scale axes across all components. One word, one meaning: `variant` is always visual weight, `tone` is always status color, `size` is always the `sm`/`md`/`lg` scale.

## variant

Visual weight of interactive elements.

| Value | Meaning |
|-------|---------|
| `primary` | Highest-emphasis action; filled with the primary color |
| `secondary` | Subdued alternative; filled with the secondary color |
| `outline` | Bordered, no fill; lower visual weight than primary |
| `ghost` | No border, no fill; lowest visual weight (inline actions) |
| `destructive` | Signals a destructive or irreversible action |

## tone

Semantic status color of stateful display components (Alert, Badge, Toast, ConfirmDialog).

| Value | Meaning |
|-------|---------|
| `neutral` | Default, no special status |
| `success` | Positive outcome or confirmation |
| `warning` | Caution or pending attention |
| `destructive` | Error, danger, or data-loss risk |

## size

Uniform scale for interactive and display components.

| Value | Meaning |
|-------|---------|
| `sm` | Small — compact layouts, dense tables |
| `md` | Medium — default; use when size is not specified |
| `lg` | Large — prominent actions or accessible targets |
