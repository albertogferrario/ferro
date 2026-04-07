# ferro do:init

Generates the DigitalOcean App Platform deployment spec (`.do/app.yaml`) and
supporting scaffolding for a Ferro project.

## Usage

```bash
ferro do:init --repo owner/repo [--region nyc] [--force]
```

## What it writes

- `.do/app.yaml` — App Platform spec derived from `Cargo.toml` (package name,
  binaries) and `.env.example` (env vars, managed database binding).
- `Dockerfile` + rewrite script — container image for the app (via
  `docker:init`).
- `.github/workflows/ci.yml` — canonical Ferro lint gate (fmt + clippy + test +
  `api:check` + `validate:contracts`). See [ci:init](./ci-init.md) for the
  standalone command that writes this same file.

## CI workflow

Since Phase 124 (D-13), `do:init` also drops `.github/workflows/ci.yml` using
the canonical template rendered by
`ferro_cli::templates::ci_workflow::render_ci_workflow`. This is the exact same
renderer used by [`ferro ci:init`](./ci-init.md) — there is a single source of
truth for CI workflow contents.

This means a project deployed via `do:init` ships with CI from day one: no
second command required.

## Idempotency

- If `.do/app.yaml` already exists, `do:init` refuses to run without `--force`.
- If `.github/workflows/ci.yml` already exists, it is left untouched and a
  notice is printed (the command does not abort). Pass `--force` to overwrite
  both files.

## Related commands

- [`ferro ci:init`](./ci-init.md) — standalone CI scaffold (same renderer).
- [`ferro doctor`](./doctor.md) — introspection & environment checks.
