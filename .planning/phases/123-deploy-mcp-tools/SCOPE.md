# Phase 123 — Deploy MCP tools

## Context
During gestiscilo deployment prep the agent had to manually grep for chromium
usage, diff `.env` against the proposed `app.yaml` envs, and check whether the
local ferro commits were pushed before the Docker build could resolve git
deps. Each of these is a small read-only operation that the agent reinvented
from scratch every time. Surfacing them as MCP tools means future deploys
become a single `deploy_check` call instead of an ad-hoc investigation, and
the same checks can run from CI or pre-commit hooks. This phase is purely
diagnostic — no file mutation, no deploy triggering — so it can ship before or
after Phase 122 without coupling.

## Goal
Expose deploy lifecycle helpers via `ferro-mcp` so agents can validate and
diagnose deploy state without leaving the MCP surface. All tools are read-only.

## Scope

### `deploy_check`
Pre-flight validation. Returns a structured report with severity per finding:
- Missing env vars: keys present in `.env.example` but absent from
  `.do/app.yaml` envs block (or vice versa).
- Path deps still present in `Cargo.toml` for `ferro*` crates (block deploy).
- `DATABASE_URL` pointing at sqlite (block prod deploy).
- Missing `Dockerfile` or `.do/app.yaml`.
- Dirty git tree or unpushed commits on the deploy branch.
- ferro git ref (`--ferro-ref` from generated rewrite script) not reachable on
  remote.

### `deploy_diff_env`
Compare local `.env` against `.do/app.yaml` envs block:
- Keys present in one but not the other.
- Keys with differing scope/type classification (e.g. should be SECRET).
- Output as a 3-column table: key, local, remote.

### `runtime_requirements`
Scan source for known runtime dependencies and report needed apt packages:
- `chromiumoxide` / `headless_chrome` → `chromium`, `fonts-liberation`.
- `ffmpeg-next` → `ffmpeg`.
- `pdfium` → `libpdfium`.
- Generic registry of crate→apt-package mappings, extensible via
  `ferro-cli/src/deploy/runtime_deps.rs`.
- Cross-check against the runtime extras block in the current `Dockerfile` and
  flag missing entries.

## Verification
- Running each MCP tool against gestiscilo returns the expected findings:
  `runtime_requirements` flags `chromium`; `deploy_check` flags path deps until
  Phase 122 rewrite is in place; `deploy_diff_env` shows the gestiscilo `cd`
  file vs the generated `app.yaml` envs block.
- Running against mkmenu (already deployed) returns a clean report.

## Out of scope
- Triggering actual DO App Platform deploys (use `digitalocean-apps` MCP).
- Writing/modifying any project files.
