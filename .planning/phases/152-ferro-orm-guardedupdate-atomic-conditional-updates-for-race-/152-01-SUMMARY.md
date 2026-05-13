---
phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-
plan: 01
subsystem: database
tags: [sea-orm, thiserror, leaf-crate, wave-1a, scaffolding]

# Dependency graph
requires: []
provides:
  - ferro-orm crate scaffold (Cargo.toml, src/lib.rs, src/error.rs, src/guarded.rs stub, README.md)
  - GuardedError enum with NoRowsAffected | TooManyRows | EmptyUpdate | Db variants
  - Forward-declared GuardedUpdate<E> type (body lands in plan 03)
  - Targeted SeaORM re-exports at crate root (no wildcard)
  - Compile boundary for plans 02-06
affects: [152-02, 152-03, 152-04, 152-05, 152-06, 153, 154]

# Tech tracking
tech-stack:
  added: [sea-orm 1.1.19, thiserror 2.0.17]
  patterns:
    - "Wave 1a leaf-crate Cargo.toml shape mirrored from ferro-wallet"
    - "Per-crate Error enum with name-prefixed Display ('guarded: …') matching workspace convention"
    - "Targeted SeaORM re-exports — no `pub use sea_orm::*` blanket"
    - "PhantomData<E> stub for forward-declared builder type (downstream plans can register the crate before the builder body exists)"

key-files:
  created:
    - ferro-orm/Cargo.toml
    - ferro-orm/src/lib.rs
    - ferro-orm/src/error.rs
    - ferro-orm/src/guarded.rs
    - ferro-orm/README.md
  modified:
    - Cargo.toml (workspace members append, Rule 3 deviation — see below)
    - Cargo.lock (cargo-regenerated)

key-decisions:
  - "runtime-tokio-native-tls (not runtime-tokio-rustls) for dev-dep sea-orm — avoids sqlx runtime feature collision with framework's existing native-tls (Pitfall 3 mitigation)"
  - "EmptyUpdate variant kept as load-bearing runtime guard (Pitfall 1): sea-orm's Updater::exec short-circuits with rows_affected: 0 on empty SET, which would otherwise masquerade as predicate miss"
  - "Expr added to crate-root re-exports beyond CONTEXT D-03's explicit list (Open Question 3 from RESEARCH): the canonical inventory-decrement example needs Expr::col(...).sub(needed), so omitting it would force every consumer to also depend on sea-orm directly"
  - "Workspace registration pulled forward from plan 02 (Rule 3 deviation) — plan 01's Task 3 acceptance criterion `cargo build -p ferro-orm exits 0` is unreachable without `[workspace.members]` containing 'ferro-orm' because the manifest uses `version.workspace = true`"

patterns-established:
  - "Wave 1a Cargo.toml: workspace-inherited version/edition/license + description/keywords/categories/homepage/repository/readme metadata, with no features on the runtime sea-orm dep (consumers provide driver/runtime at link time)"
  - "Crate-root rustdoc structure: title, killer-feature one-liner, rust,ignore Example block, atomicity-guarantee callout with explicit footgun reminder"
  - "Stub pattern for forward-declared types: `#![allow(dead_code)]` + PhantomData<E> field so clippy is silent until the real body replaces the file in a follow-up plan"

requirements-completed: []

# Metrics
duration: ~18min
completed: 2026-05-13
---

# Phase 152 Plan 01: ferro-orm crate scaffold Summary

**ferro-orm crate scaffolded as a Wave 1a leaf with GuardedError complete, GuardedUpdate forward-declared, and targeted SeaORM re-exports establishing the compile boundary for plans 02-06.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-13T15:17:39Z (worktree spawn)
- **Tasks:** 4
- **Files created:** 5 (ferro-orm/{Cargo.toml, src/lib.rs, src/error.rs, src/guarded.rs, README.md})
- **Files modified:** 2 (Cargo.toml workspace members append, Cargo.lock regenerated)

## Accomplishments

- New `ferro-orm` workspace member compiles clean against sea-orm 1.1.19 and thiserror 2.0.17.
- `GuardedError` enum complete with four variants, all `"guarded: …"`-prefixed for grep-friendly logs.
- Four `#[cfg(test)]` Display/From-impl assertions land at the same time as the type; `cargo test -p ferro-orm` green.
- `cargo build -p ferro-orm`, `cargo test -p ferro-orm`, `cargo clippy -p ferro-orm --all-targets -- -D warnings`, and `cargo fmt --all -- --check` all exit 0.
- Module-level rustdoc on `lib.rs` carries the canonical inventory-decrement example AND the atomicity-per-statement footgun callout (D-15).
- README.md is 11 lines, project-agnostic (no APP_NAME, no tenant identifiers, no marketing trigger phrases).

