# Data Binding

In JSON-UI, data flows from the handler into spec elements through two expression shapes placed as prop values. The handler provides a plain JSON object; expressions read from it at render time.

## Handler Data Shape

The handler assembles data and passes it as the second argument to `JsonUi::render_file`:

```rust
#[handler]
pub async fn index(req: Request) -> Response {
    let orders = Order::find_all(&req.db()).await?;
    let stats = Stats::load(&req.db()).await?;

    JsonUi::render_file(
        "src/views/orders.json",
        serde_json::json!({
            "orders": orders,
            "stats": {
                "total": stats.total,
                "revenue": stats.revenue_formatted
            },
            "user": {
                "name": req.auth_user().name,
                "role": req.auth_user().role
            }
        }),
    )
}
```

`render_file` loads the spec file (with caching), merges the handler data into `spec.data`, resolves all expressions, and returns the rendered HTML response. The handler contains no component-building code — only data assembly.

## Expressions

Expressions are JSON objects with a single recognized key. They appear as prop values inside elements. There are two expression types: `$data` and `$template`.

### $data — type-preserving extraction

Format:
```json
{ "$data": "/json/pointer/path" }
```

`$data` extracts the value at the given [JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901) path from the spec's data context. The resolved value preserves its original type — a string stays a string, a number stays a number, a boolean stays a boolean, an array stays an array.

Missing paths resolve to `null`.

**Example — extract a number:**
```json
"revenue_stat": {
  "type": "StatCard",
  "props": {
    "label": "Total Revenue",
    "value": { "$data": "/stats/revenue" }
  }
}
```

If the data is `{ "stats": { "revenue": 12345 } }`, the `value` prop resolves to `12345` (number).

**Example — extract a string:**
```json
"user_badge": {
  "type": "Badge",
  "props": {
    "label": { "$data": "/user/role" }
  }
}
```

With data `{ "user": { "role": "admin" } }`, `label` resolves to `"admin"` (string).

**Example — extract an array:**
```json
"product_list": {
  "type": "DataTable",
  "props": {
    "data_path": "/products"
  }
}
```

Note: `data_path` in `DataTable`, `KanbanBoard`, `Input`, `Select`, `Checkbox`, and `Switch` is a plain string JSON Pointer — not a `$data` expression. The component reads rows, columns, or pre-fills values from that path at render time, but the path itself is literal.

### $template — string interpolation

Format:
```json
{ "$template": "literal text {/path} more text" }
```

`$template` produces a string by substituting `{/path}` placeholders with values from the data context. Each placeholder uses JSON Pointer syntax. The result is always a string regardless of what the placeholders resolve to.

Missing placeholders substitute as `""` (empty string). To emit a literal `{` or `}` character, escape it with a backslash: `\{` and `\}`.

**Example — greeting message:**
```json
"welcome_text": {
  "type": "Text",
  "props": {
    "content": { "$template": "Welcome, {/user/name}!" },
    "element": "h2"
  }
}
```

With data `{ "user": { "name": "Alice" } }`, `content` resolves to `"Welcome, Alice!"`.

**Example — formatted label:**
```json
"order_heading": {
  "type": "Text",
  "props": {
    "content": { "$template": "Order #{/order/id} — {/order/status}" },
    "element": "h3"
  }
}
```

With data `{ "order": { "id": 1042, "status": "pending" } }`, `content` resolves to `"Order #1042 — pending"`.

## Where Expressions Apply

Expressions are resolved in `element.props` values only. They are **not** resolved in:

- `spec.title` — literal string
- `spec.layout` — literal string
- `spec.data` — the data source itself
- `element.children` — always a list of element ID strings
- `element.action` — handler name and method are literal
- `element.visible` — visibility condition fields are literal

Expressions can appear at any depth inside a props object or array, but not as keys — only as values.

## Complete Example

A handler + spec showing `$data`, `$template`, and a plain `data_path`:

**Handler (`src/controllers/payments.rs`):**
```rust
#[handler]
pub async fn index(req: Request) -> Response {
    let payments = Payment::find_all(&req.db()).await?;
    let total = payments.iter().map(|p| p.amount).sum::<f64>();

    JsonUi::render_file(
        "src/views/payments.json",
        serde_json::json!({
            "payments": payments,
            "stats": {
                "total": format!("€{:.2}", total),
                "count": payments.len()
            },
            "user": {
                "name": req.auth_user().name
            }
        }),
    )
}
```

