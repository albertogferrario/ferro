---
phase: 182
plan: 01
subsystem: ferro-json-ui/runtime
tags: [json-ui, runtime, intersection-observer, lazy-loading, video]
requires: []
provides:
  - "FERRO_RUNTIME_JS now contains setupLazyHeroes primitive"
  - "Public DOM contract: data-lazy-hero, data-lazy-hero-margin, data-lazy-hero-promoted"
affects:
  - "Every page rendered via DefaultLayout or DashboardLayout"
tech-stack:
  added: []
  patterns:
    - "IntersectionObserver group-by-rootMargin bucketing"
    - "ES5-only JS sibling-runtime authoring style"
key-files:
  created:
    - ferro-json-ui/src/runtime/hero_lazy.rs
  modified:
    - ferro-json-ui/src/runtime/mod.rs
decisions:
  - "[Rule 1 - Bug] Plan-spec asserted double-quoted \"auto\" but JS uses single-quoted 'auto'; corrected the assertion literal to match the actual bundle content"
  - "[Note] Byte delta to FERRO_RUNTIME_JS is ~1455 bytes vs D-10 soft target ~500; RESEARCH §Example 5 line 585 pre-authorized this overshoot in exchange for sibling-consistency"
metrics:
  duration: ~7min
  tasks: 2
  files: 2
  completed: 2026-06-06
---

# Phase 182 Plan 01: ferro-json-ui setupLazyHeroes runtime primitive — Summary

ES5 setupLazyHeroes primitive added to ferro-json-ui runtime bundle; promotes below-the-fold `<video preload="none" data-lazy-hero>` elements to `preload="auto"` on viewport approach via IntersectionObserver, with per-element `data-lazy-hero-margin` overrides bucketed into one observer per distinct rootMargin (D-01), idempotency enforced through the `data-lazy-hero-promoted` marker (D-03), and Safari `<video>.load()` throws swallowed by a defensive try/catch (RESEARCH Pitfall 2).

## Tasks Executed

| Task | Name                                                                             | Commit    | Files                                           |
| ---- | -------------------------------------------------------------------------------- | --------- | ----------------------------------------------- |
| 1    | Create `hero_lazy.rs` with `setupLazyHeroes` `SOURCE` constant                   | 1d9f7866  | `ferro-json-ui/src/runtime/hero_lazy.rs` (NEW)  |
| 2    | Wire `hero_lazy` into `runtime/mod.rs` and extend three string-presence tests   | 2f96f794  | `ferro-json-ui/src/runtime/mod.rs`              |

## Implementation Notes

### `hero_lazy.rs` (Task 1)

- File-shape mirrored verbatim from `sidebar.rs`: `pub(super) const SOURCE: &str = r#"…"#;` with ASCII-box header comment, ES5-only JS body (4-space indented), no trailing newline before `"#;`.
- Rust source-comment block (5 lines) above the constant explains the inverse-polarity feature-detect guard (`typeof IntersectionObserver === 'undefined' return`) vs the positive guard in `plugins/map.rs §306` — sibling polarity matches every other early-return guard in the runtime tree.
- JS body:
  - Feature-detect early return.
  - Selector `video[preload="none"][data-lazy-hero]:not([data-lazy-hero-promoted])` (D-02 video-only + D-03 idempotency).
  - Selector-empty early return (no idle observers on pages without heroes).
  - Group by `(getAttribute('data-lazy-hero-margin') || '200px 0px').replace(/^\s+|\s+$/g, '')` — default 200px 0px + whitespace-trim guard against Safari SyntaxError (RESEARCH Pitfall 1).
  - One `new IntersectionObserver(..., { rootMargin: key })` per distinct margin bucket (D-01).
  - Promote action inside callback: `setAttribute('preload', 'auto')` + `setAttribute('data-lazy-hero-promoted', '1')` + `try { e.target.load(); } catch (_) {}` + `obs.unobserve(e.target)` (D-04 per-element unobserve, observer left alive).
  - Indexed `for` loops throughout — no `.forEach()`, no arrow functions, no template literals, no `let`/`const`.

### `runtime/mod.rs` (Task 2)

