# Phase 24: Component Catalog - Research

**Researched:** 2026-02-09
**Domain:** Server-rendered UI component catalog modeled on shadcn/ui, with Tailwind CSS styling
**Confidence:** HIGH

<research_summary>
## Summary

Researched shadcn/ui's component catalog, variant system (CVA pattern), CSS variable theming, and the A2UI protocol to inform Ferro's JSON-UI component catalog. The goal is to define typed Rust component types whose props mirror shadcn/ui conventions, so the Phase 28 HTML renderer can emit Tailwind classes that match shadcn/ui's visual output.

Key finding: shadcn/ui uses a **variant + size** system powered by class-variance-authority (CVA). Each component has a small set of named variants (e.g., Button has `default`, `secondary`, `destructive`, `outline`, `ghost`, `link`) and sizes (`xs`, `sm`, `default`, `lg`). The Phase 24 component catalog should model these exact variant/size enums in Rust so the renderer can map them to Tailwind class sets.

The existing Phase 23 schema has 10 components (Card, Table, Form, Button, Input, Select, Alert, Badge, Modal, Text). Research shows we need ~10 more components for a complete CRUD application toolkit, plus prop enrichment on existing components to match shadcn/ui capabilities.

**Primary recommendation:** Expand the component catalog with shadcn/ui-aligned variants and props. Add missing components (Separator, Tabs, Breadcrumb, Pagination, Progress, Checkbox, Switch, Avatar, Skeleton, DescriptionList). Keep the catalog at ~20 components — enough for CRUD apps without over-engineering.
</research_summary>

<standard_stack>
## Standard Stack

### Reference: shadcn/ui Component System

| Concept | shadcn/ui Pattern | Ferro JSON-UI Equivalent |
|---------|-------------------|--------------------------|
| Variant system | CVA (class-variance-authority) | Rust enums per component |
| Theming | CSS variables (OKLCH) | CSS variable names in rendered HTML |
| Composition | Slot-based (CardHeader, CardContent, etc.) | Nested ComponentNode children |
| Dark mode | `.dark` class selector | Body class toggle |
| Styling | Tailwind utility classes | Renderer emits Tailwind classes |

### shadcn/ui CSS Variables (Required for Rendering)

These semantic color tokens MUST be used in the HTML renderer (Phase 28) for theme consistency:

| Variable | Purpose | Usage |
|----------|---------|-------|
| `--background` / `--foreground` | Page surface and text | `bg-background text-foreground` |
| `--card` / `--card-foreground` | Card surfaces | `bg-card text-card-foreground` |
| `--primary` / `--primary-foreground` | Primary actions | `bg-primary text-primary-foreground` |
| `--secondary` / `--secondary-foreground` | Secondary actions | `bg-secondary text-secondary-foreground` |
| `--muted` / `--muted-foreground` | Disabled/inactive | `bg-muted text-muted-foreground` |
| `--accent` / `--accent-foreground` | Highlights | `bg-accent text-accent-foreground` |
| `--destructive` / `--destructive-foreground` | Errors/delete | `bg-destructive text-destructive-foreground` |
| `--border` | Borders | `border-border` |
| `--input` | Input backgrounds | `border-input` |
| `--ring` | Focus rings | `ring-ring` |
| `--radius` | Border radius | `rounded-(--radius)` |

### shadcn/ui Full Component List (66 Components)

Complete official catalog: Accordion, Alert, Alert Dialog, Aspect Ratio, Avatar, Badge, Breadcrumb, Button, Button Group, Calendar, Card, Carousel, Chart, Checkbox, Collapsible, Combobox, Command, Context Menu, Data Table, Date Picker, Dialog, Direction, Drawer, Dropdown Menu, Empty, Field, Hover Card, Input, Input Group, Input OTP, Item, Kbd, Label, Menubar, Native Select, Navigation Menu, Pagination, Popover, Progress, Radio Group, Resizable, Scroll Area, Select, Separator, Sheet, Sidebar, Skeleton, Slider, Sonner, Spinner, Switch, Table, Tabs, Textarea, Toast, Toggle, Toggle Group, Tooltip, Typography.

