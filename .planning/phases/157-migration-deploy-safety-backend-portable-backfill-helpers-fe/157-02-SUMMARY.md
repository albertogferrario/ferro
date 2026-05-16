---
plan: 157-02
phase: 157
status: complete
started: 2026-05-14T03:00:00Z
completed: 2026-05-14T13:43:46Z
self_check: PASSED
---

## Summary

Wired the existing `{{JOBS_BLOCK}}` template token to emit a PRE_DEPLOY migrate job in `.do/app.yaml`. Every project that runs `ferro do:init` now gets a working migrate gate without hand-editing. The `render_jobs_block` function computes the job block from existing `web_bin`/`repo`/`branch` fields — no new `AppYamlContext` fields needed.

## What Was Built

- **`ferro-cli/src/templates/do.rs`** — added `render_jobs_block` private function producing the full PRE_DEPLOY job YAML block; wired into `render_app_yaml` with `.replace("{{JOBS_BLOCK}}", &jobs_block)`; added `render_app_yaml_emits_predeploy_migrate_job` unit test
- **`ferro-cli/tests/fixtures/gestiscilo/app.yaml`** — updated integration fixture to include the jobs block (PRE_DEPLOY migrate job)
- **`ferro-cli/tests/gestiscilo_fixture.rs`** — fixed `render_app_yaml_uses_preserved_identity_over_defaults` test to assert only on the top-level `name:` field, not all occurrences (the jobs block legitimately uses `web_bin` in `run_command`)

## Key Files

- `ferro-cli/src/templates/do.rs:123-138` — `render_jobs_block`
- `ferro-cli/src/templates/do.rs:62,70` — `jobs_block` computation and `.replace` call
- `ferro-cli/src/templates/do.rs:347-377` — new `render_app_yaml_emits_predeploy_migrate_job` test

## Commits

- `4ff1ef05` — feat(157-02): wire {{JOBS_BLOCK}} to emit PRE_DEPLOY migrate job, update fixture

## Deviations

None. Task 2 (no-regression sweep) confirmed correct by inspection — `AppYamlContext` has no new fields, so all construction sites compile without changes.

## Self-Check

- [x] `render_jobs_block` function present in `do.rs`
- [x] `{{JOBS_BLOCK}}` replaced in `render_app_yaml`
- [x] New test `render_app_yaml_emits_predeploy_migrate_job` present
- [x] Job has `kind: PRE_DEPLOY` and `deploy_on_push: false`
- [x] Fixture updated to include jobs block
- [x] Integration test assertion corrected
