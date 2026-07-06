# Phase 156: frontend/src/types/ — Generator-Owned Convention Cleanup

**Gathered:** 2026-05-13 (extended same day — see Production-build gap below); updated 2026-05-14 (exit questions resolved from codebase scan)
**Status:** Ready for planning
**Milestone:** v11.12 DX Housekeeping (proposed — single-phase milestone, or fold into v11.11)
**Driver:** mkmenu translatable-tags field test (2026-05-13); broadened after a downstream production deploy failure surfaced a second leg of the same convention contradiction (Dockerfile path, see In-scope §6).
**Killer feature:** Make the "ignore the output, regenerate from Rust" convention internally consistent across every entry point a Ferro project has — dev loop, fresh clone, and Docker production build — so neither persistent git noise nor broken deploys can occur.

<domain>
## Phase Boundary

Ferro's scaffold ships `gitignore.tpl:14-15` declaring `frontend/src/types/` as generator-owned and therefore git-ignored. Ferro's own reference app (`app/frontend/src/types/`) **violates this convention** — it tracks `inertia-props.ts` and `routes.ts` in git, both produced by the type generator. Every server restart re-emits those files; consumer apps see `M` against the tracked-but-regenerated copies; developers add ad-hoc gitignore lines per-project to suppress the noise. The convention exists but is not enforced anywhere — not in Ferro's reference app, not in scaffold tests, not in a doctor check.

There is a second, more severe leg of the same contradiction: the scaffolded production Dockerfile (`ferro-cli/src/templates/docker.rs`, `FRONTEND_STAGE_BODY`) runs `npm run build` (which invokes `tsc`) inside a `node:20-bookworm-slim` stage that has no Rust toolchain available. With `frontend/src/types/` gitignored per the convention, the generated files are never copied into the build context, `tsc` fails with `TS2307: Cannot find module './inertia-props'` cascading across every page, and the Docker build aborts. This is the same contradiction as the dev-loop noise — the convention assumes "Rust runs first" — but with much higher blast radius (broken production deploys instead of cosmetic git noise). Confirmed downstream on 2026-05-13 when a mkmenu deploy hit this exact failure mode.

The phase reconciles the contradiction by adopting the **ignore-the-output** convention everywhere, end-to-end, while filling the gaps that prevent the convention from holding (gitignore audit, doctor check, docs, and Dockerfile types-gen stage). It does NOT add generator-determinism work, CI drift-checks, or migration scripts beyond what's needed for the reference app — those were considered and rejected (see D-04). The phase remains small in surface area but now spans both dev and prod build paths.

**In scope:**
1. Untrack the two generated files in Ferro's reference `app/frontend/src/types/`.
2. Verify the scaffold gitignore template still says `frontend/src/types/` (it does — audit + comment).
3. Add a single docs page codifying the convention: "`frontend/src/types/` is owned by `ferro generate-types`. Hand-written types go in `frontend/src/lib/types/`."
4. Add a `ferro doctor` check that flags hand-written files under `frontend/src/types/` in any Ferro project.
5. Ensure new-clone DX is sane: document that `cargo run` (or `ferro setup`) must run once before frontend builds.
6. Update the Dockerfile renderer (`ferro-cli/src/templates/docker.rs` `FRONTEND_STAGE_BODY` + `render_dockerfile`) to insert a Rust-toolchain `types-gen` stage before the frontend stage, so `frontend/src/types/` is regenerated from current Rust source on every Docker build. The frontend stage `COPY --from=types-gen` brings the generated files in before `npm run build` runs. See D-15…D-17. Also document, in the docs page from §3, that consumers of older scaffolds must regenerate via `ferro docker:init --force` to pick up the fix.
7. Fix the `generate_types.rs` header comment (lines 710-711) which incorrectly directs users to place custom types in `frontend/src/types/` — change to `frontend/src/lib/types/` to match the convention (see D-18).

