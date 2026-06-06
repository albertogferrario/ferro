---
phase: 183-ferro-bundle-capability-new-crate
plan: 04
subsystem: infra
tags: [publish-bootstrap, crates-io, ci-wiring, ferro-bundle, manual-action, dry-run]

# Dependency graph
requires:
  - phase: 183-ferro-bundle-capability-new-crate
    provides: "Plan 03 integration tests merged (Wave 3 complete); ferro-bundle v0.2.43 is lint-clean, test-green, and Cargo.toml metadata is publish-ready"
provides:
  - "Verified `cargo publish -p ferro-bundle --dry-run` exit 0 against the standalone-packaged source layout (resolved `ferro-rs = \"0.2\"` against crates.io)"
  - "Durable record of the manual first-publish bootstrap command + prerequisites + recovery procedure"
  - "Runbook section reusable by future new ferro-* crate phases"
  - "BUNDLE-01..06 traceability table mapping success criteria to delivering plan"
affects:
  - "Post Phase 183 master merge — when the user runs the documented manual `cargo publish -p ferro-bundle` command, the crates.io listing comes online and subsequent CI Wave 3 publishes are automatic"
  - "gestiscilo Phase 185 — bumps `ferro-bundle` in Cargo.toml after the publish lands"

# Tech tracking
tech-stack:
  added: []  # No new deps; this plan is gate + documentation only
  patterns:
    - "Manual first-publish bootstrap for any new ferro-* crate: required because the CI publish token is scoped `publish-update` only, not `publish-new` (see project memory `project_ferro_publish_token_scoping.md`)"
    - "Dry-run gates publish-readiness without network write — resolves declared deps against crates.io, verifies packaged source layout compiles standalone"

key-files:
  created:
    - .planning/phases/183-ferro-bundle-capability-new-crate/183-04-SUMMARY.md
  modified: []  # Documentation-only plan; no code edits

key-decisions:
  - "Real `cargo publish -p ferro-bundle` execution DEFERRED to user (per user request: do not publish to crates.io now; Phase 182 + Phase 183 commits are not pushed to master yet either)"
  - "Dry-run accepted as final automated gate for the phase; the publish itself is captured as a documented user action with prerequisites and recovery"
  - "Wave 3 shape (Shape B) retained from Plan 01: `WAVE3_CRATES=\"ferro-cli ferro-bundle\"` for-loop matching Wave 2 structure"

patterns-established:
  - "Plans with `checkpoint:human-action` for a crates.io publish are completed with dry-run + documentation; the SUMMARY records the deferred action so future audits read it instead of re-deriving the manual step"

requirements-completed: [BUNDLE-06]

# Metrics
duration: 13m0s
completed: 2026-06-06
---

# Phase 183 Plan 04: Publish Bootstrap Summary

**Workspace-wide lint + test gate green; `cargo publish -p ferro-bundle --dry-run` exit 0; real publish to crates.io deferred to user (per explicit instruction); manual bootstrap command + recovery procedure + new-crate runbook recorded for the post-merge action.**

## Performance

- **Duration:** 13m0s
- **Started:** 2026-06-06T18:27:45Z
- **Completed:** 2026-06-06T18:40:45Z
- **Tasks:** 3 (Task 1 automated gate; Task 2 documented as deferred-to-user; Task 3 this SUMMARY)
- **Files created:** 1 (this SUMMARY)
- **Files modified:** 0

## Task Outcomes

### Task 1 — Full gate + dry-run (automated, completed)

All four sub-gates exit 0:

| Gate | Command | Exit | Notes |
|------|---------|------|-------|
| Format | `cargo fmt --all -- --check` | 0 | The pre-existing ferro-json-ui fmt drift logged in `deferred-items.md` during Plan 01 has been resolved on master between Plan 01 and Plan 04 base — workspace-wide fmt is now clean. |
| Clippy | `cargo clippy --all --all-targets --all-features -- -D warnings` | 0 | Full workspace, no warnings. CI-matching command per memory `feedback_ci_clippy_command_match.md`. |
| Test | `cargo test --all-features` | 0 | 91 result blocks pass / 0 fail. ferro-bundle: 5 unit + 3 integration + 1 doc-test (ignored) = 8 tests. |
| Dry-run | `cargo publish -p ferro-bundle --dry-run` | 0 | Packaged 9 files, 128.1KiB (34.2KiB compressed). Standalone compile via crates.io dep resolution succeeded — see "Dry-run notes" below. |

