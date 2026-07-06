---
phase: 180
plan: "05"
subsystem: docs
tags: [docs, action, mdbook, security]
dependency_graph:
  requires:
    - framework/src/http/action.rs (Plan 01 — ActionError, ActionResult, ActionOverrides)
    - ferro-macros/src/action.rs (Plan 03 — #[action] proc-macro)
    - framework/tests/action_handler.rs (Plan 04 — killer-feature contract locked)
  provides:
    - docs/src/the-basics/action-handlers.md (user guide for #[action])
  affects:
    - docs/src/SUMMARY.md (TOC entry)
    - docs/src/the-basics/controllers.md (cross-link)
tech_stack:
  added: []
  patterns:
    - mdbook documentation page
    - neutral architectural voice (no internal strategy framing)
key_files:
  created:
    - docs/src/the-basics/action-handlers.md (197 lines)
  modified:
    - docs/src/SUMMARY.md (1 line added — TOC entry after Controllers)
    - docs/src/the-basics/controllers.md (8 lines added — ## See also cross-link)
decisions:
  - "Cross-link added to controllers.md (not handlers.md): there is no docs/src/the-basics/handlers.md file in the repo. controllers.md is the closest analog page showing #[handler] usage and is already in the SUMMARY.md The Basics section."
  - "TOC entry placed after Controllers (not after a handlers.md entry): SUMMARY.md The Basics section lists Routing, Middleware, Controllers, Request & Response. Action Handlers inserted between Controllers and Request & Response — the natural position for a handler-layer page."
  - "User signature documented as req: Request (not &mut Request): per Plan 04 SUMMARY key-decisions, the macro classifies only the unwrapped Request shape; &mut is generated internally by generate_action_extraction. Corrected from Plan 05 prescribed prose which specified req: &mut Request."
  - "handle_action_result not referenced: per Plan 03 SUMMARY and Plan 05 critical constraints, this is pub #[doc(hidden)] — macro-generated code only. Not mentioned anywhere in the user docs."
  - "async move { ... }.await body wrapper explained functionally: Plan 04 SUMMARY documents this as a load-bearing fix. The docs explain the ? ergonomics consequence (? propagates to ActionResult) without exposing the internal wrapper mechanism."
metrics:
  duration: "~15 minutes"
  completed: "2026-05-30"
  tasks: 1
  files: 3
---

# Phase 180 Plan 05: `#[action]` Documentation Summary

One-liner: user guide for `#[action]` covering when-to-use, macro shape, `?` ergonomics via `async move` body wrapper, success-side overrides via `req.flash`/`req.redirect_to`, flash transport, back-compat query string, and all three security mitigations (T-180-01/02/03).

## Acceptance Grep Counts

| Check | Command | Result | Required |
|-------|---------|--------|----------|
| `#[action(` occurrences | `grep -c '#\[action(' action-handlers.md` | 5 | >= 3 |
| `req.flash\|req.redirect_to` | `grep -cE 'req\.flash\|req\.redirect_to' action-handlers.md` | 5 | >= 2 |
| `/accedi` (must be 0) | `grep -c '/accedi' action-handlers.md` | 0 | = 0 |
| Security section | `grep -cE '## Security\|### .*(T-180)' action-handlers.md` | 4 | >= 1 |
| T-180-01/02/03 present | `grep -c 'T-180-01\|T-180-02\|T-180-03' action-handlers.md` | 3 | >= 3 |
| TOC entry | `grep -c 'action-handlers.md' docs/src/SUMMARY.md` | 1 | = 1 |
| Cross-link | `grep -c 'action-handlers' controllers.md` | 1 | >= 1 |
| Internal voice triggers | `grep -cE 'killer feature\|bet on\|named weakness\|forcing function' action-handlers.md` | 0 | = 0 |

## mdbook Build

```
mdbook build docs
```

Exit code: 0. Output: `INFO HTML book written to .../ferro/docs/book`. No broken links, no missing files.

## Voice Review

Trigger phrases from CLAUDE.md "Repository documents must read as neutral" that were considered and avoided:

| Trigger phrase | Disposition |
|---------------|-------------|
| "killer feature" | Not used. `?` ergonomics described functionally: "the macro wraps the body in an `async move { ... }.await` block typed as `ActionResult`, so `?` propagates to `ActionResult`". |
| "bet on" / "betting on" | Not used. No strategic framing anywhere. |
| "named weakness" / "load-bearing weakness" | Not used. The `FormRequest` limitation is documented as a current constraint with a workaround, not as a "weakness". |
| "forcing function" | Not used. |
| "we accept that" / "the risk we're taking" | Not used. |
| Session provenance dates | Not used. |
| `/accedi` (consumer-specific path) | Not used. Replaced with `/your-login-path` as specified by D-08. |

The page reads as neutral architectural documentation: describes what the API does, how to use it, and what the security properties are — without any internal strategy notes, competitive framing, or personal voice.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Cross-link added to controllers.md instead of handlers.md**

- **Found during:** Task 1 — inspecting `docs/src/the-basics/` directory
- **Issue:** The plan prescribes adding a cross-link to `docs/src/the-basics/handlers.md`, but this file does not exist in the repository. The SUMMARY.md The Basics section lists: Routing, Middleware, Controllers, Request & Response — no handlers.md entry.
- **Fix:** Added the cross-link `## See also` section at the bottom of `controllers.md`, which is the existing page that shows `#[handler]` usage most extensively and is the natural landing page for users exploring handlers. The TOC entry was placed between Controllers and Request & Response.
- **Files modified:** `docs/src/the-basics/controllers.md`
- **Commit:** `0a9e127a`

**2. [Rule 1 - Bug] User signature corrected from `req: &mut Request` to `req: Request`**

- **Found during:** Reading Plan 04 SUMMARY key-decisions before authoring
- **Issue:** The plan's prescribed prose in the quick example uses `req: &mut Request`. Per Plan 04 SUMMARY: "Fixtures use `req: Request` (per macro doc comment), not `_req: &mut Request`... The macro's `classify_param_type` recognises only the unwrapped Request shape; the `&mut` is generated internally by `action.rs::generate_action_extraction`."
- **Fix:** All code examples in the docs use `req: Request` (the user-written form). The docs explain that the macro generates `&mut __ferro_req` internally.
- **Files modified:** `docs/src/the-basics/action-handlers.md`
- **Commit:** `0a9e127a`

## Known Stubs

None. The documentation page covers all required surface areas. The `FormRequest` limitation is documented in the `#[action]` source (ferro-macros/src/action.rs) with a `compile_error!` at the call site — not documented in this user guide because it is a current compiler-enforced constraint, not a user-visible configuration option.

## Threat Flags

None. This plan modifies documentation only.

## Commits

| Commit | Message |
|--------|---------|
| `0a9e127a` | docs(180-05): add #[action] user guide, TOC entry, and cross-link |

## Self-Check: PASSED

- `docs/src/the-basics/action-handlers.md` exists ✓
- `grep -c '#\[action(' action-handlers.md` = 5 (>= 3) ✓
- `grep -cE 'req\.flash\|req\.redirect_to' action-handlers.md` = 5 (>= 2) ✓
- `grep -c '/accedi' action-handlers.md` = 0 ✓
- `grep -c 'T-180-01\|T-180-02\|T-180-03' action-handlers.md` = 3 ✓
- `grep -c 'action-handlers.md' docs/src/SUMMARY.md` = 1 ✓
- `grep -c 'action-handlers' docs/src/the-basics/controllers.md` = 1 ✓
- `grep -cE 'killer feature|bet on|named weakness|forcing function' action-handlers.md` = 0 ✓
- `mdbook build docs` exits 0 ✓
- Commit `0a9e127a` exists ✓
