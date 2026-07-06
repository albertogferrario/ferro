# ferro-json-ui Component Requirements — Dashboard Shell

**Source:** Phase 4 (Dashboard Shell) context decisions
**Date:** 2026-03-11
**Purpose:** Define what ferro-json-ui components the gestiscilo dashboard needs so the library can be developed to match this real-world case scenario.

---

## Dashboard Layout

The dashboard is a persistent shell: fixed sidebar on the left, header bar at the top, scrollable content area.

### Shell (`DashboardLayout`)
- Fixed sidebar (left) + header (top) + content area (right)
- Sidebar and header persist across all dashboard pages — never unmount or reflow
- Content area swaps based on route
- Mobile: sidebar collapses into hamburger menu

### Sidebar (`Sidebar`)
- **Dynamic composition from data** — accepts a list of groups and items, not hardcoded
- **Collapsible service groups** — each group has a label ("Cassa", "Prenotazioni") and contains child nav items
- **Active state** — current page highlighted
- **Conditional rendering** — groups/items appear or disappear based on tenant's active services and modules
- **Fixed items** — "Home" at top, "Impostazioni" at bottom (always present regardless of services)
- **Data shape example:**
  ```json
  {
    "fixed_top": [{ "label": "Home", "href": "/dashboard", "icon": "home" }],
    "groups": [
      {
        "label": "Cassa",
        "collapsed": false,
        "items": [
          { "label": "Prodotti", "href": "/dashboard/cassa/prodotti", "icon": "package" },
          { "label": "Ordini", "href": "/dashboard/cassa/ordini", "icon": "receipt" },
          { "label": "Pagamenti", "href": "/dashboard/cassa/pagamenti", "icon": "credit-card" }
        ]
      },
      {
        "label": "Prenotazioni",
        "collapsed": false,
        "items": [
          { "label": "Calendario", "href": "/dashboard/prenotazioni/calendario", "icon": "calendar" },
          { "label": "Prenotazioni", "href": "/dashboard/prenotazioni/lista", "icon": "users" }
        ]
      }
    ],
    "fixed_bottom": [{ "label": "Impostazioni", "href": "/dashboard/impostazioni", "icon": "settings" }]
  }
  ```

### Header (`Header`)
- Business name displayed (left or center)
- **Bell notification icon** (right) — shows unread count badge
- Bell click opens a **notification dropdown** with recent notifications list
- Logout button or user avatar with dropdown

---

## Components

### 1. Stat Card (`StatCard`)
- Displays a single metric: label, value, optional icon
- Value formats: integer count ("12"), currency ("€145,00")
- Row layout: multiple stat cards side by side (responsive — stack on mobile)
- **Live update** — value can be replaced via JS when SSE event arrives, without full page re-render
- States: normal (with value), zero ("0"), loading (optional skeleton)
- Example:
  ```
  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
  │ 📦 Ordini   │ │ 💰 Incasso  │ │ 📅 Prenotaz.│
  │    12       │ │  €345,00    │ │     8       │
  │    oggi     │ │    oggi     │ │    oggi     │
  └─────────────┘ └─────────────┘ └─────────────┘
  ```

### 2. Quick Action Button (`ActionButton`)
- Prominent button for primary actions ("Nuovo ordine", "Nuova prenotazione")
- Variants: primary (filled), secondary (outline)
- Can include an icon + label
- Row layout: multiple buttons side by side

### 3. Checklist (`Checklist`)
- Container with title and dismiss button ("Ho capito" / X)
- List of items, each with:
  - Checkbox state (unchecked, checked with strikethrough)
  - Label text
  - Optional link (clicking navigates to the relevant page)
- Auto-hides when all items are checked
- Dismissible manually at any point
- **State persistence** — needs a data attribute to track which items are completed (server-side)
- Example:
  ```
  ┌─ Primi passi ──────────────── [Ho capito] ┐
  │ ☐ Esplora la dashboard                     │
  │ ☑ ~~Aggiungi un prodotto~~                 │
  │ ☐ Condividi il tuo link                    │
  └────────────────────────────────────────────┘
  ```

### 4. Toast Notification (`Toast`)
- Appears at top-right (or bottom-right) of the viewport
- Auto-dismisses after N seconds (configurable, ~5s default)
- Manual dismiss via X button
- Variants: info, success, warning, error
- Stackable (multiple toasts can appear simultaneously)
- **Triggered by SSE events** — JS creates a toast when an SSE message arrives
- Example: "Nuovo ordine #42 ricevuto"