**Dry-run notes:**
- The package was assembled at `target/package/ferro-bundle-0.2.43/` and compiled standalone (not as part of the workspace). This is the canonical signal that the crate is publishable.
- During standalone resolution, `ferro-rs = { path = "../framework", version = "0.2" }` resolved to `ferro-rs v0.2.41` from crates.io — the caret requirement `"0.2"` matched the latest published `0.2.x`. Phase 182's `ferro-rs 0.2.42` is not yet on crates.io and Phase 183's `0.2.43` will not be on crates.io until after the next workspace-wide publish; neither blocks the dry-run because the caret requirement is satisfied by `0.2.41`.
- Terminating output: `Uploading ferro-bundle v0.2.43 (...)` followed by `warning: aborting upload due to dry run`. This is the dry-run completion signal in the current cargo version (older plan text expected `Upload skipped, due to --dry-run.` — the substance is identical and the exit code is the source of truth).
- No file modifications occurred during Task 1.

### Task 2 — Manual first-publish bootstrap (DEFERRED to user)

This task is a `checkpoint:human-action`. Per the executor's explicit operating directive for this plan, **the real `cargo publish -p ferro-bundle` was NOT executed**. The user has stated:

> Phase 182 and Phase 183 commits are not yet on master / pushed, and the user does not want to publish ferro-bundle to crates.io at this time.

**Disposition:** `skipped` — explicit deferral, no failure. Phase 183's code is complete and shippable; only the crates.io publication step is intentionally outstanding.

**Action the user runs when ready (verbatim):**

```bash
cd /Users/alberto/repositories/albertogferrario/ferro
cargo publish -p ferro-bundle
```

**Prerequisites the user must verify before running the command:**

1. **Phase 182 (`ferro-rs 0.2.42`) is published to crates.io.** The CI Wave 2 publish step ships `ferro-rs`; if Phase 182's master merge has triggered that publish, this is done. Otherwise the user must first land Phase 182 on master and let CI publish `ferro-rs` (or publish it manually from local terminal if CI runs into a `publish-update`-token edge case).
2. **Phase 183 commits are on master.** The Plan 01/02/03 wave commits (`5ff11d1b`, `c613e27d`, `05e7bd98`, `ba2a7ee2`, `472daa77`, `e64a8a6e`, `1b18624f`, `45f2dedd`, `67a3810a`, `5ce014f9`) and this Plan 04 SUMMARY commit must be on master. The workspace version on master must equal `0.2.43`.
3. **The local `CARGO_REGISTRY_TOKEN` has `publish-new` scope.** If `cargo login` was run from this maintainer's machine with a token that includes the `publish-new` checkbox at https://crates.io/me, this is done. No new login is required if a prior session set it up.
4. **The current working directory is the workspace root.** `cargo publish -p $crate` from outside the workspace will resolve a different `Cargo.toml` or no manifest at all.

**Expected output after running the command:**

```
   Updating crates.io index
   Packaging ferro-bundle v0.2.43 (...)
   Packaged N files, X.X KiB (X.X KiB compressed)
   Verifying ferro-bundle v0.2.43 (...)
   Compiling ferro-bundle v0.2.43 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in Xs
   Uploading ferro-bundle v0.2.43 (...)
   Uploaded ferro-bundle v0.2.43 to registry `crates-io`
    note: waiting for ferro-bundle v0.2.43 to be available at registry `crates-io`.
    ...
   Published ferro-bundle v0.2.43 at registry `crates-io`
```

**Post-publish verification:**

```bash
curl -sI https://crates.io/api/v1/crates/ferro-bundle | grep -E '^HTTP/'
# Expected: HTTP/2 200 within ~60 seconds of the publish command returning
```

Or visit https://crates.io/crates/ferro-bundle in a browser.

**Failure modes and recovery:**

