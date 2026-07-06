# Gestiscilo Migration Brief — COMP-01 Slice A (Browse + Process + Summarize)

> **Hand-off document.** The code change for this slice lives in the **gestiscilo repo**
> (`/Users/alberto/repositories/gestiscilo-it/app`), so it is planned and executed in a
> **gestiscilo GSD session**, not from ferro. This brief is ferro's research output
> (entity selection + wiring + constraints) packaged so a gestiscilo session can turn it
> into its own phase. Ferro phase 209 is validation-only and *consumes* this work's outputs
> (see `209-02-PLAN.md`).
>
> Source of the analysis below: `209-RESEARCH.md` (§1 entity selection, §2 wiring, §5 gaps),
> resolved against gestiscilo source.

## Why this is a separate gestiscilo phase

Ferro must not mutate the gestiscilo tree, and gestiscilo's planning has its own milestone /
phase numbering (milestone v6.6; latest phases 200–206). The migration touches gestiscilo
controllers, its `Cargo.toml`, its running server, and merges to gestiscilo master — all
gestiscilo concerns. Ferro 209 owns only: the ferro-projections intent-assertion tests, the
render-equivalence records, the weakness note, and the publish decision.

**To start it:** in a gestiscilo session, run `/gsd-discuss-phase` (or `/gsd-plan-phase`) for a
new gestiscilo phase using this brief as the source. Pick the gestiscilo phase number per its
own roadmap.

## Scope (locked)

- Migrate **exactly three** entities — one Browse, one Process, one Summarize — from
  `JsonUi::render_file(...)` to projection-driven rendering via `ServiceDef` + `JsonUiRenderer`.
