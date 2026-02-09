# Phase 32: Documentation - Research

**Researched:** 2026-02-09
**Domain:** JSON-UI feature documentation for Ferro framework (mdBook)
**Confidence:** HIGH

<research_summary>
## Summary

Researched documentation strategy for Ferro's JSON-UI system — a server-driven UI component catalog with 20 components, action system, visibility rules, data binding, layout system, and HTML rendering. The docs site uses mdBook, deployed at docs.ferro-rs.dev.

The primary audience is dual: **AI agents** (who consume MCP tools and generate code) and **developers** (who read docs to understand the system). Existing MCP tools already provide comprehensive AI-facing documentation (component catalog, code templates, generation context). Phase 32's focus should be developer-facing guides and reference in mdBook.

**Primary recommendation:** Add a "JSON-UI" section to the existing mdBook docs with 4-5 pages following the established documentation pattern (introduction → component examples → systems → reference). Reuse the component catalog structure from MCP tools as the API reference. Keep it concise — the system is well-designed and the code examples speak for themselves.
</research_summary>

<standard_stack>
## Standard Stack

### Core (already in place)
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| mdBook | current | Documentation site | Already used for docs.ferro-rs.dev |
| Vercel | - | Deployment | Already configured (docs/vercel.json) |
| Tailwind CDN | - | Examples rendering | JSON-UI uses Tailwind classes |

### Supporting (no new deps needed)
| Tool | Purpose | Status |
|------|---------|--------|
| `cargo doc` | Rust API reference | Already works via rustdoc comments |
| MCP tools | Agent-facing docs | json_ui_catalog, json_ui_generate, json_ui_inspect already exist |
| `ferro make:json-view` | CLI scaffolding | Already provides AI-powered view generation |

### No New Dependencies
Phase 32 requires zero new libraries. mdBook is already set up. The documentation is pure Markdown content creation.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Documentation Structure (follows existing pattern)

The existing docs follow a consistent hierarchy:
```
docs/src/
├── SUMMARY.md              # Table of contents
├── introduction.md         # Overview
├── getting-started/        # Onboarding
├── the-basics/             # Core concepts
├── features/               # Feature deep-dives (10 pages)
│   ├── database.md
│   ├── validation.md
│   ├── inertia.md          # Current UI system docs
│   └── json-ui.md          # NEW: JSON-UI feature docs
├── json-ui/                # NEW: JSON-UI section
│   ├── getting-started.md  # Quick start with JSON-UI
│   ├── components.md       # Component reference (all 20)
│   ├── actions.md          # Action system docs
│   ├── data-binding.md     # Data paths and visibility
│   └── layouts.md          # Layout system and customization
└── reference/
    └── cli.md              # CLI reference (update with make:json-view)
```

### Page Structure Pattern (from existing docs analysis)

Every feature page follows:
1. **H1**: Feature name + 1-2 sentence description
2. **How it works**: Brief conceptual overview
3. **Basic Usage**: Simplest working example with imports
4. **Detailed Examples**: Real-world scenarios with progressive complexity
5. **Advanced Patterns**: Complex use cases
6. **API Reference**: Props tables, method signatures

### Code Example Conventions (from existing docs)
- Always include `use ferro_rs::...` imports
- Show realistic data (user management, not foo/bar)
- Both Rust builder code AND equivalent JSON shown
- Comments only where non-obvious
- Progressive complexity: minimal → full-featured

### Dual Format: Rust + JSON

JSON-UI is unique in requiring dual code examples:
```rust
// Rust builder API
let view = JsonUiView::new()
    .title("Users")
    .component(ComponentNode { ... });
```

```json
// Equivalent JSON
{
  "$schema": "ferro-json-ui/v1",
  "title": "Users",
  "components": [{ "key": "...", "type": "Card", ... }]
}
```

Both representations are needed since agents may generate either.

### Component Reference Pattern (from shadcn/ui research)

Each component documented with:
1. Name + one-sentence description
2. Props table (name, type, required, default, description)
3. Variants list with visual meaning
4. Minimal code example
5. JSON equivalent
6. Notes (default behavior, common patterns)

