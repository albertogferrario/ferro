# Components

JSON-UI includes 26 built-in component types organized into six groups. Every component serializes to JSON with a `"type"` discriminant and is wrapped in a `ComponentNode` that adds a unique key, an optional action binding, and optional visibility rules.

## Component Overview

| Category | Components |
|----------|-----------|
| **Layout** | Card, Tabs, Separator, Modal, Skeleton |
| **Data Display** | Table, DescriptionList, Badge, Avatar, Text, Progress, Breadcrumb, Pagination, StatCard |
| **Forms** | Form, Input, Select, Checkbox, Switch, Button |
| **Feedback** | Alert, Toast |
| **Navigation** | Sidebar, Header, NotificationDropdown |
| **Onboarding** | Checklist |
| **Extensible** | Plugin |

## ComponentNode

Every component is wrapped in a `ComponentNode`:

```rust
use ferro::{ComponentNode, Component, CardProps};

ComponentNode {
    key: "my-card".to_string(),          // unique identifier on the page
    component: Component::Card(CardProps { /* ... */ }),
    action: None,                         // optional Action binding
    visibility: None,                     // optional Visibility condition
}
```

Convenience constructors are available for common components:

```rust
// Equivalent to the struct literal above, but more concise
let node = ComponentNode::card("my-card", CardProps {
    title: "Hello".to_string(),
    description: None,
    children: vec![],
    footer: vec![],
});
```

## Shared Types

### Size

Controls sizing for Button, Avatar, and other components.

| Value | Serialized |
|-------|-----------|
| `Size::Xs` | `"xs"` |
| `Size::Sm` | `"sm"` |
| `Size::Default` | `"default"` |
| `Size::Lg` | `"lg"` |

### ButtonVariant

Visual styles for the Button component (aligned to shadcn/ui).

| Value | Serialized | Use Case |
|-------|-----------|----------|
| `ButtonVariant::Default` | `"default"` | Primary actions |
| `ButtonVariant::Secondary` | `"secondary"` | Secondary actions |
| `ButtonVariant::Destructive` | `"destructive"` | Delete, remove |
| `ButtonVariant::Outline` | `"outline"` | Bordered style |
| `ButtonVariant::Ghost` | `"ghost"` | Minimal style |
| `ButtonVariant::Link` | `"link"` | Link appearance |

### AlertVariant

Visual styles for Alert and Toast components.

| Value | Serialized |
|-------|-----------|
| `AlertVariant::Info` | `"info"` |
| `AlertVariant::Success` | `"success"` |
| `AlertVariant::Warning` | `"warning"` |
| `AlertVariant::Error` | `"error"` |

### BadgeVariant

Visual styles for the Badge component (aligned to shadcn/ui).

| Value | Serialized |
|-------|-----------|
| `BadgeVariant::Default` | `"default"` |
| `BadgeVariant::Secondary` | `"secondary"` |
| `BadgeVariant::Destructive` | `"destructive"` |
| `BadgeVariant::Outline` | `"outline"` |

### ColumnFormat

Display format for Table columns and DescriptionList items.

| Value | Serialized |
|-------|-----------|
| `ColumnFormat::Date` | `"date"` |
| `ColumnFormat::DateTime` | `"date_time"` |
| `ColumnFormat::Currency` | `"currency"` |
| `ColumnFormat::Boolean` | `"boolean"` |

### TextElement

Semantic HTML element for the Text component.

| Value | Serialized | HTML |
|-------|-----------|------|
| `TextElement::P` | `"p"` | `<p>` |
| `TextElement::H1` | `"h1"` | `<h1>` |
| `TextElement::H2` | `"h2"` | `<h2>` |
| `TextElement::H3` | `"h3"` | `<h3>` |
| `TextElement::Span` | `"span"` | `<span>` |
| `TextElement::Div` | `"div"` | `<div>` |
| `TextElement::Section` | `"section"` | `<section>` |

### ToastVariant

Visual styles for the Toast component. Mirrors AlertVariant.

| Value | Serialized |
|-------|-----------|
| `ToastVariant::Info` | `"info"` |
| `ToastVariant::Success` | `"success"` |
| `ToastVariant::Warning` | `"warning"` |
| `ToastVariant::Error` | `"error"` |

---

## Layout Components

### Card

Container with title, optional description, nested children, and footer.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | `String` | Yes | - | Card title |
| `description` | `Option<String>` | No | `None` | Description below the title |
| `children` | `Vec<ComponentNode>` | No | `[]` | Nested components in the card body |
| `footer` | `Vec<ComponentNode>` | No | `[]` | Components in the card footer |

```rust
use ferro::{ComponentNode, CardProps, ButtonProps, ButtonVariant, Size};

let node = ComponentNode::card("user-card", CardProps {
    title: "User Details".to_string(),
    description: Some("Account information".to_string()),
    children: vec![
        ComponentNode::button("edit-btn", ButtonProps {
            label: "Edit".to_string(),
            variant: ButtonVariant::Outline,
            size: Size::Default,
            disabled: None,
            icon: None,
            icon_position: None,
        }),
    ],
    footer: vec![],
});
```

JSON output:

```json
{
  "key": "user-card",
  "type": "Card",
  "title": "User Details",
  "description": "Account information",
  "children": [
    { "key": "edit-btn", "type": "Button", "label": "Edit", "variant": "outline", "size": "default" }
  ]
}
```

### Tabs

Tabbed content with multiple panels. Each tab contains its own set of child components.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `default_tab` | `String` | Yes | - | Value of the initially active tab |
| `tabs` | `Vec<Tab>` | Yes | - | Tab definitions |

**Tab** defines a tab panel:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `value` | `String` | Yes | Tab identifier (matches `default_tab`) |
| `label` | `String` | Yes | Tab label text |
| `children` | `Vec<ComponentNode>` | No | Components displayed when the tab is active |

