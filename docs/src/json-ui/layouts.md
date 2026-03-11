# Layouts

Layouts wrap JSON-UI pages with consistent navigation, headers, and page structure.

## How Layouts Work

Each JSON-UI view can specify a layout name. At render time, the framework looks up the layout in a `LayoutRegistry` and wraps the rendered component HTML in a full HTML page shell.

1. View specifies a layout: `JsonUiView::new().layout("dashboard")`
2. Components are rendered to HTML
3. The layout wraps the HTML in a complete page with `<head>`, navigation, and `<body>` structure
4. The view JSON and data are embedded as `data-view` and `data-props` attributes for potential frontend hydration

## Using a Layout

Set the layout name on the view builder:

```rust
use ferro::JsonUiView;

let view = JsonUiView::new()
    .title("Dashboard")
    .layout("dashboard");
```

If no layout is set, the `"default"` layout is used. If a named layout is not found in the registry, rendering falls back to the default layout.

## Default Layout

The built-in `"default"` layout produces a minimal HTML page with no navigation or sidebar. Use it for simple pages, reports, or content that does not require persistent navigation.

```rust
// No .layout() call — uses "default" automatically:
let view = JsonUiView::new()
    .title("Report");

// Or explicitly:
let view = JsonUiView::new()
    .title("Simple Page")
    .layout("default");
```

## DashboardLayout

`DashboardLayout` is the primary layout for application dashboards. It renders a persistent sidebar on the left (collapsible on mobile), a sticky header at the top, and a content area in the main panel.

Unlike the default layout, `DashboardLayout` requires per-application configuration (sidebar navigation and header data) and must be registered at startup. The layout also injects the ferro JS runtime automatically, enabling SSE live-value updates, toast notifications, and sidebar toggle behavior.

### DashboardLayoutConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sidebar` | `SidebarProps` | Yes | Sidebar navigation data |
| `header` | `HeaderProps` | Yes | Header data (business name, user info, notifications) |
| `sse_url` | `Option<String>` | No | SSE endpoint URL for live updates |

**SidebarProps** fields:

| Field | Type | Description |
|-------|------|-------------|
| `fixed_top` | `Vec<SidebarNavItem>` | Items pinned at the top (logo, home link) |
| `groups` | `Vec<SidebarGroup>` | Collapsible navigation groups |
| `fixed_bottom` | `Vec<SidebarNavItem>` | Items pinned at the bottom (settings, logout) |

**HeaderProps** fields:

| Field | Type | Description |
|-------|------|-------------|
| `business_name` | `String` | Application name displayed in the header |
| `notification_count` | `Option<u32>` | Unread notification count for badge display |
| `user_name` | `Option<String>` | Current user's name |
| `user_avatar` | `Option<String>` | Current user's avatar URL |
| `logout_url` | `Option<String>` | URL for the logout link |

### Registering the Dashboard Layout

Register `DashboardLayout` at application startup, before the server handles requests:

```rust
use ferro::{
    DashboardLayout, DashboardLayoutConfig, HeaderProps, SidebarProps,
    SidebarGroup, SidebarNavItem, register_layout,
};

register_layout("dashboard", DashboardLayout::new(DashboardLayoutConfig {
    sidebar: SidebarProps {
        fixed_top: vec![
            SidebarNavItem {
                label: "Dashboard".to_string(),
                href: "/".to_string(),
                icon: Some("home".to_string()),
                active: false, // set per-request in handler
            },
        ],
        groups: vec![
            SidebarGroup {
                label: "Management".to_string(),
                collapsed: false,
                items: vec![
                    SidebarNavItem { label: "Users".to_string(), href: "/users".to_string(), icon: Some("users".to_string()), active: false },
                    SidebarNavItem { label: "Orders".to_string(), href: "/orders".to_string(), icon: Some("shopping-bag".to_string()), active: false },
                ],
            },
        ],
        fixed_bottom: vec![
            SidebarNavItem { label: "Settings".to_string(), href: "/settings".to_string(), icon: Some("cog".to_string()), active: false },
        ],
    },
    header: HeaderProps {
        business_name: "My App".to_string(),
        notification_count: None,
        user_name: Some("Alice".to_string()),
        user_avatar: None,
        logout_url: Some("/logout".to_string()),
    },
    sse_url: Some("/dashboard/events".into()),
}));
```

### Using It in a View