### Anti-Patterns to Avoid
- **Duplicating MCP catalog as docs**: MCP tools already serve AI agents. Docs target human developers.
- **Documenting every test case**: Show patterns, not exhaustive combinations.
- **Separate page per component**: 20 components don't need 20 pages. Group logically (display, form, navigation, utility).
- **Screenshots**: Server-rendered HTML changes with Tailwind versions. Code examples are more durable.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| API reference tables | Manual markdown tables | Extract from existing `CatalogComponent` structs in MCP | Catalog already has all 20 components with props, types, required flags |
| Code examples | New examples from scratch | Adapt from existing test cases in ferro-json-ui | Tests already cover all components with verified working code |
| Getting started guide | New tutorial from scratch | Adapt from `json_ui_generate.rs` VIEW_EXAMPLE | The view example used for AI generation is already a perfect tutorial |
| Component descriptions | Rewrite descriptions | Reuse from `json_ui_catalog.rs` CatalogComponent | Already human-readable descriptions for all 20 components |
| CLI docs | New docs | Extend existing `reference/cli.md` | `make:json-view` follows same pattern as other CLI commands |

**Key insight:** The MCP tooling (Phase 31) already created excellent documentation artifacts — component catalog, code examples, conventions, templates. Phase 32 should transform these into mdBook format, not recreate them.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Documentation Scope Creep
**What goes wrong:** Writing tutorial-level docs for every component variant and prop combination
**Why it happens:** 20 components x many props = temptation to document everything
**How to avoid:** Component reference is a table, not a tutorial. Show 1 example per component.
**Warning signs:** Any component section exceeding 30 lines

### Pitfall 2: Inconsistent with Existing Docs Style
**What goes wrong:** JSON-UI docs look different from the rest of the documentation
**Why it happens:** Not referencing existing patterns in validation.md, inertia.md, etc.
**How to avoid:** Use the established page template: intro → basic usage → examples → reference
**Warning signs:** Different heading structure, different code block style, different import patterns

### Pitfall 3: Missing the JSON Format
**What goes wrong:** Only showing Rust builder API, not the JSON equivalent
**Why it happens:** Developer naturally writes Rust examples
**How to avoid:** Every Rust example should have a JSON equivalent or at least mention JSON output
**Warning signs:** Page has zero JSON code blocks

### Pitfall 4: Not Updating SUMMARY.md
**What goes wrong:** Pages exist but aren't discoverable in navigation
**Why it happens:** Forgetting to update the table of contents
**How to avoid:** Update SUMMARY.md as the first step when adding any new page
**Warning signs:** New .md files not linked in sidebar

### Pitfall 5: Stale Examples After API Changes
**What goes wrong:** Examples compile but don't reflect current API
**Why it happens:** CardProps has `footer` field that earlier docs might miss
**How to avoid:** Examples should match actual struct definitions in component.rs
**Warning signs:** Examples that omit required fields or use deprecated patterns
</common_pitfalls>

<code_examples>
## Code Examples

Verified patterns from the codebase:

### Basic Handler (from framework/src/json_ui/mod.rs)
```rust
use ferro_rs::{handler, JsonUi, JsonUiView, ComponentNode, Component, CardProps, Response};

#[handler]
pub async fn index() -> Response {
    let view = JsonUiView::new()
        .title("Dashboard")
        .layout("app")
        .component(ComponentNode {
            key: "welcome".to_string(),
            component: Component::Card(CardProps {
                title: "Welcome".to_string(),
                description: Some("Your dashboard".to_string()),
                children: vec![],
                footer: vec![],
            }),
            action: None,
            visibility: None,
        });

    JsonUi::render(&view, &serde_json::json!({}))
}
```

### Form with Validation (from framework/src/json_ui/mod.rs tests)
```rust
use ferro_rs::{
    handler, JsonUi, JsonUiView, ComponentNode, Component,
    FormProps, InputProps, InputType, Action, HttpMethod, Response,
};

pub fn create_form() -> JsonUiView {
    JsonUiView::new()
        .title("Create User")
        .layout("app")
        .component(ComponentNode {
            key: "form".to_string(),
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
                            ..Default::default() // won't work, but shows intent
                        }),
                        action: None,
                        visibility: None,
                    },
                ],
                method: None,
            }),
            action: None,
            visibility: None,
        })
}
```

### Action Builder API (from ferro-json-ui/src/action.rs)
```rust
// Navigation
Action::get("users.index")

// Form submission
Action::new("users.store")

// Destructive with confirmation
Action::delete("users.destroy")
    .confirm_danger("Delete user?")
    .on_success(ActionOutcome::Redirect { url: "/users".to_string() })
```

### Data Binding (from ferro-json-ui/src/data.rs)
```rust
// Handler data
let data = serde_json::json!({
    "user": {"name": "Alice", "email": "alice@example.com"}
});

// Input with data_path pre-fills from handler data
Component::Input(InputProps {
    field: "name".to_string(),
    label: "Name".to_string(),
    data_path: Some("/data/user/name".to_string()),
    ..
})
```