| Failure signature | Cause | Recovery |
|-------------------|-------|----------|
| `error: failed to publish ... 403 Forbidden` | Local `CARGO_REGISTRY_TOKEN` lacks `publish-new` scope. | Generate a new token at https://crates.io/me with the `publish-new` checkbox enabled, then `cargo login <token>` and retry the publish. |
| `error: failed to publish ... 422 Unprocessable Entity` with "crate name X is already taken" | The crate name was claimed externally between Phase 183 planning and the publish attempt. | Escalate. A rename discussion is required before any further publish attempt — Phase 183 cannot proceed without an architectural decision on the new crate name. |
| `error: no matching package named 'ferro-rs' found` | The `ferro-rs = "0.2"` caret requirement could not be satisfied against crates.io. | Verify Phase 182's `ferro-rs 0.2.42` (or any `ferro-rs 0.2.x`) is published. The dry-run already validated this against `ferro-rs 0.2.41` at the time of this plan, so this failure indicates ferro-rs was unpublished or yanked between dry-run and real publish. |
| `error: failed to publish ... package contains uncommitted changes` | Unstaged or untracked changes in the worktree. | Commit or stash the changes; rerun. The publish command requires a clean tree. |

**After the real publish succeeds:** the next workspace version bump that triggers CI Wave 3 will publish `ferro-bundle 0.2.44` (or whatever version is current) automatically. No further manual bootstraps are needed for this crate.

### Task 3 — This SUMMARY (completed)

`.planning/phases/183-ferro-bundle-capability-new-crate/183-04-SUMMARY.md` exists with the bootstrap command, traceability table, runbook, and deferral record.

## Runbook: New ferro-* crate first-publish bootstrap

Distilled from Phase 183 for reuse by future new-crate phases. Any future phase that adds a new top-level `ferro-*` workspace crate follows this sequence:

1. **Plan + scaffold the crate.** New `Cargo.toml` with `version.workspace = true`, `src/lib.rs`, README. Add to root `Cargo.toml` `workspace.members`. Bump `workspace.package.version` if dep contracts shifted.
2. **Wire CI publish wave.** Append the crate name to the correct wave variable in `.github/workflows/publish.yml` (Wave 1 leaf, Wave 2 framework, Wave 3 framework-consumers, etc.). Match the wave's existing shape (e.g. for-loop with `sleep 5` between crates).
3. **Land all scaffold plans on master.** Implementation, tests, README, workspace integration must all merge before publish.
4. **Run `cargo publish -p <new-crate> --dry-run` from local terminal at workspace root.** Exit 0 is the publish-readiness signal. Failures here mean the metadata, dep version requirements, or packaged source layout need fixing before the real publish.
5. **From local terminal at workspace root, run `cargo publish -p <new-crate>`** (no `--dry-run`). The local `CARGO_REGISTRY_TOKEN` must have `publish-new` scope — the CI token does not (see project memory `project_ferro_publish_token_scoping.md`).
6. **Verify the crates.io listing exists** within ~60 seconds of the command returning.
7. **Subsequent versions ship via CI Wave-N for that crate automatically.** No further manual bootstraps for this crate.

The single command that distinguishes "new crate" from "version bump on existing crate" is step 5. Every other step is standard workspace hygiene.

## Success Criteria Traceability — BUNDLE-01..06

| Success Criterion | Description | Plan | Verification command |
|-------------------|-------------|------|----------------------|
| BUNDLE-01 | Deterministic SHA-256 hash → 8-hex URL handle | 02 | `cargo test -p ferro-bundle --lib hash_is_deterministic` |
| BUNDLE-02 | 200 cold response with cache headers + 304 fast-path on `If-None-Match` | 03 | `cargo test -p ferro-bundle --test serve_cold --test serve_304` |
| BUNDLE-03 | Alias path 301 redirect to current hashed URL | 03 | `cargo test -p ferro-bundle --test alias_redirect` |
| BUNDLE-04 | Default `Content-Type: application/octet-stream` when none supplied | 02 | `cargo test -p ferro-bundle --lib default_content_type_is_octet_stream` |
| BUNDLE-05 | README documents bundle-vs-filesystem split with "do not fold these" load-bearing assertion | 01 | `grep -F 'do not fold these' ferro-bundle/README.md` |
| BUNDLE-06 | Publishes via existing GH Actions workflow (after first manual bootstrap) | 01 (CI wiring) + 04 (manual bootstrap action, deferred to user) | `grep -F 'ferro-bundle' .github/workflows/publish.yml` (CI-wiring side); crates.io listing returns HTTP 200 (bootstrap side — pending user execution of `cargo publish -p ferro-bundle`) |