## Task Commits

1. **Task 1: ferro-orm/Cargo.toml** — `4901210d` (feat)
2. **Task 2: GuardedError enum + Display tests** — `f40d571d` (feat)
3. **Task 3: lib.rs + guarded.rs stub + workspace registration** — `b57bf24c` (feat)
   - **Task 3b: lib.rs comment rephrase** — `b62df887` (style; resolves a brittle plan-verify substring match against the forbidden-import comment)
4. **Task 4: README.md** — `232ffa4c` (docs)

## Files Created/Modified

- `ferro-orm/Cargo.toml` — Wave 1a leaf-crate manifest. Workspace-inherited version/edition/license, sea-orm 1.0 + thiserror 2 deps (no features on sea-orm), dev-deps with `runtime-tokio-native-tls` to match framework.
- `ferro-orm/src/lib.rs` — crate root with module rustdoc (canonical example + atomicity-guarantee section), targeted SeaORM re-exports (`Expr`, `IntoCondition`, `SimpleExpr`, `Value`, `ColumnTrait`, `ConnectionTrait`, `DbErr`, `EntityTrait`), `pub use error::GuardedError`, `pub use guarded::GuardedUpdate`.
- `ferro-orm/src/error.rs` — `GuardedError` enum (`NoRowsAffected | TooManyRows { affected: u64 } | EmptyUpdate | Db(#[from] sea_orm::DbErr)`) with four Display/From-impl assertions.
- `ferro-orm/src/guarded.rs` — minimal stub: `pub struct GuardedUpdate<E: EntityTrait> { _entity: PhantomData<E> }` with `#![allow(dead_code)]`. Plan 03 overwrites wholesale.
- `ferro-orm/README.md` — 11-line crate description, neutral voice, no tenant identifiers.
- `Cargo.toml` — workspace `[workspace.members]` array appended with `"ferro-orm"`.
- `Cargo.lock` — cargo-regenerated on first resolve.

## Decisions Made

- **Expr included in crate-root re-exports.** RESEARCH Open Question 3: CONTEXT D-03 lists `IntoCondition`, `SimpleExpr`, `Value` but the canonical inventory-decrement example needs `sea_orm::sea_query::Expr` for `Expr::col(Column::Quantity).sub(needed)`. Without re-exporting `Expr` at the crate root, every consumer would also have to depend on `sea-orm` directly, defeating the crate's cleanliness motivation. Added.
- **`runtime-tokio-native-tls` chosen for dev-dep sea-orm.** Pitfall 3: matches `framework`'s existing runtime to avoid sqlx feature-collision under `cargo test --all-features` workspace builds. RESEARCH explicitly recommends Option A (match framework).
- **`#![allow(dead_code)]` at the top of `guarded.rs` stub.** Without it, clippy `-D warnings` would fail on the unused `_entity: PhantomData<E>` field that exists only so `E` is not an unused-generic. Plan 03 removes the attribute when it lands the real body.
- **STATE.md "workspace version: 0.2.24" reading is stale.** `Cargo.toml` actually says `0.2.30` (RESEARCH Open Question 1). Phase 152's version-bump concern is irrelevant to plan 01 — left as-is for plan 06 / CI's `check-version` job to resolve.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Added `"ferro-orm"` to workspace `[workspace.members]` in plan 01 (plan 02 scope)**
- **Found during:** Task 3 (Create lib.rs + guarded.rs stub)
- **Issue:** Task 3 acceptance criterion is `cargo build -p ferro-orm exits 0`. Plan 02 owns workspace registration (its files_modified lists `Cargo.toml`). But `ferro-orm/Cargo.toml` uses `version.workspace = true`, which fails to resolve unless the package is a workspace member — so `cargo build -p ferro-orm` cannot pass in plan 01 without the registration. The plan author encoded a verification step that requires plan 02's edit to have already happened.
- **Fix:** Appended `"ferro-orm",` after `"ferro-wallet",` in the workspace members array. Idempotent with plan 02's identical edit (plan 02's Edit will be a no-op or already-applied detection).
- **Files modified:** `Cargo.toml`, `Cargo.lock` (cargo regenerated on resolve)
- **Verification:** `cargo build -p ferro-orm` finished in 13.43s with zero errors after the change.
- **Committed in:** `b57bf24c` (Task 3 commit)