### Ferro JSON-UI Target Catalog (~20 Components)

Subset selected for CRUD application coverage:

**Already defined (Phase 23, 10 components):**
Card, Table, Form, Button, Input, Select, Alert, Badge, Modal, Text

**New components needed (Phase 24, ~10 components):**

| Component | shadcn/ui Source | Why Needed |
|-----------|------------------|------------|
| Separator | Separator | Visual dividers between sections |
| Tabs | Tabs | Multi-section views (common in detail pages) |
| Breadcrumb | Breadcrumb | Navigation context |
| Pagination | Pagination | Table/list pagination |
| Progress | Progress | Loading/progress indicators |
| Checkbox | Checkbox | Boolean form fields |
| Switch | Switch | Toggle form fields |
| Avatar | Avatar | User display in tables/cards |
| Skeleton | Skeleton | Loading placeholders |
| DescriptionList | (custom) | Key-value detail displays (show pages) |

### Alternatives Considered

| Instead of | Could Use | Decision |
|------------|-----------|----------|
| DescriptionList | Card with Text children | DescriptionList is more semantic for show pages |
| Tabs | Multiple Cards | Tabs is standard UX for sectioned detail |
| Skeleton | Text with loading state | Skeleton is the standard loading pattern |
| Breadcrumb | Text with links | Breadcrumb has proper semantics |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: CVA-Inspired Variant Enums

shadcn/ui uses CVA to define variant → class mappings:

```typescript
// shadcn/ui pattern
const buttonVariants = cva("base-classes", {
  variants: {
    variant: {
      default: "bg-primary text-primary-foreground",
      destructive: "bg-destructive text-destructive-foreground",
      outline: "border border-input bg-background",
      secondary: "bg-secondary text-secondary-foreground",
      ghost: "hover:bg-accent hover:text-accent-foreground",
      link: "text-primary underline-offset-4 hover:underline",
    },
    size: {
      default: "h-9 px-4 py-2",
      sm: "h-8 rounded-md px-3 text-xs",
      lg: "h-10 rounded-md px-8",
      icon: "h-9 w-9",
    },
  },
  defaultVariants: { variant: "default", size: "default" },
});
```

**Ferro equivalent:** Rust enums that the renderer maps to Tailwind classes:

```rust
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    Xs,
    Sm,
    #[default]
    Default,
    Lg,
}
```

### Pattern 2: Shared Variant Types

shadcn/ui reuses the same variant names across components (e.g., `variant: "destructive"` works on Button, Alert, Badge). Ferro should share enums where applicable:

**Shared across components:**
- `Size` — xs, sm, default, lg (Button, Input, Avatar, Badge)
- `AlertVariant` — default, destructive (Alert, AlertDialog)

**Component-specific:**
- `ButtonVariant` — default, secondary, destructive, outline, ghost, link
- `BadgeVariant` — default, secondary, destructive, outline
- `InputType` — text, email, password, number, textarea, hidden, date, time, url, tel, search

### Pattern 3: Compositional Children (shadcn/ui Card Pattern)

shadcn/ui composes cards from sub-components:

```tsx
<Card>
  <CardHeader>
    <CardTitle>Title</CardTitle>
    <CardDescription>Description</CardDescription>
    <CardAction><Button /></CardAction>
  </CardHeader>
  <CardContent>...</CardContent>
  <CardFooter>...</CardFooter>
</Card>
```

Ferro models this with nested `children: Vec<ComponentNode>`:

```json
{
  "key": "user-card",
  "type": "Card",
  "title": "Users",
  "description": "Manage your users",
  "children": [
    {"key": "create-btn", "type": "Button", "label": "Create", "variant": "default"}
  ],
  "footer": [
    {"key": "cancel", "type": "Button", "label": "Cancel", "variant": "outline"}
  ]
}
```

### Pattern 4: Tabs Component Structure

shadcn/ui Tabs use a trigger list + content panels:

```tsx
<Tabs defaultValue="general">
  <TabsList>
    <TabsTrigger value="general">General</TabsTrigger>
    <TabsTrigger value="security">Security</TabsTrigger>
  </TabsList>
  <TabsContent value="general">...</TabsContent>
  <TabsContent value="security">...</TabsContent>
</Tabs>
```