BUNDLE-06 is split into two levels:
- **CI-wiring level (Plan 01):** ferro-bundle appears in `WAVE3_CRATES="ferro-cli ferro-bundle"` in `.github/workflows/publish.yml`. The workflow will attempt to publish ferro-bundle on every workspace-version-bumping master merge. Confirmed by grep.
- **First-publish bootstrap level (Plan 04):** The first attempt by CI to publish ferro-bundle will fail (CI token has `publish-update` scope only). The manual bootstrap is the user action documented above. After it succeeds, every subsequent CI run publishes the next version automatically. **This level is pending user action** — Phase 183's code is complete and shippable, only the crates.io publication is intentionally deferred.

## Publish-wave shape — Shape B retained

From Plan 01's decision (carried forward verbatim):

**Selected:** append `ferro-bundle` to existing Wave 3 alongside `ferro-cli`; rename step from `Publish Wave 3 (CLI)` to `Publish Wave 3 (framework-consumers)`; convert single-crate inline publish to a `WAVE3_CRATES="ferro-cli ferro-bundle"` for-loop matching Wave 2's structure verbatim (including the `sleep 5` between crates within a wave).

**Rationale:** smaller diff than inserting a Wave 2.5; matches the "post-framework consumers" semantic; both `ferro-cli` and `ferro-bundle` depend on `framework`/`ferro-rs` (which publishes in Wave 2) so they correctly belong in the same wave.

## Phase 183 Totals (across Plans 01–04)

| Metric | Value |
|--------|-------|
| Plans completed | 4 |
| Plan SUMMARYs | 4 (`183-01-SUMMARY.md`, `183-02-SUMMARY.md`, `183-03-SUMMARY.md`, `183-04-SUMMARY.md`) |
| New crate | `ferro-bundle/` (v0.2.43) |
| Workspace version bump | 0.2.42 → 0.2.43 (Plan 01 Task 2) |
| ferro-bundle tests | 5 unit + 3 integration + 1 doc-test (ignored) = 8 total |
| ferro-bundle source size | `src/lib.rs` ≈ 354 lines (Plan 02: 331 + Plan 03 shim: +23) |
| ferro-bundle public API surface | `Bundle::new`, `.content_type`, `.with_alias`, `.hashed_url`, `Bundle::serve`, `pub enum Error { NotFound, DuplicateName }`, `#[doc(hidden)] pub mod __test_internals` |
| Wave 3 publish entries | 2 (`ferro-cli`, `ferro-bundle`) |
| Plan commits | 11 (Plan 01: 3 + chore wave-close; Plan 02: 2 + chore; Plan 03: 2 + SUMMARY + chore wave-close; Plan 04: this SUMMARY commit) |
| Auto-fixed deviations (cumulative across plans) | 6 (Plan 01: 1 deferred-as-log; Plan 02: 2 lint/fmt; Plan 03: 3 lint/visibility/fmt; Plan 04: 0) |
| Architectural decisions reversed | 1 (Wave 1B → Wave 3, finalized in Plan 01 per RESEARCH §critical finding) |
| Plan 04 deviations | 0 — gate-only plan, no code edits |

## Decisions Made (Plan 04 only)

1. **Real `cargo publish` execution deferred to user.** Per the executor's explicit operating directive: the user does not want to publish to crates.io at this time (Phase 182 + 183 are not yet on master / pushed). The dry-run is the final automated gate; the real publish is a single documented user action with full prerequisites + recovery captured here.
2. **Dry-run accepted with `ferro-rs 0.2.41` as the resolved framework dep.** The caret requirement `version = "0.2"` matched the latest published `0.2.x` on crates.io; Phase 182's `0.2.42` is not yet published. This is correct behavior — the dry-run's job is to verify the published crate compiles against the crates.io graph as it exists at publish time, and it did.
3. **Workspace-wide fmt drift from Plan 01 confirmed resolved.** Plan 01's `deferred-items.md` recorded `ferro-json-ui/src/lib.rs:46-62` as fmt-dirty on master. Task 1's `cargo fmt --all -- --check` exit 0 indicates this was fixed between Plan 01's base and Plan 04's base (presumably by master-side activity). No action needed.

## Deviations from Plan

### Auto-fixed Issues

None. This plan is gate + documentation only; no code edits were performed.

### Deferred Actions

**1. Real `cargo publish -p ferro-bundle` execution — deferred to user**

