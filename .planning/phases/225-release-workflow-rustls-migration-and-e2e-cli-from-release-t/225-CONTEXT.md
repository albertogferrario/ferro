# Phase 225: Release Workflow rustls Migration and E2E CLI-from-Release Test - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults auto-selected; review decisions below)

<domain>
## Phase Boundary

This phase delivers two coupled CI/release-engineering outcomes:

1. **rustls migration** — Move the TLS-bearing transitive dependencies of `ferro-cli`
   (and, for coherence, the workspace `sea-orm` backend) from native-tls/OpenSSL to
   rustls, so release artifacts and `cargo install ferro-cli` build with **no system
   OpenSSL** — no `libssl-dev`, no `pkg-config`, no C-openssl cross-build. This is the
   root cause behind the COMP-04 cold-Debian friction (`cargo install ferro-cli` needed
   `libssl-dev`+`pkg-config`) and the fragility of cross-compiling
   `aarch64-unknown-linux-gnu`.

2. **E2E CLI-from-release test** — Add a CI test that exercises the **actual released
   `ferro` binary** scaffolding a real app and compiling it against the **published**
   `ferro-rs` library — catching the "published artifact is broken" class of failure
   (COMP-04 found the published 0.2.55 scaffold fails `cargo build` with 52 errors)
   before a user does.

**Killer feature:** the from-release e2e is the first gate that tests the *real shipped
artifact against the real published library*, not a `cargo run -p ferro-cli` against
path-deps. It closes the exact blind spot that let a non-compiling scaffold ship.

**NOT in scope (belongs to other phases):**
- Fixing the scaffold-template ↔ published-library API drift that produces the 52 errors.
  This phase adds the test that **detects** it; the **alignment** is a separate phase
  (named as follow-up in `project_comp04_published_scaffold_no_compile`).
- `ferro-wallet`'s direct `openssl = "0.10"` — not in `ferro-cli`'s dependency tree,
  does not affect the release binary.
- Changing runtime DB-TLS *behaviour* for consumers beyond the backend swap.

</domain>

<decisions>
## Implementation Decisions

### TLS Backend Migration
- **D-01:** Migrate `ferro-cli`'s entire transitive dependency tree to rustls.
  Concretely: `reqwest` → `default-features = false, features = ["blocking", "json", "rustls-tls"]`
  (currently `features = ["blocking", "json"]`, which pulls reqwest's default native-tls);
  `sea-orm` and `sea-orm-migration` → `runtime-tokio-rustls` (drop `runtime-tokio-native-tls`).
  Apply the **same sea-orm backend swap workspace-wide** — every crate currently on
  `runtime-tokio-native-tls` (framework, ferro-queue, ferro-mcp, ferro-orm, ferro-audit,
  ferro-migration, ferro-projection, ferro-deployments, ferro-reservation, ferro-mcp-oauth,
  ferro-mcp-server, app, …) moves to `runtime-tokio-rustls`. One TLS backend = one source
  of truth; split backends across the workspace would be a drift hazard and would still pull
  openssl-sys into `--all-features` CI builds. `lettre` in ferro-notifications →
  `tokio1-rustls-tls` for the same reason (not in CLI tree, but part of the coherence pass).
- **D-02:** rustls crypto provider = **`ring`**, not `aws-lc-rs`. aws-lc-rs reintroduces a
  C compiler + cmake build requirement, violating the "no external build tooling" rule in
  CLAUDE.md and re-breaking cross-compilation — the exact barrier this migration removes.
  Researcher MUST confirm the resolved reqwest/sea-orm/sqlx rustls feature wiring selects a
  ring-backed provider (not aws-lc-rs by default) and pin it explicitly if the default drifts.
- **D-03:** `ferro-wallet`'s direct `openssl = "0.10"` is **out of scope** (not a release-binary
  or `cargo install ferro-cli` dependency). Recorded as a follow-up coherence item, not a blocker.

