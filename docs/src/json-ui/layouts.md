# Layouts

Layouts wrap JSON-UI pages with consistent navigation, headers, and page structure.

## How Layouts Work

Each JSON-UI view can specify a layout name. At render time, the framework looks up the layout in a `LayoutRegistry` and wraps the rendered component HTML in a full HTML page shell.

1. View specifies a layout: `JsonUiView::new().layout("app")`
2. Components are rendered to HTML
3. The layout wraps the HTML in a complete page with `<head>`, navigation, and `<body>` structure
4. The view JSON and data are embedded as `data-view` and `data-props` attributes for potential frontend hydration

## Using a Layout

Set the layout name on the view builder:

```rust
use ferro_rs::JsonUiView;

let view = JsonUiView::new()
    .title("Dashboard")
    .layout("app");
```

If no layout is set, the `"default"` layout is used. If a named layout is not found in the registry, rendering falls back to the default layout.

## Default Layouts

Three layouts are included out of the box:

### default

Minimal HTML page. Wraps content in a valid document with doctype, meta tags, title, and the ferro-json-ui wrapper div. No navigation or sidebar.

```rust
// No .layout() call, or explicit:
let view = JsonUiView::new()
    .title("Simple Page");
```

### app

Dashboard-style layout with a horizontal navigation bar, a sidebar on the left, and a main content area on the right. Uses a flex layout. By default renders empty navigation and sidebar placeholders -- create a custom layout to populate them.

```rust
let view = JsonUiView::new()
    .title("Dashboard")
    .layout("app");
```

### auth

Centered card layout for authentication pages. Centers content vertically and horizontally within a max-width container. No navigation or sidebar.

```rust
let view = JsonUiView::new()
    .title("Login")
    .layout("auth");
```

## Creating Custom Layouts

Implement the `Layout` trait to create a custom layout:

```rust
use ferro_rs::{Layout, LayoutContext};

pub struct CustomLayout;

impl Layout for CustomLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    {head}
</head>
<body>
    <header>My App</header>
    <main>{content}</main>
    <footer>Copyright 2026</footer>
</body>
</html>"#,
            title = ctx.title,
            head = ctx.head,
            content = ctx.content,
        )
    }
}
```

The `Layout` trait requires `Send + Sync` for thread-safe access from the global registry.

### Registering Custom Layouts

Register layouts at application startup:

```rust
use ferro_rs::register_layout;

// Register globally
register_layout("custom", CustomLayout);
```

Or register directly on a `LayoutRegistry`:

```rust
use ferro_rs::LayoutRegistry;

let mut registry = LayoutRegistry::new();
registry.register("custom", CustomLayout);
```

Registering with an existing name replaces the previous layout.

## Layout Context

The `LayoutContext` struct provides all data a layout needs to produce a complete HTML page:

| Field | Type | Description |
|-------|------|-------------|
| `title` | `&str` | Page title from the view (defaults to `"Ferro"`) |
| `content` | `&str` | Rendered component HTML fragment |
| `head` | `&str` | Additional `<head>` content (Tailwind CDN, custom styles) |
| `body_class` | `&str` | CSS classes for the `<body>` element |
| `view_json` | `&str` | Serialized view JSON for the `data-view` attribute |
| `data_json` | `&str` | Serialized data JSON for the `data-props` attribute |

The `view_json` and `data_json` fields enable frontend JavaScript to hydrate the page from the server-rendered HTML. All built-in layouts embed these in a `<div id="ferro-json-ui">` wrapper.

## Navigation Helpers

The layout module provides partial rendering functions for building navigation:

### NavItem

```rust
use ferro_rs::NavItem;

let items = vec![
    NavItem::new("Home", "/").active(),
    NavItem::new("Users", "/users"),
    NavItem::new("Settings", "/settings"),
];
```

Active items are highlighted with distinct styling. The `active()` builder method marks an item as the current page.

### SidebarSection

```rust
use ferro_rs::{SidebarSection, NavItem};

let sections = vec![
    SidebarSection::new("Main Menu", vec![
        NavItem::new("Dashboard", "/"),
        NavItem::new("Users", "/users"),
    ]),
    SidebarSection::new("Settings", vec![
        NavItem::new("Profile", "/settings/profile"),
        NavItem::new("Security", "/settings/security"),
    ]),
];
```

The built-in `navigation()` and `sidebar()` functions render these into HTML with Tailwind classes. Use them in custom layouts to build consistent navigation.

## Render Configuration

`JsonUiConfig` controls rendering behavior:

```rust
use ferro_rs::JsonUiConfig;

let config = JsonUiConfig::new()
    .tailwind_cdn(false)       // Disable Tailwind CDN (default: true)
    .body_class("dark bg-black") // Custom body CSS classes
    .custom_head(r#"<link rel="stylesheet" href="/custom.css">"#);
```

| Field | Default | Description |
|-------|---------|-------------|
| `tailwind_cdn` | `true` | Include Tailwind CDN `<script>` in `<head>` |
| `custom_head` | `None` | Custom HTML to inject into `<head>` |
| `body_class` | `"bg-white text-gray-900"` | CSS classes for `<body>` |

Pass the config to the render call:

```rust
use ferro_rs::{JsonUi, JsonUiView, JsonUiConfig};

let view = JsonUiView::new()
    .title("Dashboard")
    .layout("app");

let config = JsonUiConfig::new().tailwind_cdn(false);
JsonUi::render_with_config(&view, &serde_json::json!({}), &config)
```

For production, disable the Tailwind CDN and serve your own compiled CSS via `custom_head`.
