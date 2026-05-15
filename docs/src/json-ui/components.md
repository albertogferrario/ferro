# Components

Every component in a v2 JSON-UI spec is referenced by its `"type"` string in a flat element map. For the full spec format and workflow, see [Getting Started](getting-started.md).

Each element follows this shape:

```json
"element_id": {
  "type": "ComponentTypeName",
  "props": {
    "prop_name": "prop_value"
  },
  "children": ["child_id"],
  "action": { "handler": "route.name", "method": "POST" },
  "visible": { "field": "/data/status", "op": "eq", "value": "active" }
}
```

The sections below document every built-in component: its props table (with JSON types) and a complete element example.

---

## Component Overview

| Category | Components |
|----------|------------|
| **Layout** | Card, Grid, Tabs, Separator, Modal, Skeleton, Collapsible, FormSection |
| **Data Display** | Text, DataTable, Table, DescriptionList, Badge, Avatar, Progress, Breadcrumb, Pagination, StatCard, Image |
| **Forms** | Form, Input, Select, Checkbox, Switch, Button, ButtonGroup, DropdownMenu |
| **Feedback** | Alert, Toast, EmptyState |
| **Navigation** | Sidebar, Header, PageHeader, NotificationDropdown |
| **Action** | ActionCard |
| **Onboarding** | Checklist |
| **Commerce** | ProductTile |
| **Extensible** | Plugin (see [Plugins](plugins.md)) |

---

## Shared Enum Values

Several props accept fixed-string enum values. The valid strings are listed here; each component section references these by name.

**size** — `"xs"` | `"sm"` | `"default"` | `"lg"`

**button_variant** — `"default"` | `"secondary"` | `"destructive"` | `"outline"` | `"ghost"` | `"link"`

**alert_variant** — `"info"` | `"success"` | `"warning"` | `"error"`

**badge_variant** — `"default"` | `"secondary"` | `"destructive"` | `"outline"`

**column_format** — `"date"` | `"date_time"` | `"currency"` | `"boolean"`

**text_element** — `"p"` | `"h1"` | `"h2"` | `"h3"` | `"span"` | `"div"` | `"section"`

**toast_variant** — `"info"` | `"success"` | `"warning"` | `"error"`

**input_type** — `"text"` | `"email"` | `"password"` | `"number"` | `"textarea"` | `"hidden"` | `"date"` | `"time"` | `"url"` | `"tel"` | `"search"`

**orientation** — `"horizontal"` | `"vertical"`

**icon_position** — `"left"` | `"right"`

**sort_direction** — `"asc"` | `"desc"`

**form_max_width** — `"sm"` | `"md"` | `"lg"` | `"xl"` | `"full"`

**gap_size** — `"none"` | `"xs"` | `"sm"` | `"md"` | `"lg"` | `"xl"`

**action_card_variant** — `"default"` | `"outline"` | `"ghost"`

---

## Layout Components

### Card

Container with title, optional description, nested children, and footer.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Card heading |
| `description` | `string \| null` | Secondary text below the title |

Children are element IDs listed in the `"children"` array on the element, not in props.

```json
"user_card": {
  "type": "Card",
  "props": {
    "title": "User Details",
    "description": "Account information"
  },
  "children": ["name_text", "email_text"]
}
```

### Grid

Responsive grid layout for arranging child elements in columns.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `number \| null` | Number of columns (default: 2) |
| `gap` | `gap_size \| null` | Gap between items: `"none"`, `"xs"`, `"sm"`, `"md"`, `"lg"`, `"xl"` |

```json
"stats_grid": {
  "type": "Grid",
  "props": {
    "columns": 3,
    "gap": "md"
  },
  "children": ["revenue_stat", "orders_stat", "users_stat"]
}
```

### Tabs

Tabbed content with multiple panels.

| Prop | Type | Description |
|------|------|-------------|
| `default_tab` | `string` | Value of the initially active tab |
| `tabs` | `array` | Tab definitions |

Each object in `tabs`:

| Field | Type | Description |
|-------|------|-------------|
| `value` | `string` | Tab identifier (matches `default_tab`) |
| `label` | `string` | Tab label text |
| `children` | `array of strings` | Element IDs shown when the tab is active |

