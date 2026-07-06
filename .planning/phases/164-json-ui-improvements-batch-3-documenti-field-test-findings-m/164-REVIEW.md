---
phase: 164-json-ui-improvements-batch-3
reviewed: 2026-05-17T00:00:00Z
depth: standard
files_reviewed: 21
files_reviewed_list:
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/spec.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/visibility.rs
  - ferro-json-ui/src/projection/builder.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/containers.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-json-ui/tests/fixtures/reject/six_level_nesting.json
  - ferro-json-ui/tests/reject.rs
  - ferro-mcp/src/service.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - ferro-mcp/src/tools/json_ui_validate_spec.rs
  - ferro-mcp/src/tools/mod.rs
  - ferro-cli/tests/fixtures/migrate_v1/in_all_verbs.rs
  - ferro-cli/tests/json_ui_migrate_v1.rs
  - framework/src/json_ui/mod.rs
  - docs/src/json-ui/components.md
  - docs/src/json-ui/expressions.md
  - docs/src/json-ui/plugins.md
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 164: Code Review Report

**Reviewed:** 2026-05-17
**Depth:** standard
**Files Reviewed:** 21
**Status:** issues_found

## Summary

Phase 164 covers the json-ui v2 improvements batch: `CheckboxList`, `RichTextEditor`, `PageHeader.actions` lax deserializer, `$each`/`$if` directives, the v1 codemod, and MCP tooling. The structural code (serde, validation pipeline, data-path resolution, catalog drift guard) is sound. No security bugs or correctness-breaking logic errors were found.

Four warnings are raised:

1. A stale comment in the test for `BUILTIN_TYPES.len()` says "must be 40" but asserts 41.
2. The docs `components.md` documents three enum types with values that do not match the Rust wire format: `form_max_width`, `gap_size`, and `action_card_variant`.
3. The docs `components.md` shows a `visible` field using `"field"` and `"op"` keys that do not match the actual `Visibility` wire format (`"path"` and `"operator"`).
4. The `statcard_metadata_is_orphan_element` test pins an architectural anomaly (a reachability-invisible element) as intentional — the spec-level contract is documented but could silently confuse consumers who traverse `spec.elements` expecting everything to be reachable from the root.

Three info items are raised for minor quality issues.

---

## Warnings

### WR-01: Stale comment contradicts assertion in `builtin_types_count_matches_dispatch`

**File:** `ferro-json-ui/src/render/mod.rs:528-532`

**Issue:** The comment says "BUILTIN_TYPES must be 40 entries" but the `assert_eq` immediately below checks for 41. There are actually 41 entries in `BUILTIN_TYPES` (verifiable by counting the array). The discrepancy is in the comment only, but it will mislead anyone adding the next component — they may update the count to 42 in the assertion but leave the comment at 40, or update the comment to 41 and miss that it should become 42.

**Fix:** Update the comment to say 41, and consider making the number self-documenting by reading `BUILTIN_TYPES.len()` in the comment:

```rust
// Defense-in-depth check: BUILTIN_TYPES must be 41 entries.
// The dispatch match in `render_element` has one arm per entry plus a
// default arm. A compile-time mismatch would be caught by rustc; this
// runtime check pins the invariant for future edits.
assert_eq!(BUILTIN_TYPES.len(), 41);
```

---

### WR-02: `docs/src/json-ui/components.md` documents wrong enum variants for three types

**File:** `docs/src/json-ui/components.md:66-70`

**Issue:** The "Shared Enum Values" table in the component docs documents three types with values that do not match the `#[serde(rename_all = "snake_case")]` wire format in `component.rs`:

| Enum | Doc says | Rust wire format |
|------|----------|-----------------|
| `form_max_width` | `"sm" \| "md" \| "lg" \| "xl" \| "full"` | `"default" \| "narrow" \| "wide"` |
| `gap_size` | `"none" \| "xs" \| "sm" \| "md" \| "lg" \| "xl"` | `"none" \| "sm" \| "md" \| "lg" \| "xl"` (`"xs"` does not exist) |
| `action_card_variant` | `"default" \| "outline" \| "ghost"` | `"default" \| "setup" \| "danger"` |

These are not cosmetic mismatches — an agent reading the docs would generate specs with invalid prop values that fail catalog validation silently or at runtime.

**Fix:**

In `docs/src/json-ui/components.md` line ~66, replace:

```
**form_max_width** — `"sm"` | `"md"` | `"lg"` | `"xl"` | `"full"`

**gap_size** — `"none"` | `"xs"` | `"sm"` | `"md"` | `"lg"` | `"xl"`

**action_card_variant** — `"default"` | `"outline"` | `"ghost"`
```

with:

```
**form_max_width** — `"default"` | `"narrow"` | `"wide"`

**gap_size** — `"none"` | `"sm"` | `"md"` (default) | `"lg"` | `"xl"`

**action_card_variant** — `"default"` | `"setup"` | `"danger"`
```

---

### WR-03: `docs/src/json-ui/components.md` documents the `visible` field with wrong key names

**File:** `docs/src/json-ui/components.md:15`

