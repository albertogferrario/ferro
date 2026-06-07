# Phase 186: ferro-deployments — Immutable Deployments + Atomic Promote - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-07
**Phase:** 186-ferro-deployments-immutable-deployments-atomic-promote
**Mode:** auto (all areas auto-selected, recommended options chosen)
**Areas discussed:** Pointer ownership & promote mechanics, Schema & identifier design, DeploymentStorage trait & ferro-storage coupling, API surface & lifecycle

---

## Pointer ownership & promote mechanics

| Option | Description | Selected |
|--------|-------------|----------|
| Crate-owned pointer table keyed by opaque `owner_key` string | Race test possible inside the crate (success criterion 2 requires it); consumer-agnostic | ✓ |
| Consumer-owned pointer column + crate helper | Matches gestiscilo `frontends.active_deployment_id` literally, but crate can't own the race test | |

**Choice rationale:** the roadmap criterion places the race test in this crate, which forces crate-owned pointer storage. Consumers keep denormalized FKs if they want.

| Option | Description | Selected |
|--------|-------------|----------|
| Last-write-wins single atomic UPDATE | Matches roadmap criterion 2 verbatim; optimistic guards stay consumer-side | ✓ |
| Built-in `promote_if_newer` optimistic check | Duplicates a consumer-side control surface (gestiscilo PITFALLS B-01); deferred | |

| Option | Description | Selected |
|--------|-------------|----------|
| Pointer row carries `deployment_id` + `previous_deployment_id`; one UPDATE flips both | SET expressions read pre-update values on both Postgres and SQLite — true single-statement atomic flip | ✓ |
| Transaction with SELECT-then-UPDATE | Two statements; more lock surface | |

---

## Schema & identifier design

| Option | Description | Selected |
|--------|-------------|----------|
| i64 PK + DNS-safe unique `identifier` string column | Portability per Phase 185 D-05; subdomain-safe label for preview URLs | ✓ |
| String/ULID primary key | Single column, but breaks the workspace i64-PK precedent | |

| Option | Description | Selected |
|--------|-------------|----------|
| `building` / `ready` / `failed`, terminal immutable | Roadmap names these exactly | ✓ |
| Add more states (e.g. `promoted`) | Active-ness lives in the pointer, not deployment status | |

| Option | Description | Selected |
|--------|-------------|----------|
| Include nullable `artifact_deleted_at` from day one | gestiscilo PITFALLS B-03 hard requirement for safe future rollback | ✓ |
| Defer the column | Schema change later for a known requirement | |

---

## DeploymentStorage trait & ferro-storage coupling

| Option | Description | Selected |
|--------|-------------|----------|
| Direct dependency on ferro-storage; default impl wraps `Storage`/`Disk` | Both leaf crates; ferro-deployments lands in publish Wave 1b | ✓ |
| Feature-flagged ferro-storage dependency | Unneeded indirection — S3-compatible default is a phase requirement | |

| Option | Description | Selected |
|--------|-------------|----------|
| Prefix-scoped artifact file ops; `artifact_location` opaque string | Artifact-shape agnostic (criterion 5); minimal trait | ✓ |
| Rich artifact manifest model | Over-modeling; shape belongs to consumers | |

---

## API surface & lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| `Deployments` handle struct wrapping `DatabaseConnection` | Mirrors ferro-queue connection threading; methods create/mark_ready/mark_failed/promote/rollback/active/get/list | ✓ |
| Free functions taking `&DatabaseConnection` | More surface to document; no shared config carrying | |

| Option | Description | Selected |
|--------|-------------|----------|
| `DEPLOYMENT_PREVIEW_DOMAIN` env via `from_env()` config; `preview_url -> Option<String>` | Project-agnostic crates rule (no hardcoded domains); "consumers may leave it unwired" maps to Option | ✓ |
| Derive from APP_URL | Preview domain is a distinct concern from app URL | |

| Option | Description | Selected |
|--------|-------------|----------|
| Doc-test/example storing JSON spec bundle | Criterion 5 verbatim — non-HTML artifact proof | ✓ |
| HTML-only example | Violates criterion 5 | |

---

## Claude's Discretion

- Exact flip SQL per backend (race test on both backends is the gate)
- Identifier generation scheme (DNS-label-safe, unique)
- Exact `DeploymentStorage` method signatures; streaming support
- Pointer table naming/structure
- ferro-mcp deployments tool now vs at framework integration

## Deferred Ideas

- Deployment retention/GC tooling (column ships now, lifecycle later)
- `promote_if_newer` variant (consumer-side until a second consumer needs it)
- ferro-mcp deployments introspection tool
