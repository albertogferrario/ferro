//! Channel-agnostic transition-execution kernel.
//!
//! Defines [`WriteDispatcher`], [`ExecutorFn`], [`GuardEvaluatorFn`], and
//! [`OverrideFn`] — the app-registered callback set that makes write actions
//! executable — and [`dispatch_write`], the single execution pipeline
//! (guard re-eval → idempotency → confirm seam → persist → audit → override).
//!
//! This kernel was relocated out of `ferro-mcp-server` so more than one channel
//! (MCP, visual/form, …) can drive the same guarded write path. It owns its own
//! [`WriteError`] (it does NOT depend on any channel's error type); each channel
//! maps `WriteError` into its own error at the framing boundary. The audit reason
//! prefix is parameterized by a `channel` argument — the kernel never hardcodes a
//! channel name.

use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_projections::{ActionDef, CrudPlan};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

// ── Kernel-local error ───────────────────────────────────────────────────────

/// Errors returned by the transition-execution kernel.
///
/// Self-contained: the kernel does not depend on any channel's error type. Each
/// channel that drives [`dispatch_write`] maps these variants into its own error
/// at the framing boundary (e.g. `From<WriteError> for ferro_mcp_server::Error`).
#[derive(Error, Debug)]
pub enum WriteError {
    /// A database operation failed. May contain SQL fragments / table or column
    /// names — channels MUST redact this before returning it to an untrusted caller.
    #[error("database error: {0}")]
    Database(String),
    /// JSON (de)serialization of an idempotency result or persist payload failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// A precondition or transition guard returned false or errored at execution
    /// time. Never discloses which guard or what state it checked.
    #[error("guard failed: {0}")]
    GuardFailed(String),
    /// Input validation failed (required field missing, wrong type, etc.).
    #[error("validation error: {0}")]
    Validation(String),
    /// The resolved action name is not found in any exposed `ServiceDef`.
    #[error("action not found: {0}")]
    ActionNotFound(String),
    /// A destructive action was called without a valid confirmation context.
    /// Feature-gated: only reachable when the `confirmation` feature is enabled.
    #[cfg(feature = "confirmation")]
    #[error("confirmation required for action: {0}")]
    ConfirmationRequired(String),
    /// A CRUD verb (create/update/delete) was called on a service that has not
    /// opted into that verb. Should not normally occur if the framing layer only
    /// emits tools for enabled verbs, but checked defensively.
    #[error("crud verb not enabled: {0}")]
    CrudVerbNotEnabled(String),
    /// A CRUD update or soft-delete targeted a row that does not exist or has
    /// already been soft-deleted (`deleted_at IS NOT NULL`).
    #[error("record not found or already deleted")]
    RecordNotFound,
}

/// Result alias for the transition-execution kernel.
pub type WriteResult<T> = Result<T, WriteError>;

// ── Boxed-future callback types (no async-trait dep) ─────────────────────────

/// App-registered write executor.
///
/// Called by [`dispatch_write`] after guards pass and idempotency is checked.
/// Receives the action name, validated inputs, the authenticated tenant id
/// (never from the call payload), and a DB connection.
/// Returns a JSON `Value` that becomes the `result` payload in the structured
/// channel response.
///
/// # Audit contract
///
/// The returned `Value` is stored **verbatim** in the append-only `audit_log`
/// table via [`ferro_audit::AuditEntry`]. It MUST NOT contain secrets,
/// credentials, PII, or any field that should not appear in a forensic log
/// readable by audit reviewers. Executors are responsible for returning only
/// audit-safe fields (typically identifiers and status values, e.g.
/// `{"id": 42, "status": "approved"}`). A full PII scrub at the
/// [`dispatch_write`] call site is not performed; the executor is the enforcement
/// point.
pub type ExecutorFn = Box<
    dyn Fn(
            &str,   // action_name
            &Value, // validated inputs
            i64,    // tenant_id (from auth, never from payload)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = WriteResult<Value>> + Send>>
        + Send
        + Sync,
>;

/// App-registered guard evaluator.
///
/// Called once per precondition name in `action.preconditions` BEFORE the
/// executor runs. Must read LIVE DB state — never `ctx.evaluated_guards`.
/// Returns `Ok(true)` to allow, `Ok(false)` or `Err(_)` to deny.
pub type GuardEvaluatorFn = Box<
    dyn Fn(
            &str,   // guard_name
            i64,    // tenant_id
            &Value, // validated inputs (for record-scoped guards)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = WriteResult<bool>> + Send>>
        + Send
        + Sync,
>;

/// App-registered post-persist override hook (EXEC-03).
///
/// Runs AFTER the derived/base persist, inside [`dispatch_write`]. Receives the
/// action name, validated inputs, the authenticated tenant id, a DB connection,
/// and the base persist result — so it can chain related-record writes keyed off
/// the just-persisted state.
///
/// Because it runs strictly after the guarded base persist, an override cannot
/// suppress the base guard or transition (threat T-231-05, mitigated).
///
/// # Ordering vs. audit/idempotency (WR-01)
///
/// The override runs AFTER the base persist's idempotency key and audit entry
/// are sealed (steps 5–6 in [`dispatch_write`]). There is no surrounding
/// transaction, so an override that returns `Err` does NOT roll back the base
/// persist — and the base persist's audit entry and idempotency key remain
/// committed. An override author must therefore treat the base transition as
/// already durable when the hook fires: a failing override surfaces its error to
/// the caller, but the base write stays applied (and audited).
pub type OverrideFn = Box<
    dyn Fn(
            &str,   // action_name
            &Value, // validated inputs
            i64,    // tenant_id (from auth, never from payload)
            &DatabaseConnection,
            &Value, // base persist result
        ) -> Pin<Box<dyn Future<Output = WriteResult<()>> + Send>>
        + Send
        + Sync,
>;

/// Holds the app-registered write callback, guard evaluator, and optional
/// post-persist override hooks.
///
/// Constructed by the consumer app and passed to the channel framing layer,
/// which threads it into [`dispatch_write`].
pub struct WriteDispatcher {
    /// The write executor (base persist).
    pub executor: ExecutorFn,
    /// The live-state guard evaluator.
    pub guard_evaluator: GuardEvaluatorFn,
    /// Optional app-specific post-persist side effects, keyed by action name.
    /// An absent key is the common path (declaration-only) — the override seam
    /// adds nothing to a write whose action has no registered hook.
    pub overrides: std::collections::HashMap<String, OverrideFn>,
}

impl WriteDispatcher {
    /// Construct a dispatcher with an empty override registry.
    pub fn new(executor: ExecutorFn, guard_evaluator: GuardEvaluatorFn) -> Self {
        Self {
            executor,
            guard_evaluator,
            overrides: std::collections::HashMap::new(),
        }
    }

    /// Register a post-persist override hook for `action` (consuming builder).
    pub fn with_override(mut self, action: impl Into<String>, hook: OverrideFn) -> Self {
        self.overrides.insert(action.into(), hook);
        self
    }
}

// ── SQL helpers (CRUD executor) ───────────────────────────────────────────────

/// Build a backend-specific placeholder for parameterized SQL.
///
/// SQLite uses `?`; Postgres uses `$N` (1-based index).
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}

/// Coerce a `serde_json::Value` to the closest `sea_orm::Value` for binding.
///
/// Null → `String(None)`, Bool → `Bool`, integer Number → `BigInt`,
/// float Number → `Double`, String → `String(Box)`, anything else → JSON
/// string representation via `to_string()`.
fn json_to_sea_value(val: &serde_json::Value) -> sea_orm::Value {
    match val {
        serde_json::Value::Null => sea_orm::Value::String(None),
        serde_json::Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                sea_orm::Value::BigInt(Some(i))
            } else {
                sea_orm::Value::Double(n.as_f64())
            }
        }
        serde_json::Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        other => sea_orm::Value::String(Some(Box::new(other.to_string()))),
    }
}

