# ferro-json-ui

JSON-based server-driven UI schema types for the [Ferro](https://ferro-rs.dev) web framework.

Define UI components, layouts, and data bindings as structured JSON. Ferro renders them to
HTML on the server — no frontend build step required.

## Features

- **30+ built-in components** — tables, forms, cards, alerts, badges, buttons, tabs, charts, and more
- **Layout system** — stack, grid, sidebar, and dashboard layout primitives
- **Action system** — navigate, submit form, call API, open modal, trigger events
- **Data binding** — field-level value extraction with validation error resolution
- **Plugin system** — register custom components with associated CSS/JS assets
- **Compile-time validation** — component schema is checked at compile time via schemars

## Usage

```rust
use ferro_json_ui::{JsonUiView, Component, LayoutComponent};

let view = JsonUiView {
    layout: LayoutComponent::Stack {
        gap: Some("md".into()),
        children: vec![
            Component::Heading {
                text: "Users".into(),
                level: 1,
            },
            Component::Table {
                columns: vec!["Name".into(), "Email".into(), "Role".into()],
                rows: users
                    .iter()
                    .map(|u| vec![u.name.clone(), u.email.clone(), u.role.clone()])
                    .collect(),
                actions: vec![],
            },
        ],
    },
};

// In a Ferro handler, return the view directly
Ok(view.into_response())
```

## Documentation

Full documentation at [docs.ferro-rs.dev](https://docs.ferro-rs.dev).

## License

MIT