```json
"settings_tabs": {
  "type": "Tabs",
  "props": {
    "default_tab": "general",
    "tabs": [
      { "value": "general", "label": "General", "children": ["general_form"] },
      { "value": "security", "label": "Security", "children": ["security_form"] }
    ]
  }
}
```

### Separator

Visual divider between content sections.

| Prop | Type | Description |
|------|------|-------------|
| `orientation` | `orientation \| null` | `"horizontal"` (default) or `"vertical"` |

```json
"divider": {
  "type": "Separator",
  "props": {}
}
```

### Modal

Dialog overlay with title, body children, footer children, and a trigger button label.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Modal heading |
| `description` | `string \| null` | Modal description text |
| `trigger_label` | `string \| null` | Label for the button that opens the modal |

Children of the modal body go in the element `"children"` array. Footer children use a `"footer_children"` prop listing element IDs.

```json
"delete_modal": {
  "type": "Modal",
  "props": {
    "title": "Delete Item",
    "description": "This action cannot be undone.",
    "trigger_label": "Delete"
  },
  "children": ["confirm_text"],
  "action": { "handler": "items.destroy", "method": "DELETE" }
}
```

### Skeleton

Loading placeholder with configurable dimensions.

| Prop | Type | Description |
|------|------|-------------|
| `width` | `string \| null` | CSS width (e.g., `"100%"`, `"200px"`) |
| `height` | `string \| null` | CSS height (e.g., `"40px"`) |
| `rounded` | `boolean \| null` | Use rounded corners |

```json
"loading_placeholder": {
  "type": "Skeleton",
  "props": {
    "width": "100%",
    "height": "40px",
    "rounded": true
  }
}
```

### Collapsible

An expandable/collapsible section with a trigger label.

| Prop | Type | Description |
|------|------|-------------|
| `trigger` | `string` | Label for the toggle |
| `open` | `boolean \| null` | Initially open when `true` |

```json
"advanced_section": {
  "type": "Collapsible",
  "props": {
    "trigger": "Advanced Options",
    "open": false
  },
  "children": ["timeout_input", "retry_input"]
}
```

### FormSection

Groups form fields under a section heading with an optional description.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Section heading |
| `description` | `string \| null` | Section description |

```json
"billing_section": {
  "type": "FormSection",
  "props": {
    "title": "Billing Information",
    "description": "Used for invoice generation."
  },
  "children": ["address_input", "city_input", "postal_input"]
}
```

---

## Data Display Components

### Text

Renders text content with a semantic HTML element.

| Prop | Type | Description |
|------|------|-------------|
| `content` | `string` | Text content |
| `element` | `text_element \| null` | HTML element: `"p"` (default), `"h1"`, `"h2"`, `"h3"`, `"span"`, `"div"`, `"section"` |

```json
"page_heading": {
  "type": "Text",
  "props": {
    "content": "Welcome to the dashboard",
    "element": "h1"
  }
}
```

Content can use a `$template` expression to interpolate data:

```json
"greeting": {
  "type": "Text",
  "props": {
    "content": { "$template": "Welcome, {/user/name}!" },
    "element": "h2"
  }
}
```

### DataTable

Data-bound table with column definitions, row actions, and sorting. Rows are loaded from the spec's data via `data_path`.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `array` | Column definitions (see below) |
| `data_path` | `string` | JSON Pointer to the row data array (e.g., `"/orders"`) |
| `row_actions` | `array \| null` | Actions available per row |
| `empty_message` | `string \| null` | Message when no data is present |
| `sortable` | `boolean \| null` | Enable column sorting |
| `sort_column` | `string \| null` | Currently sorted column key |
| `sort_direction` | `sort_direction \| null` | `"asc"` or `"desc"` |

Each column object:

| Field | Type | Description |
|-------|------|-------------|
| `key` | `string` | Data field key in the row object |
| `label` | `string` | Column header text |
| `format` | `column_format \| null` | Display format |

