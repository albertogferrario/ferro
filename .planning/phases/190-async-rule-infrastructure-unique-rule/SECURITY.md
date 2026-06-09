# Security Audit — Phase 190: Async Rule Infrastructure / Unique Rule

**Audited:** 2026-06-09
**ASVS Level:** 1
**Threats Closed:** 5/5
**Open Threats:** 0
**block_on:** high

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-190-01 | Tampering (SQL identifier injection) | mitigate | CLOSED | rules_async.rs:78–84 `validate_identifier` rejects any char outside `[A-Za-z0-9_]` and empty strings. Called at lines 120, 122, 125 before any DB access. `quote_ident` at line 87 double-quotes identifiers. Values are bound via `Statement::from_sql_and_values` at line 144, never interpolated. Guard test at lines 346–359 proves `unique("bad;name","slug")` short-circuits before `DB::connection()`. |
| T-190-02 | Tampering/DoS (infra error masked as validation pass) | mitigate | CLOSED | rules_async.rs:133 maps `DB::connection()` error to `__infra_error__:` prefix. Lines 148 and 155 map `query_one` and `try_get` errors to the same prefix. Line 156 emits sentinel on a missing COUNT row (prevents silent pass). async_validator.rs:269–272 intercepts the prefix via `strip_prefix("__infra_error__:")` and returns `AsyncValidationError::Infra(FrameworkError::database(...))`, never inserting the message into the field error map. |
| T-190-03 | Tampering (SQL injection via field value) | mitigate | CLOSED | rules_async.rs:176–195 `json_value_to_sea_value` converts any `serde_json::Value` to `sea_orm::Value`. Line 139 (no-ignore path) and line 140 (ignore path) supply these as the `values` vec to `Statement::from_sql_and_values` at line 144, never string-interpolated. |
| T-190-04 | Information disclosure (infra detail leaked into user-facing field error) | mitigate | CLOSED | async_validator.rs:269–273: sentinel prefix is stripped (`rest.trim()`), wrapped in `FrameworkError::database(...)`, and returned immediately as `AsyncValidationError::Infra`. The raw message is never passed to `errors.add()`. Test `async_validator_infra_error_shape` (lines 514–539) asserts the field error map does not contain any string containing `__infra_error__`. |
| T-190-05 | Elevation / API misuse (consumer treats Infra as Validation) | mitigate | CLOSED | `AsyncValidationError` enum is public (async_validator.rs:32–39) with documented variants. The blanket `From<AsyncValidationError> for ActionError` was deliberately omitted (async_validator.rs:52–58) because a lossy conversion would silently drop field errors and redirect URL. Callers are required to match variants explicitly — the rustdoc usage example (lines 21–31) shows the correct `Infra(fe) => ActionError::from(fe)` branch. lib.rs:317 exports `AsyncValidationError`. Integration test `redirect_back_shape` (async_validation_integration.rs:119–151) proves the `Validation` path terminates in a redirect ActionError, not an Infra/500. |

---

## Unregistered Threat Flags from SUMMARY.md

None. All SUMMARY.md `## Threat Surface Scan` sections either map to registered threat IDs (T-190-01 through T-190-05) or report no new attack surface.

---

## Accepted Risks Log

None. All five threats are mitigated.

---

## Notes

- The `From<AsyncValidationError> for ActionError` impl required by the T-190-05 mitigation plan was superseded by an explicit-match-only design. The security property (Infra never silently treated as Validation) is preserved: the absence of an implicit conversion removes the failure mode rather than routing it. This is a stronger control than the planned one.
- `seed_widget` in inline test fixtures uses string interpolation for integer IDs (not user-supplied values). This is in test code only and represents no production risk.
