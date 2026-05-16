# `ferro doctor`

Single-command project health diagnostics. Runs eleven checks in declared
order, prints colored human output by default, or a stable JSON schema with
`--json` for agent / CI consumption.

```bash
ferro doctor
ferro doctor --json | jq '.summary'
```

## Relationship to `ferro:info` MCP tool

`ferro doctor` is **complementary** to the `ferro:info` MCP introspection tool
— it does **not** replace it (D-22). `ferro:info` describes _what_ the project
is (routes, models, installed crates); `ferro doctor` answers _is it healthy?_.
Use `ferro:info` for understanding, `ferro doctor` for validation.

## Checks

Checks run in this exact order (D-01):

| #  | Name                                 | Category | Purpose                                                                    |
| -- | ------------------------------------ | -------- | -------------------------------------------------------------------------- |
| 1  | `toolchain_match`                    | General  | `rustc --version` vs `rust-toolchain.toml` channel                         |
| 2  | `db_connection`                      | General  | `DATABASE_URL` reachable via `cargo run -- db:status`                      |
| 3  | `migrations_pending`                 | General  | Pending vs applied migration count                                         |
| 4  | `local_env_parity`                   | General  | Every key in `.env.example` is set in `.env`                               |
| 5  | `deploy_env_parity`                  | General  | `.env.production` keys match the commented envs scaffold in `.do/app.yaml` |
| 6  | `copy_dirs_dockerignore_collision`   | Deploy   | `copy_dirs` entries not silently excluded by `.dockerignore`               |
| 7  | `docker_template_drift`              | Deploy   | Committed `Dockerfile` matches current scaffolder output                   |
| 8  | `generated_artifacts`                | General  | `Dockerfile`, `.dockerignore`, `.do/app.yaml` present                      |
| 9  | `database_url_sqlite_in_prod`        | General  | Warns if `DATABASE_URL` in `.env.production` points at SQLite              |
| 10 | `git_clean_and_pushed`               | General  | Working tree clean and `HEAD` pushed to the tracked remote                 |
| 11 | `frontend_types_convention`          | General  | No hand-written files in `frontend/src/types/` (see [frontend-types](frontend-types.md)) |

Check #4 (`local_env_parity`) and #5 (`deploy_env_parity`) are powered by the
`deploy::env_production::parse_env_production_keys` module, which parses
`.env.production` for key names only (values are never read).

## Status semantics

| Status  | Meaning                                                                 |
| ------- | ----------------------------------------------------------------------- |
| `ok`    | Check passed                                                            |
| `warn`  | Non-blocking issue (recommended fix; does not affect exit code)         |
| `error` | Blocking issue (forces non-zero exit)                                   |

## Exit code contract (D-09)

| Overall status | Exit code |
| -------------- | --------- |
| All `ok`       | `0`       |
| Any `warn`     | `0`       |
| Any `error`    | `1`       |

The contract: **non-zero iff at least one check returned `error`**. Warnings
never block. This makes doctor safe to drop into CI without false positives
on dev-mode path deps.

## JSON schema

`ferro doctor --json` emits this stable shape:

```json
{
  "summary": {
    "overall": "warn",
    "ok": 5,
    "warn": 2,
    "error": 0
  },
  "checks": [
    {
      "name": "toolchain_match",
      "status": "ok",
      "message": "rustc 1.88.0 matches channel 1.88.0"
    },
    {
      "name": "generated_artifacts",
      "status": "warn",
      "message": "1 artifact(s) missing",
      "details": "missing: .do/app.yaml"
    }
  ]
}
```

Field reference:

- `summary.overall` — `ok` | `warn` | `error` — worst status across all checks.
- `summary.ok` / `warn` / `error` — counts.
- `checks[].name` — stable identifier (one of the eleven names above).
- `checks[].status` — `ok` | `warn` | `error`.
- `checks[].message` — short human-readable summary.
- `checks[].details` — optional, present only when extra context exists.

## Examples

```bash
# Full human report (default)
ferro doctor

# Machine output for CI / agents
ferro doctor --json

# Just the overall status
ferro doctor --json | jq -r '.summary.overall'

# List failing checks
ferro doctor --json | jq '.checks[] | select(.status == "error")'

# Use in CI as a gate
ferro doctor || exit 1
```

## Deploy filter (`--deploy`)

`ferro doctor --deploy` runs only the checks with category `Deploy`
(currently `copy_dirs_dockerignore_collision`). The same filter is
available via the `deploy_check` MCP tool (see below). Combining
`--deploy` with `--json` produces the same Report schema as a full run,
filtered to deploy-category checks.

```bash
ferro doctor --deploy
ferro doctor --deploy --json | jq '.summary'
```

## Preflight checks

`Deploy`-category checks catch failures that would otherwise surface only
after a 1–10 minute Docker round-trip.

- **`copy_dirs_dockerignore_collision`** — flags any `copy_dirs` entry in
  `[package.metadata.ferro.deploy]` that is excluded by `.dockerignore`, which
  would silently drop files from the Docker build context.

## `ferro deploy:init`

Scaffolds the `[package.metadata.ferro.deploy]` table in the root `Cargo.toml`.
Detects sensible defaults (binary name, common directories) and writes the block
in-place using `toml_edit`, preserving comments and key order.

```bash
ferro deploy:init [--yes] [--dry-run]
```

- `--yes` — accept all detected defaults without interactive prompts. Errors if
  a required value (e.g., `web_bin`) cannot be inferred.
- `--dry-run` — print the block that would be written without modifying any
  files.

Example block produced for a project with a `migrations/` directory and a
binary named `myapp`:

```toml
[package.metadata.ferro.deploy]
runtime_apt = []
copy_dirs = ["migrations"]
web_bin = "myapp"
```

If `[package.metadata.ferro.deploy]` already exists in `Cargo.toml`, the
command prompts for a collision policy: **abort** (default), **overwrite**
(replace existing values), or **merge** (add missing keys only). Passing
`--yes` without a pre-existing table always succeeds; with a pre-existing table
it defaults to abort — use interactive mode to select overwrite or merge.

## `deploy_check` MCP tool

The `deploy_check` MCP tool is the MCP surface of the same deploy check
registry. Internally it runs `ferro doctor --deploy --json` from the project
root and returns the JSON Report verbatim.
