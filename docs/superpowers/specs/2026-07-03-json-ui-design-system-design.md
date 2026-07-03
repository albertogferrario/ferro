# JSON-UI Design System — Design

**Date:** 2026-07-03
**Status:** Approved
**Scope:** ferro-theme, ferro-json-ui, ferro-cli, ferro-mcp, docs; consumer adoption phase in the reference application

## Overview

ferro-json-ui already has the lower half of a design system: a fixed 23-slot semantic
token vocabulary (`ferro-theme/v1`), 47 builtin components that emit semantic classes
exclusively, a generated `ferro-base.css`, and per-request theme resolution via
`ThemeMiddleware`. What is missing is everything above the token layer:

- no density, motion, or focus-ring tokens;
- no canonical variant vocabulary across components (`variant`/`tone`/`size` naming
  drifts between components);
- composition patterns (page anatomy, list/kanban/form conventions) exist only as
  informal guidance, not as a versioned, machine-readable artifact;
- nothing lets an agent authoring a spec conform to the system by construction.

This design completes the system. The primary capability: **the design system is
enforced at the agent-authoring boundary** — an agent reads the system through
ferro-mcp, authors a spec, and a design lint validates conformance before any human
review. Pattern rules are keyed by the existing seven projection intents, so the
design system extends the framework's core abstraction rather than adding a parallel
vocabulary.

## 1. Token vocabulary v2 (ferro-theme)

The vocabulary grows from 23 to 30 slots. Every new slot ships with a default in the
base CSS and default theme, so **every valid v1 theme remains a valid v2 theme**
without changes.

| Category | New tokens | Purpose |
|---|---|---|
| Density | `--spacing` | Tailwind v4 base spacing unit. One token rescales all padding/margin/gap utilities — single-knob density control. |
| Motion | `--motion-duration-fast`, `--motion-duration-base`, `--motion-duration-slow`, `--motion-ease` | Consistent transitions across components (`duration-base`, `ease-base` utilities via `@theme`). Base CSS honors `prefers-reduced-motion` by collapsing durations. |
| Focus ring | `--color-ring` | Uniform `focus-visible` ring on every interactive component. Accessibility as a token. |
| Typography | `--font-display` | Heading/display font family; defaults to `var(--font-sans)`. |

Deliberate exclusions:

- **Type scale slots.** Tailwind text sizes are rem-based; a theme scales all text by
  setting root `font-size` in its `tokens.css`. Documented as the supported mechanism —
  adding per-size tokens would duplicate an existing control surface.
- **Font weight tokens.** No demonstrated theming need.

Changes:

- `ferro-theme/src/token.rs`: 7 new constants, `ALL_TOKENS` → 30, doc header updated to
  `ferro-theme/v2`.
- `ferro-json-ui/assets/input.css`: `@theme inline` additions mapping the new slots to
  utilities (`duration-fast/base/slow`, `ease-base`, ring color, `font-display`,
  spacing base); regenerate `ferro-base.css`.
- `ferro-theme/assets/default.css`: defaults for all new slots (light + dark).
- `docs/src/features/themes.md`: v2 token reference, migration note (none required),
  root-font-size type-scaling recipe.

## 2. Component variant discipline (ferro-json-ui)

Audit all 47 builtin components and normalize prop vocabulary to canonical enums,
enforced by the catalog's schemars-generated prop schemas:

- **`variant`** — visual weight of interactive elements:
  `primary | secondary | outline | ghost | destructive`.
- **`tone`** — semantic status color for stateful display components (Badge, Alert,
  Toast, StatCard, CalendarCell, …): `neutral | success | warning | destructive`.
- **`size`** — `sm | md | lg` wherever a size prop exists.

Components whose props deviate are renamed to the canonical vocabulary. Pre-1.0,
breaking renames are acceptable; a migration table in the phase notes lists every
rename for consumers.

Interactive-state pass across all components:

- hover treatment present and consistent;
- `focus-visible` ring using `--color-ring`;
- disabled state (opacity + `pointer-events`) consistent;
- transitions use the motion tokens.

`ferro-base.css` is regenerated after class changes; catalog drift guards extend to
the canonical enum sets.

## 3. Pattern layer — `design` module in ferro-json-ui

### Spec extension

`Spec` gains one optional field (serde-default, absent from serialized output when
unset):

```json
"design": {
  "intent": "browse",
  "allow": ["prefer-data-table"]
}
```

- `intent` — one of the seven projection intents (`browse`, `focus`, `collect`,
  `process`, `summarize`, `analyze`, `track`). Declares the page archetype. When
  absent, lint infers the intent from spec content (DataTable → browse, KanbanBoard →
  process, root Form → collect, StatCard cluster → summarize, …) and reports the
  inference as an info-level finding so authors can confirm or declare.
- `allow` — rule ids exempted for this page; the per-page escape hatch. Unknown ids
  are themselves reported as findings.

### Rule engine

`ferro-json-ui/src/design/` module:

```rust
pub struct DesignRule {
    pub id: &'static str,        // "prefer-data-table"
    pub title: &'static str,
    pub rationale: &'static str,
    pub intents: &'static [Intent], // empty = all intents
    // check(&Spec, resolved_intent) -> Vec<Finding>
}

pub struct Finding {
    pub rule: &'static str,
    pub element_id: Option<String>,
    pub severity: Severity,       // Info | Warning
    pub message: String,
    pub suggestion: String,
}

pub fn lint(spec: &Spec) -> Vec<Finding>;
```