```rust
use ferro::{ComponentNode, Component, TabsProps, Tab};

ComponentNode {
    key: "settings-tabs".to_string(),
    component: Component::Tabs(TabsProps {
        default_tab: "general".to_string(),
        tabs: vec![
            Tab {
                value: "general".to_string(),
                label: "General".to_string(),
                children: vec![/* ... */],
            },
            Tab {
                value: "security".to_string(),
                label: "Security".to_string(),
                children: vec![/* ... */],
            },
        ],
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "settings-tabs",
  "type": "Tabs",
  "default_tab": "general",
  "tabs": [
    { "value": "general", "label": "General", "children": [] },
    { "value": "security", "label": "Security", "children": [] }
  ]
}
```

### Separator

Visual divider between content sections.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `orientation` | `Option<Orientation>` | No | `Horizontal` | Direction: `horizontal` or `vertical` |

```rust
use ferro::{ComponentNode, Component, SeparatorProps};

ComponentNode {
    key: "divider".to_string(),
    component: Component::Separator(SeparatorProps {
        orientation: None, // defaults to horizontal
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "divider", "type": "Separator" }
```

### Modal

Dialog overlay with title, content, footer, and trigger button.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | `String` | Yes | - | Modal title |
| `description` | `Option<String>` | No | `None` | Modal description |
| `children` | `Vec<ComponentNode>` | No | `[]` | Content components inside the modal body |
| `footer` | `Vec<ComponentNode>` | No | `[]` | Components in the modal footer |
| `trigger_label` | `Option<String>` | No | `None` | Label for the trigger button |

```rust
use ferro::{ComponentNode, Component, ModalProps, ButtonProps, ButtonVariant, Size, Action};

ComponentNode {
    key: "delete-modal".to_string(),
    component: Component::Modal(ModalProps {
        title: "Delete Item".to_string(),
        description: Some("This action cannot be undone.".to_string()),
        children: vec![],
        footer: vec![
            ComponentNode {
                key: "delete-btn".to_string(),
                component: Component::Button(ButtonProps {
                    label: "Delete".to_string(),
                    variant: ButtonVariant::Destructive,
                    size: Size::Default,
                    disabled: None,
                    icon: None,
                    icon_position: None,
                }),
                action: Some(Action::delete("items.destroy").confirm_danger("Confirm deletion")),
                visibility: None,
            },
        ],
        trigger_label: Some("Delete".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "delete-modal",
  "type": "Modal",
  "title": "Delete Item",
  "description": "This action cannot be undone.",
  "trigger_label": "Delete",
  "children": [],
  "footer": [{ "key": "delete-btn", "type": "Button", "label": "Delete", "variant": "destructive", "size": "default" }]
}
```

### Skeleton

Loading placeholder with configurable dimensions for content that is still loading.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `width` | `Option<String>` | No | `None` | CSS width (e.g., `"100%"`, `"200px"`) |
| `height` | `Option<String>` | No | `None` | CSS height (e.g., `"40px"`) |
| `rounded` | `Option<bool>` | No | `None` | Whether to use rounded corners |

```rust
use ferro::{ComponentNode, Component, SkeletonProps};

ComponentNode {
    key: "loading-placeholder".to_string(),
    component: Component::Skeleton(SkeletonProps {
        width: Some("100%".to_string()),
        height: Some("40px".to_string()),
        rounded: Some(true),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "loading-placeholder", "type": "Skeleton", "width": "100%", "height": "40px", "rounded": true }
```

---

## Data Display Components

### Table

Data table with column definitions, row actions, and sorting support. Rows are loaded from handler data via `data_path`.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `columns` | `Vec<Column>` | Yes | - | Column definitions |
| `data_path` | `String` | Yes | - | Path to the row data array (e.g., `"/data/users"`) |
| `row_actions` | `Option<Vec<Action>>` | No | `None` | Actions available per row |
| `empty_message` | `Option<String>` | No | `None` | Message when no data |
| `sortable` | `Option<bool>` | No | `None` | Enable column sorting |
| `sort_column` | `Option<String>` | No | `None` | Currently sorted column key |
| `sort_direction` | `Option<SortDirection>` | No | `None` | Sort direction: `asc` or `desc` |

**Column** defines a table column:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `String` | Yes | Data field key matching the row object |
| `label` | `String` | Yes | Column header text |
| `format` | `Option<ColumnFormat>` | No | Display format (Date, DateTime, Currency, Boolean) |

```rust
use ferro::{ComponentNode, TableProps, Column, ColumnFormat, Action};

let node = ComponentNode::table("users-table", TableProps {
    columns: vec![
        Column { key: "name".to_string(), label: "Name".to_string(), format: None },
        Column { key: "email".to_string(), label: "Email".to_string(), format: None },
        Column {
            key: "created_at".to_string(),
            label: "Created".to_string(),
            format: Some(ColumnFormat::Date),
        },
    ],
    data_path: "/data/users".to_string(),
    row_actions: Some(vec![
        Action::get("users.edit"),
        Action::delete("users.destroy").confirm_danger("Delete this user?"),
    ]),
    empty_message: Some("No users found.".to_string()),
    sortable: Some(true),
    sort_column: None,
    sort_direction: None,
});
```

JSON output:

```json
{
  "key": "users-table",
  "type": "Table",
  "data_path": "/data/users",
  "columns": [
    { "key": "name", "label": "Name" },
    { "key": "email", "label": "Email" },
    { "key": "created_at", "label": "Created", "format": "date" }
  ],
  "sortable": true
}
```

### DescriptionList

