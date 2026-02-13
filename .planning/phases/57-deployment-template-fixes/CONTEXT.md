# Phase 57: Deployment Template Fixes

## Source

User report from mkmenu-ferro field test: `../mkmenu-ferro/.planning/ferro-deployment-report.md`

## Issue Analysis

### Confirmed bugs (must fix)

| # | Issue | File | Severity |
|---|-------|------|----------|
| 1 | Health check path `/health` should be `/_ferro/health` | `ferro-cli/src/templates/files/do/app.yaml.tpl:20` | High — deployment health checks fail |
| 2 | Misleading tip "Add a /health endpoint" — endpoint already exists | `ferro-cli/src/commands/do_init.rs:88` | Low — confusing but not broken |
| 3 | Rust version `1.75` should be `1.88` | `ferro-cli/src/templates/files/docker/Dockerfile.tpl:19` | High — builds fail on any modern Rust feature |

### Considered but deprioritized

| # | Issue | Reason to deprioritize |
|---|-------|----------------------|
| 4 | No `GITHUB_TOKEN` ARG for private repos | Ferro targets crates.io publishing; private git deps are transitional. Adds template complexity for an edge case. |
| 5 | Replace empty-project caching with cargo-chef | `cargo-chef` is excellent but adds dependency on third-party Docker image. Current caching works for registry deps. Revisit when Ferro is published on crates.io and the issue is moot. |

### Verdict

Items 1-3 are straightforward fixes in a single plan. Items 4-5 could be a second optional plan if the user wants private-git-dep support.
