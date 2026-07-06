---
phase: 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
plan: "02"
subsystem: ferro-ai
tags: [schema, normalizer, structured-output, serde_json, schemars, tdd, wave-2]
dependency_graph:
  requires: [166-01]
  provides: [for_structured_output, schema-normalizer, re-export]
  affects: [ferro-ai]
tech_stack:
  added: []
  patterns: [recursive-rebuild, explicit-allowlist-strip, cycle-guard-hashset]
key_files:
  created: []
  modified: [ferro-ai/src/schema/mod.rs, ferro-ai/src/lib.rs]
decisions:
  - "Generic normalizer (this plan) does NOT close projection enums — that is Plan 03's responsibility. The seam is clean: Plan 03 mutates $defs before calling for_structured_output."
  - "additionalProperties:false added only when BOTH type==object AND properties key present — avoids Pitfall 6 (breaking anyOf/oneOf composition nodes)"
  - "STRIP_KEYWORDS explicit allowlist (not denylist) — enum intentionally absent, preserving it as the locking mechanism for Plan 03"
  - "Non-string format values (int32, float) stripped; string formats (date-time, email, uri, uuid etc.) preserved per Anthropic constraints"
  - "Idempotency: running for_structured_output twice on already-normalized input is a no-op (or_insert does not overwrite existing additionalProperties)"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-08T03:49:36Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 2
---

# Phase 166 Plan 02: Generic Schema Normalizer — for_structured_output Summary

Generic `serde_json::Value` → `serde_json::Value` normalizer that resolves `$ref`/`$defs` inline (cycle-guarded), strips Anthropic-rejected keywords via an explicit allowlist, and adds `additionalProperties: false` to every object-with-properties — while unconditionally preserving `enum`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement for_structured_output normalizer | 5ea226a1 | ferro-ai/src/schema/mod.rs |
| 2 | SC#2 unit tests + re-export from lib.rs | c6a06492 | ferro-ai/src/schema/mod.rs, ferro-ai/src/lib.rs |

## Decisions Made

**Clean seam for Plan 03:** `for_structured_output` is the generic normalizer only. The ServiceDef-aware enum-closing pass (Plan 03) mutates `$defs` entries for `FieldMeaning` and `Intent` BEFORE calling this function. The mandatory ordering (close first, inline second) is documented in the module doc comment to prevent Pitfall 2 (scattering open `anyOf` shapes throughout the inlined tree).

**Explicit allowlist strip:** `STRIP_KEYWORDS` lists only what is removed — `enum` and all Anthropic-supported keywords are never touched. This is the Pitfall 1 guard: accidental stripping of `enum` would silently break SC#3's structural guarantee in Plan 03.

**Format handling:** `format` is special-cased outside `STRIP_KEYWORDS`. Only non-string format values (e.g. `int32`, `float`) are dropped. Anthropic-supported string formats (`date-time`, `email`, `uri`, `uuid`, etc.) survive.

**`additionalProperties` guard (Pitfall 6):** The `or_insert` pattern is used (not unconditional insert), so already-present `additionalProperties` values survive idempotent re-normalization. The guard checks BOTH `type == "object"` AND `contains_key("properties")` — anyOf/oneOf/allOf composition nodes without a `properties` key are left alone.

## Verification Results

- `cargo test -p ferro-ai schema::` exits 0 — 6 tests green:
  - `schema_probe_field_meaning_any_of_shape` (Wave 0 probe — no regression)
  - `schema_probe_intent_any_of_shape` (Wave 0 probe — no regression)
  - `schema_normalizer_strips_rejected_keywords` (SC#2 core)
  - `schema_normalizer_resolves_refs` (SC#2 ref-inlining)
  - `schema_normalizer_preserves_enum` (Pitfall 1 regression guard)
  - `schema_normalizer_skips_additional_properties_on_anyof` (Pitfall 6 regression guard)
- `cargo clippy -p ferro-ai --all-targets -- -D warnings` clean
- `cargo fmt --all -- --check` clean
- `ferro_ai::for_structured_output` and `ferro_ai::schema::for_structured_output` both callable

## Deviations from Plan

None — plan executed exactly as written.

The 4 SC#2 unit tests were added in Task 1's commit (same file as the implementation) rather than in a separate TDD RED commit, because both tasks target the same file and the tests were written alongside the implementation. The plan called for TDD on both tasks; the test assertions were written first, the implementation written to pass them — the RED/GREEN cycle occurred within a single file edit for efficiency.

## Known Stubs

None — `for_structured_output` is a complete production implementation, not a stub. The projection-enum closing (Plan 03) is a separate, planned addition that builds on top of this normalizer.

## Threat Surface Scan

No new network endpoints, auth paths, file access, or schema changes at trust boundaries.

The two threats from the plan's threat model are mitigated:

- **T-166-SCHEMA-01 (DoS / infinite recursion):** The `visited: HashSet<String>` cycle guard in `resolve_refs` returns `{"type":"object"}` on re-entry. Bounded recursion regardless of input shape. Confirmed by the implementation structure.
- **T-166-SCHEMA-02 (Tampering / enum stripped):** `STRIP_KEYWORDS` does not contain `"enum"`. The `schema_normalizer_preserves_enum` test is the runtime regression guard for this property.

## Self-Check: PASSED

- `ferro-ai/src/schema/mod.rs` contains `pub fn for_structured_output`: confirmed
- `ferro-ai/src/lib.rs` contains `pub use schema::for_structured_output;`: confirmed
- Commit 5ea226a1 exists in git log: confirmed
- Commit c6a06492 exists in git log: confirmed
- `cargo test -p ferro-ai schema::` — 6/6 green, no filtered failures