Key-value pairs displayed as a description list. Reuses `ColumnFormat` for value formatting.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `items` | `Vec<DescriptionItem>` | Yes | - | Key-value items |
| `columns` | `Option<u8>` | No | `None` | Number of columns for layout |

**DescriptionItem** defines a key-value pair:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `String` | Yes | Item label |
| `value` | `String` | Yes | Item value |
| `format` | `Option<ColumnFormat>` | No | Display format |

```rust
use ferro::{ComponentNode, Component, DescriptionListProps, DescriptionItem, ColumnFormat};

ComponentNode {
    key: "user-info".to_string(),
    component: Component::DescriptionList(DescriptionListProps {
        items: vec![
            DescriptionItem { label: "Name".to_string(), value: "Alice Johnson".to_string(), format: None },
            DescriptionItem {
                label: "Joined".to_string(),
                value: "2026-01-15".to_string(),
                format: Some(ColumnFormat::Date),
            },
            DescriptionItem {
                label: "Active".to_string(),
                value: "true".to_string(),
                format: Some(ColumnFormat::Boolean),
            },
        ],
        columns: Some(2),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "user-info",
  "type": "DescriptionList",
  "columns": 2,
  "items": [
    { "label": "Name", "value": "Alice Johnson" },
    { "label": "Joined", "value": "2026-01-15", "format": "date" },
    { "label": "Active", "value": "true", "format": "boolean" }
  ]
}
```

### DetailForm

Split-mode detail page with inline edit. Renders the same description-list-style
scaffold in two modes — View and Edit — driven by a URL query parameter.

**When to use.** Use `DetailForm` instead of a pair of `DescriptionList` (for
viewing) + `Form` (for editing) when you want the view and edit states to share
a single structural container, so the user sees the same layout whether they are
reading or editing. The mode toggle is URL-driven (`?mode=edit`); there is no
client-side JavaScript state.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `mode` | `EditMode` | No | `EditMode::View` | Which mode to render. Typically derived from the URL via `EditMode::from_query(req.query("mode").as_deref())` |
| `action` | `Action` | Yes | - | Form submit target. Resolver populates `action.url` from `action.handler` |
| `fields` | `Vec<DetailField>` | Yes | - | The rows |
| `edit_url` | `String` | Yes | - | Href for the "Modifica" link in View mode. Emitted verbatim after `html_escape`; not resolved by the route registry |
| `cancel_url` | `String` | Yes | - | Href for the "Annulla" link in Edit mode. Emitted verbatim after `html_escape`; not resolved by the route registry |
| `edit_label` | `Option<String>` | No | `"Modifica"` | Override for the default "Modifica" label |
| `save_label` | `Option<String>` | No | `"Salva"` | Override for the default "Salva" label |
| `cancel_label` | `Option<String>` | No | `"Annulla"` | Override for the default "Annulla" label |
| `method` | `Option<HttpMethod>` | No | - | HTTP method override (else uses `action.method`); PUT/PATCH/DELETE auto-emit `<input type="hidden" name="_method">` spoofing |

**DetailField** defines one row:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `String` | Yes | Description term shown in both modes as the field label |
| `value` | `String` | Yes | Display string shown in View mode (plain text, html-escaped at render) |
| `input` | `ComponentNode` | Yes | Component rendered in Edit mode in place of `value` (typically `Input`, `Select`, `Textarea`, `Switch`, `Checkbox`, or a plugin) |

**EditMode** controls which mode to render:

| Variant | JSON | Description |
|---------|------|-------------|
| `View` | `"view"` | Read-only display with a "Modifica" link (default) |
| `Edit` | `"edit"` | Inline-edit form with "Salva" / "Annulla" actions |

```rust
use ferro::{ComponentNode, Component, DetailForm, DetailFormProps, DetailField,
            EditMode, InputProps, InputType, Action, HttpMethod};

let mode = EditMode::from_query(req.query("mode").as_deref());

let node = ComponentNode::detail_form(
    "user-detail",
    DetailFormProps {
        mode,
        action: Action::new("users.update"),
        fields: vec![
            DetailField::new(
                "Name",
                user.name.clone(),
                // Option A: input label must be "" — the <dt> provides the visible label.
                ComponentNode::input("name", InputProps {
                    field: "name".to_string(),
                    label: "".to_string(),
                    input_type: InputType::Text,
                    default_value: Some(user.name.clone()),
                    ..Default::default()
                }),
            ),
            DetailField::new(
                "Email",
                user.email.clone(),
                ComponentNode::input("email", InputProps {
                    field: "email".to_string(),
                    label: "".to_string(),
                    input_type: InputType::Email,
                    default_value: Some(user.email.clone()),
                    ..Default::default()
                }),
            ),
        ],
        edit_url: format!("/users/{}?mode=edit", user.id),
        cancel_url: format!("/users/{}", user.id),
        edit_label: None,
        save_label: None,
        cancel_label: None,
        method: Some(HttpMethod::Put),
    },
);
```

JSON output (mode = Edit):

```json
{
  "key": "user-detail",
  "type": "DetailForm",
  "mode": "edit",
  "action": { "handler": "users.update", "method": "PUT" },
  "fields": [
    {
      "label": "Name",
      "value": "Ada Lovelace",
      "input": { "type": "Input", "field": "name", "label": "", "input_type": "text", "default_value": "Ada Lovelace" }
    },
    {
      "label": "Email",
      "value": "ada@example.com",
      "input": { "type": "Input", "field": "email", "label": "", "input_type": "email", "default_value": "ada@example.com" }
    }
  ],
  "edit_url": "/users/1?mode=edit",
  "cancel_url": "/users/1",
  "method": "PUT"
}
```

