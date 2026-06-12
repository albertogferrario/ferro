# Phase 209: COMP-01 Slice A — Gestiscilo Migration (Browse + Process + Summarize) - Research

**Researched:** 2026-06-12
**Domain:** Cross-repo projection/intent migration — gestiscilo (consumer) + ferro (framework)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Equivalence evidence = before/after screenshots via Chrome DevTools MCP + short functional checklist (data fields shown, actions available, primary-use-case flow). HTML diffs optional supplementary only.
- **D-02:** Equivalence bar = functional parity for primary use case, not pixel-identity. Intentional visual deltas are documented; only a functional regression (missing data, missing action, broken flow) blocks merge.
- **D-03:** Strictly sequential — one entity per gestiscilo merge. Each entity is its own short-lived branch, merged to gestiscilo master before the next branch opens.
- **D-04:** No ferro API changes on master while a gestiscilo migration branch is open. Gaps recorded, worked around in gestiscilo, deferred to a follow-up ferro phase.
- **D-05:** When a migration hits a ServiceDef field with no clean mapping or a renderer output needing a workaround: note-and-workaround. Record gap in weakness note, apply smallest gestiscilo-side workaround. An empty weakness note fails the phase (SC#5).
- **D-06:** Default expectation = zero ferro source changes. Do not bump ferro version speculatively. Single publish at slice end only if a discovered gap forces a minimal, safe ferro fix.
- **D-07:** Selection criteria (locked): pick the clearest exemplar of each of Browse / Process / Summarize that (a) has a direct `JsonUi::render_file` call, (b) carries the least bespoke/one-off HTML, (c) maps to a model whose shape exercises the intent's defining signals. Prefer representative CRUD/list/dashboard entities over edge cases. Selection itself resolved at plan-time by reading gestiscilo source.
- **D-08:** All COMP-01 validation artifacts live in this ferro phase directory (`.planning/phases/209-comp-01-slice-a-gestiscilo-migration/`), linking to corresponding gestiscilo migration commit/PR.

### Claude's Discretion

- Exact screenshot tooling instance (chrome-devtools / -2 / -3), file naming for equivalence records, and the markdown shape of the weakness note — consistent with D-01/D-08.

### Deferred Ideas (OUT OF SCOPE)

- Full gestiscilo migration (~127 remaining views, ~66 models) — out of v13.0 scope.
- Ferro fixes for gaps discovered during the slice — captured in weakness note, addressed in a later v13.x phase.
- CRUD-handler proc macros — relocated to v13.1 Phase 212.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-01 | Three gestiscilo entities (Browse, Process, Summarize) migrated from `JsonUi::render_file` to `ServiceDef` + `JsonUiRenderer`, one-per-merge, with render-equivalence records and a weakness note | Entity selection (§1), wiring pattern (§2), ferro surface confirmation (§3), validation architecture (§4), and abstraction gap forecast (§5) below |
</phase_requirements>

---

## Summary

Gestiscilo is a multi-tenant jet ski rental SaaS built on ferro. It has 67 `JsonUi::render_file` call sites and **zero existing `ServiceDef`/`JsonUiRenderer` usage** — this slice establishes the wiring pattern from scratch. The three migration entities recommended below are: **Staff list** (Browse), **Orders kanban** (Process), and **Statistics dashboard** (Summarize). These are selected because each has a single, clean `JsonUi::render_file` call site; each view's JSON spec is 58–218 lines with no bespoke raw HTML exception in the primary path; and each backing model's field shape directly exercises the target intent's defining derive.rs signals.

A critical wiring gap exists before any migration code can compile: gestiscilo's `Cargo.toml` currently declares `ferro = { version = "0.2.54", features = ["json-ui", "theme"] }` and `ferro-json-ui = "0.2.54"` but does **not** include the `projections` feature. `ServiceDef`, `derive_intents`, and `JsonUiRenderer` are all gated behind `features = ["projections"]` in ferro-rs 0.2.54. Adding `"projections"` to the existing ferro feature list is a one-line `Cargo.toml` change that does not require a ferro version bump — it activates already-published code. This is the Wave 0 prerequisite for all three migrations.

**Primary recommendation:** Start with Staff list (Browse) to establish the wiring pattern, then Orders kanban (Process), then Statistics dashboard (Summarize). Staff is the most structurally clean model; Orders is the most operationally important view; Statistics is the richest Summarize signal and the most likely source of abstraction-gap friction (server-side SVG chart field has no clean `FieldMeaning` equivalent).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ServiceDef construction | API / Backend (gestiscilo controller) | — | `ServiceDef::from_model()` or manual builder runs server-side at handler time |
| Intent derivation | API / Backend | — | `derive_intents(&svc)` called server-side before rendering |
| HTML spec generation | API / Backend (JsonUiRenderer) | — | `JsonUiRenderer::render()` produces a `Spec`; `JsonUi::render()` turns it to HTML |
| Rendered HTML delivery | API / Backend | Browser | Ferro's JSON-UI runtime (FERRO_RUNTIME_JS) handles client-side tab/SSE interactions |
| Before/after screenshot capture | Browser / Client | — | Chrome DevTools MCP captures the live running app |
| Equivalence record storage | — (static artifact) | — | Markdown files committed to this ferro phase directory (D-08) |

---

## 1. Entity Selection (PRIMARY OUTPUT)

### 1.1 Browse — Staff List

**Controller file:** `app/src/controllers/staff/list.rs`
**Handler:** `pub async fn index(req: Request) -> Response` (line 51)
**Render_file call:** `JsonUi::render_file("src/views/staff/list.json", data)` (line 110)
**View spec size:** 142 lines [VERIFIED: file read]
**Backing model:** `app/src/models/entities/staff.rs` — `Staff` model

**Model field shape** [VERIFIED: file read]:
```
id: i64 (PK)
tenant_id: i64
slug: String
name: String         ← EntityName signal
bio: Option<String>  ← FreeText signal (optional)
avatar_url: Option<String>  ← ImageUrl signal (optional)
sort_order: i32      ← Quantity signal
active: bool         ← Status-adjacent
deleted_at: Option<DateTimeWithTimeZone>
created_at: DateTimeWithTimeZone
updated_at: DateTimeWithTimeZone
```

**derive.rs signals this model fires:**
- `EntityName` (name) → Browse weight 0.20
- `FreeText` (bio, optional) → Focus weight 0.25
- `ImageUrl` (avatar_url, optional) → Focus weight 0.25
- `Quantity` (sort_order) → Summarize weight 0.30
- Baseline Browse 0.10
- Mostly read-only (active, avatar, sort_order not in the write path for the LIST view) → Summarize weight 0.20

Raw primary intent: Browse wins at confidence ~0.4-0.5 with EntityName + baseline. An `IntentHint::Primary(Intent::Browse)` override in the ServiceDef is likely needed to lock Browse as primary for this list view, because Focus (bio + avatar_url) competes.

**Why clearest Browse exemplar:**
- Staff list is a canonical DataTable listing entities with Name + Status + sortable row
- The view JSON is 142 lines, cleanest structure in the codebase with no RawHtml or bespoke inline-SVG
- Actions are standard row-CRUD (View, Edit, Toggle Active, Delete) — maps directly to `ServiceDef.actions`
- Flat model (no M2M, no join tables, no pre-aggregated computed fields)
- No period-switching, no Stripe coupling, no computed display strings beyond `if s.active { "Attivo" } else { "Disattivato" }`

**Bespoke-HTML assessment:** LOW. The view uses DataTable + PageHeader + Toast + EmptyState — all standard components. The only controller-side computation is `avatar_initial_color(s.id)` (deterministic color hash) and `signed_url(key, 3600)` (presigned S3 URL). These are data values injected into the spec, not bespoke HTML. The view itself has no `RawHtml` component.

**Backup candidate for Browse:** `clienti.rs::index` → `src/views/clienti/index.json` (124 lines, similar DataTable shape, but the `clienti.rs` controller is 1300+ lines with 9 find_for_tenant calls — higher overall complexity, though the `index` handler itself is clean).

---

### 1.2 Process — Orders Kanban

**Controller file:** `app/src/controllers/cassa/orders.rs`
**Handler:** `pub async fn index(_req: Request) -> Response` (line 158)
**Render_file call:** `JsonUi::render_file("src/views/cassa/orders_index.json", data)` (line 213)
**View spec size:** 58 lines [VERIFIED: file read]
**Backing model:** `app/src/models/entities/orders.rs` — `Order` model

**Model field shape** [VERIFIED: file read]:
```
id: i64 (PK)
tenant_id: i64
order_number: i32
status: String          ← Status signal (PRIMARY Process driver)
total_cents: i64        ← Money signal → Summarize
created_at: DateTimeWithTimeZone
updated_at: DateTimeWithTimeZone
payment_method: Option<String>
paid_at: Option<DateTimeWithTimeZone>
receipt_token: Option<String>
customer_name: Option<String>   ← EntityName signal → Browse
payment_state: String           ← Status signal
reservation_expires_at: Option<DateTimeWithTimeZone>
stripe_session_id/stripe_payment_intent_id: Option<String>
email: Option<String>
```

**State machine** (from `order_status.rs`) [VERIFIED: file read]:
- States: `Confermato → InCorso → Rientrato → Chiuso` (forward), `Annullato` (terminal from any)
- Branching from `Confermato` (advance OR cancel), `InCorso` (advance, revert, or cancel), `Rientrato` (advance, revert, cancel)
- Guards: `can_cancel()`, `can_pay()`, `is_editable()` — every non-trivial transition has a precondition

**derive.rs signals this model fires:**
- `Status` (status, payment_state) → Track weight 0.50 (two status fields)
- `Money` (total_cents) → Summarize weight 0.30
- `EntityName` (customer_name) → Browse weight 0.20
- State machine with branching states (Confermato has 2 outgoing: advance + cancel) → Process weight 0.55
- Guarded transitions (can_cancel, can_pay) → Process weight ~0.40
- Transition triggers (advance, revert, cancel actions) → Process weight ~0.25
- Baseline Browse 0.10

Raw primary intent: **Process wins clearly** due to guarded transitions + branching state machine. This is the strongest Process signal in the gestiscilo entity catalog.

**Why clearest Process exemplar:**
- The Orders kanban IS the canonical Process view: 4 columns (Confermato/InCorso/Rientrato/Chiuso), forward-only workflow with reversals, guarded per-card actions
- The view JSON is only 58 lines — the simplest spec file in the list (KanbanBoard component handles the column layout; card shape is already data-driven via `$each` and `$data`)
- The `orders.rs::index` handler is clean: it calls `Order::find_all_kanban(business.id)`, maps to JSON rows, and calls `build_status_kanban_columns()`. No file I/O, no per-row signed URLs, no external service calls.
- `build_order_kanban_actions` and `build_order_kanban_description` are already extracted shared helpers — they compute per-card action sets from `OrderStatus`, which maps cleanly to `ServiceDef.actions` + `StateMachine` in the ServiceDef.

**Bespoke-HTML assessment:** VERY LOW. The 58-line view spec uses KanbanBoard (standard component), Card ($each), DropdownMenu ($each) with data-driven items array. No RawHtml, no inline SVG, no period switching. The kanban column structure is entirely data-driven (`kanban_columns` array injected by the controller helper).

**Backup candidate for Process:** `calendario/bookings.rs` kanban view — but the bookings controller is 3900+ lines with 8 render_file calls and extreme complexity. Not suitable for Slice A.

---

### 1.3 Summarize — Statistics Dashboard

**Controller file:** `app/src/controllers/statistiche.rs`
**Handler:** `pub async fn index(req: Request) -> Response` (line 33)
**Render_file call:** `JsonUi::render_file("src/views/statistiche/index.json", data)` (line 211)
**View spec size:** 218 lines [VERIFIED: file read]
**Backing model:** `app/src/models/analytics.rs` — `SummaryStats`, `TopProduct`, `RevenueTrend` (query-result structs, not ORM entities)

**Model field shape (SummaryStats)** [VERIFIED: file read]:
```
total_revenue_cents: i64    ← Money signal
order_count: i64            ← Quantity signal
average_order_cents: i64    ← Money signal
```

**derive.rs signals the analytics ServiceDef fires:**
- `Money` (total_revenue_cents, average_order_cents × 2) → Summarize weight 0.60
- `Quantity` (order_count) → Summarize weight 0.30
- Mostly read-only (all fields are read-only computed values) → Summarize weight 0.20
- Baseline Browse 0.10

Raw primary intent: **Summarize wins decisively** (Money + Quantity + read-only). This is the most unambiguous Summarize signal in gestiscilo.

**Why clearest Summarize exemplar:**
- The statistics page is the canonical Summarize view: three StatCard components showing revenue, order count, and average order value — exactly Money + Quantity field types
- The SummaryStats struct is trivially representable as a ServiceDef: three read-only fields, no writable fields, no state machine, no relationships
- The view spec (218 lines) is medium-complexity but all standard components (StatCard, DataTable, DescriptionList, Card, Tabs, EmptyState)

**Bespoke-HTML assessment:** MEDIUM. The key friction source is `chart_svg` — the revenue trend bar chart is server-side rendered as inline SVG via `bar_chart_svg()` in the controller. The view injects it via:
```json
"chart_image": {
  "type": "Image",
  "props": { "inline_svg": { "$data": "/chart_svg" } }
}
```
The `Image` component with `inline_svg` prop handles this in ferro-json-ui. However, the chart data itself (`trend_raw`, zero-filling, period-aware label formatting) has no `FieldMeaning` equivalent — it is a computed derived dataset, not a model field. This is the **highest-probability abstraction gap for SC#5**. The ServiceDef will not cleanly express "a time-series revenue chart rendered as SVG" through structural field meanings. The workaround: pass chart data as opaque display strings in the `data` object alongside the ServiceDef render, or treat the chart as a supplementary non-projection element (see §5).

**Backup candidate for Summarize:** `home_stats.rs`-driven dashboard widget (revenue + booking stats in `dashboard.rs`) — but the dashboard controller is 600+ lines with complex module-switching logic, SSE integration, and WhatsApp notification rendering. The standalone `statistiche.rs` controller is far cleaner as a first migration.

---

### 1.4 Selection Summary Table

| Slot | Entity | Controller | Handler | View Spec | View Lines | Model | Primary Intent Confidence | Bespoke-HTML Risk |
|------|--------|-----------|---------|-----------|-----------|-------|--------------------------|------------------|
| Browse | Staff list | `staff/list.rs` | `index` | `staff/list.json` | 142 | `entities/staff.rs` | HIGH (EntityName + baseline, IntentHint needed) | LOW |
| Process | Orders kanban | `cassa/orders.rs` | `index` | `cassa/orders_index.json` | 58 | `entities/orders.rs` | VERY HIGH (guarded state machine) | VERY LOW |
| Summarize | Statistics dashboard | `statistiche.rs` | `index` | `statistiche/index.json` | 218 | `analytics.rs` SummaryStats | VERY HIGH (Money+Quantity+read-only) | MEDIUM (SVG chart field) |

**Recommended migration order:** Staff → Orders → Statistics.
- Staff establishes the basic wiring pattern cleanly.
- Orders validates the Process intent + state machine mapping with minimal bespoke risk.
- Statistics is last because it is the most likely to surface a genuine abstraction gap (the chart data), which becomes the SC#5 weakness note.

---

## 2. Wiring Pattern — Before → After Handler Transformation

### 2.1 Prerequisite: Cargo.toml change (Wave 0)

Before any handler can use `ServiceDef` or `JsonUiRenderer`, gestiscilo's `Cargo.toml` must enable the `projections` feature on ferro-rs. [VERIFIED: current Cargo.toml and ferro framework/Cargo.toml read]

```toml
# Before (current gestiscilo Cargo.toml):
ferro = { version = "0.2.54", package = "ferro-rs", features = ["json-ui", "theme"] }

# After:
ferro = { version = "0.2.54", package = "ferro-rs", features = ["json-ui", "theme", "projections"] }
```

This activates `ferro_projections` re-exports (`ServiceDef`, `derive_intents`, `DataType`, `FieldMeaning`, `IntentHint`) and `ferro_json_ui` re-exports (`JsonUiRenderer`, `VisualContext`, `RenderMode`) through the framework crate. No version bump required — all of this exists at 0.2.54. [VERIFIED: framework/Cargo.toml features = ["projections"]]

The `ferro-json-ui` `projections` feature (already published at 0.2.54, gated as `optional` in ferro-json-ui/Cargo.toml) is activated transitively through the ferro-rs `projections` feature which sets `ferro-json-ui/projections`.

---

### 2.2 Before pattern (current gestiscilo)

Using Staff list as the canonical example:

```rust
// BEFORE: staff/list.rs index handler
#[handler]
pub async fn index(req: Request) -> Response {
    let business = resolve_tenant().await?;
    // ... sidebar, notifications, flash ...
    let staff_rows = Staff::find_all_for_tenant(business.id).await?;
    // ... per-row JSON construction with signed URLs + initials ...
    let data = json!({
        "_sidebar": ..., "_header": ..., "_sse_url": ...,
        "staff": rows_json,
        "is_empty": is_empty,
        "has_staff": !is_empty,
        "flash_message": flash_message,
        "flash_variant": flash_variant,
    });
    JsonUi::render_file("src/views/staff/list.json", data)  // ← REPLACED
}
```

### 2.3 After pattern (projection-driven)

```rust
// AFTER: staff/list.rs index handler (projection-driven)
use ferro::{
    derive_intents, DataType, FieldMeaning, IntentHint, Intent,
    JsonUiRenderer, ServiceDef, VisualContext,
};
use ferro_projections::render::Renderer;

#[handler]
pub async fn index(req: Request) -> Response {
    let business = resolve_tenant().await?;
    // ... sidebar, notifications, flash ... (unchanged)
    let staff_rows = Staff::find_all_for_tenant(business.id).await?;

    // 1. Build the ServiceDef for the Staff entity (Browse intent)
    let service = ServiceDef::new("staff")
        .display_name("Staff")
        .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .optional_field("bio", DataType::String, FieldMeaning::FreeText)
        .optional_field("avatar_url", DataType::String, FieldMeaning::ImageUrl)
        .field("sort_order", DataType::Integer, FieldMeaning::Quantity)
        .field("active", DataType::Boolean, FieldMeaning::Status)
        .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        // Force Browse as primary — bio+avatar_url would pull toward Focus otherwise
        .intent_hint(IntentHint::Primary(Intent::Browse));

    // 2. Derive intents and render
    let intents = derive_intents(&service);
    let renderer = JsonUiRenderer;
    let spec = renderer
        .render(&service, &intents, &VisualContext::default())
        .map_err(|e| error_response(500, &format!("Projection render error: {e}")))?;

    // 3. Inject runtime data (row array, flash, sidebar, header)
    //    The rendered spec's data fields are populated from the existing rows_json.
    let rows_json: Vec<ferro::serde_json::Value> = /* same as before */;
    let merged_spec = spec.merge_data(json!({
        "_sidebar": ..., "_header": ..., "_sse_url": ...,
        "staff": rows_json,
        "has_staff": !is_empty,
        "flash_message": flash_message,
    }));

    let render_data = merged_spec.data.clone();
    JsonUi::render(&merged_spec, &render_data)  // ← render Spec directly, not render_file
}
```

### 2.4 Key transformation points

| Aspect | Before | After |
|--------|--------|-------|
| View source | `render_file("src/views/staff/list.json", data)` reads JSON from disk | `JsonUiRenderer::render()` generates `Spec` in memory from `ServiceDef` |
| Spec authoring | Manual JSON file maintained by hand | Structural derivation from model field types and meanings |
| Runtime data injection | Passed as flat JSON to `render_file` | `spec.merge_data(json!({...}))` then `JsonUi::render(&spec, &data)` |
| Intent evidence | None — the view is opaque | `derive_intents(&service)` produces verifiable `IntentScore` list |
| Equivalence evidence | None | Before/after screenshots + `derive_intents()` assertion |

Note: The `spec.merge_data()` method is used by `dashboard.rs` today (lines 595-598 in `controllers/dashboard.rs`), confirming the pattern is already established in gestiscilo for the dynamic-spec path. [VERIFIED: controllers/dashboard.rs read]

---

## 3. Ferro Surface Confirmation

### 3.1 Published version check

Gestiscilo pins `ferro = "0.2.54"`. Current published ferro workspace version is `0.2.54`. [VERIFIED: gestiscilo Cargo.toml, ferro Cargo.toml, `cargo search ferro-rs`]

### 3.2 ServiceDef::from_model() availability

`ServiceDef::from_model(meta: &ModelMetadata)` is defined in `ferro-projections/src/service.rs` lines 300-327 and is part of the `ferro-projections` crate at 0.2.54. It derives DataType from column_type strings and FieldMeaning from field names via `infer_meaning()`. [VERIFIED: file read]

For all three migration entities, the manual `ServiceDef::new("...").field(...)` builder pattern is preferred over `ServiceDef::from_model()` because:
- The Staff, Order, and SummaryStats models have fields requiring deliberate meaning overrides (e.g., `payment_state` on Order could auto-infer as generic Status, but manual builder makes the intent explicit)
- `ServiceDef::from_model()` requires populating `ModelMetadata` from SeaORM entities — this is doable but adds ceremony for the first migration where explicitness aids the equivalence record

### 3.3 JsonUiRenderer availability

`JsonUiRenderer` is defined in `ferro-json-ui/src/projection/mod.rs` and exported from `ferro-json-ui` under the `projections` feature flag. It implements `Renderer` (from `ferro-projections::render`). [VERIFIED: file read]

The `projections` feature is re-exported through `ferro-rs` as shown in `framework/Cargo.toml:18`:
```toml
projections = ["dep:ferro-projections", "dep:ferro-json-ui", "ferro-json-ui/projections"]
```

**Gap:** Gestiscilo currently declares `ferro-json-ui = "0.2.54"` as a **direct** dependency WITHOUT the `projections` feature. If gestiscilo code uses `ferro_json_ui::JsonUiRenderer` directly (not through `ferro::JsonUiRenderer`), it needs the feature added to the direct dep too:
```toml
ferro-json-ui = { version = "0.2.54", features = ["projections"] }
```
Or alternatively, use only the `ferro::` re-export path which transitively enables the feature. The simplest and safest approach: add `"projections"` to the `ferro` dep and use `ferro::JsonUiRenderer`, not `ferro_json_ui::JsonUiRenderer` directly.

### 3.4 Renderer trait usage pattern

The `Renderer` trait is defined in `ferro-projections::render` (re-exported as `ferro::Renderer`). Usage:
```rust
use ferro_projections::render::Renderer;  // or: use ferro::Renderer;
let result = renderer.render(&service, &intents, &ctx);
```
[VERIFIED: ferro-json-ui/src/projection/mod.rs read]

### 3.5 Version mismatch risk

**No version mismatch risk.** The `projections` feature activation adds a direct zero-cost feature flag to already-published 0.2.54 code. No new crates need to be published for the migration to proceed. [ASSUMED: published 0.2.54 includes the projection module content as read from source — cargo search confirms 0.2.54 is the current published version]

---

## 4. Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `#[test]` + manual checklist markdown files |
| Config file | None — ferro-side tests in `ferro-projections/tests/catalog.rs` (Phase 207 baseline); gestiscilo-side equivalence records in this ferro phase directory |
| Quick run command | `cargo test -p ferro-projections --test catalog -- browse` (per-intent filter) |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-01 | Staff ServiceDef derives Browse as primary intent | unit (Rust) | `cargo test -p ferro-projections -- staff_browse_intent` | ❌ Wave 0 |
| COMP-01 | Orders ServiceDef derives Process as primary intent | unit (Rust) | `cargo test -p ferro-projections -- orders_process_intent` | ❌ Wave 0 |
| COMP-01 | SummaryStats ServiceDef derives Summarize as primary intent | unit (Rust) | `cargo test -p ferro-projections -- stats_summarize_intent` | ❌ Wave 0 |
| COMP-01 | Staff: before and after screenshots show functional parity | visual + manual | Chrome DevTools MCP capture at migration PR time | ❌ Wave 0 |
| COMP-01 | Orders: before and after screenshots show functional parity | visual + manual | Chrome DevTools MCP capture at migration PR time | ❌ Wave 0 |
| COMP-01 | Stats: before and after screenshots show functional parity | visual + manual | Chrome DevTools MCP capture at migration PR time | ❌ Wave 0 |
| COMP-01 | At least one abstraction gap named in weakness note | manual | Human review of weakness note | ❌ Wave 0 |

### Per-Entity Functional Checklist Template

Each migration's equivalence record (`.planning/phases/209-comp-01-slice-a-gestiscilo-migration/EQUIV-{entity}.md`) MUST assert:

1. **Data fields shown:** All data columns visible in the before screenshot appear in the after screenshot (field names may differ; data values must match).
2. **Actions available:** All row actions (View, Edit, Delete, status transitions) reachable in before state are reachable in after state.
3. **Primary-use-case flow:** The most common operator action (e.g., "click a staff row to view detail") works in the migrated view.
4. **Intent confirmation:** `derive_intents(&service)[0].intent == Expected` assertion passes in a `#[test]`.
5. **Intentional visual deltas documented:** Any layout/markup differences between before and after are listed explicitly. Unlisted differences block the merge.

### Sampling Rate

- **Per migration commit:** Run `derive_intents()` assertion for that entity's ServiceDef
- **Per wave merge (gestiscilo):** Chrome DevTools MCP screenshot capture + functional checklist review
- **Phase gate:** All three entities migrated, all three equivalence records filed, weakness note non-empty, before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-projections/tests/` — add three canonical gestiscilo ServiceDef fixtures (staff, order, stats) with `derive_intents()` assertions. These can live in `catalog.rs` (Phase 207 file) as a new `real_world_slice_a` sub-module, or in a new `tests/gestiscilo.rs` file.
- [ ] Gestiscilo `Cargo.toml` — enable `projections` feature: `ferro = { ..., features = ["json-ui", "theme", "projections"] }`
- [ ] Three equivalence record markdown files (stubs): `EQUIV-staff-browse.md`, `EQUIV-orders-process.md`, `EQUIV-stats-summarize.md` in this phase directory

---

## 5. Likely Abstraction Gaps (SC#5 Instrumentation)

The migration MUST find and name at least one real gap (D-05, SC#5). Based on the entity analysis, here are the three highest-probability friction points:

### Gap 1: Server-side SVG chart (Statistics — HIGH probability)

**What will happen:** The `statistiche.rs` controller computes a revenue trend bar chart as raw SVG text via `bar_chart_svg()`. The view injects it as `inline_svg` into an `Image` component. The `ServiceDef` has no `FieldMeaning` that maps to "server-rendered chart SVG." When the Statistics ServiceDef is built, the chart SVG field will either:
- Be omitted (the rendered projection won't include a chart), or
- Be represented as `FieldMeaning::FreeText` with `DataType::String` (lossy — loses the "chart" semantic)

The `FieldMeaning` enum in `ferro-projections/src/field.rs` does not include a `Chart` or `Visualization` variant. [ASSUMED: based on the field.rs module structure and `derive.rs` signal list — field.rs not directly read but its public API is inferred from service.rs and derive.rs usage]

**Expected workaround:** Pass chart SVG as an opaque `data` field (not in the ServiceDef) and merge it into the rendered spec's data object post-render via `spec.merge_data()`. The `ServiceDef` describes the summary stats (`total_revenue_cents`, `order_count`, `average_order_cents`); the chart is an auxiliary visualization that the controller keeps generating server-side as today.

**Weakness note content:** "No `FieldMeaning` variant for server-rendered chart/visualization data; chart fields pass through as opaque `data` map entries outside the ServiceDef, losing structural intent signal."

### Gap 2: Per-row signed URL computation (Staff — MEDIUM probability)

**What will happen:** The Staff list controller computes `signed_url(key, 3600)` per row to generate presigned S3 URLs for avatar images. This is async I/O per row that produces a display string. The ServiceDef has `FieldMeaning::ImageUrl` for `avatar_url`, but `ImageUrl` in the projection context implies a static URL stored in the DB, not a computed presigned URL generated at render time.

**Expected workaround:** The per-row signed URL computation stays in the controller, and the `rows_json` array (with computed `avatar_url` values) is merged into the rendered spec's data map as today. The ServiceDef declares `avatar_url` as `FieldMeaning::ImageUrl` for intent-derivation purposes, but runtime data population remains controller-side.

**Weakness note content:** "Computed presigned URLs (S3 signed at render time) have no structural hook in ServiceDef — `FieldMeaning::ImageUrl` signals the field's semantic but cannot express the URL computation. Controllers retain the per-row URL signing loop; the projection abstraction is structurally incomplete for storage-backed image fields."

### Gap 3: Kanban column construction (Orders — LOW-MEDIUM probability)

**What will happen:** The Orders kanban index calls `build_status_kanban_columns()` with an explicit ordered list of statuses and column labels. The `ServiceDef.state_machine` can declare states and transitions, but the mapping from state names to kanban column labels (Italian: "Confermati", "In corso", etc.) has no clean analogue in the ServiceDef API. The KanbanBoard component requires a `data_path` pointing to a pre-structured column array.

**Expected workaround:** The kanban column array construction stays in the controller helper. The ServiceDef expresses the state machine for intent-derivation purposes (ensuring Process derivation), but the kanban column data remains controller-generated. The rendered spec's `data` map receives `kanban_columns` as today.

**Weakness note content:** "State machine column-label mapping (kanban display names for workflow states) has no projection-level representation. `ServiceDef.state_machine` declares state names for intent derivation but cannot encode display labels or column ordering; these remain hardcoded in controller helpers."

---

## 6. Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Intent derivation from model fields | Custom scoring logic | `derive_intents(&service)` | Already implemented with 5 analyzers, proptest-verified |
| HTML rendering from ServiceDef | Custom template engine | `JsonUiRenderer::render()` + `JsonUi::render()` | Already handles all 42 built-in components + layout |
| Feature activation | Forking ferro-json-ui | Add `"projections"` to `ferro` features in Cargo.toml | Feature-gated at compile time, no source change needed |
| Spec data merging | Rebuild entire spec structure | `spec.merge_data(json!({...}))` | Already used in gestiscilo dashboard controller |
| Screenshot capture | Manual browser screenshots | Chrome DevTools MCP (`mcp__chrome-devtools__*`) | Programmatic capture with reproducible coordinates |

---

## 7. Risks and Open Questions

### Risk 1: `IntentHint::Primary(Browse)` needed for Staff (MEDIUM)

The Staff model has `bio: Option<String>` (FreeText) and `avatar_url: Option<String>` (ImageUrl). derive.rs gives FreeText and ImageUrl each 0.25 weight per field, while EntityName gets 0.20. With 2 focus-signal fields vs 1 browse-signal field plus the 0.10 baseline for Browse, Browse may not win without a hint. An `IntentHint::Primary(Intent::Browse)` override resolves this cleanly and is the standard override mechanism. But this means the Staff ServiceDef's Browse classification is explicitly forced, not structurally derived — which is itself a weak-signal finding worth noting.

### Risk 2: `Spec::merge_data()` API surface (LOW)

The migration uses `spec.merge_data(json!({...}))` to inject runtime data (rows, sidebar, header) into the rendered spec. This method is used in `dashboard.rs` today, confirming it exists at 0.2.54. However, `merge_data` behavior (shallow merge vs deep merge, handling of existing `data` keys) needs to be verified for each migration's data shape. [ASSUMED for Statistics: `chart_svg` is a new key not in the ServiceDef's derived spec — shallow merge should add it cleanly]

### Risk 3: Statistics view period-switching Tabs (LOW)

The Statistics view has a period switcher (`Tabs` component with `default_tab: { "$data": "/period_str" }`). The ServiceDef derivation does not know about period switching — this is a controller-side concern. The rendered projection spec will not include the Tabs component unless added manually post-render. The workaround: the Statistics migration may require `spec.merge_data()` supplemented with `spec.elements.insert("period_tabs", Element::...)` to inject the Tabs element. Or the Statistics migration may be structured as: ServiceDef renders the stat cards only, and the wrapper (Tabs, chart card) is expressed as a supplementary spec layer merged on top. This is a structural design decision the planner needs to resolve.

**Open question for planner — RESOLVED (Plan 04):** For the Statistics migration, should the `ServiceDef` + `JsonUiRenderer` render ONLY the stat cards (StatCard × 3), with the chart and Tabs expressed as supplementary data-driven elements outside the projection? Or should the full Statistics page be re-expressed as a composite spec that uses the projection for the card section only? **Resolution:** stat-cards-only — the projection renders the StatCard section; chart/Tabs/trend-table are opaque `merge_data` passthroughs. This is the cleanest first-slice structure and the deliberate SC#5 gap surface (the SVG chart has no `FieldMeaning`). See `209-04-PLAN.md` `<objective>`.

### Risk 4: First-migration friction budget (SC#5 gate)

The SC#5 requirement that "an empty weakness note fails the phase" means the migration MUST find friction. Based on §5, the Statistics SVG chart gap is the highest-probability genuine abstraction gap. If Staff and Orders migrate perfectly with no friction, Statistics is the deliberate friction target. The planner should sequence Statistics last so the weakness note is complete before the phase is closed.

---

## Common Pitfalls

### Pitfall 1: Using `render_file` path alongside `JsonUiRenderer`
**What goes wrong:** Calling `JsonUi::render_file(path, data)` and `JsonUiRenderer::render()` side by side is not an either/or — `render_file` loads a JSON spec from disk, while `JsonUiRenderer::render()` generates a spec in memory. After migration, the original `render_file` call must be **deleted**, not left as a fallback.
**How to avoid:** Each migration is a full replacement of the `render_file` call, consistent with CLAUDE.md's "delete old code completely" principle.

### Pitfall 2: Forgetting the `projections` feature flag
**What goes wrong:** `ServiceDef`, `derive_intents`, `JsonUiRenderer` are all `#[cfg(feature = "projections")]` — they do not compile without the feature. Gestiscilo's current Cargo.toml lacks this feature.
**How to avoid:** Wave 0 task enables the feature before any migration code is written.

### Pitfall 3: `derive_intents()` not matching expected intent without a hint
**What goes wrong:** The Staff model's bio+avatar fields pull toward Focus. An agent writing the Staff migration might observe that `derive_intents()[0].intent == Intent::Focus` and be confused.
**How to avoid:** Add `IntentHint::Primary(Intent::Browse)` to the Staff ServiceDef. Document the hint rationale in the equivalence record and the weakness note.

### Pitfall 4: Treating the migration as a JSON spec rewrite
**What goes wrong:** Manually replicating the exact JSON spec elements from `staff/list.json` inside the `ServiceDef` builder. This defeats the purpose — the migration should use what `JsonUiRenderer` generates, not reproduce the existing spec.
**How to avoid:** Call `JsonUiRenderer::render()`, inspect what Spec it generates, then compare to the before spec. Differences are either expected (intent-template-driven layout) or abstraction gaps to record.

---

## Standard Stack

### Core (already in gestiscilo at 0.2.54)
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| `ferro-rs` | 0.2.54 | Framework — `json-ui`, `theme` features already active | Add `projections` feature |
| `ferro-json-ui` | 0.2.54 | JSON-UI spec types + render pipeline | Add `projections` feature |

### Activated by feature flag (zero new dependencies)
| Library | Version | Purpose | Activation |
|---------|---------|---------|------------|
| `ferro-projections` | 0.2.54 | `ServiceDef`, `derive_intents`, `Renderer` trait | `ferro` `projections` feature |
| `ferro-theme` | 0.2.54 | `ThemeTemplates` for `VisualContext` | `ferro-json-ui` `projections` feature (transitive) |

**Installation (gestiscilo Cargo.toml edit — no new crate downloads):**
```toml
# Change this line only:
ferro = { version = "0.2.54", package = "ferro-rs", features = ["json-ui", "theme", "projections"] }
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `JsonUi::render_file(path, data)` — spec on disk | `ServiceDef` + `JsonUiRenderer::render()` — spec in memory | v11.5 (Phase 135) | Spec is structural, introspectable, intent-annotated |
| Manual JSON spec authoring | `ServiceDef` builder API | v9.0 (Phase 84) | Fields typed with meaning; derive_intents() validates structure |
| Renderer in ferro-projections | Renderer in output crate (ferro-json-ui) | v11.5 (Phase 134) | Modality-agnostic Renderer trait; no ferro-projections→ferro-theme dep |

**Not deprecated:**
- `JsonUi::render_file` is NOT deprecated — it remains valid for views that cannot or should not use the projection pipeline. The migration replaces it selectively for entities that map cleanly to ServiceDef.

---

## Environment Availability

This phase is purely code/config changes in the gestiscilo repo plus artifact files in the ferro phase directory. No new external tools required.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Chrome DevTools MCP | Before/after screenshot capture | ✓ | via `~/.claude.json` mcp config | Manual screenshot |
| gestiscilo server (local) | Screenshot capture of running app | — | Per CLAUDE.md: "Server is always ran by the user" | N/A — operator runs server |
| cargo build (gestiscilo) | Verifying compilation after Cargo.toml edit | ✓ | rustc 1.88.0 | — |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Published ferro 0.2.54 includes `ServiceDef::from_model()`, `JsonUiRenderer`, and `derive_intents()` as read from local source tree | §3 | If the local source tree is ahead of what's published (local edits), a ferro version bump is needed |
| A2 | `spec.merge_data()` does a shallow merge that adds new keys without overwriting existing ones | §2.3, §7 Risk 2 | If merge_data overwrites, runtime data injection for Statistics (chart_svg, period_str) could fail silently |
| A3 | `ferro-projections/src/field.rs` does not contain a `Chart` or `Visualization` FieldMeaning variant | §5 Gap 1 | If a Chart variant exists, the SVG gap narrows |
| A4 | The `VisualContext::default()` with `RenderMode::Display` generates a Browse-compatible list layout for Staff (DataTable-like output, not a form) | §2.3 | If the default Browse layout from `intent_layout.rs` is not DataTable but something else, visual parity may require a different RenderMode or IntentHint |

**Note on A1:** `cargo search ferro-rs` returned `ferro-rs = "0.2.54"` and the local workspace version is `0.2.54`, so the source reflects what is published. If any post-0.2.54 commits have been made without a publish, those would not be available via crates.io. This risk is LOW given the phase constraint D-06 (zero ferro changes expected).

---

## Sources

### Primary (HIGH confidence)
- Local ferro source tree at 0.2.54: `ferro-projections/src/derive.rs`, `ferro-projections/src/service.rs`, `ferro-projections/src/intent.rs`, `ferro-json-ui/src/projection/mod.rs`, `framework/Cargo.toml` — verified via Read tool
- Local gestiscilo source tree: `app/Cargo.toml`, `src/controllers/staff/list.rs`, `src/controllers/cassa/orders.rs`, `src/controllers/statistiche.rs`, `src/models/entities/staff.rs`, `src/models/entities/orders.rs`, `src/models/analytics.rs`, `src/models/order_status.rs`, `src/views/staff/list.json`, `src/views/cassa/orders_index.json`, `src/views/statistiche/index.json` — verified via Read tool
- `cargo search ferro-rs` → 0.2.54 confirmed as published version

### Secondary (MEDIUM confidence)
- `.planning/phases/202-adopt-ferro-crud-macros/202-EVIDENCE.md` — controller duplication survey, grep counts accurate as of gestiscilo HEAD `efe4f8d7` on 2026-06-12
- `.planning/phases/209-comp-01-slice-a-gestiscilo-migration/209-CONTEXT.md` — locked decisions D-01..D-08

### Tertiary (LOW confidence)
- `ferro-projections/src/field.rs` FieldMeaning variants not directly read — inferred from `derive.rs` signal constants and `service.rs` usage patterns [A3]

---

## Metadata

**Confidence breakdown:**
- Entity selection: HIGH — verified from controller source, model source, view JSON, and derive.rs signal analysis
- Wiring pattern: HIGH — verified from existing gestiscilo dashboard.rs usage of `spec.merge_data()` and ferro-json-ui projection module
- Ferro surface: HIGH — verified from published 0.2.54 source + cargo search; one medium-risk assumption (A1)
- Validation architecture: HIGH — derived from CONTEXT.md D-01/D-02 + verified test infrastructure
- Abstraction gaps: MEDIUM — reasoning from source analysis; actual gaps confirmed only during execution

**Research date:** 2026-06-12
**Valid until:** 2026-07-12 (stable ferro API; gestiscilo controller drift is possible but unlikely for these specific handlers)