```json
"users_table": {
  "type": "DataTable",
  "props": {
    "data_path": "/users",
    "columns": [
      { "key": "name", "label": "Name" },
      { "key": "email", "label": "Email" },
      { "key": "created_at", "label": "Created", "format": "date" }
    ],
    "row_actions": [
      { "handler": "users.edit", "method": "GET" },
      { "handler": "users.destroy", "method": "DELETE", "confirm": { "message": "Delete this user?" } }
    ],
    "empty_message": "No users found.",
    "sortable": true
  }
}
```

### Table

Simple table without a data binding path. Use `DataTable` for data-bound tables; use `Table` for static content.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `array` | Column definitions (same structure as DataTable) |
| `rows` | `array` | Static row objects (key-value maps) |

```json
"static_table": {
  "type": "Table",
  "props": {
    "columns": [
      { "key": "plan", "label": "Plan" },
      { "key": "price", "label": "Price", "format": "currency" }
    ],
    "rows": [
      { "plan": "Starter", "price": "9.00" },
      { "plan": "Pro", "price": "29.00" }
    ]
  }
}
```

### DescriptionList

Key-value pairs displayed as a description list.

| Prop | Type | Description |
|------|------|-------------|
| `items` | `array` | Description items (see below) |
| `columns` | `number \| null` | Number of columns for layout |

Each item object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Item label |
| `value` | `string` | Item value |
| `format` | `column_format \| null` | Display format |

```json
"user_info": {
  "type": "DescriptionList",
  "props": {
    "columns": 2,
    "items": [
      { "label": "Name", "value": { "$data": "/user/name" } },
      { "label": "Joined", "value": { "$data": "/user/created_at" }, "format": "date" },
      { "label": "Active", "value": { "$data": "/user/active" }, "format": "boolean" }
    ]
  }
}
```

### Badge

Small label with variant-based styling.

| Prop | Type | Description |
|------|------|-------------|
| `label` | `string` | Badge text |
| `variant` | `badge_variant \| null` | Visual style (default: `"default"`) |

```json
"status_badge": {
  "type": "Badge",
  "props": {
    "label": "Active",
    "variant": "default"
  }
}
```

### Avatar

User avatar with image, fallback initials, and size.

| Prop | Type | Description |
|------|------|-------------|
| `alt` | `string` | Alt text (required for accessibility) |
| `src` | `string \| null` | Image URL |
| `fallback` | `string \| null` | Fallback initials when no image |
| `size` | `size \| null` | `"xs"`, `"sm"`, `"default"`, `"lg"` |

```json
"user_avatar": {
  "type": "Avatar",
  "props": {
    "alt": "Alice Johnson",
    "src": { "$data": "/user/avatar_url" },
    "fallback": "AJ",
    "size": "lg"
  }
}
```

### Progress

Progress bar with a percentage value.

| Prop | Type | Description |
|------|------|-------------|
| `value` | `number` | Percentage value (0-100) |
| `max` | `number \| null` | Maximum value |
| `label` | `string \| null` | Label text above the bar |

```json
"upload_progress": {
  "type": "Progress",
  "props": {
    "value": 75,
    "max": 100,
    "label": "Uploading..."
  }
}
```

### Breadcrumb

Navigation breadcrumb trail.

| Prop | Type | Description |
|------|------|-------------|
| `items` | `array` | Breadcrumb items (see below) |

Each item object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Breadcrumb text |
| `url` | `string \| null` | Link URL (omit for the current page) |

```json
"breadcrumbs": {
  "type": "Breadcrumb",
  "props": {
    "items": [
      { "label": "Home", "url": "/" },
      { "label": "Users", "url": "/users" },
      { "label": "Edit User" }
    ]
  }
}
```

### Pagination

Page navigation for paginated data.

| Prop | Type | Description |
|------|------|-------------|
| `current_page` | `number` | Current page number |
| `per_page` | `number` | Items per page |
| `total` | `number` | Total item count |
| `base_url` | `string \| null` | Base URL for page links |

```json
"users_pagination": {
  "type": "Pagination",
  "props": {
    "current_page": { "$data": "/meta/page" },
    "per_page": 25,
    "total": { "$data": "/meta/total" },
    "base_url": "/users"
  }
}
```

### StatCard

Metric card for dashboards. Displays a label and value, with an optional SSE target for live updates.