**Authoring rule (Option A).** When a `DetailField.input` is an `Input`, `Select`,
`Textarea`, `Checkbox`, or `Switch` component, the caller MUST set its `label` prop
to the empty string `""`. The `<dt>` already provides the visible label; a non-empty
input label produces duplicate UI text. DetailForm does not mutate caller-supplied
props. For accessibility, callers SHOULD also set `aria-label` on each input derived
from the field's `label` value so screen readers retain the field name.

**Not included in v1.** Client-side mode toggle (no JS). Optimistic updates.
Per-field mode override. Custom action buttons beyond Modifica/Salva/Annulla.
Top-level error banner. i18n binding for default labels (currently Italian literals).

### Badge

Small label with variant-based styling.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `label` | `String` | Yes | - | Badge text |
| `variant` | `BadgeVariant` | No | `Default` | Visual style |

```rust
use ferro::{ComponentNode, Component, BadgeProps, BadgeVariant};

ComponentNode {
    key: "status".to_string(),
    component: Component::Badge(BadgeProps {
        label: "Active".to_string(),
        variant: BadgeVariant::Default,
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "status", "type": "Badge", "label": "Active", "variant": "default" }
```

### Avatar

User avatar with image source, fallback text, and size variants.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | `Option<String>` | No | `None` | Image URL |
| `alt` | `String` | Yes | - | Alt text (required for accessibility) |
| `fallback` | `Option<String>` | No | `None` | Fallback initials when no image |
| `size` | `Option<Size>` | No | `Default` | Avatar size: `xs`, `sm`, `default`, `lg` |

```rust
use ferro::{ComponentNode, Component, AvatarProps, Size};

ComponentNode {
    key: "user-avatar".to_string(),
    component: Component::Avatar(AvatarProps {
        src: Some("/images/alice.jpg".to_string()),
        alt: "Alice Johnson".to_string(),
        fallback: Some("AJ".to_string()),
        size: Some(Size::Lg),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "user-avatar", "type": "Avatar", "alt": "Alice Johnson", "src": "/images/alice.jpg", "fallback": "AJ", "size": "lg" }
```

### Image

Renders a bounded visual asset — either an external image via URL, or a
server-constructed inline SVG.

**Props**

| Name                | Type              | Required  | Description                                                                              |
|---------------------|-------------------|-----------|------------------------------------------------------------------------------------------|
| `src`               | `String`          | one-of    | Image source URL (URL variant). Attribute is HTML-escaped.                               |
| `svg`               | `String`          | one-of    | Inline SVG emitted verbatim (SVG variant). See **Safety note** below.                    |
| `alt`               | `String`          | yes       | Alt text for accessibility — required on both variants (compile-enforced).               |
| `aspect_ratio`      | `Option<String>`  | no        | CSS aspect ratio (e.g., `"16/9"`).                                                       |
| `placeholder_label` | `Option<String>`  | no        | Label shown in the skeleton placeholder (URL variant only).                              |

Exactly one of `src` or `svg` must be set. Backward-compatibility note: existing
JSON sending `{"type":"Image","src":"…","alt":"…"}` continues to work unchanged.

> **Safety note — `svg` variant:** The `svg` value is emitted verbatim without
> HTML escaping. Intended for server-constructed SVG (charts, sparklines, icons).
> **Not suitable for user-supplied strings.** Callers that incorporate user data
> into the SVG output are responsible for sanitization before constructing the
> `svg` variant. The `alt` attribute is HTML-escaped on both variants.

**Rust**

```rust
use ferro_json_ui::{ComponentNode, ImageProps};

// URL variant
let url_node = ComponentNode::image(
    "hero",
    ImageProps::url("/img/hero.png", "Hero image"),
);

// SVG variant — server-constructed chart (e.g. from a Rust helper)
let svg = bar_chart_svg(&weekly_data, 800, 300);
let chart_node = ComponentNode::image(
    "revenue-chart",
    ImageProps::inline_svg(svg, "Incassi settimanali: 150€ lun, 320€ mar, …"),
);
```

**JSON**

URL variant:

```json
{
  "type": "Image",
  "src": "/img/hero.png",
  "alt": "Hero image",
  "aspect_ratio": "16/9"
}
```

SVG variant:

```json
{
  "type": "Image",
  "svg": "<svg viewBox=\"0 0 800 300\">…</svg>",
  "alt": "Incassi settimanali: 150€ lun, 320€ mar, …"
}
```

**Use cases for the SVG variant**

- Server-rendered charts (bar, line, sparkline)
- Diagrams assembled from typed data on the server
- Decorative vector assets constructed by Rust code
- Server-rendered icon sets

**No generic HTML escape hatch.** For rendering HTML (not SVG), no generic
`HtmlEmbed`-style component exists — by design. If a real use-case demands
HTML embedding, author a narrower component scoped to the specific content
shape rather than a generic string-to-HTML escape hatch.

### Text

Renders text content with semantic HTML element selection.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | `String` | Yes | - | Text content |
| `element` | `TextElement` | No | `P` | HTML element: `p`, `h1`, `h2`, `h3`, `span`, `div`, `section` |

```rust
use ferro::{ComponentNode, Component, TextProps, TextElement};

ComponentNode {
    key: "heading".to_string(),
    component: Component::Text(TextProps {
        content: "Welcome to the dashboard".to_string(),
        element: TextElement::H1,
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "heading", "type": "Text", "content": "Welcome to the dashboard", "element": "h1" }
```

### Progress

Progress bar with percentage value.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | `u8` | Yes | - | Percentage value (0-100) |
| `max` | `Option<u8>` | No | `None` | Maximum value |
| `label` | `Option<String>` | No | `None` | Label text above the bar |

```rust
use ferro::{ComponentNode, Component, ProgressProps};

ComponentNode {
    key: "upload-progress".to_string(),
    component: Component::Progress(ProgressProps {
        value: 75,
        max: Some(100),
        label: Some("Uploading...".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "upload-progress", "type": "Progress", "value": 75, "max": 100, "label": "Uploading..." }
```

