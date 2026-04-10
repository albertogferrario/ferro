# Proposal: Invert the local-vs-published dependency convention

**Status**: draft — awaiting ferro agent review
**Scope**: `ferro-cli`, project scaffolding, doctor checks
**Origin**: downstream DX report from a project consuming ferro from crates.io

## Summary

Today, projects scaffolded by `ferro new` are expected to depend on `ferro*` crates via **path dependencies** pointing to a sibling checkout, with a parallel `Cargo.docker.toml` file carrying the version-based dependencies used by docker and CI builds. A doctor check (`cargo_docker_toml_staleness`) keeps the two files in sync.

This proposal argues for inverting the model: the committed `Cargo.toml` should use crates.io versions as the canonical state, and local development against an unpublished ferro checkout should happen through a transient `[patch.crates-io]` block managed by explicit CLI verbs.

## Current state (as observed)

- `ferro new` Cargo template (`ferro-cli/src/templates/files/backend/Cargo.toml.tpl:12`) already uses `ferro = { version = "0.2" }` — scaffolding is correct.
- In practice, downstream projects replace the version dep with a path dep for local development, then maintain a parallel `Cargo.docker.toml` with version deps for docker builds.
- `cargo_docker_toml_staleness` doctor check (`ferro-cli/src/doctor/checks/cargo_docker_toml_staleness.rs`) compares `ferro*` versions between the two files and warns on drift.
- No CLI verb exists to toggle between local and published dependencies. Hand-editing the dep lines is the only path.

## Problem

A new collaborator cloning a downstream project cannot build without also cloning the ferro repo at a precise relative path (`../../albertogferrario/ferro/`), because the canonical `Cargo.toml` contains path deps. This is undocumented in most downstream repos and breaks the "clone and run" expectation.

Secondary problems:

- Two parallel Cargo manifests drift easily despite the doctor check (the check only compares `ferro*` entries, not the full dep set).
- Docker builds read a different manifest from local dev, so bugs reproduced in one environment can be invisible in the other.
- Agents (human and LLM) have no explicit verb for "use local ferro" vs "use published ferro", so they hand-edit dep lines — noisy diffs, easy to commit accidentally.

## Proposed convention

1. **Canonical `Cargo.toml`** always uses crates.io versions for all `ferro*` crates. This is the committed state.
2. **Local development** against an unpublished ferro checkout uses a `[patch.crates-io]` block, which is treated as a transient local override and never committed.
3. **Docker and CI** build against the canonical `Cargo.toml`. The `Cargo.docker.toml` file is retired.
4. **Explicit CLI verbs** manage the override:
   - `ferro dev:link` — append a `[patch.crates-io]` block for all `ferro*` crates, pointing to `$FERRO_DEV_PATH` (default `../../albertogferrario/ferro`).
   - `ferro dev:link <crate>` — patch only the named crate.
   - `ferro dev:unlink` — remove the entire block.
   - `ferro dev:status` — report whether any override is active and which crates.
5. **Doctor check replacement** — `cargo_docker_toml_staleness` is deprecated. A new check, `cargo_toml_has_local_patch`, warns if a committed `Cargo.toml` contains `[patch.crates-io]` with path entries targeting ferro crates. Wired into `ferro doctor` so the pre-commit gate is a single verb.
6. **`ferro new` scaffold polish** — leave dep lines as version-based (already correct); add a one-line comment near `[dependencies]` pointing to `ferro dev:link` for local development.
7. **Optional: `ferro git:install-hooks`** — writes a `.githooks/pre-commit` that runs `ferro doctor --check cargo_toml_has_local_patch` and sets `core.hooksPath`. Opt-in; shared via git.
8. **Optional: CLAUDE.md section in `ferro claude:install`** — documents the convention so downstream agents discover it on session start without per-project manual setup.

## Trade-offs considered

**Alternative A — keep the dual-file pattern, fix downstream docs.** Low cost, no ferro changes, but leaves the "clone and run" failure mode intact and keeps the drift surface between two manifests.

**Alternative B — publish path + version in the same dep line.** Cargo supports this, but the path must exist on every machine or the build fails. Same blocker as today, just relocated.

**Alternative C — git dependency with a pinned rev.** Makes clone-and-run work without any downstream setup, but forces every ferro iteration to land in a pushed commit before a downstream can test it. Worse inner loop than `[patch]`.

**Alternative D — this proposal.** Highest setup cost (new commands, new doctor check, deprecation path), but gives every downstream project clone-and-run DX by default while preserving a fast inner loop for simultaneous ferro + downstream development.

## Migration impact

### In the ferro repo

- Add `commands/dev_link.rs`, `commands/dev_unlink.rs`, `commands/dev_status.rs` in `ferro-cli`.
- Add `doctor/checks/cargo_toml_has_local_patch.rs`. Register in the doctor registry.
- Mark `cargo_docker_toml_staleness` as deprecated for one release, then remove.
- Update the `ferro new` Cargo template with the hint comment.
- Add tests for each new command and the new doctor check.
- Update `ferro claude:install` CLAUDE.md template with a short section on the convention.
- Update ferro's own `FERRO-BRIEF.md` / relevant docs to describe the new convention.

### In downstream projects (one-time per repo)

- Remove `Cargo.docker.toml`.
- Update Dockerfile (if it references `Cargo.docker.toml`) to use `Cargo.toml` directly.
- Rewrite `ferro*` dep lines in `Cargo.toml` from `path = ...` to `version = "..."`.
- Commit `Cargo.lock` (binary crates should always commit it).
- Optionally run `ferro git:install-hooks`.
- Daily workflow becomes: `ferro dev:link` → hack → `ferro dev:unlink` → commit.

## Open questions for the ferro agent

1. **Deprecation window.** One release cycle, or hard-cut in the next minor version? The dual-file pattern is used by at least one downstream project today (confirmed in a consumer repo).
2. **Command namespace.** `ferro dev:link` vs `ferro patch:link` vs `ferro link` — which fits the existing command surface best?
3. **Default `FERRO_DEV_PATH`.** Hard-code `../../albertogferrario/ferro`, or require the env var to be set explicitly on first use? Hard-coding matches current reality; env var is more portable.
4. **Should `ferro dev:link` edit `Cargo.toml` in place, or write the patch block to a separate file (e.g. `Cargo.toml.local`) that cargo is told to include?** In-place edit is simpler and matches how `[patch]` works natively; a separate file would require workspace gymnastics that cargo does not support cleanly. Recommendation: in-place edit, with the new doctor check as the safety net.
5. **Does ferro want a single unified `ferro doctor --fix` path that auto-runs `dev:unlink` before committing?** Convenient but opinionated.
6. **Scope of the doctor check.** Only warn on `ferro*` path overrides, or any `[patch.crates-io]` path entry? The narrower rule is less noisy; the broader rule catches accidental vendoring of other dependencies.
7. **Coordination with `ferro claude:install`.** Should the CLAUDE.md section land in the same phase as the commands, or as a follow-up?

## Recommendation

Proceed with a single phase that delivers the minimum useful slice: new commands (`dev:link`, `dev:unlink`, `dev:status`), new doctor check (`cargo_toml_has_local_patch`), deprecation notice on `cargo_docker_toml_staleness`, and the scaffold comment. Defer the pre-commit hook installer and the CLAUDE.md section to a follow-up phase so the core convention lands with a narrow diff.

The phase should ship with tests covering:

- Round-trip link → status → unlink on a fresh scaffold.
- Doctor check flags a committed `Cargo.toml` with a ferro path patch.
- Doctor check passes on a clean `Cargo.toml`.
- `ferro new` output contains version deps and the hint comment.
