---
phase: 164-json-ui-improvements-batch-3
fixed_at: 2026-05-17T00:00:00Z
review_path: .planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/164-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 1
skipped: 3
status: partial
---

# Phase 164: Code Review Fix Report

**Fixed at:** 2026-05-17
**Source review:** `.planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/164-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-01, WR-02, WR-03, WR-04)
- Fixed: 1
- Skipped: 3

## Fixed Issues

### WR-01: Stale comment contradicts assertion in `builtin_types_count_matches_dispatch`

**Files modified:** `ferro-json-ui/src/render/mod.rs`
**Commit:** `9505183d`
**Applied fix:** Updated the comment at line 528 from "BUILTIN_TYPES must be 40 entries" to "BUILTIN_TYPES must be 41 entries", matching the `assert_eq!(BUILTIN_TYPES.len(), 41)` directly below it.

## Skipped Issues

### WR-02: `docs/src/json-ui/components.md` documents wrong enum variants for three types

**File:** `docs/src/json-ui/components.md:66-70`
**Reason:** already_fixed — commit `1d7c9339` (referenced in prompt) already applied this fix. Verified: `components.md` lines 66-70 already show `"default" | "narrow" | "wide"` for `form_max_width`, `"none" | "sm" | "md" (default) | "lg" | "xl"` for `gap_size`, and `"default" | "setup" | "danger"` for `action_card_variant`.
**Original issue:** Docs showed incorrect enum variants (`"sm"/"md"/"lg"/"xl"/"full"` for form_max_width, `"xs"` for gap_size, `"outline"/"ghost"` for action_card_variant) that do not match the Rust wire format.

### WR-03: `docs/src/json-ui/components.md` documents the `visible` field with wrong key names

**File:** `docs/src/json-ui/components.md:15`
**Reason:** already_fixed — commit `1d7c9339` (referenced in prompt) already applied this fix. Verified: `components.md` line 15 already shows `"visible": { "path": "/data/status", "operator": "eq", "value": "active" }` with the correct `"path"` and `"operator"` keys.
**Original issue:** Docs showed `"field"` and `"op"` keys which do not match the actual `Visibility` wire format.

### WR-04: Orphan element produced by `emit_statcard_root` is validated but unreachable — no consumer warning

**File:** `ferro-json-ui/src/projection/builder.rs:380-415`
**Reason:** skipped: fix would require non-trivial design change. Neither option is a safe one-liner. Option A ("data-ferro-orphan" attribute) requires adding a new field to the `Element` struct or embedding a sentinel prop — both touch the public data model and affect serde serialization. Option B (deferred `Catalog::validate_reachability` helper) is a new feature requiring additions to `ferro-mcp/src/tools/json_ui_validate_spec.rs` and the catalog module. The existing doc comment at lines 377-391 of `builder.rs` is already comprehensive and the `statcard_metadata_is_orphan_element` regression test pins the contract explicitly. Both mitigations already in place. Deferring to a follow-up phase where the StatCard-with-metadata wrapper (mentioned in the doc comment) is designed.
**Original issue:** The `metadata_list` DescriptionList element is placed in `spec.elements` but not reachable from the StatCard root's `children`. MCP `json_ui_validate_spec` silently passes this spec despite the orphaned element.

---

_Fixed: 2026-05-17_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
