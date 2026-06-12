# Equivalence Record — Statistics dashboard (Summarize)

**Status:** NOT MIGRATED — assessed from confirmed ferro source. Not worth a live migration until ferro binds StatCard values.
**gestiscilo migration branch:** none (no `feat/209` opened)
**ferro intent test:** `cargo test -p ferro-projections --test catalog stats_summarize_intent` — PASS

## Source (gestiscilo repo)

- Controller (before): `src/controllers/statistiche.rs`
- View JSON (before): `src/views/statistiche/index.json`
- Backing model: `src/models/analytics.rs` (`SummaryStats`: total_revenue_cents, order_count, average_order_cents)

## Functional Checklist (D-02)

1. **Data fields shown:** **PREDICTED FAIL** — `builder.rs::emit_statcard_root` builds `StatCardProps { value: String::new() }`; the stat values are empty and not data-bound. A Summarize migration would render the three stat-card labels with no numbers.
2. **Actions available:** N/A — a dashboard has no row actions; the period-switcher Tabs would not render (Tabs is not part of the Summarize template).
3. **Primary-use-case flow:** **PREDICTED FAIL** — "see the revenue/order numbers" is the entire use case, and the values are empty.
4. **Intent confirmation:** **PASS** — `derive_intents(&service)[0].intent == Summarize` (Money×2 + Quantity + read-only; ferro test green).
5. **Visual deltas:** the SVG revenue chart (forecast Gap 1 — `chart_svg` has no `FieldMeaning`) and the period Tabs have no projection representation; they would remain opaque `merge_data` with no component to render them.

## Evidence

Source assessment only (no live capture). Root cause: `ferro-json-ui/src/projection/builder.rs::emit_statcard_root` — `value: String::new()`.

## Abstraction gaps surfaced

### Gap C (MEDIUM) — Summarize StatCard value is not data-bound

`emit_statcard_root` emits a StatCard with an empty `value` and no `data_path`/binding. Summarize renders the metric *label* (the service display name) but not the *number*. Combined with the SVG-chart gap (no `FieldMeaning` for a server-rendered visualization) and the period-Tabs gap, the Statistics dashboard cannot be reproduced by the projection at 0.2.54.

**Deferred ferro follow-up:** StatCard value binding to runtime data; optionally a chart/visualization `FieldMeaning`.

## Assessment

Summarize was not migrated because the source analysis (confirmed during the Orders root-cause trace) shows the StatCard render is an empty-value placeholder — a live migration would only re-demonstrate Gap C at the cost of another build. The finding is folded into WEAKNESS-NOTE.md Gap C. Like Process, Summarize is layout-correct (StatCard selected) but content-incomplete (value unbound).