### Breadcrumb

Navigation breadcrumb trail. The last item typically has no URL (current page).

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `items` | `Vec<BreadcrumbItem>` | Yes | - | Breadcrumb items |

**BreadcrumbItem** defines a breadcrumb entry:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `String` | Yes | Breadcrumb text |
| `url` | `Option<String>` | No | Link URL (omit for the current page) |

```rust
use ferro::{ComponentNode, Component, BreadcrumbProps, BreadcrumbItem};

ComponentNode {
    key: "breadcrumbs".to_string(),
    component: Component::Breadcrumb(BreadcrumbProps {
        items: vec![
            BreadcrumbItem { label: "Home".to_string(), url: Some("/".to_string()) },
            BreadcrumbItem { label: "Users".to_string(), url: Some("/users".to_string()) },
            BreadcrumbItem { label: "Edit User".to_string(), url: None },
        ],
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "breadcrumbs",
  "type": "Breadcrumb",
  "items": [
    { "label": "Home", "url": "/" },
    { "label": "Users", "url": "/users" },
    { "label": "Edit User" }
  ]
}
```

### Pagination

Page navigation for paginated data. Computes page count from `total` and `per_page`.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `current_page` | `u32` | Yes | - | Current page number |
| `per_page` | `u32` | Yes | - | Items per page |
| `total` | `u32` | Yes | - | Total number of items |
| `base_url` | `Option<String>` | No | `None` | Base URL for page links |

```rust
use ferro::{ComponentNode, Component, PaginationProps};

ComponentNode {
    key: "users-pagination".to_string(),
    component: Component::Pagination(PaginationProps {
        current_page: 1,
        per_page: 25,
        total: 150,
        base_url: Some("/users".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "users-pagination", "type": "Pagination", "current_page": 1, "per_page": 25, "total": 150, "base_url": "/users" }
```

### StatCard

Live-updatable metric card with an optional SSE target for real-time value updates. Used in dashboards to display KPIs, counts, and monetary totals.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `label` | `String` | Yes | - | Metric label (e.g., "Total Revenue") |
| `value` | `String` | Yes | - | Current metric value (e.g., "€12,345") |
| `icon` | `Option<String>` | No | `None` | Icon name |
| `subtitle` | `Option<String>` | No | `None` | Secondary text below the value |
| `sse_target` | `Option<String>` | No | `None` | SSE event target key for live updates |

The `sse_target` field connects this card to the JS runtime's SSE listener. When the server emits a Server-Sent Event with a matching key, the runtime updates the displayed value without a page reload:

```rust
use ferro::{ComponentNode, StatCardProps};

let node = ComponentNode::stat_card("revenue", StatCardProps {
    label: "Total Revenue".to_string(),
    value: "€12,345".to_string(),
    icon: Some("currency-euro".to_string()),
    subtitle: Some("This month".to_string()),
    sse_target: Some("revenue_total".to_string()),
});
```

The server sends updates via SSE as JSON:

```
event: live-value
data: {"target": "revenue_total", "value": "€13,210"}
```

JSON output:

```json
{
  "key": "revenue",
  "type": "StatCard",
  "label": "Total Revenue",
  "value": "€12,345",
  "icon": "currency-euro",
  "subtitle": "This month",
  "sse_target": "revenue_total"
}
```

---

## Forms Components

### Form

Form container with action binding and field components. The `action` defines the submit endpoint.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `action` | `Action` | Yes | - | Action to execute on form submit |
| `fields` | `Vec<ComponentNode>` | Yes | - | Form field components (Input, Select, Checkbox, etc.) |
| `method` | `Option<HttpMethod>` | No | `None` | HTTP method override (GET, POST, PUT, PATCH, DELETE) |

```rust
use ferro::{ComponentNode, Component, FormProps, InputProps, InputType, Action};

ComponentNode {
    key: "create-form".to_string(),
    component: Component::Form(FormProps {
        action: Action::new("users.store"),
        fields: vec![
            ComponentNode {
                key: "name-input".to_string(),
                component: Component::Input(InputProps {
                    field: "name".to_string(),
                    label: "Name".to_string(),
                    input_type: InputType::Text,
                    placeholder: Some("Enter name".to_string()),
                    required: Some(true),
                    disabled: None,
                    error: None,
                    description: None,
                    default_value: None,
                    data_path: None,
                    step: None,
                }),
                action: None,
                visibility: None,
            },
        ],
        method: None,
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "create-form",
  "type": "Form",
  "action": { "handler": "users.store", "method": "POST" },
  "fields": [
    { "key": "name-input", "type": "Input", "field": "name", "label": "Name", "placeholder": "Enter name", "required": true }
  ]
}
```

### Input

Text input field with type variants, validation error display, and data binding.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `field` | `String` | Yes | - | Form field name for data binding |
| `label` | `String` | Yes | - | Input label text |
| `input_type` | `InputType` | No | `Text` | Input type |
| `placeholder` | `Option<String>` | No | `None` | Placeholder text |
| `required` | `Option<bool>` | No | `None` | Whether the field is required |
| `disabled` | `Option<bool>` | No | `None` | Whether the field is disabled |
| `error` | `Option<String>` | No | `None` | Validation error message |
| `description` | `Option<String>` | No | `None` | Help text below the input |
| `default_value` | `Option<String>` | No | `None` | Pre-filled value |
| `data_path` | `Option<String>` | No | `None` | Data path for pre-filling from handler data (e.g., `"/data/user/name"`) |
| `step` | `Option<String>` | No | `None` | HTML step attribute for number inputs (e.g., `"any"`, `"0.01"`) |

**InputType** variants:

| Value | Serialized |
|-------|-----------|
| `InputType::Text` | `"text"` |
| `InputType::Email` | `"email"` |
| `InputType::Password` | `"password"` |
| `InputType::Number` | `"number"` |
| `InputType::Textarea` | `"textarea"` |
| `InputType::Hidden` | `"hidden"` |
| `InputType::Date` | `"date"` |
| `InputType::Time` | `"time"` |
| `InputType::Url` | `"url"` |
| `InputType::Tel` | `"tel"` |
| `InputType::Search` | `"search"` |

```rust
use ferro::{ComponentNode, Component, InputProps, InputType};

ComponentNode {
    key: "email-input".to_string(),
    component: Component::Input(InputProps {
        field: "email".to_string(),
        label: "Email Address".to_string(),
        input_type: InputType::Email,
        placeholder: Some("user@example.com".to_string()),
        required: Some(true),
        disabled: None,
        error: None,
        description: Some("Your work email".to_string()),
        default_value: None,
        data_path: Some("/data/user/email".to_string()),
        step: None,
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "email-input",
  "type": "Input",
  "field": "email",
  "label": "Email Address",
  "input_type": "email",
  "placeholder": "user@example.com",
  "required": true,
  "description": "Your work email",
  "data_path": "/data/user/email"
}
```

### Select

Dropdown select field with options, validation error, and data binding.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `field` | `String` | Yes | - | Form field name for data binding |
| `label` | `String` | Yes | - | Select label text |
| `options` | `Vec<SelectOption>` | Yes | - | Options list |
| `placeholder` | `Option<String>` | No | `None` | Placeholder text |
| `required` | `Option<bool>` | No | `None` | Whether the field is required |
| `disabled` | `Option<bool>` | No | `None` | Whether the field is disabled |
| `error` | `Option<String>` | No | `None` | Validation error message |
| `description` | `Option<String>` | No | `None` | Help text below the select |
| `default_value` | `Option<String>` | No | `None` | Pre-selected value |
| `data_path` | `Option<String>` | No | `None` | Data path for pre-filling from handler data |

**SelectOption** defines a value-label pair:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `value` | `String` | Yes | Option value submitted with the form |
| `label` | `String` | Yes | Display text shown to the user |

```rust
use ferro::{ComponentNode, Component, SelectProps, SelectOption};

ComponentNode {
    key: "role-select".to_string(),
    component: Component::Select(SelectProps {
        field: "role".to_string(),
        label: "Role".to_string(),
        options: vec![
            SelectOption { value: "admin".to_string(), label: "Administrator".to_string() },
            SelectOption { value: "editor".to_string(), label: "Editor".to_string() },
            SelectOption { value: "viewer".to_string(), label: "Viewer".to_string() },
        ],
        placeholder: Some("Select a role".to_string()),
        required: Some(true),
        disabled: None,
        error: None,
        description: None,
        default_value: None,
        data_path: Some("/data/user/role".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "role-select",
  "type": "Select",
  "field": "role",
  "label": "Role",
  "placeholder": "Select a role",
  "required": true,
  "data_path": "/data/user/role",
  "options": [
    { "value": "admin", "label": "Administrator" },
    { "value": "editor", "label": "Editor" },
    { "value": "viewer", "label": "Viewer" }
  ]
}
```

### Checkbox

Boolean checkbox field with label, description, and data binding.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `field` | `String` | Yes | - | Form field name for data binding |
| `label` | `String` | Yes | - | Checkbox label text |
| `description` | `Option<String>` | No | `None` | Help text below the checkbox |
| `checked` | `Option<bool>` | No | `None` | Default checked state |
| `data_path` | `Option<String>` | No | `None` | Data path for pre-filling from handler data |
| `required` | `Option<bool>` | No | `None` | Whether the field is required |
| `disabled` | `Option<bool>` | No | `None` | Whether the field is disabled |
| `error` | `Option<String>` | No | `None` | Validation error message |

```rust
use ferro::{ComponentNode, Component, CheckboxProps};

ComponentNode {
    key: "terms-checkbox".to_string(),
    component: Component::Checkbox(CheckboxProps {
        field: "terms".to_string(),
        label: "Accept Terms of Service".to_string(),
        description: Some("You must accept to continue.".to_string()),
        checked: None,
        data_path: None,
        required: Some(true),
        disabled: None,
        error: None,
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "terms-checkbox", "type": "Checkbox", "field": "terms", "label": "Accept Terms of Service", "description": "You must accept to continue.", "required": true }
```

### Switch

Toggle switch — a visual alternative to Checkbox with identical props. The frontend renderer handles the visual difference.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `field` | `String` | Yes | - | Form field name for data binding |
| `label` | `String` | Yes | - | Switch label text |
| `description` | `Option<String>` | No | `None` | Help text below the switch |
| `checked` | `Option<bool>` | No | `None` | Default checked state |
| `data_path` | `Option<String>` | No | `None` | Data path for pre-filling from handler data |
| `required` | `Option<bool>` | No | `None` | Whether the field is required |
| `disabled` | `Option<bool>` | No | `None` | Whether the field is disabled |
| `error` | `Option<String>` | No | `None` | Validation error message |

```rust
use ferro::{ComponentNode, Component, SwitchProps};

ComponentNode {
    key: "notifications-switch".to_string(),
    component: Component::Switch(SwitchProps {
        field: "notifications".to_string(),
        label: "Enable Notifications".to_string(),
        description: Some("Receive email notifications".to_string()),
        checked: Some(true),
        data_path: Some("/data/user/notifications_enabled".to_string()),
        required: None,
        disabled: None,
        error: None,
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "notifications-switch", "type": "Switch", "field": "notifications", "label": "Enable Notifications", "description": "Receive email notifications", "checked": true, "data_path": "/data/user/notifications_enabled" }
```