Ferro models this as:

```rust
pub struct TabsProps {
    pub default_tab: String,
    pub tabs: Vec<Tab>,
}

pub struct Tab {
    pub value: String,
    pub label: String,
    pub children: Vec<ComponentNode>,
}
```

### Anti-Patterns to Avoid

- **Exposing raw CSS classes in the schema:** Component types should use semantic props (variant, size), not class names. The renderer owns the class mapping.
- **Too many component types:** 66 shadcn/ui components is for a JS library. Ferro should have ~20 for CRUD. Complex UIs use Inertia.
- **Duplicating HTML semantics:** Don't create Div, Span, Ul, Li components. Use semantic components (Card, Table, DescriptionList) instead.
- **Icon components with arbitrary SVG:** Define an icon enum with known icons, not arbitrary SVG paths.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Variant → class mapping | Custom if/else chains | Enum → match pattern (Rust equivalent of CVA) | Type-safe, exhaustive, easy to extend |
| Color system | Custom color names | shadcn/ui CSS variable names | Ecosystem compatibility, theme support |
| Form layout | Custom grid/flex logic | shadcn/ui Field pattern (label + description + control) | Consistent accessible form structure |
| Icon rendering | Custom SVG embedding | Lucide icon names as string enum | Standard icon set, consistent sizing |
| Loading states | Custom spinners | Skeleton component | Standard pattern, consistent UX |
| Table pagination | Custom page logic | Pagination component with page/per_page props | Declarative, handler-driven |

**Key insight:** The component catalog defines WHAT to render (typed props). The renderer (Phase 28) defines HOW to render (Tailwind classes). Phase 24 must not leak rendering concerns into the schema — no class names, no inline styles, no HTML fragments in props.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Variant Sprawl
**What goes wrong:** Every component gets its own variant enum with overlapping names
**Why it happens:** Copy-pasting variant definitions per component without sharing
**How to avoid:** Define shared enums (Size, Align) and component-specific enums (ButtonVariant, BadgeVariant) explicitly. Document which components share which enums.
**Warning signs:** Three different enums all containing "default", "primary", "secondary" with slightly different members

### Pitfall 2: Props Too Shallow for Real Use
**What goes wrong:** Components lack props needed for real CRUD pages
**Why it happens:** Defining minimal props in schema, discovering gaps during rendering
**How to avoid:** Cross-reference shadcn/ui props for each component. For each existing Ferro component, verify it has all props needed for the sample app's CRUD pages (user list, user detail, user create form, user edit form).
**Warning signs:** Need to create Card+Text workarounds for things that should be a single component prop

### Pitfall 3: Missing Footer/Header Slots
**What goes wrong:** Card and Modal components can't place actions in the right location
**Why it happens:** Only modeling `children` without distinguishing header/content/footer zones
**How to avoid:** Card and Modal should have explicit `footer: Vec<ComponentNode>` alongside `children` (body content). This matches shadcn/ui's CardHeader/CardContent/CardFooter pattern.
**Warning signs:** Agents putting Button components at the end of children[] hoping they render as footer actions

### Pitfall 4: Table Without Pagination Awareness
**What goes wrong:** Table component has no way to express pagination state
**Why it happens:** Treating Table as pure display without considering server-side pagination
**How to avoid:** Table props should include optional pagination (current_page, per_page, total). A separate Pagination component handles the controls.
**Warning signs:** Custom JSON hacks to show "Page 1 of 10" below tables

### Pitfall 5: Form Fields Without Error Display
**What goes wrong:** Input/Select components can't show validation errors
**Why it happens:** Not modeling the error_message prop in form field components
**How to avoid:** All form field components (Input, Select, Checkbox, Switch, Textarea) need an optional `error` prop for validation error display. This connects to Phase 27 (Validation Integration).
**Warning signs:** Separate Alert components used above forms instead of inline field errors
</common_pitfalls>

<code_examples>
## Code Examples

### Enriched Button Props (Phase 24 target)

