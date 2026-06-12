# Equivalence Record — Orders kanban (Process)

**Status:** stub (filled in Plan 02 from the gestiscilo migration outputs)
**gestiscilo migration commit/PR:** <link to the gestiscilo migration merge commit>
**ferro intent test:** `cargo test -p ferro-projections --test catalog orders_process_intent`

## Source (gestiscilo repo)

- Controller (before): `src/controllers/cassa/orders.rs`
- View JSON (before, deleted after migration): `src/views/cassa/orders_index.json`
- Backing model: `src/models/entities/orders.rs`

## Functional Checklist (D-02: functional parity, not pixel-identity)

1. **Data fields shown:** <every data column in the before screenshot appears in the after screenshot; field names may differ, values must match> — [ ] PASS
2. **Actions available:** <every before action reachable after> — [ ] PASS
3. **Primary-use-case flow:** <most common operator action works in the migrated view> — [ ] PASS
4. **Intent confirmation:** `derive_intents(&service)[0].intent == Process` — [ ] PASS (ref: ferro test `orders_process_intent`)
5. **Intentional visual deltas documented:** <list every layout/markup difference; unlisted differences block the merge>

## Evidence

- Before screenshot: <path / chrome-devtools MCP capture ref>
- After screenshot: <path / chrome-devtools MCP capture ref>

## Abstraction gaps surfaced

<record any ServiceDef/renderer friction hit during this migration; feeds the phase-close weakness note (SC#5)>
