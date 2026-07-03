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
| **Forms** | Form, Input, Select, Checkbox, CheckboxList, CheckboxGroup, Switch, Button, ButtonGroup, ActionGroup |
| **Feedback** | Alert, Toast, EmptyState |
| **Navigation** | Sidebar, Header, PageHeader, NotificationDropdown |
| **Action** | ActionCard |
| **Onboarding** | Checklist |
| **Commerce** | ProductTile |
| **Kanban** | KanbanBoard, KanbanColumn |
| **Extensible** | RawHtml, Plugin (see [Plugins](plugins.md)) |

---

## Shared Enum Values

Three shared enums cover the weight, status, and scale axes across all components. One word, one meaning: `variant` is always visual weight, `tone` is always status color, `size` is always the `sm`/`md`/`lg` scale.

**variant** (visual weight of interactive elements) — `"primary"` | `"secondary"` | `"outline"` | `"ghost"` | `"destructive"`

**tone** (status color of stateful display components) — `"neutral"` | `"success"` | `"warning"` | `"destructive"`

**size** — `"sm"` | `"md"` (default) | `"lg"`

## Component-Specific Enum Values

Additional fixed-string enums scoped to individual components; each component section references these by name.

**card_appearance** — `"bordered"` (default) | `"elevated"`

**column_format** — `"date"` | `"date_time"` | `"currency"` | `"boolean"` | `"badge"` | `"image"` | `"icon"`

**text_element** — `"p"` | `"h1"` | `"h2"` | `"h3"` | `"span"` | `"div"` | `"section"`

**input_type** — `"text"` | `"email"` | `"password"` | `"number"` | `"textarea"` | `"hidden"` | `"date"` | `"time"` | `"url"` | `"tel"` | `"search"` | `"file"`

**orientation** — `"horizontal"` | `"vertical"`

**icon_position** — `"left"` | `"right"`

**sort_direction** — `"asc"` | `"desc"`

**form_max_width** — `"default"` | `"narrow"` | `"wide"`

**gap_size** — `"none"` | `"sm"` | `"md"` (default) | `"lg"` | `"xl"` (prop `gap`; a component-scoped scale, not the shared `size`)

## Component vocabulary migration

The canonical `variant`/`tone`/`size` vocabulary replaced the per-component enums. There are no aliases: retired **values** on surviving prop names fail at spec parse, and retired **prop names** (e.g. `Card.variant`, `Badge.variant`, `confirm.variant`) fail catalog validation with an error naming the replacement prop. Old → new mapping:

| Component | Old prop | Old value | New prop | New value |
|-----------|----------|-----------|----------|-----------|
| Button, ActionGroup item | `variant` | `default` | `variant` | `primary` |
| Button, ActionGroup item | `variant` | `link` | `variant` | `ghost` (link style removed) |
| Button, ActionGroup item | `variant` | `secondary`/`outline`/`ghost`/`destructive` | `variant` | unchanged |
| Button, Avatar, SegmentedControl | `size` | `xs` | `size` | `sm` |
| Button, Avatar, SegmentedControl | `size` | `default` | `size` | `md` |
| Alert, Toast | `variant` | `info` | `tone` | `neutral` |
| Alert, Toast | `variant` | `error` | `tone` | `destructive` |
| Alert, Toast | `variant` | `success`/`warning` | `tone` | unchanged |
| Badge | `variant` | `default`/`secondary`/`outline` | `tone` | `neutral` |
| Badge | `variant` | `warning`/`destructive` | `tone` | unchanged |
| Card | `variant` | `bordered`/`elevated` | `appearance` | unchanged values |
| ActionCard | `variant` | `default`/`setup`/`danger` | `tone` | `neutral`/`warning`/`destructive` (+ `success`) |
| DataTable badge column | row `{"variant": ..}` | old badge values | row `{"tone": ..}` | canonical tone values |
| MediaCardGrid | `badge_variant_key` | `outline`/`destructive`/`default` | `badge_tone_key` | canonical tone values |
| ConfirmDialog | `variant` | `default`/`danger` | `tone` | `neutral`/`destructive` |
| Notify outcome | `variant` | `info`/`error` (+`success`/`warning`) | `tone` | `neutral`/`destructive` (+unchanged) |

Behavior and visual deltas to expect when migrating:

- Badge `default`/`secondary`/`outline` all map to `tone: "neutral"`, rendered with one outlined treatment (`border border-border text-text`) — the old primary-tinted default fill is gone.
- Alert `info` was primary-tinted; `tone: "neutral"` renders on the neutral surface family (`bg-surface border-border text-text`).
- Relationship (link) buttons lose the underline-link look — the `link` variant is removed; use `"ghost"`.
- ActionCard's neutral left border changed from `border-l-primary` to `border-l-border`; a `success` tone arm is new.
- CalendarCell `dot_colors` still accepts raw Tailwind class strings (pre-existing behavior outside the semantic vocabulary; a candidate for the design lint).

---

## Layout Components

### Card

Container with title, optional description, nested children, and footer.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Card heading |
| `description` | `string \| null` | Secondary text below the title |
| `subtitle` | `string \| null` | Muted secondary identifier rendered between title and description (e.g. staff name beneath customer name) |
| `badge` | `string \| null` | Small Badge-styled pill rendered to the right of the title (secondary-tinted chrome) for status indicators, counters, or countdown labels |

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

Optional `subtitle` and `badge` slots add a muted secondary identifier and a Badge-styled pill respectively. Vertical stacking: title → subtitle → description.

```json
"booking_card": {
  "type": "Card",
  "props": {
    "title": "Booking #1",
    "subtitle": "Marco Rossi",
    "description": "Pending email confirmation",
    "badge": "Scade tra 9m"
  },
  "children": []
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
      "props": {
        "action": { "handler": "auth.login", "method": "POST" },
        "max_width": "narrow"
      },
      "children": ["email_input", "submit_btn"]
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

#### Appearance

`Card` accepts an optional `appearance` prop controlling chrome and padding.

**card_appearance** — `"bordered"` (default) | `"elevated"`

| Value | Classes applied | Padding | Typical use |
|-------|-----------------|---------|-------------|
| `"bordered"` | `border border-border bg-card shadow-sm overflow-visible` | `p-4` | Dashboard cards in dense layouts |
| `"elevated"` | `bg-card shadow-md overflow-visible` (no border) | `p-8` | Auth pages, error pages, standalone marketing cards |

`appearance` defaults to `"bordered"` when omitted.

```json
"auth_card": {
  "type": "Card",
  "props": {
    "title": "Sign in",
    "appearance": "elevated"
  },
  "children": ["login_form"]
}
```

### Grid

Responsive grid layout for arranging child elements in columns.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `number \| null` | Number of columns (default: 2) |
| `gap` | `gap_size \| null` | Gap between items: `"none"`, `"sm"`, `"md"`, `"lg"`, `"xl"` |

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

#### Visibility

`visible` is an element-level field that lives on every JSON-UI element. It is not a `GridProps` prop. When the visibility condition evaluates `false` against the spec's `data` payload, the Grid and all of its children are absent from the rendered DOM — the entire subtree is omitted (no `hidden` attribute, no empty wrapper).

```json
"staff_chips_row": {
  "type": "Grid",
  "props": { "columns": 1, "gap": "sm" },
  "children": ["staff_chip"],
  "visible": { "path": "/has_staff", "operator": "eq", "value": true }
}
```

Identical semantics apply to every other v2 component — `Card`, `Form`, `Button`, `Badge`, and all plugin components. The visibility check runs once per element in the walker before component dispatch (`ferro-json-ui/src/render/mod.rs` element-level visibility check), so there is no per-component scope shifting and no component-specific visibility behavior.

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
| `id` | `string` | HTML id of the `<dialog>` element; the trigger button references it |
| `title` | `string` | Modal heading |
| `description` | `string \| null` | Modal description text |
| `trigger_label` | `string \| null` | Label for the button that opens the modal |
| `footer` | `array \| null` | Element IDs rendered in the modal footer |

Children of the modal body go in the element `"children"` array. Footer children use the `"footer"` prop listing element IDs.

```json
"delete_modal": {
  "type": "Modal",
  "props": {
    "id": "delete_modal",
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

An expandable/collapsible section (`<details>`/`<summary>`) with a toggle label.

| Prop | Type | Description |
|------|------|-------------|
| `title` | `string` | Label for the toggle |
| `expanded` | `boolean \| null` | Initially open when `true` (default: `false`) |

```json
"advanced_section": {
  "type": "Collapsible",
  "props": {
    "title": "Advanced Options",
    "expanded": false
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

Data-bound table with column definitions, per-row dropdown actions, mobile card fallback, and empty state. Rows are loaded from the spec's data via `data_path`.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `array` | Column definitions (see below) |
| `data_path` | `string` | JSON Pointer to the row data array (e.g., `"/orders"`) |
| `row_actions` | `array \| null` | Per-row dropdown actions — `{"label", "action", "destructive"?, "visible_if"?}` objects |
| `empty_message` | `string \| null` | Message when no data is present |
| `row_key` | `string \| null` | Row field used for `{row_key}` substitution in action URLs (defaults to `id`) |
| `row_href` | `string \| null` | URL pattern for row-click navigation; use `{row_key}` as placeholder |

Each column object:

| Field | Type | Description |
|-------|------|-------------|
| `key` | `string` | Data field key in the row object |
| `label` | `string` | Column header text |
| `format` | `column_format \| null` | Display format |

For `format: "badge"` the cell value is an object `{"tone": .., "label": ..}` with a canonical `tone` value, rendered as a Badge pill. `format: "image"` reads the cell as an image URL; `format: "icon"` reads it as a built-in icon name.

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
      { "label": "Edit", "action": { "handler": "users.edit", "method": "GET" } },
      {
        "label": "Delete",
        "destructive": true,
        "action": {
          "handler": "users.destroy",
          "method": "DELETE",
          "confirm": { "title": "Delete this user?" }
        }
      }
    ],
    "empty_message": "No users found."
  }
}
```

### Table

Lightweight data-bound table. Rows load from the spec's data via `data_path` — like `DataTable`, but with plain per-row action buttons instead of the dropdown menu, row-click navigation, and mobile card fallback. Prefer `DataTable` for entity list pages.

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `array` | Column definitions (same structure as DataTable) |
| `data_path` | `string` | JSON Pointer to the row data array (e.g., `"/plans"`) |
| `row_actions` | `array \| null` | Plain Action objects rendered per row |
| `empty_message` | `string \| null` | Message when no data is present |
| `sortable` | `boolean \| null` | Enable column sorting |
| `sort_column` | `string \| null` | Currently sorted column key |
| `sort_direction` | `sort_direction \| null` | `"asc"` or `"desc"` |

```json
"plans_table": {
  "type": "Table",
  "props": {
    "data_path": "/plans",
    "columns": [
      { "key": "plan", "label": "Plan" },
      { "key": "price", "label": "Price", "format": "currency" }
    ]
  }
}
```

With handler data:

```json
{
  "plans": [
    { "plan": "Starter", "price": "9.00" },
    { "plan": "Pro", "price": "29.00" }
  ]
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

Small tone-styled status label.

| Prop | Type | Description |
|------|------|-------------|
| `label` | `string` | Badge text |
| `tone` | `tone \| null` | Status color (default: `"neutral"`, rendered outlined) |

```json
"status_badge": {
  "type": "Badge",
  "props": {
    "label": "Active",
    "tone": "success"
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
| `size` | `size \| null` | `"sm"`, `"md"` (default), `"lg"` |

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
| `tone` | `tone \| null` | Status accent on the value and icon (default: `"neutral"` — no accent) |
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
| `action` | `object` | Submit action — `{"handler", "method"}` (required) |
| `method` | `string \| null` | HTTP method override (`"GET"`, `"POST"`, `"PUT"`, `"PATCH"`, `"DELETE"`) |
| `max_width` | `form_max_width \| null` | Max form width: `"default"`, `"narrow"`, `"wide"` |

The submit action is set in props (`props.action`), unlike Button and Modal which attach the action on the element's `"action"` field.

```json
"create_form": {
  "type": "Form",
  "props": {
    "action": { "handler": "users.store", "method": "POST" },
    "max_width": "narrow"
  },
  "children": ["name_input", "email_input", "submit_btn"]
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
| `variant` | `variant \| null` | Visual weight (default: `"primary"`) |
| `size` | `size \| null` | Button size (default: `"md"`) |
| `disabled` | `boolean \| null` | Disable the button |
| `icon` | `string \| null` | Icon name |
| `icon_position` | `icon_position \| null` | `"left"` (default) or `"right"` |
| `button_type` | `string \| null` | HTML button type: `"button"` (default), `"submit"` |

```json
"save_btn": {
  "type": "Button",
  "props": {
    "label": "Save Changes",
    "variant": "primary",
    "size": "md",
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
      { "label": "All", "variant": "primary" },
      { "label": "Active", "variant": "outline" },
      { "label": "Archived", "variant": "outline" }
    ]
  }
}
```

### ActionGroup

Renders one ordered action list as inline buttons plus a trailing overflow kebab. The
component partitions the list structurally: the first `max_inline` non-destructive items
render as inline buttons (the first is the primary action), any remaining non-destructive
items move into the kebab, and every `destructive` item is forced into the kebab and rendered
last regardless of input order. The kebab is omitted entirely when nothing overflows. Non-GET
inline actions are wrapped in a `<form>` automatically; GET actions render as plain links.

| Prop | Type | Description |
|------|------|-------------|
| `items` | `array \| {"$data":"/path"}` | Ordered action items (literal list or a data binding) |
| `menu_id` | `string` | Required — pairs the overflow popover |
| `max_inline` | `number \| null` | Inline non-destructive button cap (default `2`) |
| `overflow_label` | `string \| null` | Kebab aria-label (default `"Azioni"`) |
| `row_key` | `string \| null` | Row identifier for `{row_key}` substitution in DataTable/Kanban contexts |

Each item object:

| Field | Type | Description |
|-------|------|-------------|
| `label` | `string` | Button / menu item text |
| `action` | `object` | Action declaration (`{ "handler": ..., "method": ... }`) |
| `destructive` | `boolean` | `true` forces the item into the kebab, rendered last (default `false`) |
| `variant` | `variant \| null` | Button weight for the inline rendering |
| `icon` | `string \| null` | Optional icon name |
| `visible_if` | `string \| null` | Row field gate (fail-closed: an absent or falsy field hides the item) |

`items` accepts a literal array or a `{"$data":"/path"}` binding; in DataTable/Kanban contexts
the bound rows support `{row_key}` substitution and the `visible_if` per-row gate.

```json
"row_actions": {
  "type": "ActionGroup",
  "props": {
    "menu_id": "order_actions",
    "max_inline": 2,
    "items": [
      { "label": "View Details", "action": { "handler": "orders.show", "method": "GET" } },
      { "label": "Mark Shipped", "action": { "handler": "orders.ship", "method": "POST" } },
      { "label": "Refund", "action": { "handler": "orders.refund", "method": "POST" } },
      { "label": "Delete", "action": { "handler": "orders.destroy", "method": "DELETE" }, "destructive": true }
    ]
  }
}
```

In this example "View Details" and "Mark Shipped" render as inline buttons (View first, as the
primary action), "Refund" overflows into the kebab (beyond `max_inline`), and "Delete" is in the
kebab and rendered last because it is destructive.

---

## Feedback Components

### Alert

Alert message with tone-based styling and optional title.

| Prop | Type | Description |
|------|------|-------------|
| `message` | `string` | Alert message content |
| `tone` | `tone \| null` | Status color (default: `"neutral"`) |
| `title` | `string \| null` | Alert title |

```json
"trial_warning": {
  "type": "Alert",
  "props": {
    "message": "Your trial expires in 3 days.",
    "tone": "warning",
    "title": "Trial Ending"
  }
}
```

### Toast

Declarative notification rendered as an overlay by the JS runtime. When a Toast element is in the spec, the runtime displays it on page load and dismisses it after the timeout.

| Prop | Type | Description |
|------|------|-------------|
| `message` | `string` | Toast message content |
| `tone` | `tone \| null` | Status color (default: `"neutral"`) |
| `timeout` | `number \| null` | Seconds before auto-dismiss (default: 5). `0` with `dismissible: true` keeps the toast visible until manually closed |
| `dismissible` | `boolean \| null` | Render a manual close button (default: `true`). When `false`, `timeout` is clamped to a minimum of 1 second so the toast always auto-dismisses |

```json
"save_toast": {
  "type": "Toast",
  "props": {
    "message": "Changes saved successfully.",
    "tone": "success",
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
| `tone` | `tone \| null` | Left-border status color (default: `"neutral"`) |

```json
"create_product": {
  "type": "ActionCard",
  "props": {
    "title": "Add Product",
    "description": "Create a new product listing.",
    "icon": "plus",
    "tone": "neutral"
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

Touch-friendly product tile with quantity controls. Renders the product name, price, and +/− buttons that drive a hidden form input via the JS runtime — used for POS-style product selection inside an order form.

| Prop | Type | Description |
|------|------|-------------|
| `product_id` | `string` | Product identifier |
| `name` | `string` | Product name |
| `price` | `string` | Formatted price string (e.g., `"€29.00"`) |
| `field` | `string` | Form field name the selected quantity is written to |
| `default_quantity` | `number \| null` | Initial quantity (default: 0) |

```json
"product_tile": {
  "type": "ProductTile",
  "props": {
    "product_id": { "$data": "/product/id" },
    "name": { "$data": "/product/name" },
    "price": { "$data": "/product/price_formatted" },
    "field": "quantities[1]"
  }
}
```

Place ProductTile elements inside a `Form` — the quantity value submits with the surrounding form.

---

## Kanban Components

### KanbanBoard

Kanban board with fixed lanes. On mobile, lanes switch to tabs.

A kanban is fixed lanes plus items sorted into them by a status field.
`columns` is **structure** (lane `id` + `title`) and is always rendered — an
empty lane still shows its header and a zero count. Card **content** is
data-bound: `items_path` resolves a flat array of entity objects, each bucketed
into the lane whose `id` equals the item's `group_by` value, then rendered as a
card via the `card_*` / `row_*` bindings. This is the same prescribed-card +
field-key convention used by [`DataTable`](#datatable) and
[`MediaCardGrid`](#mediacardgrid).

| Prop | Type | Description |
|------|------|-------------|
| `columns` | `array \| null` | Lane structure — `KanbanColumnProps` objects (`id` + `title`). Always rendered. |
| `items_path` | `string \| null` | JSON Pointer to a flat array of entity objects to bucket into lanes. |
| `group_by` | `string \| null` | Field on each item selecting its lane: `column.id == item[group_by]`. |
| `card_title_key` | `string \| null` | Item field whose value becomes the card title. |
| `card_description_key` | `string \| null` | Item field whose value becomes the card subtitle. |
| `row_actions` | `array \| null` | Per-card dropdown actions. `{row_key}` / `{id}` interpolate from the item. |
| `row_key` | `string \| null` | Item field used for `{row_key}` substitution in action URLs (defaults to `id`). |
| `mobile_default_column` | `string \| null` | Lane `id` selected by default on mobile tab view. |
| `empty_label` | `string \| null` | Placeholder text shown inside empty lanes. |

```json
"order_board": {
  "type": "KanbanBoard",
  "props": {
    "columns": [
      { "id": "pending",    "title": "Pending" },
      { "id": "processing", "title": "Processing" },
      { "id": "done",       "title": "Done" }
    ],
    "items_path": "/data/order",
    "group_by": "status",
    "card_title_key": "name",
    "card_description_key": "total"
  }
}
```

Handler data — a flat array; the renderer buckets by `status`, so handlers need
no per-lane grouping:
```json
{
  "data": {
    "order": [
      { "id": 1, "name": "#1", "total": "€ 16,00", "status": "pending" },
      { "id": 2, "name": "#2", "total": "€ 40,00", "status": "done" }
    ]
  }
}
```

For fully-custom card structure (badges, nested elements) rather than the
prescribed title/description card, template the cards with the
[`$each` directive inside a fixed `KanbanColumn`](expressions.md#example-kanban-cards-from-a-data-array)
instead.

### KanbanColumn

A single lane definition inside `KanbanBoard.columns` — a column object, not a standalone element type.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `string` | Lane key matched against each item's `group_by` value |
| `title` | `string` | Lane heading |
| `count` | `number \| null` | Lane count badge (static specs only) |
| `children` | `array \| null` | Element IDs rendered inside the lane (static specs only) |

```json
{
  "id": "pending",
  "title": "Pending",
  "children": ["pending_card_template"]
}
```

`count` and `children` are honored only when the board sets neither `items_path` nor `group_by`; in the data-bound path the renderer computes counts and renders cards from `items_path`. See [`$each` inside a KanbanColumn](expressions.md#example-kanban-cards-from-a-data-array) for custom card templating.

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

### StreamText

Connects to a server-sent-events endpoint and renders token-by-token output as
plain text. Tokens are appended as text nodes — no HTML interpretation.

| Prop | Type | Description |
|------|------|-------------|
| `sse_url` | `string` | URL of the SSE endpoint that streams tokens |
| `placeholder` | `string?` | Text shown inside the content area before the first token arrives |
| `loading_text` | `string?` | Status indicator shown while the stream is open |

```json
"response_area": {
  "type": "StreamText",
  "props": {
    "sse_url": "/ai/generate",
    "placeholder": "Response will appear here…",
    "loading_text": "Generating…"
  }
}
```

**Server contract.** The SSE endpoint must emit `event: done` when the stream
is complete:

```rust
tx.send(SseEvent::new().event("done").data("")).await.ok();
```

Without `event: done`, the browser's `EventSource` auto-reconnects after the
connection closes, causing the component to re-fetch the endpoint in a loop.

**Security.** Tokens are appended as plain text nodes — `innerHTML` is never
called. Streamed content cannot inject HTML or execute scripts regardless of
its content.

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
      "props": {
        "action": { "handler": "profile.update", "method": "POST" },
        "max_width": "narrow"
      },
      "children": [
        "name_view", "name_edit",
        "email_view", "email_edit",
        "save_btn"
      ]
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