| Prop | Type | Description |
|------|------|-------------|
| `label` | `string` | Metric label (e.g., `"Total Revenue"`) |
| `value` | `string` | Current metric value (e.g., `"€12,345"`) |
| `icon` | `string \| null` | Icon name |
| `subtitle` | `string \| null` | Secondary text below the value |
| `sse_target` | `string \| null` | SSE event key for live value updates |

```json
"revenue_stat": {
  "type": "StatCard",
  "props": {
    "label": "Total Revenue",
    "value": { "$data": "/stats/revenue_formatted" },
    "icon": "currency-euro",
    "subtitle": "This month",
    "sse_target": "revenue_total"
  }
}
```

When `sse_target` is set and the server emits a Server-Sent Event with a matching key, the runtime updates the displayed value in place:

```
event: live-value
data: {"target": "revenue_total", "value": "€13,210"}
```

### Image

Renders an `<img>` element.

| Prop | Type | Description |
|------|------|-------------|
| `src` | `string` | Image URL |
| `alt` | `string` | Alt text |
| `width` | `number \| null` | CSS width in pixels |
| `height` | `number \| null` | CSS height in pixels |
| `class` | `string \| null` | Additional CSS classes |

```json
"hero_image": {
  "type": "Image",
  "props": {
    "src": "/images/hero.jpg",
    "alt": "Dashboard hero",
    "width": 1200,
    "height": 400
  }
}
```

---

## Form Components

### Form

Form container with an action binding. Field components go in the element `"children"` array.

| Prop | Type | Description |
|------|------|-------------|
| `method` | `string \| null` | HTTP method override (`"GET"`, `"POST"`, `"PUT"`, `"PATCH"`, `"DELETE"`) |
| `max_width` | `form_max_width \| null` | Max form width: `"sm"`, `"md"`, `"lg"`, `"xl"`, `"full"` |

The submit action is set on the element's `"action"` field, not in props.

```json
"create_form": {
  "type": "Form",
  "props": {
    "max_width": "md"
  },
  "children": ["name_input", "email_input", "submit_btn"],
  "action": { "handler": "users.store", "method": "POST" }
}
```

### Input

Text input field with type, label, validation error, and optional data binding.

| Prop | Type | Description |
|------|------|-------------|
| `field` | `string` | Form field name |
| `label` | `string` | Input label |
| `input_type` | `input_type \| null` | Input type (default: `"text"`) |
| `placeholder` | `string \| null` | Placeholder text |
| `required` | `boolean \| null` | Mark as required |
| `disabled` | `boolean \| null` | Disable the field |
| `error` | `string \| null` | Validation error message |
| `description` | `string \| null` | Help text below the input |
| `default_value` | `string \| null` | Pre-filled static value |
| `data_path` | `string \| null` | JSON Pointer for pre-filling from handler data |
| `step` | `string \| null` | HTML step attribute for number inputs (e.g., `"0.01"`) |

`data_path` is a plain string JSON Pointer (not a `$data` expression). The renderer reads the value from the spec data at that pointer and pre-fills the field.

```json
"email_input": {
  "type": "Input",
  "props": {
    "field": "email",
    "label": "Email Address",
    "input_type": "email",
    "placeholder": "user@example.com",
    "required": true,
    "description": "Your work email",
    "data_path": "/user/email"
  }
}
```

### Select

Dropdown select field with options and optional data binding.

| Prop | Type | Description |
|------|------|-------------|
| `field` | `string` | Form field name |
| `label` | `string` | Select label |
| `options` | `array` | Option objects: `{ "value": string, "label": string }` |
| `placeholder` | `string \| null` | Placeholder text |
| `required` | `boolean \| null` | Mark as required |
| `disabled` | `boolean \| null` | Disable the field |
| `error` | `string \| null` | Validation error message |
| `description` | `string \| null` | Help text below the select |
| `default_value` | `string \| null` | Pre-selected static value |
| `data_path` | `string \| null` | JSON Pointer for pre-selecting from handler data |

```json
"role_select": {
  "type": "Select",
  "props": {
    "field": "role",
    "label": "Role",
    "placeholder": "Select a role",
    "required": true,
    "data_path": "/user/role",
    "options": [
      { "value": "admin", "label": "Administrator" },
      { "value": "editor", "label": "Editor" },
      { "value": "viewer", "label": "Viewer" }
    ]
  }
}
```