Four edits applied via the Edit tool (not Write):

1. `mod hero_lazy;` declaration inserted alphabetically between `mod form_guards;` and `mod kanban;` (line 11).
2. `s.push_str(hero_lazy::SOURCE);` appended after `scroll_preserve::SOURCE` in the IIFE assembly (line 42).
3. `\x20       setupLazyHeroes();\n\` inserted into the dispatcher block as the last invocation before `\x20   }\n\` (line 57).
4. Test module updated:
   - `bundle_contains_all_setup_functions` array gained `"setupLazyHeroes"`.
   - `dispatcher_invokes_every_setup` array gained `"setupLazyHeroes();"`.
   - New test `runtime_contains_lazy_hero_setup` appended after `dispatcher_invokes_every_setup`.

## Verification

Full per-task gate run on Task 2 (the wire-up commit, since Task 1 alone leaves the module unreferenced):

```
$ cargo fmt --all -- --check
  (no output — clean)

$ cargo clippy --all --all-targets -- -D warnings
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 14s
  (no warnings emitted)

$ cargo test -p ferro-json-ui --lib runtime::tests
  running 12 tests
  test runtime::tests::runtime_contains_lazy_hero_setup ... ok
  test runtime::tests::bundle_contains_all_setup_functions ... ok
  test runtime::tests::dispatcher_invokes_every_setup ... ok
  test runtime::tests::bundle_is_single_iife ... ok
  test runtime::tests::bundle_contains_dispatcher ... ok
  (… 7 pre-existing tests …)
  test result: ok. 12 passed; 0 failed; 0 ignored

$ cargo test --all-features
  86 test suites, TOTAL passed: 2823, failed: 0
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `runtime_contains_lazy_hero_setup` assertion literal mismatch**

- **Found during:** Task 2 verification (`cargo test -p ferro-json-ui --lib runtime::tests`).
- **Issue:** The plan's specified new test asserts `FERRO_RUNTIME_JS.contains("\"auto\"")` — a substring with double-quoted `"auto"`. But the JS source uses single-quoted `setAttribute('preload', 'auto')` per sibling-runtime convention (every sibling `runtime/*.rs` file uses single quotes inside the `r#"…"#` SOURCE strings). The bundle therefore contains `'auto'` but not `"auto"`, and the test failed with `assertion failed: FERRO_RUNTIME_JS.contains("\"auto\"")` after 11 of the other 11 tests passed.
- **Fix:** Changed the assertion to `assert!(FERRO_RUNTIME_JS.contains("'auto'"));` (Rust escape of single quote inside double-quoted Rust string literal). Added a one-line comment above the assertion explaining the JS-single-quote sibling convention. This matches the actual promote-action wiring (`setAttribute('preload', 'auto')`) and catches future regressions where the JS literal would be reverted to double quotes.
- **Files modified:** `ferro-json-ui/src/runtime/mod.rs` (line 206 area).
- **Commit:** `2f96f794` (squashed into Task 2 commit because the assertion was added in the same commit as the wire-up edits).

The plan-spec assertion was a copy-paste from RESEARCH §Example 4 which under-specified the quote convention; the JS code itself is correct and unchanged.

### Out-of-scope Discoveries

None. No pre-existing warnings, no unrelated test failures, no orphaned files.

### Auth Gates

None required for this plan.

## Byte Delta to `FERRO_RUNTIME_JS`

| Component                                | Bytes |
| ---------------------------------------- | ----- |
| `hero_lazy::SOURCE` JS body              | 1428  |
| Dispatcher line `        setupLazyHeroes();\n` | 27 |
| **Total delta**                          | **1455** |

D-10 frames a ~400-byte soft target and a ~700-byte redesign trigger. The 1455-byte delta overshoots both. The overshoot was pre-authorized in RESEARCH §Example 5 line 585: *"The planner has discretion to either (a) preserve indentation for sibling consistency and accept the soft-target overshoot, or (b) strip the inner-most indentation to hit the ≤500-byte target. Recommended: (a) — sibling-consistency outweighs the byte-budget guideline."* Stripping indentation would make `hero_lazy.rs` the only sibling in `runtime/` without 4-space JS body indentation, breaking the file-shape convention every other sibling (`sidebar.rs`, `dropdowns.rs`, `scroll_preserve.rs`, `kanban.rs`, …) follows. Recommended approach (a) was taken.

