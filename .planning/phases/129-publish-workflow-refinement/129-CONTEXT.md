# Phase 129: Publish workflow refinement - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning
**Mode:** `--auto` (Claude selected recommended defaults)

<domain>
## Phase Boundary

Refine the crates.io publish workflow so that workspace members are only
released when a *library* crate actually changed. Also establish documentation
and a forward-looking extension point for the day crates desync from the
current lockstep release cadence.

Scope anchor: absorbs REPORT items **8** and **14** from
`.planning/phases/126-deploy-experience-feedback/REPORT.md`. Everything else
in that report belongs elsewhere.

In scope:
- Gate the auto-patch-bump in `.github/workflows/publish.yml` on whether any
  library crate changed since the last tagged version.
- Exclude `ferro-cli/` and `docs/` (and other non-library paths) from
  triggering a bump.
- Document in `PUBLISHING.md` that `ferro_version` in
  `[package.metadata.ferro.deploy]` is currently a **single global field**
  applied to all ferro crates (lockstep).
- Add a **schema-level override hook** for per-crate `ferro_version` so
  downstream tooling has a forward-compatible place to write per-crate
  values. Parser accepts the field; no functional desync behavior yet.

Out of scope:
- Implementing actual per-crate version resolution / desync handling (deferred
  until a real desync forces it — item 14 explicit deferral).
- Other REPORT items (3, 4, 5, 6, 7, 9, 10, 12, 13, 15, 18, …) — those belong
  to their own phases.
- Changes to the release workflow or tagging scheme.

</domain>

<decisions>
## Implementation Decisions

### Bump Gating (REPORT item 8)

- **D-01:** Detect "library crate changed" via **git diff of paths** between
  the last published tag and `HEAD`, not via `cargo metadata` timestamps.
  Simpler, deterministic, and matches how the publish workflow already
  reasons about changes.
- **D-02:** **Library paths = all workspace member crate directories
  *except* `ferro-cli/`**. `ferro-cli` is an installable binary, not a
  library consumed by downstream apps; its changes must not churn library
  crate versions. If future binary-only crates appear, they extend this
  exclusion list.
- **D-03:** Non-crate path changes that must NOT trigger a bump:
  `docs/`, `.github/`, `README*`, `CHANGELOG*`, `PUBLISHING.md`, `.planning/`,
  top-level `*.md`, top-level config files that don't affect published
  artifacts (`.gitignore`, `.editorconfig`, `rustfmt.toml`, etc.).
- **D-04:** When no library crate changed since the last tag, the
  `check-version` job sets `should_publish=no` (new value) and the
  `bump-version` / publish jobs are skipped entirely. No tag, no commit, no
  release churn.
- **D-05:** When at least one library crate changed, behavior is unchanged
  from today (bump patch, commit, tag, publish waves).
- **D-06:** The gate check runs inside the existing `check-version` job in
  `.github/workflows/publish.yml`; no new jobs. Keeps the pipeline flat.

### Per-Crate Override Hook (REPORT item 14)

- **D-07:** Extend the `[package.metadata.ferro.deploy]` schema in the
  consumer-project `Cargo.toml` (the schema consumed by `ferro-cli` deploy
  commands) with an **optional** `ferro_versions` table keyed by crate name:
  ```toml
  [package.metadata.ferro.deploy]
  ferro_version = "0.2.0"                    # global, still authoritative
  # ferro_versions = { "ferro-json-ui" = "0.2.1" }  # future override, not wired
  ```
- **D-08:** The parser in `ferro-cli/src/deploy/` MUST accept and
  round-trip `ferro_versions` without error, but the rewrite logic
  continues to use `ferro_version` globally for every ferro crate. A TODO
  comment + tracking reference in the code points at the future resolution
  path.
- **D-09:** **No CLI flag, no UX surface, no doctor check** for
  `ferro_versions` yet. This is a schema-only reservation to avoid a
  breaking schema change the day desync actually happens.

### Documentation (REPORT item 14)

- **D-10:** Add a "Version Model" section to `PUBLISHING.md` that states:
  - Ferro currently ships lockstep: every library crate in the workspace
    is published at the same version on every release.
  - Consumer projects pin ferro via a single `ferro_version` field in
    `[package.metadata.ferro.deploy]` which `ferro-cli` applies to every
    ferro dependency during `docker:init` rewrite.
  - This is an intentional simplification, not an architectural stance.
    If a future release requires per-crate versions, the
    `ferro_versions` override hook is reserved for that purpose.
- **D-11:** Also document the bump-gating rule: "A push only triggers a
  patch bump + publish if at least one file under a library crate
  directory changed since the last tag." List the excluded paths.

### Testing

- **D-12:** Shell-script / workflow changes in `.github/workflows/publish.yml`
  are verified by **inline comment annotations and a manual scenario table**
  in `PUBLISHING.md`. Workflow unit testing is out of scope — there's no
  CI-for-CI harness in this repo.
