use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::Value;

use ferro_projections::{schema_contract, SchemaContract, ServiceDef};

use crate::http::Request;
use crate::inertia::context::Inertia;
use crate::permitted_actions::permitted_actions;
use crate::projection_read::{dispatch, DispatchResult};
use crate::Response;

/// Query parameters for a projection-driven Inertia page.
///
/// Default: `filters = {}`, `limit = 25`, `offset = 0`
/// — matches the MCP data-surface defaults so the same `ServiceDef`
/// drives both surfaces identically without additional configuration.
#[derive(Debug, Clone)]
pub struct ProjectionQuery {
    /// Equality and range filters forwarded to [`dispatch`].
    pub filters: Value,
    /// Maximum rows to return. Clamped server-side to `MAX_LIMIT = 100`.
    pub limit: u64,
    /// Row offset for pagination.
    pub offset: u64,
}

impl Default for ProjectionQuery {
    fn default() -> Self {
        Self {
            filters: Value::Object(Default::default()),
            limit: 25,
            offset: 0,
        }
    }
}

impl ProjectionQuery {
    /// Override the filter map (consuming builder).
    pub fn filters(mut self, f: Value) -> Self {
        self.filters = f;
        self
    }

    /// Override the page size (consuming builder). Clamped to `MAX_LIMIT = 100`.
    pub fn limit(mut self, n: u64) -> Self {
        self.limit = n;
        self
    }

    /// Override the row offset (consuming builder).
    pub fn offset(mut self, n: u64) -> Self {
        self.offset = n;
        self
    }
}

/// Serialized props delivered to the Inertia component.
///
/// All six keys are always present so the component can rely on a stable shape.
#[derive(Debug, Serialize)]
struct ProjectionProps {
    schema: SchemaContract,
    data: Vec<Value>,
    permitted_actions: Vec<String>,
    total: u64,
    limit: u64,
    offset: u64,
}

impl Inertia {
    /// Render an Inertia response from a projection declaration.
    ///
    /// Assembles `{ schema, data, permitted_actions, total, limit, offset }` props
    /// by combining three derivation cores:
    /// - [`schema_contract`] — pure schema derivation from `service` (no runtime deps).
    /// - [`permitted_actions`] — visibility filter over `evaluated_guards`.
    /// - [`dispatch`] — tenant-scoped SQL read from `service`'s declared field set.
    ///
    /// On a data-query error the method returns a rendered Inertia error page
    /// (props `{ "error": "<message>" }`) — never a panic.
    ///
    /// `permitted_actions` in props is **advisory display data only**, not an
    /// authorization grant. Per-record guard enforcement happens at `dispatch_write`
    /// time via the live `GuardEvaluatorFn`. This helper adds READ derivation only
    /// and changes no write path.
    ///
    /// # Arguments
    ///
    /// - `req` — the current request (used to detect Inertia XHR vs full-page load).
    /// - `component` — the Inertia component name to render.
    /// - `service` — the `ServiceDef` declaration to project from.
    /// - `query` — filter/limit/offset parameters; defaults are 25 rows, no filters.
    /// - `db` — database connection for the tenant-scoped row read.
    /// - `tenant_id` — must be `Some` when `service.tenant_column` is set; the value
    ///   is never sourced from the request body — callers pass `current_tenant().map(|t| t.id)`.
    /// - `evaluated_guards` — pre-computed guard map (`absent key = allow`,
    ///   `Some(false) = deny`). Build this the same way as for MCP `tools/list`.
    pub async fn from_projection(
        req: &Request,
        component: &str,
        service: &ServiceDef,
        query: ProjectionQuery,
        db: &DatabaseConnection,
        tenant_id: Option<i64>,
        evaluated_guards: &HashMap<String, bool>,
    ) -> Response {
        let schema = schema_contract(service);
        let actions = permitted_actions(service, evaluated_guards);

        let result: DispatchResult = match dispatch(
            service,
            query.filters,
            query.limit,
            query.offset,
            db,
            tenant_id,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                return Inertia::render(
                    req,
                    component,
                    serde_json::json!({ "error": e.to_string() }),
                );
            }
        };

        let props = ProjectionProps {
            schema,
            data: result.rows,
            permitted_actions: actions,
            total: result.total,
            limit: result.limit,
            offset: result.offset,
        };

        Inertia::render(req, component, props)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef};
    use serde_json::json;

    /// ProjectionQuery::default() produces the documented default values.
    #[test]
    fn projection_query_default_values() {
        let q = ProjectionQuery::default();
        assert_eq!(q.limit, 25);
        assert_eq!(q.offset, 0);
        assert_eq!(q.filters, json!({}));
    }

    /// Builder methods are consuming and return a modified copy.
    #[test]
    fn projection_query_builder_methods() {
        let q = ProjectionQuery::default()
            .limit(50)
            .offset(10)
            .filters(json!({"status": "active"}));
        assert_eq!(q.limit, 50);
        assert_eq!(q.offset, 10);
        assert_eq!(q.filters, json!({"status": "active"}));
    }

    /// ProjectionProps serializes to an object with exactly the six expected keys.
    #[test]
    fn projection_props_serializes_six_keys() {
        use ferro_projections::schema_contract;

        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"))
            .action(ActionDef::new("submit"));

        let schema = schema_contract(&service);

        let props = ProjectionProps {
            schema,
            data: vec![json!({"id": 1})],
            permitted_actions: vec!["submit".to_string()],
            total: 1,
            limit: 25,
            offset: 0,
        };

        let value = serde_json::to_value(&props).expect("serialize ok");
        let obj = value.as_object().expect("is object");

        for key in &[
            "schema",
            "data",
            "permitted_actions",
            "total",
            "limit",
            "offset",
        ] {
            assert!(obj.contains_key(*key), "missing key: {key}");
        }
        assert_eq!(obj.len(), 6, "exactly six keys");
    }

    /// permitted_actions in props excludes an action whose guard is Some(false).
    #[test]
    fn permitted_actions_excludes_denied_guard() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("is_manager"))
            .action(ActionDef::new("approve").precondition("is_manager"))
            .action(ActionDef::new("submit"));

        let guards: HashMap<String, bool> =
            [("is_manager".to_string(), false)].into_iter().collect();

        let allowed = permitted_actions(&service, &guards);
        assert!(!allowed.contains(&"approve".to_string()));
        assert!(allowed.contains(&"submit".to_string()));
    }
}
