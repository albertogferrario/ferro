# Components

Every component in a JSON-UI spec is referenced by its `"type"` string in a flat element map. For the full spec format and workflow, see [Getting Started](getting-started.md).

Each element follows this shape:

```json
"element_id": {
  "type": "ComponentTypeName",
  "props": {
    "prop_name": "prop_value"
  },
  "children": ["child_id"],
  "action": { "handler": "route.name", "method": "POST" },
  "visible": { "path": "/data/status", "operator": "eq", "value": "active" }
}
```

The sections below document every built-in component: its props table (with JSON types) and a complete element example.

---

## Component Overview

| Category | Components |
|----------|------------|
| **Layout** | Card, Grid, Tabs, Separator, Modal, Skeleton, Collapsible, FormSection |
| **Data Display** | Text, DataTable, Table, DescriptionList, Badge, Avatar, Progress, Breadcrumb, Pagination, StatCard, Image, CalendarCell |
| **Forms** | Form, Input, Select, Checkbox, CheckboxList, CheckboxGroup, Switch, Button, ButtonGroup, DropdownMenu |
| **Feedback** | Alert, Toast, EmptyState |
| **Navigation** | Sidebar, Header, PageHeader, NotificationDropdown |
| **Action** | ActionCard |
| **Onboarding** | Checklist |
| **Commerce** | ProductTile |
| **Kanban** | KanbanBoard, KanbanColumn |
| **Extensible** | RawHtml, Plugin (see [Plugins](plugins.md)) |

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

**form_max_width** — `"default"` | `"narrow"` | `"wide"`

**gap_size** — `"none"` | `"sm"` | `"md"` (default) | `"lg"` | `"xl"`

**action_card_variant** — `"default"` | `"setup"` | `"danger"`

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

**Children semantics:** On container components (`Card`, `Form`, `Grid`, etc.) `children` is an array of element ID strings that reference entries in the spec's flat `elements` map. The elements themselves are siblings at the top level — not nested.

```json
{
  "$schema": "ferro-json-ui/v2",
  "root": "card_main",
  "elements": {
    "card_main": {
      "type": "Card",
      "props": { "title": "Welcome" },
      "children": ["heading", "form_login"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "Sign in", "element": "h2" }
    },
    "form_login": {
      "type": "Form",
      "props": { "max_width": "sm" },
      "children": ["email_input", "submit_btn"],
      "action": { "handler": "auth.login", "method": "POST" }
    },
    "email_input": {
      "type": "Input",
      "props": { "field": "email", "label": "Email", "input_type": "email" }
    },
    "submit_btn": {
      "type": "Button",
      "props": { "label": "Sign in", "button_type": "submit" }
    }
  }
}
```

All elements — `card_main`, `heading`, `form_login`, `email_input`, `submit_btn` — are siblings in the `elements` map. The tree structure is expressed purely through `children` ID references.

#### Variant

`Card` accepts an optional `variant` prop controlling chrome and padding.

**card_variant** — `"bordered"` (default) | `"elevated"`

| Value | Classes applied | Padding | Typical use |
|-------|-----------------|---------|-------------|
| `"bordered"` | `border border-border bg-card shadow-sm overflow-visible` | `p-4` | Dashboard cards in dense layouts |
| `"elevated"` | `bg-card shadow-md overflow-visible` (no border) | `p-8` | Auth pages, error pages, standalone marketing cards |

`variant` defaults to `"bordered"` when omitted.