- **Strictly sequential, one entity per merge** to gestiscilo master. Each entity is its own
  short-lived branch, merged before the next opens. No parallel migration branches. No branch
  alive > 2 weeks. (Ferro 209 SC#1, SC#3 / CONTEXT D-03.)
- **No ferro API change while a gestiscilo migration branch is open** (CONTEXT D-04). If a gap is
  hit, note-and-workaround gestiscilo-side; defer any ferro fix to a later v13.x slice (D-05).
- Migrate against the **published ferro 0.2.54** crates.io pin — default expectation is **zero
  ferro source changes** (D-06).

## Wave 0 — prerequisite (one line)

Enable the `projections` feature on the gestiscilo `ferro` dependency. `Cargo.toml` line 34:

```toml
# before
ferro = { version = "0.2.54", package = "ferro-rs", features = ["json-ui", "theme"] }
# after
ferro = { version = "0.2.54", package = "ferro-rs", features = ["json-ui", "theme", "projections"] }
```

Do **not** touch `ferro-json-ui = "0.2.54"` (line 35) — the `ferro::` re-export path transitively
enables `ferro-json-ui/projections`. Do **not** bump any version (activates already-published
0.2.54 code). Verify with `cargo check` from the gestiscilo root.

## Entities (resolved — RESEARCH §1)

| Intent | Entity | Controller | View JSON (delete after) | Model | Notes |
|--------|--------|-----------|--------------------------|-------|-------|
| Browse | Staff list | `src/controllers/staff/list.rs` (render_file ~line 110) | `src/views/staff/list.json` | `src/models/entities/staff.rs` | LOW bespoke risk. Requires `IntentHint::Primary(Intent::Browse)` (bio+avatar pull toward Focus). Recommended **first** if you want the cleanest Browse; Orders is the cleanest overall. |
| Process | Orders kanban | `src/controllers/cassa/orders.rs` | `src/views/cassa/orders_index.json` | `src/models/entities/orders.rs` | VERY LOW bespoke risk — full guarded state machine, strongest Process signal. Cleanest exemplar; good wiring-establishing first migration. |
| Summarize | Statistics dashboard | `src/controllers/statistiche.rs` | `src/views/statistiche/index.json` | `src/models/analytics.rs` | MEDIUM bespoke risk. Migrate the **stat-card section only**; chart/Tabs/trend-table stay as opaque `merge_data` passthroughs (RESEARCH §7 resolution). Surfaces the SC#5 gap (SVG chart has no FieldMeaning). |

Backups per intent exist in RESEARCH §1 if a primary hits a blocker.

## Wiring pattern (before → after, RESEARCH §2.3)

Per handler, replace the `JsonUi::render_file(...)` call with the projection path. Keep all
existing controller logic — `resolve_tenant()`, auth/sidebar/flash setup, the tenant-scoped query
(`find_all_for_tenant(business.id)` etc.), and per-row runtime data construction (e.g. Staff's
`signed_url(key, 3600)` avatars). Only the render mechanism changes.

```rust
use ferro::{
    derive_intents, DataType, FieldMeaning, IntentHint, Intent,
    JsonUiRenderer, ServiceDef, VisualContext,
};
use ferro_projections::render::Renderer; // brings .render() into scope

let service = ServiceDef::new("staff")
    .display_name("Staff")
    .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("name", DataType::String, FieldMeaning::EntityName)
    // ... per-entity field shape (mirror the ferro fixture in
    //     ferro-projections/tests/catalog.rs::real_world_slice_a for this entity) ...
    .intent_hint(IntentHint::Primary(Intent::Browse)); // Staff only

let intents = derive_intents(&service);
let spec = JsonUiRenderer
    .render(&service, &intents, &VisualContext::default())
    .map_err(/* the controller's existing error helper */)?;

// shallow-merge the same runtime data the before handler passed to render_file
let merged = spec.merge_data(json!({ /* _sidebar, _header, rows, flash, ... */ }));
let render_data = merged.data.clone();
JsonUi::render(&merged, &render_data) // <- replaces JsonUi::render_file
```

`merge_data` is already used in gestiscilo `dashboard.rs:595`. **DELETE** the old
`JsonUi::render_file(...)` call entirely (not commented out — ferro 209 SC#1; CLAUDE.md "delete
old code completely"). Do NOT replicate the hand-authored `*.json` spec inside the builder
(RESEARCH Pitfall 4) — the projection composes the layout from the intent template; differences
from the bespoke HTML are expected and documented as intentional visual deltas, not bugs.

## Auth / data-scoping preservation (required check per entity)

The migration must NOT widen data exposure or drop a guard. Confirm per handler that
`resolve_tenant` and the tenant-scoped query (`find_all_for_tenant` / equivalent) remain, and that
the ServiceDef declares only fields the before view already showed (the runtime `rows_json` is the
same one the before handler built — no new columns selected).

## Per-entity acceptance (gestiscilo-side)

- `grep -c "JsonUi::render_file" <controller>` returns 0 (deletion proof).
- `grep -c "JsonUiRenderer" <controller>` ≥ 1.
- `cargo build` from the gestiscilo root exits 0; commit gate
  `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings` green (serialize
  CPU — one at a time).
- Entity on its own branch, merged to gestiscilo master before the next entity's branch opens.
- Before/after captured (Chrome DevTools MCP, user runs the server) — handed to ferro 209 for the
  equivalence record.

## Likely abstraction gaps to capture (RESEARCH §5 — feeds ferro 209 SC#5)

1. **Statistics SVG chart** (HIGH) — `chart_svg` has no `FieldMeaning`; pass it as opaque
   `merge_data`, outside the ServiceDef. Record the gap.
2. **Staff signed avatar URLs** (MEDIUM) — `signed_url(key, 3600)` is computed per-row at render
   time; `FieldMeaning::ImageUrl` can't express the computation. Keep the loop controller-side;
   record the gap.
3. **Orders kanban column labels** (LOW-MED) — state→column-label mapping has no projection
   representation; keep `build_status_kanban_columns()` controller-side; record the gap.

Each gap goes into the corresponding `EQUIV-*.md` "Abstraction gaps surfaced" section in the ferro
209 phase directory, which the ferro weakness note (SC#5) synthesizes.

## Hand-back to ferro 209

After each entity merges, give ferro 209 Plan 02: the migration merge SHA, the before/after
screenshots (or the data to capture them), and the gaps hit. Ferro 209 fills the equivalence
records, the weakness note, and the publish decision. Ferro is **not** edited by this migration
(D-06, default zero ferro changes).