Lint is a pure diagnostic pass: it never affects rendering or validation. Findings are
warnings (or info for inferences); the only failure mode is the opt-in `--deny` CLI
flag.

### Initial rule set (~10 rules)

| id | intents | Rule |
|---|---|---|
| `page-header` | all | Pages whose `spec.layout` is a dashboard-family layout start with a `PageHeader` carrying a title. |
| `prefer-data-table` | browse | Raw `Table` discouraged for entity lists; use `DataTable` (responsive, mobile cards). |
| `list-empty-state` | browse | List pages define an empty state (`EmptyState` component or `DataTable` empty config) with a create CTA. |
| `row-actions-grouped` | browse, process | Per-row/per-card actions live in an `ActionGroup`, not loose inline `Button`s. |
| `process-kanban` | process | Status-workflow pages use `KanbanBoard` with column count badges. |
| `create-separate-page` | collect | Entity creation is a dedicated page; `Modal` containing a `Form` is flagged. |
| `breadcrumb-on-subpages` | collect, focus | Create/edit/detail pages include a `Breadcrumb` back to the list page. |
| `form-default-values` | collect | When any form field binds `default_value` via a `$data` path (the page is an edit form), sibling fields lacking a `default_value` are flagged. Pure create forms (no bindings anywhere) produce no findings. |
| `destructive-confirmation` | all | Actions styled destructive carry a confirmation behavior. |
| `card-actions-in-menu` | process | Kanban card actions: detail action first, destructive actions last, all inside the `ActionGroup`. |

Each rule ships with a unit-test pair: one violating spec (finding produced) and one
conforming spec (no finding).

## 4. Surfaces

### ferro-cli

`ferro design:lint [path] [--json] [--deny]`

- default path `src/views`, recursive over `*.json` specs;
- human-readable findings grouped by file; `--json` for machine consumption;
- exit 0 always, unless `--deny` (CI mode: non-zero when any warning-level finding
  exists).

### ferro-mcp

- New tool `design_lint`: accepts a spec (inline JSON or path), returns structured
  findings. Closes the author→validate loop inside the agent session.
- `json_ui_catalog`: extended with the canonical variant vocabulary and per-component
  design guidance.
- `generation_context`: gains a design-system summary — token vocabulary, per-intent
  pattern expectations — so agents author conformant specs on the first pass.

### Documentation

New chapter `docs/src/design-system/`:

- principles (semantic tokens, intent-keyed patterns, lint as diagnostics);
- token v2 reference;
- variant vocabulary;
- pattern catalog (each rule: rationale, example, how to `allow`);
- linting guide (CLI + MCP);
- theming updates cross-linked with `features/themes.md`.

## 5. Delivery plan (ferro)

Four phases, one publish at the end (mid-stream publishes would freeze the API before
consumer feedback can revise it):

1. **Tokens v2 + default theme** — token constants, `input.css` `@theme` additions,
   `ferro-base.css` regeneration, `default.css` refresh, themes docs.
2. **Variant discipline** — 47-component audit, canonical enums, interactive-state and
   motion pass, catalog + drift guards, migration table.
3. **Design module + lint** — `design::lint`, rule set, `Spec.design` field,
   `ferro design:lint` CLI command.
4. **MCP surface + docs + publish** — `design_lint` tool, catalog/generation-context
   extensions, `docs/src/design-system/`, version bump, crates.io publish.

## 6. Reference-case adoption (gestiscilo)

A consumer adoption phase is created in the reference application's repository
(gestiscilo, 68 json-ui specs across 16 domains), gated on the ferro publish in phase
4:

- pin the new ferro / ferro-json-ui versions; absorb variant renames from the
  migration table;
- extend `themes/gestiscilo/tokens.css` with v2 values (motion, ring, density,
  display font);
- declare `design.intent` on all 68 specs; run `ferro design:lint` across them; fix
  every finding or `allow` it with a one-line justification;
- visual verification (browser automation) on one page per archetype — list, kanban,
  form, detail, dashboard — checked against sibling pages for consistency;
- wire `ferro design:lint --deny` into CI so the application stays lint-clean;
- produce a FRICTION.md capturing rules that misfired, missing rules, and token gaps,
  feeding the next ferro iteration.

The friction report is what makes the consumer a reference case rather than only a
consumer: the rule set is validated against 68 real screens before the design system
is considered complete.

## 7. Testing & error handling

- Per-rule unit-test pairs in `ferro-json-ui/src/design/`.
- Catalog drift guards extended to variant enums (single source in ferro-json-ui;
  ferro-mcp count remains a documented mirror).
- Sample `app/` views must lint clean — enforced by a test.
- ferro-mcp tool tests for `design_lint` (inline spec + path input).
- Lint never fails rendering; invalid `design.intent` values and unknown `allow` ids
  are findings, not errors.
- Full CI-exact gate before the publish push: `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, docs build.

## 8. Non-goals

- No new crate: pattern rules live in ferro-json-ui, which already owns spec
  validation and the catalog; tokens stay in ferro-theme.
- No hard validation: rendering never rejects a spec on design grounds.
- No per-size type tokens or font-weight tokens (see exclusions above).
- No retroactive redesign of intent templates (`ThemeTemplates`) — unchanged surface.
