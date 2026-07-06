# Phase 164: JSON-UI improvements batch 3 — V7-RUNTIME frictions, v1-deletion-readiness audit, COMPLETED.md — Research

**Researched:** 2026-05-17
**Domain:** ferro-json-ui v2 spec format, catalog validation pipeline, plugin surface, CLI codemod, MCP introspection, v12.0 closing-batch audit and documentation
**Confidence:** HIGH (every implementation site verified against the source tree on `v12.0/json-ui-v2` at commit `ce44ac77`)

## Summary

Phase 164 is the **closing batch** of the v12.0 friction loop. It absorbs three input streams into one phase: (1) eight ferro-side fixes from `V7-RUNTIME-FRICTION.md` (D-12..D-19), (2) residual compile-time items from Phase 138 `FRICTION.md` not absorbed by 162/163/163.1, (3) the v1-deletion-readiness audit gating Phase 160, and (4) the `COMPLETED.md` summary that feeds Phase 160's planner and Phase 161's CHANGELOG.

Every D-12..D-19 implementation site has been verified against the source tree. The architectural decisions are well-scoped: D-12/D-13/D-15 are mechanical struct-field additions; D-14 is a single-constant bump; D-18 ships full Rust code in the friction file; D-19 splits cleanly into three independent fixes (codemod tweak, error-message improvement, optional `None` acceptance). The two architecturally meaningful decisions are D-16 (validation pipeline reorder — affects `Catalog::validate` callers across the framework) and D-17 (`Component::RawHtml` reintroduction — needs careful framing because Phase 115 D-01 explicitly killed the v1 `Component::Plugin` escape hatch).

**Primary recommendation:** Ship D-12, D-13a+b, D-14, D-15, D-17a, D-18, D-19-F5 verbatim per CONTEXT defaults. Treat D-16 as the heaviest plan (architectural pipeline rework) and gate it behind a dedicated test fixture (Alert.variant=`""` with `visible: { exists: /flash_message }` gating). For the v1-deletion audit, since the v1 surface is already deleted on the branch (verified — `view.rs` absent, `JsonUiView`/`Component`/`ComponentNode`/`PluginProps` all removed in commit `dbe5adaf`), the audit reduces to a **gap-coverage table** proving every removed v1 capability has a documented v2 path.

## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-12 (F1) — Allow `$data` bindings on `Spec.title`.** Change `Spec.title: Option<String>` to accept either a literal `String` or an expression binding (`{"$data": "/path"}`). Implementation choice at planning time: introduce a `TitleBinding` enum mirroring how `Element.props` accept bindings, or generalise the binding resolution. Touches `ferro-json-ui/src/spec.rs` and the renderer's title emission path. Closes a 23-spec authoring constraint.

**D-13 (F3) — Add `data_path` to `KanbanBoardProps`.** Currently `columns: Vec<KanbanColumnProps>` must be inlined statically. Two implementation options for the planner:
- **D-13a:** Add `data_path: Option<String>` to `KanbanBoardProps` + a column factory pattern (`column_template` + grouping key) that emits columns from a JSON array at runtime.
- **D-13b:** Document the existing `$each` directive (Phase 163) as the way to template kanban columns from a data path; add a worked example to `docs/src/json-ui/expressions.md`.
Default: ship D-13a (`data_path`) as the primary path; document D-13b as the templated alternative. D-13a aligns with `DataTable`'s pattern and removes the need to nest a directive inside the spec for a common use case.

**D-14 (F4) — Raise `MAX_NESTING_DEPTH` from 3 to 5.** Real-world dashboard hit depth 4 (root → grid → card → badge). Implementation: change `pub const MAX_NESTING_DEPTH: usize = 3` to `5` in `ferro-json-ui/src/spec.rs:37`; update the test at line 1705 and any callers; consider warn-only at depth 6 if a soft cap is desired. Document the constraint in `docs/src/json-ui/spec-construction.md`.

**D-15 (F7) — Add `data_path` to `ImageProps` and `DescriptionListProps`.** Both currently enforce static fields (`src` for Image, `items` for DescriptionList). Add `data_path: Option<String>` that, when present, resolves the dynamic value from request data and overrides the static field. Touches `ferro-json-ui/src/component.rs` and the respective renderers.

**D-16 (F8) — Validate after `expand_directives` + visibility.** Reorder the pipeline: `parse → expand_directives → apply_visibility → validate`. Recommended: full deferral of enum-shape validation; structural validation (element references, footer IDs, depth) remains at parse-time.

**D-17 (F9) — Resolve `Plugin` component type.** Default: **D-17a** (`Component::RawHtml`) — a server-injected HTML island carrying sanitised HTML in props. Document the v2 plugin-registered path (D-17b) as the recommended alternative for richer widgets.

**D-18 (F10) — Add `CardVariant` enum to `CardProps`.** Verbatim from V7-RUNTIME-FRICTION §F10 lines 109–138:
- `CardVariant::Bordered` (default): `border border-border bg-card shadow-sm overflow-visible` + `p-4`. Current dashboard look.
- `CardVariant::Elevated`: `rounded-lg bg-card shadow-md overflow-visible` + `p-8`. Auth pages, error pages, standalone marketing cards.
- `#[serde(default)]` on the `variant` field; serde rename `lowercase`.

**D-19 — Cross-repo coordination for gestiscilo-side items (F5, F6) + F2.**
- **F2 ferro codemod**: extend `ferro json-ui:migrate-v1` codemod (Phase 163.x) to upper-case HTTP method values on emission (already does — verify).
- **F5 ferro error message**: improve the `Visibility` untagged-enum parse error to name the rejected variant shape.
- **F6 ferro `PageHeader.actions`**: consider accepting `actions: None` (lower priority; planner decides).

**D-01..D-03 (v1-deletion-readiness audit):** Run a sweep over the v1 public surface and produce a `V1-DELETION-AUDIT.md`. Resolution column: `MIGRATED` / `INTENTIONAL_DROP` / `BLOCKER`. Phase 160 gates on zero `BLOCKER` rows.

**D-04 — Surface validator errors and warnings via ferro-mcp.** Either new tool (e.g. `json_ui_validate_spec`) or extension of existing tool.

**D-05 — Add validator coverage for Phase 163 directives:** `$each.path` resolves to a JSON array (already done — `SpecError::EachPathNotArray`), `$if.path` resolves cleanly (already done — `SpecError::IfPathMissing`), no circular references, no `children` references to absent elements unless gated by `$if`.

**D-06..D-07 — Plugin surface audit.** Paper exercise — verify a fresh plugin author could implement: (a) Stripe payment widget, (b) WhatsApp connection flow, (c) chart renderer.

**D-08..D-09 — Documentation pass.** Final sweep + new sections for D-12..D-18 + "v1 → v2 cheat sheet" at the top of `migration-v1-to-v2.md`.

**D-10..D-11 — COMPLETED.md.** Sections: Shipped across Phases 162-164 / Runtime frictions resolved / Intentional gaps / Deferred to future milestones / v1 → v2 surface migration table.

**Release cadence:** No mid-loop publish. Single publish at Phase 161.

### Claude's Discretion

- Exact column structure of D-01's audit table — planner picks.
- Whether D-04's MCP surface is a new tool or an extension of an existing tool — implementation choice.
- Whether D-06 audit produces a written artefact or is a verbal checkpoint with the user — depends on how many gaps it surfaces.
- D-13 implementation choice (data_path vs $each example) — planner picks; default is "ship both".
- D-16 sub-decisions (validation pipeline split, breaking-change ledger) — planner picks; default is "full deferral with structural validation retained at parse-time".
- D-17 implementation choice (RawHtml vs plugin-only vs Slot) — planner picks; default is D-17a (RawHtml) + document D-17b.

### Deferred Ideas (OUT OF SCOPE)

- Host-based tenancy gap — separate tenancy-layer phase; tracked in `.planning/backlog/host-based-tenancy.md`.
- `Fragment` / `Group` borderless container — Phase 162 D-06 explicitly rejected.
- `#[handler(name = "...")]` attribute — Phase 162 D-10 explicitly rejected.
- `$template` separate element — Phase 163 D-05 explicitly rejected.
- Codemod directory-recursive mode — Phase 163 D-10 explicitly rejected.
- Modal chrome sweep (related to D-18 question 2) — answer in friction file is "probably no action".
- Granular `padding`/`elevation` props on `CardProps` (D-18 question 3) — friction file recommendation: ship `variant` only.