**Out of scope (deferred):**
- Making the type generator output deterministic (sorted keys, stable import order). Not needed if we don't track the output; revisit only if a future use case forces it.
- CI drift-check (`ferro generate-types --check`). Same reason — no value when the output isn't tracked.
- Migration scripts for consumer apps that already have hand-written types in `frontend/src/types/` (e.g. mkmenu's `parsed-menu.ts`, `theme-config.ts`, `time-windows*`). Consumer-side cleanup is each project's responsibility; `ferro doctor` (D-09) is how Ferro signals the violation.
- Moving hand-written types in the reference `app/` (the reference app currently has only generated files in `types/`, so this is already clean — only the tracking is wrong).
- Generating `parsed-menu.ts`-style domain types from Rust structs (a separate, larger generator feature).
</domain>

<decisions>
## Implementation Decisions

### Convention direction

- **D-01: Ignore generator output. Do not track it.** Decided after weighing track-vs-ignore explicitly. Tracking would require (a) a deterministic generator, (b) a CI drift-check, (c) a regenerate-before-commit discipline, and (d) per-PR review of derived files that duplicate information from the Rust diff. Ignoring sidesteps all four. Standard industry practice (`dist/`, OpenAPI-derived TS clients, `target/`) supports ignore. The one residual concern — "what if the generator drifts from Rust?" — only arises when the output is tracked; with ignore, every server restart regenerates from current Rust, so drift is impossible by construction.

- **D-02: `frontend/src/types/` is reserved for `ferro generate-types` output.** No hand-written files belong there. This is the rule the existing gitignore template implies but doesn't document or enforce.

- **D-03: Hand-written types live in `frontend/src/lib/types/`.** Conventional location, mirrors `lib/` for non-component code in many React projects. Specific path is a recommendation, not a hard requirement — any path NOT under `frontend/src/types/` works. The convention only locks down what `types/` means, not what `lib/types/` means.

- **D-04: No determinism work in this phase.** Considered and rejected. Determinism would only matter if we tracked the output (per D-01, we don't). The maintenance cost of sorting keys / stabilizing import order would buy nothing as long as ignore is the convention. Revisit only if a future requirement forces tracking.

### Reference app cleanup

- **D-05: `git rm --cached app/frontend/src/types/{inertia-props,routes}.ts`** — untrack only, do not delete from disk. The running dev server keeps using the on-disk copy; no rebuild needed. Commit the deletion in a single chore commit.

- **D-06: Add a comment to `gitignore.tpl:14`** noting that this rule is **load-bearing** for reference app consistency. Current comment is `# generated_types` — augment it to make clear that removing the `frontend/src/types/` line breaks the convention. Without the comment, a future contributor may inline-delete the line not realizing it's enforcing convention. Confirmed from codebase scan: `ferro-cli/src/templates/files/root/gitignore.tpl` line 15 currently reads `# generated_types` with no load-bearing warning.

- **D-07: No edits to existing scaffolded projects' gitignore.** This phase does not touch any consumer-app `.gitignore` — including mkmenu's. Each project owns its own gitignore drift; `ferro doctor` (D-09) is how Ferro signals the violation.

### Documentation

- **D-08: One new docs page.** Title: "Frontend types: generator-owned convention." Location: alongside the existing scaffold/CLI docs (verify exact path during planning — likely `docs/dx/frontend-types.md` or similar). Covers:
  - What `ferro generate-types` produces.
  - Why `frontend/src/types/` is gitignored.
  - Where hand-written types should live (`frontend/src/lib/types/`).
  - How to debug "missing types" errors on a fresh clone (run `cargo run` once — see D-11; no `ferro setup` command exists per D-19).
  - Link from the scaffold README template if one exists.
  - That consumers of older scaffolds must run `ferro docker:init --force` to pick up the Dockerfile fix (D-15).

### Doctor check

- **D-09: `ferro doctor` flags hand-written files under `frontend/src/types/`.** Confirmed from codebase scan: `ferro doctor` exists at `ferro-cli/src/commands/doctor.rs` with a `default_checks()` registry pattern (`ferro-cli/src/doctor/registry.rs`). The new check follows the exact same pattern as the 10 existing checks. Add a new `FrontendTypesConventionCheck` struct in `ferro-cli/src/doctor/checks/frontend_types_convention.rs` and register it in `default_checks()`. Severity: WARNING with an actionable suggestion: "move to `frontend/src/lib/types/` to comply with the generator-owned convention." Does NOT fail the build — advisory only.

- **D-10: `ferro doctor` is opt-in for v0.** Does not run automatically on `cargo build`, `ferro generate-types`, or any deploy command. Developers invoke it manually. Auto-running on `cargo build` is a separate, larger DX question (deferred).

- **D-20: `FrontendTypesConventionCheck` heuristic — use explicit known-generated allowlist.** The generator writes exactly two files: `inertia-props.ts` and `routes.ts`. Any file found in `frontend/src/types/` that is NOT one of these two names is flagged as likely hand-written. Using an explicit allowlist avoids false positives if the directory is otherwise empty. If neither generated file exists (fresh clone before `cargo run`), the directory may not even exist — check only if the directory exists. Auto-resolved from codebase scan: confirmed generator writes exactly those two filenames.

### New-clone DX

- **D-11: Document the bootstrap sequence in the scaffold README template.** Mention that `cargo run` must be executed at least once before `npm run dev` / `npm run build`, because the frontend imports from `frontend/src/types/inertia-props.ts` and `routes.ts` which only exist after the generator runs. No code change to add a prebuild hook — keep the bootstrap manual to avoid coupling frontend npm scripts to the Rust binary.

- **D-12: No frontend `package.json` `prebuild`/`predev` hook.** Considered and rejected. Coupling the frontend build to a Rust toolchain via npm scripts adds fragility (frontend-only contributors hit confusing errors when Cargo isn't installed). The manual bootstrap (run `cargo run` once) is explicit and surfaces the requirement.

- **D-19: No `ferro setup` command exists.** Confirmed from codebase scan: no `setup.rs` in `ferro-cli/src/commands/`. The new-clone docs (D-11) say "run `cargo run` once" — do NOT reference `ferro setup`.

### Versioning & release

- **D-13: This phase touches Ferro's own `app/` reference + the `ferro-cli` doctor command + the Dockerfile renderer + docs.** Cascade workspace version bump per existing release convention. Auto-publish via the standard GH Actions flow. No new crate.

- **D-14: No `ferro-cli` API breaking change.** `ferro doctor` is purely additive — a new check inside the existing doctor registry. The Dockerfile renderer change (D-15) regenerates only when consumers re-run `ferro docker:init --force`; existing rendered Dockerfiles are not touched in place.

### Dockerfile reconciliation

- **D-15: Add a `types-gen` Rust stage to the Dockerfile renderer that runs before the frontend stage.** Concretely: the `FRONTEND_STAGE_BODY` constant in `ferro-cli/src/templates/docker.rs` gains a new `types-gen` stage emitted unconditionally when `has_frontend == true`. The stage uses the same `rust:{{RUST_IMAGE_TAG}}` base as `chef`, installs `ferro-cli` pinned to `{{FERRO_VERSION}}`, and runs `ferro generate-types`. The frontend stage gains a `COPY --from=types-gen /app/frontend/src/types ./src/types` line before `RUN npm run build`. This satisfies D-01 (output stays gitignored) and D-12 (no npm `prebuild` hook). Sketch:
  ```dockerfile
  FROM rust:{{RUST_IMAGE_TAG}} AS types-gen
  WORKDIR /app
  RUN cargo install ferro-cli --version {{FERRO_VERSION}} --locked
  COPY . .
  RUN ferro generate-types

  FROM node:20-bookworm-slim AS frontend-builder
  WORKDIR /frontend
  COPY frontend/package.json frontend/package-lock.json* ./
  RUN npm ci || npm install
  COPY frontend/ ./
  COPY --from=types-gen /app/frontend/src/types ./src/types
  RUN npm run build
  ```

- **D-16: Pin `ferro-cli` version in the rendered Dockerfile.** The `types-gen` stage uses `cargo install ferro-cli --version {{FERRO_VERSION}} --locked`. The `{{FERRO_VERSION}}` token is resolved by adding a `ferro_version: String` field to `DockerContext` (confirmed: field does NOT currently exist in `ferro-cli/src/templates/docker.rs` `DockerContext` struct). The `render_dockerfile` function adds the `.replace("{{FERRO_VERSION}}", &ctx.ferro_version)` substitution alongside the existing token replacements. Cost: ~30–60s `cargo install` on cold builds, fully cached otherwise. Cheaper alternatives deferred per D-16.

- **D-17: Bind types-gen ordering, not the rest of the build.** The `types-gen` stage's only consumer is `frontend-builder` via `COPY --from=types-gen`. The `chef`/`planner`/`backend-builder` chain is unaffected. No restructuring of the Rust build pipeline. The new stage is additive.

- **D-21: `FERRO_VERSION` source — parse from project's `Cargo.lock`.** Auto-resolved from codebase scan: `DockerContext` has no `ferro_version` field; it must be added. The `docker_init` command's call site (`ferro-cli/src/commands/docker_init.rs`) must parse the project's `Cargo.lock` and extract the resolved version of the `ferro-rs` package (package name in Cargo.lock is `ferro-rs`). This gives the exact version the project compiles against, matching D-16's preference for accuracy. The caller resolves it from `Cargo.lock` and passes it into `DockerContext::ferro_version`; `render_dockerfile` stays pure (no I/O). The `docker_template_drift` doctor check's call site must also be updated to supply `ferro_version`. Fallback if Cargo.lock is absent or ferro-rs not found: use the current binary's own version (`env!("CARGO_PKG_VERSION")`).

### Generator header fix

- **D-18: Fix `generate_types.rs` header comment.** Confirmed from codebase scan: `ferro-cli/src/commands/generate_types.rs` lines 710-711 currently say:
  ```
  // For custom types not generated here, create manual type files in:
  // frontend/src/types/
  ```
  This contradicts D-02/D-03 — it directs users to the generator-owned directory. Fix to:
  ```
  // For custom types not generated here, create manual type files in:
  // frontend/src/lib/types/
  ```
  In scope: this is part of the "docs codifying the convention" deliverable (§3/§7 of phase boundary).

### Claude's Discretion
- Exact docs page path (`docs/dx/frontend-types.md` vs another location) — verify during planning what the docs file naming convention is.
- Whether the `debug_assert!` in `render_dockerfile` for unresolved `{{` tokens needs a counterpart for `{{FERRO_VERSION}}` in the types-gen path.
- CI Docker build verification approach (D-07 success criterion): if no existing CI Docker step exists, add a local verification recipe to the docs page rather than a new CI step, unless CI already has one.

</decisions>

<risks>
## Risks

1. **Consumer apps that have hand-written files in `frontend/src/types/` will see `ferro doctor` warnings after upgrade.** Acceptable — that's the intended signal. mkmenu has 5 such files (`parsed-menu.ts`, `theme-config.ts`, `time-windows.ts`, `time-windows.test.ts`, `shared.ts`). The warnings are advisory, not blocking.

2. **The doctor heuristic uses an explicit allowlist (`inertia-props.ts`, `routes.ts`).** Confirmed from codebase scan that the generator writes exactly these two files. If the generator is extended to emit additional files, the allowlist in `FrontendTypesConventionCheck` must be kept in sync. This is a low-probability maintenance hazard — document the allowlist's relationship to the generator in a comment.

3. **Frontend-only contributors on a fresh clone will hit `Cannot find module './types/inertia-props'` errors until they run `cargo run`.** Mitigated by docs (D-11); not eliminated. A future phase could add a `ferro setup` command that bootstraps types without starting the full server — out of scope here.

4. **Existing scaffolded projects' Dockerfiles continue to break until consumers run `ferro docker:init --force`.** D-15 only updates the renderer; rendered Dockerfiles in existing projects are untouched (per D-14). Mitigation: docs page (D-08) names the upgrade command explicitly.

5. **Cold Docker builds gain ~30–60s for `cargo install ferro-cli` in the `types-gen` stage (D-16).** Cached on the second build onward. Acceptable trade-off versus broken builds.

6. **`FERRO_VERSION` in `DockerContext` is parsed from `Cargo.lock`.** If `Cargo.lock` is absent (rare for binary crates, possible in fresh-clone scenarios before `cargo build`) or doesn't have a `ferro-rs` entry (even rarer), the fallback uses `env!("CARGO_PKG_VERSION")`. This is a known edge case; the fallback should be documented in a comment near the parse logic.

</risks>

<success_criteria>
## Success Criteria

- `app/frontend/src/types/inertia-props.ts` and `app/frontend/src/types/routes.ts` are no longer tracked in git. `git status` is clean after a fresh `cargo run` against the reference app.
- `gitignore.tpl:14-15` carries a load-bearing-comment annotation.
- `generate_types.rs` header comment points to `frontend/src/lib/types/` not `frontend/src/types/`.
- A docs page exists explaining the convention, linked from the scaffold README template if present, and naming `ferro docker:init --force` as the upgrade path for existing scaffolded projects.
- `ferro doctor` flags hand-written files under `frontend/src/types/` with a clear actionable message. Verified against the reference app (clean — no warnings) and against a synthetic project with a planted hand-written file (one warning).
- The scaffold Dockerfile renderer, after this phase, produces a working `docker build .` against any Ferro project that has `frontend/src/types/` gitignored and no committed copy of `inertia-props.ts` / `routes.ts`. Verified end-to-end against Ferro's reference `app/` (no committed types after D-05) by running an actual `docker build` in CI or locally; `tsc` resolves all imports from `./types/` and `npm run build` exits 0.
- Renderer tests in `ferro-cli/src/templates/docker.rs` cover: (a) `types-gen` stage is present when `has_frontend == true`, (b) absent when `has_frontend == false`, (c) frontend stage `COPY --from=types-gen` line appears immediately before `RUN npm run build`, (d) `{{FERRO_VERSION}}` is resolved (no unrendered template tokens).
- `DockerContext` has `ferro_version: String` field; `render_dockerfile` replaces `{{FERRO_VERSION}}` correctly.
- Ferro workspace version bumps cleanly; CI green; package publishes.

</success_criteria>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above.

### Key implementation files (read before writing code)

- `ferro-cli/src/commands/generate_types.rs` — generator implementation; fix header comment (D-18) at lines 710-711
- `ferro-cli/src/templates/docker.rs` — `FRONTEND_STAGE_BODY`, `DockerContext`, `render_dockerfile`; add types-gen stage (D-15), `ferro_version` field (D-16/D-21)
- `ferro-cli/src/commands/docker_init.rs` — call site that builds `DockerContext`; add `ferro_version` parsing from Cargo.lock
- `ferro-cli/src/doctor/registry.rs` — `default_checks()` registry; add new `FrontendTypesConventionCheck`
- `ferro-cli/src/doctor/checks/` — directory where new `frontend_types_convention.rs` goes; follow existing check pattern
- `ferro-cli/src/templates/files/root/gitignore.tpl` — line 15 `# generated_types`; augment comment (D-06)
- `ferro-cli/src/doctor/checks/docker_template_drift.rs` — second call site for `DockerContext`; update to supply `ferro_version`
- `app/frontend/src/types/` — reference app; verify tracked files to untrack (D-05)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `DoctorCheck` trait in `ferro-cli/src/doctor/check.rs` — new `FrontendTypesConventionCheck` implements this
- `CheckResult::warn()` / `CheckResult::ok()` — use for the check output
- `default_checks()` in `ferro-cli/src/doctor/registry.rs` — register new check here (10 checks currently; becomes 11)
- `render_dockerfile()` in `ferro-cli/src/templates/docker.rs` — pure function; add `{{FERRO_VERSION}}` replace call

### Established Patterns
- Doctor checks: one file per check in `ferro-cli/src/doctor/checks/`, struct + `DoctorCheck` impl, `check_impl()` fn for testability, unit tests with `TempDir`
- Template tokens: `{{TOKEN_NAME}}` syntax, resolved in `render_dockerfile` via chained `.replace()` calls
- `DockerContext` is caller-resolved (no I/O in renderer) — `ferro_version` must be resolved at the `docker_init.rs` and `docker_template_drift.rs` call sites

### Integration Points
- `docker_template_drift.rs` doctor check constructs a `DockerContext` — must also supply `ferro_version` after D-16
- `generate_types.rs` `run()` function emits the header — fix the comment in the hardcoded `output.push_str()` calls around line 710

</code_context>

<specifics>
## Specific Ideas

- The `types-gen` Docker stage is a new inline constant `TYPES_GEN_STAGE_BODY` analogous to `FRONTEND_STAGE_BODY` — or it can be inlined directly into the `FRONTEND_STAGE_BODY` constant. Simpler to keep them separate and concatenate: `let frontend_stage = TYPES_GEN_STAGE_BODY + FRONTEND_STAGE_BODY` when `has_frontend`.
- The `generate_types.rs` header fix is a trivial two-line string change; it should be committed as a separate chore commit to keep the diff legible.

</specifics>

<deferred>
## Deferred Ideas

- Making the type generator output deterministic (sorted keys, stable import order)
- CI drift-check (`ferro generate-types --check`)
- Migration scripts for consumer apps with hand-written types in `frontend/src/types/`
- `ferro setup` command for bootstrapping types without starting the full server
- Generating `parsed-menu.ts`-style domain types from Rust structs
- `ferro doctor` check that detects an outdated rendered Dockerfile (heuristic: `frontend-builder` stage present without preceding `types-gen` stage)
- Cheaper alternatives to `cargo install` in types-gen stage (cargo-binstall, prebuilt release binary download)

</deferred>

---

*Phase: 156-frontend-types-directory-generator-owned-convention*
*Context gathered: 2026-05-13; updated 2026-05-14 (exit questions resolved)*