**2. [Rule 1 — Bug] Rephrased forbidden-import comment in lib.rs**
- **Found during:** Task 3 verification (post-commit re-run of the static-grep verify chain)
- **Issue:** The lib.rs comment line `// Do NOT add \`pub use sea_orm::*;\` (D-03).` contained the literal substring `pub use sea_orm::*`. The plan's negative verify grep `! grep -q 'pub use sea_orm::\*' ferro-orm/src/lib.rs` matched the comment itself, flagging a false positive. The grep intent is to forbid the directive, not to prevent documenting it.
- **Fix:** Rephrased to "Do NOT add a wildcard re-export of `sea_orm` (D-03)." — same meaning, no literal substring trigger.
- **Files modified:** `ferro-orm/src/lib.rs`
- **Verification:** Static verify chain now passes; build/test/clippy/fmt unchanged.
- **Committed in:** `b62df887` (Task 3b style commit)

**3. [Rule 1 — Bug] Reformatted `TooManyRows` `#[error(...)]` attribute per rustfmt**
- **Found during:** Task 3 verification (`cargo fmt --all -- --check`)
- **Issue:** rustfmt requires line breaks inside the `#[error("...")]` macro call when the inner string literal pushes the line past max-width. The hand-written single-line form from the plan's `<action>` block did not match what rustfmt would emit, so `cargo fmt --all -- --check` failed.
- **Fix:** Ran `cargo fmt -p ferro-orm`; rustfmt broke the attribute over three lines. Behavior identical (the Display string literal is byte-equal). Tests still pass.
- **Files modified:** `ferro-orm/src/error.rs`
- **Verification:** `cargo fmt --all -- --check` exits 0; `cargo test -p ferro-orm` still passes 4/4.
- **Committed in:** `b57bf24c` (folded into Task 3 commit alongside the workspace registration)

---

**Total deviations:** 3 auto-fixed (1 Rule 3 blocking — workspace registration; 2 Rule 1 bugs — comment substring match, rustfmt reformat).
**Impact on plan:** All three are mechanical, non-architectural. The Rule 3 deviation pulls forward two lines from plan 02 (`"ferro-orm",` in the members array) to keep plan 01's verification self-contained — plan 02's existing edit becomes idempotent. No scope creep beyond what plan 01's own acceptance criteria demand.

## Issues Encountered

None — all four tasks executed cleanly. The three deviations above were auto-handled and did not produce visible problems during execution.

## User Setup Required

None — this is a scaffold-only plan with no external service configuration.

## Next Phase Readiness

- **Plan 02 (workspace registration, publish.yml, CLAUDE.md):** The workspace-members append is already in place from this plan's Rule 3 deviation. Plan 02's edits to `.github/workflows/publish.yml` and `CLAUDE.md` remain in-scope; the `Cargo.toml` step will detect the line already exists.
- **Plan 03 (GuardedUpdate body):** Will replace `ferro-orm/src/guarded.rs` wholesale. The stub's `#![allow(dead_code)]` and `PhantomData` artifacts must be removed when the real body lands.
- **Plans 04-06 (integration test, docs, release):** All can proceed against the public surface this plan establishes; no further compile-boundary work is needed.
- **No blockers, no concerns.**

## Self-Check: PASSED

- `ferro-orm/Cargo.toml` — FOUND
- `ferro-orm/src/lib.rs` — FOUND
- `ferro-orm/src/error.rs` — FOUND
- `ferro-orm/src/guarded.rs` — FOUND
- `ferro-orm/README.md` — FOUND
- Commit `4901210d` (Task 1) — FOUND
- Commit `f40d571d` (Task 2) — FOUND
- Commit `b57bf24c` (Task 3) — FOUND
- Commit `b62df887` (Task 3b style) — FOUND
- Commit `232ffa4c` (Task 4) — FOUND
- `cargo check -p ferro-orm` exits 0 — VERIFIED
- `cargo test -p ferro-orm` 4/4 passing — VERIFIED
- `cargo clippy -p ferro-orm --all-targets -- -D warnings` exits 0 — VERIFIED
- `cargo fmt --all -- --check` exits 0 — VERIFIED

---
*Phase: 152-ferro-orm-guardedupdate-atomic-conditional-updates-for-race-*
*Completed: 2026-05-13*