### Button

Interactive button with visual variants, sizing, and optional icon.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `label` | `String` | Yes | - | Button label text |
| `variant` | `ButtonVariant` | No | `Default` | Visual style |
| `size` | `Size` | No | `Default` | Button size |
| `disabled` | `Option<bool>` | No | `None` | Whether the button is disabled |
| `icon` | `Option<String>` | No | `None` | Icon name |
| `icon_position` | `Option<IconPosition>` | No | `Left` | Icon placement: `left` or `right` |

Buttons are typically combined with an `action` on the `ComponentNode` to bind click behavior:

```rust
use ferro::{ComponentNode, ButtonProps, ButtonVariant, Size, IconPosition};

let node = ComponentNode::button("save-btn", ButtonProps {
    label: "Save Changes".to_string(),
    variant: ButtonVariant::Default,
    size: Size::Default,
    disabled: None,
    icon: Some("save".to_string()),
    icon_position: Some(IconPosition::Left),
});
```

JSON output:

```json
{ "key": "save-btn", "type": "Button", "label": "Save Changes", "variant": "default", "size": "default", "icon": "save", "icon_position": "left" }
```

---

## Feedback Components

### Alert

Alert message with variant-based styling and optional title.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `message` | `String` | Yes | - | Alert message content |
| `variant` | `AlertVariant` | No | `Info` | Visual style |
| `title` | `Option<String>` | No | `None` | Alert title |

```rust
use ferro::{ComponentNode, Component, AlertProps, AlertVariant};

ComponentNode {
    key: "warning".to_string(),
    component: Component::Alert(AlertProps {
        message: "Your trial expires in 3 days.".to_string(),
        variant: AlertVariant::Warning,
        title: Some("Trial Ending".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "warning", "type": "Alert", "message": "Your trial expires in 3 days.", "variant": "warning", "title": "Trial Ending" }
```

### Toast

Declarative notification intent rendered by the JS runtime. When a Toast component is included in a view, the runtime displays it as an overlay notification (top-right corner) and dismisses it after the configured timeout.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `message` | `String` | Yes | - | Toast message content |
| `variant` | `ToastVariant` | No | `Info` | Visual style (info, success, warning, error) |
| `timeout` | `Option<u32>` | No | `None` | Seconds before auto-dismiss (default: 5) |
| `dismissible` | `bool` | No | `true` | Whether the user can dismiss the toast |

```rust
use ferro::{ComponentNode, Component, ToastProps, ToastVariant};

ComponentNode {
    key: "save-toast".to_string(),
    component: Component::Toast(ToastProps {
        message: "Changes saved successfully.".to_string(),
        variant: ToastVariant::Success,
        timeout: Some(3),
        dismissible: true,
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{ "key": "save-toast", "type": "Toast", "message": "Changes saved successfully.", "variant": "success", "timeout": 3, "dismissible": true }
```

---

## Navigation Components

### Sidebar

Sidebar navigation shell with fixed top items, grouped items, and fixed bottom items. Typically used as a component inside a DashboardLayout. When used standalone it renders as a vertical navigation panel.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `fixed_top` | `Vec<SidebarNavItem>` | No | `[]` | Items pinned at the top (e.g., logo/home link) |
| `groups` | `Vec<SidebarGroup>` | No | `[]` | Collapsible navigation groups |
| `fixed_bottom` | `Vec<SidebarNavItem>` | No | `[]` | Items pinned at the bottom (e.g., settings, logout) |

**SidebarNavItem** defines a navigation link:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `String` | Yes | Link text |
| `href` | `String` | Yes | Link URL |
| `icon` | `Option<String>` | No | Icon name |
| `active` | `bool` | No (default: false) | Whether this is the current page |

**SidebarGroup** defines a labeled, collapsible group:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `String` | Yes | Group heading text |
| `collapsed` | `bool` | No (default: false) | Whether the group starts collapsed |
| `items` | `Vec<SidebarNavItem>` | Yes | Navigation links in this group |

```rust
use ferro::{ComponentNode, Component, SidebarProps, SidebarNavItem, SidebarGroup};

ComponentNode {
    key: "sidebar".to_string(),
    component: Component::Sidebar(SidebarProps {
        fixed_top: vec![
            SidebarNavItem { label: "Dashboard".to_string(), href: "/".to_string(), icon: Some("home".to_string()), active: true },
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
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "sidebar",
  "type": "Sidebar",
  "fixed_top": [{ "label": "Dashboard", "href": "/", "icon": "home", "active": true }],
  "groups": [
    {
      "label": "Management",
      "collapsed": false,
      "items": [
        { "label": "Users", "href": "/users", "icon": "users", "active": false },
        { "label": "Orders", "href": "/orders", "icon": "shopping-bag", "active": false }
      ]
    }
  ],
  "fixed_bottom": [{ "label": "Settings", "href": "/settings", "icon": "cog", "active": false }]
}
```

### Header

Application header shell with business name, user info, notification count, and logout link. Typically used inside DashboardLayout.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `business_name` | `String` | Yes | - | Application name shown in the header |
| `notification_count` | `Option<u32>` | No | `None` | Unread notification count for badge display |
| `user_name` | `Option<String>` | No | `None` | Current user's name |
| `user_avatar` | `Option<String>` | No | `None` | Current user's avatar URL |
| `logout_url` | `Option<String>` | No | `None` | URL for the logout link |

```rust
use ferro::{ComponentNode, Component, HeaderProps};

ComponentNode {
    key: "header".to_string(),
    component: Component::Header(HeaderProps {
        business_name: "My App".to_string(),
        notification_count: Some(3),
        user_name: Some("Alice Johnson".to_string()),
        user_avatar: None,
        logout_url: Some("/logout".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "header",
  "type": "Header",
  "business_name": "My App",
  "notification_count": 3,
  "user_name": "Alice Johnson",
  "logout_url": "/logout"
}
```

