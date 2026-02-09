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

---

## Form Components

### Form

Form container with action binding and field components. The `action` defines the submit endpoint.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `action` | `Action` | Yes | - | Action to execute on form submit |
| `fields` | `Vec<ComponentNode>` | Yes | - | Form field components (Input, Select, Checkbox, etc.) |
| `method` | `Option<HttpMethod>` | No | `None` | HTTP method override (GET, POST, PUT, PATCH, DELETE) |

```rust
use ferro::*;

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
                }),
                action: None,
                visibility: None,
            },
            ComponentNode {
                key: "email-input".to_string(),
                component: Component::Input(InputProps {
                    field: "email".to_string(),
                    label: "Email".to_string(),
                    input_type: InputType::Email,
                    placeholder: Some("user@example.com".to_string()),
                    required: Some(true),
                    disabled: None,
                    error: None,
                    description: None,
                    default_value: None,
                    data_path: None,
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
use ferro::*;

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
    }),
    action: None,
    visibility: None,
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
use ferro::*;

ComponentNode {
    key: "role-select".to_string(),
    component: Component::Select(SelectProps {
        field: "role".to_string(),
        label: "Role".to_string(),
        options: vec![
            SelectOption {
                value: "admin".to_string(),
                label: "Administrator".to_string(),
            },
            SelectOption {
                value: "editor".to_string(),
                label: "Editor".to_string(),
            },
            SelectOption {
                value: "viewer".to_string(),
                label: "Viewer".to_string(),
            },
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
use ferro::*;

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

### Switch

Toggle switch -- a visual alternative to Checkbox with identical props. The frontend renderer handles the visual difference.

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
use ferro::*;

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

---

## Navigation Components

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
use ferro::*;

ComponentNode {
    key: "settings-tabs".to_string(),
    component: Component::Tabs(TabsProps {
        default_tab: "general".to_string(),
        tabs: vec![
            Tab {
                value: "general".to_string(),
                label: "General".to_string(),
                children: vec![
                    ComponentNode {
                        key: "name-input".to_string(),
                        component: Component::Input(InputProps {
                            field: "name".to_string(),
                            label: "Name".to_string(),
                            input_type: InputType::Text,
                            placeholder: None,
                            required: None,
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                ],
            },
            Tab {
                value: "security".to_string(),
                label: "Security".to_string(),
                children: vec![
                    ComponentNode {
                        key: "password-input".to_string(),
                        component: Component::Input(InputProps {
                            field: "password".to_string(),
                            label: "Password".to_string(),
                            input_type: InputType::Password,
                            placeholder: None,
                            required: None,
                            disabled: None,
                            error: None,
                            description: None,
                            default_value: None,
                            data_path: None,
                        }),
                        action: None,
                        visibility: None,
                    },
                ],
            },
        ],
    }),
    action: None,
    visibility: None,
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
use ferro::*;

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
use ferro::*;

ComponentNode {
    key: "breadcrumbs".to_string(),
    component: Component::Breadcrumb(BreadcrumbProps {
        items: vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/".to_string()),
            },
            BreadcrumbItem {
                label: "Users".to_string(),
                url: Some("/users".to_string()),
            },
            BreadcrumbItem {
                label: "Edit User".to_string(),
                url: None,
            },
        ],
    }),
    action: None,
    visibility: None,
}
```

---

## Feedback Components

### Progress

Progress bar with percentage value.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `value` | `u8` | Yes | - | Percentage value (0-100) |
| `max` | `Option<u8>` | No | `None` | Maximum value |
| `label` | `Option<String>` | No | `None` | Label text above the bar |

```rust
use ferro::*;

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

### Avatar

User avatar with image source, fallback text, and size variants.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `src` | `Option<String>` | No | `None` | Image URL |
| `alt` | `String` | Yes | - | Alt text (required for accessibility) |
| `fallback` | `Option<String>` | No | `None` | Fallback initials when no image |
| `size` | `Option<Size>` | No | `Default` | Avatar size: `xs`, `sm`, `default`, `lg` |

```rust
use ferro::*;

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

### Skeleton

Loading placeholder with configurable dimensions for content that is still loading.

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `width` | `Option<String>` | No | `None` | CSS width (e.g., `"100%"`, `"200px"`) |
| `height` | `Option<String>` | No | `None` | CSS height (e.g., `"40px"`) |
| `rounded` | `Option<bool>` | No | `None` | Whether to use rounded corners |

```rust
use ferro::*;

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

---

## Layout Components

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
use ferro::*;

ComponentNode {
    key: "delete-modal".to_string(),
    component: Component::Modal(ModalProps {
        title: "Delete Item".to_string(),
        description: Some("This action cannot be undone.".to_string()),
        children: vec![
            ComponentNode {
                key: "confirm-text".to_string(),
                component: Component::Text(TextProps {
                    content: "Are you sure you want to delete this item?".to_string(),
                    element: TextElement::P,
                }),
                action: None,
                visibility: None,
            },
        ],
        footer: vec![
            ComponentNode {
                key: "cancel-btn".to_string(),
                component: Component::Button(ButtonProps {
                    label: "Cancel".to_string(),
                    variant: ButtonVariant::Outline,
                    size: Size::Default,
                    disabled: None,
                    icon: None,
                    icon_position: None,
                }),
                action: None,
                visibility: None,
            },
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
                action: Some(Action::delete("items.destroy")
                    .confirm_danger("Confirm deletion")),
                visibility: None,
            },
        ],
        trigger_label: Some("Delete".to_string()),
    }),
    action: None,
    visibility: None,
}
```

---

## JSON Output

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
  ],
  "footer": []
}
```

The `ComponentNode` fields (`key`, `action`, `visibility`) are flattened into the same JSON object alongside the component-specific props. This produces clean, predictable JSON that frontend renderers can consume directly.