/// Convert a single `sea_orm::QueryResult` row to a `serde_json::Value` object.
///
/// Attempts column extraction as `i64`, then `f64`, then `bool`, then `String`,
/// in that order, falling back to `null` for columns that can't be decoded.
fn row_to_json(row: &sea_orm::QueryResult) -> serde_json::Value {
    let columns: Vec<String> = row.column_names().iter().map(|s| s.to_string()).collect();
    let mut obj = serde_json::Map::new();
    for col in &columns {
        let val = row
            .try_get_by::<i64, _>(col.as_str())
            .map(|v| serde_json::Value::Number(v.into()))
            .or_else(|_| {
                row.try_get_by::<f64, _>(col.as_str()).map(|v| {
                    serde_json::Number::from_f64(v)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                })
            })
            .or_else(|_| {
                row.try_get_by::<bool, _>(col.as_str())
                    .map(serde_json::Value::Bool)
            })
            .or_else(|_| {
                row.try_get_by::<String, _>(col.as_str())
                    .map(serde_json::Value::String)
            })
            .unwrap_or(serde_json::Value::Null);
        obj.insert(col.clone(), val);
    }
    serde_json::Value::Object(obj)
}

// ── Generic CRUD executor ─────────────────────────────────────────────────────

/// Framework-provided generic CRUD executor.
///
/// Interprets a [`CrudPlan`] into parameterized SQL and executes it against
/// `db`. All VALUES are bound via `sea_orm::Value` through
/// `Statement::from_sql_and_values` (never string-interpolated). Table and
/// column identifiers come exclusively from the `CrudPlan`, which is derived
/// from a developer-authored `ServiceDef` projection — never from untrusted
/// agent input.
///
/// # SQL shapes
///
/// - `Create` → `INSERT INTO <table> (<cols>, created_at[, tenant_col]) VALUES (<ph>, datetime('now')/NOW()[, ?])`
///   followed by `SELECT * FROM <table> WHERE id = last_insert_rowid()` (SQLite) or
///   `INSERT … RETURNING *` (Postgres). Returns the inserted record including its `id`.
/// - `Update` → `UPDATE <table> SET <col>=? … WHERE <id_column>=? AND <soft_delete_column> IS NULL[AND tenant_col=?]`.
///   Zero rows affected → [`WriteError::RecordNotFound`].
/// - `Delete` → `UPDATE <table> SET <soft_delete_column>=<now> WHERE <id_column>=? AND <soft_delete_column> IS NULL[AND tenant_col=?]`.
///   Zero rows affected → [`WriteError::RecordNotFound`]. Returns `{"id": <id_value>, "deleted": true}`.
///
/// # Security
///
/// - T-241-05: all values are bound parameters — no string interpolation of values.
/// - T-241-06: Update and Delete predicates include `AND <soft_delete_column> IS NULL`
///   so a soft-deleted row is unaddressable.
/// - T-242-02: When `tenant_column` is `Some`, `tenant_id` is injected into INSERT (Create)
///   or appended to the WHERE predicate (Update/Delete) as a bound parameter. A foreign-tenant
///   row yields 0 affected rows → existing `WriteError::RecordNotFound` path (D-08 non-disclosure).
/// - T-242-03: The post-update SELECT also carries the tenant predicate to prevent a concurrent
///   cross-tenant reassignment race from returning foreign data (Pitfall 5).
async fn execute_crud_plan(
    plan: &CrudPlan,
    tenant_id: i64,
    db: &DatabaseConnection,
) -> WriteResult<Value> {
    let backend = db.get_database_backend();

    // Backend-specific "now" expression injected as a SQL literal (not a bound
    // parameter) for created_at/deleted_at server timestamps.
    let now_expr = match backend {
        DatabaseBackend::Postgres => "NOW()",
        _ => "datetime('now')",
    };

    match plan {
        CrudPlan::Create {
            table,
            columns,
            tenant_column,
        } => {
            // Build column list and placeholder list.
            // `created_at` is NOT in the plan (Plan 01 contract) — inject it here
            // as a SQL literal expression so the server sets it, not the agent.
            let mut col_names: Vec<String> = columns.iter().map(|(c, _)| c.clone()).collect();
            col_names.push("created_at".to_string());
            // Phase 242: inject tenant column after created_at.
            if let Some(ref tc) = tenant_column {
                col_names.push(tc.column.clone());
            }

            let mut ph_parts: Vec<String> = (1..=columns.len())
                .map(|i| placeholder(backend, i))
                .collect();
            ph_parts.push(now_expr.to_string()); // literal, not a bound param
                                                 // Phase 242: tenant placeholder index = columns.len() + 1.
                                                 // created_at is a SQL literal (now_expr) and does NOT consume a placeholder
                                                 // slot, so the tenant index is columns.len() + 1, NOT + 2.
            if tenant_column.is_some() {
                ph_parts.push(placeholder(backend, columns.len() + 1));
            }

            let col_list = col_names.join(", ");
            let ph_list = ph_parts.join(", ");

            let mut values: Vec<sea_orm::Value> =
                columns.iter().map(|(_, v)| json_to_sea_value(v)).collect();
            // Phase 242: tenant_id comes last in bound values (after all column values).
            if tenant_column.is_some() {
                values.push(sea_orm::Value::BigInt(Some(tenant_id)));
            }

            match backend {
                DatabaseBackend::Postgres => {
                    // Postgres: INSERT … RETURNING * in a single round-trip.
                    let sql =
                        format!("INSERT INTO {table} ({col_list}) VALUES ({ph_list}) RETURNING *");
                    let stmt = Statement::from_sql_and_values(backend, &sql, values);
                    let row = db
                        .query_one(stmt)
                        .await
                        .map_err(|e| WriteError::Database(e.to_string()))?
                        .ok_or_else(|| {
                            WriteError::Database("INSERT RETURNING returned no row".to_string())
                        })?;
                    Ok(row_to_json(&row))
                }
                _ => {
                    // SQLite: INSERT then SELECT last_insert_rowid() then SELECT *.
                    let sql = format!("INSERT INTO {table} ({col_list}) VALUES ({ph_list})");
                    let stmt = Statement::from_sql_and_values(backend, &sql, values);
                    db.execute(stmt)
                        .await
                        .map_err(|e| WriteError::Database(e.to_string()))?;

                    // Retrieve the auto-generated id.
                    let id_row = db
                        .query_one(Statement::from_string(
                            backend,
                            "SELECT last_insert_rowid() AS id".to_string(),
                        ))
                        .await
                        .map_err(|e| WriteError::Database(e.to_string()))?
                        .ok_or_else(|| {
                            WriteError::Database("last_insert_rowid() returned no row".to_string())
                        })?;
                    let inserted_id: i64 = id_row
                        .try_get("", "id")
                        .map_err(|e| WriteError::Database(e.to_string()))?;

                    // Fetch the full inserted record.
                    let select_sql = format!("SELECT * FROM {table} WHERE id = ?");
                    let select_stmt = Statement::from_sql_and_values(
                        backend,
                        &select_sql,
                        vec![sea_orm::Value::BigInt(Some(inserted_id))],
                    );
                    let record_row = db
                        .query_one(select_stmt)
                        .await
                        .map_err(|e| WriteError::Database(e.to_string()))?
                        .ok_or_else(|| {
                            WriteError::Database("SELECT after INSERT returned no row".to_string())
                        })?;
                    Ok(row_to_json(&record_row))
                }
            }
        }

        CrudPlan::Update {
            table,
            id_column,
            id_value,
            patch,
            soft_delete_column,
            tenant_column,
        } => {
            if patch.is_empty() {
                return Err(WriteError::Validation(
                    "patch must contain at least one field".into(),
                ));
            }
            // UPDATE <table> SET col1=?, col2=? … WHERE <id_column>=? AND <soft_delete_column> IS NULL
            // Phase 242: append AND <tenant_column>=? when tenant_column is Some.
            let set_clauses: Vec<String> = patch
                .iter()
                .enumerate()
                .map(|(i, (col, _))| format!("{col} = {}", placeholder(backend, i + 1)))
                .collect();
            let set_sql = set_clauses.join(", ");
            let id_ph = placeholder(backend, patch.len() + 1);
            let sql = if let Some(ref tc) = tenant_column {
                // tenant placeholder index = patch.len() + 2 (id is patch.len() + 1)
                let tenant_ph = placeholder(backend, patch.len() + 2);
                format!(
                    "UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} \
                     AND {soft_delete_column} IS NULL AND {tc_col} = {tenant_ph}",
                    tc_col = tc.column
                )
            } else {
                format!(
                    "UPDATE {table} SET {set_sql} WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL"
                )
            };

            let mut values: Vec<sea_orm::Value> =
                patch.iter().map(|(_, v)| json_to_sea_value(v)).collect();
            values.push(json_to_sea_value(id_value));
            // Phase 242: tenant_id after id_value in the bound values vec.
            if tenant_column.is_some() {
                values.push(sea_orm::Value::BigInt(Some(tenant_id)));
            }

            let stmt = Statement::from_sql_and_values(backend, &sql, values);
            let exec_result = db
                .execute(stmt)
                .await
                .map_err(|e| WriteError::Database(e.to_string()))?;

            if exec_result.rows_affected() == 0 {
                return Err(WriteError::RecordNotFound);
            }

            // Fetch the updated record for the return value.
            // Phase 242 (Pitfall 5): also add the tenant predicate to the post-update
            // SELECT — prevents a concurrent cross-tenant reassignment race from returning
            // foreign data between UPDATE and SELECT. The tenant_id bound below matches
            // the UPDATE that just succeeded.
            let id_ph2 = placeholder(backend, 1);
            let (select_sql, select_values) = if let Some(ref tc) = tenant_column {
                let t_ph2 = placeholder(backend, 2);
                let sql = format!(
                    "SELECT * FROM {table} WHERE {id_column} = {id_ph2} \
                     AND {soft_delete_column} IS NULL AND {tc_col} = {t_ph2}",
                    tc_col = tc.column
                );
                let vals = vec![
                    json_to_sea_value(id_value),
                    sea_orm::Value::BigInt(Some(tenant_id)),
                ];
                (sql, vals)
            } else {
                let sql = format!(
                    "SELECT * FROM {table} WHERE {id_column} = {id_ph2} AND {soft_delete_column} IS NULL"
                );
                (sql, vec![json_to_sea_value(id_value)])
            };
            let select_stmt = Statement::from_sql_and_values(backend, &select_sql, select_values);
            let row = db
                .query_one(select_stmt)
                .await
                .map_err(|e| WriteError::Database(e.to_string()))?
                .ok_or_else(|| {
                    WriteError::Database("SELECT after UPDATE returned no row".to_string())
                })?;
            Ok(row_to_json(&row))
        }

        CrudPlan::Delete {
            table,
            id_column,
            id_value,
            soft_delete_column,
            tenant_column,
        } => {
            // Soft-delete: UPDATE <table> SET <soft_delete_column>=now WHERE <id_column>=?
            //              AND <soft_delete_column> IS NULL
            // Phase 242: append AND <tenant_column>=? when tenant_column is Some.
            // Soft-delete only — no physical row removal (CRUD-03).
            let id_ph = placeholder(backend, 1);
            let sql = if let Some(ref tc) = tenant_column {
                // tenant placeholder index = 2 (id is 1)
                let tenant_ph = placeholder(backend, 2);
                format!(
                    "UPDATE {table} SET {soft_delete_column} = {now_expr} \
                     WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL \
                     AND {tc_col} = {tenant_ph}",
                    tc_col = tc.column
                )
            } else {
                format!(
                    "UPDATE {table} SET {soft_delete_column} = {now_expr} \
                     WHERE {id_column} = {id_ph} AND {soft_delete_column} IS NULL"
                )
            };
            let mut stmt_values = vec![json_to_sea_value(id_value)];
            // Phase 242: tenant_id after id_value in the bound values vec.
            if tenant_column.is_some() {
                stmt_values.push(sea_orm::Value::BigInt(Some(tenant_id)));
            }
            let stmt = Statement::from_sql_and_values(backend, &sql, stmt_values);
            let exec_result = db
                .execute(stmt)
                .await
                .map_err(|e| WriteError::Database(e.to_string()))?;

            if exec_result.rows_affected() == 0 {
                return Err(WriteError::RecordNotFound);
            }

            Ok(serde_json::json!({ "id": id_value, "deleted": true }))
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build the live guard set for a write: `preconditions` followed by the
/// transition-level guard (if any), deduplicated by name with order preserved.
///
/// A guard appearing on BOTH the action precondition and the transition guard
/// — as `is_manager` does on the order projection — is evaluated exactly once
/// (EXEC-02). The common case (no transition guard) returns `preconditions`
/// unchanged, keeping the non-transition path back-compatible.
///
/// Exposed so channel framing (e.g. the MCP confirm pre-check) evaluates the
/// SAME guard union as [`dispatch_write`].
pub fn merged_guards(preconditions: &[String], transition_guard: Option<&str>) -> Vec<String> {
    let mut guards: Vec<String> = preconditions.to_vec();
    if let Some(g) = transition_guard {
        if !guards.iter().any(|existing| existing == g) {
            guards.push(g.to_string());
        }
    }
    guards
}

/// Look up a stored idempotency result scoped by BOTH tenant_id AND idempotency_key.
///
/// Cross-tenant replay is prevented at the SQL level: the WHERE clause requires
/// BOTH columns, matching the UNIQUE index on `(tenant_id, idempotency_key)`.
async fn lookup_idempotency(
    tenant_id: i64,
    key: &str,
    db: &DatabaseConnection,
) -> WriteResult<Option<Value>> {
    let backend = db.get_database_backend();
    let (sql, values) = match backend {
        DatabaseBackend::Postgres => (
            "SELECT result FROM mcp_idempotency_keys WHERE tenant_id = $1 AND idempotency_key = $2"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
            ],
        ),
        _ => (
            "SELECT result FROM mcp_idempotency_keys WHERE tenant_id = ? AND idempotency_key = ?"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
            ],
        ),
    };
    let stmt = Statement::from_sql_and_values(backend, &sql, values);
    match db
        .query_one(stmt)
        .await
        .map_err(|e| WriteError::Database(e.to_string()))?
    {
        None => Ok(None),
        Some(row) => {
            let json_text: String = row
                .try_get("", "result")
                .map_err(|e| WriteError::Database(e.to_string()))?;
            let value: Value = serde_json::from_str(&json_text)
                .map_err(|e| WriteError::Database(e.to_string()))?;
            Ok(Some(value))
        }
    }
}

/// Store an idempotency result scoped by (tenant_id, idempotency_key).
///
/// Uses INSERT OR IGNORE (SQLite) / ON CONFLICT DO NOTHING (Postgres) for
/// concurrency safety — a second concurrent identical request will not cause
/// a UNIQUE constraint error.
async fn store_idempotency(
    tenant_id: i64,
    key: &str,
    result: &Value,
    db: &DatabaseConnection,
) -> WriteResult<()> {
    let backend = db.get_database_backend();
    let json_text = serde_json::to_string(result).map_err(WriteError::Serialization)?;

    let (sql, values) = match backend {
        DatabaseBackend::Postgres => (
            "INSERT INTO mcp_idempotency_keys (tenant_id, idempotency_key, result, created_at) \
             VALUES ($1, $2, $3, NOW()) ON CONFLICT (tenant_id, idempotency_key) DO NOTHING"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
                sea_orm::Value::String(Some(Box::new(json_text))),
            ],
        ),
        _ => (
            "INSERT OR IGNORE INTO mcp_idempotency_keys \
             (tenant_id, idempotency_key, result) VALUES (?, ?, ?)"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
                sea_orm::Value::String(Some(Box::new(json_text))),
            ],
        ),
    };
    let stmt = Statement::from_sql_and_values(backend, &sql, values);
    db.execute(stmt)
        .await
        .map_err(|e| WriteError::Database(e.to_string()))?;
    Ok(())
}