```rust
use ferro::{JsonUiView, ComponentNode, ComponentNode::stat_card, StatCardProps};

let view = JsonUiView::new()
    .title("Dashboard")
    .layout("dashboard")
    .component(ComponentNode::stat_card("revenue", StatCardProps {
        label: "Total Revenue".to_string(),
        value: "€12,345".to_string(),
        icon: Some("currency-euro".to_string()),
        subtitle: Some("This month".to_string()),
        sse_target: Some("revenue_total".to_string()),
    }));
```

### Mobile Behavior

On screens narrower than the `md` breakpoint (768px):

- The sidebar is hidden by default (`hidden md:flex` Tailwind class)
- A hamburger button appears in the header (`data-sidebar-toggle`)
- Clicking the hamburger toggles the `data-sidebar-open` attribute on the `<body>` element
- The JS runtime toggles sidebar visibility in response

No additional JavaScript configuration is needed. The runtime handles this automatically.

### JS Runtime

The `DashboardLayout` injects the ferro JS runtime as a `<script>` tag before `</body>`. The runtime is a small self-contained IIFE that activates on `DOMContentLoaded` and handles three behaviors:

**Sidebar toggle** — The hamburger button toggles mobile sidebar visibility.

**SSE live-value updates** — If `sse_url` is set on `DashboardLayoutConfig`, the runtime opens an `EventSource` connection. Incoming `live-value` events update elements with matching `data-sse-target` attributes. Use this with `StatCard.sse_target` to update metric values without page reloads.

Server-sent event format:
```
event: live-value
data: {"target": "revenue_total", "value": "€13,210"}
```

**Toast notifications** — Incoming `toast` events display overlay notifications. A `data-toast-container` div is injected by the layout for mounting toasts.

Server-sent event format:
```
event: toast
data: {"message": "New order received", "variant": "success"}
```

You can also display toasts declaratively by including a `Toast` component in any view rendered by `DashboardLayout`.

## Creating Custom Layouts

Implement the `Layout` trait to create a custom layout:

```rust
use ferro::{Layout, LayoutContext};

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
    {scripts}
</body>
</html>"#,
            title = ctx.title,
            head = ctx.head,
            content = ctx.content,
            scripts = ctx.scripts,
        )
    }
}
```

The `Layout` trait requires `Send + Sync` for thread-safe access from the global registry.

### Registering Custom Layouts

Register layouts at application startup:

```rust
use ferro::register_layout;

register_layout("custom", CustomLayout);
```

Or register directly on a `LayoutRegistry`:

```rust
use ferro::LayoutRegistry;

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
| `scripts` | `&str` | JS assets and init scripts for plugins, injected before closing body tag |

The `view_json` and `data_json` fields enable frontend JavaScript to hydrate the page from the server-rendered HTML. All built-in layouts embed these in a `<div id="ferro-json-ui">` wrapper.

Always include `ctx.scripts` in custom layouts — it contains plugin JS assets and the ferro runtime when `render_to_html_with_plugins` is used.

## Navigation Helpers

The layout module provides partial rendering functions for building navigation:

### NavItem

```rust
use ferro::NavItem;

let items = vec![
    NavItem::new("Home", "/").active(),
    NavItem::new("Users", "/users"),
    NavItem::new("Settings", "/settings"),
];
```

Active items are highlighted with distinct styling. The `active()` builder method marks an item as the current page.

### SidebarSection

```rust
use ferro::{SidebarSection, NavItem};

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

The built-in `navigation()` and `sidebar()` functions render these into HTML with Tailwind classes. Use them in fully custom layout implementations to build consistent navigation.

## Render Configuration

`JsonUiConfig` controls rendering behavior:

```rust
use ferro::JsonUiConfig;

let config = JsonUiConfig::new()
    .tailwind_cdn(false)          // Disable Tailwind CDN (default: true)
    .body_class("dark bg-black")  // Custom body CSS classes
    .custom_head(r#"<link rel="stylesheet" href="/custom.css">"#);
```

| Field | Default | Description |
|-------|---------|-------------|
| `tailwind_cdn` | `true` | Include Tailwind CDN `<script>` in `<head>` |
| `custom_head` | `None` | Custom HTML to inject into `<head>` |
| `body_class` | `"bg-white text-gray-900"` | CSS classes for `<body>` |

Pass the config to the render call:

```rust
use ferro::{JsonUi, JsonUiView, JsonUiConfig};

let view = JsonUiView::new()
    .title("Dashboard")
    .layout("dashboard");

let config = JsonUiConfig::new().tailwind_cdn(false);
JsonUi::render_with_config(&view, &serde_json::json!({}), &config)
```

For production, disable the Tailwind CDN and serve your own compiled CSS via `custom_head`.
