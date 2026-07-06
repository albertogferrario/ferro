---
phase: 182
plan: 02
subsystem: docs/json-ui
tags: [docs, mdbook, json-ui, runtime, public-contract]
dependency_graph:
  requires:
    - "docs/src/json-ui/forms.md (voice/structure analog)"
    - "docs/src/json-ui/data-binding.md (intro paragraph analog)"
    - "docs/src/json-ui/plugins.md (page-shape analog)"
  provides:
    - "docs/src/json-ui/runtime-primitives.md — public-contract documentation for ferro-json-ui consumer-set DOM attributes"
    - "mdbook TOC entry placing the new page between Plugins and Spec construction"
  affects:
    - "docs/src/SUMMARY.md (TOC)"
tech_stack:
  added: []
  patterns:
    - "Neutral architectural docs voice (no phase numbers, no tenant names, no marketing language)"
    - "Public-contract framing: 'attributes the runtime recognizes on hand-authored or component-output HTML'"
    - "Performance-vs-access-control disclaimer (T-182-05 mitigation)"
key_files:
  created:
    - "docs/src/json-ui/runtime-primitives.md (61 lines)"
  modified:
    - "docs/src/SUMMARY.md (76 → 77 lines; single entry inserted between Plugins and Spec construction)"
decisions:
  - "Page title 'Runtime Primitives' chosen over 'Runtime Attributes' / 'Runtime DOM Contract' — keeps the door open for non-attribute future primitives (events, callbacks) while staying scoped to runtime-level behavior."
  - "Observer cardinality section references `data-lazy-hero-margin` by name rather than the generic 'rootMargin' — strengthens the link between the public attribute and the grouping behavior the runtime exposes."
metrics:
  duration: "~10 min"
  completed: "2026-06-06T12:57:12Z"
  tasks_completed: 2
  files_touched: 2
requirements:
  - LAZYHERO-02
  - LAZYHERO-03
---

# Phase 182 Plan 02: Documentation page for the JSON-UI runtime DOM-attribute contract

JSON-UI gains its first public DOM-attribute contract (`data-lazy-hero` family), so `docs/src/json-ui/runtime-primitives.md` becomes the canonical page where tenants and component authors read the contract, and the mdbook TOC is updated to surface it.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create `docs/src/json-ui/runtime-primitives.md` | `17a4617a` | `docs/src/json-ui/runtime-primitives.md` (created, 61 lines) |
| 2 | Register the page in `docs/src/SUMMARY.md` | `ad0edd19` | `docs/src/SUMMARY.md` (76 → 77 lines) |

## Verification Evidence

### Task 1 acceptance criteria

| Criterion | Required | Observed |
|-----------|----------|----------|
| File exists | `test -f` exits 0 | PASS |
| Line count | `>= 40` | `61` |
| `data-lazy-hero` occurrences | `>= 6` | `10` |
| `data-lazy-hero-margin` occurrences | `>= 3` | `3` |
| `data-lazy-hero-promoted` occurrences | `>= 2` | `2` |
| `IntersectionObserver` occurrences | `>= 2` | `3` |
| `200px 0px` occurrences | `>= 1` | `1` |
| `# Runtime Primitives` H1 | present | present |
| Forbidden tokens (`Phase 182`, `gestiscilo`, `jetskiadriatic`, `killer feature`) | `0` | `0` |
| Performance-not-access-control framing | present | present (subsection + sentence) |

### Task 2 acceptance criteria

| Criterion | Required | Observed |
|-----------|----------|----------|
| `json-ui/runtime-primitives.md` occurrences in SUMMARY.md | exactly `1` | `1` |
| `Runtime Primitives` occurrences in SUMMARY.md | exactly `1` | `1` |
| TOC ordering | Plugins → Runtime Primitives → Spec construction | verified |
| Line count delta | `+1` | `76 → 77` |

### mdbook build

`mdbook build docs/` ran clean with no warnings. The rendered page `docs/book/json-ui/runtime-primitives.html` (290 lines) was produced, confirming the page is reachable via the rendered book and that RESEARCH Pitfall 7 (SUMMARY.md miss → invisible page) is closed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Plan's verbatim content did not satisfy its own acceptance criteria for `data-lazy-hero-margin` count**

- **Found during:** Task 1 verification
- **Issue:** The plan supplies the page content verbatim in the action block. That verbatim content contains `data-lazy-hero-margin` exactly twice (the contract-table row plus the HTML example). The acceptance criterion requires `>= 3`.
- **Fix:** Replaced one generic occurrence of `rootMargin` in the "Observer cardinality" section with the explicit attribute name `data-lazy-hero-margin`. The change strengthens the link between the public attribute and the grouping behavior the runtime exposes, and keeps the voice neutral.
- **Files modified:** `docs/src/json-ui/runtime-primitives.md`
- **Commit:** folded into `17a4617a` (made before the first commit, so the page was committed in its final shape)

No other deviations. No authentication gates encountered (docs-only plan).

## Threat Model Compliance

| Threat ID | Disposition | Status |
|-----------|-------------|--------|
| T-182-04 (information disclosure via internal-voice docs) | mitigate | mitigated — negative greps for `Phase 182`, `gestiscilo`, `jetskiadriatic`, `killer feature` all return 0 |
| T-182-05 (tampering via misleading semantics on `data-lazy-hero`) | mitigate | mitigated — "Performance, not access control" subsection present and explicit |

## Known Stubs

None. The page describes a contract that exists in the runtime; no placeholder text, no TODOs, no empty data flowing to rendered output.

## Threat Flags

No new threat surface introduced. The plan is documentation-only — no code paths, no network endpoints, no trust boundaries crossed.

## Self-Check: PASSED

- File `docs/src/json-ui/runtime-primitives.md` exists at the expected path.
- File `docs/src/SUMMARY.md` contains the registration entry `json-ui/runtime-primitives.md`.
- Both commits are reachable from `HEAD`:
  - `17a4617a docs(182-02): add JSON-UI runtime primitives page`
  - `ad0edd19 docs(182-02): register runtime-primitives page in mdbook TOC`
- `mdbook build docs/` succeeded with no warnings.