// ── Core pipeline ─────────────────────────────────────────────────────────────

/// Execute a write action with guard re-evaluation, idempotency, and audit.
///
/// Pipeline order (D-07):
/// 1. Guard re-evaluation (D-02) — LIVE state via `dispatcher.guard_evaluator`.
///    CRITICAL: `ctx.evaluated_guards` is the 218 list-time visibility cache and
///    is NEVER consulted here. Authorization at call time must use live DB state.
/// 2. Idempotency check (D-04) — replay stored result without re-executing.
/// 3. D-08 confirmation seam — gate for destructive actions (Phase 220).
/// 4. Execute callback (D-01).
/// 5. Store idempotency result (D-04).
/// 6. Audit via ferro-audit (D-05).
/// 7. Post-persist override hook (EXEC-03).
///
/// The `channel` argument is the audit reason prefix: the success-path audit
/// records `format!("{channel}.action.{name}")`. Each channel passes its own
/// literal (e.g. MCP passes `"mcp"`, the visual surface passes `"web"`), so a
/// write is never auditable without a channel tag.
///
/// WR-01: steps 5–6 (idempotency-store + audit of the base persist) run BEFORE
/// the override hook (step 7). There is no surrounding transaction, so the base
/// persist (step 4) has already committed by the time the override runs; sealing
/// its idempotency key and audit entry first guarantees a committed base
/// transition is always recorded and never re-executable, even if the
/// app-specific override side effect later fails.
///
/// The `is_confirmed` parameter (confirmation feature only) signals that this
/// call came from the channel's confirm handler after token validation — the
/// D-08 seam is bypassed when `true`. Bare callers always pass `false` (or omit
/// when the feature is off).
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    channel: &str,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
    crud_plan: Option<&CrudPlan>,
) -> WriteResult<Value> {
    // 1. Guard re-evaluation (D-02, T-219-02 — load-bearing security gate).
    //
    // Calls the app-registered GuardEvaluatorFn for EVERY guard in the UNION of
    // action.preconditions and the transition-level guard (EXEC-02), deduped by
    // name, against LIVE DB state. Fail-closed: a guard returning Ok(false) OR
    // any Err immediately returns Err(GuardFailed).
    //
    // IMPORTANT: ctx.evaluated_guards (the 218 list-time visibility cache) is
    // intentionally NOT consulted here. An agent may bypass tools/list entirely
    // and call tools/call directly — only this live re-evaluation prevents the
    // guard-bypass privilege-escalation class (PITFALLS §2 / T-219-02).
    let guards = merged_guards(&action.preconditions, transition_guard);
    for guard_name in &guards {
        let passes = (dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db)
            .await
            .map_err(|e| WriteError::GuardFailed(format!("{guard_name}: {e}")))?;
        if !passes {
            return Err(WriteError::GuardFailed(format!(
                "precondition '{guard_name}' not met"
            )));
        }
    }

    // 2. Idempotency check (D-04).
    //
    // Lookup is scoped by BOTH tenant_id AND idempotency_key to prevent
    // cross-tenant replay (T-219-01). Absent key = no guard (key is optional).
    //
    // Length cap: idempotency_key is not declared in ActionDef.inputs so
    // validate_action_inputs() never sees it. Reject keys longer than 128
    // characters to prevent unbounded TEXT storage (storage DoS surface).
    if let Some(key) = inputs.get("idempotency_key").and_then(|v| v.as_str()) {
        if key.len() > 128 {
            return Err(WriteError::Validation(
                "idempotency_key must not exceed 128 characters".into(),
            ));
        }
    }
    let idempotency_key = inputs.get("idempotency_key").and_then(|v| v.as_str());
    if let Some(key) = idempotency_key {
        if let Some(stored_result) = lookup_idempotency(tenant_id, key, db).await? {
            // Replay: return stored result without re-executing or re-auditing.
            // The original call was already audited; replaying does not add a new entry.
            return Ok(stored_result);
        }
    }

    // 3. D-08 SEAM (Phase 220): confirmation gate for destructive actions.
    //
    // When the `confirmation` feature is on, a bare call to a destructive action
    // returns Err(ConfirmationRequired) — the executor never fires. "Destructive"
    // means: a transition with transition_trigger (existing behavior) OR a CRUD
    // Delete verb (Phase 241 extension — D-06 / CRUD-03 / T-241-07).
    // The channel's confirm handler sets is_confirmed=true to bypass this seam
    // after token validation.
    //
    // Feature-off: fall through to executor (Phase 219 behavior preserved).
    #[cfg(feature = "confirmation")]
    {
        let is_destructive = action.transition_trigger.is_some()
            || matches!(crud_plan, Some(CrudPlan::Delete { .. }));
        if is_destructive && !is_confirmed {
            return Err(WriteError::ConfirmationRequired(action.name.clone()));
        }
    }
    #[cfg(not(feature = "confirmation"))]
    let _ = (&action.transition_trigger, crud_plan);

    // 4. Execute callback (D-01) or generic CRUD executor (D-04).
    //    When crud_plan is Some, the framework-provided execute_crud_plan interprets
    //    the CrudPlan into parameterized SQL — the app-registered executor is bypassed.
    //    When crud_plan is None, the registered ExecutorFn runs (transition path).
    //    The executor owns TenantScoped enforcement (D-03): find_for_tenant(id, tenant_id)
    //    returning None is the cross-tenant denial primitive.
    let result = if let Some(plan) = crud_plan {
        execute_crud_plan(plan, tenant_id, db).await?
    } else {
        (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?
    };

    // 5. Store idempotency result (D-04).
    //    INSERT OR IGNORE / ON CONFLICT DO NOTHING for concurrency safety.
    //
    //    WR-01: sealed BEFORE the override hook. There is no surrounding
    //    transaction, so the executor's base persist has already committed by
    //    this point. If the override (an app-specific side effect) ran first and
    //    failed, its `?` would short-circuit BEFORE the key is stored, leaving a
    //    committed base transition re-executable on retry (no stored key). Storing
    //    the key here guarantees a committed base persist is never re-executed,
    //    even when a later override fails.
    if let Some(key) = idempotency_key {
        store_idempotency(tenant_id, key, &result, db).await?;
    }

    // 6. Audit (D-05, SC#4) — record after every successful base persist.
    //    Denial audit (guard-failed path) is recorded in the channel framing.
    //
    //    WR-01: sealed BEFORE the override hook for the same reason. The base
    //    persist committed at step 4; the "every successful execution is audited"
    //    invariant must hold regardless of whether the app-specific override side
    //    effect later succeeds. An override failure does NOT roll back the base
    //    persist (no transaction), so the base persist is audited here
    //    unconditionally — the audit entry never overstates what persisted.
    let record_id = inputs.get("id").map(|v| v.to_string()).unwrap_or_default();
    // CRUD verbs use a distinct audit prefix (`{channel}.crud.{name}`) so audit
    // logs are queryable by verb class (D-08). Transitions keep `.action.` unchanged.
    let audit_action = if crud_plan.is_some() {
        format!("{channel}.crud.{}", &action.name)
    } else {
        format!("{channel}.action.{}", &action.name)
    };
    AuditEntry::record(audit_action)
        .tenant(tenant_id.to_string())
        .actor(AuditActor::User(tenant_id.to_string()))
        .target(AuditTarget::new(&action.name, record_id))
        .after(result.clone())
        .reason(&action.name)
        .write(db)
        .await
        .map_err(|e| WriteError::Database(e.to_string()))?;

    // 7. Post-persist override hook (EXEC-03).
    //    Runs AFTER the base persist is durable, idempotency-keyed, and audited
    //    (steps 4–6) — it cannot suppress the base guard or transition
    //    (T-231-05). An override `Err` propagates via `?` WITHOUT erasing the
    //    base persist's idempotency key or audit entry: those are already
    //    committed above. Absent key = common path (declaration-only); the
    //    override adds nothing.
    if let Some(hook) = dispatcher.overrides.get(&action.name) {
        (hook)(&action.name, inputs, tenant_id, db, &result).await?;
    }

    Ok(result)
}

// ── Kernel unit tests (relocated from ferro-mcp-server) ──────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::ActionDef;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// In-memory SQLite with the `mcp_idempotency_keys` and `audit_log` tables
    /// created via raw SQL.
    async fn setup_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant_id INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL,
                result TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE (tenant_id, idempotency_key)
            )"
            .to_string(),
        ))
        .await
        .expect("create mcp_idempotency_keys table");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY NOT NULL,
                tenant_id TEXT,
                actor_kind TEXT NOT NULL,
                actor_id TEXT,
                action TEXT NOT NULL,
                target_kind TEXT,
                target_id TEXT,
                before TEXT,
                after TEXT,
                reason TEXT,
                correlation_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
            .to_string(),
        ))
        .await
        .expect("create audit_log table");
        // Phase 241: orders table for CRUD executor dispatch tests.
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                status TEXT NOT NULL DEFAULT 'draft',
                amount TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                deleted_at TEXT
            )"
            .to_string(),
        ))
        .await
        .expect("create orders table");
        db
    }

    fn approve_action() -> ActionDef {
        ActionDef::new("approve")
            .transition_trigger("approve")
            .precondition("is_manager")
    }

    fn submit_action() -> ActionDef {
        ActionDef::new("submit").transition_trigger("submit")
    }

    fn update_action() -> ActionDef {
        ActionDef::new("update")
    }

    /// SC#1 (T-219-02): A guard evaluator returning `Ok(false)` must cause
    /// `dispatch_write` to return `Err(GuardFailed(_))` WITHOUT invoking the
    /// executor.
    #[tokio::test]
    async fn guard_denied_at_call_time() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must not run when guard fails") })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &approve_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            None,
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::GuardFailed(_))),
            "expected Err(GuardFailed(_)), got: {result:?}"
        );
    }

    /// A guard_evaluator returning `Ok(false)` for a transition that carries a
    /// `Transition.guard` causes `dispatch_write` to return `Err(GuardFailed(_))`
    /// and the executor never runs.
    #[tokio::test]
    async fn guard_rejects_illegal_transition() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must not run when transition guard fails") })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            None,
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::GuardFailed(_))),
            "expected Err(GuardFailed(_)) from the transition guard, got: {result:?}"
        );
    }

    /// A transition-level guard present on the PLAN but absent from
    /// `action.preconditions` is still evaluated.
    #[tokio::test]
    async fn transition_guard_evaluated_at_call_time() {
        let db = setup_db().await;
        let saw_transition_guard = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new({
                let flag = saw_transition_guard.clone();
                move |name: &str, _, _, _| {
                    if name == "is_manager" {
                        flag.store(true, Ordering::SeqCst);
                    }
                    Box::pin(async { Ok(true) })
                }
            }),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
            None,
        )
        .await;

        assert!(
            result.is_ok(),
            "guard returns true, write must succeed: {result:?}"
        );
        assert!(
            saw_transition_guard.load(Ordering::SeqCst),
            "the transition-level guard 'is_manager' must be evaluated live"
        );
    }

    /// A guard name present on BOTH the transition AND the action precondition
    /// fires the evaluator exactly ONCE (deduped by name).
    #[tokio::test]
    async fn guard_deduped_when_on_both() {
        let db = setup_db().await;
        let call_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new({
                let count = call_count.clone();
                move |name: &str, _, _, _| {
                    if name == "is_manager" {
                        count.fetch_add(1, Ordering::SeqCst);
                    }
                    Box::pin(async { Ok(true) })
                }
            }),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "approved" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &approve_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
            None,
        )
        .await;

        assert!(
            result.is_ok(),
            "both guards pass; write must succeed: {result:?}"
        );
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "is_manager must be evaluated exactly once (deduped), not twice"
        );
    }

    /// An override registered for an action runs AFTER the base persist.
    #[tokio::test]
    async fn override_hook_runs_post_persist() {
        let db = setup_db().await;
        let order = Arc::new(std::sync::Mutex::new(Vec::<&'static str>::new()));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let order = order.clone();
                move |_, _, _, _| {
                    order.lock().unwrap().push("persist");
                    Box::pin(async { Ok(json!({ "status": "submitted" })) })
                }
            }),
            overrides: std::collections::HashMap::new(),
        }
        .with_override(
            "submit",
            Box::new({
                let order = order.clone();
                move |_action, _inputs, _tenant, _db, base_result: &Value| {
                    assert_eq!(base_result["status"], "submitted");
                    order.lock().unwrap().push("override");
                    Box::pin(async { Ok(()) })
                }
            }),
        );

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
            None,
        )
        .await;

        assert!(result.is_ok(), "write must succeed: {result:?}");
        assert_eq!(
            result.unwrap(),
            json!({ "status": "submitted" }),
            "base result must be returned unchanged by the override"
        );
        assert_eq!(
            *order.lock().unwrap(),
            vec!["persist", "override"],
            "override must run AFTER the base persist"
        );
    }

    /// With NO override registered, `dispatch_write` behaves exactly as before.
    #[tokio::test]
    async fn no_override_is_declaration_only() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
            None,
        )
        .await;

        assert!(result.is_ok(), "no-override path must succeed: {result:?}");
        assert_eq!(result.unwrap(), json!({ "status": "submitted" }));
    }

    /// An override returning `Err` causes `dispatch_write` to return `Err`, while
    /// the base persist's idempotency key AND audit entry remain sealed (WR-01).
    #[tokio::test]
    async fn override_error_surfaces() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        }
        .with_override(
            "submit",
            Box::new(|_action, _inputs, _tenant, _db, _base| {
                Box::pin(async { Err(WriteError::Validation("override failed".into())) })
            }),
        );

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1, "idempotency_key": "ovr-fail-1"}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
            None,
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::Validation(ref m)) if m == "override failed"),
            "override error must propagate, got: {result:?}"
        );

        // WR-01: the base persist's audit entry must exist despite the override
        // failure — and it must carry the channel-prefixed action `mcp.action.submit`.
        let audit_count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM audit_log WHERE action = 'mcp.action.submit'"
                    .to_string(),
            ))
            .await
            .expect("audit query must succeed")
            .expect("audit count row must exist")
            .try_get::<i64>("", "c")
            .expect("audit count column");
        assert_eq!(
            audit_count, 1,
            "base persist must be audited even when the override fails (WR-01)"
        );

        // WR-01: the idempotency key must be stored despite the override failure.
        let stored = lookup_idempotency(1, "ovr-fail-1", &db)
            .await
            .expect("idempotency lookup must succeed");
        assert_eq!(
            stored,
            Some(json!({ "status": "submitted" })),
            "base persist's idempotency key must be stored even when the override fails (WR-01)"
        );
    }

    /// SC#3 (T-219-03): Two identical `dispatch_write` calls with the same
    /// `idempotency_key` produce equal results and fire the executor once.
    #[tokio::test]
    async fn idempotent_replay_does_not_re_execute() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(json!({ "status": "submitted" })) })
                }
            }),
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            overrides: std::collections::HashMap::new(),
        };

        let args = json!({ "id": 1, "idempotency_key": "k-abc" });

        let result1 = dispatch_write(
            &update_action(),
            &args,
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            None,
        )
        .await;
        let result2 = dispatch_write(
            &update_action(),
            &args,
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            None,
        )
        .await;

        assert!(result1.is_ok(), "first call must succeed; got: {result1:?}");
        assert!(
            result2.is_ok(),
            "second call must succeed (replay); got: {result2:?}"
        );
        assert_eq!(
            result1.unwrap(),
            result2.unwrap(),
            "idempotent replay must return the same result"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            1,
            "executor must fire exactly once despite two identical calls"
        );
    }

    /// The audit reason prefix is built from the `channel` argument: passing a
    /// different channel writes `web.action.submit`, not `mcp.action.submit`.
    #[tokio::test]
    async fn audit_channel_is_parameterized() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "web",
            #[cfg(feature = "confirmation")]
            true,
            None,
        )
        .await;
        assert!(result.is_ok(), "write must succeed: {result:?}");

        let web_count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM audit_log WHERE action = 'web.action.submit'"
                    .to_string(),
            ))
            .await
            .expect("audit query must succeed")
            .expect("audit count row must exist")
            .try_get::<i64>("", "c")
            .expect("audit count column");
        assert_eq!(
            web_count, 1,
            "audit action must be prefixed by the channel argument ('web')"
        );

        let mcp_count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM audit_log WHERE action = 'mcp.action.submit'"
                    .to_string(),
            ))
            .await
            .expect("audit query must succeed")
            .expect("audit count row must exist")
            .try_get::<i64>("", "c")
            .expect("audit count column");
        assert_eq!(
            mcp_count, 0,
            "no mcp-channel audit must be written when channel is 'web'"
        );
    }

    // ── CRUD dispatch tests (VALIDATION rows #6–#13) ──────────────────────────

    /// Helpers shared across CRUD tests.
    fn crud_action(name: &str) -> ActionDef {
        ActionDef::new(name)
    }

    fn allow_dispatcher() -> WriteDispatcher {
        WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            // The executor is bypassed on the CRUD path (crud_plan=Some).
            // A panic here would mean the CRUD branch is not working.
            executor: Box::new(|_, _, _, _| {
                Box::pin(async {
                    panic!("executor must NOT run on the CRUD path (crud_plan=Some)")
                })
            }),
            overrides: std::collections::HashMap::new(),
        }
    }

    fn create_plan(columns: Vec<(&str, serde_json::Value)>) -> CrudPlan {
        CrudPlan::Create {
            table: "orders".to_string(),
            columns: columns
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            tenant_column: None,
        }
    }

    fn update_plan(id: i64, patch: Vec<(&str, serde_json::Value)>) -> CrudPlan {
        CrudPlan::Update {
            table: "orders".to_string(),
            id_column: "id".to_string(),
            id_value: serde_json::json!(id),
            patch: patch.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            soft_delete_column: "deleted_at".to_string(),
            tenant_column: None,
        }
    }

    fn delete_plan(id: i64) -> CrudPlan {
        CrudPlan::Delete {
            table: "orders".to_string(),
            id_column: "id".to_string(),
            id_value: serde_json::json!(id),
            soft_delete_column: "deleted_at".to_string(),
            tenant_column: None,
        }
    }

    /// VALIDATION #6 / SC#1: CREATE inserts a row; returned payload contains `id`.
    #[tokio::test]
    async fn crud_create_inserts_row() {
        let db = setup_db().await;
        let plan = create_plan(vec![("status", json!("draft")), ("amount", json!("99.00"))]);
        let result = dispatch_write(
            &crud_action("create_order"),
            &json!({}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&plan),
        )
        .await
        .expect("crud create must succeed");

        assert!(
            result.get("id").is_some(),
            "returned record must have an id: {result:?}"
        );

        let count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM orders".to_string(),
            ))
            .await
            .expect("count query")
            .expect("count row")
            .try_get::<i64>("", "c")
            .expect("count column");
        assert_eq!(count, 1, "exactly one row must be in orders after create");
    }

    /// VALIDATION #7 / SC#2: UPDATE patches only supplied fields on a non-deleted row.
    #[tokio::test]
    async fn crud_update_patches_row() {
        let db = setup_db().await;

        // Pre-insert a row.
        let create = create_plan(vec![("status", json!("draft")), ("amount", json!("10.00"))]);
        dispatch_write(
            &crud_action("create_order"),
            &json!({}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&create),
        )
        .await
        .expect("pre-insert must succeed");

        // Update only `amount`.
        let upd = update_plan(1, vec![("amount", json!("55.00"))]);
        let result = dispatch_write(
            &crud_action("update_order"),
            &json!({"id": 1}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&upd),
        )
        .await
        .expect("crud update must succeed");

        // Returned record must reflect the patched amount.
        assert_eq!(
            result["amount"],
            json!("55.00"),
            "amount must be updated: {result:?}"
        );
        // Status must be unchanged.
        assert_eq!(
            result["status"],
            json!("draft"),
            "status must remain draft: {result:?}"
        );
    }

    /// VALIDATION #8 / SC#2 + CRUD-03: UPDATE on a soft-deleted row → RecordNotFound.
    #[tokio::test]
    async fn crud_update_soft_deleted_not_found() {
        let db = setup_db().await;

        // Pre-insert then soft-delete.
        let create = create_plan(vec![("status", json!("draft"))]);
        dispatch_write(
            &crud_action("create_order"),
            &json!({}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&create),
        )
        .await
        .expect("pre-insert must succeed");

        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "UPDATE orders SET deleted_at = datetime('now') WHERE id = 1".to_string(),
        ))
        .await
        .expect("manual soft-delete must succeed");

        // Update must fail with RecordNotFound because deleted_at IS NOT NULL.
        let upd = update_plan(1, vec![("status", json!("submitted"))]);
        let result = dispatch_write(
            &crud_action("update_order"),
            &json!({"id": 1}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&upd),
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::RecordNotFound)),
            "update on soft-deleted row must return RecordNotFound, got: {result:?}"
        );
    }

    /// VALIDATION #9 / SC#2 + CRUD-03: DELETE sets deleted_at; row still physically present.
    #[tokio::test]
    async fn crud_delete_sets_deleted_at() {
        let db = setup_db().await;

        // Pre-insert.
        let create = create_plan(vec![("status", json!("draft"))]);
        dispatch_write(
            &crud_action("create_order"),
            &json!({}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&create),
        )
        .await
        .expect("pre-insert must succeed");

        // Confirmed delete.
        let del = delete_plan(1);
        let result = dispatch_write(
            &crud_action("delete_order"),
            &json!({"id": 1}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
            Some(&del),
        )
        .await
        .expect("confirmed delete must succeed");

        assert_eq!(
            result["deleted"],
            json!(true),
            "result must carry deleted:true: {result:?}"
        );

        // Row must still be physically present.
        let count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM orders WHERE id = 1".to_string(),
            ))
            .await
            .expect("count query")
            .expect("count row")
            .try_get::<i64>("", "c")
            .expect("count column");
        assert_eq!(
            count, 1,
            "row must still exist physically after soft-delete"
        );

        // deleted_at must be set.
        let deleted_at: Option<String> = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT deleted_at FROM orders WHERE id = 1".to_string(),
            ))
            .await
            .expect("deleted_at query")
            .expect("deleted_at row")
            .try_get::<Option<String>>("", "deleted_at")
            .expect("deleted_at column");
        assert!(
            deleted_at.is_some(),
            "deleted_at must be set after soft-delete"
        );
    }

    /// VALIDATION #10 / CRUD-03: After soft-delete, `deleted_at IS NULL` filter hides the row.
    #[tokio::test]
    async fn crud_deleted_row_hidden_from_list() {
        let db = setup_db().await;

        let create = create_plan(vec![("status", json!("draft"))]);
        dispatch_write(
            &crud_action("create_order"),
            &json!({}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&create),
        )
        .await
        .expect("pre-insert must succeed");

        let del = delete_plan(1);
        dispatch_write(
            &crud_action("delete_order"),
            &json!({"id": 1}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
            Some(&del),
        )
        .await
        .expect("confirmed delete must succeed");

        // The list predicate (`deleted_at IS NULL`) must hide this row.
        let hidden_count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM orders WHERE id = 1 AND deleted_at IS NULL".to_string(),
            ))
            .await
            .expect("hidden count query")
            .expect("hidden count row")
            .try_get::<i64>("", "c")
            .expect("hidden count column");
        assert_eq!(
            hidden_count, 0,
            "soft-deleted row must be hidden by deleted_at IS NULL filter"
        );
    }

    /// VALIDATION #11 / CRUD-03: Bare DELETE without confirmation → ConfirmationRequired.
    #[cfg(feature = "confirmation")]
    #[tokio::test]
    async fn crud_delete_requires_confirmation() {
        let db = setup_db().await;

        // Pre-insert so the row exists (confirmation seam fires before executor).
        let create = create_plan(vec![("status", json!("draft"))]);
        dispatch_write(
            &crud_action("create_order"),
            &json!({}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            false,
            Some(&create),
        )
        .await
        .expect("pre-insert must succeed");

        let del = delete_plan(1);
        let result = dispatch_write(
            &crud_action("delete_order"),
            &json!({"id": 1}),
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            false, // is_confirmed = false → seam must fire
            Some(&del),
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::ConfirmationRequired(_))),
            "bare delete without confirmation must return ConfirmationRequired, got: {result:?}"
        );
    }

    /// VALIDATION #12 / SC#3: Override hook runs after generic CRUD create; row still inserted.
    #[tokio::test]
    async fn crud_override_replaces_generic() {
        let db = setup_db().await;
        let hook_fired = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must NOT run on the CRUD path") })
            }),
            overrides: std::collections::HashMap::new(),
        }
        .with_override(
            "create_order",
            Box::new({
                let flag = hook_fired.clone();
                move |_action, _inputs, _tenant, _db, base_result: &Value| {
                    // Override fires with the generic-created result.
                    assert!(
                        base_result.get("id").is_some(),
                        "override receives the inserted record with id: {base_result:?}"
                    );
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    Box::pin(async { Ok(()) })
                }
            }),
        );

        let plan = create_plan(vec![("status", json!("draft"))]);
        let result = dispatch_write(
            &crud_action("create_order"),
            &json!({}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&plan),
        )
        .await
        .expect("create with override must succeed");

        assert!(
            result.get("id").is_some(),
            "result must have id: {result:?}"
        );
        assert!(
            hook_fired.load(std::sync::atomic::Ordering::SeqCst),
            "override hook must have fired"
        );

        // The generic insert must have run: row exists in DB.
        let count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM orders".to_string(),
            ))
            .await
            .expect("count query")
            .expect("count row")
            .try_get::<i64>("", "c")
            .expect("count column");
        assert_eq!(
            count, 1,
            "generic insert must have run alongside the override hook"
        );
    }

    /// VALIDATION #13 / CRUD-06: Second create with the same idempotency_key returns
    /// the stored result; the DB has exactly one row.
    #[tokio::test]
    async fn crud_create_idempotent() {
        let db = setup_db().await;
        let plan = create_plan(vec![("status", json!("draft"))]);

        let inputs = json!({ "idempotency_key": "idem-create-1" });

        let result1 = dispatch_write(
            &crud_action("create_order"),
            &inputs,
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&plan),
        )
        .await
        .expect("first create must succeed");

        let result2 = dispatch_write(
            &crud_action("create_order"),
            &inputs,
            1,
            &db,
            &allow_dispatcher(),
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
            Some(&plan),
        )
        .await
        .expect("second create (idempotent replay) must succeed");

        assert_eq!(
            result1, result2,
            "idempotent replay must return the same result"
        );

        let count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM orders".to_string(),
            ))
            .await
            .expect("count query")
            .expect("count row")
            .try_get::<i64>("", "c")
            .expect("count column");
        assert_eq!(count, 1, "idempotent replay must not insert a second row");
    }

    // ── Phase 242 tenant injection tests (CRUD-05 / T-242-02 / T-242-03) ──────

    /// In-memory SQLite with a `tenanted_orders` table that includes a `tenant_id`
    /// column, plus the standard idempotency and audit tables.
    async fn setup_tenant_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant_id INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL,
                result TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE (tenant_id, idempotency_key)
            )"
            .to_string(),
        ))
        .await
        .expect("create mcp_idempotency_keys table");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY NOT NULL,
                tenant_id TEXT,
                actor_kind TEXT NOT NULL,
                actor_id TEXT,
                action TEXT NOT NULL,
                target_kind TEXT,
                target_id TEXT,
                before TEXT,
                after TEXT,
                reason TEXT,
                correlation_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
            .to_string(),
        ))
        .await
        .expect("create audit_log table");
        // tenanted_orders: same shape as orders plus tenant_id column for Phase 242 tests.
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS tenanted_orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                amount TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                deleted_at TEXT
            )"
            .to_string(),
        ))
        .await
        .expect("create tenanted_orders table");
        db
    }

    fn tenanted_create_plan(columns: Vec<(&str, serde_json::Value)>) -> CrudPlan {
        use ferro_projections::TenantColumn;
        CrudPlan::Create {
            table: "tenanted_orders".to_string(),
            columns: columns
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            tenant_column: Some(TenantColumn {
                column: "tenant_id".to_string(),
            }),
        }
    }

    fn tenanted_update_plan(id: i64, patch: Vec<(&str, serde_json::Value)>) -> CrudPlan {
        use ferro_projections::TenantColumn;
        CrudPlan::Update {
            table: "tenanted_orders".to_string(),
            id_column: "id".to_string(),
            id_value: serde_json::json!(id),
            patch: patch.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            soft_delete_column: "deleted_at".to_string(),
            tenant_column: Some(TenantColumn {
                column: "tenant_id".to_string(),
            }),
        }
    }

    fn tenanted_delete_plan(id: i64) -> CrudPlan {
        use ferro_projections::TenantColumn;
        CrudPlan::Delete {
            table: "tenanted_orders".to_string(),
            id_column: "id".to_string(),
            id_value: serde_json::json!(id),
            soft_delete_column: "deleted_at".to_string(),
            tenant_column: Some(TenantColumn {
                column: "tenant_id".to_string(),
            }),
        }
    }

    /// Seed a row in `tenanted_orders` with the given tenant_id and status.
    /// Returns the inserted row id.
    async fn seed_tenanted_row(
        db: &sea_orm::DatabaseConnection,
        tenant_id: i64,
        status: &str,
    ) -> i64 {
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO tenanted_orders (tenant_id, status) VALUES (?, ?)",
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(status.to_string()))),
            ],
        ))
        .await
        .expect("seed tenanted row");

        let id_row = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT last_insert_rowid() AS id".to_string(),
            ))
            .await
            .expect("last_insert_rowid query")
            .expect("last_insert_rowid row");
        id_row.try_get::<i64>("", "id").expect("last_insert_rowid")
    }

    /// CRUD-05 / T-242-02: CREATE with tenant_column Some + tenant_id=7 →
    /// the inserted row's tenant_id column equals 7.
    #[tokio::test]
    async fn crud_create_injects_tenant() {
        let db = setup_tenant_db().await;
        let plan = tenanted_create_plan(vec![("status", json!("draft"))]);

        execute_crud_plan(&plan, 7, &db)
            .await
            .expect("tenant-aware create must succeed");

        // Verify the row's tenant_id was set to 7 (injected, not from agent input).
        let tid: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT tenant_id FROM tenanted_orders WHERE id = 1".to_string(),
            ))
            .await
            .expect("select query")
            .expect("row must exist")
            .try_get::<i64>("", "tenant_id")
            .expect("tenant_id column");
        assert_eq!(
            tid, 7,
            "CREATE must inject tenant_id=7 into the row, got: {tid}"
        );
    }

    /// CRUD-05 / T-242-02: Same-tenant UPDATE succeeds (1 row affected, row mutated).
    #[tokio::test]
    async fn crud_update_tenant_predicate() {
        let db = setup_tenant_db().await;
        let row_id = seed_tenanted_row(&db, 7, "draft").await;

        let plan = tenanted_update_plan(row_id, vec![("status", json!("approved"))]);
        let result = execute_crud_plan(&plan, 7, &db)
            .await
            .expect("same-tenant update must succeed");

        assert_eq!(
            result["status"],
            json!("approved"),
            "status must be updated for same-tenant row: {result:?}"
        );
    }

    /// CRUD-05 / T-242-02: Same-tenant DELETE succeeds (deleted_at set).
    #[tokio::test]
    async fn crud_delete_tenant_predicate() {
        let db = setup_tenant_db().await;
        let row_id = seed_tenanted_row(&db, 7, "draft").await;

        let plan = tenanted_delete_plan(row_id);
        let result = execute_crud_plan(&plan, 7, &db)
            .await
            .expect("same-tenant delete must succeed");

        assert_eq!(
            result["deleted"],
            json!(true),
            "same-tenant delete must return deleted:true: {result:?}"
        );

        // deleted_at must be set on the row.
        let deleted_at: Option<String> = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT deleted_at FROM tenanted_orders WHERE id = {row_id}"),
            ))
            .await
            .expect("deleted_at query")
            .expect("row")
            .try_get::<Option<String>>("", "deleted_at")
            .expect("deleted_at column");
        assert!(
            deleted_at.is_some(),
            "deleted_at must be set after same-tenant delete"
        );
    }

    /// CRUD-05 / T-242-03: Cross-tenant UPDATE → RecordNotFound; row is UNCHANGED.
    ///
    /// Non-disclosure: a row owned by tenant 2 is unaddressable to tenant 7 —
    /// the error is indistinguishable from a missing row (D-08).
    #[tokio::test]
    async fn crud_cross_tenant_update_not_found() {
        let db = setup_tenant_db().await;
        // Seed a row owned by tenant 2.
        let row_id = seed_tenanted_row(&db, 2, "original").await;

        // Attempt to update it as tenant 7.
        let plan = tenanted_update_plan(row_id, vec![("status", json!("tampered"))]);
        let result = execute_crud_plan(&plan, 7, &db).await;

        assert!(
            matches!(result, Err(WriteError::RecordNotFound)),
            "cross-tenant update must return RecordNotFound, got: {result:?}"
        );

        // Verify the row is UNCHANGED (non-disclosure: no partial write or leakage).
        let status: String = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT status FROM tenanted_orders WHERE id = {row_id}"),
            ))
            .await
            .expect("status query")
            .expect("row must still exist")
            .try_get::<String>("", "status")
            .expect("status column");
        assert_eq!(
            status, "original",
            "cross-tenant update must leave the row completely unchanged; got status: {status}"
        );
    }

    /// CRUD-05 / T-242-03: Cross-tenant DELETE → RecordNotFound; deleted_at stays NULL.
    ///
    /// Non-disclosure: a row owned by tenant 2 is unaddressable to tenant 7 —
    /// the error is indistinguishable from a missing row (D-08).
    #[tokio::test]
    async fn crud_cross_tenant_delete_not_found() {
        let db = setup_tenant_db().await;
        // Seed a row owned by tenant 2.
        let row_id = seed_tenanted_row(&db, 2, "draft").await;

        // Attempt to delete it as tenant 7.
        let plan = tenanted_delete_plan(row_id);
        let result = execute_crud_plan(&plan, 7, &db).await;

        assert!(
            matches!(result, Err(WriteError::RecordNotFound)),
            "cross-tenant delete must return RecordNotFound, got: {result:?}"
        );

        // Verify deleted_at is still NULL (no partial write or leakage).
        let deleted_at: Option<String> = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                format!("SELECT deleted_at FROM tenanted_orders WHERE id = {row_id}"),
            ))
            .await
            .expect("deleted_at query")
            .expect("row must still exist")
            .try_get::<Option<String>>("", "deleted_at")
            .expect("deleted_at column");
        assert!(
            deleted_at.is_none(),
            "cross-tenant delete must leave deleted_at NULL; row is untouched"
        );
    }
}
