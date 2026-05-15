# Expressions

Expressions are JSON objects placed as prop values inside elements. They are resolved at render time by the framework against the handler data. There are exactly two expression types: `$data` and `$template`.

Expressions appear only inside `element.props`. They are not resolved elsewhere in the spec.

## $data — Type-Preserving Extraction

Format:
```json
{ "$data": "/json/pointer/path" }
```

`$data` extracts the value at the given [JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901) path from the spec's data context. The resolved value replaces the entire expression object and preserves its original type.

Missing paths resolve to `null`.

**Examples:**
```json
{ "$data": "/user/name" }      // "Alice"  (string preserved)
{ "$data": "/order/total" }    // 99.50    (number preserved)
{ "$data": "/flags/active" }   // true     (boolean preserved)
{ "$data": "/items" }          // [...]    (array preserved)
{ "$data": "/missing" }        // null     (path not found)
```

**In a complete element:**
```json
"total_card": {
  "type": "StatCard",
  "props": {
    "label": "Total Revenue",
    "value": { "$data": "/stats/revenue" }
  }
}
```

With data `{ "stats": { "revenue": 12345 } }`, the `value` prop resolves to `12345` (number).

## $template — String Interpolation

Format:
```json
{ "$template": "text {/path} more text" }
```

`$template` produces a string by substituting `{/path}` placeholders with values from the data context. Each placeholder uses JSON Pointer syntax. The result is always a string regardless of what the placeholders resolve to.

Missing placeholders substitute as `""` (empty string). To emit a literal `{` or `}` character, escape it with a backslash: `\{` and `\}`.

**Examples:**
```json
{ "$template": "Hello, {/user/name}!" }          // "Hello, Alice!"
{ "$template": "Order #{/order/id}" }             // "Order #1042"
{ "$template": "Items: {/cart/count} in cart" }   // "Items: 3 in cart"
```

**In a complete element:**
```json
"greeting": {
  "type": "Text",
  "props": {
    "content": { "$template": "Welcome, {/user/name}!" },
    "element": "h1"
  }
}
```

## Where Expressions Apply

Expressions are resolved in `element.props` values only. They are **not** resolved in:

- `spec.title` — literal string
- `spec.layout` — literal string
- `spec.data` — the data source itself
- `element.children` — always a list of element ID strings
- `element.action` — handler name and method are literal
- `element.visible` — visibility condition fields are literal

Expressions can appear at any depth inside a props object or array, but only as values — not as keys.

**Using both expression types in one element:**
```json
"order_header": {
  "type": "Text",
  "props": {
    "content": { "$template": "Welcome, {/user/name}!" },
    "element": "h1"
  }
}
```

## Single-Pass Guarantee

If a `$data` expression resolves to a string value that looks like `{"$data": "/another/path"}`, the inner expression is not re-resolved. Expressions are evaluated in a single pass. This prevents injection and makes expression evaluation predictable.

## Hard Cap — What Does Not Exist

The expression language is intentionally minimal. The following do **not** exist and will not be added:

- `$if` — no conditional rendering in expressions
- `$for` — no loops in expressions
- `$state` — no client-side state
- `$bind` — no two-way binding
- `$map` — no array transformation in expressions
- `$reduce` — no aggregation in expressions

Conditional logic belongs in the Rust handler before calling `render_file`. Use the `"visible"` field on elements for simple show/hide logic based on data values. Complex branching is handled server-side — shape the data differently, or call `render_file` with a different spec path.

## Infallible Semantics

Malformed expressions degrade to literal JSON values — the framework never panics on invalid expression objects. An expression with a non-string value or extra sibling keys is passed through unchanged. This is intentional for rendering reliability.
