# Crates.io Publishing Checklist

Publishing checklist for ferro framework crates.

## Prerequisites

- [ ] `cargo login` with crates.io token
- [ ] Verify crates.io account has publish permissions
- [ ] All CI tests passing on master branch

## Pre-publish Verification

Run these checks before publishing any crate:

```bash
# Verify workspace builds
cargo build --workspace

# Run tests
cargo test --workspace

# Check for publishing issues
cargo publish --dry-run -p <crate-name>
```

## Publish Gating

The Publish workflow (`.github/workflows/publish.yml`) only runs a patch
bump and crates.io release when at least one file under a library crate
path has changed since the last `v*` tag. Pushes that touch only non-library
paths leave the workspace version untouched and exit the workflow after the
`check-version` job.

### Excluded paths

The following paths do NOT count as library changes:

- `ferro-cli/*` — installable binary, not a published library
- `app/*` — sample application, not published
- `docs/*` — user documentation
- `.github/*` — CI configuration
- `.planning/*` — planning notes
- `scripts/*` — developer scripts
- Top-level `*.md` files (`README.md`, `PUBLISHING.md`, `CLAUDE.md`, etc.)
- `LICENSE`
- `Cargo.lock` — regenerated automatically
- Top-level config files: `.gitignore`, `.editorconfig`, `rustfmt.toml`,
  `bacon.toml`, `deny.toml`, `dev.sh`, `llms.txt`, `rust-toolchain.toml`

All other paths — including the workspace-root `Cargo.toml` and every
library crate directory (`framework/`, `ferro-macros/`, `ferro-events/`,
`ferro-queue/`, `ferro-notifications/`, `ferro-broadcast/`, `ferro-storage/`,
`ferro-cache/`, `ferro-mcp/`, `ferro-inertia/`, `ferro-json-ui/`,
`ferro-lang/`, `ferro-api-mcp/`, `ferro-projections/`, `ferro-stripe/`,
`ferro-theme/`, `ferro-ai/`, `ferro-whatsapp/`) — count as library changes.

`ferro-cli` is excluded because it is the only workspace binary consumed
directly by end users via `cargo install`; its own commits must not churn
the versions of the libraries it embeds. `app/` is excluded because it is
a reference application, not a published crate. If future binary-only
crates are added, extend the exclusion list in `publish.yml` accordingly.

### First run

When no `v*` tag exists yet, the gate treats every file as a library
change and the workflow publishes normally. This preserves bootstrap
behavior on an empty tag history.

### Scenarios

| Changed paths since last `v*` tag | Gate output | Result |
|-----------------------------------|-------------|--------|
| Only `docs/` | `should_publish=none` | No bump, no publish |
| Only `ferro-cli/src/...` | `should_publish=none` | No bump, no publish |
| Only `.github/workflows/...` | `should_publish=none` | No bump, no publish |
| Only top-level `*.md` | `should_publish=none` | No bump, no publish |
| `framework/src/lib.rs` | `should_publish=yes` | Bump + publish waves run |
| Workspace-root `Cargo.toml` | `should_publish=yes` | Bump + publish waves run |
| Mix of `docs/` and `framework/` | `should_publish=yes` | Bump + publish waves run |
| No `v*` tag exists yet | `should_publish=yes` | Publish (first run) |

## Publishing Order

Crates must be published in dependency order. Wait for each wave to be indexed on crates.io before proceeding to the next.

### Wave 1: Independent Crates (no internal dependencies)

These crates have no dependencies on other ferro crates and can be published in parallel:

```bash
cargo publish -p ferro-macros
cargo publish -p ferro-events
cargo publish -p ferro-queue
cargo publish -p ferro-notifications
cargo publish -p ferro-broadcast
cargo publish -p ferro-storage
cargo publish -p ferro-cache
cargo publish -p ferro-inertia
cargo publish -p ferro-mcp
```

**Wait 5-10 minutes for crates.io to index these crates before proceeding.**

### Wave 2: Main Framework

Depends on all Wave 1 crates:

```bash
cargo publish -p ferro
```

**Wait for crates.io to index ferro before proceeding.**

### Wave 3: CLI

Depends on ferro-mcp:

```bash
cargo publish -p ferro-cli
```

## Path Dependency Handling

Before publishing, path dependencies must be replaced with version-only dependencies. The current Cargo.toml files use both path and version:

```toml
# Current (works for publishing)
ferro-macros = { path = "../ferro-macros", version = "0.1" }
```

This format allows `cargo publish` to automatically use the version when uploading to crates.io.

If you see errors about path dependencies:
1. Comment out the `path = "..."` portion
2. Publish the crate
3. Restore the path for local development

## Post-publish Verification

After publishing each crate:

```bash
# Verify crate is available
cargo search <crate-name>

# Test installation from crates.io
cargo add <crate-name>
```

## Version Model

Ferro ships in lockstep: every library crate in the workspace is published at
the same version on every release. The authoritative version lives in the
workspace-root `Cargo.toml` `version` field and is propagated to each member
crate on publish.

Consumer projects pin ferro by depending on `ferro-rs` directly in their
project `Cargo.toml`. Developers working against an unpublished local
checkout add an uncommitted `[patch.crates-io]` block at the bottom of
the project `Cargo.toml` — Docker builds then consume that manifest as-is
without any rewrite step.

Release checklist:

1. Bump the workspace-root `Cargo.toml` `version` field.
2. Commit the version bump as a single atomic change.
3. Push to `master`. The Publish workflow handles tagging and crates.io
   upload per the gating rule documented above.

### Per-crate override reservation

The `[package.metadata.ferro.deploy]` schema also accepts an optional
`ferro_versions` table keyed by crate name:

```toml
[package.metadata.ferro.deploy]
ferro_version = "0.2.0"

# Reserved. Parsed and round-tripped today; not yet consulted by the
# rewrite pipeline. See Phase 129 in .planning/ for the rationale.
# [package.metadata.ferro.deploy.ferro_versions]
# ferro-json-ui = "0.2.1"
```

This field is a schema reservation for the day library crates desync from
the current lockstep cadence. The parser in `ferro-cli` accepts and
round-trips it, but every rewrite still uses the global `ferro_version`
above. When a desync occurs, the rewriter will resolve per-crate overrides
from this table without requiring a breaking schema change in downstream
projects.

## Troubleshooting

### "crate not found" errors during publish

The dependent crate isn't indexed yet. Wait 5-10 minutes and retry.

### "already uploaded" errors

The crate version already exists on crates.io. Bump the version number and try again.

### Path dependency rejection

Comment out `path = "..."` from the dependency declaration, publish, then restore.

## Crate Summary

| Crate | Package Name | Wave |
|-------|--------------|------|
| ferro-macros | ferro-macros | 1 |
| ferro-events | ferro-events | 1 |
| ferro-queue | ferro-queue | 1 |
| ferro-notifications | ferro-notifications | 1 |
| ferro-broadcast | ferro-broadcast | 1 |
| ferro-storage | ferro-storage | 1 |
| ferro-cache | ferro-cache | 1 |
| ferro-inertia | ferro-inertia | 1 |
| ferro-mcp | ferro-mcp | 1 |
| framework | ferro | 2 |
| ferro-cli | ferro-cli | 3 |