### 5. Notification Dropdown (`NotificationDropdown`)
- Anchored to bell icon in header
- Shows list of recent notifications (most recent first)
- Each notification: icon, text, timestamp ("2 min fa")
- "Segna come lette" action to clear the badge count
- Empty state: "Nessuna notifica"

### 6. Tabs (`Tabs`)
- Horizontal tab bar with labels
- Content area switches based on active tab
- Supports 2-5 tabs
- Active tab visually distinct (underline or filled)
- Used for settings page: Generale / Servizi / Account
- Example:
  ```
  [ Generale ]  [ Servizi ]  [ Account ]
  ─────────────────────────────────────
  (tab content area)
  ```

### 7. Form (`Form`)
- Standard form with labeled fields
- Field types needed: text input, dropdown/select, toggle switch
- Inline validation (error message below field)
- Submit button (primary action)
- Live validation for slug field (format + uniqueness check via fetch)

### 8. Toggle Switch (`Toggle`)
- On/off switch for boolean settings
- Used for service activation and module activation in Servizi tab
- Label + description text + toggle aligned right
- Grouped: toggles nested under a service heading
- Example:
  ```
  Cassa                              [ON]
    Prodotti                         [ON]
    Ordini                           [ON]
    Pagamenti                        [OFF]

  Prenotazioni                       [ON]
    Calendario                       [ON]
    Prenotazioni                     [ON]
  ```

### 9. Alert / Confirmation (`Alert`)
- Used for destructive actions (account deletion)
- Shows warning text + confirmation input (type "ELIMINA")
- Cancel + Confirm buttons
- Already exists in account deletion flow — needs to work within settings tab context

---

## Client-Side Behaviors

### SSE Integration
- Dashboard page opens an SSE connection to `/dashboard/events`
- Events arrive as JSON with type + payload
- On event:
  1. Create a Toast notification
  2. Increment bell counter
  3. Add notification to dropdown list
  4. Update relevant StatCard values (re-fetch or delta)
- Connection auto-reconnects on drop (EventSource default behavior)

### Dynamic Sidebar
- Sidebar content is rendered server-side from DB query
- No client-side sidebar updates needed (page navigation triggers full re-render of sidebar)
- Active page detection: server marks current nav item based on route

### Form Validation
- Slug field: debounced fetch to `/dashboard/api/check-slug?slug=xxx` for uniqueness
- Other fields: HTML5 validation attributes (required, minlength, pattern)

---

## Page Compositions

### Home Page (`/dashboard`)
```
┌──────────┬──────────────────────────────────────┐
│ Sidebar  │  [StatCard] [StatCard] [StatCard]     │
│          │                                        │
│  Home ●  │  [ActionButton] [ActionButton]         │
│          │                                        │
│  Cassa   │  ┌─ Checklist ─────────────────────┐  │
│  Prodotti│  │ ☐ Item 1                        │  │
│  Ordini  │  │ ☑ ~~Item 2~~                    │  │
│  Pagam.  │  │ ☐ Item 3                        │  │
│          │  └─────────────────────────────────┘  │
│  Prenot. │                                        │
│  Calend. │                                        │
│  Prenot. │                                        │
│          │                                        │
│  Impost. │                                        │
└──────────┴──────────────────────────────────────┘
```

### Settings Page (`/dashboard/impostazioni`)
```
┌──────────┬──────────────────────────────────────┐
│ Sidebar  │  [ Generale ]  [ Servizi ]  [ Account]│
│          │  ─────────────────────────────────────│
│          │                                        │
│  ...     │  (Tab content: form fields, toggles,  │
│          │   or account management depending on   │
│  Impost.●│   active tab)                         │
│          │                                        │
└──────────┴──────────────────────────────────────┘
```

---

## Summary of Required Components

| Component | Priority | SSE-aware | Notes |
|-----------|----------|-----------|-------|
| DashboardLayout (shell) | Critical | No | Sidebar + header + content |
| Sidebar | Critical | No | Dynamic groups from data |
| Header | Critical | No | Business name + bell icon |
| StatCard | High | Yes | Live-updating values |
| ActionButton | High | No | Primary/secondary variants |
| Checklist | High | No | Persistent state, dismissible |
| Toast | High | Yes | Triggered by SSE events |
| NotificationDropdown | High | Yes | Bell dropdown with list |
| Tabs | High | No | Settings page layout |
| Form | High | No | Text, select, validation |
| Toggle | High | No | Service/module activation |
| Alert | Medium | No | Destructive action confirmation |

---

*Generated from Phase 4 context decisions — 2026-03-11*