- **Task affected:** Task 2 (the `checkpoint:human-action`).
- **Disposition:** `skipped` — explicit user-deferred. The user stated Phase 182 + 183 are not yet on master and the publish should not happen now.
- **What is documented for the user:** the exact command (`cargo publish -p ferro-bundle` from `/Users/alberto/repositories/albertogferrario/ferro`), the four prerequisites, the expected output, the verification curl, and four named failure-mode recoveries.
- **Reopen condition:** when the user has Phase 182 + 183 on master and has confirmed `ferro-rs 0.2.42` (or later) is on crates.io, the user runs the documented command.
- **Phase status implication:** Phase 183's code is complete and shippable. Only the crates.io publication step is intentionally outstanding. BUNDLE-06's bootstrap side is the single criterion pending — and only on the publication side, not the CI-wiring side.

---

**Total deviations:** 0 auto-fixed, 1 user-deferred (the publish action itself, which is the plan's only `checkpoint:human-action`).
**Impact on plan:** None to code or crate readiness. The phase is functionally complete; the durable record is here for the post-merge action.

## Pre-existing conditions surfaced by Task 1

None. Workspace fmt is clean, clippy is clean against all crates, all 91 test result blocks pass. The `deferred-items.md` ferro-json-ui fmt drift logged by Plan 01 has been resolved on master between Plan 01 and Plan 04 base — Task 1 verified `cargo fmt --all -- --check` exit 0.

## Consumer pairing — gestiscilo Phase 185

Per CONTEXT.md and the roadmap discovery note: gestiscilo Phase 185 is cross-tracked as `[FERRO REPO]` and will consume `ferro-bundle` once it lands on crates.io.

**Hand-off sequence for gestiscilo Phase 185:**

1. User runs the documented `cargo publish -p ferro-bundle` from local terminal (Task 2 action).
2. crates.io listing for `ferro-bundle 0.2.43` returns HTTP 200.
3. Gestiscilo Phase 185 bumps `ferro-bundle = "0.2"` (or pins `= "0.2.43"`) in its `Cargo.toml` and migrates the `/embed/v1.js` SDK bundle from the filesystem static-file handler to `Bundle::new("embed-v1", include_bytes!("...")).content_type("application/javascript").with_alias("/embed/v1.js")`.
4. Verifies `/bundles/embed-v1.<sha8>.js` serves 200 with `Cache-Control: public, max-age=31536000, immutable` and the plain `/embed/v1.js` 301-redirects to the hashed URL.

The consumer's adoption depends entirely on the Task 2 publication landing.

## User Setup Required

The user must run, when ready, exactly:

```bash
cd /Users/alberto/repositories/albertogferrario/ferro
cargo publish -p ferro-bundle
```

Prerequisites (repeated from Task 2 section for skimmability):

- Phase 182 (`ferro-rs 0.2.42`) on crates.io
- Phase 183 commits (including this SUMMARY) on master
- Workspace version on master = `0.2.43`
- Local `CARGO_REGISTRY_TOKEN` has `publish-new` scope (already set up if `cargo login` was used in a prior session)
- Working directory is the workspace root

After it returns: verify `https://crates.io/crates/ferro-bundle` shows `0.2.43`.

## Self-Check

- `.planning/phases/183-ferro-bundle-capability-new-crate/183-04-SUMMARY.md` exists — verified (this file).
- Contains the exact bootstrap command `cargo publish -p ferro-bundle` — verified (appears 5+ times in this SUMMARY).
- Contains a BUNDLE-01..06 traceability table — verified.
- Contains "Runbook: New ferro-* crate first-publish bootstrap" section — verified.
- Records Task 2 outcome as `skipped` (user-deferred) — verified.
- Documents Wave 3 shape (Shape B) and rationale — verified.
- References project memory `project_ferro_publish_token_scoping.md` — verified.
- Documents gestiscilo Phase 185 consumer pairing — verified.

## Self-Check: PASSED

## Next Phase Readiness

Phase 183 is **code-complete and shippable** — the only outstanding action is the deferred user-side `cargo publish -p ferro-bundle`. When the user runs it (after Phase 182 + 183 are on master and `ferro-rs 0.2.42`+ is on crates.io), Phase 183 is fully closed and `ferro-bundle 0.2.43` is the first crates.io listing.

No phase-internal blockers remain. The roadmap-side next action is whatever STATE.md points at (per task-specific notes, STATE.md is NOT to be updated by this executor — the orchestrator handles it).

---
*Phase: 183-ferro-bundle-capability-new-crate*
*Plan: 04-publish-bootstrap*
*Completed: 2026-06-06*
