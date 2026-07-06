---
phase: 164
plan: "08"
subsystem: ferro-json-ui
tags: [serde, deserializer, visibility, page-header, error-messages, D-19]
dependency_graph:
  requires: [164-05, 164-06]
  provides: [D-19-F5, D-19-F6]
  affects: [ferro-json-ui/src/visibility.rs, ferro-json-ui/src/component.rs]
tech_stack:
  added: []
  patterns:
    - hand-rolled serde Deserialize impl with key-presence dispatch (first in ferro-json-ui)
    - deserialize_with helper for lax field acceptance (first in ferro-json-ui)
key_files:
  created: []
  modified:
    - ferro-json-ui/src/visibility.rs
    - ferro-json-ui/src/component.rs
decisions:
  - "Removed Deserialize from Visibility derive list; kept Serialize and JsonSchema so the serialize direction and schema generation are unaffected"
  - "Hand-rolled Deserialize dispatches by key presence (contains_key) not by try-each-variant — per RESEARCH Pitfall 4 and PATTERNS D-19/F5"
  - "deserialize_actions_lax is a file-local fn (not pub) — laxness is intentionally scoped to PageHeader.actions only"
  - "Rust type of PageHeaderProps.actions stays Vec<String> — no API break per RESEARCH Pitfall 6"
metrics:
  duration_minutes: 15
  completed: "2026-05-17"
  tasks_completed: 3
  files_modified: 2
---

# Phase 164 Plan 08: Visibility Error Message + PageHeader.actions Lax Deserializer Summary

Two ferro-json-ui deserialization improvements addressing D-19 cross-repo coordination items (F5 and F6) surfaced from the gestiscilo v7 runtime walkthrough.

## What Was Built

### D-19/F5 — Hand-rolled Visibility::Deserialize with shape-listing error

`ferro-json-ui/src/visibility.rs` previously derived `Deserialize` on the `Visibility` enum via `#[serde(untagged)]`. When an unrecognized JSON shape was provided, the error was:

```
data did not match any variant of untagged enum Visibility
```

This message is useless for debugging — it names no accepted shapes and does not echo the offending input.

The fix removes `Deserialize` from the derive list (keeping `Serialize` and `JsonSchema`) and adds a hand-rolled `impl<'de> serde::Deserialize<'de> for Visibility` that:

1. Deserializes to `serde_json::Value` first.
2. Dispatches by key presence: `"and"` → `And`, `"or"` → `Or`, `"not"` → `Not`, `"path"` + `"operator"` → `Condition(VisibilityCondition)`.
3. On no match, emits: `invalid Visibility shape: <offending JSON>. Accepted shapes: {"and": [...]}, {"or": [...]}, {"not": {...}}, {"path": "/p", "operator": "...", "value": ...}`.

All four shapes continue to round-trip (serialize then deserialize equals original). Six new tests cover the four shape parsers, a four-variant round-trip regression test, and an error-message content assertion.

**This is the first hand-rolled `Deserialize` impl in ferro-json-ui.** The lax-deserializer pattern (Task 2 below) is the first `deserialize_with` usage. Both patterns are now established in the crate for future use.

### D-19/F6 — Lax deserializer for PageHeader.actions

`PageHeaderProps.actions` previously accepted only missing field and `[]` (via `#[serde(default)]`). The gestiscilo controller sometimes emits `""` (empty string) when no actions exist, which caused a deserialize failure.

The fix:
- Adds `deserialize_actions_lax` — a file-local helper fn using `deserialize_with`.
- Applies it via `#[serde(default, deserialize_with = "deserialize_actions_lax", skip_serializing_if = "Vec::is_empty")]` on `PageHeaderProps.actions`.
- Rust type remains `Vec<String>` — no API break for consumers constructing `PageHeaderProps` directly.
- Laxness is narrow: accepts `null`, `""`, `[]`, and `["a", "b", ...]`; rejects non-empty strings and arrays of non-strings.

Seven new tests cover all four allowed inputs and both rejection paths.

### Pre-commit gate

Full workspace `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` passes clean.

## Deviations from Plan

None — plan executed exactly as written.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | `47ef1030` | feat(164-08): hand-roll Visibility::Deserialize with shape-listing error (D-19/F5) |
| 2 | `034b12fc` | feat(164-08): lax deserializer for PageHeader.actions accepting null/empty-string/array (D-19/F6) |
| 3 (fmt) | `d9593736` | style(164-08): apply cargo fmt to visibility.rs |

## Known Stubs

None — both changes are fully wired.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced beyond those documented in the plan's threat model (T-164-08-01 through T-164-08-03).

## Note for Future Plans

The lax-deserializer pattern (`deserialize_with = "some_fn"` + `serde_json::Value` dispatch) is now established in ferro-json-ui. Future "accept a wider wire shape" requests should evaluate whether:
1. The same `deserialize_actions_lax` helper can be reused (unlikely — it is scoped to PageHeader.actions semantics).
2. A generalized `deserialize_lax_string_array` helper promoted to crate-level would serve multiple fields.
3. The type should change instead (only if wire format and Rust API both benefit — per RESEARCH Pitfall 6 this requires careful coordination).

## Self-Check: PASSED

- ferro-json-ui/src/visibility.rs: FOUND
- ferro-json-ui/src/component.rs: FOUND
- Commit 47ef1030: FOUND
- Commit 034b12fc: FOUND
- Commit d9593736: FOUND
