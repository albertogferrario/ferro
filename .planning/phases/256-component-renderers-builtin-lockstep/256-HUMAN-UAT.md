---
status: partial
phase: 256-component-renderers-builtin-lockstep
source: [256-VERIFICATION.md]
started: 2026-07-06T03:20:00Z
updated: 2026-07-06T03:20:00Z
---

## Current Test

[awaiting human testing — natural venue: Phase 257 when /cassa flips to the new components]

## Tests

### 1. Live SelectionPanel reconciler
expected: Tapping a tile adds a line to the SelectionPanel with name, unit price, and line total; the running total updates in integer cents; decrementing a line to 0 (or tapping remove) removes it; EmptyState reappears when the last line is removed.
result: [pending]

### 2. Filter tab client-side filtering
expected: Tapping a category tab hides non-matching tiles (data-filter-tokens matching); tapping the All tab shows every tile; the search input narrows tiles by case-insensitive substring of data-filter-text.
result: [pending]

### 3. iOS 16px search input (no zoom on focus)
expected: Focusing the TileGrid search input on iOS Safari does NOT trigger viewport zoom (text-base = 16px minimum holds).
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
