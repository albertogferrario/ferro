# `ferro ci:init`

Generate a GitHub Actions CI workflow at `.github/workflows/ci.yml`.

## Usage

```bash
ferro ci:init           # write the workflow if missing
ferro ci:init --force   # overwrite an existing workflow
```

The command walks up from the current directory to locate `Cargo.toml`,
then writes the workflow under `<project-root>/.github/workflows/ci.yml`.

## What gets written

A single `lint-and-test` job that runs the canonical Ferro lint gate:

| Step                       | Command                                       |
| -------------------------- | --------------------------------------------- |
| Format check               | `cargo fmt --all -- --check`                  |
| Lint                       | `cargo clippy --all-targets -- -D warnings`   |
| Tests                      | `cargo test --all-features`                   |
| API readiness              | `cargo run -p ferro-cli -- api:check`         |
| Inertia contract validation| `cargo run -p ferro-cli -- validate:contracts`|

The toolchain is installed via `dtolnay/rust-toolchain@stable` (with
`rustfmt` and `clippy` components) and the cargo registry plus target
directory are cached via `Swatinem/rust-cache@v2`.

## Triggers

The workflow runs on:

- `pull_request` — every PR, regardless of base branch.
- `push` to `main` — to keep the main branch green and to populate the
  Swatinem cache for subsequent PRs.

## Idempotency

`ferro ci:init` is idempotent: if `.github/workflows/ci.yml` already
exists, the command exits non-zero and refuses to overwrite. Pass
`--force` to regenerate the file from the current template.

The renderer is a pure function — running it twice produces byte-identical
output, so `--force` is safe to wire into automation.

## Why `cargo run -p ferro-cli` instead of installing the binary

The `api:check` and `validate:contracts` steps shell out via
`cargo run -p ferro-cli` rather than relying on a globally installed
`ferro` binary. This keeps CI hermetic — every run uses the exact
ferro-cli version pinned in the project's `Cargo.lock`, with no extra
install step and no risk of version drift between local and CI.

## Relationship to `do:init`

`ci:init` and [`do:init`](./do-init.md) are intentionally decoupled:

- `do:init` scaffolds DigitalOcean App Platform deploy config
  (`.do/app.yaml`).
- `ci:init` scaffolds the GitHub Actions workflow
  (`.github/workflows/ci.yml`).

They are independent commands with no shared state and no implicit
chaining — a project can adopt CI without deploying to DO, and vice
versa. Run whichever you need; both are idempotent.