```json
"auth_card": {
  "type": "Card",
  "props": {
    "title": "Sign in",
    "variant": "elevated"
  },
  "children": ["login_form"]
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

#### Dynamic items via data_path

`data_path` (optional) takes precedence over `items` when set. It resolves to a JSON array decoded as `Vec<DescriptionItem>` (`{ "label": string, "value": string, "format"?: string }`). Falls back to `items` when the path is missing.

```json
"document_details": {
  "type": "DescriptionList",
  "props": {
    "columns": 2,
    "data_path": "/document/fields"
  }
}
```

Handler data: `{ "document": { "fields": [{ "label": "Author", "value": "Alice" }, { "label": "Created", "value": "2026-05-17", "format": "date" }] } }`.

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

#### Dynamic source via data_path

`data_path` (optional) takes precedence over `src` when set. It is a JSON Pointer resolved against handler data at render time; the resolved value is used as the `<img src>` attribute. Falls back to the static `src` value when the path is missing or resolves to a non-string.

```json
"product_image": {
  "type": "Image",
  "props": {
    "src": "/images/placeholder.jpg",
    "alt": "Product image",
    "data_path": "/product/image_url"
  }
}
```

Handler data: `{ "product": { "image_url": "/uploads/product-42.jpg" } }` → rendered `src` is `/uploads/product-42.jpg`.

### CalendarCell

Renders a single day cell in a month grid. Intended for use inside a custom calendar layout; not a standalone page component.

| Prop | Type | Description |
|------|------|-------------|
| `day` | `number` | Day of month (1–31) |
| `is_today` | `boolean \| null` | Highlights the cell as today (default: `false`) |
| `is_current_month` | `boolean \| null` | Dims the cell when outside the current month (default: `false`) |
| `event_count` | `number \| null` | Event indicator dot count (default: `0`) |
| `dot_colors` | `array \| null` | Per-event Tailwind color classes (e.g. `"bg-blue-500"`). When non-empty, colored dots replace plain primary dots. |

```json
"day_14": {
  "type": "CalendarCell",
  "props": {
    "day": 14,
    "is_today": true,
    "is_current_month": true,
    "event_count": 3,
    "dot_colors": ["bg-blue-500", "bg-green-500", "bg-red-500"]
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

State-flip toggle. Use Switch when the semantic is "flip this state" (on/off, open/closed, enabled/disabled) — distinct from Checkbox, which expresses a binary choice within a set of options. The renderer emits `role="switch"` and `aria-checked` so browsers and assistive technology recognize the toggle affordance.

| Prop | Type | Description |
|------|------|-------------|
| `field` | `string` | Form field name |
| `label` | `string` | Switch label |
| `description` | `string \| null` | Help text below the switch |
| `checked` | `boolean \| null` | Default checked state |
| `data_path` | `string \| null` | JSON Pointer for pre-filling from handler data |
| `required` | `boolean \| null` | Mark as required |
| `disabled` | `boolean \| null` | Disable the field |
| `compact` | `boolean \| null` | Scale the toggle down (scale-75) for use in dense grid layouts |
| `error` | `string \| null` | Validation error message |
| `action` | `Action \| null` | When present, wraps the switch in a `<form>` and auto-submits on change |

```json
"day_open_switch": {
  "type": "Switch",
  "props": {
    "field": "day_1_is_open",
    "label": "Aperto",
    "data_path": "/schedule/day_1_is_open",
    "compact": true,
    "action": { "handler": "schedule.toggle_day", "method": "POST", "url": "/schedule/toggle" }
  }
}
```

#### Substitution: Checkbox styled as switch

For consumers who do not need state-flip semantics and prefer to compose from Checkbox primitives, render a `Checkbox` and apply the Tailwind utility classes that yield a switch appearance (rounded-full track, translated indicator, etc.). Switch remains the recommended path when the semantic is "flip this state" — its dedicated rendering emits the `role="switch"` ARIA marker and the visual affordance browsers and assistive technology recognize.

There is no `variant: "switch"` prop on `Checkbox` today. The substitution is purely visual via custom class hooks, not an API-level feature.

### CheckboxList

A group of checkboxes sharing a single form field name. Each checked option submits as `field=value`. Supports both static option lists and data-driven options resolved from handler data.

| Prop | Type | Description |
|------|------|-------------|
| `field` | `string` | Form field name; each selected checkbox submits as `field=value` |
| `options` | `array \| null` | Static option list: `[{ "value": string, "label": string }]` |
| `options_path` | `string \| null` | JSON Pointer to a data array of `{ "value", "label" }` objects (used when `options` is empty) |
| `selected_path` | `string \| null` | JSON Pointer to a `string[]` of pre-selected values |
| `label` | `string \| null` | Group label |
| `description` | `string \| null` | Help text below the group |
| `disabled` | `boolean \| null` | Disable all checkboxes |
| `error` | `string \| null` | Validation error message |

`options_path` and `selected_path` are plain JSON Pointer strings, not `$data` expressions.

```json
"services_list": {
  "type": "CheckboxList",
  "props": {
    "field": "services",
    "label": "Choose Services",
    "options_path": "/available_services",
    "selected_path": "/user/selected_services"
  }
}
```

### CheckboxGroup

An alias for [`CheckboxList`](#checkboxlist). Accepts identical props and produces identical HTML output — a `<fieldset>` with one `<input type="checkbox">` per option, each with `name="field"` for form submission. Use whichever name reads more clearly in a given spec; there is no behavioral difference.

```json
"copy_targets": {
  "type": "CheckboxGroup",
  "props": {
    "field": "copy_to",
    "label": "Copia su",
    "options": [
      { "value": "tue", "label": "Martedì" },
      { "value": "wed", "label": "Mercoledì" },
      { "value": "thu", "label": "Giovedì" }
    ]
  }
}
```

Each checked option submits as `copy_to=<value>`. When multiple options are checked, the browser sends repeated `copy_to` parameters (standard HTML multi-value form semantics).

#### Substitution: composing from Checkbox primitives

As an alternative, the same array-submit semantics can be composed directly from individual `Checkbox` elements whose `field` ends in `[]`. The `[]` suffix causes the browser to collect all checked values under a single array key:

```json
"copy_tue": {
  "type": "Checkbox",
  "props": { "field": "copy_to[]", "label": "Martedì", "value": "tue" }
},
"copy_wed": {
  "type": "Checkbox",
  "props": { "field": "copy_to[]", "label": "Mercoledì", "value": "wed" }
},
"copy_thu": {
  "type": "Checkbox",
  "props": { "field": "copy_to[]", "label": "Giovedì", "value": "thu" }
}
```

Each checked input submits as `copy_to[]=<value>`, which most server frameworks decode as an array under the key `copy_to`.

**When to use `CheckboxGroup`:** data-driven multi-select where the option list comes from handler data (`options_path`) or is defined once statically. Compact and concise.

**When to compose from `Checkbox`:** per-option conditional visibility (`"visible"` rules), per-option custom layout inside a `Grid` or `FormSection`, or per-option `data_path` binding. The explicit form is more verbose but gives full control over each item's placement and visibility.

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

Page-level header with a title, optional subtitle, optional breadcrumb, and optional action buttons.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Page title |
| `breadcrumb` | `array \| null` | Breadcrumb items (same shape as Breadcrumb `items`) |
| `actions` | `array \| null` | Element IDs of action button elements rendered to the right of the title |

```json
"page_header": {
  "type": "PageHeader",
  "props": {
    "title": "Orders",
    "breadcrumb": [
      { "label": "Home", "url": "/" },
      { "label": "Orders" }
    ],
    "actions": ["new_order_btn"]
  }
}
```

#### actions — lax acceptance

`actions` accepts any of the following forms, all of which deserialize to an empty or populated list:

| Wire value | Result |
|------------|--------|
| omitted | empty list |
| `null` | empty list |
| `""` (empty string) | empty list |
| `["btn_id", ...]` | list of element IDs |

Controllers that pass `""` or omit the field when there are no actions do not need a special-case branch — all lax forms produce an empty list.

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
| `columns` | `array \| null` | Static `KanbanColumnProps` objects. Used when `data_path` is absent. |
| `data_path` | `string \| null` | JSON Pointer to an array of `KanbanColumnProps` objects in handler data. Takes precedence over `columns` when set. |
| `mobile_default_column` | `string \| null` | Column `id` selected by default on mobile tab view |

```json
"order_board": {
  "type": "KanbanBoard",
  "props": {
    "columns": ["pending_col", "processing_col", "completed_col"]
  }
}
```

#### Dynamic columns via data_path

When `data_path` is set, the renderer resolves the array at that path against handler data and decodes each entry as a `KanbanColumnProps` object (`{ "id", "title", "count", "children"? }`). The static `columns` value is ignored.

```json
"order_board": {
  "type": "KanbanBoard",
  "props": {
    "data_path": "/order_columns"
  }
}
```

Handler data:
```json
{
  "order_columns": [
    { "id": "pending",    "title": "Pending",    "count": 4, "children": [] },
    { "id": "processing", "title": "Processing", "count": 2, "children": [] },
    { "id": "done",       "title": "Done",       "count": 9, "children": [] }
  ]
}
```

Use `data_path` when the column set varies per request (e.g., one column per order status fetched from the database). For finer-grained templating — one element type per card row inside a fixed column — see the [$each directive for kanban cards](expressions.md#example-kanban-cards-from-a-data-array) in expressions.md.

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

### RawHtml

Server-injected HTML island for narrow HTML-fragment use cases: status pills, badge decorations, link wrappers, and similar one-off markup that does not warrant a first-class plugin.

| Prop | Type | Description |
|------|------|-------------|
| `html` | `string` | Server-constructed HTML emitted verbatim into the response |

```json
"status_pill": {
  "type": "RawHtml",
  "props": {
    "html": "<span class=\"pill pill-green\">Active</span>"
  }
}
```

**Trust boundary.** `html` is emitted verbatim with no sanitization. The consumer is responsible for ensuring the value is safe before embedding it in the spec. For untrusted input (e.g., user-supplied content), run it through a sanitizer such as [`ammonia`](https://crates.io/crates/ammonia) in the handler before assigning it to `html`. This mirrors the discipline required by `RichTextEditor`.

For richer widgets that are interactive, need asset injection (CSS/JS bundles), or are reused across multiple pages, use the first-class plugin system instead — see [plugins.md](plugins.md).

For plugin components (third-party or custom types not in the built-in catalog), see **[Plugins](plugins.md)**.

---

## Inline view/edit pattern

An inline view/edit page is built from a `Form` element whose children include both read-only display items and editable inputs, each toggled by a `visible` condition on a query parameter.

This pattern requires no Rust code to distinguish view and edit modes — the spec handles it entirely through `visible` rules on `query.mode`.

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Profile",
  "layout": "dashboard",
  "root": "profile_card",
  "elements": {
    "profile_card": {
      "type": "Card",
      "props": { "title": "Profile" },
      "children": ["edit_btn", "profile_form"]
    },
    "edit_btn": {
      "type": "Button",
      "props": { "label": "Edit", "variant": "outline" },
      "action": { "url": "?mode=edit" },
      "visible": { "ne": ["query.mode", "edit"] }
    },
    "profile_form": {
      "type": "Form",
      "props": { "max_width": "md" },
      "children": [
        "name_view", "name_edit",
        "email_view", "email_edit",
        "save_btn"
      ],
      "action": { "handler": "profile.update", "method": "POST" }
    },
    "name_view": {
      "type": "DescriptionList",
      "props": {
        "items": [{ "label": "Name", "value": { "$data": "/user/name" } }]
      },
      "visible": { "ne": ["query.mode", "edit"] }
    },
    "name_edit": {
      "type": "Input",
      "props": {
        "field": "name",
        "label": "Name",
        "data_path": "/user/name"
      },
      "visible": { "eq": ["query.mode", "edit"] }
    },
    "email_view": {
      "type": "DescriptionList",
      "props": {
        "items": [{ "label": "Email", "value": { "$data": "/user/email" } }]
      },
      "visible": { "ne": ["query.mode", "edit"] }
    },
    "email_edit": {
      "type": "Input",
      "props": {
        "field": "email",
        "label": "Email Address",
        "input_type": "email",
        "data_path": "/user/email"
      },
      "visible": { "eq": ["query.mode", "edit"] }
    },
    "save_btn": {
      "type": "Button",
      "props": { "label": "Save", "button_type": "submit" },
      "visible": { "eq": ["query.mode", "edit"] }
    }
  }
}
```

The `visible` condition `{ "eq": ["query.mode", "edit"] }` shows the element only when `?mode=edit` is present in the URL. The inverse `{ "ne": ["query.mode", "edit"] }` shows the element in all other cases (view mode). See [Data Binding & Visibility](data-binding.md) for the full `visible` condition reference.