## Project Constraints (from CLAUDE.md)

These directives apply to every plan and task in Phase 164. The planner must respect them; the executor MUST NOT recommend approaches that contradict them.

- **Pre-commit gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`. `--all-targets` is mandatory (catches test-code issues that `--all` alone misses). CI enforces `-D warnings`.
- **Repository documents read as neutral.** RESEARCH/PLAN/VERIFICATION docs must use neutral architectural voice. No "killer feature" / "the bet" / strategic framings. Internal voice goes in personal memory files only.
- **No co-author lines in commits.** No "Generated with Claude" attribution.
- **Project-agnostic crates.** No hardcoded app identity (`"gestiscilo"`, `"Ferro Application"`, `"https://example.com"`) in any `ferro-*` crate. CONTEXT cites gestiscilo as the friction source, but ferro-side fixes must not bake gestiscilo-specific strings into ferro code.
- **Update docs when framework changes.** `docs/src/json-ui/` MUST reflect every new field / variant / behaviour added by D-12..D-19.
- **Update ferro-mcp when surface changes.** New variants in `CardVariant`, new `data_path` props on Image/DescriptionList/KanbanBoard, new `RawHtml` component (D-17a if shipped) — all touch `ferro-mcp/src/tools/json_ui_catalog.rs` AND the 40-component assertion list at `ferro-mcp/src/tools/json_ui_catalog.rs:289` and `ferro-json-ui/src/render/mod.rs:530`.
- **Hunt for the killer feature.** Phase 164's killer feature is "the v12.0 surface is now field-validated end-to-end" — proven by COMPLETED.md showing zero `BLOCKER` rows. Plans that don't move that needle are commodity-tier and should be merged or descoped.

## Phase Requirements

This phase has no formal REQ-IDs in `.planning/REQUIREMENTS.md` (it is friction-driven, not requirements-driven). The source-of-truth artefact is `164-CONTEXT.md` decisions **D-12 through D-19** (V7-RUNTIME ferro-side fixes), **D-01 through D-03** (v1-deletion audit), **D-04..D-09** (validator polish + plugin surface audit + docs pass), and **D-10..D-11** (COMPLETED.md). Each decision maps to a concrete plan or set of plans. The planner is free to bundle small decisions into single plans (e.g. D-15a Image + D-15b DescriptionList in one plan; D-19 F5+F6 in one plan).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `Spec.title` accepts `{$data}` binding (D-12) | ferro-json-ui (spec types) | framework/json_ui (renderer's title emission) | Type lives in Spec; renderer reads `spec.title` at `framework/src/json_ui/mod.rs:89` |
| KanbanBoard `data_path` (D-13a) | ferro-json-ui (component) | ferro-json-ui (render/containers.rs) | Prop + renderer in same crate; no framework-tier touch |
| Document `$each` for kanban columns (D-13b) | docs | — | Pure documentation; expressions.md extension |
| Raise `MAX_NESTING_DEPTH` 3→5 (D-14) | ferro-json-ui (spec constant) | docs | One-constant change + test fixture; `framework/src/lib.rs:91` re-exports the constant |
| Image/DescriptionList `data_path` (D-15) | ferro-json-ui (component + render) | — | Mirror of D-13 pattern; localised to two render functions |
| Validation pipeline reorder (D-16) | ferro-json-ui (resolve.rs) + framework/json_ui (resolve callsite) | ferro-json-ui (catalog.rs — Stage 2/3 ordering relative to expand_directives) | Architectural change. Currently `load_cached` calls `from_json` (structural) → `catalog.validate` (enum). `expand_directives` runs LATER in `JsonUi::resolve`. Must invert: directive expansion before catalog enum check |
| `Component::RawHtml` / `Plugin` (D-17a) | ferro-json-ui (component + render + catalog) | ferro-mcp (catalog tool 40→41) | New built-in component; full integration path |
| `CardVariant` enum (D-18) | ferro-json-ui (component + render) | ferro-mcp (catalog schema regen) | Enum + render branch + schema |
| Codemod uppercase methods (D-19/F2) | ferro-cli (json_ui_migrate_v1.rs) | — | Already done (line 521) — VERIFY only, no change |
| Visibility error message (D-19/F5) | ferro-json-ui (visibility.rs Deserialize) | — | Custom Deserialize impl or visitor on the untagged enum |
| `PageHeader.actions` accepts None (D-19/F6) | ferro-json-ui (component) | — | Field type change `Vec<String>` → `Option<Vec<String>>` |
| MCP validate-spec tool (D-04) | ferro-mcp (tools/) | ferro-json-ui (re-export catalog errors) | New tool wrapping `global_catalog().validate()` and `Spec::from_json` |
| Directive validator coverage (D-05) | ferro-json-ui (spec.rs `validate_directives`) | — | Already partially shipped (Phase 163); audit for completeness |
| v1-deletion audit (D-01..D-03) | docs / planning | — | `V1-DELETION-AUDIT.md` artefact; no code change unless BLOCKER row surfaces |
| Plugin surface audit (D-06..D-07) | docs (plugins.md) | ferro-json-ui (plugin.rs — only if gap found) | Paper exercise; concrete code only if doc-gap blocks an exemplar |
| Docs pass (D-08..D-09) | docs | — | Cross-link sweep + cheat-sheet table |
| COMPLETED.md (D-10..D-11) | `.planning/phases/164-.../` | — | Phase artefact; input to Phase 160 + 161 |

## Standard Stack

### Core (already present in the workspace — no new deps)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | workspace pin | (de)serialization of `Spec`, `Element`, all `*Props` | Single (de)serialization stack across the workspace |
| `serde_json` | workspace pin | `Value` for type-erased props; codemod JSON emission | Workspace standard |
| `jsonschema` | workspace pin | per-component prop validation in `Catalog::validate` (`ferro-json-ui/src/catalog.rs:672`) | Pre-existing; D-04 MCP tool wraps the same validator |
| `schemars` | workspace pin | `#[derive(JsonSchema)]` on every `*Props` struct | Drives catalog schema generation; D-12/D-13/D-15/D-17/D-18 new fields must derive `JsonSchema` |
| `strum` | workspace pin | `AsRefStr` on variant enums (Phase 162 D-11) | `CardVariant` (D-18) MUST follow the same pattern |
| `thiserror` | workspace pin | `SpecError`, `CatalogError`, `LoadError` enums | New error variants (D-19/F5) follow existing pattern |
| `syn` | workspace pin | AST in `ferro-cli/src/commands/json_ui_migrate_v1.rs` | F2 codemod fix lives here (lines 520–528) |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `ammonia` | NOT in workspace | HTML sanitisation for D-17a `RawHtml` if server-side sanitisation is desired | **Recommendation: do NOT add.** `RichTextEditorProps` rustdoc (`component.rs:273`) already documents "sanitization on submit is the consumer's responsibility — handle this in the form handler before persisting (e.g. via `ammonia`)." Same discipline for `RawHtml`: doc the trust boundary, don't add a dep. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Component::RawHtml` (D-17a) | `Component::Slot` with server-side template lookup (D-17c) | Slot is more general but needs a registry + key-based lookup at render time. RawHtml is the smallest possible primitive that closes the F9 friction. Friction file `whatsapp.json` evidence shows gestiscilo emits `{html: {$data: "/owner_commands_html"}}` — pre-rendered HTML, no slot logic needed. |
| `Component::RawHtml` (D-17a) | Consumer migration to first-class `JsonUiPlugin` (D-17b) | Requires consumer to write `JsonUiPlugin` trait impls + register at startup. Heavy. Use as the recommended path for *richer* widgets (Stripe Connect status), not for simple HTML islands. |
| Full pipeline reorder (D-16) | Migrate consumers from `visible` to `$if` (Phase 163 directive) | Complementary path noted in V7-RUNTIME §F8 footnote. `$if` removes the element from the spec before validation, side-stepping the parse-order problem. BUT: doesn't fix the architectural inconsistency (validation runs against pre-resolution shape). D-16 is the architectural fix; `$if` is the consumer workaround. |