- **D-13:** Schema changes in `ferro-cli` get a Rust regression test in the
  same module as the existing `preserves_package_rename_and_features` test:
  `parses_and_roundtrips_ferro_versions_override`.

### Claude's Discretion
- Exact bash / `git diff --name-only` incantation for the gate check.
- Exact wording of the `PUBLISHING.md` "Version Model" section.
- Whether to inline the excluded-paths list as an env var at the top of
  `publish.yml` or hardcode in the gate step (pick whichever reads better).
- Name of the new output (`should_publish=no` vs `should_publish=skip`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source report (absorbs)
- `.planning/phases/126-deploy-experience-feedback/REPORT.md` §8 — publish
  workflow auto-bumps on every push, needs library-change gating.
- `.planning/phases/126-deploy-experience-feedback/REPORT.md` §14 —
  `ferro_version` is one global field for many crates; document + reserve
  override hook.

### Files this phase will touch
- `.github/workflows/publish.yml` — gate `check-version` on library changes.
- `PUBLISHING.md` — Version Model + gate documentation.
- `ferro-cli/src/deploy/` (specifically the `Cargo.docker.toml` rewrite and
  metadata parser modules — `rewrite_ferro_version.rs` is referenced in
  REPORT §1 and is the most likely home) — accept + round-trip
  `ferro_versions` override map.

### Project conventions
- `CLAUDE.md` — "Run fmt + clippy + tests before every commit" rule applies.
- Memory: `project_ferro_publication.md`, `project_ferro_publish_token_scoping.md`
  (user-local, not in repo; referenced here so planner knows publish-token
  scoping constraints exist).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.github/workflows/publish.yml` — `check-version` job already owns
  tag-detection + `should_publish` output. Extend it, don't fork it.
- `ferro-cli/src/deploy/rewrite_ferro_version.rs` — already parses
  `[package.metadata.ferro.deploy]` and preserves dep metadata (fixed in
  0.2.1). Natural home for `ferro_versions` deserialization and
  round-trip preservation. Existing test module already has the pattern.
- `PUBLISHING.md` at repo root — sole doc for the publish story today;
  add sections there rather than creating a new doc.

### Established Patterns
- Workflow jobs emit outputs via `$GITHUB_OUTPUT`; downstream jobs gate
  via `if: needs.check-version.outputs.should_publish == 'bump'`. New
  `should_publish=no` value slots in naturally — every downstream job
  already gates on this output.
- `ferro-cli` deploy modules use `toml::Value` (not `toml_edit`) per
  REPORT §5. `ferro_versions` round-trip must survive the current
  serializer — test must assert the field is still present after parse +
  re-serialize.
- Workspace version is a single top-level `version = "..."` in root
  `Cargo.toml` — confirms lockstep model the doc should describe.

### Integration Points
- `check-version` job: add a gate step before `should_publish` output.
- `rewrite_ferro_version` parser struct: add `#[serde(default)]`
  `ferro_versions: Option<BTreeMap<String, String>>` field.
- `PUBLISHING.md`: append new "Version Model" and "Publish Gating"
  sections at the appropriate location (planner decides).

</code_context>

<specifics>
## Specific Ideas

- The README mentions the existing 0.2.1 hotfix. Planner should verify the
  last-tag detection logic uses annotated tags of the form `v<semver>`
  (current convention) and does not accidentally treat `v0.2.1-preview` or
  similar as "last published".
- The excluded-paths list should be defined once, centrally in the workflow
  file, so future maintainers extending it (e.g. adding another binary-only
  crate) have a single place to edit.
- Per REPORT §14: "Probably fine for now (lockstep release), but worth a note
  in PUBLISHING.md and a future per-crate override if crates ever desync."
  This phrasing is the user's explicit framing — CONTEXT.md matches it.

</specifics>

<deferred>
## Deferred Ideas

- **Actual per-crate version resolution / desync handling.** Only the schema
  reservation ships in this phase. Wiring `ferro_versions` into the
  rewrite pipeline is a separate phase triggered when a real desync
  forces it.
- **Doctor check for `ferro_versions` correctness** (e.g. warning if the
  map references unknown crates) — belongs with the eventual wiring
  phase, not now.
- **`ferro deploy:check` promotion to CLI command** (REPORT §12) — separate
  phase.
- **Dockerfile ENTRYPOINT / CMD fix** (REPORT §18) — separate phase,
  highest user-pain, should sequence before this one in roadmap if not
  already covered.
- **Silent `copy_dirs` / `.dockerignore` collision** (REPORT §3) — note:
  this appears already touched in the current working tree
  (`ferro-cli/src/doctor/checks/copy_dirs_dockerignore_collision.rs`
  modified), so likely another active phase.
- **`toml_edit` migration for deploy-file round-trip** (REPORT §5) —
  orthogonal cleanup, not required for this phase.

</deferred>

---

*Phase: 129-publish-workflow-refinement*
*Context gathered: 2026-04-09 (auto mode)*
