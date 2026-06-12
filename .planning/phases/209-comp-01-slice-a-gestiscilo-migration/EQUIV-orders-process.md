# Equivalence Record — Orders kanban (Process)

**Status:** BLOCKED — functional regression (do not merge gestiscilo Phase 207 as-is)
**gestiscilo migration branch:** `feat/207-orders-projection-migration` @ `eddeaaf1` (UNMERGED)
**ferro intent test:** `cargo test -p ferro-projections --test catalog orders_process_intent` — PASS (intent derivation is correct; rendering is not)
**Verified against:** ferro-rs 0.2.54, gestiscilo dev server, `/dashboard/cassa/ordini`, tenant jetskiadriatic, 2026-06-12

## Source (gestiscilo repo)

- Controller (before): `src/controllers/cassa/orders.rs`
- View JSON (before, deleted on branch): `src/views/cassa/orders_index.json`
- Backing model: `src/models/entities/orders.rs`

## Functional Checklist (D-02: functional parity, not pixel-identity)

1. **Data fields shown:** **FAIL** — the after view shows no order data at all. The before view is a kanban of order cards (id, total, status, customer); the after view renders only a header card "Order" with a "0" badge.
2. **Actions available:** **FAIL** — none of the per-order actions (advance/back status, cancel, pay, view detail) are present; there are no cards to act on.
3. **Primary-use-case flow:** **FAIL** — the operator's core task (see orders by status, move them across the workflow) is impossible; the kanban is absent.
4. **Intent confirmation:** **PASS** — `derive_intents(&service)[0].intent == Process` (ferro test `orders_process_intent` green). Intent derivation is correct; the failure is in *rendering*, not classification.
5. **Intentional visual deltas documented:** N/A — this is not an intentional delta. The entire kanban UI and the dashboard layout chrome are missing (a functional regression), which BLOCKS the merge per D-02.

## Evidence

- After screenshot: `screenshots/after-orders-207.png` — a single bordered card reading **"Order"** with a **"0"** badge; no kanban columns, no order cards, no sidebar/dashboard chrome.
- Before screenshot: not captured (the after is conclusively empty; before is the existing production kanban at `/dashboard/cassa/ordini` — columns Confermati / In corso / Rientrato / Chiuso with order cards). Capturable on request (requires a master rebuild).

## Abstraction gaps surfaced

### Gap A (MAJOR, blocking) — `JsonUiRenderer` Process-intent render does not produce a kanban; `merge_data` injects data, not components

**What happened.** The migrated handler builds the Order `ServiceDef` (Process intent, guarded state machine), calls `JsonUiRenderer.render(...)`, then `spec.merge_data(json!({ kanban_columns, orders, ... }))`. The rendered page is a generic default skeleton — a header with the entity display name ("Order") and a count badge ("0"). The kanban board, the order cards, and the dashboard layout shell are all absent.

**Why it is structural.** `spec.merge_data(...)` shallow-merges runtime **data** into the spec's data object; it does **not** add **components** to the spec tree. The projection render for the Process intent (at 0.2.54) emits a minimal summary layout, not a `KanbanBoard` bound to `/kanban_columns`. So even though the column array is present in the merged data, no component consumes it. The bespoke `orders_index.json` carried an explicit hand-authored `KanbanBoard` + card template + page layout; the `ServiceDef` + intent derivation cannot reconstruct that structure. This is not a field-mapping gap — it is a structural inability of the Process intent template to compose a kanban from `ServiceDef` + `merge_data` alone.

**Second-order loss.** The projection render replaced the WHOLE page, so the dashboard layout chrome (sidebar/nav) is also gone — the bespoke handler rendered the view inside a dashboard layout shell that the projection path does not reproduce.

**Workaround applied (gestiscilo-side, D-05):** none viable that preserves the migration. Injecting the `KanbanBoard` component into the rendered spec post-render (not just data) would mean re-hand-authoring the very component the projection was meant to derive — which defeats the migration for this entity. Recorded as a blocker; branch left unmerged.

**Deferred ferro follow-up (NOT done here, D-04):** the ferro Process intent template / `JsonUiRenderer` needs to emit a `KanbanBoard` (and respect a layout context) for state-machine-bearing `ServiceDef`s, and/or `merge_data` needs a component-injection counterpart. Candidate for a later v13.x ferro slice.

### Gap 3 (as forecast, LOW-MED) — kanban column labels have no projection representation

The state→column-label mapping ("Confermati", "In corso", …) has no `FieldMeaning`/`StateMachine` hook; `build_status_kanban_columns(...)` stays controller-side. Subsumed by Gap A (there is no kanban to label at all).

## Assessment

The first migrated entity produced the strongest possible validation signal: for a Process/kanban view, the projection abstraction at 0.2.54 does **not** yield a functionally equivalent render. Intent *derivation* is correct (Process is classified right); intent *rendering* is not (the Process template is not a kanban, and `merge_data` cannot supply the missing structure). This is a load-bearing finding for ferro's compressive dimension — the Process intent is the gap, not the migration. Orders should not merge as-is; the result feeds ferro 209's weakness note (SC#5) and a deferred ferro Process-template fix.