**Spec (`src/views/payments.json`):**
```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Payments",
  "layout": "dashboard",
  "root": "page_header",
  "elements": {
    "page_header": {
      "type": "PageHeader",
      "props": {
        "title": "Payments",
        "description": { "$template": "Welcome, {/user/name}" }
      },
      "children": ["stats_grid"]
    },
    "stats_grid": {
      "type": "Grid",
      "props": { "columns": 2, "gap": "md" },
      "children": ["total_stat", "count_stat"]
    },
    "total_stat": {
      "type": "StatCard",
      "props": {
        "label": "Total",
        "value": { "$data": "/stats/total" }
      }
    },
    "count_stat": {
      "type": "StatCard",
      "props": {
        "label": "Payments",
        "value": { "$data": "/stats/count" }
      }
    },
    "payments_table": {
      "type": "DataTable",
      "props": {
        "data_path": "/payments",
        "columns": [
          { "key": "date", "label": "Date", "format": "date" },
          { "key": "description", "label": "Description" },
          { "key": "amount", "label": "Amount", "format": "currency" },
          { "key": "status", "label": "Status" }
        ],
        "empty_message": "No payments recorded."
      }
    }
  }
}
```

`$data` and `$template` resolve at render time from the handler data. `data_path` in `DataTable` is a plain string pointer the component uses to read rows from the same data.

## Single-Pass Guarantee

If a `$data` expression resolves to a string value that looks like `{"$data": "/another/path"}`, the inner expression is not re-resolved. This is intentional — it prevents injection. Expressions are evaluated in a single pass.

## Hard Cap — What Does Not Exist

The expression language is intentionally minimal. The following do **not** exist and will not be added:

- `$if` — no conditional rendering in expressions
- `$for` — no loops in expressions
- `$state` — no client-side state
- `$bind` — no two-way binding
- `$map` — no array transformation in expressions

Conditional logic belongs in the Rust handler. Use the `visible` condition on elements for simple show/hide logic based on data values (see [Visibility](visibility.md)). Complex branching is handled server-side before `render_file` is called — shape the data differently, or call `render_file` with a different spec path.

## Visibility Conditions

Elements can be conditionally shown or hidden with the `"visible"` field. Visibility is not an expression — it is a separate condition checked against data paths during render.

```json
"admin_panel": {
  "type": "Card",
  "props": { "title": "Admin Tools" },
  "visible": { "field": "/user/role", "op": "eq", "value": "admin" }
}
```

For the full visibility operator reference, see [Visibility](visibility.md).

## data_path Reference

`data_path` is a plain JSON-pointer string (not a `$data` expression). It is supported by three component families and follows a consistent resolution convention: the path is walked against the full handler data object, and the component reads whatever value resides there.

### DataTable — row array

```
data_path: "/data/{service.name}"
```

Resolves to an array of row objects. Each row is one `<tr>`; column `key` fields project as cell text.

```json
"staff_table": {
  "type": "DataTable",
  "props": {
    "data_path": "/data/staff",
    "columns": [
      { "key": "name",   "label": "Name" },
      { "key": "active", "label": "Active", "format": "boolean" }
    ]
  }
}
```

Handler provides `{ "data": { "staff": [ ... ] } }`.

### KanbanBoard — column array

```
data_path: "/data/{service.name}/columns"
```

Resolves to an array of `KanbanColumnProps` objects (`{ id, title, count, children }`). The `/columns` suffix distinguishes the column array from the flat item array at `/data/{service.name}`.

```json
"order_kanban": {
  "type": "KanbanBoard",
  "props": {
    "data_path": "/data/order/columns",
    "columns": []
  }
}
```

Handler provides:

```json
{
  "data": {
    "order": {
      "columns": [
        { "id": "draft",     "title": "Draft",     "count": 2, "children": [] },
        { "id": "submitted", "title": "Submitted", "count": 1, "children": [] }
      ]
    }
  }
}
```

The static `columns` array in the spec (derived from the service's state machine when using the projection renderer) serves as the schema reference and render fallback when `data_path` fails to resolve. When `data_path` resolves, it takes precedence.

### StatCard — scalar value

`StatCardProps.value_path` resolves to a single scalar (string or number):

```
value_path: "/data/{service.name}/{field.name}"
```

```json
"revenue_stat": {
  "type": "StatCard",
  "props": {
    "label": "Total Revenue",
    "value": "",
    "value_path": "/data/statistics/total_revenue"
  }
}
```

Handler provides `{ "data": { "statistics": { "total_revenue": "€12,450" } } }`. The static `value` string is the fallback when `value_path` is absent or fails to resolve.
