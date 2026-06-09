---
phase: 190
slug: async-rule-infrastructure-unique-rule
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-09
---

# Phase 190 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| handler code → `AsyncRule` / `unique(table, col)` | Rule definitions and identifiers are developer-controlled (trusted source), guarded defense-in-depth. | Table/column/pk identifiers |
| field value → COUNT query | End-user-supplied value crosses into SQL. | Untrusted field value (bound as parameter) |
| DB → rule result | A DB/infra failure must not be reported as a validation pass or a field error. | Query result / DbErr |
| async rule result → validator | A rule's `Err(String)` may be a field message OR an infra sentinel; the validator must classify it correctly. | Validation message vs `__infra_error__:` sentinel |
| consumer code → public async validation API | The newly public surface is the contract consumers build against; must expose only the safe path. | `unique`, `AsyncValidationError`, `validate_async` |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-190-01 | Tampering (SQL identifier injection) | `Unique` table/col/pk_col args → COUNT SQL | mitigate | `validate_identifier` (`rules_async.rs:78–84`) rejects chars outside `[A-Za-z0-9_]`; called at lines 120/122/125 before any DB access; `quote_ident` double-quotes; guard test (lines 346–359) proves short-circuit before `DB::connection()`. | closed |
| T-190-02 | Tampering/DoS (infra error masked as pass) | `Unique::validate` DbErr handling + `validate_async` loop | mitigate | `DB::connection()`/`query_one`/`try_get`/missing-row errors emit `__infra_error__:` sentinel (`rules_async.rs:133,148,155,156`); `async_validator.rs:269–272` intercepts via `strip_prefix` and returns `AsyncValidationError::Infra` (→ 500), never inserted into the field error map. | closed |
| T-190-03 | Tampering (SQL injection via field value) | COUNT query value binding | mitigate | `json_value_to_sea_value` (`rules_async.rs:176–195`) converts the checked value to `sea_orm::Value`, bound via `Statement::from_sql_and_values` (line 144). No interpolation path. | closed |
| T-190-04 | Information disclosure (infra detail in field error) | sentinel handling | mitigate | `async_validator.rs:269–273` strips the prefix, wraps in `FrameworkError::database`, returns as `Infra`; never reaches `errors.add()`. `async_validator_infra_error_shape` test (lines 514–539) asserts no `__infra_error__` string in field errors. | closed |
| T-190-05 | Elevation / API misuse (Infra treated as Validation) | public `AsyncValidationError` | mitigate | Public enum with documented variants (`async_validator.rs:32–39`). Blanket `From` impl deliberately omitted (`async_validator.rs:52–58`) — forces explicit variant matching, a stronger control than implicit coercion. `redirect_back_shape` integration test (`async_validation_integration.rs:119–151`) proves the `Validation` path yields a redirect, not a 500. Exported at `lib.rs:317`. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-09 | 5 | 5 | 0 | gsd-security-auditor (sonnet) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-09
