# Phase 129: Publish workflow refinement - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 129-publish-workflow-refinement
**Mode:** `--auto` (Claude selected recommended defaults for every gray area)
**Areas discussed:** Bump-gating detection, Library-path definition, Override schema shape, Documentation placement

---

## Bump-gating detection

| Option | Description | Selected |
|--------|-------------|----------|
| git diff paths vs last tag | Shell `git diff --name-only vX.Y.Z..HEAD`, match against library paths | ✓ |
| cargo metadata mtime | Resolve workspace members and diff per-crate | |
| Dedicated change-detection action | Third-party `dorny/paths-filter` or similar | |

**User's choice:** git diff paths (auto — simplest, deterministic, no external dependency)
**Notes:** Matches existing shell-based `check-version` job style.

---

## Library-path definition

| Option | Description | Selected |
|--------|-------------|----------|
| All workspace crates except `ferro-cli/` | Binary installable, not a published library consumers depend on at runtime | ✓ |
| All workspace crates, no exclusions | Simplest, but defeats the purpose | |
| Manual allowlist in workflow | Explicitly list every library crate | |

**User's choice:** Exclude `ferro-cli/` and non-crate dirs (`docs/`, `.github/`, `.planning/`, top-level md)
**Notes:** Excluded path set documented in D-03.

---

## Per-crate override schema shape

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro_versions` map in `[package.metadata.ferro.deploy]` | Optional table keyed by crate name, parser round-trips, no runtime wiring | ✓ |
| New top-level `[package.metadata.ferro.versions]` table | Separate namespace | |
| Command-line flag only | No schema change, pass overrides at `docker:init` time | |

**User's choice:** Optional map alongside existing `ferro_version`
**Notes:** Schema-only reservation. Parser accepts + round-trips; rewrite logic unchanged.

---

## Documentation placement

| Option | Description | Selected |
|--------|-------------|----------|
| New sections in `PUBLISHING.md` | Single source of truth for publish story already | ✓ |
| New `docs/src/publish/version-model.md` | Fits mdBook structure | |
| Only code comments | Undiscoverable | |

**User's choice:** `PUBLISHING.md` sections (Version Model + Publish Gating)
**Notes:** `docs/src/` entry can be added later if the publish story grows; not worth splitting now.

---

## Claude's Discretion

- Exact `git diff` invocation and tag-resolution
- Workflow output name (`should_publish=no` vs `skip`)
- Wording of `PUBLISHING.md` new sections
- Whether excluded-paths list lives in env var or inline

## Deferred Ideas

See `129-CONTEXT.md` <deferred> section.
