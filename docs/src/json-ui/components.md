# Components

JSON-UI includes 20 built-in components for building complete application interfaces from Rust handlers.

## Component Overview

| Category | Components |
|----------|-----------|
| **Display** | Card, Table, Badge, Alert, Separator, DescriptionList, Text, Button |
| **Form** | Form, Input, Select, Checkbox, Switch |
| **Navigation** | Tabs, Breadcrumb, Pagination |
| **Feedback** | Progress, Avatar, Skeleton |
| **Layout** | Modal |

Every component is wrapped in a `ComponentNode` that provides a unique `key`, an optional `action` binding, and optional `visibility` rules:

```rust
use ferro::*;

ComponentNode {
    key: "my-card".to_string(),
    component: Component::Card(CardProps { /* ... */ }),
    action: None,
    visibility: None,
}
```

## Shared Types

These enums are used across multiple components.

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

Visual styles for the Alert component.

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

---

## Display Components

### Card

Container with title, optional description, nested children, and footer.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | `String` | Yes | - | Card title |
| `description` | `Option<String>` | No | `None` | Description below the title |
| `children` | `Vec<ComponentNode>` | No | `[]` | Nested components in the card body |
| `footer` | `Vec<ComponentNode>` | No | `[]` | Components in the card footer |

```rust
use ferro::*;

ComponentNode {
    key: "user-card".to_string(),
    component: Component::Card(CardProps {
        title: "User Details".to_string(),
        description: Some("Account information".to_string()),
        children: vec![
            ComponentNode {
                key: "name".to_string(),
                component: Component::Text(TextProps {
                    content: "Alice Johnson".to_string(),
                    element: TextElement::H3,
                }),
                action: None,
                visibility: None,
            },
        ],
        footer: vec![
            ComponentNode {
                key: "edit-btn".to_string(),
                component: Component::Button(ButtonProps {
                    label: "Edit".to_string(),
                    variant: ButtonVariant::Outline,
                    size: Size::Default,
                    disabled: None,
                    icon: None,
                    icon_position: None,
                }),
                action: Some(Action::get("users.edit")),
                visibility: None,
            },
        ],
    }),
    action: None,
    visibility: None,
}
```

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
use ferro::*;

ComponentNode {
    key: "users-table".to_string(),
    component: Component::Table(TableProps {
        columns: vec![
            Column {
                key: "name".to_string(),
                label: "Name".to_string(),
                format: None,
            },
            Column {
                key: "email".to_string(),
                label: "Email".to_string(),
                format: None,
            },
            Column {
                key: "created_at".to_string(),
                label: "Created".to_string(),
                format: Some(ColumnFormat::Date),
            },
        ],
        data_path: "/data/users".to_string(),
        row_actions: Some(vec![
            Action::get("users.edit"),
            Action::delete("users.destroy")
                .confirm_danger("Delete this user?"),
        ]),
        empty_message: Some("No users found.".to_string()),
        sortable: Some(true),
        sort_column: None,
        sort_direction: None,
    }),
    action: None,
    visibility: None,
}
```

The `data_path` points to an array in the handler data. Each object in the array maps its keys to column `key` fields.

### Badge

Small label with variant-based styling.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `label` | `String` | Yes | - | Badge text |
| `variant` | `BadgeVariant` | No | `Default` | Visual style |

```rust
use ferro::*;

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

### Alert

Alert message with variant-based styling and optional title.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `message` | `String` | Yes | - | Alert message content |
| `variant` | `AlertVariant` | No | `Info` | Visual style |
| `title` | `Option<String>` | No | `None` | Alert title |

```rust
use ferro::*;

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

### Separator

Visual divider between content sections.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `orientation` | `Option<Orientation>` | No | `Horizontal` | Direction: `horizontal` or `vertical` |

```rust
use ferro::*;

ComponentNode {
    key: "divider".to_string(),
    component: Component::Separator(SeparatorProps {
        orientation: None, // defaults to horizontal
    }),
    action: None,
    visibility: None,
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
use ferro::*;

ComponentNode {
    key: "user-info".to_string(),
    component: Component::DescriptionList(DescriptionListProps {
        items: vec![
            DescriptionItem {
                label: "Name".to_string(),
                value: "Alice Johnson".to_string(),
                format: None,
            },
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

### Text

Renders text content with semantic HTML element selection.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `content` | `String` | Yes | - | Text content |
| `element` | `TextElement` | No | `P` | HTML element: `p`, `h1`, `h2`, `h3`, `span` |

```rust
use ferro::*;

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
use ferro::*;

ComponentNode {
    key: "save-btn".to_string(),
    component: Component::Button(ButtonProps {
        label: "Save Changes".to_string(),
        variant: ButtonVariant::Default,
        size: Size::Default,
        disabled: None,
        icon: Some("save".to_string()),
        icon_position: Some(IconPosition::Left),
    }),
    action: Some(Action::new("users.update")),
    visibility: None,
}
```
