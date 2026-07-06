# gestiscilo fixture (6f6d397)

This directory freezes the hand-maintained `Dockerfile` and `.do/app.yaml` from
gestiscilo-it/app at commit `6f6d397` as the Phase 131 reference baseline.

**Do not edit these files to make tests pass.** If a test fails because the
scaffolder output does not match these files, fix the scaffolder — not the
fixture. The fixture represents the desired scaffolder output; any delta is a
real gap that needs to be addressed in a downstream plan.

The `Cargo.toml` in this directory is a minimal reconstruction of the
gestiscilo project shape: two `[[bin]]` entries in declaration order and a
`[package.metadata.ferro.deploy]` table derived from what the committed
Dockerfile evidences.

No `frontend/` directory is present — gestiscilo is server-rendered. The
absence of `frontend/package.json` is part of the fixture contract: the
scaffolder must not emit a Node.js frontend build stage for this project.