### Checkbox

Boolean checkbox field.

| Prop | Type | Description |
|------|------|-------------|
| `field` | `string` | Form field name |
| `label` | `string` | Checkbox label |
| `description` | `string \| null` | Help text below the checkbox |
| `checked` | `boolean \| null` | Default checked state |
| `data_path` | `string \| null` | JSON Pointer for pre-filling from handler data |
| `required` | `boolean \| null` | Mark as required |
| `disabled` | `boolean \| null` | Disable the field |
| `error` | `string \| null` | Validation error message |

```json
"terms_checkbox": {
  "type": "Checkbox",
  "props": {
    "field": "terms",
    "label": "Accept Terms of Service",
    "description": "You must accept to continue.",
    "required": true
  }
}
```

### Switch

Toggle switch — visually distinct from Checkbox but with identical props. The renderer handles the visual difference.

| Prop | Type | Description |
|------|------|-------------|
| `field` | `string` | Form field name |
| `label` | `string` | Switch label |
| `description` | `string \| null` | Help text below the switch |
| `checked` | `boolean \| null` | Default checked state |
| `data_path` | `string \| null` | JSON Pointer for pre-filling from handler data |
| `required` | `boolean \| null` | Mark as required |
| `disabled` | `boolean \| null` | Disable the field |
| `error` | `string \| null` | Validation error message |

```json
"notifications_switch": {
  "type": "Switch",
  "props": {
    "field": "notifications",
    "label": "Enable Notifications",
    "description": "Receive email notifications",
    "checked": true,
    "data_path": "/user/notifications_enabled"
  }
}
```

### Button

Interactive button. Attach the click action on the element's `"action"` field.

| Prop | Type | Description |
|------|------|-------------|
| `label` | `string` | Button label |
| `variant` | `button_variant \| null` | Visual style (default: `"default"`) |
| `size` | `size \| null` | Button size (default: `"default"`) |
| `disabled` | `boolean \| null` | Disable the button |
| `icon` | `string \| null` | Icon name |
| `icon_position` | `icon_position \| null` | `"left"` (default) or `"right"` |
| `button_type` | `string \| null` | HTML button type: `"button"`, `"submit"`, `"reset"` |

```json
"save_btn": {
  "type": "Button",
  "props": {
    "label": "Save Changes",
    "variant": "default",
    "size": "default",
    "icon": "save",
    "icon_position": "left"
  },
  "action": { "handler": "profile.update", "method": "PUT" }
}
```

### ButtonGroup

A horizontal group of buttons rendered together.

| Prop | Type | Description |
|------|------|-------------|
| `buttons` | `array` | Button definitions (same props as Button, plus `"action"`) |

```json
"filter_group": {
  "type": "ButtonGroup",
  "props": {
    "buttons": [
      { "label": "All", "variant": "default" },
      { "label": "Active", "variant": "outline" },
      { "label": "Archived", "variant": "outline" }
    ]
  }
}
```

### DropdownMenu

A button that opens a dropdown with action items. Useful for per-row table actions.

| Prop | Type | Description |
|------|------|-------------|
| `label` | `string` | Trigger button label |
| `actions` | `array` | Action items (see below) |

Each action object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Menu item text |
| `handler` | `string` | Route handler name |
| `method` | `string` | HTTP method |
| `variant` | `string \| null` | `"destructive"` for danger actions |

```json
"row_actions": {
  "type": "DropdownMenu",
  "props": {
    "label": "Actions",
    "actions": [
      { "label": "View Details", "handler": "orders.show", "method": "GET" },
      { "label": "Delete", "handler": "orders.destroy", "method": "DELETE", "variant": "destructive" }
    ]
  }
}
```

---

## Feedback Components

### Alert

Alert message with variant-based styling and optional title.

| Prop | Type | Description |
|------|------|-------------|
| `message` | `string` | Alert message content |
| `variant` | `alert_variant \| null` | Visual style (default: `"info"`) |
| `title` | `string \| null` | Alert title |