If the byte budget is later tightened to a hard fail, the cheapest minification path is to (i) drop the ASCII-box comment header (`// ── Lazy hero video promotion …`) — saves ~75 bytes, (ii) strip 4-space indentation throughout — saves ~400 bytes, leaving ~1000 bytes which still exceeds 700. The structural cost (group-by-margin bucketing, two nested loops, promote action) is the floor.

## Success Criteria Alignment

| ID            | Status      | Note                                                                                                                                                                                                                                                                                                                       |
| ------------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| LAZYHERO-01   | code-ready  | In-browser preload promotion path created in `hero_lazy.rs`; in-browser verification is consumer-side UAT (gestiscilo Phase 186 against jetskiadriatic landing page after 0.2.42 publishes). Plan 01 deliverable scope-complete.                                                                                            |
| LAZYHERO-02   | implemented | Per-element `data-lazy-hero-margin` override is honored at setup via the group-by-margin bucketing layer; one observer per distinct margin value. Verified by `runtime_contains_lazy_hero_setup` asserting the attribute name appears in the bundle.                                                                       |
| LAZYHERO-03   | implemented | Idempotency: selector `:not([data-lazy-hero-promoted])` filters at setup; `hasAttribute('data-lazy-hero-promoted')` re-checks in the callback (belt-and-suspenders); `setAttribute('data-lazy-hero-promoted', '1')` marks; `obs.unobserve(e.target)` frees per-element bookkeeping. Re-running the primitive is a no-op. |
| LAZYHERO-04   | guideline overrun | Byte delta 1455 vs soft target ~500 / hard trigger ~700. RESEARCH §Example 5 pre-authorized the overshoot in exchange for sibling-consistency. No automated assertion per D-10. Documented above.                                                                                                                  |

## Plan Notes for 182-02 / 182-03

- Plans 02 and 03 in this phase are `182-02-PLAN.md` (docs page `docs/src/json-ui/runtime-primitives.md` + `docs/src/SUMMARY.md` TOC entry per D-09) and `182-03-PLAN.md` (workspace version bump 0.2.41 → 0.2.42 + Cargo.lock sync + Wave 1A publish per D-08). Plan 01's runtime primitive is the only code-side change; docs and publish are independent.
- The `FERRO_RUNTIME_JS` bundle now ships `setupLazyHeroes` to every page rendered via `DefaultLayout` or `DashboardLayout` — `layout.rs::with_runtime` consumes the bundle unchanged. No layout-side or plugin-side changes required.
- The new public DOM contract surface (three attribute names: `data-lazy-hero`, `data-lazy-hero-margin`, `data-lazy-hero-promoted`) is the first ferro-json-ui runtime attribute set documented as a public contract for tenant HTML to use directly. Plan 02 (docs page) documents the contract.
- Plan 03 ships the primitive to crates.io as part of the 0.2.42 workspace bump. Per D-08 and `feedback_friction_loop_release_cadence.md`, no mid-loop publish — single end-of-phase publish.

## Self-Check: PASSED

Verified post-write:

- `test -f ferro-json-ui/src/runtime/hero_lazy.rs` → FOUND
- `test -f ferro-json-ui/src/runtime/mod.rs` → FOUND (modified)
- `git log --oneline | grep -q 1d9f7866` → FOUND (Task 1 commit)
- `git log --oneline | grep -q 2f96f794` → FOUND (Task 2 commit)
- `grep -c '^mod hero_lazy;' ferro-json-ui/src/runtime/mod.rs` → 1
- `grep -c 'fn runtime_contains_lazy_hero_setup' ferro-json-ui/src/runtime/mod.rs` → 1
- `cargo test -p ferro-json-ui --lib runtime::tests` → 12 passed, 0 failed
- `cargo test --all-features` → 2823 passed across 86 suites, 0 failed
- `cargo clippy --all --all-targets -- -D warnings` → no warnings
- `cargo fmt --all -- --check` → no diff