**Installation:** No new crate dependencies needed. All work is internal to `ferro-json-ui`, `ferro-mcp`, `ferro-cli`, and `docs/`.

**Version verification:** Workspace version remains 0.2.35 throughout Phase 164 per D-23/D-24 release cadence. Single publish at Phase 161.

## Architecture Patterns

### System Architecture Diagram

```
                                ┌─────────────────────────────────────────┐
JSON spec file                  │ ferro-json-ui                           │
    │                           │                                         │
    ▼                           │                                         │
[Spec::from_json]──────────────►│ validate_structure (D-05 +)             │
    │                           │   - validate_ids                        │
    │                           │   - validate_no_dangling                │
    │                           │   - validate_directives ($each/$if)     │
    │                           │   - validate_footer_ids (Phase 162 D-07)│
    │                           │   - check_depth (≤ MAX_NESTING_DEPTH)   │  ← D-14: 3 → 5
    │                           │                                         │
    │                  ┌────────┤ (parse-time structural validation)      │
    │                  │        └─────────────────────────────────────────┘
    ▼                  │
[loader::load_cached]  │                ┌──────────────────────────────────┐
    │                  ▼                │ Catalog::validate                │
    │            [Spec instance]───────►│   Stage 1: type_name whitelist   │
    │                  │                │   Stage 2: per-component schema  │
    │                  │                │     (jsonschema enum check)      │  ← D-16: REORDER (run AFTER expand_directives)
    │                  │                │   Stage 3: full-spec envelope    │
    │                  │                └──────────────────────────────────┘
    │                  │                          │
    │                  ▼                          │
    └────►[JsonUi::render_file]                   │
              │                                   │
              ▼                                   │
    ┌──────────────────────────────────┐          │
    │ JsonUi::resolve                  │          │
    │   1. expand_directives           │          │
    │      ($if remove + $each clone)  │          │
    │   2. resolve_actions             │          │
    │   3. resolve_expressions         │          │
    └──────────────────────────────────┘          │
              │                                   │  ← D-16 architectural fix:
              ▼                                   │     move catalog.validate
    ┌──────────────────────────────────┐          │     here, AFTER expand
    │ render_spec_to_html_with_plugins │◄─────────┘
    │   - render_element dispatch      │
    │   - per-component renderers      │  ← D-12/D-13/D-15/D-17/D-18 renderer edits
    │   - plugin asset collection      │
    └──────────────────────────────────┘
              │
              ▼
        HTML response

Boundaries:
  ferro-json-ui   = types + validation + rendering + plugin trait
  framework       = JsonUi facade + HTTP response wrapping
  ferro-cli       = codemod (D-19/F2) + scaffolder (untouched in 164)
  ferro-mcp       = catalog/inspect/validate tools (D-04)
  docs/src/json-ui = consumer-facing surface (D-08/D-09)
```

**Reading the diagram:** A `.json` spec file enters via `loader::load_cached` (used by `JsonUi::render_file`); parses through `Spec::from_json` (structural validation only); is then handed to `Catalog::validate` for per-component prop validation (THIS IS WHERE F8 TRIPS — Alert.variant=`""` fails before `expand_directives` runs). The directive expansion happens later, inside `JsonUi::resolve`. D-16 reorders this so `Catalog::validate` runs after `expand_directives`, letting `$if`-removed elements skip enum validation.

### Recommended Project Structure (delta from current)

```
ferro-json-ui/src/
├── spec.rs              # D-12 (Spec.title binding); D-14 (depth 3→5); D-16 (validate reorder if pipeline lives here)
├── component.rs         # D-13 (KanbanBoard.data_path); D-15 (Image/DescList.data_path); D-18 (CardVariant);
│                        # D-19/F6 (PageHeader.actions Option); new RawHtmlProps for D-17a
├── visibility.rs        # D-19/F5 (untagged-enum error message via custom Deserialize)
├── catalog.rs           # 40 → 41 components if D-17a ships (RawHtml entry)
├── render/
│   ├── atoms.rs         # render_image (D-15); render_description_list (D-15); new render_raw_html (D-17a)
│   ├── containers.rs    # render_card (D-18 variant branch); render_kanban_board (D-13a)
│   └── mod.rs           # BUILTIN_TYPES table: 40 → 41 entries if D-17a ships
├── resolve.rs           # expand_directives (already present); D-05 audit if any directive validator gap

framework/src/json_ui/
└── mod.rs               # D-16: invert resolve/validate order if D-16 puts validate inside JsonUi::resolve;
                         # D-12: handle expression-bound title in build_response (line 89 title extraction)

ferro-cli/src/commands/
└── json_ui_migrate_v1.rs  # D-19/F2: verify line 521 already uppercases — likely no change needed

ferro-mcp/src/tools/
├── json_ui_validate_spec.rs  # NEW (D-04) — wraps Spec::from_json + global_catalog().validate()
├── json_ui_catalog.rs        # update 40-component assertion (line 289) + expected list (line 296)

docs/src/json-ui/
├── components.md         # D-08 sections for CardVariant, RawHtml, data_path props
├── expressions.md        # D-08/D-13b worked example: $each for kanban columns
├── spec-construction.md  # D-08/D-14 documentation of new depth limit
├── migration-v1-to-v2.md # D-09 cheat-sheet table at top
└── plugins.md            # D-08 cross-link to D-06 audit findings

.planning/phases/164-.../
├── V1-DELETION-AUDIT.md  # D-01..D-03 audit artefact
└── COMPLETED.md          # D-10..D-11 — gates Phase 160
```

### Pattern 1: Expression-binding-or-literal field (D-12 model)

**What:** A field that accepts either a plain string literal or `{"$data": "/path"}` for runtime resolution.
**When to use:** When a top-level Spec field (`title`, future `layout`?) needs to be dynamic. Distinct from `Element.props` (which is `serde_json::Value` and accepts expressions natively).
**Two implementation options:**

**Option A — TitleBinding enum:**
```rust
// ferro-json-ui/src/spec.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TitleBinding {
    Literal(String),
    Binding(ExpressionRef),  // { "$data": "/path" } shape
}

pub struct Spec {
    pub title: Option<TitleBinding>,
    // ...
}
```
Then `framework/src/json_ui/mod.rs:89` becomes:
```rust
let title = match &spec.title {
    Some(TitleBinding::Literal(s)) => s.as_str(),
    Some(TitleBinding::Binding(e)) => &resolve_title_binding(e, &spec.data),
    None => "Ferro",
};
```

**Option B — Pre-resolve in `JsonUi::resolve`:**
Walk the spec and, if `spec.title` matches the `{$data:...}` shape, mutate to the resolved literal before `build_response` reads `spec.title.as_deref()`. Keeps the public type as `Option<String>` — only the deserializer changes.

Recommendation: **Option A** (typed enum). Aligns with how `Visibility` and `EachDirective` are typed; preserves the binding shape for re-serialization (`render_json` route returns the spec verbatim).

### Pattern 2: data_path-overrides-static-field (D-13/D-15 model)

**What:** A static-typed field (`columns: Vec<KanbanColumnProps>`, `src: String`, `items: Vec<DescriptionItem>`) gains a sibling `data_path: Option<String>` that, when set, overrides the static field with a runtime-resolved value.
**When to use:** Any catalog component where the consumer needs either inline or data-driven shape — and the v2 `$each` directive (which produces clones) is the wrong tool because the component itself expects an array prop, not a templated-element-tree.
**Example (Image, D-15):**
```rust
pub struct ImageProps {
    #[serde(default)]  // src becomes optional when data_path is set
    pub src: String,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    // ...
}
```
Renderer (`render/atoms.rs:365` `render_image`):
```rust
let resolved_src = props.data_path
    .as_deref()
    .and_then(|p| data.pointer(p)?.as_str())
    .map(String::from)
    .unwrap_or(props.src.clone());
```
**Trade-off:** `src` was previously required; making it `#[serde(default)]` loosens the contract. Acceptable because `data_path` overrides it. The Catalog Stage 2 schema validates this is coherent (`oneOf: [{required: ["src"]}, {required: ["data_path"]}]` — needs schemars annotation or hand-written schema overlay).

### Pattern 3: Variant enum on a Props struct (D-18 model)