```json
"trial_warning": {
  "type": "Alert",
  "props": {
    "message": "Your trial expires in 3 days.",
    "variant": "warning",
    "title": "Trial Ending"
  }
}
```

### Toast

Declarative notification rendered as an overlay by the JS runtime. When a Toast element is in the spec, the runtime displays it on page load and dismisses it after the timeout.

| Prop | Type | Description |
|------|------|-------------|
| `message` | `string` | Toast message content |
| `variant` | `toast_variant \| null` | Visual style (default: `"info"`) |
| `timeout` | `number \| null` | Seconds before auto-dismiss (default: 5) |
| `dismissible` | `boolean \| null` | Allow manual dismiss (default: `true`) |

```json
"save_toast": {
  "type": "Toast",
  "props": {
    "message": "Changes saved successfully.",
    "variant": "success",
    "timeout": 3,
    "dismissible": true
  }
}
```

### EmptyState

Displayed when a list or table has no data. Provides a call-to-action.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Empty state heading |
| `description` | `string \| null` | Supporting text |
| `action_label` | `string \| null` | CTA button label |
| `icon` | `string \| null` | Icon name |

Pair with an element `"action"` for the CTA navigation.

```json
"no_orders": {
  "type": "EmptyState",
  "props": {
    "title": "No orders yet",
    "description": "Create your first order to get started.",
    "action_label": "New Order",
    "icon": "shopping-bag"
  },
  "action": { "handler": "orders.create", "method": "GET" }
}
```

---

## Navigation Components

### Sidebar

Sidebar navigation shell with fixed top items, grouped items, and fixed bottom items. Typically used inside the `dashboard` layout.

| Prop | Type | Description |
|------|------|-------------|
| `fixed_top` | `array \| null` | Items pinned at the top (e.g., logo/home) |
| `groups` | `array \| null` | Collapsible navigation groups |
| `fixed_bottom` | `array \| null` | Items pinned at the bottom (e.g., settings, logout) |

Navigation item object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Link text |
| `href` | `string` | Link URL |
| `icon` | `string \| null` | Icon name |
| `active` | `boolean \| null` | Mark as current page |

Navigation group object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Group heading |
| `collapsed` | `boolean \| null` | Start collapsed |
| `items` | `array` | Navigation items in this group |

```json
"sidebar": {
  "type": "Sidebar",
  "props": {
    "fixed_top": [
      { "label": "Dashboard", "href": "/", "icon": "home", "active": true }
    ],
    "groups": [
      {
        "label": "Management",
        "collapsed": false,
        "items": [
          { "label": "Users", "href": "/users", "icon": "users" },
          { "label": "Orders", "href": "/orders", "icon": "shopping-bag" }
        ]
      }
    ],
    "fixed_bottom": [
      { "label": "Settings", "href": "/settings", "icon": "cog" }
    ]
  }
}
```

### Header

Application header with business name, user info, notification count, and logout link. Typically used inside the `dashboard` layout.

| Prop | Type | Description |
|------|------|-------------|
| `business_name` | `string` | Application name |
| `notification_count` | `number \| null` | Unread notification count |
| `user_name` | `string \| null` | Current user's name |
| `user_avatar` | `string \| null` | Current user's avatar URL |
| `logout_url` | `string \| null` | Logout link URL |

```json
"app_header": {
  "type": "Header",
  "props": {
    "business_name": "My App",
    "notification_count": { "$data": "/notifications/unread" },
    "user_name": { "$data": "/auth/user/name" },
    "logout_url": "/logout"
  }
}
```

### PageHeader

Page-level header with a title, optional subtitle, optional breadcrumb, and optional action button.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Page title |
| `description` | `string \| null` | Page description / subtitle |
| `breadcrumb` | `array \| null` | Breadcrumb items (same shape as Breadcrumb `items`) |
| `action_label` | `string \| null` | Primary action button label |
| `action_variant` | `button_variant \| null` | Action button variant |

Pair with an element `"action"` for the primary action button navigation.

```json
"page_header": {
  "type": "PageHeader",
  "props": {
    "title": "Orders",
    "description": "Manage all customer orders.",
    "breadcrumb": [
      { "label": "Home", "url": "/" },
      { "label": "Orders" }
    ],
    "action_label": "New Order",
    "action_variant": "default"
  },
  "action": { "handler": "orders.create", "method": "GET" }
}
```

