# JSON-UI

JSON-UI is a server-driven UI system that renders Tailwind-styled HTML from Rust data structures. No frontend build step, no React, no Node.js -- define your interface as a component tree and the framework renders it to HTML.

## How It Works

1. Define a `JsonUiView` containing a tree of `ComponentNode` values
2. Attach data, actions, and visibility rules to components
3. Call `JsonUi::render()` to produce a full HTML page with Tailwind classes
4. The framework resolves route names to URLs and binds data automatically

JSON-UI is an alternative to [Inertia.js](inertia.md). Both use the same handler pattern and return `Response`, but JSON-UI outputs server-rendered HTML while Inertia delegates rendering to a React frontend.

## When to Use JSON-UI vs Inertia

| Use Case | JSON-UI | Inertia |
|----------|---------|---------|
| Admin panels and dashboards | Ideal | Overkill |
| CRUD applications | Ideal | Works, but heavier setup |
| Rapid prototyping | Ideal | Slower iteration |
| Server-rendered pages | Built for this | Not designed for this |
| Rich interactive UIs | Limited | Ideal |
| Complex client state | Not suited | Ideal |
| SPA behavior | Not suited | Ideal |

Both can coexist in the same application on different routes.

## Quick Example

```rust
use ferro::{handler, JsonUi, JsonUiView, ComponentNode, Component, CardProps, TableProps,
    Column, Action, Response};

#[handler]
pub async fn index() -> Response {
    let view = JsonUiView::new()
        .title("Users")
        .layout("app")
        .component(ComponentNode {
            key: "header".to_string(),
            component: Component::Card(CardProps {
                title: "User Management".to_string(),
                description: Some("View and manage users".to_string()),
                children: vec![],
                footer: vec![],
            }),
            action: None,
            visibility: None,
        })
        .component(ComponentNode {
            key: "users-table".to_string(),
            component: Component::Table(TableProps {
                columns: vec![
                    Column { key: "name".to_string(), label: "Name".to_string(), format: None },
                    Column { key: "email".to_string(), label: "Email".to_string(), format: None },
                ],
                data_path: "/data/users".to_string(),
                row_actions: None,
                empty_message: Some("No users found".to_string()),
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });

    let data = serde_json::json!({
        "users": [
            {"name": "Alice", "email": "alice@example.com"},
            {"name": "Bob", "email": "bob@example.com"},
        ]
    });

    JsonUi::render(&view, &data)
}
```

## Key Concepts

- **[Components](../json-ui/components.md)** -- 20 built-in component types: Card, Table, Form, Button, Input, Select, Alert, Badge, Modal, Text, Checkbox, Switch, Separator, DescriptionList, Tabs, Breadcrumb, Pagination, Progress, Avatar, and Skeleton.

- **[Actions](../json-ui/actions.md)** -- Route-based navigation and form submission. Actions reference handler names (`"users.store"`) that resolve to URLs at render time.

- **[Data Binding & Visibility](../json-ui/data-binding.md)** -- Pre-fill form fields from handler data via `data_path`, and conditionally show/hide components with visibility rules.

- **[Layouts](../json-ui/layouts.md)** -- Page structure with navigation. Built-in `"app"` layout includes sidebar and header; `"auth"` layout centers content. Custom layouts via the `Layout` trait.

## CLI Support

Scaffold views with the CLI:

```bash
ferro make:json-view UserIndex
```

The command uses AI-powered generation when an Anthropic API key is configured. It reads your models and routes to produce a complete view file. Without an API key, it falls back to a static template.

See [CLI Reference](../reference/cli.md) for details.