### Release Workflow
- **D-04:** Once OpenSSL is gone, **drop the `cross`/Docker path** for
  `aarch64-unknown-linux-gnu`. Build it natively with `rustup target add` + the
  `gcc-aarch64-linux-gnu` cross-linker (linker set via `.cargo/config.toml` or a
  `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER` env). Pure-Rust TLS removes the only
  C-cross barrier; dropping `cross` removes the `cargo install cross` + Docker steps and
  speeds the release. **Fallback (Claude's discretion):** if the researcher finds a residual
  C dependency in the CLI tree that still needs cross-linking infrastructure, keep `cross`
  for that one target only — do not let it block the rustls swap.
- **D-05:** `cargo-deny` (the `deny` CI job) must stay green after the swap. Removing
  native-tls/openssl-sys should *reduce* the advisory/license surface; verify it does not add.

### E2E CLI-from-Release Test
- **D-06:** The test runs the **actual released `ferro` binary** (the artifact built earlier
  in `release.yml`), not `cargo run -p ferro-cli`. It scaffolds an app and runs `cargo build`
  against the **published** `ferro-rs` library (the generated `Cargo.toml` pins
  `ferro = { package = "ferro-rs", version = "0.2" }` from crates.io).
- **D-07:** Trigger/placement: a job in `release.yml` gated `needs: build`, consuming the
  just-built linux artifact; **plus** `workflow_dispatch` and a scheduled cron so
  published-library drift is caught **between** releases, not only at tag time (the generated
  app pins crates.io ferro, so drift is continuous). Cron cadence = Claude's discretion.
- **D-08:** Test surface = mirror the COMP-04 benchmark sequence:
  `ferro new` → `make:auth` → `make:scaffold` ×N → `make:job` → `cargo build` — precisely the
  surface that produced the 52 errors. **Reuse/adapt the existing apparatus** at
  `ferro-cli/tests/benchmark_new_project.rs` + `tests/fixtures/benchmark/` rather than authoring
  a new harness. Run it **non-Docker** in CI (the runner is already a clean Linux box) for speed;
  the existing Dockerfile remains the cold-cache reproduction artifact.
- **D-09:** **Complement, do not replace** the existing fast `scaffold-smoke` job
  (`scaffold_builds_against_workspace_ferro`, runs every PR against the path-dep workspace ferro).
  Two layers: workspace-smoke catches drift pre-merge; from-release e2e catches published-artifact
  reality post-tag.

### Sequencing Risk (planner must resolve)
- **D-10:** A from-release e2e that builds against the **published** library will go RED if the
  currently-published `ferro-rs` scaffold still carries the COMP-04 drift. The rustls half (D-01..05)
  is **independent** and can land regardless. For the e2e half, the planner must pick one:
  (a) verify the current published scaffold compiles first (the template-alignment follow-up phase
  runs ahead of this), or (b) land the e2e in a non-blocking/tracking mode (`continue-on-error`)
  until alignment lands. Decide explicitly in PLAN.md — do not ship a permanently-red required job.

### Claude's Discretion
- Mechanism for the aarch64 cross-linker (`.cargo/config.toml` vs workflow env).
- Whether to add `x86_64-unknown-linux-musl` as a fully-static target now that rustls makes
  musl trivial — optional, only if cheap; do not expand scope.
- Cron cadence for the scheduled from-release run.
- `--release` vs debug build of the generated app in the e2e (debug is faster for a compile-only smoke).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Release / CI workflows being changed
- `.github/workflows/release.yml` — the workflow this phase migrates (cross-compile matrix,
  `cross` usage for aarch64-linux, artifact upload, install-script update job).
- `.github/workflows/ci.yml` — contains the existing `scaffold-smoke` job
  (`cargo test -p ferro-cli scaffold_builds_against_workspace_ferro`) and the `deny` job;
  the from-release e2e complements `scaffold-smoke`.
- `.github/workflows/publish.yml` — crates.io publish waves; relevant because the published
  library that the e2e builds against is produced here (timing/drift surface).

### TLS dependency surface
- `ferro-cli/Cargo.toml` — `reqwest` (default native-tls), `sea-orm` + `sea-orm-migration`
  (`runtime-tokio-native-tls`). Primary migration target.
- `ferro-cli/src/commands/api_check.rs` — the only ferro-cli code path using `reqwest` (network).
- Workspace `Cargo.toml` files using `runtime-tokio-native-tls` (coherence sweep, D-01):
  framework, ferro-queue, ferro-mcp, ferro-orm, ferro-audit, ferro-migration, ferro-projection,
  ferro-deployments, ferro-reservation, ferro-mcp-oauth, ferro-mcp-server, app.
- `ferro-notifications/Cargo.toml` — `lettre` `tokio1-native-tls` → `tokio1-rustls-tls`.
- `ferro-wallet/Cargo.toml` — direct `openssl = "0.10"` (OUT of scope, D-03; noted only).

### E2E test apparatus to reuse
- `ferro-cli/tests/benchmark_new_project.rs` — existing `FERRO_BENCH=1` `#[ignore]` benchmark
  AND the `scaffold_builds_against_workspace_ferro` smoke test; the from-release e2e adapts this.
- `ferro-cli/tests/fixtures/benchmark/Dockerfile` — cold-cache crates.io install + scaffold + build
  reproduction (the COMP-04 apparatus).
- `ferro-cli/tests/fixtures/benchmark/RESULTS.md` — recorded COMP-04 findings.
- `.planning/phases/211-comp-04-time-to-working-app-benchmark/211-WEAKNESSES.md` — the 52-error
  root-cause list the from-release e2e is meant to catch.

### Install path (touched by release.yml)
- `scripts/install.sh`, `scripts/create-app.sh` — release.yml rewrites the repo name in these.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-cli/tests/benchmark_new_project.rs` already drives the full scaffold sequence
  (`new`→`make:auth`→`make:scaffold`→`make:job`→`cargo build`) — adapt for the from-release e2e
  instead of writing new harness code.
- `tests/fixtures/benchmark/Dockerfile` already proves the cold-cache crates.io path; keep as the
  Docker reproduction, derive the CI job from its steps.
- `ci.yml` already has a `scaffold-smoke` job pattern to mirror for job structure.

### Established Patterns
- CI pins toolchain `1.88.0` (MSRV); `RUSTFLAGS: -Dwarnings`; `Swatinem/rust-cache@v2`.
- Workspace `Cargo.toml` `[profile.dev]`/`[profile.test]` already tuned for runner disk
  (`debug=false`, `incremental=false`, `strip=true`) — the from-release e2e's `cargo build`
  of the generated app should inherit equivalent frugality (see `project_ferro_ci_disk_and_push`).
- The recent `ci(release): rustup target add for non-cross targets` commit (3fee6120) already
  added the `rustup target add` step — D-04 extends this to also cover aarch64 once `cross` is dropped.

### Integration Points
- `release.yml` build matrix (where `cross` is removed and the rustls-built binary is produced).
- A new `release.yml` job (`needs: build`) + `workflow_dispatch`/`schedule` triggers for the e2e.
- Every `*/Cargo.toml` carrying the sea-orm/lettre TLS feature (coherence sweep).

### Constraints from prior learnings
- `aws-lc-rs` / native build tooling is forbidden by CLAUDE.md "No external build tooling" — drives D-02.
- gh token here lacks `workflow` scope (`project_ferro_ci_disk_and_push`); pushing `.github/workflows/*`
  changes needs `gh auth refresh -s workflow` or operator SSH/PAT. Flag to the operator at push time.
- `cargo test --all-features` is disk-fragile on the runner; do not balloon the test build.

</code_context>

<specifics>
## Specific Ideas

- The migration's payoff is concrete and measurable: after it, a clean `debian:bookworm-slim`
  should `cargo install ferro-cli` with **no** `libssl-dev`/`pkg-config` apt step — verify against
  the COMP-04 Dockerfile (which currently installs them).
- The from-release e2e is the structural guarantee that COMP-04's "ships silently broken" can never
  recur — frame it as a required gate (subject to the D-10 sequencing decision).

</specifics>

<deferred>
## Deferred Ideas

- **Scaffold-template ↔ published-library API alignment** (the actual 52-error fix) — separate
  phase, named as the follow-up in `project_comp04_published_scaffold_no_compile`. This phase only
  adds the detecting test; see D-10 for how the two sequence.
- **`ferro-wallet` OpenSSL → rustls/ring** — coherence follow-up; not in the CLI/release tree (D-03).
- **`x86_64-unknown-linux-musl` fully-static release target** — newly cheap once rustls lands; add
  only if trivial, else its own small phase.

</deferred>

---

*Phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t*
*Context gathered: 2026-06-14*
