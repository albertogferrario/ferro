# Phase 128: Deploy preflight - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 128-deploy-preflight
**Mode:** --auto (recommended defaults selected without user prompting)
**Areas discussed:** Check registry & surface, Checks to add, `ferro deploy:init` scaffolder

---

## Check registry & surface

| Option | Description | Selected |
|--------|-------------|----------|
| Extend existing `default_checks()` registry with a category filter | Single registry, new `CheckCategory` enum lets `doctor --deploy` and MCP `deploy_check` filter | ✓ |
| Parallel "preflight" registry | Second `default_deploy_checks()` list | |
| Standalone preflight command with no doctor integration | Bypass doctor, new command only | |

**Auto-selected:** Extend existing registry — honors Phase 122.2 / 126 D-07 "one implementation, two surfaces".

---

## Checks to add

| Option | Description | Selected |
|--------|-------------|----------|
| 3 new checks (copy_dirs collision, ferro_version_skew) + extend existing staleness check | Covers items 3, 4, 13, 17 without duplication | ✓ |
| Single mega-check "deploy_consistency" | One check covers everything | |
| Add all four as separate new checks | Duplicates cargo_docker_toml_staleness | |

**Auto-selected:** 3 new + extend existing — item 17 overlaps the existing staleness check.

---

## `ferro deploy:init` scaffolder

| Option | Description | Selected |
|--------|-------------|----------|
| Interactive prompts + `--dry-run` + `--yes` non-interactive | Mirrors Phase 127 docker:init / do:init pattern | ✓ |
| Interactive only | No automation path | |
| Template file the user edits | No prompts | |

**Auto-selected:** Interactive + `--dry-run` + `--yes` — consistent with Phase 127 convention.

### Follow-up: existing `[package.metadata.ferro.deploy]` table behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Abort by default, prompt for overwrite/merge | Safest | ✓ |
| Always merge | Risk of clobbering hand-tuned values | |
| Always overwrite | Destructive | |

**Auto-selected:** Abort-default with prompt.

---

## Claude's Discretion

- Exact error/warning message wording.
- Whether `cargo_docker_toml_staleness` is renamed or extended in place.
- Data structure for the category filter (enum vs trait method vs tag set).
- Prompt library choice for `deploy:init` (match existing scaffolders).
- Test fixture layout.

## Deferred Ideas

- Auto-fix for `copy_dirs` / `.dockerignore` collisions — diagnose-only in this phase.
- `ferro deploy:doctor` alias — nice-to-have only if filter mechanism makes it trivial.
- Phase 129 publish-workflow gating (already its own phase).