### NotificationDropdown

A dropdown list of notification items, typically rendered inside a Header.

| Prop | Type | Description |
|------|------|-------------|
| `notifications` | `array` | Notification items (see below) |
| `empty_text` | `string \| null` | Text when list is empty |

Each notification object:

| Field | Type | Description |
|-------|------|-------------|
| `text` | `string` | Notification message |
| `icon` | `string \| null` | Icon name |
| `timestamp` | `string \| null` | Human-readable time string |
| `read` | `boolean \| null` | Whether the notification has been read |
| `action_url` | `string \| null` | URL to navigate to on click |

```json
"notifications": {
  "type": "NotificationDropdown",
  "props": {
    "empty_text": "No new notifications",
    "notifications": [
      {
        "icon": "bell",
        "text": "New order received",
        "timestamp": "5 minutes ago",
        "read": false,
        "action_url": "/orders/123"
      },
      {
        "text": "Payment processed",
        "timestamp": "1 hour ago",
        "read": true
      }
    ]
  }
}
```

---

## Action Components

### ActionCard

A card that acts as a clickable action item.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Card heading |
| `description` | `string \| null` | Supporting text |
| `icon` | `string \| null` | Icon name |
| `variant` | `action_card_variant \| null` | Visual style: `"default"`, `"outline"`, `"ghost"` |

```json
"create_product": {
  "type": "ActionCard",
  "props": {
    "title": "Add Product",
    "description": "Create a new product listing.",
    "icon": "plus",
    "variant": "outline"
  },
  "action": { "handler": "products.create", "method": "GET" }
}
```

---

## Onboarding Components

### Checklist

Step-by-step onboarding checklist with optional server-side state persistence.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Checklist heading |
| `items` | `array` | Checklist items (see below) |
| `dismissible` | `boolean \| null` | Allow dismissal (default: `true`) |
| `dismiss_label` | `string \| null` | Custom dismiss button label |
| `data_key` | `string \| null` | Server-side state persistence key |

Each item object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Step description |
| `checked` | `boolean \| null` | Whether this step is complete |
| `href` | `string \| null` | Link to complete the step |

```json
"setup_checklist": {
  "type": "Checklist",
  "props": {
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
}
```

---

## Commerce Components

### ProductTile

Product display card with image, title, price, and optional action.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Product name |
| `price` | `string` | Formatted price string (e.g., `"€29.00"`) |
| `description` | `string \| null` | Product description |
| `image_url` | `string \| null` | Product image URL |
| `badge` | `string \| null` | Badge text (e.g., `"New"`, `"Sale"`) |
| `action_label` | `string \| null` | Action button label |

```json
"product_tile": {
  "type": "ProductTile",
  "props": {
    "title": { "$data": "/product/name" },
    "price": { "$data": "/product/price_formatted" },
    "description": { "$data": "/product/description" },
    "image_url": { "$data": "/product/image_url" },
    "badge": "New",
    "action_label": "Add to Cart"
  },
  "action": { "handler": "cart.add", "method": "POST" }
}
```

---

## Kanban Components

### KanbanBoard

Kanban board with multiple columns. On mobile, columns switch to tabs.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `array` | Column element IDs (the columns must be `KanbanColumn` elements) |

```json
"order_board": {
  "type": "KanbanBoard",
  "props": {
    "columns": ["pending_col", "processing_col", "completed_col"]
  }
}
```

### KanbanColumn

A single column in a KanbanBoard.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Column heading |
| `data_path` | `string` | JSON Pointer to the card data array |
| `count` | `number \| null` | Badge count shown in the column header |
| `empty_message` | `string \| null` | Message when the column has no cards |

```json
"pending_col": {
  "type": "KanbanColumn",
  "props": {
    "title": "Pending",
    "data_path": "/orders/pending",
    "count": { "$data": "/orders/pending_count" },
    "empty_message": "No pending orders"
  },
  "children": ["pending_card_template"]
}
```

---

## Extensible Components

For plugin components (third-party or custom types not in the built-in catalog), see **[Plugins](plugins.md)**.
