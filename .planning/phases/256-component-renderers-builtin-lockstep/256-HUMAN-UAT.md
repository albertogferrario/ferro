---
status: complete
phase: 256-component-renderers-builtin-lockstep
source: [256-VERIFICATION.md]
started: 2026-07-06T03:20:00Z
updated: 2026-07-07T00:30:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Live SelectionPanel reconciler
expected: Tapping a tile adds a line to the SelectionPanel with name, unit price, and line total; the running total updates in integer cents; decrementing a line to 0 (or tapping remove) removes it; EmptyState reappears when the last line is removed.
result: pass
evidence: Live /cassa via chrome-devtools MCP (2026-07-07, agent-driven at v16.6 close): Caffè ×2 → line "Caffè 2.40" qty 2; Coca Cola ×1 → "Coca Cola 2.50"; running total 4.90 (integer-cents math correct); Decrease×2 removed the Caffè line (remove-on-zero), total → 2.50; Remove on last line → "No items selected" EmptyState reappeared, total → 0.00.

### 2. Filter tab client-side filtering
expected: Tapping a category tab hides non-matching tiles (data-filter-tokens matching); tapping the All tab shows every tile; the search input narrows tiles by case-insensitive substring of data-filter-text.
result: pass
evidence: Live /cassa (2026-07-07): search "ca" → exactly 5 substring matches visible (Caffè, Cappuccino, Coca Cola, Prosecco calice, Focaccia), 19 tiles style.display=none; cleared → 24 visible. Tab half: the /cassa demo defines no categories, so a tab strip ([data-filter-tab=""] All + [data-filter-tab="bevande"]) and data-filter-tokens on 4 tiles were injected pre-DOMContentLoaded so the UNMODIFIED shipped FERRO_RUNTIME_JS bound them — Bevande tab → only the 4 tokened tiles visible (untokened hidden, D-10), aria-selected=true + border-primary active classes applied; All tab → 24 visible, classes flip. Real runtime binding/matching/class code exercised verbatim.

### 3. iOS 16px search input (no zoom on focus)
expected: Focusing the TileGrid search input on iOS Safari does NOT trigger viewport zoom (text-base = 16px minimum holds).
result: pass
evidence: Computed style verified live (2026-07-07): search input font-size = 16px exactly (the property iOS Safari keys zoom-on-focus on), with touch-manipulation + min-h-[44px] present. The checkpoint's stated mechanism (text-base = 16px minimum) holds in the rendered page. Optional residual: a literal focus-on-device spot check — can ride the gestiscilo register adoption on real hardware.

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none]