```rust
// Maps directly to shadcn/ui Button variants
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    Xs,
    Sm,
    #[default]
    Default,
    Lg,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonProps {
    pub label: String,
    #[serde(default)]
    pub variant: ButtonVariant,
    #[serde(default)]
    pub size: Size,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,  // Lucide icon name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_position: Option<IconPosition>,
}
```

### Enriched Card Props with Footer Slot

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardProps {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
    // New: footer slot for action buttons
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footer: Vec<ComponentNode>,
}
```

### New Tabs Component

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabsProps {
    pub default_tab: String,
    pub tabs: Vec<Tab>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    pub value: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ComponentNode>,
}
```

### New Pagination Component

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginationProps {
    pub current_page: u32,
    pub per_page: u32,
    pub total: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}
```

### New DescriptionList Component (for Show Pages)

```rust
/// Key-value pairs for detail/show pages. Renders as a definition list.
/// No direct shadcn/ui equivalent but standard in admin UIs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DescriptionListProps {
    pub items: Vec<DescriptionItem>,
    #[serde(default)]
    pub columns: Option<u8>,  // 1 or 2 column layout
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DescriptionItem {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ColumnFormat>,  // Reuse table's format enum
}
```

### New Checkbox Component

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxProps {
    pub field: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

### Enriched Input Props with Error Support

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputProps {
    pub field: String,
    pub label: String,
    #[serde(default)]
    pub input_type: InputType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    // New: validation error message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // New: helper text
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // New: default value for pre-filled forms
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}
```

### Full JSON Example: User Edit Page

