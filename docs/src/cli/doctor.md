# `ferro doctor`

Single-command project health diagnostics. Runs nine checks in declared
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

| # | Name                           | Purpose                                                                           |
| - | ------------------------------ | --------------------------------------------------------------------------------- |
| 1 | `toolchain_match`              | `rustc --version` vs `rust-toolchain.toml` channel                                |
| 2 | `db_connection`                | `DATABASE_URL` reachable via `cargo run -- db:status`                             |
| 3 | `migrations_pending`           | Pending vs applied migration count                                                |
| 4 | `local_env_parity`             | Every key in `.env.example` is set in `.env`                                      |
| 5 | `deploy_env_parity`            | `.env.production` keys match the commented envs scaffold in `.do/app.yaml`        |
| 6 | `cargo_docker_toml_staleness`  | `Cargo.docker.toml` is up to date vs `Cargo.toml` path deps                       |
| 7 | `generated_artifacts`          | `Dockerfile`, `.dockerignore`, `.do/app.yaml` present                             |
| 8 | `database_url_sqlite_in_prod`  | Warns if `DATABASE_URL` in `.env.production` points at SQLite                     |
| 9 | `git_clean_and_pushed`         | Working tree clean and `HEAD` pushed to the tracked remote                        |

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
- `checks[].name` — stable identifier (one of the nine names above).
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
