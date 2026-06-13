# Phase 217: Tenant Context + Per-Tenant API-Key Auth - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 217-tenant-context-per-tenant-api-key-auth
**Mode:** `--auto` (gray areas auto-selected; recommended defaults chosen and logged)
**Areas discussed:** Validator placement, Branch detection, Key hashing/storage, Scope model, McpContext threading, Table ownership, Test surface

---

## Validator placement

| Option | Description | Selected |
|--------|-------------|----------|
| Validator in `ferro-mcp-oauth` (parallel to `validate_bearer`, returns `BearerCheck`) | One auth outcome type; auth crate owns auth | ✓ |
| `resolve_tenant_from_api_key` in `ferro-mcp-server/src/auth.rs` (ARCHITECTURE Phase-1 sketch) | Keeps DB lookup near the server | |

**Auto-selected:** ferro-mcp-oauth.
**Notes:** SC#2 requires the same `BearerCheck::Authenticated` outcome type; `BearerCheck` lives in ferro-mcp-oauth; STACK.md assigns the validator to the auth crate. Resolves the cross-doc discrepancy in favor of a single auth outcome type. ferro-mcp-server keeps a thin `resolve_tenant` unifier.

## Branch detection

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro_` token prefix routes to API-key path; else JWT | Single header, two branches | ✓ |
| Separate header / separate endpoint | Two surfaces | |

**Auto-selected:** Prefix-based branch on one `Authorization: Bearer` header.

## Key hashing & storage

| Option | Description | Selected |
|--------|-------------|----------|
| SHA-256 hex lookup on `key_hash`, plaintext never stored, soft-revoke | `sha2 0.10` already present | ✓ |
| Plaintext compare / reversible storage | Rejected — secret at rest | |

**Auto-selected:** SHA-256 hashed lookup. `subtle` available for constant-time compare if wanted.

## Scope model

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit `read \| read_write` enum on the key record, present from first migration | Matches SC#3; scope on key from day one (PITFALLS §4) | ✓ |
| Reuse `ServiceDef.mcp_ability` only (no key scope) | Conflates credential scope with tenant ability | |
| Fine-grained `abilities[]` array now | Over-scoped for v15.0; deferred to v15.x | |

**Auto-selected:** Coarse `read`/`read_write` on the key. Orthogonal to `mcp_ability`; not a duplicate control surface.

## McpContext threading

| Option | Description | Selected |
|--------|-------------|----------|
| Embed `tenant_id` + `evaluated_guards`; thread tenant_id into existing `dispatch()`; guards populated later | Minimal reshape; dispatch scoping already exists | ✓ |
| Add tenant_id only now, add guards in 218 | Reshapes context twice | |

**Auto-selected:** Embed both fields now; populate `evaluated_guards` in write-tool phases.

## Table ownership

| Option | Description | Selected |
|--------|-------------|----------|
| Framework defines canonical `api_keys` schema; consumer runs the migration | Single source of truth, project-agnostic | ✓ |
| Consumer fully defines its own table; framework only does lookup | Divergent per-consumer schemas | |

**Auto-selected:** Framework-owned schema + generator helper; consumer runs migration. Researcher to confirm v8.1 `make:api-key` schema reuse.

## Test surface

| Option | Description | Selected |
|--------|-------------|----------|
| Extend `ferro-mcp-server/tests/mcp_tenant_isolation.rs` (non-ignored) in the same commit | PITFALLS §1 mandate | ✓ |
| Add a follow-up test later | Rejected — leak risk between commits | |

**Auto-selected:** Same-commit fixture extension.

## Claude's Discretion

- Exact `api_keys` column names/types and scope storage representation (enum vs check-constrained text vs small int).
- Token entropy length / exact `ferro_` prefix format.
- Whether `validate_api_key` takes `expected_tenant: Option<i64>` like `validate_bearer`.

## Deferred Ideas

- Write tools (218), write dispatch + guard re-eval + idempotency + audit (219), confirmation gating (220), NL loop (221).
- Fine-grained per-action `abilities[]` key scope (v15.x).
- DB-backed confirmation store, per-call key-usage audit trail.