```json
{
  "$schema": "ferro-json-ui/v1",
  "layout": "app",
  "title": "Edit User",
  "components": [
    {
      "key": "breadcrumb",
      "type": "Breadcrumb",
      "items": [
        {"label": "Users", "url": "/users"},
        {"label": "Edit User"}
      ]
    },
    {
      "key": "edit-card",
      "type": "Card",
      "title": "Edit User",
      "description": "Update user information",
      "children": [
        {
          "key": "user-form",
          "type": "Form",
          "action": {"handler": "users.update", "method": "PUT"},
          "fields": [
            {
              "key": "name-input",
              "type": "Input",
              "field": "name",
              "label": "Name",
              "default_value": "John Doe",
              "required": true
            },
            {
              "key": "email-input",
              "type": "Input",
              "field": "email",
              "label": "Email",
              "input_type": "email",
              "default_value": "john@example.com",
              "required": true
            },
            {
              "key": "role-select",
              "type": "Select",
              "field": "role",
              "label": "Role",
              "options": [
                {"value": "admin", "label": "Administrator"},
                {"value": "user", "label": "User"}
              ]
            },
            {
              "key": "active-switch",
              "type": "Switch",
              "field": "is_active",
              "label": "Active",
              "description": "Inactive users cannot log in"
            }
          ]
        }
      ],
      "footer": [
        {"key": "cancel", "type": "Button", "label": "Cancel", "variant": "outline"},
        {"key": "save", "type": "Button", "label": "Save Changes", "variant": "default"}
      ]
    }
  ]
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom variant systems | CVA (class-variance-authority) | 2023 | Type-safe variant → class mapping |
| HSL color variables | OKLCH color variables | shadcn v4 (2025) | Better perceptual uniformity |
| Component packages (npm) | Copy-paste component code | shadcn/ui (2023) | Full ownership, customizable |
| Arbitrary JSON for SDUI | A2UI Protocol v0.9 | Dec 2025 | Standardized streaming SDUI spec |
| Per-component theming | CSS variable semantic tokens | shadcn/ui | Consistent theming across components |

**New patterns to consider:**
- **A2UI Protocol:** Standardized JSON streaming protocol for server-driven UI. Ferro's schema is simpler (no streaming needed) but aligns with A2UI's component model.
- **shadcn/ui Field pattern:** New in recent versions — wraps label, description, and control into accessible Field groups. Ferro should model this in form components.
- **Tailwind v4 @theme inline:** CSS variables exposed as Tailwind utilities automatically.

**Deprecated/outdated:**
- **HSL color format:** shadcn/ui moved to OKLCH
- **tailwind.config.js for theming:** Tailwind v4 uses CSS-first configuration
- **Generic "any" props:** Type all props strictly (CVA pattern proves this works at scale)
</sota_updates>

<open_questions>
## Open Questions

1. **Icon system approach**
   - What we know: shadcn/ui uses Lucide icons. Icons are React components.
   - What's unclear: How to handle icons in server-rendered HTML (SVG inline? CSS icon font? Image URLs?)
   - Recommendation: Define icon as `Option<String>` with Lucide icon names. Defer rendering decision to Phase 28. Consider inlining SVG or using a CDN.

2. **Textarea vs Input with type=textarea**
   - What we know: shadcn/ui has separate Textarea component. Current Ferro schema uses `InputType::Textarea`.
   - What's unclear: Whether to keep Textarea as an InputType or promote to separate component.
   - Recommendation: Keep as `InputType::Textarea` for now. The renderer can handle it. Avoid component sprawl.

3. **RadioGroup component**
   - What we know: shadcn/ui has RadioGroup. Common in forms.
   - What's unclear: Whether to add now or defer.
   - Recommendation: Add as a component if the catalog stays under 22 total. Radio buttons are common in forms.

4. **Link/Anchor component**
   - What we know: Navigation links are fundamental. shadcn/ui Button has `variant: "link"`.
   - What's unclear: Whether to model links as Button with link variant + action, or as separate Link component.
   - Recommendation: Use Button with `variant: "link"` for now. Add dedicated Link component only if needed.

5. **Table row selection**
   - What we know: shadcn/ui DataTable supports row selection with checkboxes.
   - What's unclear: How to model bulk actions in JSON-UI.
   - Recommendation: Defer to Phase 26 (Action System). Table schema can add `selectable: bool` prop later.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [shadcn/ui docs](https://ui.shadcn.com/docs/components) — Full component catalog (66 components), via Context7 `/shadcn-ui/ui` and `/websites/ui_shadcn`
- [shadcn/ui theming](https://ui.shadcn.com/docs/theming) — CSS variable system, OKLCH colors, dark mode
- [CVA docs](https://cva.style/docs) — Variant definition pattern, compound variants, default variants
- [Vercel Academy: Anatomy of shadcn/ui](https://vercel.com/academy/shadcn-ui/extending-shadcn-ui-with-custom-components) — Component structure, composition patterns, type safety

### Secondary (MEDIUM confidence)
- [A2UI Protocol v0.9](https://a2ui.org/specification/v0.9-a2ui/) — Standardized SDUI JSON protocol, component model, data binding
- [shadcn/ui Badge](https://ui.shadcn.com/docs/components/radix/badge) — Badge variant system
- [shadcn/ui Button](https://ui.shadcn.com/docs/components/radix/button) — Button variants, sizes, icon patterns

### Tertiary (LOW confidence — needs validation during implementation)
- [SDUI best practices (Medium)](https://medium.com/@aubreyhaskett/server-driven-ui-what-airbnb-netflix-and-lyft-learned-building-dynamic-mobile-experiences-20e346265305) — Enterprise SDUI patterns from Airbnb/Netflix/Lyft
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: shadcn/ui component catalog, CVA variant system
- Ecosystem: Tailwind CSS v4, OKLCH colors, CSS variables, Lucide icons
- Patterns: Variant enums, composition via children, semantic props
- Pitfalls: Variant sprawl, missing slots, shallow props, rendering leaks

**Confidence breakdown:**
- Component list: HIGH — verified via Context7 + official docs
- Variant system: HIGH — CVA pattern is well-documented
- Props design: HIGH — cross-referenced with shadcn/ui source code via Context7
- New component needs: MEDIUM — based on CRUD app analysis, may need adjustment
- CSS variable system: HIGH — verified from official theming docs

**Research date:** 2026-02-09
**Valid until:** 2026-03-11 (30 days — shadcn/ui ecosystem stable)
</metadata>

---

*Phase: 24-component-catalog*
*Research completed: 2026-02-09*
*Ready for planning: yes*