### Visibility Rules (from ferro-json-ui/src/visibility.rs)
```json
// Simple condition
{"path": "/data/users", "operator": "not_empty"}

// Compound (AND)
{"and": [
    {"path": "/auth/user/role", "operator": "eq", "value": "admin"},
    {"path": "/data/users", "operator": "not_empty"}
]}

// Negation
{"not": {"path": "/data/is_deleted", "operator": "eq", "value": true}}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Storybook for component docs | Component catalog in code + mdBook | 2026 (Ferro v3.0) | No separate tooling needed |
| Interactive playgrounds | JSON examples + server rendering | 2026 | Simpler, no JS build step |
| Per-component doc pages | Grouped component reference | 2025+ | Less navigation, faster scanning |

**Documentation as Code:**
- Component catalog already defined as Rust structs (`CatalogComponent` in MCP)
- Props, types, variants are all derivable from source code
- mdBook docs complement this with narrative guides and examples

**Agent-First Documentation:**
- MCP tools serve as primary documentation for AI agents
- Human docs focus on concepts, patterns, and decision-making
- Both audiences need component reference, but in different formats

**No New Patterns Needed:**
- mdBook is stable and well-suited
- Ferro's existing doc structure is clean and consistent
- shadcn/ui's documentation pattern (name → props table → examples) maps directly
</sota_updates>

<open_questions>
## Open Questions

1. **Should JSON-UI get its own section or a single feature page?**
   - What we know: Existing features (events, queues, etc.) are single pages. Inertia is a single page. JSON-UI has more surface area (20 components, actions, visibility, layouts).
   - What's unclear: Whether a single page is sufficient or a dedicated section is needed
   - Recommendation: Start with a dedicated section (4-5 pages). A single page for 20 components + 4 subsystems would be overwhelming.

2. **Should we rebuild `docs/book/` (the generated output)?**
   - What we know: Built HTML exists at `docs/book/`. It's checked into the repo.
   - What's unclear: Whether the user rebuilds and commits, or if CI handles it
   - Recommendation: Write the markdown, defer the build to the user. They can `mdbook build` when ready.

3. **How deep should component examples go?**
   - What we know: 20 components range from simple (Separator, Badge) to complex (Table, Form, Tabs)
   - What's unclear: Whether every optional prop needs demonstration
   - Recommendation: Show required props + most useful optionals. Complex components (Table, Form) get detailed examples. Simple ones (Badge, Separator) get one-liners.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- `ferro-json-ui/src/component.rs` - All 20 component types with props (verified, line-by-line review)
- `ferro-json-ui/src/action.rs` - Action system with builder API (verified)
- `ferro-json-ui/src/view.rs` - View structure and builder (verified)
- `ferro-json-ui/src/visibility.rs` - Visibility rules (verified)
- `ferro-json-ui/src/data.rs` - Data path resolution (verified)
- `ferro-json-ui/src/render.rs` - HTML render engine (verified)
- `ferro-json-ui/src/layout.rs` - Layout system (verified)
- `ferro-json-ui/src/config.rs` - Render configuration (verified)
- `framework/src/json_ui/mod.rs` - Framework integration, 778 lines (verified)
- `framework/src/lib.rs` - Public re-exports (verified)
- `ferro-mcp/src/tools/json_ui_catalog.rs` - Component catalog for MCP (verified)
- `ferro-mcp/src/tools/json_ui_generate.rs` - Generation context with examples (verified)
- `ferro-cli/src/ai.rs` - CLI AI context with compact catalog (verified)
- `docs/src/SUMMARY.md` - Current doc structure (verified)
- `docs/src/features/inertia.md` - Pattern for UI feature docs (verified)
- `docs/book.toml` - mdBook configuration (verified)

### Secondary (MEDIUM confidence)
- shadcn/ui component documentation pattern (button page structure) - verified via WebFetch
- Leptos Book documentation structure - verified via WebFetch

### Tertiary (LOW confidence - needs validation)
- None. All findings are from primary sources (codebase) or verified secondary sources.
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: mdBook documentation for Ferro JSON-UI
- Ecosystem: Existing docs infrastructure, MCP tooling, CLI scaffolding
- Patterns: Component documentation, feature guide structure, dual-format examples
- Pitfalls: Scope creep, consistency, missing JSON format, stale examples

**Confidence breakdown:**
- Standard stack: HIGH - no new tools, using existing mdBook
- Architecture: HIGH - follows established patterns in existing docs
- Pitfalls: HIGH - derived from analyzing existing docs and component catalog
- Code examples: HIGH - all from verified codebase sources

**Research date:** 2026-02-09
**Valid until:** 2026-03-09 (30 days - documentation patterns are stable)
</metadata>

---

*Phase: 32-documentation*
*Research completed: 2026-02-09*
*Ready for planning: yes*
