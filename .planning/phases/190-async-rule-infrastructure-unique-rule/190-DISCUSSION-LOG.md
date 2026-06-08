# Phase 190: Async Rule Infrastructure + `unique` Rule - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-09
**Phase:** 190-async-rule-infrastructure-unique-rule
**Mode:** `--auto` (recommended option auto-selected per area)
**Areas discussed:** AsyncRule mechanism, AsyncValidator composition, unique query + identifier safety, exclude-self id type, scoped uniqueness, default message + lang key, DB-failure semantics

> Note: most of the phase contract is pre-locked by ROADMAP Phase-190 success
> criteria (1–5) and REQUIREMENTS VALID-01..03. Those were treated as decided
> and not re-opened. The areas below are the genuine remaining gray areas.

---

## AsyncRule trait mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| `async-trait` crate | Boxes the future; trait stays dyn-compatible → `Box<dyn AsyncRule>` mirrors `Box<dyn Rule>`. Already a dep. | ✓ |
| Native async fn in trait (AFIT) | No extra dep, but not dyn-compatible without RPITIT workarounds — breaks the boxed-rule storage model. | |
| Manual `Pin<Box<dyn Future>>` | Full control, no macro; more boilerplate per rule. | |

**User's choice:** `async-trait` (auto: recommended).
**Notes:** `async-trait = "0.1"` already present at `framework/Cargo.toml:30`; keeps parity with the existing sync `Box<dyn Rule>` pattern.

---

## AsyncValidator composition

| Option | Description | Selected |
|--------|-------------|----------|
| One validator holds sync + async rules | Single builder; `validate_async` runs sync first (fail-fast), then async on clean fields. Returns existing `ValidationError`. | ✓ |
| Separate async-only validator | Dev runs sync `Validator` then a second async pass manually — more boilerplate, easy to forget. | |

**User's choice:** Combined validator (auto: recommended).
**Notes:** Locked by success criterion 3 ("runs sync rules first, skips async on fields with sync errors"). Reuses `ValidationError` → `with_old_input` flow (criterion: VALID-03).

---

## `unique` query construction + identifier safety

| Option | Description | Selected |
|--------|-------------|----------|
| Parameterized `SELECT COUNT(*)`, quoted identifiers | Value bound as SQL param; table/column quoted per backend + charset-validated. | ✓ |
| Raw string-interpolated SQL | Simplest, but injection risk if identifiers ever came from input. | |
| Typed SeaORM entity query | Type-safe, but ROADMAP locks string `table`/`column` args — no entity available. | |

**User's choice:** Parameterized COUNT with guarded identifiers (auto: recommended).
**Notes:** Identifiers are developer-controlled (not end-user input); guarded by quoting + `[A-Za-z0-9_]` validation; trust boundary documented. Backend via `get_database_backend()`.

---

## Exclude-self id type

| Option | Description | Selected |
|--------|-------------|----------|
| `.ignore(impl Into<sea_orm::Value>)`, pk default `"id"` | Accepts i64/Uuid/String/&str; adds `AND id <> ?`. Non-`id` PK via explicit override. | ✓ |
| `.ignore(i64)` only | Simpler but breaks Uuid/string PKs. | |

**User's choice:** Generic `Into<sea_orm::Value>`, default pk `"id"` (auto: recommended).
**Notes:** VALID-02 mandates exclude-self in v1. Non-default PK spelling left to planner.

---

## Scoped / conditional uniqueness (per-tenant)

| Option | Description | Selected |
|--------|-------------|----------|
| v1 = table+column+ignore only; defer scoping | Matches locked success criteria; per-tenant scope is a flagged fast-follow pending tenancy-model confirmation. | ✓ |
| Add `.where_eq(col, val)` scope now | Useful for per-tenant slug uniqueness, but expands Phase 190 past its locked criteria. | |

**User's choice:** Defer scoping (auto: recommended).
**Notes:** Open question recorded for the researcher: confirm gestiscilo-it tenancy model (separate-DB-per-tenant vs `tenant_id` column). Captured in CONTEXT.md Deferred Ideas.

---

## Default message + ferro-lang key

| Option | Description | Selected |
|--------|-------------|----------|
| `validation.unique` key + English fallback | Mirrors every existing rule's `translate_validation(...).unwrap_or_else(...)`. | ✓ |
| Hardcoded English only | Breaks localization parity with the rest of the validation module. | |

**User's choice:** `validation.unique` key (auto: recommended).
**Notes:** Param `("attribute", field)`, fallback "The {field} has already been taken." Phases-list bullet explicitly requires the translation key.

---

## DB / infrastructure failure semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Infra error propagates as framework error (→500) | DB outage is not a validation result; never silently passes/fails the field. | ✓ |
| Treat query error as validation failure | Wrong — would show a misleading field error on a DB outage. | |
| Treat query error as pass | Wrong — would let a duplicate through when the DB is flaky. | |

**User's choice:** Propagate as framework error (auto: recommended).
**Notes:** Concrete `Result` encoding (validation-vs-infra) left to planner.

## Claude's Discretion

- Precise `AsyncValidator` constructor / `validate_async` signature (parity with sync `new(&data)` recommended; ROADMAP `validate_async(&req)` snippet treated as illustrative).
- Exact non-default-PK exclude-self API spelling.
- Concrete `Result` type for the validation-vs-infra distinction.
- File split within `framework/src/validation/`.

## Deferred Ideas

- Scoped/per-tenant uniqueness (`.where_eq`) — pending tenancy-model confirmation.
- Additional async rules (`exists`, `custom_async`) — only if free from the `AsyncRule` trait.