**Issue:** The element shape example at the top of `components.md` shows:

```json
"visible": { "field": "/data/status", "op": "eq", "value": "active" }
```

The actual `Visibility` wire format (defined in `visibility.rs` and validated by serde) uses `"path"` not `"field"`, and `"operator"` not `"op"`. A spec written from the docs example will fail `Spec::from_json` with an "invalid Visibility shape" error because the required `"path"` and `"operator"` keys are absent.

**Fix:**

Change the element shape example at line 15 to:

```json
"visible": { "path": "/data/status", "operator": "eq", "value": "active" }
```

---

### WR-04: Orphan element produced by `emit_statcard_root` is validated but unreachable — no consumer warning

**File:** `ferro-json-ui/src/projection/builder.rs:380-415`

**Issue:** When the `Summarize` intent maps to a `StatCard` layout with a `"metadata"` slot, `emit_statcard_root` intentionally places a `DescriptionList` element into `spec.elements` that is NOT referenced from the root's `children`. The test `statcard_metadata_is_orphan_element` pins this as a known contract. The problem is not the implementation itself, but the absence of any signal in the returned `Spec` that an element is intentionally unreachable. Consumers who walk `spec.elements` (e.g. analytics, accessibility audits, future serializers) have no way to distinguish this intentional orphan from a bug.

The catalog validates the orphan as valid (since it only checks element-level schema, not reachability). The MCP `json_ui_validate_spec` tool would also silently pass this spec, giving agents a false "clean" signal about a spec that has an orphaned element.

**Fix (two options — choose one):**

Option A (minimal): Add a `data-ferro-orphan` data attribute to the emitted element so it is discoverable:

```rust
// In emit_statcard_root: mark the metadata element as an intentional orphan
// so downstream tooling can distinguish it from accidental dangling refs.
let id = "metadata_list".to_string();
// ... existing element construction ...
aux.push((id.clone(), element_with_props("DescriptionList", props)));
// intentionally NOT pushed to children_out — orphan by design
```

Document it in the emitter comment only (currently done). This is the status-quo; the warning is about raising awareness for the next phase.

Option B (preferred): Introduce a deferred `Catalog::validate_reachability(spec)` helper that accepts a list of known-orphan element IDs and emits a warning rather than an error. This would let `json_ui_validate_spec` surface orphans as `warnings` rather than missing them entirely.

---

## Info

### IN-01: `deserialize_actions_lax` allocates a full `serde_json::Value` for every PageHeader deserialization

**File:** `ferro-json-ui/src/component.rs:998-1018`

**Issue:** `deserialize_actions_lax` deserializes to `serde_json::Value` first, then pattern-matches. For the common case (an array of strings), this allocates an intermediate `Value::Array` and then clones each `String` out of it. A direct serde visitor implementation would avoid the intermediate allocation. This is an info-level note because the deserializer is only called on spec load (not per-request), and the number of actions is small.

**Fix (optional):** Implement a custom `serde::de::Visitor` that handles `Null`, `Str` (empty-string guard), and `Seq` without intermediate `Value` allocation. Only worth doing if profiling shows load-time cost.

---

### IN-02: `six_level_nesting.json` fixture has a comment gap — depth semantics may confuse future editors

**File:** `ferro-json-ui/tests/reject.rs:64-75` / `ferro-json-ui/tests/fixtures/reject/six_level_nesting.json`

**Issue:** The test comment says "Six levels (root + 5 children chain): one past MAX_NESTING_DEPTH=5." The fixture has 6 nodes total (root, A, B, C, D, E) forming a depth-6 chain. `MAX_NESTING_DEPTH` is 5, so depth 6 exceeds it. This is correct, but the comment "root + 5 children" could mislead — it is 5 edges, yielding depth 6 at the leaf. The assertion `found > 5` is correct but loose (it would pass even if `found == 6` or `found == 100`). A future change to the DFS that miscounts depth by off-by-one would still pass the test.

**Fix:** Tighten the assertion:

```rust
assert_eq!(found, 6, "six-level fixture must report found=6, not {found}");
```

---

### IN-03: `BUILTIN_SPECS` comment says "Order MUST match BUILTIN_TYPES" but the drift guard only checks length, not order

**File:** `ferro-json-ui/src/catalog.rs:121-123`

**Issue:** The comment on `BUILTIN_SPECS` states "Order MUST match `crate::render::BUILTIN_TYPES` exactly". The `Catalog::build` drift guard at line 537 checks only `BUILTIN_SPECS.len() != crate::render::BUILTIN_TYPES.len()`. The catalog test at line 1066 does check set equality of names (`assert_eq!(specs, types, ...)`), which verifies that every name appears in both lists. However, neither guard checks that the position of each name is identical across the two lists. Since the catalog uses a `HashMap` (not a slice-indexed lookup) and the render dispatch uses a `match` (not a slice index), order is not actually semantically required — the comment is misleading.

**Fix:** Either:
- Remove "Order MUST match" from the comment and replace with "Names MUST match": the set-equality test already enforces this.
- Or, if insertion-order matters for some future use, add an explicit positional check.

---

_Reviewed: 2026-05-17_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
