# ferro do:init

Generates the DigitalOcean App Platform deployment spec (`.do/app.yaml`) for a
Ferro project.

## Usage

```bash
ferro do:init [--force]
```

The command takes no positional arguments. The GitHub repository is
auto-detected from `git remote get-url origin`; the region is hardcoded to
`fra1` in the generated template.

## What it writes

Only `.do/app.yaml`. The spec contains:

- A sanitized app `name` derived from the crate name.
- `region: fra1` (hardcoded in the template — edit the file afterwards if you
  need a different region).
- `github.repo` auto-detected from the `origin` git remote, with
  `deploy_on_push: true`.
- A `services:` block with one web entry running the default binary.
- A `workers:` block with one entry per non-test/dev/debug `[[bin]]` declared
  in `Cargo.toml`.
- A commented-out `envs:` scaffold listing every key read from
  `.env.production`. Values are **not** emitted — only the key names — so the
  scaffold is safe to commit. Uncomment and wire up secrets via the DO
  dashboard or `doctl`.

No `databases:` block is generated. Attach managed databases by editing
`.do/app.yaml` directly.

## `.env.production` is required

`do:init` reads `.env.production` to enumerate the keys that belong in the
commented envs scaffold. If the file is missing the command fails hard with:

```
.env.production not found — copy .env.example to .env.production and fill in production values
```

This is intentional: the production env file is the single source of truth
for which keys the deployed app needs. The command never falls back to
`.env.example`.

## Post-scaffold editing is user-owned

After `do:init` writes `.do/app.yaml`, the file is yours. Edit it directly to:

- Change region.
- Uncomment and configure the `envs:` block.
- Add managed databases, domains, alerts, or additional components.
- Adjust resource sizes or instance counts.

`do:init` will refuse to overwrite an existing spec without `--force`.

## Idempotency

- Missing `.do/app.yaml` → written.
- Existing `.do/app.yaml` + no `--force` → refuses to run.
- Existing `.do/app.yaml` + `--force` → regenerated from the current template
  (hand edits are lost).

## CI is decoupled

`do:init` does **not** write `.github/workflows/ci.yml`. CI scaffolding lives
in a separate command: [`ferro ci:init`](./ci-init.md). The two commands are
intentionally independent — one handles DigitalOcean deploy spec, the other
handles GitHub Actions. Run them independently as needed.

## Related commands

- [`ferro ci:init`](./ci-init.md) — standalone CI workflow scaffold.
- `ferro docker:init` — Dockerfile generation (see [CLI reference](../reference/cli.md#ferro-dockerinit)).
- [`ferro doctor`](./doctor.md) — project health diagnostics.
</content>
</invoke>