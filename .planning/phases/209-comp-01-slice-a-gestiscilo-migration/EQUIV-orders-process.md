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

### Gap A (MAJOR, blocking) — ferro's Process projection emits a deliberate placeholder kanban, not a state-machine-aware, data-bound board (ferro-documented deferred limitation)

**Root cause (confirmed against ferro source — NOT a migration mistake).** The migrated handler called the API correctly. The rendered "Order / 0" is exactly the output of `ferro-json-ui/src/projection/builder.rs::emit_kanban_root`, whose own doc comment states:

> *"Process root. KanbanBoard emits a single root element with columns in props — no child elements ... Full state-machine awareness is a deferred idea (see CONTEXT.md); for now we emit a single placeholder column carrying the service's display name."*

That function hard-codes a single `KanbanColumnProps { title: resolve_title(service) /* "Order" */, count: 0 }` and `KanbanBoardProps { data_path: None }`. So the Process render at 0.2.54 is, by ferro's own design, a one-placeholder-column board that is **not state-machine-aware** (it does not derive columns from the `ServiceDef`'s state machine) and **not data-bound** (`data_path: None` — merged runtime data, including `kanban_columns`, has nothing to bind to). The intent *classification* is correct (Process ✓, ferro test green); the intent *rendering* is an intentional stub.

**This is the abstraction's real maturity level, empirically confirmed:** the projection builder emits intent-correct **layouts** but its **content binding is partial**, and several slots are explicitly deferred placeholders in ferro's own code:
- **Process → KanbanBoard:** placeholder column, not state-machine-aware, not data-bound (`emit_kanban_root`). ← this entity.
- **Summarize → StatCard:** `value: String::new()` — empty, not data-bound (`emit_statcard_root`). The Statistics migration (gestiscilo 209) would render labels with no values.
- **actions slot (all intents):** `emit_actions_placeholder` — *"Intentionally empty. Deferred to Phase 118+."* No row/card actions render for any migrated view.
- **Browse → DataTable:** the one comparative bright spot — `data_path: /data/{service.name}`, genuinely data-bound; rows render IF runtime data is merged at that exact path. (Staff/Browse is the most migratable of the three, modulo the data-path contract and the missing actions slot.)

**Second-order loss.** The projection render produces a standalone spec; the dashboard layout chrome (sidebar/nav) the bespoke handler wrapped the view in is not reproduced by the projection path.

**Workaround applied (gestiscilo-side, D-05):** none viable that preserves the migration. The missing structure (real columns, cards, actions) is exactly what the projection was meant to derive; re-hand-authoring it defeats the migration. Recorded as a blocker; `feat/207` left unmerged.

**Deferred ferro follow-up (NOT done here, D-04):** ferro's projection builder needs state-machine-aware Process column derivation + data binding (`emit_kanban_root`), Summarize stat-value binding (`emit_statcard_root`), and action wiring (`emit_actions_placeholder`, ferro's own "Phase 118+"). These are pre-existing ferro deferrals; the migration confirmed they block real-world Process/Summarize adoption. Candidate for a later v13.x ferro slice.

### Gap 3 (as forecast, LOW-MED) — subsumed

The state→column-label mapping was the predicted gap, but Gap A is deeper: there is no real kanban to label — ferro emits one placeholder column regardless.

## Assessment

The first migrated entity produced the strongest possible validation signal, and root-cause analysis confirmed it is **a real ferro limitation, not a migration error**. The projection abstraction at 0.2.54 derives intent correctly and emits the intent-appropriate **layout**, but its **content binding is incomplete by design**: Process (kanban) and Summarize (stat values) and actions are ferro-documented deferred placeholders; only Browse is meaningfully data-bound. The compressive abstraction is structurally present but not yet content-complete for real views. gestiscilo Slice A is therefore **blocked on ferro maturing the projection builder**, not on the migration technique. This is precisely the empirical signal COMP-01 was designed to surface.