**What:** Add a `variant` field of an enum type with `#[serde(rename_all = "lowercase")]` and `#[derive(Default)]` so unset specs keep the previous default rendering.
**Source:** V7-RUNTIME §F10 lines 109–138 ships the exact code. Identical pattern to `AlertVariant`, `BadgeVariant`, `ButtonVariant`, `ToastVariant`, `ActionCardVariant`, `NotifyVariant`, `DialogVariant`.
**Schema regen:** `schemars::schema_for!(CardProps)` picks up the new field automatically. The MCP `json_ui_catalog` tool re-runs `schema_for!` on every call — no manual regen step.

### Anti-Patterns to Avoid

- **Don't reintroduce a generic `Component::Plugin` dispatch.** Phase 115 D-01 explicitly killed the v1 `Component::Plugin { plugin_type, props }` shape as "the wart we're removing" (verified in `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md:228`). D-17a is `Component::RawHtml` — a NEW, narrowly-scoped primitive — NOT the v1 dispatch.
- **Don't add a separate `padding` / `elevation` props to CardProps.** V7-RUNTIME §F10 line 150 explicitly answers this: "Recommendation: ship `variant` only." Adding granular props creates a 2×2 (or larger) matrix of legal combinations the renderer must handle; the variant pattern is the workspace standard.
- **Don't gate D-16 on consumer migration to `$if`.** The friction file footnote (V7-RUNTIME §F8) notes migration is "a complementary path." The architectural fix is the pipeline reorder; doing only the consumer migration leaves the parse-order inconsistency in place to recur on the next consumer.
- **Don't add `ammonia` as a workspace dep for D-17a.** Match `RichTextEditorProps` discipline (`ferro-json-ui/src/component.rs:273`): document the trust boundary, push sanitisation to the consumer's handler.
- **Don't bump `MAX_NESTING_DEPTH` past 5 in this phase.** The friction evidence is a single depth-4 spec. 5 covers depth-4 with one level of headroom; 6+ invites the over-nesting the depth limit exists to prevent.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-component prop validation against a JSON Schema | A custom validator walking `serde_json::Value` | `jsonschema::validator_for(&schema)` — already wired in `Catalog::validate` at `catalog.rs:672` | The framework already runs jsonschema for every Spec; D-04 MCP tool reuses the same compiled validator |
| Untagged-enum-with-error-naming (D-19/F5) | A custom error type that re-runs the deserializer per variant | A `Visitor` impl that attempts each variant in order, collecting errors; or a manual `from_value` that tries `VisibilityCondition` / `And` / `Or` / `Not` and reports the longest-prefix match | serde's `#[serde(untagged)]` macro-generated error message is the documented failure mode; replacing it with a `#[serde(tag = ...)]` would break the existing JSON shape |
| HTML sanitisation in renderer (D-17a) | Inline regex stripping `<script>` tags | Document the trust boundary; let the consumer pre-sanitise with `ammonia` before storing | Mirrors `RichTextEditorProps` discipline (component.rs:273). Renderer-side sanitisation is fragile and lulls authors into trusting untrusted data |
| Codemod HTTP method uppercasing (D-19/F2) | A post-emission string-rewrite step | The codemod at `json_ui_migrate_v1.rs:521` already emits `"POST"`/`"GET"`/etc. uppercase. Verify with a targeted unit test; no code change | The codemod-fix angle is already shipped; the 26 lowercase-method specs in gestiscilo were hand-authored, not codemod-output. D-19/F2 reduces to "audit + add regression test" |
| MCP validate-spec tool result envelope (D-04) | A new `ValidateResponse` struct | Reuse `Vec<CatalogError>` directly — already `serde::Serialize`able via thiserror | Single source of truth for error shape; MCP tool surfaces what `load_cached` would surface at server startup |
| RawHtml component renderer (D-17a) | A new HTML-building helper module | Inline 5-line render fn in `render/atoms.rs` that emits `<div data-ferro-raw-html>{html}</div>` (verbatim, no escaping) | The whole point is verbatim emission; helper module would obscure the trust boundary |

**Key insight:** Phase 164's implementation surface is overwhelmingly **typed-field additions** and **renderer branches**. The two non-mechanical items are D-16 (pipeline reorder — requires careful ordering audit across `loader.rs`, `JsonUi::resolve`, `JsonUi::render_with_errors`) and D-19/F5 (deserializer error-message improvement — requires understanding serde's untagged-enum failure mode and either a Visitor or a manual two-pass deserializer).

## Runtime State Inventory

Phase 164 is NOT a rename/refactor/migration phase. Skipping per the protocol — no runtime state to inventory.

(Exception: the v1-deletion audit (D-01..D-03) is technically a deletion-readiness check, but the deletion itself happens in Phase 160. Phase 164 produces only the audit artefact, no code deletion.)

## Common Pitfalls

### Pitfall 1: D-16 reorder breaks "validation at startup" semantics

**What goes wrong:** Today, `loader::load_cached` calls `Spec::from_json` then `global_catalog().validate(&spec)` — both at load time. A bad spec fails server startup. If D-16 moves catalog validation *after* `expand_directives` (which only runs at render time inside `JsonUi::resolve`), then a spec with `Alert.variant=""` and no `visible` gate will fail at the FIRST REQUEST, not at startup. This silently degrades a startup-time guarantee to a per-request failure.

**Why it happens:** The current pipeline ordering is `parse → catalog.validate → cache`. The D-16 reorder is `parse → cache → (per-request: expand → validate → render)`.

