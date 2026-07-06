# Phase 233: ferro-payments crate scaffold + PaymentIntent entity + migration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-17
**Phase:** 233-ferro-payments-crate-polymorphic-billable
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** Partial-unique index, Status/enum representation, Timestamp portability, Lifecycle preconditions, JSON metadata, Crate manifest/dependency surface

---

## Partial-unique index (cross-backend)

| Option | Description | Selected |
|--------|-------------|----------|
| Backend-dispatched raw SQL (PG+SQLite native partial index, MySQL generated-column emulation) | Branch on `get_database_backend()`; true partial index on PG/SQLite, NULLable generated column + plain UNIQUE on MySQL | ✓ |
| Drop MySQL from the partial-unique guarantee | Document the active-row guarantee as PG+SQLite only | |
| Generated/virtual column on all three backends | Uniform emulation everywhere, plain UNIQUE on a conditionally-NULL column | |

**Selected:** Backend-dispatched raw SQL (recommended).
**Notes:** SQLite is the CI test target and supports partial indexes natively (≥3.8.0);
PG uses identical syntax. MySQL lacks partial indexes → generated-column emulation, to be
validated by research (MySQL version, NULL-uniqueness). No SeaORM-native API and no
workspace precedent — emitted via raw SQL.

## Status / enum representation

| Option | Description | Selected |
|--------|-------------|----------|
| TEXT column + DeriveActiveEnum string mapping | Portable across all backends; `string_value` per variant | ✓ |
| Native DB ENUM type | PG ENUM / MySQL ENUM — not portable to SQLite | |

**Selected:** TEXT + DeriveActiveEnum (recommended). `billable_kind` stays raw TEXT (open set).

## Timestamp portability

| Option | Description | Selected |
|--------|-------------|----------|
| `timestamp_with_time_zone` + chrono DateTime<Utc>, stamps set in Rust | No non-portable DB defaults; SeaORM maps per backend | ✓ |
| DB-level `DEFAULT now()` | Non-portable default syntax across backends | |

**Selected:** SeaORM `timestamp_with_time_zone`, Rust-set NOT-NULL stamps (recommended).

## Lifecycle preconditions

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro_orm::GuardedUpdate` atomic conditional UPDATE | Status in WHERE; 0 rows = no-op = race-safe by construction | ✓ |
| Read-then-write inside a transaction | Read current status, validate, update — wider race window | |
| DB CHECK constraint | Not portable; cannot express transition logic | |

**Selected:** GuardedUpdate (recommended) — reuses existing primitive, matches design's
"second writer no-ops" race semantics. `create_reserved` = plain INSERT guarded by the
partial unique index.

## JSON metadata

| Option | Description | Selected |
|--------|-------------|----------|
| SeaORM `ColumnType::Json` + serde_json::Value, nullable | Maps JSONB/JSON/TEXT per backend automatically | ✓ |
| Plain TEXT column + manual (de)serialization | Manual handling, loses backend JSON features | |

**Selected:** `ColumnType::Json` (recommended). No PII.

## Crate manifest / dependency surface

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal deps; defer ferro-stripe + full PaymentError to 234 | 233 deps: sea-orm/chrono/serde/serde_json/thiserror/async-trait/ferro-orm; minimal error enum | ✓ |
| Add ferro-stripe now per design doc Cargo comment | Pulls in unused dep → clippy -D warnings failure in 233 | |

**Selected:** Minimal deps (recommended). `ferro-stripe` and the `Stripe`/`Loader`/
`AutoRefundTriggered` error variants land in phase 234. Version `0.1.0`; add to workspace
members + publish.yml wave after `ferro-orm`.

## Claude's Discretion

- Module file split inside `src/` (recommend shipping only `lib.rs`, `intent/`,
  `migration/`, `error.rs` in 233; stub or omit service/webhook/reaper/loader/billable).
- Exact MySQL generated-column name + SQL expression (pending D-02 research).
- Whether `BillableKind` lives in a `billable.rs` stub or alongside the entity.

## Deferred Ideas

PaymentService, Billable/BillableLoader traits (234); wire_dispatcher + webhook handlers
(235); reapers + workspace test bin + publish (236); ferro-stripe dependency wiring +
full PaymentError variants (234); design open questions 1-4 (234+).
