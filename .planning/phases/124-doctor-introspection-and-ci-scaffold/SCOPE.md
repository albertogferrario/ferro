# Phase 124 — Doctor, introspection, CI scaffold

## Context
Today, validating a Ferro project takes four separate calls: `ferro:info`
(toolchain), `ferro:db` (migrations), manual `cat .env / .env.example` diff
(env completeness), and `git status` (clean tree). Agents stitch this together
every session and the order varies. `ferro routes` is human-formatted only, so
agents scrape its stdout to find handlers — fragile and slow. And every Ferro
project lacks CI on day one because nobody remembers to add `cargo test` +
`api:check` until something breaks in prod. Bundling all three (doctor, JSON
routes, CI scaffold) into one phase because they share the same root cause:
the framework knows what's healthy but doesn't expose it in machine-readable
form.

## Goal
Consolidate the scattered "is my project healthy?" checks into a single
`ferro doctor` command, make `ferro routes` consumable by agents/MCP, and
auto-generate a CI workflow as part of `do:init` so projects ship with quality
gates from day one.

## Scope

### `ferro doctor`
Single command that runs and reports on, in order:
- Toolchain: rustc/cargo version vs `rust-toolchain.toml`.
- DB connection: opens `DATABASE_URL`, runs `SELECT 1`.
- Migrations status: pending vs applied.
- Env completeness: every key in `.env.example` set in `.env`.
- Path deps: any `ferro*` crate using `path =` (warn for prod, ok for dev).
- Workspace: cargo-chef recipe target dirs exist.
- Generated artifacts present: `Dockerfile`, `.dockerignore`, `.do/app.yaml`
  (warn-level if missing).

Output: human-readable by default, `--json` for agent consumption. Exit code
non-zero on any blocker.

### `ferro routes --json`
Extend existing `generate_routes.rs` to emit a stable JSON schema:
```json
{
  "routes": [
    { "method": "GET", "path": "/users/:id", "handler": "users::show", "name": "users.show", "middleware": ["auth"] }
  ]
}
```
Existing pretty-printed output stays as default.

### CI workflow scaffold
`do:init` (and a new `ferro ci:init` for projects not on DO) drops
`.github/workflows/ci.yml`:
- Triggers: `pull_request`, `push` to main.
- Jobs: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`,
  `ferro api:check`, `ferro validate:contracts`.
- Cached cargo registry + target dir.
- Idempotent: skip if file exists unless `--force`.

### dockerignore / gitignore sync
Single source-of-truth template `templates/files/ignore_patterns.toml` listing
patterns by category (rust, sqlite, planning, storage, secrets, ide). Both
`.dockerignore` and `.gitignore` templates render from it. New
`ferro ignore:sync` command reconciles drift in existing projects.

## Verification
- `ferro doctor` against gestiscilo reports path deps (expected) and clean
  otherwise.
- `ferro routes --json` output validates against published JSON schema and is
  consumable by ferro-mcp.
- Generated `ci.yml` runs green on a fresh `ferro new` scaffold.
- Running `ignore:sync` after editing one file mirrors the change to the other.

## Out of scope
- Replacing existing `ferro:info` MCP tool (doctor is complementary).
- GitLab/other CI providers.
