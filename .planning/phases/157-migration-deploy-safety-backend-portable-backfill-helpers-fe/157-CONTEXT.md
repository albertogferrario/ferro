# Phase 157 — Migration Deploy Safety

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Close three migration-deploy safety gaps surfaced by a 2026-05-13 gestiscilo-it production failure. In scope: backend-portable backfill helpers, `ferro do:init` PRE_DEPLOY migrate job scaffolding, `ferro doctor --deploy` `migrate_gate` check, and fixing the silent-failure migration runner in framework code. Out of scope: multi-region coordination, Postgres extension management, migration squashing.

</domain>

## What happened in the field

1. Developer adds a migration that does a backfill via raw SQL hardcoded to `DbBackend::Sqlite`. Pattern:

   ```rust
   conn.execute(Statement::from_string(
       DbBackend::Sqlite,
       "UPDATE … = lower(hex(randomblob(16))) WHERE …".to_string(),
   )).await?;
   ```

2. Deploy to DigitalOcean App Platform (Postgres backend). Migration fails because `randomblob`/`hex` are SQLite-only.

3. The runtime migration runner in `framework/src/app.rs` (`run_migrations_silent`) does `eprintln!("Warning: …")` then continues. The server starts with a stale schema.

4. `.do/app.yaml` has no `PRE_DEPLOY` job. No gate between "deploy started" and "traffic shifted to broken revision."

5. `ferro doctor --deploy` didn't catch any of this — its only deploy checks are `docker_template_drift` and `copy_dirs_dockerignore_collision`.

<decisions>
## Implementation Decisions

### D-01: Migration helper crate location
- **New `ferro-migration` crate** — not inlined into `framework`. The public API is `ferro_migration::backfill_random_hex`; this is a first-class crate, not an internal helper. Add it to Wave 1 of the CI publish workflow.

### D-02: Backfill helper API shape
- Functions take `&impl SchemaManager` + positional parameters `(table, column, len_or_unit)`. Matches the SeaORM migration file context where users call these helpers.
- Initial set: `backfill_random_hex`, `backfill_random_uuid`, `backfill_current_timestamp`, and a general `backfill` closure-based escape hatch.
- Signature candidate: `backfill_random_hex(manager, "bookings", "checkin_token", 16).await?`

### D-03: Backend coverage
- SQLite + Postgres. MySQL is out of scope — DO App Platform uses Postgres; gestiscilo-it dev environment uses SQLite.

### D-04: PRE_DEPLOY migrate job command
- `{binary_name} db:migrate` — maps to the existing `ferro db:migrate` CLI verb already present in the framework.
- Template token: `{{JOBS_BLOCK}}` in `app.yaml.tpl` (already added to template).

### D-05: `migrate_gate` doctor check severity
- **Error** (hard fail, not Warn). The purpose is to block bad deploys from reaching production traffic. A Warn allows silent bypass.
- Check fails when: repo contains a `migrations/` directory or registers a `Migrator`, AND `.do/app.yaml` exists, AND `.do/app.yaml` has no job with `kind: PRE_DEPLOY` running a migrate command.
- Suggestion text: `"Run \`ferro do:init --force\` to scaffold a migrate job, then commit."`

### D-06: Silent migration runner fix scope
- Fix `framework/src/app.rs`'s `run_migrations_silent` to call `std::process::exit(1)` on failure (not just `eprintln!` + continue).
- Also update `app/src/main.rs` (the sample app) to use the non-silent runner or replace `run_migrations_silent()` with a version that aborts.
- If `ferro new` / `make:project` templates generate `run_migrations_silent`-style code, replace the template with one that aborts.

### Claude's Discretion
- Exact naming of `backfill_*` variants beyond the four listed above.
- Whether `ferro-migration` re-exports from `framework` or is a standalone dep apps add to their migration crate.
- Internal implementation details of the doctor `migrate_gate` check (YAML parser choice, etc.).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above and incident description below.

### Key source files
- `ferro-cli/src/templates/files/do/app.yaml.tpl` — template to extend with `{{JOBS_BLOCK}}`
- `ferro-cli/src/templates/do.rs` — renderer; add `jobs_block` field and rendering logic
- `ferro-cli/src/commands/do_init.rs` — caller; pass `jobs_block` to `AppYamlContext`
- `ferro-cli/src/doctor/registry.rs` — add `MigrateGateCheck` here
- `ferro-cli/src/doctor/checks/` — add `migrate_gate.rs` following the pattern in `generated_artifacts.rs`
- `framework/src/app.rs:386` — `run_migrations_silent` — the silent-failure pattern to fix

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-cli/src/doctor/checks/generated_artifacts.rs` — canonical pattern for a new doctor check (`DoctorCheck` trait, `CheckResult::error`, `CheckResult::warn`, tempfile tests)
- `ferro-cli/src/doctor/checks/docker_template_drift.rs` — `CheckCategory::Deploy` example; `migrate_gate` should also be `Deploy` category
- `ferro-cli/src/doctor/registry.rs` — current 11 checks + 2 deploy-category checks; add `migrate_gate` as the 12th
- `ferro-cli/src/templates/do.rs` — `AppYamlContext` struct + `render_app_yaml`; extend with `jobs_block` field

### Established Patterns
- Doctor checks: struct + `DoctorCheck` impl + `check_impl(root: &Path)` free fn + unit tests with `TempDir`
- Template rendering: `include_str!` template + `{{TOKEN}}` substitution + `debug_assert!` for unresolved tokens
- App yaml rendering: `AppYamlContext` fields resolved by caller (`do_init.rs`), rendered by `render_app_yaml`
- Bootstrap exit on failure: `process::exit(1)` already used on DB connection failure in `app/src/bootstrap.rs`

### Integration Points
- New `ferro-migration` crate: must be added to workspace `Cargo.toml` and CI publish wave 1
- `migrate_gate` check: registered in `doctor/registry.rs::default_checks()` and tested in the `deploy_category_filter` test
- `do:init` jobs block: `AppYamlContext` → `render_app_yaml` → `app.yaml.tpl` chain; `do_init.rs` integration test in `ferro-cli/tests/`

</code_context>

<specifics>
## Specific Ideas

From incident report: The fix in gestiscilo was a single-purpose hot patch — branch on `manager.get_database_backend()` in migration 066, change boot-time runner to `std::process::exit(1)` on failure, hand-add a `jobs:` block to `.do/app.yaml`. Each of those is a workaround that the next ferro consumer will rediscover unless ferro itself closes the gap.

Target state:
- A new gestiscilo migration with a SQLite-only backfill statement would fail at `ferro doctor --deploy` time (pre-push), not at deploy time.
- A consumer scaffolded today via `ferro do:init` gets a working migrate gate without hand-editing `.do/app.yaml`.
- A migration backfill needing random hex is one line, not a backend match.

</specifics>

<deferred>
## Deferred Ideas

- Multi-region / blue-green migration coordination (single PRE_DEPLOY job is sufficient for today's consumer apps)
- Postgres extension management (e.g. auto-creating `pgcrypto`) — use Postgres 13+ built-ins
- Migration squashing / baseline snapshots
- MySQL backend support for backfill helpers

</deferred>

---

*Phase: 157-migration-deploy-safety-backend-portable-backfill-helpers-fe*
*Context gathered: 2026-05-14*
