# Phase 23: JSON-UI Schema - Research

**Researched:** 2026-01-16
**Domain:** Server-Driven UI (SDUI) with JSON schema, Rust HTML generation
**Confidence:** HIGH

<research_summary>
## Summary

Researched JSON-driven UI systems for defining UI components declaratively. The dominant pattern is Server-Driven UI (SDUI) where the server sends JSON descriptions that the client renders. Vercel's json-render is the most relevant reference implementation, designed for AI-generated UIs with guardrails.

Key finding: The schema should separate concerns into three layers:
1. **Component catalog** - Define available components with typed props
2. **Data binding** - JSONPath-based references to connect components to data
3. **Actions** - Declarative action definitions with confirmations and callbacks

Ferro's approach (server-side Rust → HTML) is unique. Unlike typical SDUI where JSON is sent to a JS client for rendering, Ferro will render JSON → HTML on the server, outputting Tailwind classes directly. This is simpler but requires careful schema design.

**Primary recommendation:** Design a component-first schema inspired by json-render, but optimized for server-side HTML generation. Components define props and children, actions map to Ferro handlers, and visibility rules enable conditional rendering.
</research_summary>

<standard_stack>
## Standard Stack

### Reference Implementations
| Library | Language | Purpose | Why Relevant |
|---------|----------|---------|--------------|
| [json-render](https://github.com/vercel-labs/json-render) | TypeScript | AI → JSON → UI | Best schema design reference |
| [JSON Forms](https://jsonforms.io/) | TypeScript | JSON Schema → Forms | Data/UI schema separation |
| [SDUI (Flutter)](https://github.com/wutsi/sdui) | Dart | Server-Driven UI | Action/navigation patterns |

### Rust HTML Generation Options
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| None (inline) | - | Direct string building | Ferro's current Inertia approach |
| [Maud](https://maud.lambda.xyz/) | 0.27 | Compile-time HTML macro | If templates become complex |
| [Askama](https://github.com/askama-rs/askama) | 0.12 | Jinja-like templates | If external templates needed |

### Recommended Approach for Ferro
| Component | Choice | Rationale |
|-----------|--------|-----------|
| Schema format | JSON | Already used in Ferro, serde support |
| Prop validation | Rust types + serde | Compile-time safety |
| HTML generation | Inline string building | Consistent with ferro-inertia |
| CSS framework | Tailwind classes | Already in project, no CSS file needed |

**Note:** Ferro already uses inline HTML generation in `ferro-inertia/src/response.rs`. JSON-UI should follow the same pattern for consistency.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Schema Structure
```json
{
  "$schema": "ferro-json-ui/v1",
  "layout": "page",
  "components": [
    {
      "key": "users-table",
      "type": "Table",
      "props": {
        "columns": [...],
        "dataPath": "/data/users"
      },
      "actions": [
        {
          "name": "delete",
          "handler": "users.destroy",
          "confirm": { "title": "Delete user?", "variant": "danger" }
        }
      ],
      "visibility": { "path": "/data/users", "operator": "notEmpty" }
    }
  ]
}
```

### Pattern 1: Component Catalog (json-render style)
**What:** Define available components with typed props
**When to use:** Always - this is the foundation
**Example:**
```rust
// Rust component catalog definition
pub enum Component {
    Table(TableProps),
    Form(FormProps),
    Card(CardProps),
    Button(ButtonProps),
    Input(InputProps),
    Alert(AlertProps),
    // ...
}

#[derive(Deserialize)]
pub struct TableProps {
    pub columns: Vec<Column>,
    pub data_path: String,  // JSONPath to data
    pub actions: Option<Vec<Action>>,
}
```

### Pattern 2: Separation of Concerns (JSON Forms style)
**What:** Keep data schema separate from UI schema
**When to use:** Forms with validation
**Example:**
```json
{
  "data": { "email": "", "name": "" },
  "ui": {
    "type": "Form",
    "children": [
      { "type": "Input", "props": { "field": "email", "label": "Email" } },
      { "type": "Input", "props": { "field": "name", "label": "Name" } }
    ]
  },
  "validation": {
    "email": ["required", "email"],
    "name": ["required", "min:2"]
  }
}
```

### Pattern 3: Action Declarations (SDUI style)
**What:** Map user interactions to backend handlers
**When to use:** Buttons, form submissions, navigation
**Example:**
```json
{
  "action": {
    "name": "submit",
    "handler": "posts.store",
    "method": "POST",
    "confirm": {
      "title": "Create Post?",
      "message": "This will publish the post.",
      "variant": "default"
    },
    "onSuccess": { "redirect": "/posts" },
    "onError": { "showErrors": true }
  }
}
```

### Pattern 4: Conditional Visibility (json-render style)
**What:** Show/hide components based on data or auth
**When to use:** Dynamic UIs, permission-based features
**Example:**
```json
{
  "visibility": {
    "and": [
      { "path": "/auth/user", "operator": "exists" },
      { "path": "/auth/user/role", "operator": "eq", "value": "admin" }
    ]
  }
}
```

### Anti-Patterns to Avoid
- **Unversioned schemas:** Always include `$schema` for version tracking
- **Client-side logic in JSON:** Keep JSON declarative, logic in Rust
- **Deeply nested components:** Flatten structure, use slots/partials
- **Generic "any" props:** Type all props strictly in Rust
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema validation | Custom validators | serde + Rust types | Compile-time safety, existing patterns |
| JSONPath evaluation | Custom path parser | jsonpath-rust or serde_json_path | Edge cases, spec compliance |
| HTML escaping | Manual string replace | html_escape crate or existing Ferro utils | XSS prevention, correctness |
| Form validation display | Custom error rendering | Extend existing Ferro validation | Consistency with Inertia forms |
| Component variants | Per-component CSS | Tailwind utility classes | Matches project theme |
| Confirmation dialogs | Custom modal logic | Declarative confirm in action schema | Consistency, simplicity |

**Key insight:** Ferro already has patterns for HTML generation (ferro-inertia), validation (framework/validation), and form handling. JSON-UI should extend these, not replace them.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Unstructured JSON Chaos
**What goes wrong:** Schema evolves without versioning, clients break on updates
**Why it happens:** Not treating JSON schema as a contract
**How to avoid:**
- Version schema from day one (`$schema: "ferro-json-ui/v1"`)
- Define Rust types for all components
- Validate JSON against schema at parse time
**Warning signs:** Different components accepting arbitrary props

### Pitfall 2: Performance Death by JSON Parsing
**What goes wrong:** Large JSON files parsed on every request
**Why it happens:** No caching of parsed/rendered components
**How to avoid:**
- Cache compiled component trees
- Consider compile-time validation for static views
- Use lazy rendering for complex pages
**Warning signs:** Slow page loads despite simple content

### Pitfall 3: Versioning Nightmares
**What goes wrong:** Schema changes break existing views
**Why it happens:** No migration strategy for schema updates
**How to avoid:**
- Additive changes only (new optional fields)
- Deprecation period for breaking changes
- Validate against multiple schema versions during transition
**Warning signs:** "It worked yesterday" bugs

### Pitfall 4: Offline/Error State Gaps
**What goes wrong:** JSON-UI pages fail when data loading fails
**Why it happens:** No fallback rendering strategy
**How to avoid:**
- Define error states in schema
- Provide loading skeletons
- Graceful degradation for missing data
**Warning signs:** Blank pages on API errors

### Pitfall 5: Over-Engineering the Schema
**What goes wrong:** Schema becomes so flexible it's harder than code
**Why it happens:** Trying to support every possible UI pattern
**How to avoid:**
- Start with 10 core components only
- Add components based on real usage
- Escape hatch: allow Inertia for complex custom UIs
**Warning signs:** Schema spec document longer than implementation
</common_pitfalls>

<code_examples>
## Code Examples

### JSON-UI View Definition
```json
{
  "$schema": "ferro-json-ui/v1",
  "layout": "app",
  "title": "Users",
  "components": [
    {
      "key": "header",
      "type": "Card",
      "props": { "title": "User Management" },
      "children": [
        {
          "key": "create-btn",
          "type": "Button",
          "props": { "label": "Create User", "variant": "primary" },
          "action": { "handler": "users.create" }
        }
      ]
    },
    {
      "key": "users-table",
      "type": "Table",
      "props": {
        "columns": [
          { "key": "name", "label": "Name" },
          { "key": "email", "label": "Email" },
          { "key": "created_at", "label": "Created", "format": "date" }
        ],
        "dataPath": "/data/users",
        "rowActions": [
          { "name": "edit", "handler": "users.edit", "icon": "pencil" },
          {
            "name": "delete",
            "handler": "users.destroy",
            "icon": "trash",
            "confirm": { "title": "Delete?", "variant": "danger" }
          }
        ]
      }
    }
  ]
}
```

### Rust Component Types
```rust
// Source: Pattern from json-render, adapted for Rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Component {
    Card(CardProps),
    Table(TableProps),
    Form(FormProps),
    Button(ButtonProps),
    Input(InputProps),
    Alert(AlertProps),
    Badge(BadgeProps),
    Modal(ModalProps),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CardProps {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub children: Vec<Component>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Action {
    pub name: String,
    pub handler: String,  // "controller.method" format
    #[serde(default)]
    pub method: HttpMethod,
    #[serde(default)]
    pub confirm: Option<ConfirmDialog>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfirmDialog {
    pub title: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub variant: DialogVariant,
}
```

### Renderer Structure
```rust
// Source: Pattern from ferro-inertia/src/response.rs
pub struct JsonUiRenderer;

impl JsonUiRenderer {
    pub fn render(view: &JsonUiView, data: &serde_json::Value) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html>");
        html.push_str(&Self::render_head(&view.title));
        html.push_str("<body class=\"bg-background text-foreground\">");

        for component in &view.components {
            html.push_str(&Self::render_component(component, data));
        }

        html.push_str("</body></html>");
        html
    }

    fn render_component(component: &Component, data: &serde_json::Value) -> String {
        match component {
            Component::Card(props) => Self::render_card(props, data),
            Component::Table(props) => Self::render_table(props, data),
            // ...
        }
    }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Generic JSON to JS client | Typed schemas with validation | 2024 (json-render) | Predictable, safe AI generation |
| Custom component systems | shadcn/ui-style catalogs | 2023-2024 | Consistent, themeable components |
| Server-rendered templates | JSON → HTML on server | Emerging | No JS build required |

**New tools/patterns to consider:**
- **json-render by Vercel:** AI-safe UI generation with guardrails - directly relevant
- **shadcn/ui patterns:** Variant system, CSS variables, composition - use for component design
- **Tailwind v4:** CSS-first config - watch for breaking changes

**Deprecated/outdated:**
- **Arbitrary JSON shapes:** Use typed schemas with validation
- **Client-side only SDUI:** Server-side rendering eliminates JS bundle requirement
</sota_updates>

<open_questions>
## Open Questions

1. **JSONPath library choice**
   - What we know: serde_json_path and jsonpath-rust both exist
   - What's unclear: Which has better performance/API for Ferro's use case
   - Recommendation: Evaluate both during Phase 25 (Data Binding)

2. **Coexistence with Inertia**
   - What we know: JSON-UI is meant to coexist, not replace
   - What's unclear: How to share layouts between systems
   - Recommendation: Define clear boundary - JSON-UI for CRUD, Inertia for custom

3. **Form handling complexity**
   - What we know: Forms need validation, error display, redirects
   - What's unclear: How much of Ferro's existing validation to expose
   - Recommendation: Start simple (display errors), evolve based on usage
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [Vercel json-render GitHub](https://github.com/vercel-labs/json-render) - Schema design, component catalog, actions
- [json-render.dev](https://json-render.dev/) - Documentation, visibility rules
- [JSON Forms](https://jsonforms.io/docs/uischema/) - Data/UI schema separation
- [Maud documentation](https://maud.lambda.xyz/) - Rust HTML generation

### Secondary (MEDIUM confidence)
- [Askama GitHub](https://github.com/askama-rs/askama) - Rust templating alternative
- [shadcn/ui](https://ui.shadcn.com/) - Component design patterns (verified via Vercel Academy)
- [Nativeblocks SDUI best practices](https://nativeblocks.io/blog/best-practices-and-common-pitfalls/) - Common pitfalls

### Tertiary (LOW confidence - needs validation)
- SDUI patterns from Medium articles - General patterns, verify during implementation
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: JSON Schema for UI definition
- Ecosystem: json-render, JSON Forms, SDUI patterns
- Patterns: Component catalog, action declarations, visibility rules
- Pitfalls: Versioning, performance, over-engineering

**Confidence breakdown:**
- Standard stack: HIGH - json-render is well-documented, Rust options clear
- Architecture: HIGH - Patterns verified across multiple implementations
- Pitfalls: HIGH - Documented in Nativeblocks, verified in discussions
- Code examples: MEDIUM - Adapted from references, needs validation in Ferro

**Research date:** 2026-01-16
**Valid until:** 2026-02-16 (30 days - SDUI patterns stable)
</metadata>

---

*Phase: 23-json-ui-schema*
*Research completed: 2026-01-16*
*Ready for planning: yes*