**How to avoid:**
- Option A: keep `Catalog::validate` at load time AS A WARNING (don't fail), then re-validate at render time after `expand_directives`. Two-stage with severity escalation.
- Option B: move directive expansion to load time (run it once against `spec.data` if non-null; re-run per-request only if handler data differs). The existing `validate_directives` (`spec.rs:749`) already partially does this — it checks `$each.path` against `spec.data` when non-null.
- Option C: define a "valid spec" as "every variant of every element-shape is at-least-once-reachable" — but this is brittle (combinatorial against `$if` predicates).

**Recommended:** Option A (two-stage). Startup warning preserves the "fail loud at startup" intent for fully-static specs; per-request validation catches the post-`$if`/`$each` shape. Aligns with the friction file's recommendation: "structural validation (element references, footer IDs, depth) remains at parse-time" (CONTEXT D-16).

### Pitfall 2: CardVariant rename mixes serde lowercase with Rust naming

**What goes wrong:** Friction file ships `#[serde(rename_all = "lowercase")]` on `CardVariant`. But every OTHER variant enum in `ferro-json-ui` uses `#[serde(rename_all = "snake_case")]` (e.g. `AlertVariant`, `BadgeVariant`, `ActionCardVariant`). Lowercase vs snake_case differ for multi-word variants (`Bordered` and `Elevated` are single-word so they're identical — but a future variant like `MarketingPopout` would serialize differently).

**Why it happens:** Friction file's example uses `"lowercase"`. Workspace convention is `"snake_case"`.

**How to avoid:** Use `#[serde(rename_all = "snake_case")]` for CardVariant to match every other variant enum (`component.rs` lines 580, 891, `action.rs` line 49). Single-word variants serialize identically; future-proofs the enum.

### Pitfall 3: 40-component count assertion lands in three places

**What goes wrong:** Adding `Component::RawHtml` (D-17a) bumps the BUILTIN component count. Three sites assert "40":
1. `ferro-json-ui/src/render/mod.rs:530` — `assert_eq!(BUILTIN_TYPES.len(), 40);`
2. `ferro-json-ui/src/catalog.rs:1052` — `assert_eq!(BUILTIN_SPECS.len(), 40);`
3. `ferro-mcp/src/tools/json_ui_catalog.rs:290` — `assert_eq!(catalog.components.len(), 40, ...)` + the expected-names list at lines 296–337.

Missing any one of these breaks the test suite.

**How to avoid:** Plan that ships D-17a explicitly enumerates all three update sites in its task list. Add `"RawHtml"` to the expected-names array in the MCP test.

**Warning signs:** `cargo test --all-features` fails with `BUILTIN_TYPES.len() == 41` or `catalog.components.len() == 41`.

### Pitfall 4: Visibility deserializer fix can regress on And/Or/Not

**What goes wrong:** `Visibility` is `#[serde(untagged)]` with 4 variants: `And{and}`, `Or{or}`, `Not{not}`, `Condition(VisibilityCondition)`. A Visitor-based fix that "names the variant" must try ALL FOUR variants and report which one was the closest match — not just the first one tried. A naive fix that always reports "expected VisibilityCondition" would break specs using `{"and": [...]}` shape.

**Why it happens:** untagged-enum deserialization is "try each variant in declaration order; succeed on the first that parses." The serde-generated error is `"data did not match any variant of untagged enum Visibility"` — no per-variant info.

**How to avoid:** Custom `Deserialize` impl that deserializes into a `serde_json::Value` first, then dispatches by shape:
- has key `"and"` → try `And`
- has key `"or"` → try `Or`
- has key `"not"` → try `Not`
- has `"path"` + `"operator"` → try `Condition`
- otherwise → emit error listing all four accepted shapes with examples

**Warning signs:** Phase 162 plan tests covering `Visibility` (already-existing tests in `visibility.rs`) would regress.

### Pitfall 5: D-12 expression binding shape collides with serde_json::Value flexibility

**What goes wrong:** `Spec.title: Option<TitleBinding>` with `TitleBinding::Binding(ExpressionRef)` — but what's the shape of `ExpressionRef`? The catalog uses ad-hoc `{"$data": "/path"}` and `{"$template": "..."}` shapes (see `catalog.rs:687` `strip_expr_objects`). There's no canonical `ExpressionRef` type yet.

**Why it happens:** Expression-binding values are stuffed into `Element.props` as raw `serde_json::Value`. Spec.title is a typed `Option<String>`, so it needs a typed binding shape.

**How to avoid:** Define `ExpressionRef` (or reuse if one exists in `ferro-json-ui/src/expression.rs`) as:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DataRef {
    #[serde(rename = "$data")]
    pub data: String,
}
```
Then `TitleBinding::Binding(DataRef)`. Forward-compat for `$template` if needed later.

**Warning signs:** Round-trip test: spec with `"title": {"$data": "/page_title"}` serializes back to identical JSON.

### Pitfall 6: PageHeader.actions `Vec<String> → Option<Vec<String>>` is technically breaking

**What goes wrong:** D-19/F6 changes `actions: Vec<String>` to `actions: Option<Vec<String>>`. Any consumer Rust code that built `PageHeaderProps { actions: vec![], .. }` now needs `actions: None` or `actions: Some(vec![])`. Wire format is unchanged (`Vec::is_empty()` already skip-serializes) but the Rust constructor is broken.

**Why it happens:** Type-system breaking changes are invisible in JSON wire format.

**How to avoid:** Add a `#[serde(default, deserialize_with = "deserialize_actions_lax")]` custom deserializer that accepts ALL THREE shapes: missing field, empty array, empty string. Keep the Rust type as `Vec<String>`. This closes F6 without breaking consumers.

```rust
fn deserialize_actions_lax<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::String(s) if s.is_empty() => Ok(Vec::new()),
        serde_json::Value::Array(arr) => arr.into_iter()
            .map(|v| v.as_str().map(String::from).ok_or_else(|| serde::de::Error::custom("expected string")))
            .collect(),
        other => Err(serde::de::Error::custom(format!("expected array or empty string, got {:?}", other))),
    }
}
```

**Warning signs:** Existing tests using `PageHeaderProps { actions: vec![...] }` would fail to compile if the type changed.

## Code Examples

### Example 1: CardVariant render branch (D-18 verbatim from V7-RUNTIME §F10)

```rust
// ferro-json-ui/src/component.rs (additions near line 153 — CardProps)
// Source: V7-RUNTIME-FRICTION.md §F10 lines 109–126

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]  // NOTE: snake_case, not lowercase — see Pitfall 2
pub enum CardVariant {
    #[default]
    Bordered,
    Elevated,
}

pub struct CardProps {
    pub title: String,
    #[serde(default)]
    pub variant: CardVariant,
    // ...existing fields preserved
}

// ferro-json-ui/src/render/containers.rs (render_card body at line 53)
// Source: V7-RUNTIME-FRICTION.md §F10 lines 130–135

let (outer, inner_pad) = match props.variant {
    CardVariant::Bordered => (
        "rounded-lg border border-border bg-card shadow-sm overflow-visible",
        "p-4",
    ),
    CardVariant::Elevated => (
        "rounded-lg bg-card shadow-md overflow-visible",
        "p-8",
    ),
};
let mut html = format!("<div class=\"{outer}\"><div class=\"{inner_pad}\">");
// ...rest of render_card preserved
```

### Example 2: Image data_path resolution (D-15)

```rust
// ferro-json-ui/src/component.rs (ImageProps at line 516)
pub struct ImageProps {
    #[serde(default)]            // ← changed: was required
    pub src: String,
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<String>,
    // ...existing fields (aspect_ratio, placeholder_label, inline_svg) preserved
}

// ferro-json-ui/src/render/atoms.rs (render_image at line 365)
let resolved_src = props
    .data_path
    .as_deref()
    .and_then(|p| crate::data::resolve_path(data, p))
    .and_then(|v| v.as_str().map(String::from))
    .unwrap_or_else(|| props.src.clone());
// ...use resolved_src in <img src="..."> emission
```

### Example 3: MCP json_ui_validate_spec tool (D-04)

```rust
// ferro-mcp/src/tools/json_ui_validate_spec.rs (NEW)
use ferro_json_ui::{global_catalog, Spec};
use serde::Serialize;

#[derive(Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub structural_errors: Vec<String>,  // SpecError variants stringified
    pub catalog_errors: Vec<String>,     // CatalogError variants stringified
    pub warnings: Vec<String>,           // duplicate footer/children, etc.
}

pub fn execute(spec_json: &str) -> ValidateResponse {
    let mut response = ValidateResponse {
        valid: true,
        structural_errors: Vec::new(),
        catalog_errors: Vec::new(),
        warnings: Vec::new(),
    };
    let spec = match Spec::from_json(spec_json) {
        Ok(s) => s,
        Err(e) => {
            response.valid = false;
            response.structural_errors.push(e.to_string());
            return response;
        }
    };
    if let Err(errs) = global_catalog().validate(&spec) {
        response.valid = false;
        response.catalog_errors = errs.into_iter().map(|e| e.to_string()).collect();
    }
    response
}
```

### Example 4: v1-deletion audit table (D-01..D-03) — proposed shape

```markdown
# V1-Deletion Readiness Audit

| v1 surface | v2 equivalent | gestiscilo usage | Resolution |
|------------|---------------|------------------|------------|
| `JsonUiView` | `Spec { schema, root, elements }` | Migrated in all 4 Phase 138 controllers + cassa + documenti | MIGRATED |
| `Component` enum | `Element.type_name: String` + catalog | Every Element since Phase 115 | MIGRATED |
| `ComponentNode` | `Element` in flat `Spec.elements` map | All controllers using `render_file` or `Spec::builder` | MIGRATED |
| `PluginProps { plugin_type, props }` | First-class plugin types (e.g. `"type": "Map"`) | gestiscilo settings used `"type": "Plugin"` → BLOCKER → D-17a (`Component::RawHtml`) | MIGRATED via D-17a |
| `CardProps.children` | `Element.children: Vec<String>` (ID refs) | All Card uses | MIGRATED |
| `FormProps.fields` | `Element.children` for form | All Form uses | MIGRATED |
| `GridProps.children` | `Element.children` | All Grid uses | MIGRATED |
| `CollapsibleProps.children` | `Element.children` | All Collapsible uses | MIGRATED |
| `FormSectionProps.children` | `Element.children` | All FormSection uses | MIGRATED |
| `ButtonGroupProps.buttons` | `Element.children` | All ButtonGroup uses | MIGRATED |
| `SwitchProps.compact` | Re-added in Phase 162 D-16 | 6 settings.rs sites | MIGRATED |
| `ImageProps::inline_svg` | Re-added in Phase 162 D-17 | gestiscilo statistiche bar charts | MIGRATED |
| `RichTextEditorProps` | Re-added as plugin in Phase 162 D-18 | 2 documenti templates | MIGRATED |
| `DetailFormProps` / `DetailField` / `EditMode` | Documented v2 pattern (D-15 in Phase 162) | documenti edit flows | INTENTIONAL_DROP (pattern documented; no consumer-blocking) |
| `make_node` / `make_node_with_action` builder helpers | `JsonUi::render_file` + JSON spec files; codemod for legacy | Phase 138 controllers all migrated; codemod available for stragglers | INTENTIONAL_DROP (consumer-side helpers, never part of ferro public API) |
| `view.rs` / `JsonUiView::new` chain | `Spec::builder()` / `Spec::from_json` | All controllers | MIGRATED (file deleted in commit dbe5adaf) |
```

This is the canonical shape. The audit's success criterion is **zero `BLOCKER` rows** after Phase 164 ships.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| v1 nested `Component` enum + recursive `Vec<ComponentNode>` + ~200 LoC custom ser/de | v2 flat `Spec { root, elements: HashMap }` with type-erased `Element { type_name: String }` | Phase 115 (commit `dbe5adaf` 2026-...) | Deleted `view.rs`, `Component` enum, `ComponentNode`, `PluginProps`. v2 is the only surface on `v12.0/json-ui-v2`. |
| v1 `Component::Plugin { plugin_type, props }` dispatch | First-class plugin component type names via `JsonUiPlugin` trait + `register_plugin` | Phase 115 D-01 | Killed the "some types are special" backdoor. **D-17a does NOT undo this** — RawHtml is a new built-in, not a plugin escape hatch. |
| Per-page validator at handler call site | Centralised `Catalog::validate` at `load_cached` time | Phase 117 | Catches malformed specs at server startup, not at first request. D-16 partially shifts this back to per-request for post-`$if` shapes. |
| `JsonUiView::new` Rust builder | `JsonUi::render_file("src/views/.../*.json", data)` + `Spec::builder` for runtime-shaped specs | Phase 119 (`load_cached`) | Spec authoring moves from Rust source to JSON files. Phase 163 codemod automates v1→v2 conversion. |
| `Spec.title: Option<String>` static literal only | `Spec.title: Option<TitleBinding>` accepting literal or `{$data}` binding | **D-12 (this phase)** | 23 gestiscilo specs unblocked. Renderer emits resolved title in `<title>` and `<h1>`. |
| `MAX_NESTING_DEPTH = 3` | `MAX_NESTING_DEPTH = 5` | **D-14 (this phase)** | Depth-4 dashboard specs no longer fail. Still constrains deep nesting (5 is enough headroom for current usage). |
| Card chrome hard-coded `border-border + shadow-sm + p-4` | `CardVariant::{Bordered, Elevated}` opt-in | **D-18 (this phase)** | Auth/error/marketing pages get correct elevated chrome without affecting ~30 dashboard Card uses. |
| Parse-time enum validation against raw spec | Validation after `expand_directives` for enum-shape; structural validation remains at parse-time | **D-16 (this phase)** | `Alert.variant=""` gated by `visible` (or `$if`) no longer fails at load. Closes 2 dashboard pages. |
| `Plugin` component type rejected at catalog | New `Component::RawHtml` for HTML-island use cases; plugin-registered types continue to use real type name (e.g. `"type": "Map"`) | **D-17a (this phase)** | gestiscilo settings unblocked. No regression to v1's generic Plugin dispatch — RawHtml is a narrow primitive. |

**Deprecated/outdated:**
- `JsonUiView`, `Component`, `ComponentNode`, v1 `PluginProps` — already deleted on `v12.0/json-ui-v2` (verified). Phase 160 will assert their absence is permanent.
- `make_node` / `make_node_with_action` consumer helpers — never part of ferro public API. Documented as a v1 pattern in `migration-v1-to-v2.md`.
- v1 `auth` layout's implicit `Card` wrapper — removed in commit `392b0191` (Phase 162 162-06, D-05).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Catalog `validate` is the only site enforcing enum-variant validation; no other parse-time site validates `Alert.variant` | Pitfall 1, D-16 | If a second validation site exists (e.g. render-time `from_value::<AlertProps>(...)`), D-16's reorder won't fully fix F8. Mitigation: grep for `serde_json::from_value::<.*Props>` in the render path — already done (3 hits in render/, all in test code or fall-through ButtonGroup render at containers.rs:621). HIGH confidence the enum validation is centralised in `Catalog::validate`. |
| A2 | The 26 gestiscilo specs with lowercase HTTP methods were hand-authored, not codemod output | D-19/F2 | If they came from the codemod, the codemod fix at line 521 is broken. Mitigation: the codemod was applied to ~12 controllers per Phase 138 FRICTION; the 26 lowercase-method specs are likely from the broader migration (phases 139–143). MEDIUM confidence — planner should grep gestiscilo spec history to confirm. |
| A3 | `Visibility` deserializer error improvement (D-19/F5) can be implemented as a custom Deserialize without breaking any existing valid JSON | Pitfall 4, D-19/F5 | If the custom impl is buggy, all existing `visible` specs across all consumers regress. Mitigation: round-trip every visibility test fixture in `visibility.rs` tests. HIGH confidence — the failure shape ("dispatch by key presence") is well-understood. |
| A4 | `PageHeader.actions` change (D-19/F6) can be done via lax deserializer keeping the Rust type as `Vec<String>` | Pitfall 6, D-19/F6 | If the lax deserializer is rejected for stylistic reasons, the alternative is `Option<Vec<String>>` which is a Rust API break. Mitigation: cite the existing `deserialize_with` pattern (none in `ferro-json-ui` currently — would be the first). MEDIUM confidence; planner picks lax-vs-Option. |
| A5 | Phase 115's deletion of v1 surface is fully complete; no v1 types remain in the public API | D-01 audit | If a v1 type persists in `framework/src/lib.rs` re-exports, the audit must catch it. Mitigation: grep `framework/src/lib.rs` for `JsonUiView\|ComponentNode\|Component::` — done, no hits. HIGH confidence. |
| A6 | D-17a's `Component::RawHtml` is narrow enough that it won't drift into a generic Plugin dispatch | D-17, Anti-Patterns | If consumers start passing `{ "$data": "/big_html_blob" }` for arbitrary widgets, the boundary erodes. Mitigation: doc the trust boundary strongly (mirror RichTextEditorProps:273); call out in `plugins.md` that "real" plugins use `JsonUiPlugin`. MEDIUM confidence — depends on doc discipline. |
| A7 | The D-13 default (ship D-13a `data_path` AND D-13b `$each` example) does not create two redundant ways to do the same thing in a confusing way | D-13 | If consumers can't tell when to use which, the doc burden grows. Mitigation: docs should call D-13a "one element type per column" and D-13b "one element type per card row inside a column" — they solve different sub-problems. MEDIUM confidence. |
| A8 | gestiscilo's F5 (Visibility parse fail in clienti/list, flotta/list) is caused by the controller emitting a Visibility shape the untagged enum doesn't accept (CONTEXT line 60 reads "probably `visible: { "expr": "..." }` vs the accepted `{ "path": "...", "operator": "..." }` form") | D-19/F5 | If the actual shape is different (e.g. accidental wrapper object), the error-message fix is still useful but the gestiscilo-side fix is wrong. Mitigation: read clienti/list.json line 47 to confirm before D-19/F5 implementation. LOW confidence on the gestiscilo cause; HIGH confidence the error-message improvement is universally useful. |

**If user-confirmation is needed:** Assumptions A2, A4, A6, A7, A8 (those with MEDIUM/LOW confidence) deserve a /gsd-discuss-phase pass before planning if the planner wants the lock-in. A1, A3, A5 are verified by source-tree grep and can be treated as facts.

## Open Questions

1. **D-16 sub-decision — single-pass or two-stage validation?**
   - What we know: Today `load_cached` calls `from_json` (structural) → `catalog.validate` (enum). Moving `catalog.validate` to per-request defeats the startup-fail-loud property.
   - What's unclear: Whether to keep a startup-warning pass (Option A in Pitfall 1) or fully defer to per-request (Option B).
   - Recommendation: Two-stage. Startup validation warns (not fails); per-request validation enforces. Both stages use the same `Catalog::validate` function, distinguished by whether `expand_directives` ran first.

2. **D-17a — RawHtml semantics for empty / missing data**
   - What we know: gestiscilo specs use `{html: {$data: "/owner_commands_html"}}` and similar.
   - What's unclear: What renders when `/owner_commands_html` is missing or null in `spec.data`?
   - Recommendation: Render nothing (empty string). Match the `Skeleton` component's "missing data → still emit container, no content" pattern. Document in `components.md`.

3. **D-04 — MCP tool naming and parameter shape**
   - What we know: No `json_ui_validate_spec` tool exists. Existing tools: `json_ui_catalog`, `json_ui_inspect`, `json_ui_generate`, `json_ui_verify_action`.
   - What's unclear: Whether to extend `json_ui_inspect` (which takes a single component type) or ship a new tool that takes a full spec JSON string.
   - Recommendation: New tool `json_ui_validate_spec` taking `{ spec: String }` (JSON-encoded). `json_ui_inspect`'s scope is "describe one component"; validation is a different operation.

4. **D-06 paper audit — what's the deliverable?**
   - What we know: The audit walks the v2 plugin author guide and verifies a fresh author could implement Stripe / WhatsApp / chart widgets.
   - What's unclear: Whether this produces a written artefact (e.g. `PLUGIN-SURFACE-AUDIT.md`) or is a verbal checkpoint with the user.
   - Recommendation: Written artefact if any gaps surface; verbal checkpoint if everything passes. CONTEXT D-06 lists this under "Claude's discretion."

5. **D-19/F2 — codemod fix scope**
   - What we know: The codemod at `ferro-cli/src/commands/json_ui_migrate_v1.rs:521` already emits uppercase HTTP methods.
   - What's unclear: Whether the 26 lowercase-method gestiscilo specs came from the codemod (in which case the line-521 logic is broken in a path we haven't seen) or from hand-authoring (in which case the fix is "add a regression test, nothing else").
   - Recommendation: Add a unit test that codemod-emits a `row_actions[i].action` with `method: "POST"` upper-case. If the test passes immediately, D-19/F2 ships as a verification-only plan.

6. **D-17b documentation — how much depth?**
   - What we know: The recommended alternative for richer widgets is "consumer migrates to first-class `JsonUiPlugin`."
   - What's unclear: Whether `docs/src/json-ui/plugins.md` already has enough material for this, or whether D-06's audit will surface gaps that need filling.
   - Recommendation: Defer to D-06 audit findings. If the audit produces a `BLOCKER` for any of the three exemplars, plumb the fix into D-08's doc pass.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Building / testing every plan | Verify with `which cargo && cargo --version` | — | none — required |
| `rustc` | All Rust compilation | Always available with cargo | — | none — required |
| `cargo fmt` | Pre-commit gate | Standard component | — | none — required |
| `cargo clippy` | Pre-commit gate | Standard component | — | none — required |
| `ferro-mcp` (debug build) | Manual verification of D-04 MCP tool | Built from `target/debug/ferro mcp` | local | Skip MCP smoke test if not built; rely on `cargo test -p ferro-mcp` |
| `gestiscilo-it` workspace | Cross-repo verification of fix effectiveness (F1, F8 closure) | `/Users/alberto/repositories/gestiscilo-it/app` (Phase 138 FRICTION + V7-RUNTIME-FRICTION source) | local | Skip cross-repo verification; rely on ferro-side tests |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** ferro-mcp debug build is only needed for one optional manual MCP smoke test; skip if unbuilt.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in Rust) with workspace-level invocation |
| Config file | None (default Rust test discovery via `#[cfg(test)] mod tests`) |
| Quick run command | `cargo test -p ferro-json-ui` (per-crate; ~30s) |
| Full suite command | `cargo test --all-features` (workspace; CI-equivalent) |

