# `ferro doctor`

Single-command project health diagnostics. Runs seven checks in declared
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

| # | Name               | Purpose                                                       | Reuses                                          |
| - | ------------------ | ------------------------------------------------------------- | ----------------------------------------------- |
| 1 | `toolchain`        | `rustc --version` vs `rust-toolchain.toml` channel (D-02)     | `project::resolve_rust_base_image` (Phase 122)  |
| 2 | `db_connection`    | `DATABASE_URL` reachable via `cargo run -- db:status` (D-03)  | shared subprocess helper                        |
| 3 | `migrations`       | Pending vs applied migration count (D-04)                     | `cargo run -- db:status` subprocess             |
| 4 | `env_completeness` | Every key in `.env.example` is set in `.env` (D-05)           | `deploy::env_example::parse_env_example` (P122) |
| 5 | `path_deps`        | Any `ferro*` path dep — always Warn never Error (D-06)        | `deploy::ferro_deps::find_ferro_path_deps` (P123) |
| 6 | `workspace`        | cargo-chef `target/` and `recipe.json` present (D-07)         | —                                               |
| 7 | `artifacts`        | `Dockerfile`, `.dockerignore`, `.do/app.yaml` present (D-08)  | —                                               |

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
      "name": "toolchain",
      "status": "ok",
      "message": "rustc 1.88.0 matches channel 1.88.0"
    },
    {
      "name": "artifacts",
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
- `checks[].name` — stable identifier (one of the seven names above).
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
