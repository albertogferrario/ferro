# Summary: 57-01 — Deployment Template Fixes

## Changes

| # | Fix | File | Line |
|---|-----|------|------|
| 1 | Health check path `/health` → `/_ferro/health` | `ferro-cli/src/templates/files/do/app.yaml.tpl` | 20 |
| 2 | Rust image version `1.75` → `1.88` (matches MSRV) | `ferro-cli/src/templates/files/docker/Dockerfile.tpl` | 19 |
| 3 | Tip text updated to reflect built-in health endpoint | `ferro-cli/src/commands/do_init.rs` | 88 |

## Verification

- `cargo build -p ferro-cli` — compiles clean
- `cargo test -p ferro-cli` — 184 tests passed
- `cargo clippy -p ferro-cli` — no warnings
