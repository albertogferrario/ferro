# Phase 160: Remove v1 JSON-UI API — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-17
**Phase:** 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
**Mode:** `--auto` (single-pass; all gray areas auto-resolved with recommended option)
**Areas discussed:** Cleanup scope, Migration code-templates fate, application_info v1 scanner, Internal doc-comments style, Public-doc reframe scope, Cross-repo verification approach, Release cadence

---

## Cleanup scope (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Verification-only | Confirm v1 types are absent and tests pass; leave stale comments/docs/MCP plumbing alone | |
| Verification + source cleanup | Verify + delete stale internal references in source (doc-comments, MCP migration templates, MCP scanner) | |
| Verification + source cleanup + public-doc reframe | Verify + source cleanup + reframe public docs to drop v1 narrative per "JSON-UI is the only version" feedback | ✓ |

**User's choice (auto):** Option 3 — deepest scope.
**Notes:** Phase mandate is "permanently delete v1 surface". Per `feedback_json_ui_naming.md` (user memory), public docs and agent-readable surface must describe JSON-UI as the only version that exists. Stopping at source-only would leave stale narrative prose in `docs/protocol/` and `docs/src/features/projections.md`. Option 3 closes the loop in one phase.

---

## `migration_v1_to_v2_templates()` fate (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep | Retain — agents may still help consumers migrate from v1 | |
| Delete entire function | Drop the function, its registration, and its test — no v1 consumers remain | ✓ |
| Trim to v2-only patterns | Rename category, drop v1 framing, keep useful authoring snippets | |

**User's choice (auto):** Delete entire function.
**Notes:** Per Phase 164 V1-DELETION-AUDIT.md, gestiscilo (sole consumer) has fully migrated. No v1 consumers exist. Per `feedback_json_ui_naming.md`, no migration story belongs in agent-readable surface. Option 3 (trim) is half-measure — the templates are explicitly framed as migration, not as v2 authoring.

---

## `application_info::scan_json_ui_specs` v1 scanner (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep + rename | Drop "legacy" framing, leave behavior unchanged | |
| Replace with v2 scanner | Rewrite to count v2 JSON spec files (and optionally controller call sites) | ✓ |
| Delete entirely | No introspection of JSON-UI usage in `application_info` | |

**User's choice (auto):** Replace with v2 scanner.
**Notes:** Introspection value (telling an agent how many JSON-UI views a project ships) is real. Phase 120 TODO comment already flagged the v2 scanner work; Phase 160 collapses the TODO. Option 1 is cosmetic — the actual scan logic targets `src/views/*.rs` (v1 builder file shape), which never gets re-populated post-deletion.

---

## Internal port-comments in `render/{containers,form}.rs` (D-02, D-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `Port of v1 X` line, drop diff narrative | Preserve provenance one-liners, remove the "Differences from v1" paragraphs | |
| Delete all v1 references, describe what the function does today | Full neutralization | ✓ |

**User's choice (auto):** Delete all v1 references.
**Notes:** Per feedback: no v1 framing in agent-readable surface; provenance lives in `git log`/`git blame`. The "Port of v1 ..." line in a Rust doc-comment becomes load-bearing only when the comment is the only place that semantics is recorded — that is not the case here.

---

## Public-doc reframe scope (D-07, D-08)

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal substitution | Replace `ferro-json-ui/v1` → `ferro-json-ui/v2` in the four doc sites; leave narrative as-is | |
| Full reframe | Rewrite the surrounding prose to describe what JSON-UI is today (no v1 contrast, no version label) | ✓ |

**User's choice (auto):** Full reframe.
**Notes:** Minimal substitution leaves narrative like "JsonUiRenderer produces ferro-json-ui/v2 component trees, but ..." which still implies versioning history. Per `feedback_json_ui_naming.md`, public-doc framing must describe what IS, not what was. The wire literal `"ferro-json-ui/v2"` stays where it is the literal; prose around it gets rewritten.

---

## Cross-repo verification (D-09)

| Option | Description | Selected |
|--------|-------------|----------|
| ferro-only | Verify ferro builds + tests; defer gestiscilo/ferro-code to Phase 161 | |
| All three repos | ferro builds + tests + clippy; gestiscilo and ferro-code consume ferro via local-path and pass their suites | ✓ |

**User's choice (auto):** All three repos.
**Notes:** Phase 160 goal explicitly says "all three repos compile and pass tests after deletion". Phase 161 is the merge step, not the verification step. Verification must run on the deletion commit, not after merge — otherwise a missing v2 equivalent only surfaces post-merge and reopens this phase.

---

## Release cadence (D-11)

| Option | Description | Selected |
|--------|-------------|----------|
| Publish in Phase 160 | Publish a workspace version with v1 deleted | |
| Publish in Phase 161 only | Single end-of-loop publish at v12.0 merge | ✓ |

**User's choice (auto):** Publish at Phase 161 only.
**Notes:** Per memory `feedback_friction_loop_release_cadence.md`, mid-loop publishes freeze the API before later batches can revise it. Phase 161 is the natural publish point.

## Claude's Discretion

- Exact rewording of doc comments and prose — constraint is "neutral, present-tense, no v1 framing"; specific phrasing is a planning detail.
- Whether `application_info::scan_json_ui_specs` walks controller call sites in addition to file count.
- Test reorganization in `ferro-mcp/src/tools/json_ui_inspect.rs` if D-06 forces a rename — keep coverage equivalent.

## Deferred Ideas

- `LoadError::Catalog` variant cleanup (COMPLETED.md §4)
- Unified `$if` + `visible` directive (COMPLETED.md §4 — v12.1+)
- `Modal` chrome variant (COMPLETED.md §3)
- Granular `Card` props (COMPLETED.md §3)
- Codemod directory-recursive mode (COMPLETED.md §3)
- v12.0 CHANGELOG drafting (Phase 161)
- crates.io publish (Phase 161)