### NotificationDropdown

A dropdown list of notification items, typically rendered inside a Header. Displays a list of recent notifications with read/unread state, timestamps, and optional action URLs.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `notifications` | `Vec<NotificationItem>` | Yes | - | List of notifications |
| `empty_text` | `Option<String>` | No | `None` | Text to show when the list is empty |

**NotificationItem** defines a single notification:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text` | `String` | Yes | Notification message |
| `icon` | `Option<String>` | No | Icon name |
| `timestamp` | `Option<String>` | No | Time string (e.g., "2 minutes ago") |
| `read` | `bool` | No (default: false) | Whether the notification has been read |
| `action_url` | `Option<String>` | No | URL to navigate to on click |

```rust
use ferro::{ComponentNode, Component, NotificationDropdownProps, NotificationItem};

ComponentNode {
    key: "notifications".to_string(),
    component: Component::NotificationDropdown(NotificationDropdownProps {
        notifications: vec![
            NotificationItem {
                icon: Some("bell".to_string()),
                text: "New order received".to_string(),
                timestamp: Some("5 minutes ago".to_string()),
                read: false,
                action_url: Some("/orders/123".to_string()),
            },
            NotificationItem {
                icon: None,
                text: "Payment processed".to_string(),
                timestamp: Some("1 hour ago".to_string()),
                read: true,
                action_url: None,
            },
        ],
        empty_text: Some("No new notifications".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "notifications",
  "type": "NotificationDropdown",
  "empty_text": "No new notifications",
  "notifications": [
    { "icon": "bell", "text": "New order received", "timestamp": "5 minutes ago", "read": false, "action_url": "/orders/123" },
    { "text": "Payment processed", "timestamp": "1 hour ago", "read": true }
  ]
}
```

---

## Onboarding Components

### Checklist

Step-by-step onboarding or task checklist with dismissal and optional server-side state persistence.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | `String` | Yes | - | Checklist title |
| `items` | `Vec<ChecklistItem>` | Yes | - | Checklist items |
| `dismissible` | `bool` | No | `true` | Whether the checklist can be dismissed |
| `dismiss_label` | `Option<String>` | No | `None` | Custom dismiss button label |
| `data_key` | `Option<String>` | No | `None` | Server-side state persistence key |

**ChecklistItem** defines a checklist step:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `String` | Yes | Step description |
| `checked` | `bool` | No (default: false) | Whether this step is complete |
| `href` | `Option<String>` | No | Link to take the step |

```rust
use ferro::{ComponentNode, Component, ChecklistProps, ChecklistItem};

ComponentNode {
    key: "setup-checklist".to_string(),
    component: Component::Checklist(ChecklistProps {
        title: "Get Started".to_string(),
        items: vec![
            ChecklistItem { label: "Create your account".to_string(), checked: true, href: None },
            ChecklistItem { label: "Set up billing".to_string(), checked: false, href: Some("/billing".to_string()) },
            ChecklistItem { label: "Invite your team".to_string(), checked: false, href: Some("/team/invite".to_string()) },
        ],
        dismissible: true,
        dismiss_label: Some("Done".to_string()),
        data_key: Some("onboarding_checklist".to_string()),
    }),
    action: None,
    visibility: None,
}
```

JSON output:

```json
{
  "key": "setup-checklist",
  "type": "Checklist",
  "title": "Get Started",
  "dismissible": true,
  "dismiss_label": "Done",
  "data_key": "onboarding_checklist",
  "items": [
    { "label": "Create your account", "checked": true },
    { "label": "Set up billing", "checked": false, "href": "/billing" },
    { "label": "Invite your team", "checked": false, "href": "/team/invite" }
  ]
}
```

---

## Extensible Components

### Plugin

Passes through to a registered plugin component. The `plugin_type` field selects the plugin from the global registry.

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `plugin_type` | `String` | Yes | Registered plugin type name (e.g., `"Map"`) |
| `props` | `serde_json::Value` | Yes | Raw props passed to the plugin's render function |

```rust
use ferro::{ComponentNode, Component, PluginProps};

ComponentNode {
    key: "office-map".to_string(),
    component: Component::Plugin(PluginProps {
        plugin_type: "Map".to_string(),
        props: serde_json::json!({
            "center": [51.505, -0.09],
            "zoom": 13,
            "markers": [
                { "lat": 51.505, "lng": -0.09, "popup": "Our office" }
            ]
        }),
    }),
    action: None,
    visibility: None,
}
```

See **[Plugins](plugins.md)** for the full plugin guide — how to build, register, and use custom plugin components.

JSON output:

```json
{
  "key": "office-map",
  "type": "Map",
  "center": [51.505, -0.09],
  "zoom": 13,
  "markers": [{ "lat": 51.505, "lng": -0.09, "popup": "Our office" }]
}
```

---

## JSON Serialization

Every component tree serializes to JSON via serde. The `Component` enum uses serde's tagged representation, producing a `"type"` field that identifies the component:

```json
{
  "key": "welcome-card",
  "type": "Card",
  "title": "Welcome",
  "description": "Your dashboard",
  "children": [
    {
      "key": "greeting",
      "type": "Text",
      "content": "Hello, Alice!",
      "element": "h2"
    }
  ]
}
```

The `ComponentNode` fields (`key`, `action`, `visibility`) are flattened into the same JSON object alongside the component-specific props. This produces clean, predictable JSON that frontend renderers can consume directly.

Optional fields with `None` values are omitted from serialization (`skip_serializing_if = "Option::is_none"`). Default values for enums (e.g., `ButtonVariant::Default`) are always included.