### Phase Requirements → Test Map

| Decision | Behaviour | Test Type | Automated Command | File Exists? |
|----------|-----------|-----------|-------------------|--------------|
| D-12 | `Spec.title` round-trip with `{$data}` binding parses, serializes back identical | unit | `cargo test -p ferro-json-ui spec::tests::title_binding` | ❌ Wave 0 |
| D-12 | Renderer emits resolved title in `<title>` tag (integration) | unit | `cargo test -p ferro --test render_file title_binding_resolves` | ❌ Wave 0 |
| D-13a | `KanbanBoardProps { columns: vec![], data_path: Some("/board"), .. }` parses and renders one column per data row | unit | `cargo test -p ferro-json-ui render::containers::tests::kanban_data_path` | ❌ Wave 0 |
| D-13b | docs/src/json-ui/expressions.md includes a $each-for-kanban-columns worked example | docs | `mdbook build docs/` (no broken links) | (manual review) |
| D-14 | Depth-4 spec validates (was depth 3 limit); depth-5 spec validates; depth-6 spec fails with `SpecError::DepthExceeded` | unit | `cargo test -p ferro-json-ui spec::tests::depth_4_valid spec::tests::depth_5_valid spec::tests::depth_6_rejected` | ❌ Wave 0 (existing test at line 1705 needs rewrite + 2 new tests) |
| D-15 | `ImageProps { src: "", data_path: Some("/src"), .. }` with data `{"src": "url"}` renders `<img src="url">` | unit | `cargo test -p ferro-json-ui render::atoms::tests::image_data_path` | ❌ Wave 0 |
| D-15 | `DescriptionListProps { items: vec![], data_path: Some("/items"), .. }` with data array renders one row per item | unit | `cargo test -p ferro-json-ui render::atoms::tests::description_list_data_path` | ❌ Wave 0 |
| D-16 | Spec with `Alert { variant: "", visible: { exists: /flash } }` and data `{}` (flash absent → element removed by `$if`-style logic) does NOT fail catalog validation | integration | `cargo test -p ferro --test pipeline_order alert_empty_variant_gated` | ❌ Wave 0 |
| D-16 | Same spec WITHOUT `visible` gate DOES fail with `CatalogError::PropsInvalid { type_name: "Alert" }` | unit | `cargo test -p ferro-json-ui catalog::tests::alert_empty_variant_ungated_fails` | exists (general failure tested at catalog.rs:1350) — extend |
| D-17a | `Element { type_name: "RawHtml", props: { html: "<p>hi</p>" } }` renders verbatim into the output | unit | `cargo test -p ferro-json-ui render::atoms::tests::raw_html_renders_verbatim` | ❌ Wave 0 |
| D-17a | Catalog reports 41 built-in components (was 40); MCP exposes "RawHtml" in names list | unit | `cargo test --workspace catalog_count raw_html_in_mcp_names` | ❌ Wave 0 (update existing assertions at render/mod.rs:530, catalog.rs:1052, json_ui_catalog.rs:290) |
| D-18 | `CardProps { variant: CardVariant::Bordered }` renders default chrome (border + shadow-sm + p-4) | unit | `cargo test -p ferro-json-ui render::containers::tests::card_bordered_default` | exists — extend |
| D-18 | `CardProps { variant: CardVariant::Elevated }` renders elevated chrome (shadow-md + p-8, no border) | unit | `cargo test -p ferro-json-ui render::containers::tests::card_elevated_variant` | ❌ Wave 0 |
| D-18 | `serde_json::from_str("{...}")` of CardProps without `variant` field defaults to Bordered | unit | `cargo test -p ferro-json-ui component::tests::card_variant_default` | ❌ Wave 0 |
| D-19/F2 | Codemod emits `"method": "POST"` (uppercase) for `Action::post("foo")` | unit | `cargo test -p ferro-cli json_ui_migrate_v1::tests::action_method_uppercase` | likely exists — VERIFY (line 521 logic shipped); add if missing |
| D-19/F5 | Visibility parse failure on `{"expr": "foo"}` shape produces error message containing both the offending shape AND the four accepted variant names | unit | `cargo test -p ferro-json-ui visibility::tests::error_message_lists_variants` | ❌ Wave 0 |
| D-19/F5 | Round-trip of every existing Visibility test fixture still passes (no regression) | unit | `cargo test -p ferro-json-ui visibility::tests` | exists (Phase 116 baseline) |
| D-19/F6 | `PageHeaderProps` deserializes from `{ "actions": "" }` to empty `Vec<String>` (lax) | unit | `cargo test -p ferro-json-ui component::tests::page_header_actions_lax` | ❌ Wave 0 |
| D-19/F6 | `PageHeaderProps` deserializes from missing `actions` field to empty Vec (existing behaviour preserved) | unit | `cargo test -p ferro-json-ui component::tests::page_header_actions_missing` | exists — verify |
| D-04 | `json_ui_validate_spec` MCP tool returns `valid: false` + named catalog errors for a known-bad spec | unit | `cargo test -p ferro-mcp json_ui_validate_spec::tests::reports_catalog_errors` | ❌ Wave 0 |
| D-05 | Existing `validate_directives` tests still pass (audit only — no behaviour change unless gap found) | unit | `cargo test -p ferro-json-ui spec::tests::validate_directives` | exists (Phase 163) |
| D-01..D-03 | `V1-DELETION-AUDIT.md` exists and has zero `BLOCKER` rows | docs | `! grep -q '| BLOCKER |' .planning/phases/164-.../V1-DELETION-AUDIT.md` | ❌ Wave 0 |
| D-10..D-11 | `COMPLETED.md` exists with five required sections | docs | manual review against CONTEXT D-10 section list | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui` (~30s) — covers all D-12..D-18 unit tests
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` (CI-equivalent; ~3-5min) — gates the wave
- **Phase gate:** Full suite green + `V1-DELETION-AUDIT.md` zero-BLOCKER + `COMPLETED.md` complete; then `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/spec.rs::tests` — add `title_binding_*`, `depth_4_valid`, `depth_5_valid`, `depth_6_rejected` tests (rewrite existing `nested_builder_flattens_two_levels` at line 1704 to use depth 5)
- [ ] `ferro-json-ui/src/component.rs::tests` — add `card_variant_default`, `card_variant_round_trip`, `page_header_actions_lax`
- [ ] `ferro-json-ui/src/render/atoms.rs::tests` — add `image_data_path`, `description_list_data_path`, `raw_html_renders_verbatim`
- [ ] `ferro-json-ui/src/render/containers.rs::tests` — add `card_elevated_variant`, `kanban_data_path`
- [ ] `ferro-json-ui/src/visibility.rs::tests` — add `error_message_lists_variants`
- [ ] `ferro-mcp/src/tools/json_ui_validate_spec.rs` — NEW file with tests `reports_catalog_errors`, `accepts_valid_spec`
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — update component count assertion at line 290 (40 → 41 if D-17a ships) + extend expected-names array
- [ ] `ferro-json-ui/src/catalog.rs` — update BUILTIN_SPECS count assertion at line 1052
- [ ] `ferro-json-ui/src/render/mod.rs` — update BUILTIN_TYPES count assertion at line 530
- [ ] `framework/tests/` — add `pipeline_order.rs` integration test for D-16 (full render through `JsonUi::render_file`)
- [ ] Framework install: no install needed — `cargo` and `rustc` already available

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/spec.rs` — Spec struct (line 49), `MAX_NESTING_DEPTH` (line 37), `validate_structure` (line 647), `validate_directives` (line 749), nesting depth test (line 1705)
- `ferro-json-ui/src/component.rs` — `CardProps` (line 153), `AlertProps` (line 317), `DescriptionListProps` (line 456), `ImageProps` (line 516), `PageHeaderProps` (line 797), `KanbanBoardProps` (line 863), `KanbanColumnProps` (line 852), `ActionCardVariant` (line 891 — reference for D-18 pattern)
- `ferro-json-ui/src/visibility.rs` — `Visibility` enum at line 45 (`#[serde(untagged)]`)
- `ferro-json-ui/src/catalog.rs` — `Catalog::validate` (line 637), BUILTIN_SPECS table (line 123 onward), Card entry (line 265), KanbanBoard entry (line 283), count assertion (line 1052)
- `ferro-json-ui/src/render/containers.rs` — `render_card` (line 27, body starts line 53 with hard-coded class string)
- `ferro-json-ui/src/render/atoms.rs` — `render_image` (line 365), `render_description_list` (line 563)
- `ferro-json-ui/src/render/mod.rs` — BUILTIN_TYPES (line 41), dispatch (line 178 `"Card" => containers::render_card`), count assertion (line 530)
- `ferro-json-ui/src/loader.rs` — `load_cached` (line 118), call to `Spec::from_json` (line 140), call to `global_catalog().validate` (line 141)
- `ferro-json-ui/src/resolve.rs` — `expand_directives` (line 137), pipeline doc comment (lines 110–146)
- `ferro-json-ui/src/action.rs` — `HttpMethod` enum (line 26, `rename_all = "UPPERCASE"`)
- `ferro-json-ui/src/lib.rs` — public re-exports (lines 49–86), `MAX_NESTING_DEPTH` re-export (line 84)
- `ferro-json-ui/src/plugin.rs` — `JsonUiPlugin` trait (line 66), confirms v2 plugin system intact
- `ferro-cli/src/commands/json_ui_migrate_v1.rs` — uppercase method emission (line 521)
- `ferro-mcp/src/tools/json_ui_catalog.rs` — 40-component assertion (line 290), expected-names list (lines 296–337)
- `ferro-mcp/src/tools/` — directory listing confirming absence of `json_ui_validate_spec.rs`
- `framework/src/json_ui/mod.rs` — `JsonUi::render_file` (line 161), `resolve` (line 48), `build_response` (line 81 title extraction at line 89)
- `framework/src/lib.rs` — confirms no v1 re-exports remain (only `pub use ferro_json_ui::{...}` for v2 surface)
- `CHANGELOG.md` — Phase 162 + 163 Unreleased entries (lines 6–61); confirms 40 built-in components + 2 plugins as of post-Phase-162
- gestiscilo specs `whatsapp.json` / `calendario/scan.json` — confirms `"type": "Plugin"` with `plugin_type: "InlineHtml"` shape (D-17 evidence)
- git history: commit `dbe5adaf refactor(115-02): strip v1 types from component.rs, delete view.rs, flip lib.rs re-exports` — confirms v1 surface deletion
- git history: commit `cb243597 feat(47-02): add Plugin variant` — confirms historical v1 Plugin shape
- `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md:228` — "Component::Plugin was a wart — a some-types-are-special backdoor" (D-17 framing)

