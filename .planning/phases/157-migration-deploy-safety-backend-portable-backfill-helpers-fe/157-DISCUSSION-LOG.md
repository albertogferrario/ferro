# Phase 157 — Migration Deploy Safety — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 157-migration-deploy-safety-backend-portable-backfill-helpers-fe
**Mode:** --auto (all selections are recommended defaults; no user interaction)
**Areas discussed:** migration-helper-crate, backfill-api-shape, pre-deploy-command, migrate-gate-severity, silent-runner-fix

---

## Migration Helper Crate Location

| Option | Description | Selected |
|--------|-------------|----------|
| New `ferro-migration` crate | First-class crate, public API `ferro_migration::backfill_*`, added to CI publish wave 1 | ✓ |
| Inline in `framework` | Add as a module inside the main framework crate | |

**Auto-selected:** New `ferro-migration` crate (recommended default — CONTEXT.md already names the API as `ferro_migration::backfill_random_hex`)

---

## Backfill Helper API Shape

| Option | Description | Selected |
|--------|-------------|----------|
| `SchemaManager` + positional params | `backfill_random_hex(manager, "table", "col", 16)` — matches SeaORM migration file context | ✓ |
| `DatabaseConnection` direct | Lower-level; bypasses the migration manager abstraction | |

**Auto-selected:** `SchemaManager` + positional params (recommended default — aligns with how migration files already use the manager)

---

## PRE_DEPLOY Job Command

| Option | Description | Selected |
|--------|-------------|----------|
| `{binary_name} db:migrate` | Maps to existing `ferro db:migrate` CLI verb | ✓ |
| `{binary_name} migrate` | Shorter but doesn't match the established verb | |

**Auto-selected:** `{binary_name} db:migrate` (recommended default — consistent with existing CLI)

---

## `migrate_gate` Doctor Check Severity

| Option | Description | Selected |
|--------|-------------|----------|
| Error (hard fail) | Blocks `ferro doctor --deploy`; forces fix before push | ✓ |
| Warn | Surfaced but not blocking | |

**Auto-selected:** Error (recommended default — a Warn allows silent bypass; the whole point is to block bad deploys)

---

## Silent Migration Runner Fix Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Fix framework + app template | Fix `framework/src/app.rs:run_migrations_silent` to exit(1); update sample app | ✓ |
| Docs only | Document anti-pattern without changing framework code | |

**Auto-selected:** Fix framework + app template (recommended default — the pattern lives in framework code, not just user code)

---

## Claude's Discretion

- Exact naming of `backfill_*` variants beyond the four listed
- Whether `ferro-migration` re-exports from `framework` or is standalone
- Internal implementation details of `migrate_gate` check (YAML parser choice, etc.)

## Deferred Ideas

- Multi-region / blue-green migration coordination
- Postgres extension management (auto-creating `pgcrypto`)
- Migration squashing / baseline snapshots
- MySQL backend support for backfill helpers
