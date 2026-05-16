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

## $each

`$each` instantiates one element per row in a data array.

The directive is element-level: it appears as a key on the element JSON object, alongside `type`, `props`, and `children`. It is resolved before render — by the time props are evaluated, the templated element has been replaced by N concrete clones.

```json
"order_card": {
  "type": "Card",
  "$each": { "path": "/orders", "as": "order" },
  "props": { "title": { "$data": "/order/order_number" } }
}
```

### Fields

- `path` — slash-separated [JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901) path to a JSON array in `spec.data`. Required.
- `as` — loop-variable name bound during expansion. Paths starting with `/{as}/...` in the templated element's props resolve to the current row. Required.

### Expansion

For each row at index `i` in the resolved array, ferro:

1. Clones the templated element.
2. Rewrites prop expressions: paths starting with `/{as}/...` resolve against the row data.
3. Assigns the clone the ID `{element_id}-{i}` (e.g., `order_card-0`, `order_card-1`, ...).
4. Removes the original templated element.
5. Updates any parent's `children` list to reference the clone IDs.

The resolve order is: `$if` first, then `$each`, then `resolve_actions`, then `resolve_expressions`. By the time props are walked for `$data` / `$template`, the element graph is already flat and concrete.

### Validation

The following errors are emitted at `Spec::from_json` time, before any data is bound:

- `EachPathNotArray` — emitted as a best-effort check when `spec.data` is non-null and the path resolves to a non-array value.
- `EachAsReservedName` — `as` collides with one of the reserved names: `data`, `root`, `_root`, `_each`, `this`, `self`.
- `NestedEach` — a `$each`-templated element's transitive descendant (deeper than direct children) is also `$each`-templated.
- `MismatchedEach` — a `$each`-templated element's direct child is `$each` over a different `{path, as}` pair than its parent.

### Correlated children

Sibling templates with the same `{path, as}` pair are correlated by index: the i-th clone of one sibling references the i-th clone of the other. The pattern shows up when a Card and its Badge / DropdownMenu children all iterate over the same source array — each card's badge and dropdown belong to the same row.

```json
"order_card": {
  "type": "Card",
  "$each": { "path": "/orders", "as": "order" },
  "props": { "title": { "$data": "/order/order_number" } },
  "children": ["order_badge"]
},
"order_badge": {
  "type": "Badge",
  "$each": { "path": "/orders", "as": "order" },
  "props": { "label": { "$data": "/order/status_label" } }
}
```

With three orders, expansion produces `order_card-0` through `order_card-2`, each referencing `order_badge-0` through `order_badge-2` correlated by index. Different `{path, as}` pairs at the same level are rejected as `MismatchedEach`.

### Limitations

- Per-row `action.url` values are not synthesized from row data inside `$each`. The controller pre-resolves URLs into `spec.data` and the templated element references them via `{ "$data": "/order/advance_url" }`.
- Nested `$each` is rejected. If a use case appears, file an issue with the data shape.

## $if

`$if` removes an element from the spec when its predicate evaluates false.

The directive is element-level: it appears as a key on the element JSON object. It is resolved before render; falsy elements are deleted from `Spec.elements` and no HTML is emitted for them.

```json
"btn_advance": {
  "type": "Button",
  "$if": { "path": "/can_advance", "operator": "eq", "value": true },
  "props": { "label": "Advance" }
}
```

### Distinction from `visible`

`$if` and the `visible` prop both gate elements on a runtime condition, but they differ in when the check runs and in what reaches the response.

- `$if` (resolve-time): falsy predicate → the element is removed from the spec. No HTML is emitted.
- `visible` (render-time): falsy condition → the element renders with hidden semantics. The HTML is present but suppressed.

Use `$if` when the element should not exist in the response at all — security-sensitive actions, structural omission, eliminating wasted bytes. Use `visible` when client-side toggling needs the DOM available.

### Predicate syntax

`$if` reuses the same predicate shape as `visible`. Both flat conditions and compound forms are accepted:

- Flat condition: `{ "path": "/p", "operator": "eq", "value": x }`.
- Compound: `{ "and": [ ... ] }`, `{ "or": [ ... ] }`, `{ "not": { ... } }`.

Operator names (snake_case): `exists`, `not_exists`, `eq`, `not_eq`, `gt`, `lt`, `gte`, `lte`, `contains`, `not_empty`, `empty`. **The canonical name is `eq`, not `equals`** — the `equals` form is not accepted.

### Validation

- `IfPathMissing` — emitted when `spec.data` is non-null and the predicate path resolves to `None` (the key is absent). This is distinct from a present-but-null value. The check fires at `Spec::from_json` time; runtime data is not validated this way.

### Interaction with `$each`

When both `$if` and `$each` are present on the same element, `$if` is evaluated first. A falsy `$if` removes the element before `$each` would have expanded it; no clones are produced.

### Missing-children behavior

When a parent's `children` list references an ID that was removed by `$if`, the render layer skips the reference. The page renders without the missing child; no error is raised. This is the same behavior the renderer already applies to references that point at nonexistent IDs.