### Secondary (MEDIUM confidence)

- gestiscilo `V7-RUNTIME-FRICTION.md` — 161 lines, page-by-page test results, F10 ships full Rust code for CardVariant
- gestiscilo `phases/138-.../FRICTION.md` — 338 lines, residual items in "Low-Impact Entries" and "Codebase-Wide Blast Radius" sections
- 162-CONTEXT.md + 163-CONTEXT.md — decisions D-01..D-25 (Phase 162) + D-01..D-13 (Phase 163) — verify what was already absorbed by predecessor phases

### Tertiary (LOW confidence — flagged for validation)

- A2 assumption (26 lowercase-method specs were hand-authored, not codemod output) — needs gestiscilo git log audit to confirm
- A6 assumption (D-17a `RawHtml` boundary holds against consumer drift) — depends on doc discipline; observable only post-ship
- A8 assumption (F5 cause is `{"expr": "foo"}` shape) — needs direct read of `clienti/list.json:47` to confirm

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — every cited dep is already in `Cargo.toml`; no new crates needed
- Architecture: HIGH — pipeline diagram traced through `loader.rs → spec.rs → catalog.rs → resolve.rs → render/mod.rs` with line numbers; D-16 reorder target verified
- Pitfalls: HIGH — Pitfalls 1, 3, 4, 6 verified against source. Pitfalls 2, 5 are convention/best-practice with HIGH workspace pattern grounding
- D-12..D-19 implementation sites: HIGH — every cited file:line verified
- v1-deletion audit (D-01..D-03): HIGH — v1 surface confirmed deleted; audit is documentation work, not detective work
- D-17a `Component::RawHtml` framing: MEDIUM-HIGH — Phase 115 D-01 evidence is solid; consumer-drift risk (A6) remains
- D-19/F2 codemod fix scope: MEDIUM — line 521 already shipped; depends on A2 (origin of 26 lowercase specs)

**Research date:** 2026-05-17
**Valid until:** 2026-06-17 (30 days — stable area; would shorten if `v12.0/json-ui-v2` sees further branch-level refactors)
