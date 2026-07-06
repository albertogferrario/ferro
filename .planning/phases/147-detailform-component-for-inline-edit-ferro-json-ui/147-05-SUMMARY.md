---
phase: 147
plan: 05
subsystem: ferro-mcp + docs
tags: [mcp, catalog, docs, discoverability, wave-1]
requires: [147-01]
provides:
  - "CatalogComponent entry for DetailForm in ferro-mcp exhaustive list (41st entry)"
  - "CatalogComponent entry for KeyValueEditor (phase 146 gap backfill)"
  - "### DetailForm section in docs/src/json-ui/components.md with Option-A rule"
affects:
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - docs/src/json-ui/components.md
tech_stack:
  added: []
  patterns:
    - "CatalogComponent description restates author-facing rules verbatim (UI-SPEC §14.8)"
    - "Docs section immediately after semantically-paired component (DescriptionList → DetailForm)"
key_files:
  created: []
  modified:
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - docs/src/json-ui/components.md
decisions:
  - "DetailForm catalog entry placed in the form-family cluster (immediately after Form), not appended at end — keeps related components grouped for MCP discovery"
  - "KeyValueEditor backfill placed next to DetailForm (both form-family extensions); closes phase 146 gap documented in 147-RESEARCH.md §Pitfall 6"
  - "Option-A rule duplicated in both catalog description (agent surface) and docs section (human surface) per UI-SPEC §14.7 + §14.8 — single source of truth is the plan, both derived copies match it verbatim"
metrics:
  duration: "~5 minutes"
  completed_date: "2026-04-23"
---

# Phase 147 Plan 05: ferro-mcp Catalog + Docs Reference Summary

One-liner: Two CatalogComponent entries (DetailForm + KeyValueEditor backfill) in
ferro-mcp and a full ### DetailForm section in docs/src/json-ui/components.md,
making the component discoverable by both MCP-driven agents and human callers
with the Option-A authoring rule stated verbatim in both surfaces.

## Objective Delivered

Plan 147-05 had four tasks. In this worktree (parallel executor), Tasks 1 and 2
ran normally. Task 3 (full CI gate `cargo test --all-features`) is **deferred to
the orchestrator post-merge**, because this worktree starts from the pre-Wave-1
HEAD and does not contain the impl from sibling plans 147-02 (component),
147-03 (render), or 147-04 (resolve).

## Tasks Completed

### Task 1 — CatalogComponent entries (commit `3c5268d2`)

Added two `CatalogComponent { ... }` entries to `build_catalog()` in
`ferro-mcp/src/tools/json_ui_catalog.rs`, placed in the form-family cluster
immediately after the `name: "Form"` entry (line 233):

| Entry | Line | Props |
|-------|------|-------|
| `DetailForm` | 253 | 9 (mode, action, fields, edit_url, cancel_url, edit_label, save_label, cancel_label, method) |
| `KeyValueEditor` | 314 | 6 (field, label, suggested_keys, allow_custom_keys, data_path, error) |

The DetailForm description is a single long string that includes the Option-A
authoring rule verbatim — agents reading the catalog via MCP discover the rule
without reading source (UI-SPEC §14.8).

KeyValueEditor is a backfill closing the phase 146 gap documented in
`147-RESEARCH.md §Pitfall 6 VERIFIED`.

Plan 01 Task 3 already bumped `assert_eq!(catalog.components.len(), 41, …)` and
added both names to the expected-names array; this task made those assertions
green.

Test evidence:
```
running 12 tests
test tools::json_ui_catalog::tests::test_all_components_present ... ok
[ ... 11 other json_ui_catalog tests ok ... ]
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

### Task 2 — ### DetailForm docs section (commit `05f60dca`)

Added the `### DetailForm` section to `docs/src/json-ui/components.md` at lines
473–593, positioned immediately after `### DescriptionList` (semantic twin per
147-PATTERNS.md §6).

Section contents (seven elements per UI-SPEC §14.7 and 147-PATTERNS.md §6):

1. One-paragraph "when to use" description contrasting with DescriptionList + Form
2. DetailFormProps table (nine rows)
3. DetailField fields table (three rows)
4. EditMode variants table (two rows)
5. Rust construction example using `ComponentNode::detail_form` + `DetailField::new` + `EditMode::from_query`
6. JSON round-trip example (`"type": "DetailForm"`, mode `"edit"`)
7. Option-A authoring rule stated in a callout paragraph, with aria-label accessibility note
8. "Not included in v1" callout (client-side toggle, per-field override, i18n, etc.)

### Task 3 — Full CI gate (DEFERRED to orchestrator post-merge)

Per the executor objective block, this worktree does not attempt
`cargo test --all-features` because:

- Worktree base is pre-Wave-1 HEAD (`5a2d3e2b`)
- Sibling plans 147-02 / 147-03 / 147-04 deliver the impl that the 147-01 RED
  tests exercise
- Running `cargo test --all-features` in this worktree alone would fail (the
  RED tests reference symbols that do not exist here)

**What DID run successfully in this worktree:**

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | exit 0, no diff |
| `cargo clippy -p ferro-mcp --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test -p ferro-mcp --lib json_ui_catalog` | 12/12 passing |

**What the orchestrator runs post-merge:**

```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets -- -D warnings
cargo test --all-features
```

After Wave-1 plans (02, 03, 04, 05) merge back to the base branch, the merged
state will contain: (a) DetailForm impl in component.rs + render.rs + resolve.rs
from plans 02–04, (b) the catalog entries from this plan's Task 1, (c) the docs
section from this plan's Task 2. At that point the full CI gate should pass
end-to-end, exercising the ~25 RED tests from Plan 01 as GREEN.

## Deviations from Plan

None for Tasks 1 and 2. Task 3 deviates from the plan's literal instruction
(run the full CI gate) in a structured way that was explicitly authorized in
the executor prompt's `<objective>` block — the parallel-worktree architecture
makes a single-worktree full-suite gate impossible.

## Acceptance Criteria — Status

### Task 1 (all pass)

- [x] `grep -q 'name: "DetailForm".to_string()'` → 1 match
- [x] `grep -q 'name: "KeyValueEditor".to_string()'` → 1 match
- [x] `grep -q 'Authoring rule (Option A)'` → 1 match (UI-SPEC §14.8)
- [x] `grep -q 'caller MUST set its label to'` → 1 match
- [x] DetailForm prop count = 9
- [x] KeyValueEditor prop count = 6
- [x] `cargo test -p ferro-mcp --lib json_ui_catalog` → 12/12 pass
- [x] `cargo clippy -p ferro-mcp --all-targets -- -D warnings` → exit 0
- [x] `cargo fmt --all -- --check` → exit 0

### Task 2 (all pass)

- [x] `grep -q '### DetailForm'` → 1 match (line 473)
- [x] `grep -q 'Authoring rule (Option A)'` → 1 match
- [x] `grep -q 'caller MUST set its .label. prop'` → 1 match
- [x] `grep -q 'DetailField'` → 6 matches
- [x] `grep -q 'EditMode'` → 4 matches
- [x] `grep -q 'EditMode::from_query'` → 2 matches
- [x] `grep -q '"type": "DetailForm"'` → 1 match (JSON example)
- [x] `grep -q 'aria-label'` → 1 match (accessibility guidance)
- [x] `awk '/### DescriptionList/,/### DetailForm/' | wc -l` → 59 (> 10)
- [x] Only `docs/src/json-ui/components.md` modified under docs/

### Task 3 (DEFERRED)

- [ ] `cargo fmt --all -- --check` — runs post-merge by orchestrator
- [ ] `cargo clippy --all --all-targets -- -D warnings` — runs post-merge
- [ ] `cargo test --all-features` — runs post-merge

In-worktree validation confirms the catalog + docs edits compile and test cleanly
in isolation; the cross-plan integration test runs at the orchestrator level.

## Known Stubs

None. Both tasks delivered complete content (no TODOs, no placeholder text, no
hardcoded empty arrays feeding UI).

## Threat Flags

None. Task 1 surfaces an authoring rule in the catalog description that is
explicitly public per T-147-06 (intended disclosure). Task 2 adds a docs page
consistent with the framework's "always update docs when framework changes"
convention (T-147-07).

## Commits

| Task | Commit  | Message |
|------|---------|---------|
| 1    | 3c5268d2 | feat(147-05): add DetailForm + KeyValueEditor entries to ferro-mcp catalog |
| 2    | 05f60dca | docs(147-05): add DetailForm section to json-ui components reference |
| SUM  | (pending) | docs(147-05): summary |

## Self-Check

- File `ferro-mcp/src/tools/json_ui_catalog.rs` — FOUND (modified, +104 lines)
- File `docs/src/json-ui/components.md` — FOUND (modified, +121 lines)
- Commit `3c5268d2` — FOUND (`feat(147-05): add DetailForm + KeyValueEditor entries to ferro-mcp catalog`)
- Commit `05f60dca` — FOUND (`docs(147-05): add DetailForm section to json-ui components reference`)
- CatalogComponent `DetailForm` entry — FOUND at line 253
- CatalogComponent `KeyValueEditor` entry — FOUND at line 314
- Docs `### DetailForm` header — FOUND at line 473
- exhaustive-list test `test_all_components_present` — PASSING (count 41 matches)

## Self-Check: PASSED
